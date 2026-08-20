// Integration and async tests for auto_clippy_fix: simulate_fixes, apply_fixes,
// create_fix_response, confidence ordering, and edge cases.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::services::clippy_fix::{ClippyDiagnostic, DiagnosticLevel};
    use std::path::PathBuf;

    fn create_test_diagnostic(code: &str) -> ClippyDiagnostic {
        ClippyDiagnostic {
            code: code.to_string(),
            level: DiagnosticLevel::Warning,
            message: "test message".to_string(),
            file: PathBuf::from("test.rs"),
            line_start: 1,
            line_end: 1,
            column_start: 1,
            column_end: 10,
            suggestion: None,
        }
    }

    // ========================================================================
    // Tests for simulate_fixes
    // ========================================================================

    #[tokio::test]
    async fn test_simulate_fixes_empty() {
        let engine = ClippyFixEngine::new();
        let result = simulate_fixes(&engine, vec![]).await;
        assert!(result.is_ok());

        let json = result.unwrap();
        assert_eq!(json["dry_run"], true);
        assert_eq!(json["total_fixes"], 0);
        assert!(json["fixes"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_simulate_fixes_with_diagnostics() {
        let engine = ClippyFixEngine::new();
        let diagnostics = vec![
            create_test_diagnostic("clippy::needless_return"),
            create_test_diagnostic("clippy::manual_map"),
        ];

        let result = simulate_fixes(&engine, diagnostics).await;
        assert!(result.is_ok());

        let json = result.unwrap();
        assert_eq!(json["dry_run"], true);
        assert_eq!(json["total_fixes"], 2);

        let fixes = json["fixes"].as_array().unwrap();
        assert_eq!(fixes.len(), 2);

        // Check first fix structure
        let fix0 = &fixes[0];
        assert_eq!(fix0["file"], "test.rs");
        assert_eq!(fix0["line"], 1);
        assert_eq!(fix0["code"], "clippy::needless_return");
        assert_eq!(fix0["would_fix"], true);
        assert_eq!(fix0["confidence"], "High");

        // Check second fix has different confidence
        let fix1 = &fixes[1];
        assert_eq!(fix1["confidence"], "Medium");
    }

    // ========================================================================
    // Tests for apply_fixes
    // ========================================================================

    #[tokio::test]
    async fn test_apply_fixes_empty() {
        let engine = ClippyFixEngine::new();
        let result = apply_fixes(&engine, vec![]).await;
        assert!(result.is_ok());

        let json = result.unwrap();
        assert_eq!(json["dry_run"], false);
        assert!(json["report"].is_object());
        assert!(json["detailed_results"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_apply_fixes_with_diagnostics() {
        let engine = ClippyFixEngine::new();
        let diagnostics = vec![
            create_test_diagnostic("clippy::needless_return"),
            create_test_diagnostic("clippy::manual_map"),
        ];

        let result = apply_fixes(&engine, diagnostics).await;
        assert!(result.is_ok());

        let json = result.unwrap();
        assert_eq!(json["dry_run"], false);

        // Check report structure
        let report = &json["report"];
        assert_eq!(report["total_diagnostics"], 2);
        let _ = report["successful_fixes"].as_u64().unwrap();
        assert!(report["success_rate"].is_number());
        assert!(report["total_duration_ms"].is_number());

        // Check detailed results
        let results = json["detailed_results"].as_array().unwrap();
        assert_eq!(results.len(), 2);

        for r in results {
            assert!(r["file"].is_string());
            assert!(r["line"].is_number());
            assert!(r["code"].is_string());
            assert!(r["success"].is_boolean());
            assert!(r["duration_ms"].is_number());
        }
    }

    // ========================================================================
    // Tests for create_fix_response
    // ========================================================================

    fn census(found: usize, eligible: usize) -> DiagnosticCensus {
        DiagnosticCensus {
            found,
            eligible,
            min_confidence: "High".to_string(),
        }
    }

    /// Diagnostics FOUND and diagnostics ELIGIBLE are different numbers.
    ///
    /// They were reported as one: everything downstream counted the filtered
    /// list, so on a crate where `cargo clippy` emits 76 warnings this returned
    /// `"total_diagnostics": 0` with `"message": "🔧 Clippy fixes applied
    /// successfully"` and exit 0. The default is `--confidence high`, and any
    /// lint without an explicit rule is rated Medium or Low, so every
    /// diagnostic was dropped — 75 `clippy::collapsible_if` and one
    /// `dead_code`. "None were auto-fixable at this confidence" and "the crate
    /// is clippy-clean" are opposite claims, and only the second was reported.
    #[test]
    fn diagnostics_found_survives_the_confidence_filter() {
        let response = create_fix_response(json!({}), false, &census(76, 0));
        let pmcp::Content::Text { text } = &response.content[0] else {
            unreachable!("Expected Text content")
        };
        let doc: serde_json::Value = serde_json::from_str(text).expect("valid json");

        assert_eq!(doc["diagnostics_found"], 76, "{text}");
        assert_eq!(doc["diagnostics_eligible"], 0, "{text}");
        assert_eq!(doc["diagnostics_filtered_out"], 76, "{text}");

        let message = doc["message"].as_str().expect("message");
        assert!(
            message.contains("76"),
            "the message must carry the count clippy actually reported: {message}"
        );
        assert!(
            !message.contains("successfully"),
            "0 of 76 fixed is not success: {message}"
        );
    }

    /// The counter-test: a genuinely clean crate must still read as clean.
    ///
    /// Without this, warning on every run would pass the test above.
    #[test]
    fn a_crate_with_no_diagnostics_still_reads_as_clean() {
        let response = create_fix_response(json!({}), false, &census(0, 0));
        let pmcp::Content::Text { text } = &response.content[0] else {
            unreachable!("Expected Text content")
        };
        let doc: serde_json::Value = serde_json::from_str(text).expect("valid json");

        assert_eq!(doc["diagnostics_found"], 0);
        let message = doc["message"].as_str().expect("message");
        assert!(
            !message.contains("NOT a clean result"),
            "a crate clippy had nothing to say about must not be flagged: {message}"
        );
    }

    #[test]
    fn test_create_fix_response_dry_run() {
        let results = json!({
            "dry_run": true,
            "total_fixes": 5,
            "fixes": []
        });

        let response = create_fix_response(results, true, &census(5, 5));

        assert!(!response.is_error);
        assert_eq!(response.content.len(), 1);

        if let pmcp::Content::Text { text } = &response.content[0] {
            assert!(text.contains("analyzed"));
            assert!(text.contains("clippy reported"));
            assert!(text.contains("dry_run"));
        } else {
            panic!("Expected Text content");
        }
    }

    #[test]
    fn test_create_fix_response_applied() {
        let results = json!({
            "dry_run": false,
            "report": {
                "total_diagnostics": 10,
                "successful_fixes": 8
            }
        });

        let response = create_fix_response(results, false, &census(10, 8));

        assert!(!response.is_error);
        assert_eq!(response.content.len(), 1);

        if let pmcp::Content::Text { text } = &response.content[0] {
            assert!(text.contains("applied"));
            assert!(text.contains("clippy reported"));
        } else {
            panic!("Expected Text content");
        }
    }

    // ========================================================================
    // Integration tests for auto_clippy_fix (mocked)
    // ========================================================================

    #[tokio::test]
    async fn test_auto_clippy_fix_invalid_confidence() {
        // Test with invalid confidence level
        let result = auto_clippy_fix(
            Some("/nonexistent/path".to_string()),
            Some("invalid_level".to_string()),
            Some(true),
            None,
        )
        .await;

        // Should fail on confidence parsing before running clippy
        assert!(result.is_err());
    }

    // ========================================================================
    // Property-based tests for confidence matching
    // ========================================================================

    #[test]
    fn test_confidence_transitivity() {
        // High always >= any minimum
        for min in [
            ConfidenceLevel::High,
            ConfidenceLevel::Medium,
            ConfidenceLevel::Low,
        ] {
            assert!(confidence_meets_minimum(ConfidenceLevel::High, min.clone()));
        }
    }

    #[test]
    fn test_confidence_ordering() {
        // Verify correct partial ordering
        assert!(!confidence_meets_minimum(
            ConfidenceLevel::Low,
            ConfidenceLevel::High
        ));
        assert!(!confidence_meets_minimum(
            ConfidenceLevel::Low,
            ConfidenceLevel::Medium
        ));
        assert!(!confidence_meets_minimum(
            ConfidenceLevel::Medium,
            ConfidenceLevel::High
        ));
    }

    // ========================================================================
    // Edge case tests
    // ========================================================================

    #[test]
    fn test_filter_diagnostics_empty_specific_codes() {
        let engine = ClippyFixEngine::new();
        let diagnostics = vec![create_test_diagnostic("clippy::needless_return")];

        // Empty vec means filter out everything
        let empty_codes = Some(vec![]);
        let filtered = filter_diagnostics(&engine, diagnostics, ConfidenceLevel::Low, &empty_codes);
        assert!(filtered.is_empty());
    }

    #[tokio::test]
    async fn test_simulate_fixes_preserves_diagnostic_details() {
        let engine = ClippyFixEngine::new();
        let mut diagnostic = create_test_diagnostic("clippy::needless_return");
        diagnostic.message = "specific test message".to_string();
        diagnostic.line_start = 42;

        let result = simulate_fixes(&engine, vec![diagnostic]).await.unwrap();
        let fixes = result["fixes"].as_array().unwrap();
        let fix = &fixes[0];

        assert_eq!(fix["message"], "specific test message");
        assert_eq!(fix["line"], 42);
    }

    #[test]
    fn test_parse_clippy_output_whitespace_only_lines() {
        let output = "   \n\t\n   \t   \n";
        let result = parse_clippy_output(output);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_create_fix_response_message_formatting() {
        let response_dry = create_fix_response(json!({}), true, &census(3, 3));
        let response_apply = create_fix_response(json!({}), false, &census(3, 3));

        // Verify different messages for dry run vs apply
        if let (pmcp::Content::Text { text: text_dry }, pmcp::Content::Text { text: text_apply }) =
            (&response_dry.content[0], &response_apply.content[0])
        {
            assert!(text_dry.contains("analyzed"));
            assert!(text_apply.contains("applied"));
            assert_ne!(text_dry, text_apply);
        }
    }
}
