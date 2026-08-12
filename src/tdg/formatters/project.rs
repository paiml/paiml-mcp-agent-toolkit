#![cfg_attr(coverage_nightly, coverage(off))]
use std::fmt::Write;

use super::super::ProjectScore;
use super::boxdraw::{box_blank, box_bottom, box_row, box_separator, box_top};

/// Format project-level TDG score.
///
/// Creates a comprehensive project-level report showing aggregate TDG scores,
/// file counts, and overall project health metrics.
///
/// # Arguments
/// * `project` - The project score data structure
///
/// # Returns
/// A formatted string with project-level metrics and summary
///
/// # Example
/// ```ignore
/// use pmat::tdg::ProjectScore;
/// let project = ProjectScore::new("my-project", 85.0, 42);
/// let output = format_project(&project);
/// assert!(output.contains("Project Score"));
/// ```ignore
#[must_use]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn format_project(project: &ProjectScore) -> String {
    let mut output = String::new();
    let mut line = |text: String| {
        writeln!(output, "{text}").expect("Writing to String buffer cannot fail");
    };

    line(box_top());
    line(box_row("Project TDG Score Report"));
    line(box_separator());
    // GH #704: 0 analysed files used to print "Average Score: 0.0/100 (F)"
    // right above "Total Files: 0" — a struct default rendered as a
    // measurement. Nothing analysed, nothing claimed.
    line(box_row(
        &match (project.average_score, project.average_grade) {
            (Some(score), Some(grade)) => format!("Average Score: {score:.1}/100 ({grade})"),
            _ => "Average Score: not measured (no files analysed)".to_string(),
        },
    ));
    line(box_row(&format!("Total Files: {}", project.total_files)));
    // A file that was walked but refused must be disclosed HERE, next to the
    // average it is missing from: the warning went to stderr only, so
    // `analyze tdg` on a crate whose only Rust file fails to parse printed
    // "Average Score: 100.0/100 (A+)" over the one file that survived.
    if !project.ungraded_files.is_empty() {
        line(box_row(&format!(
            "Not Graded: {} file(s) walked, not measured",
            project.ungraded_files.len()
        )));
    }
    // The #279 waiver used to be disclosed only by `check-quality --format
    // json`; a reader of the default table had no way to learn that a file with
    // critical defects was exempted from the auto-fail.
    let waived = project
        .files
        .iter()
        .filter(|f| f.critical_defects_suppressed.is_some())
        .count();
    if waived > 0 {
        line(box_row(&format!(
            "Waived (#279): {waived} file(s) with critical defects"
        )));
    }
    // A truncated list must say so next to the total it sits under, so the
    // header count and the list below it can never contradict each other.
    if project.files_truncated {
        // The flag was hardcoded as `(--top-files)`, so a `--critical-only` run
        // blamed a flag the user never passed. Name the one that applied.
        let via = project
            .list_filter
            .as_deref()
            .map(|f| format!(" ({f})"))
            .unwrap_or_default();
        line(box_row(&format!(
            "Files Listed: {} of {}{via}",
            project.files_reported, project.total_files
        )));
    }
    line(box_blank());

    line(box_row("Language Distribution:"));
    // Distributions come from the whole analysed set, never from the possibly
    // truncated `files` vector.
    for (language, count) in &project.language_distribution {
        let percentage = percent_of(*count, project.total_files);
        line(box_row(&format!(
            "├─ {:12}: {:3} files ({:4.1}%)",
            language.to_string(),
            count,
            percentage
        )));
    }

    line(box_blank());

    line(box_row("Grade Distribution:"));
    for (grade, count) in &project.grade_distribution {
        let percentage = percent_of(*count, project.total_files);
        line(box_row(&format!(
            "├─ {grade}: {count:3} files ({percentage:4.1}%)"
        )));
    }

    line(box_bottom());

    output
}

/// Percentage of `total`, or 0.0 when nothing was analysed (never NaN).
fn percent_of(count: usize, total: usize) -> f32 {
    if total == 0 {
        0.0
    } else {
        (count as f32 / total as f32) * 100.0
    }
}
