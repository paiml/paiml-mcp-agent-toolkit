//! PMAT-1363 (#1363) — `docs/roadmaps/roadmap.yaml` as a GENERATED aggregate of
//! `docs/roadmaps/entries/<id>.yaml`.
//!
//! THE DEFECT. `pmat work add/start/complete` write the roadmap in every consumer
//! repo, so N pull requests contend on one file however disjoint their code is —
//! Amdahl serial fraction 1 on the merge path. PMAT-679 made the write an APPEND
//! rather than a whole-file rewrite, which bounds the damage to the tail; it does
//! not remove the contention, because two branches still append to the same tail.
//!
//! A unique filename per ticket makes pull requests pairwise disjoint on the
//! roadmap, so the conflict rate is 0 by proof rather than by luck.
//!
//! THIS IS A PORT, not a redesign. The reference is paiml/aprender's
//! `scripts/lib/roadmap_fragments.py` (with `parse_id` from `roadmap_merge.py` and
//! `split_entries` from `roadmap_diff.py`), and the two must produce the same bytes
//! from the same inputs: aprender's roadmap is aggregated by the Python today and by
//! `pmat roadmap aggregate` once it adopts this. Every rule below names the Python
//! it mirrors. The divergences are few, each is a case where the reference emits a
//! roadmap that does not parse or does not survive its own re-aggregation, and each
//! is listed here so none is silent:
//!
//! * a fragment must hold exactly one row, declared on its first line, whose id is
//!   its filename, at the base's row indent, ending in a newline — otherwise
//!   `aggregate(aggregate(x)) != aggregate(x)` (a leading comment is glued onto the
//!   previous row and duplicated on every run; a missing newline fuses two rows);
//! * a base with no rows gets its first rows through [`append_item`], which rewrites
//!   `roadmap: []` to `roadmap:` — the reference emits `roadmap: []\n- id: …`;
//! * rows are recognised at the file's row indent (`roadmap_text::row_indent`), which
//!   is column 0 on every roadmap the reference reads, so the two agree there, and
//!   pmat's indent-2 roadmaps are not silently read as having no rows.

use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::fmt;
use std::path::{Path, PathBuf};

use crate::services::roadmap_text::{append_item, row_indent};

/// The longest id `FILENAME_SAFE` admits: `^[A-Za-z0-9][A-Za-z0-9._-]{0,110}$`.
pub const MAX_ID_LEN: usize = 111;

/// Everything aggregation can refuse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FragmentError {
    /// Two fragments claim one id. The filesystem already prevents it; the
    /// aggregator refuses it anyway rather than rely on that silently.
    DuplicateAmongFragments(String),
    /// An id that cannot be a filename, so it cannot be a fragment.
    NotAFilename(String),
    /// A fragment whose bytes would break idempotence or the YAML.
    Malformed {
        /// The fragment's id (its filename stem).
        id: String,
        /// What is wrong, in one sentence.
        reason: String,
    },
    /// A fragment or its directory could not be read or written.
    Io {
        /// The path that failed.
        path: PathBuf,
        /// The operating system's error.
        reason: String,
    },
}

impl fmt::Display for FragmentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateAmongFragments(id) => write!(f, "duplicate id among fragments: {id}"),
            Self::NotAFilename(id) => {
                write!(
                    f,
                    "id {id:?} cannot be a filename, so it cannot be a fragment"
                )
            }
            Self::Malformed { id, reason } => write!(f, "fragment {id}: {reason}"),
            Self::Io { path, reason } => write!(f, "{}: {reason}", path.display()),
        }
    }
}

impl std::error::Error for FragmentError {}

/// An id's numeral, compared as a number of ANY size.
///
/// Python's `int()` has no ceiling, so a `u64` would disagree with the reference
/// past 20 digits. The digits are kept without leading zeros and compared by
/// length, then text — which is numeric order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Numeral(String);

impl Ord for Numeral {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0
            .len()
            .cmp(&other.0.len())
            .then_with(|| self.0.cmp(&other.0))
    }
}

impl PartialOrd for Numeral {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// `(prefix, numeral)` for a PREFIX-NUMBER id, else `None` (a legacy id).
///
/// `parse_id` in `roadmap_merge.py`:
/// `^([A-Za-z][A-Za-z0-9_]*)-([0-9]+)([^0-9A-Za-z_].*)?$`. The prefix runs to the
/// FIRST dash, so `ROADMAP-RECONCILE-2026-07-04` is legacy, not the prefix
/// `ROADMAP-RECONCILE-2026-07` numbered 4 — an `rsplit` on the last dash reads it
/// the second way and sorts it among ids it has nothing to do with. The digits may
/// be followed by prose (`PMAT-12 (notes)`), never by a word character.
#[must_use]
pub fn parse_id(id: &str) -> Option<(&str, Numeral)> {
    if !id.bytes().next()?.is_ascii_alphabetic() {
        return None;
    }
    let (prefix, rest) = id.split_once('-')?;
    if !prefix
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'_')
    {
        return None;
    }
    let digits_end = rest
        .bytes()
        .position(|b| !b.is_ascii_digit())
        .unwrap_or(rest.len());
    if digits_end == 0 {
        return None;
    }
    if let Some(next) = rest[digits_end..].chars().next() {
        if next.is_ascii_alphanumeric() || next == '_' {
            return None;
        }
    }
    let digits = rest[..digits_end].trim_start_matches('0');
    let digits = if digits.is_empty() { "0" } else { digits };
    Some((prefix, Numeral(digits.to_string())))
}

/// Can this id be a filename? `FILENAME_SAFE` in `roadmap_fragments.py`.
///
/// The filename IS the mechanism: a unique name is what makes two pull requests
/// disjoint. Real data in paiml/aprender's roadmap includes the id
/// `Push completed work to origin/main (5 commits)`, which carries a path separator.
#[must_use]
pub fn is_filename_safe(id: &str) -> bool {
    let mut bytes = id.bytes();
    bytes.next().is_some_and(|b| b.is_ascii_alphanumeric())
        && id.len() <= MAX_ID_LEN
        && bytes.all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
}

/// Is this id a ticket id `pmat work add` may mint: filename-safe AND exactly
/// `PREFIX-N` (`^[A-Za-z][A-Za-z0-9_]*-[0-9]+$`)?
///
/// Stricter than [`parse_id`], which also admits a prose suffix: a suffix is how 8
/// of aprender's legacy ids came to carry spaces, and an id minted today becomes a
/// filename, a commit trailer and a branch name.
#[must_use]
pub fn is_ticket_id(id: &str) -> bool {
    is_filename_safe(id)
        && id.split_once('-').is_some_and(|(_, digits)| {
            !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit())
        })
        && parse_id(id).is_some()
}

/// A fragment's filename, or `None` when the id cannot be one.
#[must_use]
pub fn fragment_filename(id: &str) -> Option<String> {
    is_filename_safe(id).then(|| format!("{id}.yaml"))
}

/// The path a fragment lives at, given the directory holding them.
#[must_use]
pub fn fragment_path(entries_dir: &Path, id: &str) -> Option<PathBuf> {
    fragment_filename(id).map(|name| entries_dir.join(name))
}

/// Undo the quoting an id scalar can carry. `parse_id_value` in `roadmap_diff.py`.
#[must_use]
pub fn parse_id_value(raw: &str) -> String {
    let value = raw.trim();
    if value.len() >= 2 && value.starts_with('\'') && value.ends_with('\'') {
        return value[1..value.len() - 1].replace("''", "'");
    }
    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
        return value[1..value.len() - 1].replace("\\\"", "\"");
    }
    value.to_string()
}

/// The id a row-start line declares, or `None` when the line starts no row.
///
/// `ENTRY_RE` in `roadmap_diff.py` is `^- id:[ \t]*(.*)$` — the literal `- id:` at
/// column 0. Here the column is the file's row indent, which is 0 wherever the
/// reference reads, so the two agree there.
fn row_start_id(line: &str, indent: usize) -> Option<String> {
    let lead = line.get(..indent)?;
    if !lead.bytes().all(|b| b == b' ') {
        return None;
    }
    let value = line[indent..].strip_prefix("- id:")?;
    Some(parse_id_value(value.trim_start_matches([' ', '\t'])))
}

/// `(preamble, [(id, block)])` in file order. `split_entries` in `roadmap_diff.py`.
///
/// No dedup here: a duplicated id is the caller's violation, not a parse error.
/// Preamble plus every block, concatenated, is the input byte for byte.
#[must_use]
pub fn split_entries(raw: &str) -> (String, Vec<(String, String)>) {
    let indent = row_indent(raw);
    let mut starts: Vec<(usize, String)> = Vec::new();
    let mut offset = 0usize;
    for line in raw.split_inclusive('\n') {
        if let Some(id) = row_start_id(line.strip_suffix('\n').unwrap_or(line), indent) {
            starts.push((offset, id));
        }
        offset += line.len();
    }
    let Some(first) = starts.first() else {
        return (raw.to_string(), Vec::new());
    };
    let preamble = raw[..first.0].to_string();
    let mut out = Vec::with_capacity(starts.len());
    for (i, (start, id)) in starts.iter().enumerate() {
        let end = starts.get(i + 1).map_or(raw.len(), |(next, _)| *next);
        out.push((id.clone(), raw[*start..end].to_string()));
    }
    (preamble, out)
}

/// The slot `check_roadmap_sorted.sh` would demand. `insertion_index` in
/// `roadmap_fragments.py`.
///
/// Within the id's own prefix: before the first row of that prefix whose numeral is
/// greater. An unseen prefix, or a legacy id, appends — appending is always sorted
/// when nothing of that prefix precedes it.
#[must_use]
pub fn insertion_index(entries: &[(String, String)], id: &str) -> usize {
    let Some((prefix, numeral)) = parse_id(id) else {
        return entries.len();
    };
    entries
        .iter()
        .position(|(other, _)| parse_id(other).is_some_and(|(p, n)| p == prefix && n > numeral))
        .unwrap_or(entries.len())
}

/// Refuse a fragment whose bytes would break the aggregate.
///
/// `indent` is the column the base's rows start at. Each clause is an idempotence
/// precondition, not style: the reference accepts all of these and produces an
/// aggregate that changes on its own re-aggregation.
///
/// # Errors
///
/// [`FragmentError::NotAFilename`] or [`FragmentError::Malformed`].
pub fn check_fragment(id: &str, block: &str, indent: usize) -> Result<(), FragmentError> {
    let malformed = |reason: String| FragmentError::Malformed {
        id: id.to_string(),
        reason,
    };
    if !is_filename_safe(id) {
        return Err(FragmentError::NotAFilename(id.to_string()));
    }
    if !block.ends_with('\n') {
        return Err(malformed(
            "does not end in a newline, so its last line would fuse with the next row".into(),
        ));
    }
    let mut lines = block
        .split_inclusive('\n')
        .map(|l| l.trim_end_matches('\n'));
    match lines.next().and_then(|first| row_start_id(first, indent)) {
        Some(declared) if declared == id => {}
        Some(declared) => {
            return Err(malformed(format!(
                "declares id {declared:?}; a fragment's id is its filename"
            )))
        }
        None => {
            return Err(malformed(format!(
                "its first line is not `- id: {id}` at column {indent}; anything before the row \
                 would be glued onto the previous row and repeated on every aggregation"
            )))
        }
    }
    if let Some(extra) = lines.find_map(|line| row_start_id(line, indent)) {
        return Err(malformed(format!(
            "declares a second row ({extra:?}); one fragment is one ticket"
        )));
    }
    Ok(())
}

/// The base with every fragment placed at its sorted slot. `aggregate` in
/// `roadmap_fragments.py`.
///
/// IDEMPOTENT AND DETERMINISTIC BY CONSTRUCTION, and neither is decoration: the
/// aggregate is regenerated post-merge on the default branch, so a generator that is
/// not a pure function of `(base, fragments)` churns a commit on every merge. The
/// reference's first cut read the live `roadmap.yaml` as its base and raised
/// `duplicate id` on its own output — it failed on run ONE.
///
/// A fragment SUPERSEDES a base row of the same id rather than colliding with it:
/// that is what makes `entries/` the only edit path for an existing id, and in turn
/// what lets a gate forbid every write to `roadmap.yaml`.
///
/// Fragments are placed in filename order whatever order they arrive in — the order
/// `read_fragments` yields, and the reference's — so readdir order cannot reach the
/// output even through a caller that forgot to sort.
///
/// # Errors
///
/// Two fragments claiming one id, or a fragment [`check_fragment`] refuses.
pub fn aggregate(base: &str, fragments: &[(String, String)]) -> Result<String, FragmentError> {
    aggregate_with(base, fragments, insertion_index)
}

/// [`aggregate`] with the placement rule injected, so the mutation row of the case
/// table (append-only placement must be CAUGHT) exercises the real aggregation.
pub(crate) fn aggregate_with(
    base: &str,
    fragments: &[(String, String)],
    place: impl Fn(&[(String, String)], &str) -> usize,
) -> Result<String, FragmentError> {
    let (preamble, mut entries) = split_entries(base);
    let indent = row_indent(base);
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    for (id, block) in fragments {
        if !seen.insert(id.as_str()) {
            return Err(FragmentError::DuplicateAmongFragments(id.clone()));
        }
        check_fragment(id, block, indent)?;
    }
    let mut ordered: Vec<&(String, String)> = fragments.iter().collect();
    ordered.sort_by(|a, b| format!("{}.yaml", a.0).cmp(&format!("{}.yaml", b.0)));

    let base_had_rows = !entries.is_empty();
    entries.retain(|(id, _)| !seen.contains(id.as_str()));
    for (id, block) in ordered {
        let at = place(&entries, id);
        entries.insert(at, (id.clone(), block.clone()));
    }
    let rows: String = entries.iter().map(|(_, block)| block.as_str()).collect();
    if base_had_rows || rows.is_empty() {
        Ok(preamble + &rows)
    } else {
        // A base with no rows yet: `roadmap: []` cannot take a row after it.
        Ok(append_item(&preamble, &rows))
    }
}

/// Every fragment in `entries_dir`, as `(id, block)`, in filename order.
/// `read_fragments` in `roadmap_fragments.py`.
///
/// An absent directory is EMPTY, not an error: before the first fragment lands the
/// aggregate is just the base. Only `*.yaml` names are fragments, so a writer's
/// temporary file is never read as one.
///
/// # Errors
///
/// An unreadable directory or file, a non-UTF-8 name, or a `.yaml` name whose stem
/// cannot be an id.
pub fn read_fragments(entries_dir: &Path) -> Result<Vec<(String, String)>, FragmentError> {
    if !entries_dir.is_dir() {
        return Ok(Vec::new());
    }
    let io = |path: &Path, e: std::io::Error| FragmentError::Io {
        path: path.to_path_buf(),
        reason: e.to_string(),
    };
    let mut names: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(entries_dir).map_err(|e| io(entries_dir, e))? {
        let entry = entry.map_err(|e| io(entries_dir, e))?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            return Err(FragmentError::Io {
                path: entry.path(),
                reason: "a fragment name must be UTF-8".into(),
            });
        };
        if name.ends_with(".yaml") {
            names.push(name.to_string());
        }
    }
    names.sort();
    let mut out = Vec::with_capacity(names.len());
    for name in names {
        let id = name.trim_end_matches(".yaml").to_string();
        if !is_filename_safe(&id) {
            return Err(FragmentError::NotAFilename(id));
        }
        let path = entries_dir.join(&name);
        let block = std::fs::read_to_string(&path).map_err(|e| io(&path, e))?;
        out.push((id, block));
    }
    Ok(out)
}

/// Aggregate the roadmap at `roadmap_path` over the fragments in `entries_dir`.
///
/// Returns `(base, aggregate)`.
///
/// # Errors
///
/// An unreadable base (`Io`), or whatever [`read_fragments`] or [`aggregate`] refuse.
pub fn aggregate_paths(
    roadmap_path: &Path,
    entries_dir: &Path,
) -> Result<(String, String), FragmentError> {
    let base = std::fs::read_to_string(roadmap_path).map_err(|e| FragmentError::Io {
        path: roadmap_path.to_path_buf(),
        reason: e.to_string(),
    })?;
    let fragments = read_fragments(entries_dir)?;
    let out = aggregate(&base, &fragments)?;
    Ok((base, out))
}

/// The first row at which two roadmaps differ, by id, or `None` when their bytes are
/// equal. `PREAMBLE` names a difference outside every row.
///
/// This is what `pmat roadmap aggregate --check` prints, because "the aggregate is
/// stale" is not actionable in a 7,000-line file and "PMAT-1370 differs" is.
#[must_use]
pub fn first_difference(expected: &str, actual: &str) -> Option<String> {
    if expected == actual {
        return None;
    }
    let (expected_pre, expected_rows) = split_entries(expected);
    let (actual_pre, actual_rows) = split_entries(actual);
    if expected_pre != actual_pre {
        return Some("PREAMBLE".to_string());
    }
    for (want, got) in expected_rows.iter().zip(&actual_rows) {
        if want != got {
            return Some(want.0.clone());
        }
    }
    let longer = if expected_rows.len() > actual_rows.len() {
        &expected_rows
    } else {
        &actual_rows
    };
    Some(
        longer
            .get(expected_rows.len().min(actual_rows.len()))
            .map_or_else(|| "PREAMBLE".to_string(), |(id, _)| id.clone()),
    )
}

/// How many rows of a roadmap can be fragments at all. `census` in
/// `roadmap_fragments.py`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Census {
    /// Filename-safe with a PREFIX-NUMBER shape: fragmentable today.
    pub safe_prefix_n: Vec<String>,
    /// Filename-safe legacy ids (`ROADMAP-RECONCILE-2026-07-04`).
    pub safe_legacy: Vec<String>,
    /// PREFIX-NUMBER with a suffix no filename can carry (`PMAT-12 (notes)`).
    pub unsafe_prefix_n: Vec<String>,
    /// Prose ids.
    pub unsafe_legacy: Vec<String>,
}

impl Census {
    /// Every row, counted once.
    #[must_use]
    pub fn total(&self) -> usize {
        self.safe_prefix_n.len()
            + self.safe_legacy.len()
            + self.unsafe_prefix_n.len()
            + self.unsafe_legacy.len()
    }

    /// The rows that can never be a fragment.
    #[must_use]
    pub fn not_filename_safe(&self) -> usize {
        self.unsafe_prefix_n.len() + self.unsafe_legacy.len()
    }
}

/// Classify every row of `raw` by whether it can be a fragment.
#[must_use]
pub fn census(raw: &str) -> Census {
    let mut out = Census::default();
    for (id, _) in split_entries(raw).1 {
        let bucket = match (is_filename_safe(&id), parse_id(&id).is_some()) {
            (true, true) => &mut out.safe_prefix_n,
            (true, false) => &mut out.safe_legacy,
            (false, true) => &mut out.unsafe_prefix_n,
            (false, false) => &mut out.unsafe_legacy,
        };
        bucket.push(id);
    }
    out
}

/// The fragment directory beside a roadmap, when the repo has opted in.
///
/// OPT-IN IS DELIBERATE, and it is the predicate paiml/.github's shared
/// `roadmap-fragment-parity` gate uses: `[ -d docs/roadmaps/entries ]`. pmat ships
/// to every consumer repo; one that has not created the directory keeps the
/// PMAT-679 append behaviour exactly. Flipping that on a version bump would stop
/// their roadmap updating with nothing to read as an error.
#[must_use]
pub fn entries_dir_for(roadmap_path: &Path) -> Option<PathBuf> {
    let dir = parent_or_dot(roadmap_path).join("entries");
    dir.is_dir().then_some(dir)
}

/// `path`'s parent, with the empty parent of a bare file name spelled `.`.
fn parent_or_dot(path: &Path) -> PathBuf {
    match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
        _ => PathBuf::from("."),
    }
}

/// Write one ticket's row as its own file. Returns the path written.
///
/// Refuses an id that cannot be a filename rather than sanitising it: a sanitised
/// name is no longer the id, so trailer-to-filename parity would silently stop
/// meaning anything. The row is checked BEFORE anything is created, so a refused
/// fragment writes nothing at all, and it lands by rename, so a reader never sees
/// half of one.
///
/// # Errors
///
/// What [`check_fragment`] refuses, or an I/O failure.
pub fn write_fragment(entries_dir: &Path, id: &str, block: &str) -> Result<PathBuf, FragmentError> {
    let path = fragment_path(entries_dir, id)
        .ok_or_else(|| FragmentError::NotAFilename(id.to_string()))?;
    let indent = block.len() - block.trim_start_matches(' ').len();
    check_fragment(id, block, indent)?;
    let io = |path: &Path, e: std::io::Error| FragmentError::Io {
        path: path.to_path_buf(),
        reason: e.to_string(),
    };
    std::fs::create_dir_all(entries_dir).map_err(|e| io(entries_dir, e))?;
    let staging = entries_dir.join(format!(".{id}.yaml.tmp"));
    std::fs::write(&staging, block).map_err(|e| io(&staging, e))?;
    std::fs::rename(&staging, &path).map_err(|e| io(&path, e))?;
    Ok(path)
}

/// Delete one ticket's fragment. `Ok(false)` when there was none.
///
/// # Errors
///
/// An id that cannot be a filename, or an I/O failure other than absence.
pub fn remove_fragment(entries_dir: &Path, id: &str) -> Result<bool, FragmentError> {
    let path = fragment_path(entries_dir, id)
        .ok_or_else(|| FragmentError::NotAFilename(id.to_string()))?;
    match std::fs::remove_file(&path) {
        Ok(()) => Ok(true),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(FragmentError::Io {
            path,
            reason: e.to_string(),
        }),
    }
}
