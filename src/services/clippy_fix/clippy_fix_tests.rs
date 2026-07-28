#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;

    // ========================================================================
    // Tests for DiagnosticLevel
    // ========================================================================

    #[test]
    fn test_diagnostic_level_error() {
        assert_eq!(DiagnosticLevel::from_str("error"), DiagnosticLevel::Error);
    }

    #[test]
    fn test_diagnostic_level_warning() {
        assert_eq!(
            DiagnosticLevel::from_str("warning"),
            DiagnosticLevel::Warning
        );
    }

    #[test]
    fn test_diagnostic_level_note() {
        assert_eq!(DiagnosticLevel::from_str("note"), DiagnosticLevel::Note);
    }

    #[test]
    fn test_diagnostic_level_help() {
        // Unknown levels become Help
        assert_eq!(DiagnosticLevel::from_str("help"), DiagnosticLevel::Help);
        assert_eq!(DiagnosticLevel::from_str("unknown"), DiagnosticLevel::Help);
        assert_eq!(DiagnosticLevel::from_str(""), DiagnosticLevel::Help);
    }

    // ========================================================================
    // Tests for ClippyFixEngine
    // ========================================================================

    #[test]
    fn test_engine_new() {
        let engine = ClippyFixEngine::new();
        // Verify confidence rules are initialized
        assert!(!engine.confidence_rules.is_empty());
    }

    #[test]
    fn test_engine_default() {
        let engine = ClippyFixEngine::default();
        // Default should be equivalent to new()
        assert!(!engine.confidence_rules.is_empty());
    }

    #[test]
    fn test_calculate_confidence_high() {
        let engine = ClippyFixEngine::new();
        let diagnostic = ClippyDiagnostic {
            code: "clippy::needless_return".to_string(),
            level: DiagnosticLevel::Warning,
            message: "test".to_string(),
            file: PathBuf::from("test.rs"),
            line_start: 1,
            line_end: 1,
            column_start: 1,
            column_end: 10,
            suggestion: None,
        };
        assert_eq!(
            engine.calculate_confidence(&diagnostic),
            ConfidenceLevel::High
        );
    }

    #[test]
    fn test_calculate_confidence_medium() {
        let engine = ClippyFixEngine::new();
        let diagnostic = ClippyDiagnostic {
            code: "clippy::manual_map".to_string(),
            level: DiagnosticLevel::Warning,
            message: "test".to_string(),
            file: PathBuf::from("test.rs"),
            line_start: 1,
            line_end: 1,
            column_start: 1,
            column_end: 10,
            suggestion: None,
        };
        assert_eq!(
            engine.calculate_confidence(&diagnostic),
            ConfidenceLevel::Medium
        );
    }

    #[test]
    fn test_calculate_confidence_low() {
        let engine = ClippyFixEngine::new();
        let diagnostic = ClippyDiagnostic {
            code: "clippy::needless_lifetimes".to_string(),
            level: DiagnosticLevel::Warning,
            message: "test".to_string(),
            file: PathBuf::from("test.rs"),
            line_start: 1,
            line_end: 1,
            column_start: 1,
            column_end: 10,
            suggestion: None,
        };
        assert_eq!(
            engine.calculate_confidence(&diagnostic),
            ConfidenceLevel::Low
        );
    }

    #[test]
    fn test_calculate_confidence_unknown_with_suggestion() {
        let engine = ClippyFixEngine::new();
        let diagnostic = ClippyDiagnostic {
            code: "unknown::lint".to_string(),
            level: DiagnosticLevel::Warning,
            message: "test".to_string(),
            file: PathBuf::from("test.rs"),
            line_start: 1,
            line_end: 1,
            column_start: 1,
            column_end: 10,
            suggestion: Some("replace with X".to_string()),
        };
        // Unknown lint with suggestion gets Medium confidence
        assert_eq!(
            engine.calculate_confidence(&diagnostic),
            ConfidenceLevel::Medium
        );
    }

    #[test]
    fn test_calculate_confidence_unknown_without_suggestion() {
        let engine = ClippyFixEngine::new();
        let diagnostic = ClippyDiagnostic {
            code: "unknown::lint".to_string(),
            level: DiagnosticLevel::Warning,
            message: "test".to_string(),
            file: PathBuf::from("test.rs"),
            line_start: 1,
            line_end: 1,
            column_start: 1,
            column_end: 10,
            suggestion: None,
        };
        // Unknown lint without suggestion gets Low confidence
        assert_eq!(
            engine.calculate_confidence(&diagnostic),
            ConfidenceLevel::Low
        );
    }

    // ========================================================================
    // Tests for generate_report
    // ========================================================================

    #[test]
    fn test_generate_report_empty() {
        let engine = ClippyFixEngine::new();
        let report = engine.generate_report(vec![]);

        assert_eq!(report.total_diagnostics, 0);
        assert_eq!(report.successful_fixes, 0);
        assert_eq!(report.failed_fixes, 0);
        assert!(report.fixed_files.is_empty());
    }

    #[test]
    fn test_generate_report_with_results() {
        let engine = ClippyFixEngine::new();
        let results = vec![
            FixResult {
                success: true,
                diagnostic: ClippyDiagnostic {
                    code: "test".to_string(),
                    level: DiagnosticLevel::Warning,
                    message: "msg".to_string(),
                    file: PathBuf::from("file1.rs"),
                    line_start: 1,
                    line_end: 1,
                    column_start: 1,
                    column_end: 10,
                    suggestion: None,
                },
                modified_source: "fixed".to_string(),
                confidence: ConfidenceLevel::High,
                duration: Duration::from_millis(100),
                error: None,
            },
            FixResult {
                success: false,
                diagnostic: ClippyDiagnostic {
                    code: "test".to_string(),
                    level: DiagnosticLevel::Warning,
                    message: "msg".to_string(),
                    file: PathBuf::from("file2.rs"),
                    line_start: 1,
                    line_end: 1,
                    column_start: 1,
                    column_end: 10,
                    suggestion: None,
                },
                modified_source: "".to_string(),
                confidence: ConfidenceLevel::Low,
                duration: Duration::from_millis(50),
                error: Some("failed".to_string()),
            },
        ];

        let report = engine.generate_report(results);

        assert_eq!(report.total_diagnostics, 2);
        assert_eq!(report.successful_fixes, 1);
        assert_eq!(report.failed_fixes, 1);
        assert_eq!(report.success_rate, 50.0);
        assert_eq!(report.fixed_files.len(), 2);
    }

    // ========================================================================
    // Tests for ClippyDiagnostic JSON parsing
    // ========================================================================

    #[test]
    fn test_from_json_valid() {
        let json = r#"{
            "message": {
                "code": {"code": "clippy::test"},
                "level": "warning",
                "message": "test message",
                "spans": [{
                    "file_name": "src/main.rs",
                    "line_start": 10,
                    "line_end": 12,
                    "column_start": 5,
                    "column_end": 15
                }]
            }
        }"#;

        let result = ClippyDiagnostic::from_json(json);
        assert!(result.is_ok());

        let diag = result.unwrap();
        assert_eq!(diag.code, "clippy::test");
        assert_eq!(diag.level, DiagnosticLevel::Warning);
        assert_eq!(diag.message, "test message");
        assert_eq!(diag.file, PathBuf::from("src/main.rs"));
        assert_eq!(diag.line_start, 10);
        assert_eq!(diag.line_end, 12);
    }

    #[test]
    fn test_from_json_invalid() {
        let json = "not valid json";
        let result = ClippyDiagnostic::from_json(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_from_json_missing_fields() {
        // JSON with missing fields should use defaults
        let json = r#"{"message": {}}"#;
        let result = ClippyDiagnostic::from_json(json);
        assert!(result.is_ok());

        let diag = result.unwrap();
        assert_eq!(diag.code, "unknown");
        assert_eq!(diag.level, DiagnosticLevel::Warning); // default for "warning"
    }

    // ========================================================================
    // Tests for ConfidenceLevel enum
    // ========================================================================

    #[test]
    fn test_confidence_level_debug() {
        assert_eq!(format!("{:?}", ConfidenceLevel::High), "High");
        assert_eq!(format!("{:?}", ConfidenceLevel::Medium), "Medium");
        assert_eq!(format!("{:?}", ConfidenceLevel::Low), "Low");
    }

    #[test]
    fn test_confidence_level_clone() {
        let c1 = ConfidenceLevel::High;
        let c2 = c1.clone();
        assert_eq!(c1, c2);
    }
}
