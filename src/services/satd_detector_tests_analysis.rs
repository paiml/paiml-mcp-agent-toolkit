    #[tokio::test]
    async fn test_analyze_directory_empty() {
        let temp_dir = TempDir::new().expect("internal error");
        let detector = SATDDetector::new();

        let result = detector.analyze_directory(temp_dir.path()).await;
        assert!(result.is_ok());

        let debts = result.expect("internal error");
        assert_eq!(debts.len(), 0);
    }

    #[tokio::test]
    async fn test_analyze_directory_with_rust_files() {
        let temp_dir = TempDir::new().expect("internal error");
        let detector = SATDDetector::new();

        // Create files without "test" in their names
        let file1 = temp_dir.path().join("file1.rs");
        fs::write(&file1, "// TODO: Test debt in file 1\nfn main() {}").expect("internal error");

        let file2 = temp_dir.path().join("file2.rs");
        fs::write(&file2, "// FIXME: Test debt in file 2\nfn helper() {}").expect("internal error");

        let result = detector.analyze_directory(temp_dir.path()).await;
        assert!(result.is_ok());

        let debts = result.expect("internal error");
        assert_eq!(debts.len(), 2);
    }

    #[tokio::test]
    async fn test_analyze_directory_ignores_non_source_files() {
        let temp_dir = TempDir::new().expect("internal error");
        let detector = SATDDetector::new();

        // Create source file with debt
        let rust_file = temp_dir.path().join("source.rs");
        fs::write(&rust_file, "// TODO: This should be found").expect("internal error");

        // Create non-source file with debt (should be ignored)
        let text_file = temp_dir.path().join("readme.txt");
        fs::write(&text_file, "TODO: This should be ignored").expect("internal error");

        let result = detector.analyze_directory(temp_dir.path()).await;
        assert!(result.is_ok());

        let debts = result.expect("internal error");
        assert_eq!(debts.len(), 1); // Only the .rs file should be analyzed
        assert!(debts[0].file.ends_with("source.rs"));
    }

    #[test]
    fn test_generate_metrics_edge_cases() {
        let detector = SATDDetector::new();

        // Test with empty debt list
        let empty_debts = vec![];
        let metrics = detector.generate_metrics(&empty_debts, 1000);

        assert_eq!(metrics.total_debts, 0);
        assert_eq!(metrics.critical_debts.len(), 0);
        assert_eq!(metrics.debt_density_per_kloc, 0.0);
        assert_eq!(metrics.by_category.len(), 0);
        assert_eq!(metrics.debt_age_distribution.len(), 0);
    }

    #[test]
    fn test_generate_metrics_with_mixed_severities() {
        let detector = SATDDetector::new();

        let debts = vec![
            create_test_debt(DebtCategory::Design, Severity::Low),
            create_test_debt(DebtCategory::Design, Severity::Medium),
            create_test_debt(DebtCategory::Defect, Severity::High),
            create_test_debt(DebtCategory::Defect, Severity::Critical),
        ];

        let metrics = detector.generate_metrics(&debts, 2000);

        assert_eq!(metrics.total_debts, 4);
        assert_eq!(metrics.critical_debts.len(), 1); // Only Critical severity
        assert_eq!(metrics.debt_density_per_kloc, 2.0); // 4 debts per 2 KLOC
        assert_eq!(metrics.by_category.len(), 2); // Design and Defect

        // Check category breakdown
        let design_metrics = metrics.by_category.get("Design").expect("internal error");
        assert_eq!(design_metrics.count, 2);
        assert!((design_metrics.avg_severity - 1.5).abs() < 0.1); // (1+2)/2 = 1.5

        let defect_metrics = metrics.by_category.get("Defect").expect("internal error");
        assert_eq!(defect_metrics.count, 2);
        assert!((defect_metrics.avg_severity - 3.5).abs() < 0.1); // (3+4)/2 = 3.5
    }

    // TDD RED phase - Tests for calculate_average_debt_age (37 cognitive complexity)
    #[tokio::test]
    async fn test_calculate_average_debt_age_empty_debts() {
        let detector = SATDDetector::new();
        let temp_dir = tempfile::tempdir().expect("internal error");
        let project_root = temp_dir.path();

        let result = detector
            .calculate_average_debt_age(&[], project_root)
            .await
            .expect("internal error");
        assert_eq!(result, 0.0);
    }

    #[tokio::test]
    async fn test_calculate_average_debt_age_no_git() {
        let detector = SATDDetector::new();
        let temp_dir = tempfile::tempdir().expect("internal error");
        let project_root = temp_dir.path();

        // Create a test file
        let test_file = project_root.join("test.rs");
        std::fs::write(&test_file, "// TODO: test debt").expect("internal error");

        let debts = vec![create_test_debt_with_file(
            DebtCategory::Design,
            Severity::Medium,
            test_file.clone(),
            1,
        )];

        let result = detector
            .calculate_average_debt_age(&debts, project_root)
            .await
            .expect("internal error");
        assert_eq!(result, 0.0); // No git history, should default to 0
    }

    #[tokio::test]
    async fn test_calculate_average_debt_age_invalid_file_path() {
        let detector = SATDDetector::new();
        let temp_dir = tempfile::tempdir().expect("internal error");
        let project_root = temp_dir.path();

        // Create debt with path outside project root
        let external_file = PathBuf::from("/external/file.rs");
        let debts = vec![create_test_debt_with_file(
            DebtCategory::Design,
            Severity::Medium,
            external_file,
            1,
        )];

        let result = detector
            .calculate_average_debt_age(&debts, project_root)
            .await
            .expect("internal error");
        assert_eq!(result, 0.0); // External files should be skipped
    }

    // Helper function for debt with custom file
    fn create_test_debt_with_file(
        category: DebtCategory,
        severity: Severity,
        file: PathBuf,
        line: u32,
    ) -> TechnicalDebt {
        TechnicalDebt {
            text: "test debt".to_string(),
            category,
            severity,
            file,
            line,
            column: 1,
            context_hash: [0; 16], // Test hash
        }
    }

    // TDD RED phase - Tests for extract_from_content (30 cognitive complexity)
    #[test]
    fn test_extract_from_content_complex_test_blocks() {
        let detector = SATDDetector::new();

        let content = r#"
// TODO: regular debt
fn main() {
    #[cfg_attr(coverage_nightly, coverage(off))]
    #[cfg(test)]
    mod nested_tests {
        // TODO: should be ignored
        #[test] 
        fn test_with_nested_blocks() {
            if true {
                // FIXME: nested ignored
                let x = {
                    // TODO: deeply nested ignored
                    42
                };
            }
        }
    }
    // TODO: after test block
}
        "#;

        let debts = detector
            .extract_from_content(content, Path::new("test.rs"))
            .expect("internal error");

        // Should only find debts outside test blocks
        assert_eq!(debts.len(), 2);
        assert!(debts.iter().any(|d| d.text.contains("regular debt")));
        assert!(debts.iter().any(|d| d.text.contains("after test block")));
        assert!(!debts.iter().any(|d| d.text.contains("should be ignored")));
        assert!(!debts.iter().any(|d| d.text.contains("nested ignored")));
        assert!(!debts
            .iter()
            .any(|d| d.text.contains("deeply nested ignored")));
    }

    #[test]
    fn test_extract_from_content_non_rust_files() {
        let detector = SATDDetector::new();

        let content = r#"
// TODO: python debt
#[cfg(test)]  // This should not be treated as test block in Python
def test_something():
    # TODO: python test debt should be found
    pass
        "#;

        let debts = detector
            .extract_from_content(content, Path::new("test.py"))
            .expect("internal error");

        // Python files don't have Rust test block logic
        assert_eq!(debts.len(), 2);
        assert!(debts.iter().any(|d| d.text.contains("python debt")));
        assert!(debts.iter().any(|d| d.text.contains("python test debt")));
    }
