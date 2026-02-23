#![cfg_attr(coverage_nightly, coverage(off))]
use std::fmt::Write;

use crate::tdg::{Comparison, Grade};

/// Format comparison between two TDG scores.
///
/// Creates a side-by-side comparison showing the differences between
/// two TDG scores, highlighting improvements or regressions.
///
/// # Arguments
/// * `comparison` - The comparison data structure containing before/after scores
///
/// # Returns
/// A formatted string showing the comparison in a boxed layout
///
/// # Example
/// ```ignore
/// use pmat::tdg::{Comparison, TdgScore, Grade};
/// let before = TdgScore::new(75.0, Grade::B, 0.9);
/// let after = TdgScore::new(85.0, Grade::A, 0.95);
/// let comparison = Comparison::new(before, after);
/// let output = format_comparison(&comparison);
/// assert!(output.contains("improvement"));
/// ```ignore
#[must_use]
pub fn format_comparison(comparison: &Comparison) -> String {
    let mut output = String::new();

    writeln!(
        output,
        "╭─────────────────────────────────────────────────╮"
    )
    .expect("Writing to String buffer cannot fail");

    let name1 = comparison.source1.file_path.as_ref().map_or_else(
        || "source1".to_string(),
        |p| {
            p.file_name()
                .expect("Path must have a filename component")
                .to_string_lossy()
                .to_string()
        },
    );
    let name2 = comparison.source2.file_path.as_ref().map_or_else(
        || "source2".to_string(),
        |p| {
            p.file_name()
                .expect("Path must have a filename component")
                .to_string_lossy()
                .to_string()
        },
    );

    let header = format!("TDG Comparison: {name1} vs {name2}");
    let truncated_header = if header.len() > 45 {
        format!("{}...", header.chars().take(42).collect::<String>())
    } else {
        header
    };
    writeln!(output, "│  {truncated_header:47} │").expect("Writing to String buffer cannot fail");

    writeln!(
        output,
        "├─────────────────────────────────────────────────┤"
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "│                     {:>8}   {:>8}    {:>4}  │",
        name1, name2, "Δ"
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "│  Overall Score:     {:8.1}  {:8.1}  {:>+5.1} │",
        comparison.source1.total, comparison.source2.total, comparison.delta
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "│  Grade:             {:>8}  {:>8}   {:>4}  │",
        comparison.source1.grade,
        comparison.source2.grade,
        grade_delta(comparison.source1.grade, comparison.source2.grade)
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "│  Confidence:        {:>7.0}%  {:>7.0}%        │",
        comparison.source1.confidence * 100.0,
        comparison.source2.confidence * 100.0
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "│                                                 │"
    )
    .expect("Writing to String buffer cannot fail");

    writeln!(
        output,
        "│  Structural:        {:8.1}  {:8.1}  {:>+5.1} │",
        comparison.source1.structural_complexity,
        comparison.source2.structural_complexity,
        comparison.source2.structural_complexity - comparison.source1.structural_complexity
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "│  Semantic:          {:8.1}  {:8.1}  {:>+5.1} │",
        comparison.source1.semantic_complexity,
        comparison.source2.semantic_complexity,
        comparison.source2.semantic_complexity - comparison.source1.semantic_complexity
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "│  Duplication:       {:8.1}  {:8.1}  {:>+5.1} │",
        comparison.source1.duplication_ratio,
        comparison.source2.duplication_ratio,
        comparison.source2.duplication_ratio - comparison.source1.duplication_ratio
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "│  Coupling:          {:8.1}  {:8.1}  {:>+5.1} │",
        comparison.source1.coupling_score,
        comparison.source2.coupling_score,
        comparison.source2.coupling_score - comparison.source1.coupling_score
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "│  Documentation:     {:8.1}  {:8.1}  {:>+5.1} │",
        comparison.source1.doc_coverage,
        comparison.source2.doc_coverage,
        comparison.source2.doc_coverage - comparison.source1.doc_coverage
    )
    .expect("Writing to String buffer cannot fail");
    writeln!(
        output,
        "│  Consistency:       {:8.1}  {:8.1}  {:>+5.1} │",
        comparison.source1.consistency_score,
        comparison.source2.consistency_score,
        comparison.source2.consistency_score - comparison.source1.consistency_score
    )
    .expect("Writing to String buffer cannot fail");

    writeln!(
        output,
        "│                                                 │"
    )
    .expect("Writing to String buffer cannot fail");
    let winner_text = format!(
        "Winner: {} ({:.1}% improvement)",
        comparison.winner,
        comparison.improvement_percentage.abs()
    );
    let truncated_winner = if winner_text.len() > 47 {
        format!("{}...", winner_text.chars().take(44).collect::<String>())
    } else {
        winner_text
    };
    writeln!(output, "│  {truncated_winner:47} │").expect("Writing to String buffer cannot fail");

    if !comparison.improvements.is_empty() {
        writeln!(
            output,
            "│                                                 │"
        )
        .expect("Writing to String buffer cannot fail");
        writeln!(output, "│  Key Improvements:                             │")
            .expect("Writing to String buffer cannot fail");
        for improvement in &comparison.improvements {
            let improvement_line = format!("  • {improvement}");
            let truncated = if improvement_line.len() > 45 {
                format!(
                    "{}...",
                    improvement_line.chars().take(42).collect::<String>()
                )
            } else {
                improvement_line
            };
            writeln!(output, "│  {truncated:47} │").expect("Writing to String buffer cannot fail");
        }
    }

    if !comparison.regressions.is_empty() {
        writeln!(
            output,
            "│                                                 │"
        )
        .expect("Writing to String buffer cannot fail");
        writeln!(output, "│  Minor Regressions:                            │")
            .expect("Writing to String buffer cannot fail");
        for regression in &comparison.regressions {
            let regression_line = format!("  • {regression}");
            let truncated = if regression_line.len() > 45 {
                format!(
                    "{}...",
                    regression_line.chars().take(42).collect::<String>()
                )
            } else {
                regression_line
            };
            writeln!(output, "│  {truncated:47} │").expect("Writing to String buffer cannot fail");
        }
    }

    writeln!(
        output,
        "╰─────────────────────────────────────────────────╯"
    )
    .expect("Writing to String buffer cannot fail");

    output
}

pub(super) fn grade_delta(from: Grade, to: Grade) -> String {
    let from_val = grade_to_number(from);
    let to_val = grade_to_number(to);
    let delta = to_val - from_val;

    if delta > 0 {
        format!("↑{delta}")
    } else if delta < 0 {
        format!("↓{}", delta.abs())
    } else {
        "=".to_string()
    }
}

pub(super) fn grade_to_number(grade: Grade) -> i32 {
    match grade {
        Grade::APLus => 11,
        Grade::A => 10,
        Grade::AMinus => 9,
        Grade::BPlus => 8,
        Grade::B => 7,
        Grade::BMinus => 6,
        Grade::CPlus => 5,
        Grade::C => 4,
        Grade::CMinus => 3,
        Grade::D => 2,
        Grade::F => 1,
    }
}
