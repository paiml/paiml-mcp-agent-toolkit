// Tests for TestingScorer - Part 2: Scoring modes, recommendations, and config warnings
// Included from testing_scorer.rs - shares parent module scope

#[cfg(test)]
mod tests_part2 {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_score_fast_mode() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("src/lib.rs"),
            "#[cfg(test)]\nmod tests { #[test] fn t() {} }",
        )
        .unwrap();

        let scorer = TestingScorer::new();
        let result = scorer
            .score_with_mode(temp_dir.path(), ScoringMode::Fast)
            .unwrap();

        // Fast mode: coverage fallback(4) + integration(0) + doc_tests(0) + mutation(2.5) = 6.5
        assert!(result.earned >= 6.0);
        assert_eq!(result.max, 20.0);
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

        let mut cache = FileCache::new();
        cache.insert(
            temp_dir.path().join("src/lib.rs"),
            "#[test]\nfn test_something() {}".to_string(),
        );

        let scorer = TestingScorer::new();
        let result = scorer
            .score_with_cache(temp_dir.path(), ScoringMode::Fast, Some(&cache))
            .unwrap();

        assert!(result.earned >= 0.0);
        assert_eq!(result.max, 20.0);
    }

    #[test]
    fn test_recommendations_no_tests() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(temp_dir.path().join("src/lib.rs"), "fn foo() {}").unwrap();

        let scorer = TestingScorer::new();
        let recommendations = scorer.recommendations(temp_dir.path());

        // Should recommend testing areas (mutation moved to code_quality_scorer)
        assert!(recommendations.iter().any(|r| r.contains("coverage")));
        assert!(recommendations.iter().any(|r| r.contains("integration")));
        assert!(recommendations.iter().any(|r| r.contains("doc test")));
    }

    #[test]
    fn test_recommendations_with_tests() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::create_dir_all(temp_dir.path().join("tests")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("src/lib.rs"),
            "#[cfg(test)] mod tests { #[test] fn t() {} }",
        )
        .unwrap();
        fs::write(temp_dir.path().join("tests/int.rs"), "#[test] fn i() {}").unwrap();

        let scorer = TestingScorer::new();
        let recommendations = scorer.recommendations(temp_dir.path());

        // Should still have some recommendations
        assert!(!recommendations.is_empty());
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

        let scorer = TestingScorer::new();
        let result = scorer
            .score_with_mode(temp_dir.path(), ScoringMode::Quick)
            .unwrap();

        // Quick mode should produce valid scores
        assert!(result.earned >= 0.0);
        assert!(result.earned <= result.max);
    }

    #[test]
    fn test_module_doc_comments() {
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
//! Module documentation
//! ```
//! let module_example = 1;
//! ```

/// Function doc
/// ```
/// let func_example = 2;
/// ```
pub fn foo() {}
"#,
        )
        .unwrap();

        let scorer = TestingScorer::new();
        let result = scorer.score_doc_tests(temp_dir.path(), None).unwrap();

        // Should count both //! and /// doc tests
        assert!(result >= 1.0);
    }

    #[test]
    fn test_coverage_config_warning_nextest_without_guard() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Makefile"),
            r#"
coverage:
	cargo llvm-cov nextest --lib
	cargo llvm-cov report
"#,
        )
        .unwrap();

        let scorer = TestingScorer::new();
        let warning = scorer.check_coverage_config_warning(temp_dir.path());

        // Should warn about nextest without profraw guard
        assert!(warning.is_some());
        assert!(warning.unwrap().contains("nextest"));
    }

    #[test]
    fn test_coverage_config_warning_nextest_with_guard() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Makefile"),
            r#"
coverage:
	find . -name "*.profraw" -delete
	cargo llvm-cov nextest --lib
	cargo llvm-cov report
"#,
        )
        .unwrap();

        let scorer = TestingScorer::new();
        let warning = scorer.check_coverage_config_warning(temp_dir.path());

        // Should NOT warn - has profraw cleanup guard
        assert!(warning.is_none());
    }

    #[test]
    fn test_coverage_config_warning_cargo_test_ok() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Makefile"),
            r#"
coverage:
	cargo llvm-cov test --lib
	cargo llvm-cov report
"#,
        )
        .unwrap();

        let scorer = TestingScorer::new();
        let warning = scorer.check_coverage_config_warning(temp_dir.path());

        // Should NOT warn - uses cargo test (not nextest)
        assert!(warning.is_none());
    }

    #[test]
    fn test_coverage_config_warning_no_makefile() {
        let temp_dir = TempDir::new().unwrap();

        let scorer = TestingScorer::new();
        let warning = scorer.check_coverage_config_warning(temp_dir.path());

        // Should NOT warn - no Makefile
        assert!(warning.is_none());
    }
}
