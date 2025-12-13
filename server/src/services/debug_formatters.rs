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
    fn test_format_json() {
        let analysis = create_test_analysis();
        let output = format_json(&analysis).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(parsed["issue"].is_string());
        assert!(parsed["whys"].is_array());
    }

    #[test]
    fn test_format_markdown() {
        let analysis = create_test_analysis();
        let output = format_markdown(&analysis).unwrap();

        assert!(output.contains("# Five Whys Root Cause Analysis"));
        assert!(output.contains("## Why 1:"));
        assert!(output.contains("**Hypothesis**:"));
    }
}
