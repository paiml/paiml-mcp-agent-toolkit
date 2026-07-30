fn strip_ansi(s: &str) -> String {
    let re = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
    re.replace_all(s, "").to_string()
}

fn create_test_analysis() -> DebugAnalysis {
    let mut analysis = DebugAnalysis::new("Test issue".to_string());

    let mut why = WhyIteration::new(
        1,
        "Why did this happen?".to_string(),
        "Because of complexity".to_string(),
    )
    .with_confidence(0.8);

    why.add_evidence(Evidence::new(
        EvidenceSource::Complexity,
        PathBuf::from("test.rs"),
        "complexity".to_string(),
        serde_json::json!({"value": 30}),
        "High complexity".to_string(),
    ));

    analysis.whys.push(why);
    analysis.root_cause = Some("Root cause description".to_string());
    analysis
        .recommendations
        .push(Recommendation::high("Fix the issue".to_string(), None));

    analysis
}

fn create_empty_analysis() -> DebugAnalysis {
    DebugAnalysis::new("Empty issue".to_string())
}

fn create_multi_why_analysis() -> DebugAnalysis {
    let mut analysis = DebugAnalysis::new("Complex issue".to_string());

    // Add multiple why iterations with varying confidence
    for i in 1..=5 {
        let mut why = WhyIteration::new(
            i,
            format!("Why {} question?", i),
            format!("Hypothesis for why {}", i),
        )
        .with_confidence(0.9 - (i as f64 * 0.1));

        if i % 2 == 0 {
            why.add_evidence(Evidence::new(
                EvidenceSource::SATD,
                PathBuf::from(format!("file_{}.rs", i)),
                "satd".to_string(),
                serde_json::json!({"count": i}),
                format!("Found {} SATD markers", i),
            ));
        }

        analysis.whys.push(why);
    }

    analysis.root_cause = Some("Deep nested root cause".to_string());
    analysis
        .recommendations
        .push(Recommendation::high("High priority fix".to_string(), None));
    analysis.recommendations.push(Recommendation::medium(
        "Medium priority refactor".to_string(),
        None,
    ));
    analysis.recommendations.push(Recommendation::low(
        "Low priority cleanup".to_string(),
        None,
    ));

    analysis.evidence_summary.complexity_violations = 5;
    analysis.evidence_summary.satd_markers = 10;
    analysis.evidence_summary.tdg_score = 65.5;
    analysis.evidence_summary.tdg_measured = true;
    analysis.evidence_summary.git_churn_high = true;

    analysis
}

// ========================================================================
// format_text Tests
// ========================================================================

#[test]
fn test_format_text() {
    let analysis = create_test_analysis();
    let output = format_text(&analysis).unwrap();

    assert!(output.contains("PMAT Five Whys"));
    assert!(output.contains("Why 1:"));
    assert!(output.contains("Root Cause:"));
    assert!(output.contains("Recommendations:"));
}

#[test]
fn test_format_text_empty_analysis() {
    let analysis = create_empty_analysis();
    let output = format_text(&analysis).unwrap();

    assert!(output.contains("PMAT Five Whys"));
    assert!(output.contains("Issue: Empty issue"));
    // Should not contain root cause section since none set
    assert!(!output.contains("🎯 Root Cause:"));
    // Should not contain recommendations section
    assert!(!output.contains("💡 Recommendations:"));
}

#[test]
fn test_format_text_multi_why() {
    let analysis = create_multi_why_analysis();
    let output = format_text(&analysis).unwrap();

    assert!(output.contains("Why 1:"));
    assert!(output.contains("Why 2:"));
    assert!(output.contains("Why 3:"));
    assert!(output.contains("Why 4:"));
    assert!(output.contains("Why 5:"));
}

#[test]
fn test_format_text_evidence_display() {
    let analysis = create_test_analysis();
    let output = strip_ansi(&format_text(&analysis).unwrap());

    assert!(output.contains("Evidence:"));
    assert!(output.contains("High complexity"));
    assert!(output.contains("test.rs"));
}

#[test]
fn test_format_text_confidence_display() {
    let analysis = create_test_analysis();
    let output = strip_ansi(&format_text(&analysis).unwrap());

    assert!(output.contains("Confidence:"));
    assert!(output.contains("80.0%"));
}

#[test]
fn test_format_text_recommendation_priorities() {
    let analysis = create_multi_why_analysis();
    let output = strip_ansi(&format_text(&analysis).unwrap());

    assert!(output.contains("[HIGH]"));
    assert!(output.contains("[MEDIUM]"));
    assert!(output.contains("[LOW]"));
}

#[test]
fn test_format_text_evidence_summary() {
    let analysis = create_multi_why_analysis();
    let output = strip_ansi(&format_text(&analysis).unwrap());

    assert!(output.contains("Evidence Summary:"));
    assert!(output.contains("Complexity violations:"));
    assert!(output.contains("5"));
    assert!(output.contains("SATD markers:"));
    assert!(output.contains("10"));
    assert!(output.contains("TDG score:"));
    assert!(output.contains("65.5"));
    assert!(output.contains("100.0"));
    assert!(output.contains("Git churn: HIGH"));
}

#[test]
fn test_format_text_normal_churn() {
    let mut analysis = create_test_analysis();
    analysis.evidence_summary.git_churn_high = false;
    let output = strip_ansi(&format_text(&analysis).unwrap());

    assert!(output.contains("Git churn: NORMAL"));
}

// ========================================================================
// format_json Tests
// ========================================================================

#[test]
fn test_format_json() {
    let analysis = create_test_analysis();
    let output = format_json(&analysis).unwrap();

    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(parsed["issue"].is_string());
    assert!(parsed["whys"].is_array());
}

#[test]
fn test_format_json_empty_analysis() {
    let analysis = create_empty_analysis();
    let output = format_json(&analysis).unwrap();

    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed["issue"], "Empty issue");
    assert_eq!(parsed["whys"].as_array().unwrap().len(), 0);
    assert!(parsed["root_cause"].is_null());
}

#[test]
fn test_format_json_roundtrip() {
    let analysis = create_test_analysis();
    let output = format_json(&analysis).unwrap();

    let parsed: DebugAnalysis = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed.issue, "Test issue");
    assert_eq!(parsed.whys.len(), 1);
    assert_eq!(
        parsed.root_cause,
        Some("Root cause description".to_string())
    );
}

#[test]
fn test_format_json_multi_why() {
    let analysis = create_multi_why_analysis();
    let output = format_json(&analysis).unwrap();

    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed["whys"].as_array().unwrap().len(), 5);
    assert_eq!(parsed["recommendations"].as_array().unwrap().len(), 3);
}

#[test]
fn test_format_json_evidence_summary() {
    let analysis = create_multi_why_analysis();
    let output = format_json(&analysis).unwrap();

    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed["evidence_summary"]["complexity_violations"], 5);
    assert_eq!(parsed["evidence_summary"]["satd_markers"], 10);
    assert_eq!(parsed["evidence_summary"]["git_churn_high"], true);
}

// ========================================================================
// format_markdown Tests
// ========================================================================

#[test]
fn test_format_markdown() {
    let analysis = create_test_analysis();
    let output = format_markdown(&analysis).unwrap();

    assert!(output.contains("# Five Whys Root Cause Analysis"));
    assert!(output.contains("## Why 1:"));
    assert!(output.contains("**Hypothesis**:"));
}

#[test]
fn test_format_markdown_empty_analysis() {
    let analysis = create_empty_analysis();
    let output = format_markdown(&analysis).unwrap();

    assert!(output.contains("# Five Whys Root Cause Analysis"));
    assert!(output.contains("**Issue**: Empty issue"));
    // The Root Cause section is always rendered now: when the analyzer withheld
    // a cause it must say so explicitly rather than silently omit the heading,
    // which left readers unable to tell "no cause" from "section forgotten"
    // (GH #637).
    assert!(output.contains("## Root Cause"));
    assert!(
        output.contains("**Not determined.**"),
        "a withheld cause must be stated, got: {output}"
    );
    // Should not contain recommendations section
    assert!(!output.contains("## Recommendations"));
}

#[test]
fn test_format_markdown_table() {
    let analysis = create_multi_why_analysis();
    let output = format_markdown(&analysis).unwrap();

    // Check markdown table structure
    assert!(output.contains("| Metric | Value | Status |"));
    assert!(output.contains("|--------|-------|--------|"));
}

#[test]
fn test_format_markdown_evidence_section() {
    let analysis = create_test_analysis();
    let output = format_markdown(&analysis).unwrap();

    assert!(output.contains("**Evidence**:"));
    assert!(output.contains("`test.rs`"));
}

#[test]
fn test_format_markdown_confidence() {
    let analysis = create_test_analysis();
    let output = format_markdown(&analysis).unwrap();

    assert!(output.contains("**Confidence**: 80%"));
}

#[test]
fn test_format_markdown_recommendations() {
    let analysis = create_multi_why_analysis();
    let output = format_markdown(&analysis).unwrap();

    assert!(output.contains("## Recommendations"));
    assert!(output.contains("**HIGH**"));
    assert!(output.contains("**MEDIUM**"));
    assert!(output.contains("**LOW**"));
}

#[test]
fn test_format_markdown_status_icons() {
    let analysis = create_multi_why_analysis();
    let output = format_markdown(&analysis).unwrap();

    // With violations, should show warning icons
    assert!(output.contains("⚠️")); // Warning for violations
}

#[test]
fn test_format_markdown_tdg_score_icons() {
    // Low TDG score - should show red icon
    let mut analysis = create_test_analysis();
    analysis.evidence_summary.tdg_score = 30.0;
    analysis.evidence_summary.tdg_measured = true;
    // Without this the row reads "not measured": a numeric score is only shown
    // when TDG evidence actually produced one (GH #637).
    analysis.evidence_summary.tdg_measured = true;
    let output = format_markdown(&analysis).unwrap();
    assert!(output.contains("❌"));

    // Medium TDG score - should show warning
    analysis.evidence_summary.tdg_score = 70.0;
    analysis.evidence_summary.tdg_measured = true;
    let output = format_markdown(&analysis).unwrap();
    assert!(output.contains("⚠️"));

    // High TDG score - should show check
    analysis.evidence_summary.tdg_score = 90.0;
    analysis.evidence_summary.tdg_measured = true;
    let output = format_markdown(&analysis).unwrap();
    assert!(output.contains("✅"));
}

#[test]
fn test_format_markdown_no_violations() {
    let mut analysis = create_test_analysis();
    analysis.evidence_summary.complexity_violations = 0;
    analysis.evidence_summary.satd_markers = 0;
    analysis.evidence_summary.tdg_score = 95.0;
    analysis.evidence_summary.tdg_measured = true;
    analysis.evidence_summary.git_churn_high = false;

    let output = format_markdown(&analysis).unwrap();
    // Should have all green checkmarks
    assert!(output.contains("✅"));
}

#[test]
fn test_format_markdown_multi_why() {
    let analysis = create_multi_why_analysis();
    let output = format_markdown(&analysis).unwrap();

    assert!(output.contains("## Why 1:"));
    assert!(output.contains("## Why 2:"));
    assert!(output.contains("## Why 3:"));
    assert!(output.contains("## Why 4:"));
    assert!(output.contains("## Why 5:"));
}

#[test]
fn test_format_markdown_separators() {
    let analysis = create_test_analysis();
    let output = format_markdown(&analysis).unwrap();

    // Should have horizontal rules between sections
    assert!(output.contains("---"));
}
