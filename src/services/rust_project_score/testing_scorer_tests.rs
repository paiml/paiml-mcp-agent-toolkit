// Tests for TestingScorer - Part 1: Creation, coverage, integration, and doc tests
// Included from testing_scorer.rs - shares parent module scope

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_scorer_creation() {
        let scorer = TestingScorer::new();
        assert_eq!(scorer.name(), "Testing Excellence");
        assert_eq!(scorer.max_points(), 20.0);
    }

    #[test]
    fn test_scorer_implements_trait() {
        let scorer = TestingScorer::new();
        let _trait_obj: &dyn Scorer = &scorer;
    }

    #[test]
    fn test_default_trait() {
        let scorer = TestingScorer::default();
        assert_eq!(scorer.name(), "Testing Excellence");
        assert_eq!(scorer.max_points(), 20.0);
    }

    #[test]
    fn test_invalid_project_no_cargo_toml() {
        let temp_dir = TempDir::new().unwrap();
        let scorer = TestingScorer::new();

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
    fn test_coverage_fallback_no_src() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();

        let scorer = TestingScorer::new();
        let result = scorer
            .score_coverage_fallback(temp_dir.path(), None)
            .unwrap();

        // No src = 0 points
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_coverage_fallback_no_tests() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(temp_dir.path().join("src/lib.rs"), "fn no_tests_here() {}").unwrap();

        let scorer = TestingScorer::new();
        let result = scorer
            .score_coverage_fallback(temp_dir.path(), None)
            .unwrap();

        // No tests = 0 points
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_coverage_fallback_with_tests() {
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
fn foo() {}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    #[test]
    fn test_foo() {}
}
"#,
        )
        .unwrap();

        let scorer = TestingScorer::new();
        let result = scorer
            .score_coverage_fallback(temp_dir.path(), None)
            .unwrap();

        // Has tests = moderate credit
        assert_eq!(result, 4.0);
    }

    #[test]
    fn test_coverage_fallback_with_cache() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();

        let mut cache = FileCache::new();
        cache.insert(
            temp_dir.path().join("src/lib.rs"),
            "#[test]\nfn test_something() {}".to_string(),
        );

        let scorer = TestingScorer::new();
        let result = scorer
            .score_coverage_fallback(temp_dir.path(), Some(&cache))
            .unwrap();

        // Has tests = moderate credit
        assert_eq!(result, 4.0);
    }

    #[test]
    fn test_integration_tests_no_directory() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();

        let scorer = TestingScorer::new();
        let result = scorer.score_integration_tests(temp_dir.path()).unwrap();

        // No tests/ = 0 points
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_integration_tests_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("tests")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();

        let scorer = TestingScorer::new();
        let result = scorer.score_integration_tests(temp_dir.path()).unwrap();

        // Empty tests/ = 0 points
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_integration_tests_one_file() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("tests")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("tests/integration.rs"),
            "#[test]\nfn test() {}",
        )
        .unwrap();

        let scorer = TestingScorer::new();
        let result = scorer.score_integration_tests(temp_dir.path()).unwrap();

        // 1 test file = 3 points
        assert_eq!(result, 3.0);
    }

    #[test]
    fn test_integration_tests_multiple_files() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("tests")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("tests/test1.rs"),
            "#[test]\nfn t1() {}",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("tests/test2.rs"),
            "#[test]\nfn t2() {}",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("tests/test3.rs"),
            "#[test]\nfn t3() {}",
        )
        .unwrap();

        let scorer = TestingScorer::new();
        let result = scorer.score_integration_tests(temp_dir.path()).unwrap();

        // 3+ test files = full points
        assert_eq!(result, 4.0);
    }

    #[test]
    fn test_doc_tests_no_src() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();

        let scorer = TestingScorer::new();
        let result = scorer.score_doc_tests(temp_dir.path(), None).unwrap();

        // No src = 0 points
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_doc_tests_no_examples() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("src/lib.rs"),
            "/// Documentation without examples\npub fn foo() {}",
        )
        .unwrap();

        let scorer = TestingScorer::new();
        let result = scorer.score_doc_tests(temp_dir.path(), None).unwrap();

        // No doc tests = 0 points
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_doc_tests_with_examples() {
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
/// Documentation with example
/// ```
/// let x = 1;
/// ```
pub fn foo() {}

/// Another example
/// ```
/// let y = 2;
/// ```
pub fn bar() {}
"#,
        )
        .unwrap();

        let scorer = TestingScorer::new();
        let result = scorer.score_doc_tests(temp_dir.path(), None).unwrap();

        // 2 doc tests = 1 point (need 3+ for 2 pts, 5+ for full)
        assert!(result >= 1.0);
    }

    #[test]
    fn test_doc_tests_many_examples() {
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
/// ```
/// let a = 1;
/// ```
pub fn a() {}
/// ```
/// let b = 2;
/// ```
pub fn b() {}
/// ```
/// let c = 3;
/// ```
pub fn c() {}
/// ```
/// let d = 4;
/// ```
pub fn d() {}
/// ```
/// let e = 5;
/// ```
pub fn e() {}
"#,
        )
        .unwrap();

        let scorer = TestingScorer::new();
        let result = scorer.score_doc_tests(temp_dir.path(), None).unwrap();

        // 5+ doc tests = full points
        assert_eq!(result, 3.0);
    }

    #[test]
    fn test_parse_coverage_valid() {
        let scorer = TestingScorer::new();

        assert_eq!(scorer.parse_coverage("coverage: 85.0%"), Some(85.0));
        assert_eq!(scorer.parse_coverage("Total: 92.5%"), Some(92.5));
        assert_eq!(scorer.parse_coverage("line: 50%"), Some(50.0));
    }

    #[test]
    fn test_parse_coverage_invalid() {
        let scorer = TestingScorer::new();

        assert_eq!(scorer.parse_coverage("no percentage here"), None);
        assert_eq!(scorer.parse_coverage(""), None);
    }
}
