//! The roadmap's RAW text: one id scanner, one validator.
//!
//! PMAT-676. There used to be two scanners over the same bytes and no shared
//! validator:
//!
//! * the allocator (PMAT-673, `id_key_value`/`next_id_number` in
//!   `roadmap_service_operations.rs`) — deliberately over-matching, because a
//!   missed id is the one direction that mints a duplicate;
//! * the validator (PMAT-674, `collect_id_lines`/`duplicate_ids` in
//!   `ticket_validate_migrate.rs`) — the stricter one, which skips block-scalar
//!   bodies and reads flow-style rows, because it must locate a clash by line
//!   without accusing a reviewer's note.
//!
//! Neither asked the other, so `pmat work add` accepted — and rewrote through
//! the lossy serde model — a roadmap that `pmat work validate` failed with
//! exit code 1. This module is the single scanner both now use, plus the one
//! validator, [`check_roadmap_text`], that `add`, `edit` and `validate` all
//! run against the same text before anything is written.
//!
//! Scanning text rather than the parsed document is deliberate on both counts:
//! the parse has already discarded a duplicate by the time it hands back a
//! `Roadmap` (two rows sharing an id are two well-formed rows), and a line
//! number is what a reader needs in a 4,000-line file.

use crate::models::roadmap::RoadmapItem;
use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};

/// Every `id` key line of the raw roadmap text, 1-based, in file order.
///
/// Recognised at any depth (items and subtasks alike), under every spelling
/// YAML allows for the same key: `- id: X`, `-   id: X`, a bare `id: X` whose
/// dash sits on the previous line, `- "id": X` and `- 'id': X`, `- id:X`, and
/// a flow mapping `- {id: X, title: …}`.
/// Not recognised: `identity:` and `github_issue:` (different keys), a
/// commented-out `# - id:`, `-id:` — YAML needs a space after the dash, so
/// that line is the scalar "-id:" and not a mapping at all — and the explicit
/// key form `? id` (never emitted, never seen in a roadmap).
///
/// Quorum finding on PMAT-674: the body of a block scalar (`notes: |`,
/// `- >-`) is text, not YAML. Every line indented deeper than the key that
/// opened the block is skipped, so a reviewer's note quoting `id: PMAT-001`
/// cannot fail the roadmap it sits in.
///
/// Quorum finding on PMAT-673 (3/3 lanes), which this scanner also has to
/// keep: matching the literal `- id:` only made an id in use under any other
/// valid spelling invisible to the allocator — a false LOW, and a false LOW
/// mints a duplicate.
#[must_use]
pub fn id_lines(raw: &str) -> Vec<(usize, String)> {
    let mut found = Vec::new();
    // Indentation of the line that opened a block scalar; `None` outside one.
    let mut block_opened_at: Option<usize> = None;
    for (index, line) in raw.lines().enumerate() {
        let indent = line.len() - line.trim_start().len();
        if let Some(opened_at) = block_opened_at {
            if line.trim().is_empty() || indent > opened_at {
                continue;
            }
            block_opened_at = None;
        }
        if let Some(id) = id_value_on_line(line) {
            found.push((index + 1, id));
        }
        if opens_block_scalar(line) {
            block_opened_at = Some(indent);
        }
    }
    found
}

/// `key: |`, `key: >-`, `- |`: the value that starts here is a block scalar
/// whose body is every deeper-indented line that follows.
fn opens_block_scalar(line: &str) -> bool {
    let trimmed = line.trim();
    let value = match trimmed.find(':') {
        Some(colon) => &trimmed[colon + 1..],
        None => trimmed.strip_prefix('-').unwrap_or(""),
    };
    let head = value.trim().split(" #").next().unwrap_or("").trim();
    let mut chars = head.chars();
    matches!(chars.next(), Some('|' | '>')) && chars.all(|c| matches!(c, '-' | '+' | '0'..='9'))
}

/// The id declared on one raw line, if that line declares one.
fn id_value_on_line(line: &str) -> Option<String> {
    let after_dash = strip_sequence_dash(line.trim_start())?;
    if after_dash.starts_with('{') {
        return id_in_flow_mapping(after_dash);
    }
    let after_key = strip_id_key(after_dash)?;
    Some(clean_scalar(after_key))
}

/// The `id` entry of a single-line flow mapping `{id: X, title: …}`.
fn id_in_flow_mapping(flow: &str) -> Option<String> {
    let inner = flow.strip_prefix('{')?.split('}').next()?;
    inner
        .split(',')
        .find_map(|entry| strip_id_key(entry.trim()).map(clean_scalar))
}

/// Strip a leading `- ` sequence indicator, if there is one.
///
/// Returns `None` for a comment line and for `-id:`, where the dash is part of
/// the scalar rather than an indicator.
fn strip_sequence_dash(trimmed: &str) -> Option<&str> {
    if trimmed.starts_with('#') {
        return None;
    }
    let Some(after) = trimmed.strip_prefix('-') else {
        return Some(trimmed);
    };
    if after.starts_with(char::is_whitespace) {
        Some(after.trim_start())
    } else {
        None
    }
}

/// Strip the key `id` — bare, "double" or 'single' quoted — and its colon.
///
/// The colon is required immediately (whitespace aside), which is what keeps
/// `identity:` out: after `id` comes `e`, not a colon.
fn strip_id_key(rest: &str) -> Option<&str> {
    let after_key = rest
        .strip_prefix("\"id\"")
        .or_else(|| rest.strip_prefix("'id'"))
        .or_else(|| rest.strip_prefix("id"))?;
    after_key.trim_start().strip_prefix(':')
}

/// The scalar value of an `id:` line: unquoted, without a trailing comment.
fn clean_scalar(raw: &str) -> String {
    let value = raw.trim();
    for quote in ['"', '\''] {
        if let Some(inner) = value.strip_prefix(quote) {
            if let Some(end) = inner.find(quote) {
                return inner[..end].to_string();
            }
        }
    }
    match value.find(" #") {
        Some(at) => value[..at].trim_end().to_string(),
        None => value.to_string(),
    }
}

/// Ids declared on more than one line, ordered by first occurrence, each with
/// every line it was declared on.
///
/// Both ends of a clash are reported, not only the second: a reader fixing the
/// roadmap needs to see the two rows to decide which one keeps the id.
#[must_use]
pub fn duplicate_ids(raw: &str) -> Vec<(String, Vec<usize>)> {
    let mut first_seen: Vec<String> = Vec::new();
    let mut lines_by_id: HashMap<String, Vec<usize>> = HashMap::new();
    for (line, id) in id_lines(raw) {
        let lines = lines_by_id.entry(id.clone()).or_default();
        if lines.is_empty() {
            first_seen.push(id);
        }
        lines.push(line);
    }
    first_seen
        .into_iter()
        .filter_map(|id| {
            let lines = lines_by_id.remove(&id)?;
            (lines.len() > 1).then_some((id, lines))
        })
        .collect()
}

/// The next ticket number: one past every id already spoken for.
///
/// PMAT-673. Pure, and deliberately reads the RAW roadmap text rather than the
/// parsed model:
///
/// * a `- id:` under `subtasks:` is an id in use, and the model's items do not
///   carry it at the top level;
/// * any prefix counts (`GH-7` and `PMAT-3` are both numbered), because the
///   number, not the prefix, is what collides;
/// * `lock_high_water` is the number persisted in the roadmap's lock file by
///   the last mint, so an id survives even if the ticket that used it is later
///   deleted from the roadmap.
///
/// A suffix that is not a `u32` is ignored — it cannot collide with a minted
/// `PMAT-NNN`.
///
/// PMAT-676 moved this onto [`id_lines`], which changes it in two ways, both
/// towards the validator: a flow-style row `- {id: PMAT-030}` is now counted
/// (the old scanner missed it — a false LOW), and an id quoted inside a block
/// scalar body is no longer counted (the old scanner took it — harmless, but
/// it is not an id, and `work validate` has never treated it as one).
#[must_use]
pub fn next_id_number(raw: &str, lock_high_water: Option<u32>) -> u32 {
    max_id_number(raw)
        .unwrap_or(0)
        .max(lock_high_water.unwrap_or(0))
        .saturating_add(1)
}

/// The greatest numeric suffix any id line of `raw` carries, or `None` when it
/// declares no id with one.
///
/// PMAT-680 split this out of [`next_id_number`]: the id authority scans the
/// roadmap as it stands on OTHER refs, where "one past the greatest" is the
/// wrong question — those numbers are candidates to be maximised over, not a
/// mint. One scanner still, so a ref's roadmap and this checkout's are read by
/// exactly the same rules.
#[must_use]
pub fn max_id_number(raw: &str) -> Option<u32> {
    let mut max: Option<u32> = None;
    for (_, id) in id_lines(raw) {
        let Some(token) = id.split_whitespace().next() else {
            continue;
        };
        let bare = token.trim_matches(|c| c == '"' || c == '\'');
        if let Some(number) = bare.rsplit('-').next() {
            if let Ok(parsed) = number.parse::<u32>() {
                max = Some(max.map_or(parsed, |seen: u32| seen.max(parsed)));
            }
        }
    }
    max
}

/// Everything a roadmap's raw text can be rejected for.
///
/// One variant today. It carries the path because its whole point is the
/// wording: `<path>:<line>` is what `pmat work validate` prints, and a caller
/// that only sees the error must get the same text a reader would.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoadmapTextError {
    /// Ids declared on more than one line, each with every line, in first-seen
    /// order — the payload of [`duplicate_ids`] plus the file it came from.
    Duplicates {
        /// The roadmap the lines are numbered in.
        path: PathBuf,
        /// Each duplicated id and every line it was declared on.
        duplicates: Vec<(String, Vec<usize>)>,
    },
}

impl RoadmapTextError {
    /// Each duplicated id and every line it was declared on.
    #[must_use]
    pub fn duplicates(&self) -> &[(String, Vec<usize>)] {
        match self {
            Self::Duplicates { duplicates, .. } => duplicates,
        }
    }

    /// One rendered line per problem, exactly as `work validate` prints it:
    /// `duplicate id PMAT-011 at <path>:21, <path>:53`.
    #[must_use]
    pub fn lines(&self) -> Vec<String> {
        match self {
            Self::Duplicates { path, duplicates } => {
                let file = path.display().to_string();
                duplicates
                    .iter()
                    .map(|(id, lines)| format!("duplicate id {id} at {}", located(&file, lines)))
                    .collect()
            }
        }
    }
}

/// `<path>:<line>, <path>:<line>` — every occurrence, located.
#[must_use]
pub fn located(file: &str, lines: &[usize]) -> String {
    lines
        .iter()
        .map(|line| format!("{file}:{line}"))
        .collect::<Vec<_>>()
        .join(", ")
}

impl fmt::Display for RoadmapTextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rendered = self.lines();
        write!(f, "{}", rendered.join("\n"))
    }
}

impl std::error::Error for RoadmapTextError {}

/// The one validator: is this roadmap text one `pmat work` may write to?
///
/// PMAT-676. Called by `work validate` (which reports it), by
/// `RoadmapService::add_item_with_next_id` and by
/// `RoadmapService::upsert_item_checked` (which refuse before writing, under
/// the exclusive lock, on the same text they just read). A check any one of
/// them skipped is a way to launder a roadmap the other two reject.
///
/// # Errors
///
/// [`RoadmapTextError::Duplicates`] when an id is declared on more than one
/// line, rendered as the lines `work validate` prints.
pub fn check_roadmap_text(raw: &str, path: &Path) -> Result<(), RoadmapTextError> {
    let duplicates = duplicate_ids(raw);
    if duplicates.is_empty() {
        return Ok(());
    }
    Err(RoadmapTextError::Duplicates {
        path: path.to_path_buf(),
        duplicates,
    })
}

// ── PMAT-679: writing the raw text back, one row at a time ──────────────────
//
// `pmat work add` and `pmat work edit` used to serialise the whole `Roadmap`
// model over the file. One added ticket therefore rewrote all 2,532 lines of
// aprender's roadmap (#1193, #1169; aprender #2874): every concurrent branch
// conflicted on the file, and everything the model does not carry — comments,
// unknown keys, flow-style rows, the choice of block scalar, the key order a
// human wrote — was reformatted or dropped in passing.
//
// The operations below are the whole fix, and they are pure text on purpose:
// the parsed model is exactly the representation that cannot see the bytes
// this defect destroys. `add` appends a rendered row and touches nothing else;
// `edit` replaces the span of the one row it edits.

/// One roadmap item, rendered as a single YAML sequence element.
///
/// The block starts with `- id:` at column `indent` and puts every
/// continuation line at `indent + 2` (the shape `serde_yaml_ng` emits for a
/// one-element sequence, shifted); it ends with exactly one newline, so
/// appending it to a file that ends in a newline needs no glue.
///
/// Key order is the model's declaration order, unchanged — this is the row
/// pmat writes, and it is the only row in the file pmat is entitled to format.
#[must_use]
pub fn render_item_block(item: &RoadmapItem, indent: usize) -> String {
    let rendered = serde_yaml_ng::to_string(std::slice::from_ref(item))
        .expect("a roadmap item is serialisable: it is plain data with no map keys but strings");
    if indent == 0 {
        return rendered;
    }
    let pad = " ".repeat(indent);
    let mut out = String::with_capacity(rendered.len() + indent * 4);
    for line in rendered.lines() {
        if !line.is_empty() {
            out.push_str(&pad);
            out.push_str(line);
        }
        out.push('\n');
    }
    out
}

/// The column the roadmap's top-level rows start at — 0 or 2, both of which
/// occur in the wild, because YAML lets a sequence sit at its key's indent.
///
/// The least-indented `- id:` line is the top level by construction: a subtask
/// row is nested under one, and a block scalar's body must be indented deeper
/// than the key that opened it. An empty sequence has no row to read, and 0 is
/// what pmat itself writes.
#[must_use]
pub fn row_indent(raw: &str) -> usize {
    raw.lines()
        .filter(|line| line.trim_start().starts_with('-') && id_value_on_line(line).is_some())
        .map(|line| line.len() - line.trim_start().len())
        .min()
        .unwrap_or(0)
}

/// Append one rendered row, leaving every other byte of `raw` identical.
///
/// The one exception, and the only line this function may rewrite: a roadmap
/// whose sequence is the empty FLOW sequence `roadmap: []` — nothing can be
/// appended after `[]`, so that single line becomes `roadmap:` and the row
/// follows it. A roadmap with no `roadmap:` key at all gains one.
#[must_use]
pub fn append_item(raw: &str, block: &str) -> String {
    match roadmap_key_line(raw) {
        Some((span, true)) => {
            let mut out = String::with_capacity(raw.len() + block.len() + 9);
            out.push_str(&raw[..span.0]);
            out.push_str("roadmap:\n");
            out.push_str(block);
            out.push_str(&raw[span.1..]);
            out
        }
        Some((key_span, false)) => {
            // Quorum lane 1 on PR #1201: when a top-level key FOLLOWS the
            // sequence, the row must land at the end of the sequence, not at
            // EOF inside that key's mapping. With `roadmap:` last (what pmat
            // writes) this is exactly `raw + block`.
            let spans = line_spans(raw);
            let indent = row_indent(raw);
            let key_index = spans.iter().position(|&s| s == key_span).unwrap_or(0);
            match sequence_boundary(raw, &spans, indent, key_index + 1) {
                None => append_at_end(raw, block),
                Some(boundary) => {
                    let last_start = top_level_rows(raw, &spans, indent)
                        .iter()
                        .map(|(index, _)| *index)
                        .rfind(|&index| index < boundary);
                    let end = last_start.map_or(key_index, |start| {
                        last_line_of_row(raw, &spans, start, boundary, indent)
                    });
                    let head = &raw[..spans[end].1];
                    let mut out = String::with_capacity(raw.len() + block.len() + 1);
                    out.push_str(head);
                    if !head.ends_with('\n') {
                        out.push('\n');
                    }
                    out.push_str(block);
                    out.push_str(&raw[spans[end].1..]);
                    out
                }
            }
        }
        None => {
            let mut out = append_at_end(raw, "roadmap:\n");
            out.push_str(block);
            out
        }
    }
}

/// `raw` with `tail` after its last line, adding the separating newline the
/// file may lack.
fn append_at_end(raw: &str, tail: &str) -> String {
    if raw.is_empty() || raw.ends_with('\n') {
        format!("{raw}{tail}")
    } else {
        format!("{raw}\n{tail}")
    }
}

/// The span of the top-level `roadmap:` key line, and whether its value is the
/// empty flow sequence `[]`.
fn roadmap_key_line(raw: &str) -> Option<((usize, usize), bool)> {
    line_spans(raw).into_iter().find_map(|span| {
        let line = line_text(raw, span);
        let value = line.strip_prefix("roadmap:")?;
        Some((span, value.split('#').next().unwrap_or("").trim() == "[]"))
    })
}

/// Replace the span of ONE row — the row declaring `id` — with `block`.
///
/// The span runs from that row's `- id:` line to the last line that belongs to
/// it: blank lines and top-level comments before the next row belong to the
/// NEXT row, not to this one, so a `# section heading` above a ticket survives
/// an edit of the ticket above it.
///
/// Returns `None` when the id names no top-level row, and when it is declared
/// more than once anywhere in the file — two rows sharing an id are two
/// well-formed rows, and choosing between them would be a guess. (`work add`
/// and `work edit` both run [`check_roadmap_text`] first, so in practice the
/// duplicate has already been refused with a line number.)
#[must_use]
pub fn replace_item_block(raw: &str, id: &str, block: &str) -> Option<String> {
    if id_lines(raw)
        .iter()
        .filter(|(_, found)| found == id)
        .count()
        != 1
    {
        return None;
    }
    let indent = row_indent(raw);
    let spans = line_spans(raw);
    let rows = top_level_rows(raw, &spans, indent);
    let position = rows
        .iter()
        .position(|(_, found)| found.as_deref() == Some(id))?;
    let start = rows[position].0;
    let next_row = rows
        .get(position + 1)
        .map_or(spans.len(), |(index, _)| *index);
    // A top-level key after the last row ends the sequence before it too.
    let limit =
        sequence_boundary(raw, &spans, indent, start + 1).map_or(next_row, |b| b.min(next_row));
    let end = last_line_of_row(raw, &spans, start, limit, indent);
    Some(format!(
        "{}{}{}",
        &raw[..spans[start].0],
        block,
        &raw[spans[end].1..]
    ))
}

/// The byte span of every line of `raw`, its newline included.
fn line_spans(raw: &str) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut start = 0usize;
    for (index, byte) in raw.bytes().enumerate() {
        if byte == b'\n' {
            spans.push((start, index + 1));
            start = index + 1;
        }
    }
    if start < raw.len() {
        spans.push((start, raw.len()));
    }
    spans
}

/// One line's text, without its line ending.
fn line_text(raw: &str, span: (usize, usize)) -> &str {
    raw[span.0..span.1].trim_end_matches(['\n', '\r'])
}

/// Every top-level row: the index of its dash line at the row indent, and the
/// id it declares — found among the row's own keys, not only on the dash line
/// (quorum lane 1 on PR #1201: `- title: …` first and `id:` second is a row
/// too, and one that the row finder must never fold into its neighbour). A
/// row without an id is still a boundary for the row before it.
fn top_level_rows(
    raw: &str,
    spans: &[(usize, usize)],
    indent: usize,
) -> Vec<(usize, Option<String>)> {
    let starts: Vec<usize> = spans
        .iter()
        .enumerate()
        .filter_map(|(index, &span)| {
            let line = line_text(raw, span);
            let trimmed = line.trim_start();
            let dash = trimmed.strip_prefix('-')?;
            let is_item = dash.is_empty() || dash.starts_with(char::is_whitespace);
            (is_item && line.len() - trimmed.len() == indent).then_some(index)
        })
        .collect();
    starts
        .iter()
        .enumerate()
        .map(|(n, &start)| {
            let limit = starts.get(n + 1).copied().unwrap_or(spans.len());
            let limit =
                sequence_boundary(raw, spans, indent, start + 1).map_or(limit, |b| b.min(limit));
            let id = (start..limit).find_map(|index| {
                let line = line_text(raw, spans[index]);
                let column = line.len() - line.trim_start().len();
                if column <= indent + 2 {
                    id_value_on_line(line)
                } else {
                    None
                }
            });
            (start, id)
        })
        .collect()
}

/// The index of the first line at or after `from` that ends the top-level
/// `roadmap:` sequence — non-blank, not a comment, at a column no deeper than
/// the row indent and not a row's dash: the next top-level key. `None` when
/// the sequence runs to the end of the file (what pmat itself writes).
fn sequence_boundary(
    raw: &str,
    spans: &[(usize, usize)],
    indent: usize,
    from: usize,
) -> Option<usize> {
    (from..spans.len()).find(|&index| {
        let line = line_text(raw, spans[index]);
        let trimmed = line.trim_start();
        let column = line.len() - trimmed.len();
        !trimmed.is_empty()
            && !trimmed.starts_with('#')
            && column <= indent
            && !trimmed.starts_with('-')
    })
}

/// The last line index that belongs to the row starting at `start`: the last
/// line before `limit` that is neither blank nor a comment at the rows' own
/// indent. A deeper-indented `#` line is a block scalar's body, and stays.
fn last_line_of_row(
    raw: &str,
    spans: &[(usize, usize)],
    start: usize,
    limit: usize,
    indent: usize,
) -> usize {
    let mut last = start;
    for index in start..limit {
        let line = line_text(raw, spans[index]);
        let trimmed = line.trim();
        let column = line.len() - line.trim_start().len();
        if trimmed.is_empty() || (trimmed.starts_with('#') && column <= indent) {
            continue;
        }
        last = index;
    }
    last
}
