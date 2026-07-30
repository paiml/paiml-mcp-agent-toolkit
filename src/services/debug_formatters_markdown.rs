/// Format analysis as Markdown
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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

    push_root_cause_markdown(&mut output, analysis);

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

    push_evidence_summary_markdown(&mut output, analysis);

    Ok(output)
}

/// Render the Root Cause section, saying so when the cause was withheld.
///
/// Extracted to keep `format_markdown` under the cognitive-complexity gate.
///
/// The analyzer returns `None` when no evidence pertained to the reported issue
/// (GH #637). Silently dropping the section left the reader to guess, and the
/// behaviour before that — printing a hypothesis derived only from repo-wide
/// metrics — asserted a cause nothing supported.
fn push_root_cause_markdown(output: &mut String, analysis: &DebugAnalysis) {
    output.push_str("## Root Cause\n\n");
    match &analysis.root_cause {
        Some(root_cause) => output.push_str(&format!("{root_cause}\n\n")),
        None => output.push_str(
            "**Not determined.** No source location matching the reported issue was \
             found, so the evidence below describes the repository as a whole rather \
             than this issue. A root cause is withheld rather than inferred from \
             repo-wide metrics.\n\n\
             To get a located analysis, phrase the issue with terms that appear in the \
             code (identifiers, module or file names, a log string).\n\n",
        ),
    }
    output.push_str("---\n\n");
}


/// Render the Evidence Summary table.
///
/// Extracted so `format_markdown` stays under the cognitive-complexity gate:
/// the per-metric status icons are six independent conditionals that inflate the
/// enclosing function without adding any branching logic of their own. The file
/// was already over the limit (cognitive 36 against a ceiling of 25) before the
/// GH #637 root-cause changes touched it.
fn push_evidence_summary_markdown(output: &mut String, analysis: &DebugAnalysis) {
    let s = &analysis.evidence_summary;

    output.push_str("## Evidence Summary\n\n");
    output.push_str("| Metric | Value | Status |\n");
    output.push_str("|--------|-------|--------|\n");
    output.push_str(&format!(
        "| Complexity violations | {} | {} |\n",
        s.complexity_violations,
        warn_icon(s.complexity_violations > 0)
    ));
    output.push_str(&format!(
        "| SATD markers | {} | {} |\n",
        s.satd_markers,
        warn_icon(s.satd_markers > 0)
    ));
    if s.tdg_measured {
        output.push_str(&format!(
            "| TDG score | {:.1}/100 | {} |\n",
            s.tdg_score,
            tdg_icon(s.tdg_score)
        ));
    } else {
        output.push_str("| TDG score | not measured | \u{2014} |\n");
    }
    output.push_str(&format!(
        "| Git churn | {} | {} |\n",
        if s.git_churn_high { "HIGH" } else { "NORMAL" },
        warn_icon(s.git_churn_high)
    ));
    if s.evoscore_trajectory != 0.0 {
        let (label, icon) = evoscore_status(s.evoscore_trajectory);
        output.push_str(&format!(
            "| EvoScore trajectory | {:.3} ({label}) | {icon} |\n",
            s.evoscore_trajectory
        ));
    }
    if s.coverage_delta != 0.0 {
        let (label, icon) = coverage_status(s.coverage_delta);
        output.push_str(&format!(
            "| Coverage delta | {:.1}% ({label}) | {icon} |\n",
            s.coverage_delta
        ));
    }
}

/// Warning icon when the metric is in a bad state, tick otherwise.
fn warn_icon(bad: bool) -> &'static str {
    if bad {
        "\u{26a0}\u{fe0f}"
    } else {
        "\u{2705}"
    }
}

/// Three-band status icon for a 0-100 TDG score.
fn tdg_icon(score: f64) -> &'static str {
    if score < 50.0 {
        "\u{274c}"
    } else if score < 85.0 {
        "\u{26a0}\u{fe0f}"
    } else {
        "\u{2705}"
    }
}

/// Label and icon for an EvoScore trajectory.
fn evoscore_status(trajectory: f64) -> (&'static str, &'static str) {
    if trajectory >= 0.5 {
        ("improving", "\u{2705}")
    } else if trajectory >= 0.0 {
        ("mixed", "\u{26a0}\u{fe0f}")
    } else {
        ("regressing", "\u{274c}")
    }
}

/// Label and icon for a coverage delta against the baseline.
fn coverage_status(delta: f64) -> (&'static str, &'static str) {
    if delta >= 0.0 {
        ("above baseline", "\u{2705}")
    } else {
        ("below baseline", "\u{274c}")
    }
}
