// Output formatters for Five Whys analysis
//
// GREEN PHASE: Minimal implementation for test formats

use crate::models::debug_analysis::*;
use anyhow::Result;

/// Format analysis as human-readable text
pub fn format_text(analysis: &DebugAnalysis) -> Result<String> {
    let mut output = String::new();

    // Header
    output.push_str("🔍 PMAT Five Whys Root Cause Analysis\n\n");
    output.push_str(&format!("Issue: {}\n\n", analysis.issue));
    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n");

    // Why iterations
    for why in &analysis.whys {
        output.push_str(&format!("Why {}: {}\n", why.depth, why.question));
        output.push_str(&format!("   ❓ Question: {}\n", why.question));
        output.push_str(&format!("   💡 Hypothesis: {}\n", why.hypothesis));

        if !why.evidence.is_empty() {
            output.push_str("   📊 Evidence:\n");
            for evidence in &why.evidence {
                output.push_str(&format!(
                    "      • {} ({})\n",
                    evidence.interpretation,
                    evidence.file.display()
                ));
            }
        }

        output.push_str(&format!(
            "   ✅ Confidence: {:.0}%\n\n",
            why.confidence * 100.0
        ));
    }

    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n");

    // Root Cause
    if let Some(root_cause) = &analysis.root_cause {
        output.push_str("🎯 Root Cause:\n");
        output.push_str(&format!("   {}\n\n", root_cause));
    }

    // Recommendations
    if !analysis.recommendations.is_empty() {
        output.push_str("💡 Recommendations:\n");
        for (i, rec) in analysis.recommendations.iter().enumerate() {
            let priority = match rec.priority {
                Priority::High => "HIGH",
                Priority::Medium => "MEDIUM",
                Priority::Low => "LOW",
            };
            output.push_str(&format!("   {}. [{}] {}\n", i + 1, priority, rec.action));
        }
        output.push('\n');
    }

    // Evidence Summary
    output.push_str("📊 Evidence Summary:\n");
    output.push_str(&format!(
        "   • Complexity violations: {}\n",
        analysis.evidence_summary.complexity_violations
    ));
    output.push_str(&format!(
        "   • SATD markers: {}\n",
        analysis.evidence_summary.satd_markers
    ));
    output.push_str(&format!(
        "   • TDG score: {:.1}/100\n",
        analysis.evidence_summary.tdg_score
    ));
    output.push_str(&format!(
        "   • Git churn: {}\n",
        if analysis.evidence_summary.git_churn_high {
            "HIGH"
        } else {
            "NORMAL"
        }
    ));

    Ok(output)
}

/// Format analysis as JSON
pub fn format_json(analysis: &DebugAnalysis) -> Result<String> {
    let json = serde_json::to_string_pretty(analysis)?;
    Ok(json)
}

/// Format analysis as Markdown
pub fn format_markdown(analysis: &DebugAnalysis) -> Result<String> {
    let mut output = String::new();

    // Header
    output.push_str("# Five Whys Root Cause Analysis\n\n");
    output.push_str(&format!("**Issue**: {}\n\n", analysis.issue));
    output.push_str("---\n\n");

    // Why iterations
    for why in &analysis.whys {
        output.push_str(&format!("## Why {}: {}\n\n", why.depth, why.question));
        output.push_str(&format!("**Hypothesis**: {}\n", why.hypothesis));
        output.push_str(&format!(
            "**Confidence**: {:.0}%\n\n",
            why.confidence * 100.0
        ));

        if !why.evidence.is_empty() {
            output.push_str("**Evidence**:\n");
            for evidence in &why.evidence {
                output.push_str(&format!(
                    "- {} (`{}`)\n",
                    evidence.interpretation,
                    evidence.file.display()
                ));
            }
            output.push('\n');
        }

        output.push_str("---\n\n");
    }

    // Root Cause
    if let Some(root_cause) = &analysis.root_cause {
        output.push_str("## Root Cause\n\n");
        output.push_str(&format!("{}\n\n", root_cause));
        output.push_str("---\n\n");
    }

    // Recommendations
    if !analysis.recommendations.is_empty() {
        output.push_str("## Recommendations\n\n");
        for (i, rec) in analysis.recommendations.iter().enumerate() {
            let priority = match rec.priority {
                Priority::High => "**HIGH**",
                Priority::Medium => "**MEDIUM**",
                Priority::Low => "**LOW**",
            };
            output.push_str(&format!("{}. {}: {}\n", i + 1, priority, rec.action));
        }
        output.push('\n');
    }

    // Evidence Summary
    output.push_str("## Evidence Summary\n\n");
    output.push_str("| Metric | Value | Status |\n");
    output.push_str("|--------|-------|--------|\n");
    output.push_str(&format!(
        "| Complexity violations | {} | {} |\n",
        analysis.evidence_summary.complexity_violations,
        if analysis.evidence_summary.complexity_violations > 0 {
            "⚠️"
        } else {
            "✅"
        }
    ));
    output.push_str(&format!(
        "| SATD markers | {} | {} |\n",
        analysis.evidence_summary.satd_markers,
        if analysis.evidence_summary.satd_markers > 0 {
            "⚠️"
        } else {
            "✅"
        }
    ));
    output.push_str(&format!(
        "| TDG score | {:.1}/100 | {} |\n",
        analysis.evidence_summary.tdg_score,
        if analysis.evidence_summary.tdg_score < 50.0 {
            "❌"
        } else if analysis.evidence_summary.tdg_score < 85.0 {
            "⚠️"
        } else {
            "✅"
        }
    ));
    output.push_str(&format!(
        "| Git churn | {} | {} |\n",
        if analysis.evidence_summary.git_churn_high {
            "HIGH"
        } else {
            "NORMAL"
        },
        if analysis.evidence_summary.git_churn_high {
            "⚠️"
        } else {
            "✅"
        }
    ));

    Ok(output)
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

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
        let output = format_text(&analysis).unwrap();

        assert!(output.contains("📊 Evidence:"));
        assert!(output.contains("High complexity"));
        assert!(output.contains("test.rs"));
    }

    #[test]
    fn test_format_text_confidence_display() {
        let analysis = create_test_analysis();
        let output = format_text(&analysis).unwrap();

        assert!(output.contains("✅ Confidence: 80%"));
    }

    #[test]
    fn test_format_text_recommendation_priorities() {
        let analysis = create_multi_why_analysis();
        let output = format_text(&analysis).unwrap();

        assert!(output.contains("[HIGH]"));
        assert!(output.contains("[MEDIUM]"));
        assert!(output.contains("[LOW]"));
    }

    #[test]
    fn test_format_text_evidence_summary() {
        let analysis = create_multi_why_analysis();
        let output = format_text(&analysis).unwrap();

        assert!(output.contains("📊 Evidence Summary:"));
        assert!(output.contains("Complexity violations: 5"));
        assert!(output.contains("SATD markers: 10"));
        assert!(output.contains("TDG score: 65.5/100"));
        assert!(output.contains("Git churn: HIGH"));
    }

    #[test]
    fn test_format_text_normal_churn() {
        let mut analysis = create_test_analysis();
        analysis.evidence_summary.git_churn_high = false;
        let output = format_text(&analysis).unwrap();

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
        // Should not contain root cause section
        assert!(!output.contains("## Root Cause"));
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
        let output = format_markdown(&analysis).unwrap();
        assert!(output.contains("❌"));

        // Medium TDG score - should show warning
        analysis.evidence_summary.tdg_score = 70.0;
        let output = format_markdown(&analysis).unwrap();
        assert!(output.contains("⚠️"));

        // High TDG score - should show check
        analysis.evidence_summary.tdg_score = 90.0;
        let output = format_markdown(&analysis).unwrap();
        assert!(output.contains("✅"));
    }

    #[test]
    fn test_format_markdown_no_violations() {
        let mut analysis = create_test_analysis();
        analysis.evidence_summary.complexity_violations = 0;
        analysis.evidence_summary.satd_markers = 0;
        analysis.evidence_summary.tdg_score = 95.0;
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
}
