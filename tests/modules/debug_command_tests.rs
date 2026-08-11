// GREEN Tests for pmat debug command - Five Whys Root Cause Analysis
//
// Tests now use real implementations from GREEN phase

use pmat::models::debug_analysis::*;
use pmat::services::debug_formatters::{format_json, format_markdown, format_text};
use pmat::services::five_whys_analyzer::FiveWhysAnalyzer;
use serde_json::json;
use std::path::Path;

// Helper function for creating test evidence
fn create_test_evidence(source: EvidenceSource, value: serde_json::Value) -> Evidence {
    Evidence::new(
        source,
        std::path::PathBuf::from("test.rs"),
        "test_metric".to_string(),
        value,
        "Test interpretation".to_string(),
    )
}

/// True when at least one "why" carried evidence that pinned the issue to a
/// place in the source. `extract_root_cause` withholds the root cause unless
/// this holds, so it is the precondition every reported root cause must meet.
fn issue_was_located(analysis: &DebugAnalysis) -> bool {
    analysis.whys.iter().any(|why| {
        why.evidence
            .iter()
            .any(|e| e.source == EvidenceSource::IssueLocation)
    })
}

// ============================================================================
// SECTION 1: Basic Five Whys Execution
// ============================================================================

/// TEST 1: Five Whys should execute with default depth of 5
#[tokio::test]
async fn test_01_five_whys_executes_with_default_depth() {
    let analyzer = FiveWhysAnalyzer::new();
    let result = analyzer
        .analyze("Stack overflow in parser", Path::new("."), 5)
        .await;

    assert!(result.is_ok());
    let analysis = result.unwrap();
    assert!(analysis.whys.len() <= 5); // May terminate early
    assert_eq!(analysis.issue, "Stack overflow in parser");
}

/// TEST 2: Five Whys should support custom depth
#[tokio::test]
async fn test_02_five_whys_supports_custom_depth() {
    let analyzer = FiveWhysAnalyzer::new();

    // Test depth = 3
    let result = analyzer
        .analyze("Memory leak", Path::new("."), 3)
        .await
        .unwrap();
    assert!(result.whys.len() <= 3);

    // Test depth = 10
    let result = analyzer
        .analyze("API timeout", Path::new("."), 10)
        .await
        .unwrap();
    assert!(result.whys.len() <= 10);
}

/// TEST 3: Five Whys should validate depth range
#[tokio::test]
async fn test_03_five_whys_validates_depth_range() {
    let analyzer = FiveWhysAnalyzer::new();

    // Depth = 0 should fail
    let result = analyzer.analyze("Issue", Path::new("."), 0).await;
    assert!(result.is_err());

    // Depth > 10 should fail
    let result = analyzer.analyze("Issue", Path::new("."), 11).await;
    assert!(result.is_err());
}

/// TEST 4: Each Why iteration should have required fields
#[tokio::test]
async fn test_04_why_iteration_has_required_fields() {
    let analyzer = FiveWhysAnalyzer::new();
    let result = analyzer
        .analyze("Test issue", Path::new("."), 5)
        .await
        .unwrap();

    for why in &result.whys {
        assert!(why.depth >= 1 && why.depth <= 5);
        assert!(!why.question.is_empty());
        assert!(!why.hypothesis.is_empty());
        assert!(why.confidence >= 0.0 && why.confidence <= 1.0);
    }
}

// ============================================================================
// SECTION 2: Evidence Gathering
// ============================================================================

/// TEST 5: Should gather evidence from multiple PMAT services
#[tokio::test]
async fn test_05_gathers_evidence_from_services() {
    let analyzer = FiveWhysAnalyzer::new();
    let result = analyzer
        .analyze("High complexity", Path::new("."), 5)
        .await
        .unwrap();

    // Should have at least some evidence
    let total_evidence: usize = result.whys.iter().map(|w| w.evidence.len()).sum();
    assert!(total_evidence > 0);
}

/// TEST 6-9: Evidence type tests (simplified for GREEN phase)
#[tokio::test]
async fn test_06_09_evidence_gathering() {
    let analyzer = FiveWhysAnalyzer::new();
    let result = analyzer.analyze("Issue", Path::new("."), 3).await.unwrap();

    // Verify evidence has required fields
    for why in &result.whys {
        for evidence in &why.evidence {
            assert!(!evidence.metric.is_empty());
            assert!(!evidence.interpretation.is_empty());
        }
    }
}

// ============================================================================
// SECTION 3: Confidence Scoring
// ============================================================================

/// TEST 10: Confidence should increase with more evidence
#[tokio::test]
async fn test_10_confidence_increases_with_evidence() {
    let analyzer = FiveWhysAnalyzer::new();

    let evidence_few = vec![create_test_evidence(
        EvidenceSource::Complexity,
        json!({"value": 30}),
    )];
    let confidence_few = analyzer.calculate_confidence(&evidence_few).unwrap();

    let evidence_many = vec![
        create_test_evidence(EvidenceSource::Complexity, json!({"value": 30})),
        create_test_evidence(EvidenceSource::SATD, json!({"count": 5})),
        create_test_evidence(EvidenceSource::TDG, json!(35.0)),
    ];
    let confidence_many = analyzer.calculate_confidence(&evidence_many).unwrap();

    assert!(confidence_many >= confidence_few);
}

/// TEST 11: High severity evidence should increase confidence
#[tokio::test]
async fn test_11_high_severity_increases_confidence() {
    let analyzer = FiveWhysAnalyzer::new();

    // Low complexity
    let evidence_low = vec![create_test_evidence(
        EvidenceSource::Complexity,
        json!({"value": 10, "threshold": 20}),
    )];
    let confidence_low = analyzer.calculate_confidence(&evidence_low).unwrap();

    // High complexity
    let evidence_high = vec![create_test_evidence(
        EvidenceSource::Complexity,
        json!({"value": 50, "threshold": 20}),
    )];
    let confidence_high = analyzer.calculate_confidence(&evidence_high).unwrap();

    assert!(confidence_high >= confidence_low);
}

/// TEST 12: Confidence should be bounded 0.0-1.0
#[tokio::test]
async fn test_12_confidence_bounded() {
    let analyzer = FiveWhysAnalyzer::new();

    // Empty evidence
    let confidence = analyzer.calculate_confidence(&[]).unwrap();
    assert!((0.0..=1.0).contains(&confidence));

    // Extreme evidence
    let evidence = vec![
        create_test_evidence(EvidenceSource::Complexity, json!({"value": 1000})),
        create_test_evidence(EvidenceSource::SATD, json!({"count": 100})),
    ];
    let confidence = analyzer.calculate_confidence(&evidence).unwrap();
    assert!((0.0..=1.0).contains(&confidence));
}

// ============================================================================
// SECTION 4-5: Root Cause & Recommendations
// ============================================================================

/// TEST 13-17: Root cause and recommendations (simplified)
///
/// #709 / #637: this used to assert `result.root_cause.is_some()` flat out,
/// which pinned the behaviour five-whys was fixed away from — every run, on
/// any input, got *a* root cause, and for an issue the analyzer could not
/// locate anywhere in the tree that "root cause" was a repo-wide signal
/// ("Frequent changes indicate unstable or poorly understood code") dressed up
/// as a finding about the reported defect. The analyzer now withholds instead.
/// The assertion below is the anti-fabrication contract itself: a root cause
/// may be reported only when some "why" actually located the issue in the
/// source, and never for an issue nothing in the tree matched.
#[tokio::test]
async fn test_13_17_root_cause_and_recommendations() {
    let analyzer = FiveWhysAnalyzer::new();
    let result = analyzer.analyze("Issue", Path::new("."), 5).await.unwrap();

    assert!(
        result.root_cause.is_none() || issue_was_located(&result),
        "root cause reported without locating the issue: {:?}",
        result.root_cause
    );
    assert!(!result.recommendations.is_empty());

    // Check recommendations have required fields
    for rec in &result.recommendations {
        assert!(!rec.action.is_empty());
    }
}

// ============================================================================
// SECTION 6: Output Formats
// ============================================================================

/// TEST 18: Should format as text output
#[tokio::test]
async fn test_18_formats_text_output() {
    let analyzer = FiveWhysAnalyzer::new();
    let analysis = analyzer.analyze("Test", Path::new("."), 3).await.unwrap();
    let output = format_text(&analysis).unwrap();

    assert!(output.contains("PMAT Five Whys"));
    assert!(output.contains("Why 1:"));
    assert!(output.contains("Root Cause:"));
    assert!(output.contains("Recommendations:"));
}

/// TEST 19: Should format as JSON output
#[tokio::test]
async fn test_19_formats_json_output() {
    let analyzer = FiveWhysAnalyzer::new();
    let analysis = analyzer.analyze("Test", Path::new("."), 3).await.unwrap();
    let output = format_json(&analysis).unwrap();

    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert!(parsed["issue"].is_string());
    assert!(parsed["whys"].is_array());
}

/// TEST 20: Should format as Markdown output
#[tokio::test]
async fn test_20_formats_markdown_output() {
    let analyzer = FiveWhysAnalyzer::new();
    let analysis = analyzer.analyze("Test", Path::new("."), 3).await.unwrap();
    let output = format_markdown(&analysis).unwrap();

    assert!(output.contains("# Five Whys Root Cause Analysis"));
    assert!(output.contains("## Why 1:"));
    assert!(output.contains("**Hypothesis**:"));
}

// ============================================================================
// SECTION 7: Error Handling
// ============================================================================

/// TEST 21: Should handle non-existent path gracefully
#[tokio::test]
async fn test_21_handles_nonexistent_path() {
    let analyzer = FiveWhysAnalyzer::new();
    let result = analyzer
        .analyze("Issue", Path::new("/nonexistent/path/12345"), 5)
        .await;

    assert!(result.is_err());
}

/// TEST 22: Should handle empty issue description
#[tokio::test]
async fn test_22_handles_empty_issue() {
    let analyzer = FiveWhysAnalyzer::new();
    let result = analyzer.analyze("", Path::new("."), 5).await;

    assert!(result.is_err());
}

/// TEST 23-26: Integration tests (simplified)
#[tokio::test]
async fn test_23_26_integration() {
    let analyzer = FiveWhysAnalyzer::new();
    let result = analyzer
        .analyze("Real issue", Path::new("."), 5)
        .await
        .unwrap();

    // Verify complete analysis structure. #709 / #637: `root_cause.is_some()`
    // used to be asserted here too; withholding the root cause must not cost
    // the rest of the analysis, and a reported root cause must be earned by
    // an actual location in the source — see test_13_17 above.
    assert!(!result.issue.is_empty());
    assert!(!result.whys.is_empty());
    assert!(
        result.root_cause.is_none() || issue_was_located(&result),
        "root cause reported without locating the issue: {:?}",
        result.root_cause
    );
    assert!(!result.recommendations.is_empty());
}
