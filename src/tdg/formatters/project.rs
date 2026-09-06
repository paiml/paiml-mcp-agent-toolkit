#![cfg_attr(coverage_nightly, coverage(off))]
//! The project-level TDG table.
//!
//! Colour comes from [`crate::cli::colors`] and nowhere else. This renderer
//! used to build every row as a plain `String`, so `analyze tdg --format table
//! --color always` was byte-identical to `--color never` while its twin
//! (`pmat tdg <path> --format table`, `cli/handlers/tdg_handlers/formatting.rs`)
//! painted the same score and grade through `c::number` / `c::grade`. Two
//! renderers of one report disagreeing about whether `--color` exists is the
//! same defect as two renderers disagreeing about the number: the flag is not
//! the authority, `cli::colors` is.
use std::fmt::Write;

use super::super::ProjectScore;
use super::boxdraw::{box_blank, box_bottom, box_row, box_separator, box_top};
use super::ungraded::{box_entry_budget, ungraded_rows};
use crate::cli::colors as c;

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
    line(box_row(&c::header("Project TDG Score Report")));
    line(box_separator());
    // GH #704: 0 analysed files used to print "Average Score: 0.0/100 (F)"
    // right above "Total Files: 0" — a struct default rendered as a
    // measurement. Nothing analysed, nothing claimed.
    line(box_row(
        &match (project.average_score, project.average_grade) {
            (Some(score), Some(grade)) => {
                let grade = grade.to_string();
                format!(
                    "Average Score: {}/100 ({})",
                    c::number(&format!("{score:.1}")),
                    c::grade(&grade)
                )
            }
            _ => c::dim("Average Score: not measured (no files analysed)"),
        },
    ));
    line(box_row(&format!(
        "Total Files: {}",
        c::number(&project.total_files.to_string())
    )));
    // A file that was walked but refused must be disclosed HERE, next to the
    // average it is missing from: the warning went to stderr only, so
    // `analyze tdg` on a crate whose only Rust file fails to parse printed
    // "Average Score: 100.0/100 (A+)" over the one file that survived.
    // …and it must NAME them. A count alone is unactionable: paiml/aprender
    // #2462 reported "Not Graded: 159 file(s)" on a 78-crate tree and could not
    // tell which 159, or whether they mattered. Most were `include!` fragments
    // — files that compile fine as part of a parent module but not standalone —
    // and that tree has 1,772 `include!` sites, so the pattern is not an edge
    // case. The paths were already in `ungraded_files`; only the printer was
    // withholding them. The rows come from `formatters::ungraded` so this
    // renderer, `pmat tdg <path>` and the Markdown report cannot drift apart
    // again (they had three different answers).
    let ungraded = ungraded_rows(&project.ungraded_files, Some(box_entry_budget()));
    for (i, row) in ungraded.iter().enumerate() {
        line(box_row(&if i == 0 {
            c::colored(c::YELLOW, row)
        } else {
            c::dim(row)
        }));
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
        line(box_row(&c::colored(
            c::YELLOW,
            &format!("Waived (#279): {waived} file(s) with critical defects"),
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
            c::number(&project.files_reported.to_string()),
            c::number(&project.total_files.to_string())
        )));
    }
    line(box_blank());

    line(box_row(&c::label("Language Distribution:")));
    // Distributions come from the whole analysed set, never from the possibly
    // truncated `files` vector.
    for (language, count) in &project.language_distribution {
        let percentage = percent_of(*count, project.total_files);
        // Pad before colouring — see the note on the grade rows below.
        line(box_row(&format!(
            "├─ {:12}: {} files ({percentage:4.1}%)",
            language.to_string(),
            c::number(&format!("{count:3}")),
        )));
    }

    line(box_blank());

    line(box_row(&c::label("Grade Distribution:")));
    for (grade, count) in &project.grade_distribution {
        let percentage = percent_of(*count, project.total_files);
        // Pad BEFORE colouring: `{:3}` counts bytes, and an escape sequence is
        // zero columns wide but many bytes long, so colouring first silently
        // eats the column alignment.
        let grade_text = grade.to_string();
        line(box_row(&format!(
            "├─ {}: {} files ({percentage:4.1}%)",
            c::grade(&grade_text),
            c::number(&format!("{count:3}"))
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

#[cfg(test)]
mod ungraded_disclosure_tests {
    //! REGRESSION (#983): the table named the unmeasured files by pasting the
    //! raw path into a 47-column row, so every entry was clipped to the prefix
    //! the paths share — `/home/alice/src/aprender/crates/aprender-…` for all of
    //! them. Naming a file with a string that cannot identify it is the same
    //! defect as not naming it.
    use super::*;
    use crate::tdg::formatters::boxdraw::visible_width;
    use crate::tdg::{ProjectScore, UngradedFile};

    fn project_with(paths: &[&str]) -> ProjectScore {
        let mut project = ProjectScore::aggregate(Vec::new());
        for p in paths {
            project.ungraded_files.push(UngradedFile {
                path: (*p).to_string(),
                reason: "expected `;`".to_string(),
            });
        }
        project
    }

    #[test]
    fn long_paths_are_named_by_their_tail_and_stay_distinct() {
        let rendered = format_project(&project_with(&[
            "/home/alice/src/aprender/crates/aprender-core/src/oracle/arxiv_entries.rs",
            "/home/alice/src/aprender/crates/aprender-core/src/oracle/coursera_entries.rs",
        ]));
        assert!(rendered.contains("Not Graded: 2 file(s)"), "{rendered}");
        assert!(rendered.contains("arxiv_entries.rs"), "got:\n{rendered}");
        assert!(rendered.contains("coursera_entries.rs"), "got:\n{rendered}");
    }

    /// Naming them must not push the frame out of shape.
    #[test]
    fn named_rows_stay_inside_the_frame() {
        let rendered = format_project(&project_with(&[
            "/home/alice/src/aprender/crates/aprender-core/src/oracle/coursera/arxiv_entries.rs",
        ]));
        let widths: Vec<usize> = rendered
            .lines()
            .filter(|l| !l.is_empty())
            .map(visible_width)
            .collect();
        for (i, w) in widths.iter().enumerate() {
            assert_eq!(
                *w,
                widths[0],
                "line {i} is {w} cols wide, frame is {} — {:?}",
                widths[0],
                rendered.lines().nth(i)
            );
        }
    }
}

#[cfg(test)]
mod color_census_tests {
    //! `--color` must move something on every human TDG renderer.
    //!
    //! The `--color` rule has been "fixed" four times in this release and kept
    //! coming back, because every test written for it asserted only that output
    //! is PLAIN when colour is off. A renderer that has no colour at all passes
    //! that assertion, which is exactly how `analyze tdg --format table` shipped
    //! with `--color always` byte-identical to `--color never` while its twin
    //! (`pmat tdg <path> --format table`) painted the same score and grade.
    //!
    //! [`crate::cli::colors::assert_honours_color`] asserts BOTH halves: at
    //! least one escape when colour is on, none when it is off. The first half
    //! is the one that fails on an inert flag.
    use super::*;
    use crate::cli::colors::assert_honours_color;
    use crate::tdg::language_simple::Language;
    use crate::tdg::{Grade, ProjectScore, TdgScore};
    use std::collections::BTreeMap;

    fn project() -> ProjectScore {
        ProjectScore {
            average_score: Some(90.9),
            average_grade: Some(Grade::A),
            not_measured: Vec::new(),
            total_files: 7,
            files: Vec::new(),
            language_distribution: BTreeMap::from([(Language::Rust, 7)]),
            grade_distribution: BTreeMap::from([(Grade::A, 5), (Grade::F, 2)]),
            ..ProjectScore::aggregate(Vec::new())
        }
    }

    /// `analyze tdg --format table` — the reported defect.
    #[test]
    fn project_table_honours_color() {
        assert_honours_color("tdg::formatters::format_project", || {
            format_project(&project())
        });
    }

    /// The same renderer with nothing measured still has a frame and a sentence
    /// to paint, and must not start emitting escapes with colour off.
    #[test]
    fn empty_project_table_honours_color() {
        assert_honours_color("tdg::formatters::format_project (empty)", || {
            format_project(&ProjectScore::aggregate(Vec::new()))
        });
    }

    /// `analyze tdg <file>` goes to `format_human`, the twin renderer in this
    /// module. Fixing one surface of a contradiction and not the other is how
    /// this defect keeps reappearing.
    #[test]
    fn file_report_honours_color() {
        assert_honours_color("tdg::formatters::format_human", || {
            super::super::format_human(&TdgScore {
                total: 85.5,
                grade: Grade::AMinus,
                language: Language::Rust,
                confidence: 1.0,
                file_path: Some(std::path::PathBuf::from("src/test.rs")),
                ..TdgScore::default()
            })
        });
    }

    /// Colour must be invisible to a reader of the numbers: with colour off the
    /// table is byte-identical to what it printed before any of this, so the
    /// existing assertions in `formatters/tests.rs` still describe it.
    #[test]
    fn plain_table_still_reads_as_before() {
        let _guard = crate::cli::colors::ForcedColor::off();
        let rendered = format_project(&project());
        assert!(
            rendered.contains("Average Score: 90.9/100 (A)"),
            "plain table must be unchanged, got:\n{rendered}"
        );
        assert!(rendered.contains("Total Files: 7"), "got:\n{rendered}");
    }

    /// An escape is zero columns wide but many bytes long, so a renderer that
    /// pads after colouring loses its alignment only when colour is ON — the
    /// state no test could reach before `ForcedColor` existed.
    #[test]
    fn colored_table_stays_rectangular() {
        let _guard = crate::cli::colors::ForcedColor::on();
        let rendered = format_project(&project());
        let widths: Vec<usize> = rendered
            .lines()
            .filter(|l| !l.is_empty())
            .map(crate::tdg::formatters::boxdraw::visible_width)
            .collect();
        assert!(!widths.is_empty(), "nothing rendered");
        for (i, w) in widths.iter().enumerate() {
            assert_eq!(
                *w,
                widths[0],
                "coloured line {i} is {w} cols wide, frame is {} — {:?}",
                widths[0],
                rendered.lines().nth(i)
            );
        }
    }
}
