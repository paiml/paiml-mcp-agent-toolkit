#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod incremental_coverage_tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_file_metrics(path: &str, delta: f64) -> FileCoverageMetrics {
        FileCoverageMetrics {
            path: PathBuf::from(path),
            base_coverage: 70.0,
            target_coverage: 70.0 + delta,
            coverage_delta: delta,
            lines_added: 100,
            lines_covered: 70,
            lines_uncovered: 30,
        }
    }

    fn create_test_report() -> IncrementalCoverageReport {
        IncrementalCoverageReport {
            base_branch: "main".to_string(),
            target_branch: "feature".to_string(),
            coverage_threshold: 0.8,
            files: vec![
                create_test_file_metrics("src/improved.rs", 10.0),
                create_test_file_metrics("src/degraded.rs", -5.0),
                create_test_file_metrics("src/stable.rs", 0.0),
            ],
            summary: CoverageSummary {
                total_files_changed: 3,
                files_improved: 1,
                files_degraded: 1,
                overall_delta: 5.0,
                meets_threshold: true,
            },
        }
    }

    #[test]
    fn test_extract_filename() {
        assert_eq!(extract_filename(std::path::Path::new("src/main.rs")), "main.rs");
        assert_eq!(extract_filename(std::path::Path::new("a/b/c/test.rs")), "test.rs");
        assert_eq!(extract_filename(std::path::Path::new("single.rs")), "single.rs");
    }

    #[test]
    fn test_get_coverage_emoji() {
        assert_eq!(get_coverage_emoji(5.0), "📈");
        assert_eq!(get_coverage_emoji(-5.0), "📉");
        assert_eq!(get_coverage_emoji(0.0), "📉"); // 0 is not > 0
    }

    #[test]
    fn test_calculate_files_to_show() {
        let files = vec![
            create_test_file_metrics("a.rs", 1.0),
            create_test_file_metrics("b.rs", 2.0),
            create_test_file_metrics("c.rs", 3.0),
        ];

        assert_eq!(calculate_files_to_show(&files, 0), 3); // 0 means all
        assert_eq!(calculate_files_to_show(&files, 2), 2);
        assert_eq!(calculate_files_to_show(&files, 10), 3); // min of 10 and 3
    }

    #[test]
    fn test_write_coverage_header() {
        let report = create_test_report();
        let mut output = String::new();
        write_coverage_header(&mut output, &report).unwrap();

        assert!(output.contains("# Incremental Coverage Analysis"));
        assert!(output.contains("**Base Branch**: main"));
        assert!(output.contains("**Target Branch**: feature"));
        assert!(output.contains("✅ Yes"));
    }

    #[test]
    fn test_write_coverage_header_fails_threshold() {
        let mut report = create_test_report();
        report.summary.meets_threshold = false;
        let mut output = String::new();
        write_coverage_header(&mut output, &report).unwrap();

        assert!(output.contains("❌ No"));
    }

    #[test]
    fn test_write_coverage_summary() {
        let summary = CoverageSummary {
            total_files_changed: 5,
            files_improved: 3,
            files_degraded: 2,
            overall_delta: 2.5,
            meets_threshold: true,
        };
        let mut output = String::new();
        write_coverage_summary(&mut output, &summary).unwrap();

        assert!(output.contains("## Summary"));
        assert!(output.contains("Files Changed: 5"));
        assert!(output.contains("Files Improved: 3 📈"));
        assert!(output.contains("Files Degraded: 2 📉"));
    }

    #[test]
    fn test_format_incremental_coverage_summary() {
        let report = create_test_report();
        let output = format_incremental_coverage_summary(&report, 2).unwrap();

        assert!(output.contains("# Incremental Coverage Analysis"));
        assert!(output.contains("## Summary"));
        assert!(output.contains("## Top Files by Coverage Change"));
    }

    #[test]
    fn test_format_incremental_coverage_lcov() {
        let report = create_test_report();
        let output = format_incremental_coverage_lcov(&report).unwrap();

        assert!(output.contains("TN:"));
        assert!(output.contains("SF:"));
        assert!(output.contains("end_of_record"));
    }

    #[test]
    fn test_format_incremental_coverage_sarif() {
        let report = create_test_report();
        let output = format_incremental_coverage_sarif(&report).unwrap();

        assert!(output.contains("version"));
        assert!(output.contains("2.1.0"));
        assert!(output.contains("pmat-incremental-coverage"));
    }

    #[test]
    fn test_format_incremental_coverage_delta() {
        let report = create_test_report();
        let output = format_incremental_coverage_delta(&report, 10).unwrap();

        assert!(output.contains("Coverage Delta Report"));
    }

    #[test]
    fn test_file_coverage_metrics_struct() {
        let metrics = create_test_file_metrics("test.rs", 5.0);
        assert_eq!(metrics.path, PathBuf::from("test.rs"));
        assert!((metrics.coverage_delta - 5.0).abs() < 0.001);
        assert_eq!(metrics.lines_added, 100);
    }

    #[test]
    fn test_coverage_summary_struct() {
        let summary = CoverageSummary {
            total_files_changed: 10,
            files_improved: 7,
            files_degraded: 3,
            overall_delta: 4.5,
            meets_threshold: true,
        };
        assert_eq!(summary.total_files_changed, 10);
        assert!(summary.meets_threshold);
    }

    #[test]
    fn test_incremental_coverage_report_struct() {
        let report = create_test_report();
        assert_eq!(report.base_branch, "main");
        assert_eq!(report.target_branch, "feature");
        assert_eq!(report.files.len(), 3);
    }
}
