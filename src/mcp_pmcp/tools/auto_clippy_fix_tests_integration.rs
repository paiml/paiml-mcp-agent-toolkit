// Integration and async tests for auto_clippy_fix: simulate_fixes,
// create_fix_response, confidence ordering, and edge cases.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::services::clippy_fix::{ClippyDiagnostic, DiagnosticLevel};
    use std::path::PathBuf;

    /// Keys that can only be true of a tool that edits files.
    ///
    /// The response carried all four (#1086) while `md5sum` on the named file
    /// was unchanged. They are asserted absent, not merely zeroed: a key that
    /// does not exist cannot be misread.
    const MUTATION_CLAIM_KEYS: &[&str] = &[
        "successful_fixes",
        "fixed_files",
        "success_rate",
        "would_fix",
    ];

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

        let json = result.expect("simulate_fixes on an empty list");
        assert_eq!(json["preview_only"], true);
        assert_eq!(json["total_previewed"], 0);
        assert!(json["previewed"]
            .as_array()
            .expect("previewed is an array")
            .is_empty());
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

        let json = result.expect("simulate_fixes over two diagnostics");
        assert_eq!(json["preview_only"], true);
        assert_eq!(json["total_previewed"], 2);

        let previewed = json["previewed"]
            .as_array()
            .expect("previewed is an array");
        assert_eq!(previewed.len(), 2);

        // Check first entry structure
        let first = &previewed[0];
        assert_eq!(first["file"], "test.rs");
        assert_eq!(first["line"], 1);
        assert_eq!(first["code"], "clippy::needless_return");
        assert_eq!(first["confidence"], "High");

        // Check second entry has different confidence
        let second = &previewed[1];
        assert_eq!(second["confidence"], "Medium");
    }

    /// A preview must not describe an edit it cannot perform.
    ///
    /// Each entry used to carry `"would_fix": true`, hardcoded on every element
    /// — a promise about a module that contains no `fs::write` at all (#1086).
    ///
    /// Contradicts the pre-fix `auto_clippy_fix_core.rs` line
    /// `"would_fix": true,` inside `simulate_fixes`, and the sibling
    /// `"dry_run": true` / `"total_fixes"` keys of the same payload.
    #[tokio::test]
    async fn preview_entries_promise_nothing_about_writing() {
        let engine = ClippyFixEngine::new();
        let diagnostics = vec![create_test_diagnostic("clippy::needless_return")];
        let json = simulate_fixes(&engine, diagnostics)
            .await
            .expect("simulate_fixes");

        let text = serde_json::to_string(&json).expect("payload serializes");
        for key in MUTATION_CLAIM_KEYS {
            assert!(
                !text.contains(key),
                "preview payload must not carry `{key}`: {text}"
            );
        }
        assert!(
            !text.contains("dry_run"),
            "there is no wet run to contrast with: {text}"
        );
    }

    /// Previewing a diagnostic leaves the file it names byte-identical.
    ///
    /// This is a guard, not a regression test: the pre-fix code also wrote
    /// nothing — that was the defect, since it reported `"action": "applied"`
    /// and a populated `fixed_files` while doing so. The test exists so that a
    /// later attempt to make the preview path write is caught here rather than
    /// in a user's working tree, given the transform behind it is a whole-file
    /// `source.replace("return ", "")`.
    #[tokio::test]
    async fn previewing_leaves_the_named_file_untouched_on_disk() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let file = dir.path().join("main.rs");
        let original = "fn main() {\n    return println!(\"return now\");\n}\n";
        std::fs::write(&file, original).expect("seed fixture");
        let before = std::fs::read(&file).expect("read fixture");

        let engine = ClippyFixEngine::new();
        let mut diagnostic = create_test_diagnostic("clippy::needless_return");
        diagnostic.file = file.clone();

        let json = simulate_fixes(&engine, vec![diagnostic])
            .await
            .expect("simulate_fixes");
        assert_eq!(json["total_previewed"], 1);

        let after = std::fs::read(&file).expect("re-read fixture");
        assert_eq!(
            before, after,
            "preview must not modify the file it names on disk"
        );
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
        let response = create_fix_response(json!({}), &census(76, 0));
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
        let response = create_fix_response(json!({}), &census(0, 0));
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

    /// Read the single text payload out of a tool result.
    fn payload(response: &pmcp::ToolResult) -> String {
        let pmcp::Content::Text { text } = &response.content[0] else {
            unreachable!("Expected Text content")
        };
        text.clone()
    }

    #[test]
    fn test_create_fix_response_preview_shape() {
        let results = json!({
            "preview_only": true,
            "total_previewed": 5,
            "previewed": []
        });

        let response = create_fix_response(results, &census(5, 5));

        assert!(!response.is_error);
        assert_eq!(response.content.len(), 1);

        let text = payload(&response);
        assert!(text.contains("previewed"), "{text}");
        assert!(text.contains("clippy reported"), "{text}");
        assert!(text.contains("preview_only"), "{text}");
    }

    /// The response can never say "applied".
    ///
    /// This is the #1086 regression. `create_fix_response` chose its verb with
    /// `let action = if is_dry_run { "analyzed" } else { "applied" };`, and the
    /// "applied" arm was returned — with `successful_fixes` and a named
    /// `fixed_files` — by a code path with no `fs::write` behind it: the file
    /// was byte-identical afterwards. On the pre-fix code this test fails on
    /// the very first assertion, since a non-dry-run call produced exactly the
    /// string it forbids.
    ///
    /// The verb is now a constant, and the flag that used to pick it is gone
    /// from the signature, so no caller can reintroduce the claim.
    #[test]
    fn the_response_can_never_claim_a_fix_was_applied() {
        // Every census shape the tool can produce, including the ones the old
        // "apply" branch served.
        for (found, eligible) in [(0, 0), (10, 8), (76, 0), (3, 3)] {
            let results = json!({ "preview_only": true, "total_previewed": eligible });
            let text = payload(&create_fix_response(results, &census(found, eligible)));
            let doc: serde_json::Value = serde_json::from_str(&text).expect("valid json");

            assert_eq!(
                doc["action"], "previewed",
                "action must state what happened: {text}"
            );
            assert!(
                !text.contains("applied"),
                "nothing was applied, so the word must not appear: {text}"
            );
            for key in MUTATION_CLAIM_KEYS {
                assert!(
                    !text.contains(key),
                    "response must not carry `{key}`: {text}"
                );
            }
        }
    }

    /// The full payload, assembled the way `auto_clippy_fix` assembles it.
    ///
    /// Guards the seam between the two halves: a preview result wrapped by
    /// `create_fix_response` must not acquire a mutation claim on the way out.
    /// Fails on the pre-fix code, which wrote `"action": "applied"` around a
    /// results object carrying `"dry_run"` and `"would_fix"`.
    #[tokio::test]
    async fn the_assembled_payload_carries_no_mutation_claim() {
        let engine = ClippyFixEngine::new();
        let diagnostics = vec![
            create_test_diagnostic("clippy::needless_return"),
            create_test_diagnostic("clippy::manual_map"),
        ];
        let results = simulate_fixes(&engine, diagnostics)
            .await
            .expect("simulate_fixes");

        let text = payload(&create_fix_response(results, &census(2, 2)));
        let doc: serde_json::Value = serde_json::from_str(&text).expect("valid json");

        assert_eq!(doc["action"], "previewed", "{text}");
        assert_eq!(doc["results"]["preview_only"], true, "{text}");
        for key in MUTATION_CLAIM_KEYS {
            assert!(!text.contains(key), "payload must not carry `{key}`: {text}");
        }
        assert!(
            !text.contains("applied"),
            "no file was written, so nothing was applied: {text}"
        );

        let message = doc["message"].as_str().expect("message");
        assert!(
            message.contains("previewed"),
            "the message states the verb too: {message}"
        );
        assert!(
            message.contains("No file was modified"),
            "the message must say plainly that nothing was written: {message}"
        );
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

        let result = simulate_fixes(&engine, vec![diagnostic])
            .await
            .expect("simulate_fixes");
        let previewed = result["previewed"]
            .as_array()
            .expect("previewed is an array");
        let entry = &previewed[0];

        assert_eq!(entry["message"], "specific test message");
        assert_eq!(entry["line"], 42);
    }

    #[test]
    fn test_parse_clippy_output_whitespace_only_lines() {
        let output = "   \n\t\n   \t   \n";
        let result = parse_clippy_output(output);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    /// The census drives the message; there is no second mode to differ from.
    ///
    /// This test used to build two responses from the same census, one with
    /// `is_dry_run = true` and one with `false`, and assert they DIFFERED —
    /// `text_dry.contains("analyzed")`, `text_apply.contains("applied")`. That
    /// difference was the defect: the second was a claim that files had been
    /// rewritten, produced by a path that wrote nothing (#1086). The parameter
    /// is gone, so the property worth pinning is the opposite one — the same
    /// census yields the same verdict, every time.
    #[test]
    fn the_same_census_always_yields_the_same_verdict() {
        let first = payload(&create_fix_response(json!({}), &census(3, 3)));
        let second = payload(&create_fix_response(json!({}), &census(3, 3)));
        assert_eq!(first, second);

        let doc: serde_json::Value = serde_json::from_str(&first).expect("valid json");
        assert_eq!(doc["action"], "previewed", "{first}");

        // A different census must still change the message, or the assertion
        // above would pass on a constant string.
        let other = payload(&create_fix_response(json!({}), &census(9, 1)));
        assert_ne!(first, other);
    }
}
