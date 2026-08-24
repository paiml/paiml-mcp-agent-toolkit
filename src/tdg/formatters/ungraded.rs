#![cfg_attr(coverage_nightly, coverage(off))]
//! How "walked but not measured" files are disclosed — ONE implementation.
//!
//! A TDG headline is computed over the files that parsed. The ones that did not
//! are the difference between a grade and a grade over an unknown subset, so
//! every human renderer has to disclose them, and disclose them the same way.
//!
//! Before this module there were three renderers and three answers
//! (paiml/aprender#2462, pmat#983):
//!
//! * `analyze tdg --format table` (`tdg::formatters::project`) named the files,
//!   but pasted the raw absolute path into a 47-column box, so every entry was
//!   clipped to its common prefix — `/tmp/claude-1000/-home-noah-src-paiml-…` —
//!   which identifies nothing when the paths share a root, which they always do.
//! * `pmat tdg <path>` (`cli::handlers::tdg_handlers::formatting`) — the
//!   renderer the bug report actually pasted — printed only the count.
//! * `analyze tdg --format markdown` (`cli::handlers::new_tdg_handler`) printed
//!   only the count.
//!
//! [`ungraded_rows`] is the one rule: a count line, then up to
//! [`UNGRADED_SHOWN`] entries whose *tail* survives the width limit, then a
//! pointer to the JSON, which is never truncated.

use super::boxdraw::{visible_width, BODY_WIDTH};
use crate::tdg::UngradedFile;

/// How many unmeasured files a human report names before pointing at the JSON.
///
/// Enough to act on — the reader can see whether they are all `include!`
/// fragments from one directory or something worse — without turning a summary
/// into a listing.
pub(crate) const UNGRADED_SHOWN: usize = 10;

/// Columns available to one entry inside a box row (frame body minus indent).
const ENTRY_INDENT: usize = 4;
const ENTRY_BUDGET: usize = BODY_WIDTH - ENTRY_INDENT;

/// Collapse a parser message onto one line.
fn one_line(reason: &str) -> String {
    reason.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Keep the END of `text`, marking the cut with `…`.
///
/// The tail is the identifying part of a path: a set of files that cannot be
/// graded almost always shares its leading directories, so clipping from the
/// right (what `box_row` does on its own) throws away the only part that tells
/// them apart.
fn keep_tail(text: &str, budget: usize) -> String {
    if visible_width(text) <= budget || budget == 0 {
        return text.to_string();
    }
    let chars: Vec<char> = text.chars().collect();
    let keep = budget.saturating_sub(1); // room for the leading '…'
    let mut start = chars.len() - keep.min(chars.len());
    // Prefer to start the tail at a path separator, when one is close by.
    if let Some(off) = chars[start..].iter().position(|c| *c == '/') {
        if off < 12 && start + off + 1 < chars.len() {
            start += off + 1;
        }
    }
    format!("…{}", chars[start..].iter().collect::<String>())
}

/// Keep the START of `text`, marking the cut with `…`.
fn keep_head(text: &str, budget: usize) -> String {
    if visible_width(text) <= budget {
        return text.to_string();
    }
    if budget == 0 {
        return String::new();
    }
    let head: String = text.chars().take(budget.saturating_sub(1)).collect();
    format!("{head}…")
}

/// One entry line: as much of the path's tail as fits, then the reason.
fn entry(path: &str, reason: &str, budget: Option<usize>) -> String {
    let reason = one_line(reason);
    let Some(budget) = budget else {
        // No width limit (Markdown): say everything.
        return if reason.is_empty() {
            path.to_string()
        } else {
            format!("{path} — {reason}")
        };
    };
    if reason.is_empty() {
        return keep_tail(path, budget);
    }
    const SEP: &str = " — ";
    let sep_w = visible_width(SEP);
    // The reason is capped at a third of the row so a long parser message can
    // never squeeze the path — the path is what the reader needs in order to
    // look at the file.
    let reason_shown = keep_head(&reason, budget / 3);
    let path_budget = budget
        .saturating_sub(sep_w)
        .saturating_sub(visible_width(&reason_shown));
    format!("{}{SEP}{reason_shown}", keep_tail(path, path_budget))
}

/// The one sentence that states how many files were walked but not measured.
fn count_line(n: usize) -> String {
    format!("Not Graded: {n} file(s) walked, not measured")
}

/// The `… and N more` pointer, when the list was capped.
///
/// Issue #1064. The KEY NAME is the whole point of this line — it is the only
/// place a reader is told what to look for in the JSON — and it was the part
/// the frame ate. `box_row` clips from the right at `BODY_WIDTH`, and the
/// 68-column sentence below is 25 columns too wide for a 49-column box, so
/// every capped report printed
///
/// ```text
/// │      … and 5 more (--format json lists every one│
/// ```
///
/// with no key, no closing parenthesis, and no hint that either had been cut.
/// The box is fixed-width by design (`COLUMNS=200` changes nothing), so the
/// sentence has to fit the box rather than the other way round: pick the
/// longest phrasing the budget allows, and never one that has to be clipped.
fn more_line(hidden: usize, budget: Option<usize>) -> String {
    let full =
        format!("… and {hidden} more (--format json lists every one under \"ungraded_files\")");
    let Some(budget) = budget else {
        // No frame (Markdown): say the whole sentence.
        return full;
    };
    if visible_width(&full) <= budget {
        return full;
    }
    // Shortest phrasing that still names the key. 39 columns at a five-digit
    // count, against a 43-column box budget — see `the_pointer_never_loses_the_key_name`.
    format!("… and {hidden} more: json \"ungraded_files\"")
}

/// The same disclosure as Markdown: a bold count, then one bullet per file with
/// the path in a code span.
///
/// Markdown has no frame, so nothing is elided; the path is not pasted into the
/// same code span as the reason, because a parser message routinely contains
/// backticks (``expected `;` ``) and would close the span early.
pub(crate) fn ungraded_markdown_lines(files: &[UngradedFile]) -> Vec<String> {
    if files.is_empty() {
        return Vec::new();
    }
    let mut lines = vec![format!("**{}**", count_line(files.len()))];
    for f in files.iter().take(UNGRADED_SHOWN) {
        let reason = one_line(&f.reason);
        lines.push(if reason.is_empty() {
            format!("- `{}`", f.path)
        } else {
            format!("- `{}` — {reason}", f.path)
        });
    }
    if files.len() > UNGRADED_SHOWN {
        lines.push(format!(
            "- {}",
            more_line(files.len() - UNGRADED_SHOWN, None)
        ));
    }
    lines
}

/// The disclosure rows for `files`, without colour and without the box frame.
///
/// The first row is the count; the rest are indented entries. `budget` is the
/// columns an entry may use (`None` for renderers with no width limit).
/// Returns empty when nothing was refused — an empty disclosure is the only
/// honest rendering of "everything walked was measured".
pub(crate) fn ungraded_rows(files: &[UngradedFile], budget: Option<usize>) -> Vec<String> {
    if files.is_empty() {
        return Vec::new();
    }
    let mut rows = Vec::with_capacity(files.len().min(UNGRADED_SHOWN) + 2);
    rows.push(count_line(files.len()));
    for f in files.iter().take(UNGRADED_SHOWN) {
        rows.push(format!(
            "{}{}",
            " ".repeat(ENTRY_INDENT),
            entry(&f.path, &f.reason, budget)
        ));
    }
    if files.len() > UNGRADED_SHOWN {
        rows.push(format!(
            "{}{}",
            " ".repeat(ENTRY_INDENT),
            more_line(files.len() - UNGRADED_SHOWN, budget)
        ));
    }
    rows
}

/// The entry budget for the box-drawing renderers.
pub(crate) const fn box_entry_budget() -> usize {
    ENTRY_BUDGET
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f(path: &str, reason: &str) -> UngradedFile {
        UngradedFile {
            path: path.to_string(),
            reason: reason.to_string(),
        }
    }

    /// Issue #1064. The pointer line exists to name the JSON key; the box
    /// clipped it away, so the one sentence that told the reader what to look
    /// for was the one sentence that never arrived intact.
    ///
    /// RED at HEAD, observed from the binary before the fix (a 17-file tree,
    /// 15 of them `.sh`):
    ///
    /// ```text
    /// │      … and 5 more (--format json lists every one│
    /// ```
    ///
    /// The sentence is 68 columns against a 43-column budget. The width
    /// assertion below is the one that pins it: reverting `more_line` to
    /// always return the long form fails it at 68 of 43.
    #[test]
    fn the_pointer_never_loses_the_key_name() {
        // Five digits of hidden files: aprender walks 152 unmeasurable files,
        // so the count is not always one column wide.
        for hidden in [1_usize, 5, 152, 12_345] {
            let line = more_line(hidden, Some(ENTRY_BUDGET));
            assert!(
                line.contains("ungraded_files"),
                "the pointer must name the key it points at: {line}"
            );
            assert!(
                visible_width(&line) <= ENTRY_BUDGET,
                "the pointer must FIT the frame rather than be clipped by it: \
                 {} columns of {ENTRY_BUDGET}: {line}",
                visible_width(&line)
            );
        }
    }

    /// COUNTER-TEST: a renderer with no frame keeps the full sentence. Making
    /// every format wear the box's 43-column limit would be the fix
    /// over-applied — Markdown has no box.
    #[test]
    fn an_unframed_renderer_keeps_the_long_form() {
        let line = more_line(5, None);
        assert!(line.contains("--format json lists every one"), "{line}");
        assert!(line.contains("ungraded_files"), "{line}");
    }

    /// The capped box report must carry the key through the FRAME, not just
    /// out of `more_line`: this is the row as `ungraded_rows` emits it, indent
    /// included.
    #[test]
    fn the_capped_box_row_still_names_the_key() {
        let files: Vec<UngradedFile> = (0..15)
            .map(|i| {
                f(
                    &format!("/some/deep/path/s{i}.sh"),
                    "Bash source: no analyzer",
                )
            })
            .collect();
        let rows = ungraded_rows(&files, Some(ENTRY_BUDGET));
        let last = rows.last().expect("a capped list ends with the pointer");
        assert!(last.contains("ungraded_files"), "{last}");
        assert!(
            visible_width(last) <= BODY_WIDTH,
            "{} columns of {BODY_WIDTH}: {last}",
            visible_width(last)
        );
    }

    #[test]
    fn nothing_refused_discloses_nothing() {
        assert!(ungraded_rows(&[], Some(ENTRY_BUDGET)).is_empty());
    }

    /// The reported defect: entries clipped to a shared prefix identify no file.
    #[test]
    fn a_long_path_keeps_its_tail() {
        let rows = ungraded_rows(
            &[f(
                "/home/noah/src/aprender/crates/aprender-core/src/oracle/coursera/arxiv_entries.rs",
                "expected `;`",
            )],
            Some(ENTRY_BUDGET),
        );
        let line = &rows[1];
        assert!(
            line.contains("arxiv_entries.rs"),
            "the file name must survive the width limit, got: {line:?}"
        );
        assert!(
            line.starts_with("    …"),
            "an elided path must say it was elided, got: {line:?}"
        );
        assert!(
            visible_width(line) <= BODY_WIDTH,
            "entry must fit the frame body ({BODY_WIDTH}), got {} in {line:?}",
            visible_width(line)
        );
    }

    /// Two fragments from the same directory must not render identically.
    #[test]
    fn two_files_under_one_root_stay_distinguishable() {
        let rows = ungraded_rows(
            &[
                f(
                    "/very/long/shared/root/prefix/src/alpha_entries.rs",
                    "expected `;`",
                ),
                f(
                    "/very/long/shared/root/prefix/src/beta_entries.rs",
                    "expected `;`",
                ),
            ],
            Some(ENTRY_BUDGET),
        );
        assert_ne!(
            rows[1], rows[2],
            "clipped to the shared prefix, got: {rows:?}"
        );
        assert!(rows[1].contains("alpha_entries.rs"), "{rows:?}");
        assert!(rows[2].contains("beta_entries.rs"), "{rows:?}");
    }

    /// A long parser message must not squeeze the path out of the row.
    #[test]
    fn a_long_reason_never_costs_the_file_name() {
        let rows = ungraded_rows(
            &[f(
                "/a/b/c/d/e/f/g/needle.rs",
                "cannot parse string into token stream: unexpected end of input while looking for `}`",
            )],
            Some(ENTRY_BUDGET),
        );
        assert!(rows[1].contains("needle.rs"), "got: {rows:?}");
        assert!(visible_width(&rows[1]) <= BODY_WIDTH, "got: {rows:?}");
    }

    #[test]
    fn the_list_is_capped_and_says_how_many_it_hid() {
        let files: Vec<UngradedFile> = (0..25)
            .map(|i| f(&format!("src/frag_{i}.rs"), "expected `;`"))
            .collect();
        let rows = ungraded_rows(&files, Some(ENTRY_BUDGET));
        assert_eq!(rows[0], "Not Graded: 25 file(s) walked, not measured");
        assert_eq!(rows.len(), 1 + UNGRADED_SHOWN + 1);
        assert!(rows.last().unwrap().contains("and 15 more"), "{rows:?}");
        assert!(rows.last().unwrap().contains("json"), "{rows:?}");
    }

    /// Exactly `UNGRADED_SHOWN` files are all named, with no "more" line.
    #[test]
    fn a_full_but_uncapped_list_has_no_more_line() {
        let files: Vec<UngradedFile> = (0..UNGRADED_SHOWN)
            .map(|i| f(&format!("src/frag_{i}.rs"), ""))
            .collect();
        let rows = ungraded_rows(&files, Some(ENTRY_BUDGET));
        assert_eq!(rows.len(), 1 + UNGRADED_SHOWN);
        assert!(!rows.last().unwrap().contains("more"), "{rows:?}");
    }

    /// Markdown has no frame, so nothing is elided there.
    #[test]
    fn unbudgeted_rows_are_not_elided() {
        let long =
            "/home/noah/src/aprender/crates/aprender-core/src/oracle/coursera/arxiv_entries.rs";
        let rows = ungraded_rows(&[f(long, "expected `;`")], None);
        assert!(rows[1].contains(long), "got: {rows:?}");
        assert!(rows[1].contains("expected `;`"), "got: {rows:?}");
    }

    /// A reason routinely contains backticks; the path must be in a code span
    /// of its own or the span closes on the reason's first backtick and the
    /// rendered bullet is garbage.
    #[test]
    fn markdown_keeps_the_path_in_its_own_code_span() {
        let lines = ungraded_markdown_lines(&[f("src/oracle/arxiv_entries.rs", "expected `;`")]);
        assert_eq!(lines[0], "**Not Graded: 1 file(s) walked, not measured**");
        assert_eq!(lines[1], "- `src/oracle/arxiv_entries.rs` — expected `;`");
    }

    #[test]
    fn markdown_is_capped_the_same_way_the_boxes_are() {
        let files: Vec<UngradedFile> = (0..12)
            .map(|i| f(&format!("src/frag_{i}.rs"), "expected `;`"))
            .collect();
        let lines = ungraded_markdown_lines(&files);
        assert_eq!(lines.len(), 1 + UNGRADED_SHOWN + 1);
        assert!(
            lines.last().unwrap().starts_with("- … and 2 more"),
            "{lines:?}"
        );
    }

    #[test]
    fn markdown_discloses_nothing_when_nothing_was_refused() {
        assert!(ungraded_markdown_lines(&[]).is_empty());
    }
}
