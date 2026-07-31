// Unit tests and property tests for incremental coverage handler.
// Included by incremental_coverage_handler.rs — do NOT add `use` imports here.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::facades::incremental_coverage_facade::{
        ChangedFileCoverage, CoverageStatus,
    };

    fn create_test_result() -> IncrementalCoverageResult {
        IncrementalCoverageResult {
            total_files: 10,
            covered_files: 8,
            coverage_percentage: Some(80.0),
            files_above_threshold: 6,
            files_below_threshold: 4,
            files_not_measured: 0,
            changed_files: vec![
                ChangedFileCoverage {
                    file_path: "src/lib.rs".to_string(),
                    status: CoverageStatus::Improved,
                    coverage_before: Some(70.0),
                    coverage_after: Some(85.0),
                    coverage_delta: Some(15.0),
                    lines_covered: 85,
                    lines_total: 100,
                },
                ChangedFileCoverage {
                    file_path: "src/main.rs".to_string(),
                    status: CoverageStatus::New,
                    coverage_before: Some(0.0),
                    coverage_after: Some(90.0),
                    coverage_delta: Some(90.0),
                    lines_covered: 45,
                    lines_total: 50,
                },
            ],
            summary: "Test summary".to_string(),
        }
    }

    #[test]
    fn test_format_summary() {
        let result = create_test_result();
        let output = format_summary(&result, 5);
        assert!(output.contains("Test summary"));
        assert!(output.contains("# Incremental Coverage Summary"));
        assert!(output.contains("Top Changed Files"));
        assert!(output.contains("src/lib.rs"));
    }

    #[test]
    fn test_format_summary_empty() {
        let result = IncrementalCoverageResult {
            total_files: 0,
            covered_files: 0,
            coverage_percentage: Some(0.0),
            files_above_threshold: 0,
            files_below_threshold: 0,
            files_not_measured: 0,
            changed_files: vec![],
            summary: "Empty project".to_string(),
        };
        let output = format_summary(&result, 5);
        assert!(output.contains("Empty project"));
    }

    #[test]
    fn test_format_detailed() {
        let result = create_test_result();
        let output = format_detailed(&result, 5);
        assert!(output.contains("# Incremental Coverage Detailed Report"));
        assert!(output.contains("Total files analyzed: 10"));
        assert!(output.contains("Files with coverage: 8"));
        assert!(output.contains("Overall coverage: 80.0%"));
        assert!(output.contains("src/lib.rs"));
        assert!(output.contains("Improved"));
    }

    #[test]
    fn test_format_markdown() {
        let result = create_test_result();
        let output = format_markdown(&result, 5);
        assert!(output.contains("# Incremental Coverage Report"));
        assert!(output.contains("**Summary:**"));
        assert!(output.contains("## Metrics"));
        assert!(output.contains("| Metric | Value |"));
    }

    #[test]
    fn test_format_lcov() {
        let result = create_test_result();
        let output = format_lcov(&result);
        assert!(output.contains("SF:src/lib.rs"));
        assert!(output.contains("LH:85"));
        assert!(output.contains("LF:100"));
        assert!(output.contains("end_of_record"));
        // The old renderer emitted `DA:<lines_total>,<lines_covered>` — a DA
        // record is "line N was hit C times", so that was a malformed claim
        // about a line number that need not exist. Dropped rather than faked.
        assert!(!output.contains("DA:"));
    }

    /// GH #658: an unmeasured file must not appear in LCOV at all — `LH:0/LF:0`
    /// reads as "nothing is covered" rather than "we did not measure".
    #[test]
    fn lcov_omits_unmeasured_files() {
        let mut result = create_test_result();
        result.changed_files.push(ChangedFileCoverage {
            file_path: "src/unmeasured.rs".to_string(),
            status: CoverageStatus::NotMeasured,
            coverage_before: None,
            coverage_after: None,
            coverage_delta: None,
            lines_covered: 0,
            lines_total: 0,
        });

        let output = format_lcov(&result);
        assert!(output.contains("SF:src/lib.rs"));
        assert!(!output.contains("src/unmeasured.rs"));
    }

    #[test]
    fn test_format_delta() {
        let result = create_test_result();
        let output = format_delta(&result, 5);
        assert!(output.contains("Coverage Delta Report"));
        assert!(output.contains("Improved Coverage"));
        assert!(output.contains("src/lib.rs"));
    }

    #[test]
    fn test_format_sarif() {
        let result = create_test_result();
        let output = format_sarif(&result);
        assert!(output.contains("version"));
        assert!(output.contains("2.1.0"));
        assert!(output.contains("results"));
    }

    #[test]
    fn test_format_result_json() {
        let result = create_test_result();
        let output = format_result(result, IncrementalCoverageOutputFormat::Json, 5);
        assert!(output.is_ok());
        let json = output.unwrap();
        assert!(json.contains("total_files"));
        assert!(json.contains("coverage_percentage"));
    }

    #[test]
    fn test_format_result_summary() {
        let result = create_test_result();
        let output = format_result(result, IncrementalCoverageOutputFormat::Summary, 5);
        assert!(output.is_ok());
        assert!(output.unwrap().contains("# Incremental Coverage Summary"));
    }

    #[test]
    fn test_format_result_detailed() {
        let result = create_test_result();
        let output = format_result(result, IncrementalCoverageOutputFormat::Detailed, 5);
        assert!(output.is_ok());
        assert!(output
            .unwrap()
            .contains("# Incremental Coverage Detailed Report"));
    }

    #[test]
    fn test_format_result_markdown() {
        let result = create_test_result();
        let output = format_result(result, IncrementalCoverageOutputFormat::Markdown, 5);
        assert!(output.is_ok());
        assert!(output.unwrap().contains("# Incremental Coverage Report"));
    }

    #[test]
    fn test_format_result_lcov() {
        let result = create_test_result();
        let output = format_result(result, IncrementalCoverageOutputFormat::Lcov, 5);
        assert!(output.is_ok());
        assert!(output.unwrap().contains("SF:"));
    }

    #[test]
    fn test_format_result_delta() {
        let result = create_test_result();
        let output = format_result(result, IncrementalCoverageOutputFormat::Delta, 5);
        assert!(output.is_ok());
    }

    #[test]
    fn test_format_result_sarif() {
        let result = create_test_result();
        let output = format_result(result, IncrementalCoverageOutputFormat::Sarif, 5);
        assert!(output.is_ok());
    }

    #[test]
    fn test_incremental_coverage_config_clone() {
        let config = IncrementalCoverageConfig {
            project_path: PathBuf::from("/test"),
            base_branch: "main".to_string(),
            target_branch: Some("feature".to_string()),
            format: IncrementalCoverageOutputFormat::Summary,
            coverage_threshold: 80.0,
            changed_files_only: true,
            detailed: false,
            output: None,
            perf: false,
            cache_dir: None,
            force_refresh: false,
            top_files: 10,
        };
        let cloned = config.clone();
        assert_eq!(cloned.base_branch, "main");
        // Percentage, matching `--coverage-threshold`'s documented units (#658).
        assert_eq!(cloned.coverage_threshold, 80.0);
    }

    #[test]
    fn test_incremental_coverage_config_debug() {
        let config = IncrementalCoverageConfig {
            project_path: PathBuf::from("/test"),
            base_branch: "main".to_string(),
            target_branch: None,
            format: IncrementalCoverageOutputFormat::Json,
            coverage_threshold: 75.0,
            changed_files_only: false,
            detailed: true,
            output: Some(PathBuf::from("output.json")),
            perf: true,
            cache_dir: Some(PathBuf::from(".cache")),
            force_refresh: true,
            top_files: 5,
        };
        let debug = format!("{:?}", config);
        assert!(debug.contains("IncrementalCoverageConfig"));
        assert!(debug.contains("main"));
    }
}

