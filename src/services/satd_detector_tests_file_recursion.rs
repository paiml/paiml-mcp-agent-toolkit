    // TDD RED phase - Tests for collect_files_recursive (22 cognitive complexity)
    #[tokio::test]
    async fn test_collect_files_recursive_empty_directory() {
        let detector = SATDDetector::new();
        let temp_dir = tempfile::tempdir().expect("internal error");
        let empty_dir = temp_dir.path().join("empty");
        std::fs::create_dir(&empty_dir).expect("internal error");

        let mut files = Vec::new();
        detector
            .collect_files_recursive(&empty_dir, &mut files)
            .await
            .expect("internal error");

        assert_eq!(files.len(), 0);
    }

    #[tokio::test]
    async fn test_collect_files_recursive_with_source_files() {
        let detector = SATDDetector::new();
        let temp_dir = tempfile::tempdir().expect("internal error");
        let project_root = temp_dir.path();

        // Create source files
        std::fs::write(project_root.join("main.rs"), "fn main() {}").expect("internal error");
        std::fs::write(project_root.join("lib.py"), "def func(): pass").expect("internal error");
        std::fs::write(project_root.join("script.js"), "console.log('hello');")
            .expect("internal error");
        std::fs::write(project_root.join("readme.txt"), "Not a source file")
            .expect("internal error");

        let mut files = Vec::new();
        detector
            .collect_files_recursive(project_root, &mut files)
            .await
            .expect("internal error");

        assert_eq!(files.len(), 3); // Only source files
        assert!(files
            .iter()
            .any(|f| f.file_name().expect("internal error") == "main.rs"));
        assert!(files
            .iter()
            .any(|f| f.file_name().expect("internal error") == "lib.py"));
        assert!(files
            .iter()
            .any(|f| f.file_name().expect("internal error") == "script.js"));
        assert!(!files
            .iter()
            .any(|f| f.file_name().expect("internal error") == "readme.txt"));
    }

    #[tokio::test]
    async fn test_collect_files_recursive_skips_excluded_directories() {
        let detector = SATDDetector::new();
        let temp_dir = tempfile::tempdir().expect("internal error");
        let project_root = temp_dir.path();

        // Create source files in excluded directories
        std::fs::create_dir_all(project_root.join("target/debug")).expect("internal error");
        std::fs::create_dir_all(project_root.join("node_modules/lib")).expect("internal error");
        std::fs::create_dir_all(project_root.join(".git/hooks")).expect("internal error");
        std::fs::create_dir_all(project_root.join("src")).expect("internal error");

        std::fs::write(project_root.join("target/debug/main.rs"), "fn main() {}")
            .expect("internal error");
        std::fs::write(
            project_root.join("node_modules/lib/index.js"),
            "console.log('test');",
        )
        .expect("internal error");
        std::fs::write(project_root.join(".git/hooks/pre-commit.sh"), "#!/bin/bash")
            .expect("internal error");
        std::fs::write(project_root.join("src/lib.rs"), "pub fn test() {}")
            .expect("internal error");

        let mut files = Vec::new();
        detector
            .collect_files_recursive(project_root, &mut files)
            .await
            .expect("internal error");

        assert_eq!(files.len(), 1); // Only src/lib.rs should be found
        assert!(files.iter().any(|f| f.ends_with("src/lib.rs")));
    }

    #[tokio::test]
    async fn test_collect_files_recursive_skips_test_files() {
        let detector = SATDDetector::new();
        let temp_dir = tempfile::tempdir().expect("internal error");
        let project_root = temp_dir.path();

        // Create test files and regular files
        std::fs::create_dir_all(project_root.join("src")).expect("internal error");
        std::fs::create_dir_all(project_root.join("tests")).expect("internal error");

        std::fs::write(project_root.join("src/lib.rs"), "pub fn func() {}")
            .expect("internal error");
        std::fs::write(project_root.join("src/main_test.rs"), "fn test_main() {}")
            .expect("internal error");
        std::fs::write(
            project_root.join("tests/integration.rs"),
            "#[test] fn test() {}",
        )
        .expect("internal error");

        let mut files = Vec::new();
        detector
            .collect_files_recursive(project_root, &mut files)
            .await
            .expect("internal error");

        // Should only find lib.rs, not test files
        assert_eq!(files.len(), 1);
        assert!(files.iter().any(|f| f.ends_with("src/lib.rs")));
        assert!(!files.iter().any(|f| f.to_string_lossy().contains("test")));
    }

    #[tokio::test]
    async fn test_collect_files_recursive_nested_directories() {
        let detector = SATDDetector::new();
        let temp_dir = tempfile::tempdir().expect("internal error");
        let project_root = temp_dir.path();

        // Create nested directory structure
        std::fs::create_dir_all(project_root.join("src/utils/helpers")).expect("internal error");
        std::fs::create_dir_all(project_root.join("src/models")).expect("internal error");

        std::fs::write(project_root.join("src/main.rs"), "fn main() {}").expect("internal error");
        std::fs::write(project_root.join("src/utils/mod.rs"), "pub mod helpers;")
            .expect("internal error");
        std::fs::write(
            project_root.join("src/utils/helpers/string.rs"),
            "pub fn trim() {}",
        )
        .expect("internal error");
        std::fs::write(
            project_root.join("src/models/user.rs"),
            "pub struct User {}",
        )
        .expect("internal error");

        let mut files = Vec::new();
        detector
            .collect_files_recursive(project_root, &mut files)
            .await
            .expect("internal error");

        assert_eq!(files.len(), 4);
        assert!(files.iter().any(|f| f.ends_with("main.rs")));
        assert!(files.iter().any(|f| f.ends_with("mod.rs")));
        assert!(files.iter().any(|f| f.ends_with("string.rs")));
        assert!(files.iter().any(|f| f.ends_with("user.rs")));
    }

    /// RED TEST: Toyota Way - Stop the Line
    /// Markdown headers (### Security, ## Security, # Security) should NOT be flagged as SATD
    /// Found bug: changelog_manager.rs line 133 "### Security" flagged as Critical Security SATD
    #[tokio::test]
    async fn test_markdown_headers_not_flagged_as_satd() {
        let detector = SATDDetector::new();
        let temp_dir = TempDir::new().expect("internal error");

        // Test case 1: CHANGELOG template with ### Security header
        let changelog_template = r#"
# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security
"#;

        let changelog_file = temp_dir.path().join("CHANGELOG.md");
        fs::write(&changelog_file, changelog_template).expect("internal error");

        // Test case 2: Rust code with CHANGELOG template string literal
        let changelog_manager_code = r###"
const CHANGELOG_TEMPLATE: &str = r#"
### Added

### Security
"#;
"###;
        let manager_file = temp_dir
            .path()
            .join("changelog_manager")
            .with_extension("rs");
        fs::write(&manager_file, changelog_manager_code).expect("internal error");

        let result = detector
            .analyze_project(temp_dir.path(), false)
            .await
            .expect("internal error");

        // RED: This will FAIL initially - markdown headers are currently detected as SATD
        // Expected: 0 Security SATD items (markdown headers should be filtered)
        // Actual: 2 Security SATD items (false positives)
        let security_count = result
            .items
            .iter()
            .filter(|item| matches!(item.category, DebtCategory::Security))
            .count();

        assert_eq!(
            security_count, 0,
            "Markdown headers like ### Security should NOT be flagged as SATD. Found {} false positives",
            security_count
        );
    }

    /// RED TEST: Bug tracking ID references should NOT be flagged as SATD
    /// Real-world patterns from codebase:
    /// - "BUG-012: Apply language override if specified"
    /// - "BUG-064 FIX: Uses atomic write operations"
    /// - "Bug: Previously used walkdir directly"
    /// - "PMAT-BUG-001: TypeScript detection must work"
    #[tokio::test]
    async fn test_bug_tracking_ids_not_flagged_as_satd() {
        let detector = SATDDetector::new();
        let temp_dir = TempDir::new().expect("internal error");

        // Test case 1: Bug tracking IDs (like JIRA tickets)
        let bug_tracking_code = r#"
    // BUG-012: Apply language override if specified
    let override_opts = LanguageOverride {
        language,
        languages,
    };

    // BUG-064 FIX: Uses atomic write operations to prevent file corruption
    fn atomic_write(path: &Path, content: &str) -> Result<()> {
        Ok(())
    }

    // PMAT-BUG-001: TypeScript class methods must be extracted
    // Root cause: JavaScriptAnalyzer uses regex/heuristic parsing
    fn extract_methods() {}
"#;
        let tracking_file = temp_dir.path().join("tracking").with_extension("rs");
        fs::write(&tracking_file, bug_tracking_code).expect("internal error");

        // Test case 2: Fixed bug descriptions
        let fixed_bug_code = r#"
    // Bug: Previously used walkdir directly, bypassing ignore file support
    let discovery_config = FileDiscoveryConfig {
        respect_gitignore: true,
    };

    // CRITICAL FIX: Use ProjectFileDiscovery instead of WalkDir
    // This ensures .pmatignore and .paimlignore files are respected
    // Bug: Previously used walkdir directly
    fn use_project_discovery() {}
"#;
        let fixed_file = temp_dir.path().join("fixed").with_extension("rs");
        fs::write(&fixed_file, fixed_bug_code).expect("internal error");

        // Test case 3: Bug-related functionality descriptions
        let functionality_code = r#"
// Bug fix patterns
fn extract_patterns() {
    // This describes functionality for detecting bug fix commits
}

// Extract bug fix claims
fn analyze_commits() {
    // Extracting bug information from commit messages
}

/// Computes volume, difficulty, effort, programming time, and bug estimates
fn halstead_metrics() {}
"#;
        let functionality_file = temp_dir.path().join("functionality").with_extension("rs");
        fs::write(&functionality_file, functionality_code).expect("internal error");

        let result = detector
            .analyze_project(temp_dir.path(), false)
            .await
            .expect("internal error");

        // All these comments describe bug tracking IDs, fixed bugs, or bug-related functionality
        // They are NOT self-admitted technical debt (TODO/FIXME for future work)
        let defect_count = result
            .items
            .iter()
            .filter(|item| matches!(item.category, DebtCategory::Defect))
            .count();

        assert_eq!(
            defect_count, 0,
            "Bug tracking IDs and fixed bug descriptions should NOT be flagged as SATD. Found {} false positives",
            defect_count
        );
    }
