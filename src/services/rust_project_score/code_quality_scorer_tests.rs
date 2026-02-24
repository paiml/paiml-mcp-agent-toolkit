#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_scorer_creation() {
        let scorer = CodeQualityScorer::new();
        assert_eq!(scorer.name(), "Code Quality");
        assert_eq!(scorer.max_points(), 26.0);
    }

    #[test]
    fn test_scorer_implements_trait() {
        let scorer = CodeQualityScorer::new();
        let _trait_obj: &dyn Scorer = &scorer;
    }

    #[test]
    fn test_default_trait() {
        let scorer = CodeQualityScorer::default();
        assert_eq!(scorer.name(), "Code Quality");
        assert_eq!(scorer.max_points(), 26.0);
    }

    #[test]
    fn test_invalid_project_no_cargo_toml() {
        let temp_dir = TempDir::new().unwrap();
        let scorer = CodeQualityScorer::new();

        let result = scorer.score_with_mode(temp_dir.path(), ScoringMode::Fast);
        assert!(result.is_err());
        match result {
            Err(ScorerError::InvalidProject(msg)) => {
                assert!(msg.contains("No Cargo.toml found"));
            }
            _ => panic!("Expected InvalidProject error"),
        }
    }

    #[test]
    fn test_complexity_no_src_directory() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();

        let scorer = CodeQualityScorer::new();
        let result = scorer
            .score_complexity_simple(temp_dir.path(), None)
            .unwrap();

        // No code = no complexity issues = full points
        assert_eq!(result, 3.0);
    }

    #[test]
    fn test_complexity_no_deep_nesting() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("src/lib.rs"),
            "fn main() {\n    println!(\"hello\");\n}",
        )
        .unwrap();

        let scorer = CodeQualityScorer::new();
        let result = scorer
            .score_complexity_simple(temp_dir.path(), None)
            .unwrap();

        // No deep nesting = full points
        assert_eq!(result, 3.0);
    }

    #[test]
    fn test_complexity_with_moderate_nesting() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();

        // Create code with 3 deeply nested lines (indent > 32 chars)
        let deep_code = format!(
            "fn main() {{\n{}",
            "                                    nested();\n".repeat(3)
        );
        fs::write(temp_dir.path().join("src/lib.rs"), deep_code).unwrap();

        let scorer = CodeQualityScorer::new();
        let result = scorer
            .score_complexity_simple(temp_dir.path(), None)
            .unwrap();

        // 1-5 deep nesting = 2.0 points
        assert_eq!(result, 2.0);
    }

    #[test]
    fn test_complexity_with_excessive_nesting() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();

        // Create code with >20 deeply nested lines
        let deep_code = format!(
            "fn main() {{\n{}",
            "                                    nested();\n".repeat(25)
        );
        fs::write(temp_dir.path().join("src/lib.rs"), deep_code).unwrap();

        let scorer = CodeQualityScorer::new();
        let result = scorer
            .score_complexity_simple(temp_dir.path(), None)
            .unwrap();

        // >20 deep nesting = 0.0 points
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_unsafe_no_src_directory() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();

        let scorer = CodeQualityScorer::new();
        let result = scorer.score_unsafe(temp_dir.path(), None).unwrap();

        // No code = no unsafe = full points
        assert_eq!(result, 9.0);
    }

    #[test]
    fn test_unsafe_no_unsafe_blocks() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("src/lib.rs"),
            "fn safe_code() { println!(\"safe\"); }",
        )
        .unwrap();

        let scorer = CodeQualityScorer::new();
        let result = scorer.score_unsafe(temp_dir.path(), None).unwrap();

        // No unsafe = full points
        assert_eq!(result, 9.0);
    }

    #[test]
    fn test_unsafe_documented_blocks() {
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
// SAFETY: This is safe because reasons
unsafe {
    do_unsafe_thing();
}
"#,
        )
        .unwrap();

        let scorer = CodeQualityScorer::new();
        let result = scorer.score_unsafe(temp_dir.path(), None).unwrap();

        // 100% documented = full points
        assert_eq!(result, 9.0);
    }

    #[test]
    fn test_unsafe_undocumented_blocks() {
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
// No SAFETY comment here
fn foo() {
    unsafe {
        do_thing();
    }
}
"#,
        )
        .unwrap();

        let scorer = CodeQualityScorer::new();
        let result = scorer.score_unsafe(temp_dir.path(), None).unwrap();

        // 0% documented = 1.0 points
        assert_eq!(result, 1.0);
    }

    #[test]
    fn test_dead_code_no_src_directory() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();

        let scorer = CodeQualityScorer::new();
        let result = scorer.score_dead_code(temp_dir.path(), None).unwrap();

        // No code = full points
        assert_eq!(result, 2.0);
    }

    #[test]
    fn test_dead_code_no_allow_attributes() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("src/lib.rs"),
            "fn used_function() { println!(\"used\"); }",
        )
        .unwrap();

        let scorer = CodeQualityScorer::new();
        let result = scorer.score_dead_code(temp_dir.path(), None).unwrap();

        // No dead code = full points
        assert_eq!(result, 2.0);
    }

    #[test]
    fn test_dead_code_few_allow_attributes() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("src/lib.rs"),
            "#[allow(dead_code)]\nfn unused1() {}\n#[allow(dead_code)]\nfn unused2() {}",
        )
        .unwrap();

        let scorer = CodeQualityScorer::new();
        let result = scorer.score_dead_code(temp_dir.path(), None).unwrap();

        // 1-3 dead code = 1.0 points
        assert_eq!(result, 1.0);
    }

    #[test]
    fn test_dead_code_many_allow_attributes() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("src/lib.rs"),
            "#[allow(dead_code)]\nfn unused1() {}\n#[allow(dead_code)]\nfn unused2() {}\n#[allow(dead_code)]\nfn unused3() {}\n#[allow(dead_code)]\nfn unused4() {}",
        )
        .unwrap();

        let scorer = CodeQualityScorer::new();
        let result = scorer.score_dead_code(temp_dir.path(), None).unwrap();

        // >3 dead code = 0.0 points
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_score_fast_mode() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(temp_dir.path().join("src/lib.rs"), "fn main() {}").unwrap();

        let scorer = CodeQualityScorer::new();
        let result = scorer
            .score_with_mode(temp_dir.path(), ScoringMode::Fast)
            .unwrap();

        // Fast mode: complexity(3) + unsafe(9) + mutation(4, skipped) + build(2, skipped) + dead_code(2) = 20
        assert!(result.earned >= 18.0);
        assert_eq!(result.max, 26.0);
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
        fs::write(temp_dir.path().join("src/lib.rs"), "fn main() {}").unwrap();

        // Create cache
        let mut cache = FileCache::new();
        cache.insert(
            temp_dir.path().join("src/lib.rs"),
            "fn main() {}".to_string(),
        );

        let scorer = CodeQualityScorer::new();
        let result = scorer
            .score_with_cache(temp_dir.path(), ScoringMode::Fast, Some(&cache))
            .unwrap();

        assert!(result.earned >= 18.0);
        assert_eq!(result.max, 26.0);
    }

    #[test]
    fn test_recommendations_clean_code() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(temp_dir.path().join("src/lib.rs"), "fn main() {}").unwrap();

        let scorer = CodeQualityScorer::new();
        let recommendations = scorer.recommendations(temp_dir.path());

        // Should always include mutation testing recommendation
        assert!(recommendations.iter().any(|r| r.contains("cargo-mutants")));
    }

    #[test]
    fn test_recommendations_with_issues() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();

        // Create code with undocumented unsafe and dead code
        fs::write(
            temp_dir.path().join("src/lib.rs"),
            r#"
#[allow(dead_code)]
fn unused() {}

fn foo() {
    unsafe {
        do_thing();
    }
}
"#,
        )
        .unwrap();

        let scorer = CodeQualityScorer::new();
        let recommendations = scorer.recommendations(temp_dir.path());

        // Should have SAFETY comment recommendation
        assert!(recommendations.iter().any(|r| r.contains("SAFETY")));
    }

    #[test]
    fn test_complexity_with_cache() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();

        // Create cache with shallow code
        let mut cache = FileCache::new();
        cache.insert(
            temp_dir.path().join("src/lib.rs"),
            "fn main() {\n    println!(\"hello\");\n}".to_string(),
        );

        let scorer = CodeQualityScorer::new();
        let result = scorer
            .score_complexity_simple(temp_dir.path(), Some(&cache))
            .unwrap();

        // No deep nesting = full points
        assert_eq!(result, 3.0);
    }

    #[test]
    fn test_unsafe_with_cache() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();

        // Create cache with documented unsafe
        let mut cache = FileCache::new();
        cache.insert(
            temp_dir.path().join("src/lib.rs"),
            "// SAFETY: documented\nunsafe {\n    thing();\n}".to_string(),
        );

        let scorer = CodeQualityScorer::new();
        let result = scorer.score_unsafe(temp_dir.path(), Some(&cache)).unwrap();

        // 100% documented = full points
        assert_eq!(result, 9.0);
    }

    #[test]
    fn test_dead_code_with_cache() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();

        // Create cache with no dead code
        let mut cache = FileCache::new();
        cache.insert(
            temp_dir.path().join("src/lib.rs"),
            "fn used_function() {}".to_string(),
        );

        let scorer = CodeQualityScorer::new();
        let result = scorer
            .score_dead_code(temp_dir.path(), Some(&cache))
            .unwrap();

        // No dead code = full points
        assert_eq!(result, 2.0);
    }

    #[test]
    fn test_scoring_mode_quick() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(temp_dir.path().join("src/lib.rs"), "fn main() {}").unwrap();

        let scorer = CodeQualityScorer::new();
        let result = scorer
            .score_with_mode(temp_dir.path(), ScoringMode::Quick)
            .unwrap();

        // Quick mode should still produce valid scores
        assert!(result.earned >= 0.0);
        assert!(result.earned <= result.max);
    }
}
