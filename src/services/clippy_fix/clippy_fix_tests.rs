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
        // A message with neither lint code nor span is not a fixable diagnostic.
        // It used to parse into a placeholder (code "unknown", file "", line 0)
        // that was then counted as a fix.
        let json = r#"{"message": {}}"#;
        let result = ClippyDiagnostic::from_json(json);
        assert!(result.is_err(), "empty message must not parse as a fix");
    }

    #[test]
    fn test_from_json_rejects_non_diagnostic_cargo_lines() {
        // Regression: cargo's non-diagnostic JSON lines each became a "fix"
        // with file:"" line:0 code:"unknown", inflating total_fixes.
        let artifact = r#"{"reason":"compiler-artifact","package_id":"cx 0.1.0","target":{"name":"cx"}}"#;
        assert!(ClippyDiagnostic::from_json(artifact).is_err());

        let build_finished = r#"{"reason":"build-finished","success":true}"#;
        assert!(ClippyDiagnostic::from_json(build_finished).is_err());

        // The "generated N warnings" summary: a real compiler-message, but with
        // no lint code and no spans.
        let summary = r#"{"reason":"compiler-message","message":{"code":null,"level":"warning","message":"1 warning emitted","spans":[]}}"#;
        assert!(ClippyDiagnostic::from_json(summary).is_err());

        // The real diagnostic in the same stream still parses.
        let real = r#"{"reason":"compiler-message","message":{"code":{"code":"clippy::clone_on_copy"},"level":"warning","message":"using `clone` on type `i32`","spans":[{"file_name":"src/lib.rs","line_start":17,"line_end":17,"column_start":5,"column_end":14}]}}"#;
        let diag = ClippyDiagnostic::from_json(real).expect("real diagnostic must parse");
        assert_eq!(diag.code, "clippy::clone_on_copy");
        assert_eq!(diag.file, PathBuf::from("src/lib.rs"));
        assert_eq!(diag.line_start, 17);
    }

    // ========================================================================
    // Tests for what the engine does NOT do: touch the filesystem (#1086)
    // ========================================================================

    /// Applying a fix leaves the file the diagnostic names byte-identical.
    ///
    /// GUARD, not a regression test: this also passes on the pre-fix code.
    /// That was precisely the defect — `pmat analyze clippy` answered
    /// `"action": "applied"` with a non-zero `successful_fixes` and a populated
    /// `fixed_files` while the named file's bytes never changed. The assertion
    /// is kept so that anyone closing the gap by adding the missing `fs::write`
    /// has to look at `apply_fix_internal` first and see what would be written.
    #[tokio::test]
    async fn apply_fix_leaves_the_file_on_disk_byte_identical() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let path = dir.path().join("lib.rs");
        let original = "fn answer() -> i32 {\n    return 42;\n}\n";
        std::fs::write(&path, original).expect("seed fixture");
        let before = std::fs::read(&path).expect("read fixture");

        let engine = ClippyFixEngine::new();
        let diagnostic = ClippyDiagnostic {
            code: "clippy::needless_return".to_string(),
            level: DiagnosticLevel::Warning,
            message: "unneeded `return` statement".to_string(),
            file: path.clone(),
            line_start: 2,
            line_end: 2,
            column_start: 5,
            column_end: 15,
            suggestion: None,
        };

        let source = std::fs::read_to_string(&path).expect("read source");
        let result = engine
            .apply_fix(&source, &diagnostic)
            .await
            .expect("apply_fix");

        // The transform DID produce a different string...
        assert_ne!(
            result.modified_source, source,
            "the in-memory transform is what made the old report look plausible"
        );
        // ...and that string went nowhere.
        assert_eq!(
            before,
            std::fs::read(&path).expect("re-read fixture"),
            "the engine has no writer: the file must be untouched"
        );
    }

    /// The needless_return transform is a whole-file substring replace.
    ///
    /// `apply_fix_internal` runs `source.replace("return ", "")` over the entire
    /// file and never consults the span the diagnostic carries. Here the only
    /// occurrence of `return ` is inside a string literal, and it is struck all
    /// the same. GUARD, not a regression test: the pre-fix code behaves
    /// identically. It documents why #1086 was closed by dropping the "applied"
    /// claim instead of by writing this output to disk.
    #[tokio::test]
    async fn needless_return_transform_strikes_string_literals_too() {
        let engine = ClippyFixEngine::new();
        let source = "fn main() { println!(\"return code {}\", 1); }";
        let diagnostic = ClippyDiagnostic {
            code: "clippy::needless_return".to_string(),
            level: DiagnosticLevel::Warning,
            message: "unneeded `return` statement".to_string(),
            file: PathBuf::from("main.rs"),
            line_start: 1,
            line_end: 1,
            column_start: 1,
            column_end: 10,
            suggestion: None,
        };

        let result = engine
            .apply_fix(source, &diagnostic)
            .await
            .expect("apply_fix");

        assert!(
            result.modified_source.contains("\"code {}\""),
            "the string literal lost its `return `: {}",
            result.modified_source
        );
        assert!(
            !result.modified_source.contains("return "),
            "the replace is unconditional: {}",
            result.modified_source
        );
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
