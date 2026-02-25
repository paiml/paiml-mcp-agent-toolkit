#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_falsifiability_scorer_basics() {
        let scorer = FalsifiabilityScorer::new();
        assert_eq!(scorer.name(), "Falsifiability & Testability");
        assert_eq!(scorer.category_id(), 'A');
        assert_eq!(scorer.max_points(), 25.0);
        assert!(scorer.is_gateway());
    }

    #[test]
    fn test_empty_project_low_score() {
        let temp_dir = tempdir().expect("internal error");
        let scorer = FalsifiabilityScorer::new();

        let result = scorer.score(temp_dir.path()).expect("internal error");
        assert!(result.earned < 15.0); // Should fail gateway
    }

    #[test]
    fn test_project_with_tests_higher_score() {
        let temp_dir = tempdir().expect("internal error");

        // Create tests directory
        fs::create_dir_all(temp_dir.path().join("tests")).expect("internal error");
        fs::write(
            temp_dir.path().join("tests/test_main.rs"),
            "#[test]\nfn test_example() {}",
        )
        .expect("internal error");

        // Create README with claims
        fs::write(
            temp_dir.path().join("README.md"),
            "# Project\n\nThis project claims to provide >10x performance improvement.\n\n## Success Criteria\n\n- All tests pass",
        ).expect("internal error");

        let scorer = FalsifiabilityScorer::new();
        let result = scorer.score(temp_dir.path()).expect("internal error");

        // Should have earned some points
        assert!(result.earned > 0.0);
        assert!(!result.sub_scores.is_empty());
    }

    #[test]
    fn test_project_with_criterion_benchmarks() {
        let temp_dir = tempdir().expect("internal error");

        // Create benches directory with Criterion
        fs::create_dir_all(temp_dir.path().join("benches")).expect("internal error");
        fs::write(
            temp_dir.path().join("benches/bench.rs"),
            "use criterion::{criterion_group, criterion_main, Criterion};",
        )
        .expect("internal error");

        // Create Cargo.toml
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            r#"[dev-dependencies]
criterion = "0.5"
"#,
        )
        .expect("internal error");

        let scorer = FalsifiabilityScorer::new();
        let result = scorer.score(temp_dir.path()).expect("internal error");

        // Should have benchmark points
        let a3 = result
            .sub_scores
            .iter()
            .find(|s| s.id == "A3")
            .expect("internal error");
        assert!(a3.earned > 0.0);
    }
}
