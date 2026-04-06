/// Format analysis as human-readable text
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn format_text(analysis: &DebugAnalysis) -> Result<String> {
    use crate::cli::colors as c;

    let mut output = String::new();

    // Header
    output.push_str(&format!(
        "{}\n\n",
        c::header("PMAT Five Whys Root Cause Analysis")
    ));
    output.push_str(&format!("Issue: {}\n\n", analysis.issue));
    output.push_str(&format!("{}\n\n", c::rule()));

    // Why iterations
    for why in &analysis.whys {
        output.push_str(&format!(
            "{} {}\n",
            c::label(&format!("Why {}:", why.depth)),
            why.question
        ));
        output.push_str(&format!("   Question: {}\n", why.question));
        output.push_str(&format!("   Hypothesis: {}\n", why.hypothesis));

        if !why.evidence.is_empty() {
            output.push_str(&format!("   {}Evidence:{}\n", c::BOLD, c::RESET));
            for evidence in &why.evidence {
                output.push_str(&format!(
                    "      {} ({})\n",
                    evidence.interpretation,
                    c::path(&evidence.file.display().to_string())
                ));
            }
        }

        output.push_str(&format!(
            "   Confidence: {}\n\n",
            c::pct(why.confidence * 100.0, 80.0, 50.0)
        ));
    }

    output.push_str(&format!("{}\n\n", c::rule()));

    // Root Cause
    if let Some(root_cause) = &analysis.root_cause {
        output.push_str(&format!("{}\n", c::label("Root Cause:")));
        output.push_str(&format!("   {}\n\n", root_cause));
    }

    // Recommendations
    if !analysis.recommendations.is_empty() {
        output.push_str(&format!("{}\n", c::label("Recommendations:")));
        for (i, rec) in analysis.recommendations.iter().enumerate() {
            let priority_text = match rec.priority {
                Priority::High => format!("{}HIGH{}", c::RED, c::RESET),
                Priority::Medium => format!("{}MEDIUM{}", c::YELLOW, c::RESET),
                Priority::Low => format!("{}LOW{}", c::DIM, c::RESET),
            };
            output.push_str(&format!(
                "   {}. [{}] {}\n",
                c::number(&(i + 1).to_string()),
                priority_text,
                rec.action
            ));
        }
        output.push('\n');
    }

    // Evidence Summary
    output.push_str(&format!("{}\n", c::label("Evidence Summary:")));
    output.push_str(&format!(
        "   Complexity violations: {}\n",
        c::number(&analysis.evidence_summary.complexity_violations.to_string())
    ));
    output.push_str(&format!(
        "   SATD markers: {}\n",
        c::number(&analysis.evidence_summary.satd_markers.to_string())
    ));
    output.push_str(&format!(
        "   TDG score: {}\n",
        c::score(analysis.evidence_summary.tdg_score, 100.0, 80.0, 60.0)
    ));
    output.push_str(&format!(
        "   Git churn: {}\n",
        if analysis.evidence_summary.git_churn_high {
            format!("{}HIGH{}", c::RED, c::RESET)
        } else {
            format!("{}NORMAL{}", c::GREEN, c::RESET)
        }
    ));
    if analysis.evidence_summary.evoscore_trajectory != 0.0 {
        output.push_str(&format!(
            "   EvoScore trajectory: {}\n",
            if analysis.evidence_summary.evoscore_trajectory >= 0.5 {
                format!(
                    "{}{:.3} (improving){}",
                    c::GREEN,
                    analysis.evidence_summary.evoscore_trajectory,
                    c::RESET
                )
            } else if analysis.evidence_summary.evoscore_trajectory >= 0.0 {
                format!(
                    "{}{:.3} (mixed){}",
                    c::YELLOW,
                    analysis.evidence_summary.evoscore_trajectory,
                    c::RESET
                )
            } else {
                format!(
                    "{}{:.3} (regressing){}",
                    c::RED,
                    analysis.evidence_summary.evoscore_trajectory,
                    c::RESET
                )
            }
        ));
    }
    if analysis.evidence_summary.coverage_delta != 0.0 {
        output.push_str(&format!(
            "   Coverage delta: {}\n",
            if analysis.evidence_summary.coverage_delta >= 0.0 {
                format!(
                    "{}+{:.1}% (above baseline){}",
                    c::GREEN,
                    analysis.evidence_summary.coverage_delta,
                    c::RESET
                )
            } else {
                format!(
                    "{}{:.1}% (below baseline){}",
                    c::RED,
                    analysis.evidence_summary.coverage_delta,
                    c::RESET
                )
            }
        ));
    }

    Ok(output)
}
