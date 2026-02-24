#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_part2 {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_tree_pruning_none() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\n\n[dependencies]\nserde = \"1.0\"\n",
        )
        .unwrap();

        let scorer = DependencyScorer::new();
        let result = scorer.score_tree_pruning(temp_dir.path(), None).unwrap();

        // No pruning practices = 0 points
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_tree_pruning_optional_deps() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            r#"
[package]
name = "test"

[dependencies]
tokio = { version = "1.0", optional = true }
"#,
        )
        .unwrap();

        let scorer = DependencyScorer::new();
        let result = scorer.score_tree_pruning(temp_dir.path(), None).unwrap();

        // Optional deps = 1.5 points
        assert_eq!(result, 1.5);
    }

    #[test]
    fn test_tree_pruning_features_list() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            r#"
[package]
name = "test"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
"#,
        )
        .unwrap();

        let scorer = DependencyScorer::new();
        let result = scorer.score_tree_pruning(temp_dir.path(), None).unwrap();

        // Features list = 1.0 points
        assert_eq!(result, 1.0);
    }

    #[test]
    fn test_tree_pruning_disable_defaults() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            r#"
[package]
name = "test"

[dependencies]
serde = { version = "1.0", default-features = false }
"#,
        )
        .unwrap();

        let scorer = DependencyScorer::new();
        let result = scorer.score_tree_pruning(temp_dir.path(), None).unwrap();

        // Disable defaults = 0.5 points
        assert_eq!(result, 0.5);
    }

    #[test]
    fn test_tree_pruning_all_practices() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            r#"
[package]
name = "test"

[dependencies]
tokio = { version = "1.0", optional = true }
serde = { version = "1.0", features = ["derive"], default-features = false }
"#,
        )
        .unwrap();

        let scorer = DependencyScorer::new();
        let result = scorer.score_tree_pruning(temp_dir.path(), None).unwrap();

        // All practices = capped at 3.0 points
        assert_eq!(result, 3.0);
    }

    #[test]
    fn test_tree_pruning_with_cache() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_content = "[package]\nname = \"test\"\n\n[dependencies]\ntokio = { version = \"1.0\", optional = true }\n";
        fs::write(temp_dir.path().join("Cargo.toml"), cargo_content).unwrap();

        let mut cache = FileCache::new();
        cache.insert(
            temp_dir.path().join("Cargo.toml"),
            cargo_content.to_string(),
        );

        let scorer = DependencyScorer::new();
        let result = scorer
            .score_tree_pruning(temp_dir.path(), Some(&cache))
            .unwrap();

        assert_eq!(result, 1.5);
    }

    #[test]
    fn test_score_full_project() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            r#"
[package]
name = "test"

[dependencies]
serde = { version = "1.0", features = ["derive"], default-features = false }
tokio = { version = "1.0", optional = true }

[features]
default = ["std"]
std = []
async = ["tokio"]
"#,
        )
        .unwrap();

        let scorer = DependencyScorer::new();
        let result = scorer.score(temp_dir.path()).unwrap();

        // Should get high score: deps(5) + features(4) + pruning(3) = 12
        assert!(result.earned >= 10.0);
        assert_eq!(result.max, 12.0);
    }

    #[test]
    fn test_score_with_cache() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_content = r#"
[package]
name = "test"

[dependencies]
serde = "1.0"

[features]
default = []
"#;
        fs::write(temp_dir.path().join("Cargo.toml"), cargo_content).unwrap();

        let mut cache = FileCache::new();
        cache.insert(
            temp_dir.path().join("Cargo.toml"),
            cargo_content.to_string(),
        );

        let scorer = DependencyScorer::new();
        let result = scorer
            .score_with_cache(temp_dir.path(), ScoringMode::Fast, Some(&cache))
            .unwrap();

        assert!(result.earned > 0.0);
        assert_eq!(result.max, 12.0);
    }

    #[test]
    fn test_recommendations_poor_deps() {
        let temp_dir = TempDir::new().unwrap();
        let mut cargo_toml = "[package]\nname = \"test\"\n\n[dependencies]\n".to_string();
        for i in 1..=35 {
            cargo_toml.push_str(&format!("dep{} = \"1.0\"\n", i));
        }
        fs::write(temp_dir.path().join("Cargo.toml"), cargo_toml).unwrap();

        let scorer = DependencyScorer::new();
        let recommendations = scorer.recommendations(temp_dir.path());

        // Should recommend reducing dependencies
        assert!(recommendations
            .iter()
            .any(|r| r.contains("Reduce dependency")));
    }

    #[test]
    fn test_recommendations_no_features() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\n\n[dependencies]\nserde = \"1.0\"\n",
        )
        .unwrap();

        let scorer = DependencyScorer::new();
        let recommendations = scorer.recommendations(temp_dir.path());

        // Should recommend adding features
        assert!(recommendations.iter().any(|r| r.contains("feature")));
    }

    #[test]
    fn test_recommendations_no_pruning() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\n\n[dependencies]\nserde = \"1.0\"\n\n[features]\ndefault = []\nstd = []\nfull = []\n",
        )
        .unwrap();

        let scorer = DependencyScorer::new();
        let recommendations = scorer.recommendations(temp_dir.path());

        // Should recommend pruning
        assert!(recommendations
            .iter()
            .any(|r| r.contains("optional") || r.contains("default features")));
    }

    #[test]
    fn test_score_with_mode_fast() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\n\n[dependencies]\nserde = \"1.0\"\n",
        )
        .unwrap();

        let scorer = DependencyScorer::new();
        let result = scorer
            .score_with_mode(temp_dir.path(), ScoringMode::Fast)
            .unwrap();

        // Mode doesn't affect dependency scorer
        assert!(result.earned >= 0.0);
        assert_eq!(result.max, 12.0);
    }
}
