// Unit tests and property tests for AnalysisService
// Included from analysis_service.rs - shares parent scope (no use imports allowed)

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analysis_options_default() {
        let opts = AnalysisOptions::default();
        assert_eq!(opts.max_complexity, Some(20));
        assert!(!opts.include_tests);
        assert!(opts.parallel);
        assert!(matches!(opts.format, OutputFormat::Json));
    }

    #[test]
    fn test_analysis_service_new() {
        let service = AnalysisService::new();
        assert_eq!(service.name(), "AnalysisService");
    }

    #[test]
    fn test_validate_input_invalid_path() {
        let service = AnalysisService::new();
        let input = AnalysisInput {
            operation: AnalysisOperation::Complexity,
            path: PathBuf::from("/nonexistent/path"),
            options: AnalysisOptions::default(),
        };
        let result = service.validate_input(&input);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_input_invalid_complexity_zero() {
        let service = AnalysisService::new();
        let input = AnalysisInput {
            operation: AnalysisOperation::Complexity,
            path: PathBuf::from("."),
            options: AnalysisOptions {
                max_complexity: Some(0),
                ..Default::default()
            },
        };
        let result = service.validate_input(&input);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_input_invalid_complexity_too_high() {
        let service = AnalysisService::new();
        let input = AnalysisInput {
            operation: AnalysisOperation::Complexity,
            path: PathBuf::from("."),
            options: AnalysisOptions {
                max_complexity: Some(101),
                ..Default::default()
            },
        };
        let result = service.validate_input(&input);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_input_valid() {
        let service = AnalysisService::new();
        let input = AnalysisInput {
            operation: AnalysisOperation::Complexity,
            path: PathBuf::from("."),
            options: AnalysisOptions::default(),
        };
        let result = service.validate_input(&input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_complexity_results_struct() {
        let results = ComplexityResults {
            total_files: 5,
            average_complexity: 10.0,
            max_complexity: 20,
            violations: vec![ComplexityViolation {
                file: "test.rs".to_string(),
                function: "foo".to_string(),
                complexity: 25,
                line: 10,
            }],
        };
        assert_eq!(results.total_files, 5);
        assert_eq!(results.violations.len(), 1);
    }

    #[test]
    fn test_satd_results_struct() {
        let results = SatdResults {
            total_files: 3,
            total_satd: 2,
            violations: vec![SatdViolation {
                file: "test.rs".to_string(),
                line: 5,
                comment: "TODO: fix this".to_string(),
                category: "TODO".to_string(),
            }],
        };
        assert_eq!(results.total_files, 3);
        assert_eq!(results.total_satd, 2);
    }

    #[test]
    fn test_dead_code_results_struct() {
        let results = DeadCodeResults {
            total_files: 10,
            dead_code_count: 3,
            dead_code_percentage: 30.0,
            unused_items: vec![UnusedItem {
                file: "test.rs".to_string(),
                item: "unused_fn".to_string(),
                item_type: "Function".to_string(),
                line: 15,
            }],
        };
        assert_eq!(results.total_files, 10);
        assert_eq!(results.dead_code_count, 3);
    }

    #[test]
    fn test_analysis_summary_struct() {
        let summary = AnalysisSummary {
            files_analyzed: 100,
            total_issues: 5,
            critical_issues: 2,
            duration_ms: 1500,
        };
        assert_eq!(summary.files_analyzed, 100);
        assert_eq!(summary.total_issues, 5);
        assert_eq!(summary.critical_issues, 2);
    }

    #[test]
    fn test_analysis_service_default() {
        let service = AnalysisService::default();
        assert_eq!(service.name(), "AnalysisService");
    }
}

