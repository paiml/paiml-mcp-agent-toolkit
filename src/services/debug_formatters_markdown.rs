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
