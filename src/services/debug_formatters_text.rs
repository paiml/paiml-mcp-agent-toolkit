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

    push_root_cause_text(&mut output, analysis);

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

    push_evidence_summary_text(&mut output, analysis);

    Ok(output)
}

/// Render the Root Cause line, saying so when the cause was withheld.
///
/// Extracted to keep `format_text` under the cognitive-complexity gate. See
/// `push_root_cause_markdown` for why the absent case is stated explicitly
/// rather than omitted (GH #637).
fn push_root_cause_text(output: &mut String, analysis: &DebugAnalysis) {
    use crate::cli::colors as c;

    output.push_str(&format!("{}\n", c::label("Root Cause:")));
    match &analysis.root_cause {
        Some(root_cause) => output.push_str(&format!("   {root_cause}\n\n")),
        None => output.push_str(
            "   Not determined — no source location matching the reported issue\n   \
             was found, so the evidence below describes the repository as a whole.\n   \
             Phrase the issue with terms that appear in the code (identifiers,\n   \
             module or file names, a log string) to get a located analysis.\n\n",
        ),
    }
}

/// Render the Evidence Summary block.
///
/// Extracted so `format_text` stays under the cognitive-complexity gate: the
/// per-metric colouring is a run of independent conditionals that inflate the
/// enclosing function without adding logic of its own.
fn push_evidence_summary_text(output: &mut String, analysis: &DebugAnalysis) {
    use crate::cli::colors as c;

    let s = &analysis.evidence_summary;
    output.push_str(&format!("{}\n", c::label("Evidence Summary:")));
    output.push_str(&format!(
        "   Complexity violations: {}\n",
        c::number(&s.complexity_violations.to_string())
    ));
    output.push_str(&format!(
        "   SATD markers: {}\n",
        c::number(&s.satd_markers.to_string())
    ));
    output.push_str(&format!("   TDG score: {}\n", tdg_text(s)));
    output.push_str(&format!("   Git churn: {}\n", churn_text(s.git_churn_high)));
    if s.evoscore_trajectory != 0.0 {
        output.push_str(&format!(
            "   EvoScore trajectory: {}\n",
            evoscore_text(s.evoscore_trajectory)
        ));
    }
    if s.coverage_delta != 0.0 {
        output.push_str(&format!(
            "   Coverage delta: {}\n",
            coverage_text(s.coverage_delta)
        ));
    }
}

/// TDG is not part of the v2 evidence set, so a numeric score would be a
/// failing grade for an unmeasured metric (GH #637).
fn tdg_text(s: &EvidenceSummary) -> String {
    use crate::cli::colors as c;

    if s.tdg_measured {
        c::score(s.tdg_score, 100.0, 80.0, 60.0)
    } else {
        "not measured".to_string()
    }
}

fn churn_text(high: bool) -> String {
    use crate::cli::colors as c;

    if high {
        format!("{}HIGH{}", c::RED, c::RESET)
    } else {
        format!("{}NORMAL{}", c::GREEN, c::RESET)
    }
}

fn evoscore_text(trajectory: f64) -> String {
    use crate::cli::colors as c;

    let (colour, label) = if trajectory >= 0.5 {
        (c::GREEN, "improving")
    } else if trajectory >= 0.0 {
        (c::YELLOW, "mixed")
    } else {
        (c::RED, "regressing")
    };
    format!("{colour}{trajectory:.3} ({label}){}", c::RESET)
}

fn coverage_text(delta: f64) -> String {
    use crate::cli::colors as c;

    if delta >= 0.0 {
        format!("{}+{delta:.1}% (above baseline){}", c::GREEN, c::RESET)
    } else {
        format!("{}{delta:.1}% (below baseline){}", c::RED, c::RESET)
    }
}
