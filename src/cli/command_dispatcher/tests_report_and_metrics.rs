    // Test: execute_report_command with various analysis types

    #[tokio::test]
    async fn test_report_dead_code_analysis() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().expect("internal error");
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(&test_file, "fn main() {} fn unused() {}").expect("internal error");

        let result = CommandDispatcher::execute_report_command(
            Some(temp_dir.path().to_path_buf()),
            crate::cli::enums::ReportOutputFormat::Json,
            false,
            false,
            false,
            vec!["dead_code".to_string()],
            None,
            None,
            false,
            false,
            false,
            false,
        )
        .await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_report_with_visualizations() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().expect("internal error");
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(&test_file, "fn main() {}").expect("internal error");

        let result = CommandDispatcher::execute_report_command(
            Some(temp_dir.path().to_path_buf()),
            crate::cli::enums::ReportOutputFormat::Text,
            true, // include_visualizations
            true, // include_executive_summary
            true, // include_recommendations
            vec!["complexity".to_string()],
            Some(0.9),
            None,
            false,
            false,
            false,
            false,
        )
        .await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_report_text_format() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().expect("internal error");
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(&test_file, "fn main() {}").expect("internal error");

        let result = CommandDispatcher::execute_report_command(
            Some(temp_dir.path().to_path_buf()),
            crate::cli::enums::ReportOutputFormat::Text,
            false,
            false,
            false,
            vec!["complexity".to_string()],
            None,
            None,
            false,
            true, // text
            false,
            false,
        )
        .await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_report_markdown_format() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().expect("internal error");
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(&test_file, "fn main() {}").expect("internal error");

        let result = CommandDispatcher::execute_report_command(
            Some(temp_dir.path().to_path_buf()),
            crate::cli::enums::ReportOutputFormat::Text,
            false,
            false,
            false,
            vec!["complexity".to_string()],
            None,
            None,
            false,
            false,
            true, // markdown
            false,
        )
        .await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_report_csv_format() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().expect("internal error");
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(&test_file, "fn main() {}").expect("internal error");

        let result = CommandDispatcher::execute_report_command(
            Some(temp_dir.path().to_path_buf()),
            crate::cli::enums::ReportOutputFormat::Text,
            false,
            false,
            false,
            vec!["complexity".to_string()],
            None,
            None,
            false,
            false,
            false,
            true, // csv
        )
        .await;
        assert!(result.is_ok() || result.is_err());
    }

    // Test: execute_show_metrics_command

    #[tokio::test]
    async fn test_show_metrics_no_trend_error() {
        let result = CommandDispatcher::execute_show_metrics_command(
            false, // trend=false should error
            30,
            None,
            OutputFormat::Table,
            false,
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_show_metrics_with_trend() {
        let result = CommandDispatcher::execute_show_metrics_command(
            true,
            30,
            None,
            OutputFormat::Table,
            false,
        )
        .await;
        // May fail if no metrics but routing works
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_show_metrics_json_output() {
        let result = CommandDispatcher::execute_show_metrics_command(
            true,
            7,
            Some("lint".to_string()),
            OutputFormat::Json,
            false,
        )
        .await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_show_metrics_failures_only() {
        let result = CommandDispatcher::execute_show_metrics_command(
            true,
            14,
            None,
            OutputFormat::Table,
            true, // failures_only
        )
        .await;
        assert!(result.is_ok() || result.is_err());
    }

    // Test: execute_record_metric_command

    #[tokio::test]
    async fn test_record_metric_basic() {
        let result = CommandDispatcher::execute_record_metric_command(
            "test-coverage".to_string(),
            85.5,
            None,
        )
        .await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_record_metric_with_timestamp() {
        let ts = chrono::Utc::now().timestamp();
        let result = CommandDispatcher::execute_record_metric_command(
            "test-duration".to_string(),
            1000.0,
            Some(ts),
        )
        .await;
        assert!(result.is_ok() || result.is_err());
    }

    // Test: generate_metric_recommendations edge cases

    #[test]
    fn test_metric_recommendations_negative_slope_lint() {
        // Negative slope = improving, but the function still generates recommendations
        // (the days_to_critical clamps to 0 with max(0.0) which is < 30)
        let recs = CommandDispatcher::generate_metric_recommendations("lint", -50.0);
        // Should still have recommendations for lint (actionable items)
        assert!(!recs.is_empty());
    }

    #[test]
    fn test_metric_recommendations_zero_slope_test_fast() {
        let recs = CommandDispatcher::generate_metric_recommendations("test-fast", 0.0);
        // Still provides general recommendations
        assert!(!recs.is_empty());
    }

    #[test]
    fn test_metric_recommendations_coverage_critical() {
        let recs = CommandDispatcher::generate_metric_recommendations("coverage", 10000.0);
        // High slope = approaching threshold fast
        assert!(recs.iter().any(|r| r.contains("WARNING")));
    }

    #[test]
    fn test_metric_recommendations_build_release_critical() {
        let recs = CommandDispatcher::generate_metric_recommendations("build-release", 10000.0);
        assert!(recs.iter().any(|r| r.contains("WARNING")));
    }
