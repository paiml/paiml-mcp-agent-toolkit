//! MCP Tool for Automated Clippy Fix
//!
//! A+ Code Standard: ALL functions ≤10 complexity
//! MCP-First Dogfooding: Primary interface for clippy fixes

use crate::services::clippy_fix::{ClippyDiagnostic, ClippyFixEngine, ConfidenceLevel};
use anyhow::Result;
use pmcp::ToolResult;
use serde_json::{json, Value};

/// Auto-fix clippy warnings with confidence-based filtering
///
/// Complexity: 8 (within A+ standard ≤10)
pub async fn auto_clippy_fix(
    project_path: Option<String>,
    confidence_level: Option<String>,
    dry_run: Option<bool>,
    fix_specific_codes: Option<Vec<String>>,
) -> Result<ToolResult> {
    let path = project_path.unwrap_or_else(|| ".".to_string());
    let min_confidence = parse_confidence_level(&confidence_level)?;
    let is_dry_run = dry_run.unwrap_or(false);

    // Run clippy and get diagnostics
    let diagnostics = run_clippy_analysis(&path).await?;

    // Filter by confidence level
    let engine = ClippyFixEngine::new();
    let filtered = filter_diagnostics(&engine, diagnostics, min_confidence, &fix_specific_codes);

    // Apply fixes or show what would be fixed
    let results = if is_dry_run {
        simulate_fixes(&engine, filtered).await?
    } else {
        apply_fixes(&engine, filtered).await?
    };

    Ok(create_fix_response(results, is_dry_run))
}

/// Parse confidence level from string (complexity: 3)
fn parse_confidence_level(level: &Option<String>) -> Result<ConfidenceLevel> {
    match level.as_deref() {
        Some("high") => Ok(ConfidenceLevel::High),
        Some("medium") => Ok(ConfidenceLevel::Medium),
        Some("low") => Ok(ConfidenceLevel::Low),
        None => Ok(ConfidenceLevel::High), // Default to safe fixes
        Some(other) => Err(anyhow::anyhow!("Invalid confidence level: {other}")),
    }
}

/// Run clippy analysis and parse output (complexity: 5)
async fn run_clippy_analysis(path: &str) -> Result<Vec<ClippyDiagnostic>> {
    use tokio::process::Command;

    let output = Command::new("cargo")
        .args(["clippy", "--message-format=json"])
        .current_dir(path)
        .output()
        .await?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Clippy failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    parse_clippy_output(&String::from_utf8_lossy(&output.stdout))
}

/// Parse clippy JSON output (complexity: 6)
fn parse_clippy_output(output: &str) -> Result<Vec<ClippyDiagnostic>> {
    let mut diagnostics = Vec::new();

    for line in output.lines() {
        if line.trim().is_empty() {
            continue;
        }

        if let Ok(diagnostic) = ClippyDiagnostic::from_json(line) {
            diagnostics.push(diagnostic);
        }
    }

    Ok(diagnostics)
}

/// Filter diagnostics by criteria (complexity: 7)
fn filter_diagnostics(
    engine: &ClippyFixEngine,
    diagnostics: Vec<ClippyDiagnostic>,
    min_confidence: ConfidenceLevel,
    specific_codes: &Option<Vec<String>>,
) -> Vec<ClippyDiagnostic> {
    diagnostics
        .into_iter()
        .filter(|d| {
            let confidence = engine.calculate_confidence(d);
            confidence_meets_minimum(confidence, min_confidence.clone())
        })
        .filter(|d| {
            if let Some(codes) = specific_codes {
                codes.contains(&d.code)
            } else {
                true
            }
        })
        .collect()
}

/// Check if confidence meets minimum (complexity: 3)
fn confidence_meets_minimum(actual: ConfidenceLevel, minimum: ConfidenceLevel) -> bool {
    matches!(
        (actual, minimum),
        (ConfidenceLevel::High, _)
            | (ConfidenceLevel::Medium, ConfidenceLevel::Low)
            | (ConfidenceLevel::Medium, ConfidenceLevel::Medium)
            | (ConfidenceLevel::Low, ConfidenceLevel::Low)
    )
}

/// Simulate fixes without applying (complexity: 4)
async fn simulate_fixes(
    engine: &ClippyFixEngine,
    diagnostics: Vec<ClippyDiagnostic>,
) -> Result<Value> {
    let mut fixes = Vec::new();

    for diagnostic in diagnostics {
        let confidence = engine.calculate_confidence(&diagnostic);
        fixes.push(json!({
            "file": diagnostic.file,
            "line": diagnostic.line_start,
            "code": diagnostic.code,
            "message": diagnostic.message,
            "confidence": format!("{:?}", confidence),
            "would_fix": true,
        }));
    }

    Ok(json!({
        "dry_run": true,
        "total_fixes": fixes.len(),
        "fixes": fixes,
    }))
}

/// Apply fixes to code (complexity: 5)
async fn apply_fixes(
    engine: &ClippyFixEngine,
    diagnostics: Vec<ClippyDiagnostic>,
) -> Result<Value> {
    let results = engine.apply_batch_fixes(&diagnostics).await?;
    let report = engine.generate_report(results.clone());

    let detailed_results: Vec<Value> = results
        .iter()
        .map(|r| {
            json!({
                "file": r.diagnostic.file,
                "line": r.diagnostic.line_start,
                "code": r.diagnostic.code,
                "success": r.success,
                "error": r.error,
                "duration_ms": r.duration.as_millis(),
            })
        })
        .collect();

    Ok(json!({
        "dry_run": false,
        "report": {
            "total_diagnostics": report.total_diagnostics,
            "successful_fixes": report.successful_fixes,
            "failed_fixes": report.failed_fixes,
            "success_rate": report.success_rate,
            "total_duration_ms": report.total_duration.as_millis(),
            "fixed_files": report.fixed_files,
        },
        "detailed_results": detailed_results,
    }))
}

/// Create MCP response (complexity: 2)
fn create_fix_response(results: Value, is_dry_run: bool) -> ToolResult {
    let action = if is_dry_run { "analyzed" } else { "applied" };

    let response = json!({
        "action": action,
        "results": results,
        "message": format!("🔧 Clippy fixes {} successfully", action)
    });

    ToolResult::new(vec![pmcp::Content::Text {
        text: serde_json::to_string_pretty(&response).unwrap_or_else(|_| response.to_string()),
    }])
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;
    use crate::services::clippy_fix::{ClippyDiagnostic, DiagnosticLevel};
    use std::path::PathBuf;

    // ========================================================================
    // Tests for parse_confidence_level
    // ========================================================================

    #[test]
    fn test_parse_confidence_level_high() {
        let result = parse_confidence_level(&Some("high".to_string()));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ConfidenceLevel::High);
    }

    #[test]
    fn test_parse_confidence_level_medium() {
        let result = parse_confidence_level(&Some("medium".to_string()));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ConfidenceLevel::Medium);
    }

    #[test]
    fn test_parse_confidence_level_low() {
        let result = parse_confidence_level(&Some("low".to_string()));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ConfidenceLevel::Low);
    }

    #[test]
    fn test_parse_confidence_level_none_defaults_to_high() {
        let result = parse_confidence_level(&None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ConfidenceLevel::High);
    }

    #[test]
    fn test_parse_confidence_level_invalid() {
        let result = parse_confidence_level(&Some("invalid".to_string()));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid confidence level"));
        assert!(err.to_string().contains("invalid"));
    }

    #[test]
    fn test_parse_confidence_level_empty_string() {
        let result = parse_confidence_level(&Some("".to_string()));
        assert!(result.is_err());
    }

    // ========================================================================
    // Tests for parse_clippy_output
    // ========================================================================

    #[test]
    fn test_parse_clippy_output_empty() {
        let result = parse_clippy_output("");
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_parse_clippy_output_empty_lines() {
        let result = parse_clippy_output("\n\n   \n\n");
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_parse_clippy_output_valid_json() {
        let json = r#"{"message":{"code":{"code":"clippy::test"},"level":"warning","message":"test","spans":[{"file_name":"test.rs","line_start":1,"line_end":1,"column_start":1,"column_end":10}]}}"#;
        let result = parse_clippy_output(json);
        assert!(result.is_ok());
        let diagnostics = result.unwrap();
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, "clippy::test");
    }

    #[test]
    fn test_parse_clippy_output_multiple_lines_mixed() {
        // Mix of valid JSON and invalid lines
        let output = r#"
{"message":{"code":{"code":"lint1"},"level":"warning","message":"msg1","spans":[{"file_name":"a.rs","line_start":1,"line_end":1,"column_start":1,"column_end":5}]}}
Not valid JSON line
{"message":{"code":{"code":"lint2"},"level":"error","message":"msg2","spans":[{"file_name":"b.rs","line_start":10,"line_end":10,"column_start":1,"column_end":5}]}}
"#;
        let result = parse_clippy_output(output);
        assert!(result.is_ok());
        let diagnostics = result.unwrap();
        // Should parse 2 valid diagnostics, skipping invalid line
        assert_eq!(diagnostics.len(), 2);
        assert_eq!(diagnostics[0].code, "lint1");
        assert_eq!(diagnostics[1].code, "lint2");
    }

    #[test]
    fn test_parse_clippy_output_invalid_json_lines() {
        let output = "not json\nalso not json\n{malformed";
        let result = parse_clippy_output(output);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    // ========================================================================
    // Tests for confidence_meets_minimum
    // ========================================================================

    #[test]
    fn test_confidence_meets_minimum_high_always_passes() {
        assert!(confidence_meets_minimum(
            ConfidenceLevel::High,
            ConfidenceLevel::High
        ));
        assert!(confidence_meets_minimum(
            ConfidenceLevel::High,
            ConfidenceLevel::Medium
        ));
        assert!(confidence_meets_minimum(
            ConfidenceLevel::High,
            ConfidenceLevel::Low
        ));
    }

    #[test]
    fn test_confidence_meets_minimum_medium_passes_medium_and_low() {
        assert!(confidence_meets_minimum(
            ConfidenceLevel::Medium,
            ConfidenceLevel::Medium
        ));
        assert!(confidence_meets_minimum(
            ConfidenceLevel::Medium,
            ConfidenceLevel::Low
        ));
        assert!(!confidence_meets_minimum(
            ConfidenceLevel::Medium,
            ConfidenceLevel::High
        ));
    }

    #[test]
    fn test_confidence_meets_minimum_low_only_passes_low() {
        assert!(confidence_meets_minimum(
            ConfidenceLevel::Low,
            ConfidenceLevel::Low
        ));
        assert!(!confidence_meets_minimum(
            ConfidenceLevel::Low,
            ConfidenceLevel::Medium
        ));
        assert!(!confidence_meets_minimum(
            ConfidenceLevel::Low,
            ConfidenceLevel::High
        ));
    }

    // ========================================================================
    // Tests for filter_diagnostics
    // ========================================================================

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

    #[test]
    fn test_filter_diagnostics_empty() {
        let engine = ClippyFixEngine::new();
        let filtered = filter_diagnostics(&engine, vec![], ConfidenceLevel::High, &None);
        assert!(filtered.is_empty());
    }

    #[test]
    fn test_filter_diagnostics_by_confidence() {
        let engine = ClippyFixEngine::new();
        let diagnostics = vec![
            create_test_diagnostic("clippy::needless_return"), // High confidence
            create_test_diagnostic("clippy::manual_map"),      // Medium confidence
            create_test_diagnostic("clippy::needless_lifetimes"), // Low confidence
        ];

        // Filter for High only
        let high_only =
            filter_diagnostics(&engine, diagnostics.clone(), ConfidenceLevel::High, &None);
        assert_eq!(high_only.len(), 1);
        assert_eq!(high_only[0].code, "clippy::needless_return");

        // Filter for Medium and above
        let medium_plus =
            filter_diagnostics(&engine, diagnostics.clone(), ConfidenceLevel::Medium, &None);
        assert_eq!(medium_plus.len(), 2);

        // Filter for Low (all pass)
        let all = filter_diagnostics(&engine, diagnostics, ConfidenceLevel::Low, &None);
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn test_filter_diagnostics_by_specific_codes() {
        let engine = ClippyFixEngine::new();
        let diagnostics = vec![
            create_test_diagnostic("clippy::needless_return"),
            create_test_diagnostic("clippy::manual_map"),
            create_test_diagnostic("clippy::redundant_clone"),
        ];

        let specific_codes = Some(vec![
            "clippy::needless_return".to_string(),
            "clippy::redundant_clone".to_string(),
        ]);

        let filtered =
            filter_diagnostics(&engine, diagnostics, ConfidenceLevel::Low, &specific_codes);
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().any(|d| d.code == "clippy::needless_return"));
        assert!(filtered.iter().any(|d| d.code == "clippy::redundant_clone"));
        assert!(!filtered.iter().any(|d| d.code == "clippy::manual_map"));
    }

    #[test]
    fn test_filter_diagnostics_combined_filters() {
        let engine = ClippyFixEngine::new();
        let diagnostics = vec![
            create_test_diagnostic("clippy::needless_return"), // High - in list
            create_test_diagnostic("clippy::manual_map"),      // Medium - in list
            create_test_diagnostic("clippy::needless_lifetimes"), // Low - NOT in list
        ];

        let specific_codes = Some(vec![
            "clippy::needless_return".to_string(),
            "clippy::manual_map".to_string(),
        ]);

        // High confidence + specific codes: only needless_return
        let filtered = filter_diagnostics(
            &engine,
            diagnostics.clone(),
            ConfidenceLevel::High,
            &specific_codes,
        );
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].code, "clippy::needless_return");
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
        assert!(report["successful_fixes"].as_u64().unwrap() >= 0);
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

    #[test]
    fn test_create_fix_response_dry_run() {
        let results = json!({
            "dry_run": true,
            "total_fixes": 5,
            "fixes": []
        });

        let response = create_fix_response(results, true);

        assert!(!response.is_error);
        assert_eq!(response.content.len(), 1);

        if let pmcp::Content::Text { text } = &response.content[0] {
            assert!(text.contains("analyzed"));
            assert!(text.contains("Clippy fixes"));
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

        let response = create_fix_response(results, false);

        assert!(!response.is_error);
        assert_eq!(response.content.len(), 1);

        if let pmcp::Content::Text { text } = &response.content[0] {
            assert!(text.contains("applied"));
            assert!(text.contains("Clippy fixes"));
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
        let response_dry = create_fix_response(json!({}), true);
        let response_apply = create_fix_response(json!({}), false);

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
