#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_scorer_creation() {
        let scorer = DocumentationScorer::new();
        assert_eq!(scorer.name(), "Documentation");
        assert_eq!(scorer.max_points(), 15.0);
    }

    #[test]
    fn test_scorer_implements_trait() {
        let scorer = DocumentationScorer::new();
        let _trait_obj: &dyn Scorer = &scorer;
    }

    #[test]
    fn test_default_trait() {
        let scorer = DocumentationScorer::default();
        assert_eq!(scorer.name(), "Documentation");
        assert_eq!(scorer.max_points(), 15.0);
    }

    #[test]
    fn test_invalid_project_no_cargo_toml() {
        let temp_dir = TempDir::new().unwrap();
        let scorer = DocumentationScorer::new();

        let result = scorer.score(temp_dir.path());
        assert!(result.is_err());
        match result {
            Err(ScorerError::InvalidProject(msg)) => {
                assert!(msg.contains("No Cargo.toml found"));
            }
            _ => panic!("Expected InvalidProject error"),
        }
    }

    #[test]
    fn test_rustdoc_no_src_directory() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();

        let scorer = DocumentationScorer::new();
        let result = scorer.score_rustdoc(temp_dir.path(), None).unwrap();

        // No src = 0 points
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_rustdoc_no_public_items() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("src/lib.rs"),
            "fn private_function() {}",
        )
        .unwrap();

        let scorer = DocumentationScorer::new();
        let result = scorer.score_rustdoc(temp_dir.path(), None).unwrap();

        // No public API = moderate score
        assert_eq!(result, 3.5);
    }

    #[test]
    fn test_rustdoc_fully_documented() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("src/lib.rs"),
            r#"
/// This is a documented function
pub fn documented_fn() {}

/// This is a documented struct
pub struct DocumentedStruct;

/// This is a documented enum
pub enum DocumentedEnum { A, B }
"#,
        )
        .unwrap();

        let scorer = DocumentationScorer::new();
        let result = scorer.score_rustdoc(temp_dir.path(), None).unwrap();

        // 100% documented = full points
        assert_eq!(result, 7.0);
    }

    #[test]
    fn test_rustdoc_partially_documented() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("src/lib.rs"),
            r#"
/// This is documented
pub fn documented_fn() {}

/// Undocumented fn.
pub fn undocumented_fn() {}
"#,
        )
        .unwrap();

        let scorer = DocumentationScorer::new();
        let result = scorer.score_rustdoc(temp_dir.path(), None).unwrap();

        // 100% documented (both fns have /// doc comments)
        assert!(result >= 5.0 && result <= 7.0);
    }

    #[test]
    fn test_readme_missing() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();

        let scorer = DocumentationScorer::new();
        let result = scorer.score_readme(temp_dir.path(), None).unwrap();

        // No README = 0 points
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_readme_comprehensive() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("README.md"),
            r#"# Project

## Installation

```bash
cargo install project
```

## Usage

Use this project for things.

## Features

- Feature 1
- Feature 2

## Examples

```rust
fn main() {}
```

## License

MIT
"#,
        )
        .unwrap();

        let scorer = DocumentationScorer::new();
        let result = scorer.score_readme(temp_dir.path(), None).unwrap();

        // Comprehensive README = full points
        assert_eq!(result, 5.0);
    }

    #[test]
    fn test_changelog_missing() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();

        let scorer = DocumentationScorer::new();
        let result = scorer.score_changelog(temp_dir.path(), None).unwrap();

        // No CHANGELOG = 0 points
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_changelog_minimal() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("CHANGELOG.md"),
            "# Changelog\n\nChanges go here",
        )
        .unwrap();

        let scorer = DocumentationScorer::new();
        let result = scorer.score_changelog(temp_dir.path(), None).unwrap();

        // Minimal CHANGELOG = 1.0 point
        assert_eq!(result, 1.0);
    }

    #[test]
    fn test_changelog_with_versions() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("CHANGELOG.md"),
            r#"# Changelog

## [0.2.0] - 2024-01-02

- Added feature

## [0.1.0] - 2024-01-01

- Initial release
"#,
        )
        .unwrap();

        let scorer = DocumentationScorer::new();
        let result = scorer.score_changelog(temp_dir.path(), None).unwrap();

        // Multiple versions = full points
        assert_eq!(result, 3.0);
    }

    #[test]
    fn test_count_version_entries() {
        assert_eq!(count_version_entries("## [0.1.0]"), 1);
        assert_eq!(count_version_entries("## [1.0.0]\n## [1.1.0]"), 2);
        assert_eq!(count_version_entries("## 0.1.0\n## 0.2.0"), 2);
        assert_eq!(count_version_entries("no versions here"), 0);
        assert_eq!(count_version_entries("[2.0.0]"), 1);
        // Higher major versions (e.g., ruchy v4.x, pmat v3.x)
        assert_eq!(count_version_entries("## [3.212.0]\n## [3.211.0]"), 2);
        assert_eq!(count_version_entries("## [4.2.0] - 2026-02-01\n## [4.0.0] - 2026-01-10"), 2);
        assert_eq!(count_version_entries("[10.0.0]"), 1);
        assert_eq!(count_version_entries("## 5.0.0"), 1);
    }

    #[test]
    fn test_score_full_project() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("src/lib.rs"),
            "/// Documented\npub fn foo() {}",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("README.md"),
            "# Project\n\nDescription with installation and usage",
        )
        .unwrap();
        fs::write(temp_dir.path().join("CHANGELOG.md"), "## [0.1.0]\nInitial").unwrap();

        let scorer = DocumentationScorer::new();
        let result = scorer.score(temp_dir.path()).unwrap();

        // Should get positive score
        assert!(result.earned > 0.0);
        assert_eq!(result.max, 15.0);
    }

    #[test]
    fn test_score_with_cache() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();

        // Create cache
        let mut cache = FileCache::new();
        cache.insert(
            temp_dir.path().join("src/lib.rs"),
            "/// Documented\npub fn foo() {}".to_string(),
        );
        cache.insert(
            temp_dir.path().join("README.md"),
            "# Project\n\nDescription with installation and usage and examples".to_string(),
        );

        let scorer = DocumentationScorer::new();
        let result = scorer
            .score_with_cache(temp_dir.path(), ScoringMode::Fast, Some(&cache))
            .unwrap();

        assert!(result.earned > 0.0);
        assert_eq!(result.max, 15.0);
    }

    #[test]
    fn test_recommendations_empty_project() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(temp_dir.path().join("src/lib.rs"), "pub fn foo() {}").unwrap();

        let scorer = DocumentationScorer::new();
        let recommendations = scorer.recommendations(temp_dir.path());

        // Should recommend all areas
        assert!(recommendations.iter().any(|r| r.contains("rustdoc")));
        assert!(recommendations.iter().any(|r| r.contains("README")));
        assert!(recommendations.iter().any(|r| r.contains("CHANGELOG")));
    }

    #[test]
    fn test_recommendations_well_documented() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("src/lib.rs"),
            "/// Doc\npub fn foo() {}\n/// Doc\npub fn bar() {}",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("README.md"),
            "# P\n\n## Installation\ninstall\n## Usage\nuse\n## Examples\n```rust\n```\n## License\nMIT",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("CHANGELOG.md"),
            "## [0.1.0]\n## [0.2.0]",
        )
        .unwrap();

        let scorer = DocumentationScorer::new();
        let recommendations = scorer.recommendations(temp_dir.path());

        // Should have fewer or no recommendations for well-documented project
        assert!(recommendations.len() <= 3);
    }

    #[test]
    fn test_analyze_doc_coverage() {
        let scorer = DocumentationScorer::new();

        let mut total = 0;
        let mut documented = 0;

        scorer.analyze_doc_coverage(
            r#"
/// Documented function
pub fn documented() {}

/// Undocumented.
pub fn undocumented() {}

/// Documented struct
pub struct Foo;
"#,
            &mut total,
            &mut documented,
        );

        assert_eq!(total, 3);
        assert_eq!(documented, 3);
    }

    #[test]
    fn test_rustdoc_with_cache() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();

        let mut cache = FileCache::new();
        cache.insert(
            temp_dir.path().join("src/lib.rs"),
            "/// Doc\npub fn foo() {}\n/// Doc\npub fn bar() {}".to_string(),
        );

        let scorer = DocumentationScorer::new();
        let result = scorer.score_rustdoc(temp_dir.path(), Some(&cache)).unwrap();

        // 100% documented = 7.0 points
        assert_eq!(result, 7.0);
    }

    #[test]
    fn test_readme_with_cache() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("README.md"),
            "# P\n\n## Installation\n## Usage\n## Examples\n## License",
        )
        .unwrap();

        let mut cache = FileCache::new();
        cache.insert(
            temp_dir.path().join("README.md"),
            "# P\n\n## Installation\n## Usage\n## Examples\n## License".to_string(),
        );

        let scorer = DocumentationScorer::new();
        let result = scorer.score_readme(temp_dir.path(), Some(&cache)).unwrap();

        // 4+ sections = full points
        assert_eq!(result, 5.0);
    }

    #[test]
    fn test_changelog_with_cache() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("CHANGELOG.md"),
            "## [0.1.0]\n## [0.2.0]",
        )
        .unwrap();

        let mut cache = FileCache::new();
        cache.insert(
            temp_dir.path().join("CHANGELOG.md"),
            "## [0.1.0]\n## [0.2.0]".to_string(),
        );

        let scorer = DocumentationScorer::new();
        let result = scorer
            .score_changelog(temp_dir.path(), Some(&cache))
            .unwrap();

        // Multiple versions = full points
        assert_eq!(result, 3.0);
    }

    #[test]
    fn test_score_with_mode_fast() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(temp_dir.path().join("src/lib.rs"), "pub fn foo() {}").unwrap();

        let scorer = DocumentationScorer::new();
        let result = scorer
            .score_with_mode(temp_dir.path(), ScoringMode::Fast)
            .unwrap();

        // Mode doesn't affect documentation scorer
        assert!(result.earned >= 0.0);
        assert_eq!(result.max, 15.0);
    }

    #[test]
    fn test_score_with_mode_full() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(temp_dir.path().join("src/lib.rs"), "pub fn foo() {}").unwrap();

        let scorer = DocumentationScorer::new();
        let result = scorer
            .score_with_mode(temp_dir.path(), ScoringMode::Full)
            .unwrap();

        // Mode doesn't affect documentation scorer
        assert!(result.earned >= 0.0);
        assert_eq!(result.max, 15.0);
    }
}
