#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_scorer_creation() {
        let scorer = DependencyScorer::new();
        assert_eq!(scorer.name(), "Dependency Health");
        assert_eq!(scorer.max_points(), 12.0);
    }

    #[test]
    fn test_scorer_implements_trait() {
        let scorer = DependencyScorer::new();
        let _trait_obj: &dyn Scorer = &scorer;
    }

    #[test]
    fn test_default_trait() {
        let scorer = DependencyScorer::default();
        assert_eq!(scorer.name(), "Dependency Health");
        assert_eq!(scorer.max_points(), 12.0);
    }

    #[test]
    fn test_invalid_project_no_cargo_toml() {
        let temp_dir = TempDir::new().unwrap();
        let scorer = DependencyScorer::new();

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
    fn test_dependency_count_minimal() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
serde = "1.0"
tokio = "1.0"
"#,
        )
        .unwrap();

        let scorer = DependencyScorer::new();
        let result = scorer
            .score_dependency_count(temp_dir.path(), None)
            .unwrap();

        // 2 dependencies = minimal, full points
        assert_eq!(result, 5.0);
    }

    #[test]
    fn test_dependency_count_moderate() {
        let temp_dir = TempDir::new().unwrap();
        let mut cargo_toml = "[package]\nname = \"test\"\n\n[dependencies]\n".to_string();
        for i in 1..=15 {
            cargo_toml.push_str(&format!("dep{} = \"1.0\"\n", i));
        }
        fs::write(temp_dir.path().join("Cargo.toml"), cargo_toml).unwrap();

        let scorer = DependencyScorer::new();
        let result = scorer
            .score_dependency_count(temp_dir.path(), None)
            .unwrap();

        // 15 dependencies = lean, full points (#242: relaxed thresholds)
        assert_eq!(result, 5.0);
    }

    #[test]
    fn test_dependency_count_many() {
        let temp_dir = TempDir::new().unwrap();
        let mut cargo_toml = "[package]\nname = \"test\"\n\n[dependencies]\n".to_string();
        for i in 1..=25 {
            cargo_toml.push_str(&format!("dep{} = \"1.0\"\n", i));
        }
        fs::write(temp_dir.path().join("Cargo.toml"), cargo_toml).unwrap();

        let scorer = DependencyScorer::new();
        let result = scorer
            .score_dependency_count(temp_dir.path(), None)
            .unwrap();

        // 25 dependencies = moderate, good points (#242: relaxed thresholds)
        assert_eq!(result, 4.0);
    }

    #[test]
    fn test_dependency_count_excessive() {
        let temp_dir = TempDir::new().unwrap();
        let mut cargo_toml = "[package]\nname = \"test\"\n\n[dependencies]\n".to_string();
        for i in 1..=35 {
            cargo_toml.push_str(&format!("dep{} = \"1.0\"\n", i));
        }
        fs::write(temp_dir.path().join("Cargo.toml"), cargo_toml).unwrap();

        let scorer = DependencyScorer::new();
        let result = scorer
            .score_dependency_count(temp_dir.path(), None)
            .unwrap();

        // 35 dependencies = many, acceptable points (#242: relaxed thresholds)
        assert_eq!(result, 2.0);
    }

    #[test]
    fn test_dependency_count_with_cache() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_content = "[package]\nname = \"test\"\n\n[dependencies]\nserde = \"1.0\"\n";
        fs::write(temp_dir.path().join("Cargo.toml"), cargo_content).unwrap();

        let mut cache = FileCache::new();
        cache.insert(
            temp_dir.path().join("Cargo.toml"),
            cargo_content.to_string(),
        );

        let scorer = DependencyScorer::new();
        let result = scorer
            .score_dependency_count(temp_dir.path(), Some(&cache))
            .unwrap();

        // 1 dependency = minimal, full points
        assert_eq!(result, 5.0);
    }

    #[test]
    fn test_feature_flags_none() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\n\n[dependencies]\n",
        )
        .unwrap();

        let scorer = DependencyScorer::new();
        let result = scorer.score_feature_flags(temp_dir.path(), None).unwrap();

        // No features = 0 points
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_feature_flags_some() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            r#"
[package]
name = "test"

[features]
default = ["std"]
std = []
"#,
        )
        .unwrap();

        let scorer = DependencyScorer::new();
        let result = scorer.score_feature_flags(temp_dir.path(), None).unwrap();

        // 2 features = some points
        assert_eq!(result, 3.0);
    }

    #[test]
    fn test_feature_flags_comprehensive() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            r#"
[package]
name = "test"

[features]
default = ["std"]
std = []
async = ["tokio"]
full = ["std", "async"]
"#,
        )
        .unwrap();

        let scorer = DependencyScorer::new();
        let result = scorer.score_feature_flags(temp_dir.path(), None).unwrap();

        // 4 features = full points
        assert_eq!(result, 4.0);
    }

    #[test]
    fn test_feature_flags_with_cache() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_content =
            "[package]\nname = \"test\"\n\n[features]\ndefault = []\nstd = []\nfull = []\n";
        fs::write(temp_dir.path().join("Cargo.toml"), cargo_content).unwrap();

        let mut cache = FileCache::new();
        cache.insert(
            temp_dir.path().join("Cargo.toml"),
            cargo_content.to_string(),
        );

        let scorer = DependencyScorer::new();
        let result = scorer
            .score_feature_flags(temp_dir.path(), Some(&cache))
            .unwrap();

        // 3 features = full points
        assert_eq!(result, 4.0);
    }

    #[test]
    fn test_dependency_count_ignores_comments() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            r#"
[package]
name = "test"

[dependencies]
# This is a comment
serde = "1.0"
# Another comment
tokio = "1.0"
"#,
        )
        .unwrap();

        let scorer = DependencyScorer::new();
        let result = scorer
            .score_dependency_count(temp_dir.path(), None)
            .unwrap();

        // Should only count actual dependencies, not comments
        assert_eq!(result, 5.0);
    }

    #[test]
    fn test_dependency_section_ends() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            r#"
[package]
name = "test"

[dependencies]
serde = "1.0"

[dev-dependencies]
tempfile = "3.0"
"#,
        )
        .unwrap();

        let scorer = DependencyScorer::new();
        let result = scorer
            .score_dependency_count(temp_dir.path(), None)
            .unwrap();

        // Should only count [dependencies], not [dev-dependencies]
        assert_eq!(result, 5.0);
    }
}
