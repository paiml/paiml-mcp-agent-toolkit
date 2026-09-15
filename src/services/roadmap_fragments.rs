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
//! ORDERING IS THE AGGREGATOR'S JOB, not the merge's. A fragment is placed at the
//! slot the sorted-roadmap rule demands: within its own id prefix, numerals
//! ascending. Legacy ids that carry no PREFIX-NUMBER shape append, and keep their
//! relative position — the same rule `roadmap_text::numeric_id_key` encodes, reused
//! here rather than restated so the two cannot drift.

use std::path::Path;

use crate::services::roadmap_text::row_indent;

/// A fragment's filename, or `None` when the id cannot be one.
///
/// The filename IS the mechanism: a unique name is what makes two pull requests
/// disjoint. An id that cannot be a filename therefore cannot be a fragment, and
/// saying so is not pedantry — real data in paiml/aprender's roadmap includes the
/// id `Push completed work to origin/main (5 commits)`, which carries a path
/// separator, and 51 more like it out of 879.
#[must_use]
pub fn fragment_filename(id: &str) -> Option<String> {
    if id.is_empty() || id.len() > 110 {
        return None;
    }
    let first_ok = id.chars().next().is_some_and(|c| c.is_ascii_alphanumeric());
    let rest_ok = id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'));
    if !first_ok || !rest_ok {
        return None;
    }
    Some(format!("{id}.yaml"))
}

/// The path a fragment lives at, given the directory holding them.
#[must_use]
pub fn fragment_path(entries_dir: &Path, id: &str) -> Option<std::path::PathBuf> {
    fragment_filename(id).map(|name| entries_dir.join(name))
}

/// `(prefix, numeral)` for a PREFIX-NUMBER id, else `None` (a legacy id).
///
/// The numeral is compared as a NUMBER: `PMAT-90` sorts before `PMAT-100`, which a
/// string compare gets backwards.
fn prefix_and_numeral(id: &str) -> Option<(&str, u64)> {
    let (prefix, digits) = id.rsplit_once('-')?;
    if prefix.is_empty() {
        return None;
    }
    digits.parse::<u64>().ok().map(|n| (prefix, n))
}

/// `(preamble, [(id, block)])` in file order, split on the TOP-LEVEL sequence rows.
///
/// A nested `id:` (a `subtasks:` child) is not an entry, and a body line inside a
/// block scalar is prose — both are skipped, the same distinctions
/// `roadmap_text::id_lines` makes for the allocator.
#[must_use]
pub fn split_entries(raw: &str) -> (String, Vec<(String, String)>) {
    let indent = row_indent(raw);
    let mut starts: Vec<(usize, String)> = Vec::new();
    let mut block_opened_at: Option<usize> = None;
    let mut offset = 0usize;
    for line in raw.split_inclusive('\n') {
        let trimmed = line.trim_end_matches('\n');
        let this_indent = trimmed.len() - trimmed.trim_start().len();
        let in_block = match block_opened_at {
            Some(opened) if trimmed.trim().is_empty() || this_indent > opened => true,
            Some(_) => {
                block_opened_at = None;
                false
            }
            None => false,
        };
        if !in_block && this_indent == indent {
            if let Some(rest) = trimmed.trim_start().strip_prefix("- ") {
                if let Some(v) = rest.trim_start().strip_prefix("id:") {
                    let id = v.trim().trim_matches('\'').trim_matches('"').to_string();
                    if !id.is_empty() {
                        starts.push((offset, id));
                    }
                }
            }
        }
        if !in_block && trimmed.trim_end().ends_with('|') || trimmed.trim_end().ends_with(">-") {
            block_opened_at = Some(this_indent);
        }
        offset += line.len();
    }
    if starts.is_empty() {
        return (raw.to_string(), Vec::new());
    }
    let preamble = raw[..starts[0].0].to_string();
    let mut out = Vec::with_capacity(starts.len());
    for (i, (start, id)) in starts.iter().enumerate() {
        let end = starts.get(i + 1).map_or(raw.len(), |(s, _)| *s);
        out.push((id.clone(), raw[*start..end].to_string()));
    }
    (preamble, out)
}

/// Where a fragment belongs: before the first entry of its own prefix whose numeral
/// is greater; otherwise the end. An unseen prefix and a legacy id append, which is
/// always sorted when nothing of that prefix precedes them.
fn insertion_index(entries: &[(String, String)], id: &str) -> usize {
    let Some((prefix, numeral)) = prefix_and_numeral(id) else {
        return entries.len();
    };
    entries
        .iter()
        .position(|(other, _)| {
            prefix_and_numeral(other).is_some_and(|(p, n)| p == prefix && n > numeral)
        })
        .unwrap_or(entries.len())
}

/// The base with every fragment placed at its sorted slot.
///
/// IDEMPOTENT AND DETERMINISTIC BY CONSTRUCTION, and neither is decoration: the
/// aggregate is regenerated post-merge on the default branch, so a generator that is
/// not a pure function of `(base, fragments)` churns a commit on every merge. A
/// fragment SUPERSEDES a base row of the same id rather than colliding with it —
/// that is what makes `entries/` the only edit path for an existing id, and in turn
/// what lets a gate forbid every write to `roadmap.yaml`. Refusing it as a duplicate
/// would re-open the base as an edit surface.
///
/// Two fragments claiming one id is still an error; the filesystem already
/// guarantees they cannot, and relying on that silently would be a guess.
pub fn aggregate(base: &str, fragments: &[(String, String)]) -> Result<String, String> {
    let (preamble, mut entries) = split_entries(base);
    let mut seen: Vec<&str> = Vec::with_capacity(fragments.len());
    for (id, _) in fragments {
        if seen.contains(&id.as_str()) {
            return Err(format!("duplicate id among fragments: {id}"));
        }
        seen.push(id);
    }
    entries.retain(|(id, _)| !seen.contains(&id.as_str()));
    for (id, block) in fragments {
        let at = insertion_index(&entries, id);
        entries.insert(at, (id.clone(), block.clone()));
    }
    let mut out = String::with_capacity(base.len() + 256);
    out.push_str(&preamble);
    for (_, block) in &entries {
        out.push_str(block);
    }
    Ok(out)
}
