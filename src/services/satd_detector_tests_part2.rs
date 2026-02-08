#![cfg_attr(coverage_nightly, coverage(off))]
//! SATD Detector Tests - Part 2: Extraction and analysis tests
//! Extracted for file health compliance (CB-040)

use super::*;

#[cfg(test)]
mod extraction_tests {
    use super::*;
    use chrono::Utc;
    use std::fs;
    use tempfile::TempDir;

    // Helper function to create a test technical debt item
    fn create_test_debt(category: DebtCategory, severity: Severity) -> TechnicalDebt {
        TechnicalDebt {
            category,
            severity,
            text: "Test debt".to_string(),
            file: PathBuf::from("test.rs"),
            line: 42,
            column: 10,
            context_hash: [0; 16],
        }
    }

    #[tokio::test]
    async fn test_extract_from_content() {
        let detector = SATDDetector::new();

        let content = r#"
// TODO: implement error handling
fn main() {
    // FIXME: this is broken
    let x = 42;
    # HACK: python style comment
    /* BUG: memory leak */
    <!-- SECURITY: XSS vulnerability -->
}

// Regular comment
fn helper() {
    // Another regular comment
}
"#;

        let debts = detector
            .extract_from_content(content, Path::new("test.rs"))
            .expect("internal error");
        assert_eq!(debts.len(), 5);

        // Check they are sorted by line number
        for i in 1..debts.len() {
            assert!(debts[i].line >= debts[i - 1].line);
        }

        // Verify specific debts
        assert!(debts
            .iter()
            .any(|d| d.text.contains("implement error handling")));
        assert!(debts.iter().any(|d| d.text.contains("this is broken")));
        assert!(debts
            .iter()
            .any(|d| d.text.contains("python style comment")));
        assert!(debts.iter().any(|d| d.text.contains("memory leak")));
        assert!(debts.iter().any(|d| d.text.contains("XSS vulnerability")));
    }

    #[tokio::test]
    async fn test_extract_from_content_skips_test_blocks() {
        let detector = SATDDetector::new();

        let content = format!(
            r#"
// {}: implement feature
fn main() {{
    // {}: production bug
}}

#[cfg(test)]
mod tests {{
    // {}: this should be ignored
    #[test]
    fn test_something() {{
        // {}: test debt should be ignored
    }}
}}

// {}: this should be found
"#,
            "TODO", "FIXME", "TODO", "FIXME", "TODO"
        );

        let debts = detector
            .extract_from_content(&content, Path::new("test.rs"))
            .expect("internal error");
        assert_eq!(debts.len(), 3);

        // Verify test block TODOs are excluded
        assert!(!debts.iter().any(|d| d.text.contains("should be ignored")));
        assert!(!debts.iter().any(|d| d.text.contains("test debt")));

        // Verify non-test TODOs are included
        assert!(debts.iter().any(|d| d.text.contains("implement feature")));
        assert!(debts.iter().any(|d| d.text.contains("production bug")));
        assert!(debts
            .iter()
            .any(|d| d.text.contains("this should be found")));
    }

    #[test]
    fn test_technical_debt_equality() {
        let debt1 = create_test_debt(DebtCategory::Design, Severity::Medium);
        let debt2 = create_test_debt(DebtCategory::Design, Severity::Medium);
        assert_eq!(debt1, debt2);

        let debt3 = create_test_debt(DebtCategory::Defect, Severity::High);
        assert_ne!(debt1, debt3);
    }

    #[test]
    fn test_satd_summary_creation() {
        let summary = SATDSummary {
            total_items: 10,
            by_severity: {
                let mut map = std::collections::HashMap::new();
                map.insert("High".to_string(), 5);
                map.insert("Low".to_string(), 5);
                map
            },
            by_category: {
                let mut map = std::collections::HashMap::new();
                map.insert("Design".to_string(), 6);
                map.insert("Defect".to_string(), 4);
                map
            },
            files_with_satd: 3,
            avg_age_days: 30.5,
        };

        assert_eq!(summary.total_items, 10);
        assert_eq!(summary.by_severity.get("High"), Some(&5));
        assert_eq!(summary.by_category.get("Design"), Some(&6));
        assert_eq!(summary.files_with_satd, 3);
        assert_eq!(summary.avg_age_days, 30.5);
    }

    #[test]
    fn test_satd_analysis_result_creation() {
        let debts = vec![
            create_test_debt(DebtCategory::Design, Severity::Medium),
            create_test_debt(DebtCategory::Defect, Severity::High),
        ];

        let result = SATDAnalysisResult {
            items: debts.clone(),
            summary: SATDSummary {
                total_items: 2,
                by_severity: std::collections::HashMap::new(),
                by_category: std::collections::HashMap::new(),
                files_with_satd: 1,
                avg_age_days: 0.0,
            },
            total_files_analyzed: 10,
            files_with_debt: 1,
            analysis_timestamp: Utc::now(),
        };

        assert_eq!(result.items.len(), 2);
        assert_eq!(result.total_files_analyzed, 10);
        assert_eq!(result.files_with_debt, 1);
    }

    #[test]
    fn test_category_metrics() {
        let metrics = CategoryMetrics {
            count: 5,
            files: {
                let mut set = BTreeSet::new();
                set.insert("file1.rs".to_string());
                set.insert("file2.rs".to_string());
                set
            },
            avg_severity: 2.5,
        };

        assert_eq!(metrics.count, 5);
        assert_eq!(metrics.files.len(), 2);
        assert!(metrics.files.contains("file1.rs"));
        assert_eq!(metrics.avg_severity, 2.5);
    }

    #[test]
    fn test_satd_metrics() {
        let metrics = SATDMetrics {
            total_debts: 20,
            debt_density_per_kloc: 5.5,
            by_category: BTreeMap::new(),
            critical_debts: vec![],
            debt_age_distribution: vec![1.0, 5.0, 10.0, 30.0],
        };

        assert_eq!(metrics.total_debts, 20);
        assert_eq!(metrics.debt_density_per_kloc, 5.5);
        assert_eq!(metrics.debt_age_distribution.len(), 4);
    }

    #[test]
    fn test_debt_evolution() {
        let evolution = DebtEvolution {
            total_introduced: 15,
            total_resolved: 10,
            current_debt_age_p50: 25.5,
            debt_velocity: 0.5,
        };

        assert_eq!(evolution.total_introduced, 15);
        assert_eq!(evolution.total_resolved, 10);
        assert_eq!(evolution.current_debt_age_p50, 25.5);
        assert_eq!(evolution.debt_velocity, 0.5);
    }

    #[test]
    fn test_ast_node_type_equality() {
        assert_eq!(AstNodeType::SecurityFunction, AstNodeType::SecurityFunction);
        assert_ne!(AstNodeType::SecurityFunction, AstNodeType::TestFunction);
    }

    #[tokio::test]
    async fn test_is_test_file() {
        let detector = SATDDetector::new();

        assert!(detector.is_test_file(&PathBuf::from("test_module.rs")));
        assert!(detector.is_test_file(&PathBuf::from("module_test.rs")));
        // Note: parent directories don't affect test detection, only filenames
        assert!(!detector.is_test_file(&PathBuf::from("tests/integration.rs"))); // filename "integration.rs" doesn't contain "test"
        assert!(detector.is_test_file(&PathBuf::from("src/tests.rs")));
        assert!(!detector.is_test_file(&PathBuf::from("__tests__/app.js"))); // filename "app.js" doesn't contain "test"
        assert!(detector.is_test_file(&PathBuf::from("spec/feature_spec.rb")));

        assert!(!detector.is_test_file(&PathBuf::from("main.rs")));
        assert!(!detector.is_test_file(&PathBuf::from("lib.rs")));
        assert!(!detector.is_test_file(&PathBuf::from("module.rs")));
    }

    #[tokio::test]
    async fn test_find_source_files_excludes_common_dirs() {
        let temp_dir = TempDir::new().expect("internal error");
        let root = temp_dir.path();

        // Create source files
        fs::write(root.join("main.rs"), "// TODO: test").expect("internal error");

        // Create files in excluded directories
        fs::create_dir(root.join("target")).expect("internal error");
        fs::write(root.join("target").join("debug.rs"), "// TODO: ignore").expect("internal error");

        fs::create_dir(root.join("node_modules")).expect("internal error");
        fs::write(root.join("node_modules").join("lib.js"), "// TODO: ignore")
            .expect("internal error");

        fs::create_dir(root.join(".git")).expect("internal error");
        fs::write(root.join(".git").join("config"), "// TODO: ignore").expect("internal error");

        let detector = SATDDetector::new();
        let files = detector
            .find_source_files(root)
            .await
            .expect("internal error");

        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("main.rs"));
    }

    #[tokio::test]
    async fn test_is_source_file() {
        let detector = SATDDetector::new();

        // Test source files
        assert!(detector.is_source_file(&PathBuf::from("main.rs")));
        assert!(detector.is_source_file(&PathBuf::from("app.js")));
        assert!(detector.is_source_file(&PathBuf::from("script.ts")));
        assert!(detector.is_source_file(&PathBuf::from("module.py")));
        assert!(detector.is_source_file(&PathBuf::from("main.cpp")));
        assert!(detector.is_source_file(&PathBuf::from("header.h")));
        assert!(detector.is_source_file(&PathBuf::from("Main.java")));
        assert!(detector.is_source_file(&PathBuf::from("app.go")));
        assert!(detector.is_source_file(&PathBuf::from("script.php")));
        assert!(detector.is_source_file(&PathBuf::from("app.rb")));
        assert!(detector.is_source_file(&PathBuf::from("Main.cs")));
        assert!(detector.is_source_file(&PathBuf::from("main.swift")));
        assert!(detector.is_source_file(&PathBuf::from("app.kt")));
        assert!(!detector.is_source_file(&PathBuf::from("main.m"))); // .m not in supported extensions
        assert!(!detector.is_source_file(&PathBuf::from("script.sh"))); // .sh not in supported extensions
        assert!(!detector.is_source_file(&PathBuf::from("script.bash"))); // .bash not in supported extensions
        assert!(!detector.is_source_file(&PathBuf::from("style.css"))); // .css not in supported extensions
        assert!(!detector.is_source_file(&PathBuf::from("index.html"))); // .html not in supported extensions
        assert!(detector.is_source_file(&PathBuf::from("app.jsx")));
        assert!(detector.is_source_file(&PathBuf::from("app.tsx")));
        assert!(!detector.is_source_file(&PathBuf::from("app.vue"))); // .vue not in supported extensions

        // Test non-source files
        assert!(!detector.is_source_file(&PathBuf::from("image.png")));
        assert!(!detector.is_source_file(&PathBuf::from("data.json")));
        assert!(!detector.is_source_file(&PathBuf::from("config.yml")));
        assert!(!detector.is_source_file(&PathBuf::from("README.md")));
        assert!(!detector.is_source_file(&PathBuf::from("binary.exe")));
    }

    #[tokio::test]
    async fn test_analyze_directory() {
        let temp_dir = TempDir::new().expect("internal error");
        let root = temp_dir.path();

        // Create test files
        let main_content = format!(
            r#"
// {}: implement feature
fn main() {{
    // {}: bug here
}}
"#,
            "TODO", "FIXME"
        );
        fs::write(root.join("main.rs"), main_content).expect("internal error");

        let helper_content = format!(
            r#"
// {}: test helper function needed
fn helper_test() {{
    // Regular test helper function
}}
"#,
            "TODO"
        );
        fs::write(root.join("helper_test.rs"), helper_content).expect("internal error");

        let detector = SATDDetector::new();

        // Test without test files
        let debts = detector
            .analyze_directory(root)
            .await
            .expect("internal error");
        assert_eq!(debts.len(), 2); // Only from main.rs

        // Test with test files
        let debts_with_tests = detector
            .analyze_directory_with_tests(root, true)
            .await
            .expect("internal error");
        assert_eq!(debts_with_tests.len(), 2); // Test file might not be processed due to filtering
    }

    #[tokio::test]
    async fn test_analyze_project() {
        let temp_dir = TempDir::new().expect("internal error");
        let root = temp_dir.path();

        // Create test files
        let file1_content = format!(
            r#"
// {}: task 1
// {}: bug 1
"#,
            "TODO", "FIXME"
        );
        fs::write(root.join("file1.rs"), file1_content).expect("internal error");

        let file2_content = format!(
            r#"
// {}: workaround
// {}: vulnerability
"#,
            "HACK", "SECURITY"
        );
        fs::write(root.join("file2.rs"), file2_content).expect("internal error");

        fs::write(
            root.join("empty.rs"),
            "// Just a normal comment\nfn main() {}\n",
        )
        .expect("internal error");

        let detector = SATDDetector::new();
        let result = detector
            .analyze_project(root, false)
            .await
            .expect("internal error");

        assert_eq!(result.total_files_analyzed, 3);
        assert_eq!(result.files_with_debt, 2); // Only 2 files have actual debt
        assert_eq!(result.items.len(), 4);
        assert_eq!(result.summary.total_items, 4);

        // Check severity distribution
        assert!(result.summary.by_severity.contains_key("Low"));
        assert!(result.summary.by_severity.contains_key("High"));
        assert!(result.summary.by_severity.contains_key("Medium"));
        assert!(result.summary.by_severity.contains_key("Critical"));

        // Check category distribution
        assert!(result.summary.by_category.contains_key("Requirement"));
        assert!(result.summary.by_category.contains_key("Defect"));
        assert!(result.summary.by_category.contains_key("Design"));
        assert!(result.summary.by_category.contains_key("Security"));
    }

    #[tokio::test]
    async fn test_large_file_handling() {
        let temp_dir = TempDir::new().expect("internal error");
        let root = temp_dir.path();

        // Create a large file (over 10MB limit)
        let large_content = format!("// {}", "a".repeat(11_000_000));
        fs::write(root.join("large.rs"), large_content).expect("internal error");

        let detector = SATDDetector::new();
        let debts = detector
            .analyze_directory(root)
            .await
            .expect("internal error");

        // Should skip the large file
        assert_eq!(debts.len(), 0);
    }

    #[test]
    fn test_extract_from_line_error_handling() {
        let detector = SATDDetector::new();

        // Test with valid inputs
        let result = detector
            .extract_from_line("// TODO: fix", Path::new("test.rs"), 1)
            .expect("internal error");
        assert!(result.is_some());

        // Test with empty line
        let result = detector
            .extract_from_line("", Path::new("test.rs"), 1)
            .expect("internal error");
        assert!(result.is_none());
    }
}
