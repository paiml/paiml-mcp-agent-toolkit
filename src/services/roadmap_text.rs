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
    let mut max = lock_high_water.unwrap_or(0);
    for (_, id) in id_lines(raw) {
        let Some(token) = id.split_whitespace().next() else {
            continue;
        };
        let bare = token.trim_matches(|c| c == '"' || c == '\'');
        if let Some(number) = bare.rsplit('-').next() {
            if let Ok(parsed) = number.parse::<u32>() {
                max = max.max(parsed);
            }
        }
    }
    max.saturating_add(1)
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
