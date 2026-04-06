#![cfg_attr(coverage_nightly, coverage(off))]
use std::fmt::Write;

use super::super::TdgScore;
use super::helpers::{format_metric_name, grade_description};

/// Format TDG score as Markdown output.
///
/// Creates a Markdown-formatted report suitable for documentation,
/// README files, or integration with documentation systems.
///
/// # Arguments
/// * `score` - The TDG score to format as Markdown
///
/// # Returns  
/// A Markdown string with formatted tables and sections
///
/// # Example
/// ```ignore
/// use pmat::tdg::{TdgScore, Grade};
/// let score = TdgScore::new(85.5, Grade::A, 0.95);
/// let md = format_markdown(&score);
/// assert!(md.contains("## TDG Score"));
/// ```ignore
#[must_use]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn format_markdown(score: &TdgScore) -> String {
    let mut output = String::new();

    writeln!(output, "# TDG Score Report").expect("Writing to String buffer cannot fail");
    writeln!(output).expect("Writing to String buffer cannot fail");

    if let Some(path) = &score.file_path {
        writeln!(output, "**File:** `{}`", path.display())
            .expect("Writing to String buffer cannot fail");
    }
    writeln!(
        output,
        "**Language:** {} ({}% confidence)",
        score.language,
        (score.confidence * 100.0) as u8
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "**Overall Score:** {:.1}/100 ({})",
        score.total, score.grade
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(output).expect("Writing to String buffer cannot fail");

    writeln!(output, "## Score Breakdown").expect("Writing to String buffer cannot fail");
    writeln!(output).expect("Writing to String buffer cannot fail");
    writeln!(output, "| Metric | Score | Max | Percentage |")
        .expect("Writing to String buffer cannot fail");
    writeln!(output, "|--------|-------|-----|------------|")
        .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "| Structural Complexity | {:.1} | 25.0 | {:.1}% |",
        score.structural_complexity,
        (score.structural_complexity / 25.0) * 100.0
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "| Semantic Complexity | {:.1} | 20.0 | {:.1}% |",
        score.semantic_complexity,
        (score.semantic_complexity / 20.0) * 100.0
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "| Code Duplication | {:.1} | 20.0 | {:.1}% |",
        score.duplication_ratio,
        (score.duplication_ratio / 20.0) * 100.0
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "| Coupling | {:.1} | 15.0 | {:.1}% |",
        score.coupling_score,
        (score.coupling_score / 15.0) * 100.0
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "| Documentation | {:.1} | 10.0 | {:.1}% |",
        score.doc_coverage,
        (score.doc_coverage / 10.0) * 100.0
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "| Consistency | {:.1} | 10.0 | {:.1}% |",
        score.consistency_score,
        (score.consistency_score / 10.0) * 100.0
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(output).expect("Writing to String buffer cannot fail");

    if !score.penalties_applied.is_empty() {
        writeln!(output, "## Issues Found").expect("Writing to String buffer cannot fail");
        writeln!(output).expect("Writing to String buffer cannot fail");
        for penalty in &score.penalties_applied {
            writeln!(
                output,
                "- **{}**: {} (-{:.1} points)",
                format_metric_name(&penalty.source_metric),
                penalty.issue,
                penalty.amount
            )
            .expect("Writing to String buffer cannot fail");
        }
        writeln!(output).expect("Writing to String buffer cannot fail");
    }

    writeln!(output, "## Grade Description").expect("Writing to String buffer cannot fail");
    writeln!(output).expect("Writing to String buffer cannot fail");
    writeln!(output, "{}", grade_description(score.grade))
        .expect("Writing to String buffer cannot fail");

    output
}
