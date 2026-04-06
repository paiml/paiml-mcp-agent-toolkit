#![cfg_attr(coverage_nightly, coverage(off))]
use std::fmt::Write;

use super::super::TdgScore;
use super::helpers::progress_bar;

/// Format TDG score for human-readable console output.
///
/// Creates a visually appealing boxed display showing the TDG score,
/// grade, language confidence, and detailed breakdown of score components.
///
/// # Arguments
/// * `score` - The TDG score to format
///
/// # Returns
/// A formatted string with boxed output suitable for terminal display
///
/// # Example
/// ```ignore
/// use pmat::tdg::{TdgScore, Grade};
/// let score = TdgScore::new(85.5, Grade::A, 0.95);
/// let output = format_human(&score);
/// assert!(output.contains("85.5/100 (A)"));
/// ```ignore
#[must_use]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn format_human(score: &TdgScore) -> String {
    let mut output = String::new();

    writeln!(
        output,
        "╭─────────────────────────────────────────────────╮"
    )
    .expect("Writing to String buffer cannot fail");
    if let Some(path) = &score.file_path {
        writeln!(
            output,
            "│  TDG Score Report: {:30} │",
            path.display()
                .to_string()
                .chars()
                .take(30)
                .collect::<String>()
        )
        .expect("Writing to String buffer cannot fail");
    } else {
        writeln!(output, "│  TDG Score Report: Code Analysis               │")
            .expect("Writing to String buffer cannot fail");
    }
    writeln!(
        output,
        "├─────────────────────────────────────────────────┤"
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "│  Overall Score: {:.1}/100 ({})                  │",
        score.total, score.grade
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "│  Language: {} (confidence: {:.0}%)             │",
        score.language,
        score.confidence * 100.0
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "│                                                 │"
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "│  📊 Breakdown:                                  │"
    )
    .expect("Writing to String buffer cannot fail");

    writeln!(
        output,
        "│  ├─ Structural:     {:4.1}/25  {}        │",
        score.structural_complexity,
        progress_bar(score.structural_complexity, 25.0, 10)
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "│  ├─ Semantic:       {:4.1}/20  {}        │",
        score.semantic_complexity,
        progress_bar(score.semantic_complexity, 20.0, 10)
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "│  ├─ Duplication:    {:4.1}/20  {}        │",
        score.duplication_ratio,
        progress_bar(score.duplication_ratio, 20.0, 10)
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "│  ├─ Coupling:       {:4.1}/15  {}        │",
        score.coupling_score,
        progress_bar(score.coupling_score, 15.0, 10)
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "│  ├─ Documentation:  {:4.1}/10  {}        │",
        score.doc_coverage,
        progress_bar(score.doc_coverage, 10.0, 10)
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "│  └─ Consistency:    {:4.1}/10  {}        │",
        score.consistency_score,
        progress_bar(score.consistency_score, 10.0, 10)
    )
    .expect("Writing to String buffer cannot fail");

    if !score.penalties_applied.is_empty() {
        writeln!(
            output,
            "│                                                 │"
        )
        .expect("Writing to String buffer cannot fail");
        writeln!(
            output,
            "│  🔍 Issues Found:                               │"
        )
        .expect("Writing to String buffer cannot fail");
        for penalty in &score.penalties_applied {
            let issue_line = format!("  • {}", penalty.issue);
            let truncated = if issue_line.len() > 45 {
                format!("{}...", issue_line.chars().take(42).collect::<String>())
            } else {
                issue_line
            };
            writeln!(output, "│  {truncated:47} │").expect("Writing to String buffer cannot fail");
        }
    }

    if score.total >= 90.0 {
        writeln!(
            output,
            "│                                                 │"
        )
        .expect("Writing to String buffer cannot fail");
        writeln!(output, "│  ✨ Excellent code quality! No major issues.   │")
            .expect("Writing to String buffer cannot fail");
    } else if score.total >= 75.0 {
        writeln!(
            output,
            "│                                                 │"
        )
        .expect("Writing to String buffer cannot fail");
        writeln!(output, "│  👍 Good code quality with minor improvements. │")
            .expect("Writing to String buffer cannot fail");
    } else if score.total >= 60.0 {
        writeln!(
            output,
            "│                                                 │"
        )
        .expect("Writing to String buffer cannot fail");
        writeln!(output, "│  ⚠️  Code needs improvement in several areas.  │")
            .expect("Writing to String buffer cannot fail");
    } else {
        writeln!(
            output,
            "│                                                 │"
        )
        .expect("Writing to String buffer cannot fail");
        writeln!(output, "│  🔴 Code requires significant refactoring.     │")
            .expect("Writing to String buffer cannot fail");
    }

    writeln!(
        output,
        "╰─────────────────────────────────────────────────╯"
    )
    .expect("Writing to String buffer cannot fail");

    output
}
