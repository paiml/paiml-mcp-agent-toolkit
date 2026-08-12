#![cfg_attr(coverage_nightly, coverage(off))]
use std::fmt::Write;

use super::super::TdgScore;
use super::boxdraw::{
    box_blank, box_bottom, box_row, box_separator, box_top, ellipsize, BODY_WIDTH,
};
use super::helpers::progress_bar;

/// Format TDG score for human-readable console output.
///
/// Creates a visually appealing boxed display showing the TDG score,
/// grade, language confidence, and detailed breakdown of score components.
///
/// Every row is padded from its measured width (see `boxdraw`); the rows used
/// to carry hand-counted runs of spaces, so the right-hand border drifted with
/// the content (`0.0/100 (F)` vs `99.5/100 (A+)` closed in different columns).
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
    let mut line = |text: String| {
        writeln!(output, "{text}").expect("Writing to String buffer cannot fail");
    };

    line(box_top());
    match &score.file_path {
        Some(path) => line(box_row(&format!(
            "TDG Score Report: {}",
            path.display()
                .to_string()
                .chars()
                .take(30)
                .collect::<String>()
        ))),
        None => line(box_row("TDG Score Report: Code Analysis")),
    }
    line(box_separator());
    line(box_row(&format!(
        "Overall Score: {:.1}/100 ({})",
        score.total, score.grade
    )));
    line(box_row(&format!(
        "Language: {} (confidence: {:.0}%)",
        score.language,
        score.confidence * 100.0
    )));
    line(box_blank());
    line(box_row("📊 Breakdown:"));

    for (label, value, max) in [
        ("├─ Structural:    ", score.structural_complexity, 25.0),
        ("├─ Semantic:      ", score.semantic_complexity, 20.0),
        ("├─ Duplication:   ", score.duplication_ratio, 20.0),
        ("├─ Coupling:      ", score.coupling_score, 15.0),
        ("├─ Documentation: ", score.doc_coverage, 10.0),
        ("└─ Consistency:   ", score.consistency_score, 10.0),
    ] {
        line(box_row(&format!(
            "{label} {value:4.1}/{max:.0}  {}",
            progress_bar(value, max, 10)
        )));
    }

    // A waiver that changes the verdict must be disclosed on every surface that
    // reports the verdict. The #279 exemption used to be visible ONLY in
    // `tdg check-quality --format json` (`"… (1 waived under #279)"`); every
    // human renderer applied it in silence, so a reader of the default output
    // had no way to learn that a file with critical defects had been let
    // through.
    if score.has_critical_defects {
        line(box_blank());
        line(box_row(&format!(
            "⛔ Critical Defects: {}",
            score.critical_defects_count
        )));
        if let Some(reason) = &score.critical_defects_suppressed {
            line(box_row(&ellipsize(
                &format!("  auto-fail waived: {reason}"),
                BODY_WIDTH,
            )));
        }
    }

    if !score.penalties_applied.is_empty() {
        line(box_blank());
        line(box_row("🔍 Issues Found:"));
        for penalty in &score.penalties_applied {
            let issue_line = format!("  • {}", penalty.issue);
            line(box_row(&ellipsize(&issue_line, BODY_WIDTH)));
        }
    }

    line(box_blank());
    line(box_row(verdict(score.total)));
    line(box_bottom());

    output
}

/// One-line verdict for a total score.
fn verdict(total: f32) -> &'static str {
    if total >= 90.0 {
        "✨ Excellent code quality! No major issues."
    } else if total >= 75.0 {
        "👍 Good code quality with minor improvements."
    } else if total >= 60.0 {
        "⚠️  Code needs improvement in several areas."
    } else {
        "🔴 Code requires significant refactoring."
    }
}
