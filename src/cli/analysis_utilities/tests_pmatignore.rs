#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod pmatignore_integration_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// RED Test: analyze_project_files MUST respect .pmatignore exclusions
    ///
    /// Root cause: analyze_project_files() uses walkdir directly, bypassing
    /// ProjectFileDiscovery which has .pmatignore/.paimlignore support.
    ///
    /// Bug discovered in ruchy: validate.rs in tests_temp_disabled_for_sprint7_mutation/
    /// was being analyzed despite .pmatignore exclusion pattern.
    #[tokio::test]
    async fn test_analyze_project_files_respects_pmatignore() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create project structure
        fs::write(root.join("main.rs"), "fn main() { let x = 1; }").unwrap();
        fs::write(root.join("lib.rs"), "pub fn lib() { let y = 2; }").unwrap();

        // Create disabled tests directory (should be excluded)
        fs::create_dir(root.join("tests_disabled")).unwrap();
        fs::write(
            root.join("tests_disabled/validate.rs"),
            "fn validate() { let z = 3; }",
        )
        .unwrap();

        // Create .pmatignore file
        fs::write(
            root.join(".pmatignore"),
            "tests_disabled/\ntests_disabled/**\n",
        )
        .unwrap();

        // Analyze project files
        let results = analyze_project_files(root, Some("rust"), &[], 100, 100)
            .await
            .unwrap();

        // CRITICAL: Should only find main.rs and lib.rs
        assert_eq!(
            results.len(),
            2,
            "Should find exactly 2 files (main.rs, lib.rs), not including tests_disabled/"
        );

        let file_paths: Vec<String> = results.iter().map(|r| r.path.clone()).collect();

        assert!(
            file_paths.iter().any(|p| p.ends_with("main.rs")),
            "Should find main.rs"
        );
        assert!(
            file_paths.iter().any(|p| p.ends_with("lib.rs")),
            "Should find lib.rs"
        );

        // CRITICAL: Should NOT find validate.rs in tests_disabled/
        assert!(
            !file_paths.iter().any(|p| p.contains("tests_disabled")),
            ".pmatignore should exclude tests_disabled/ directory, but found: {:?}",
            file_paths
        );
        assert!(
            !file_paths.iter().any(|p| p.ends_with("validate.rs")),
            ".pmatignore should exclude validate.rs, but found: {:?}",
            file_paths
        );
    }

    /// Test that .paimlignore (legacy) still works
    #[tokio::test]
    async fn test_analyze_project_files_respects_paimlignore() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        fs::write(root.join("keep.rs"), "fn keep() {}").unwrap();
        fs::write(root.join("exclude.rs"), "fn exclude() {}").unwrap();

        // Create .paimlignore (legacy name)
        fs::write(root.join(".paimlignore"), "exclude.rs\n").unwrap();

        let results = analyze_project_files(root, Some("rust"), &[], 100, 100)
            .await
            .unwrap();

        assert_eq!(results.len(), 1);
        assert!(results[0].path.ends_with("keep.rs"));
        assert!(!results.iter().any(|r| r.path.ends_with("exclude.rs")));
    }

    /// Test that both .pmatignore and .paimlignore work together
    #[tokio::test]
    async fn test_analyze_project_files_respects_both_ignore_files() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        fs::write(root.join("keep.rs"), "fn keep() {}").unwrap();
        fs::write(root.join("exclude1.rs"), "fn exclude1() {}").unwrap();
        fs::write(root.join("exclude2.rs"), "fn exclude2() {}").unwrap();

        // Create both ignore files
        fs::write(root.join(".pmatignore"), "exclude1.rs\n").unwrap();
        fs::write(root.join(".paimlignore"), "exclude2.rs\n").unwrap();

        let results = analyze_project_files(root, Some("rust"), &[], 100, 100)
            .await
            .unwrap();

        assert_eq!(results.len(), 1, "Should only find keep.rs");
        assert!(results[0].path.ends_with("keep.rs"));
        assert!(!results.iter().any(|r| r.path.ends_with("exclude1.rs")));
        assert!(!results.iter().any(|r| r.path.ends_with("exclude2.rs")));
    }
}
