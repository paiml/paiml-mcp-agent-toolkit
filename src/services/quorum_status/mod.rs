//! FLOW-010 F3 (PMAT-1614): a quorum receipt becomes a `quorum` commit status.
//!
//! Receipt files named `<sha>.<seat>.json` are judged against a closed schema and
//! reduced to ONE status payload; posting it is the paiml-fleet App's job, this
//! module only builds the payload and the `gh api` arguments. A `.txt` verdict, or
//! any file not matching the schema, is never counted. Receipts are read, never
//! written (OUT-171): the files are audit copies of what the status says.

#[cfg(test)]
mod tests;

use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::Path;

/// The commit-status context the ruleset requires.
pub const CONTEXT: &str = "quorum";

/// Distinct passing seats required.
pub const MIN_SEATS: usize = 3;

/// `<sha>.<seat>.json`: a closed schema. Unknown fields are refused.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    schema: String,
    sha: String,
    seat: String,
    verdict: String,
    model: String,
    family: String,
    author_family: String,
}

/// The body of a `POST /repos/{repo}/statuses/{sha}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusPayload {
    /// `success` or `failure`; never `pending`.
    pub state: &'static str,
    pub context: &'static str,
    pub description: String,
}

fn is_sha(s: &str) -> bool {
    s.len() == 40
        && s.bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

/// Parse one receipt; `Err` carries why it does not count.
fn parse(sha: &str, file: &str, text: &str) -> Result<Receipt, String> {
    let stem = file
        .strip_suffix(".json")
        .ok_or_else(|| format!("{file}: not a .json receipt"))?;
    let r: Receipt = serde_json::from_str(text).map_err(|e| format!("{file}: {e}"))?;
    if r.schema != "quorum-receipt/v1" {
        return Err(format!("{file}: unknown schema {}", r.schema));
    }
    if !matches!(r.verdict.as_str(), "PASS" | "FAIL") {
        return Err(format!("{file}: verdict {}", r.verdict));
    }
    if r.sha != sha || stem != format!("{}.{}", r.sha, r.seat) {
        return Err(format!("{file}: name and content disagree"));
    }
    if [&r.seat, &r.model, &r.family, &r.author_family]
        .iter()
        .any(|s| s.is_empty())
    {
        return Err(format!("{file}: empty field"));
    }
    Ok(r)
}

/// Reduce receipts `(file name, text)` for `sha` to one status. Success needs
/// [`MIN_SEATS`] distinct-model PASS seats outside the author's family and no FAIL.
pub fn judge(sha: &str, receipts: &[(String, String)]) -> StatusPayload {
    let mk = |ok: bool, description: String| StatusPayload {
        state: if ok { "success" } else { "failure" },
        context: CONTEXT,
        description: description.chars().take(140).collect(),
    };
    if !is_sha(sha) {
        return mk(false, "quorum: not a 40-hex commit sha".into());
    }
    let (mut models, mut seats) = (BTreeSet::new(), BTreeSet::new());
    let (mut fails, mut rejected) = (0usize, 0usize);
    // Never drop silently: every receipt that does not add a seat is named.
    let (mut deduped, mut dup_seats): (Vec<String>, Vec<String>) = (vec![], vec![]);
    for (file, text) in receipts {
        let Ok(r) = parse(sha, file, text) else {
            rejected += 1;
            continue;
        };
        if !seats.insert(r.seat.clone()) {
            rejected += 1;
            dup_seats.push(format!("dup seat {}", r.seat));
        } else if r.verdict == "FAIL" {
            fails += 1;
        } else if r.family == r.author_family {
            rejected += 1;
        } else if !models.insert(r.model.clone()) {
            deduped.push(format!("{}/{}", r.seat, r.model));
        }
    }
    let n = models.len();
    let ok = n >= MIN_SEATS && fails == 0;
    let mut d = format!("quorum {n}/{MIN_SEATS} PASS, {fails} FAIL, {rejected} not counted");
    for x in dup_seats {
        d.push_str(&format!("; {x}"));
    }
    if !deduped.is_empty() {
        d.push_str(&format!("; deduped: {}", deduped.join(",")));
    }
    mk(ok, d)
}

/// Every `<sha>.*` file in `dir`, as `(name, text)`; non-`.json` ones are
/// returned too so [`judge`] can count them as refused. Read-only.
pub fn load_dir(dir: &Path, sha: &str) -> std::io::Result<Vec<(String, String)>> {
    let prefix = format!("{sha}.");
    let mut out = Vec::new();
    for e in std::fs::read_dir(dir)? {
        let e = e?;
        let name = e.file_name().to_string_lossy().into_owned();
        if name.starts_with(&prefix) && e.file_type()?.is_file() {
            out.push((name, std::fs::read_to_string(e.path()).unwrap_or_default()));
        }
    }
    out.sort();
    Ok(out)
}

/// Arguments for `gh` that post `p` as a commit status on `repo`@`sha`.
pub fn gh_args(repo: &str, sha: &str, p: &StatusPayload) -> Vec<String> {
    ["api", "-X", "POST"]
        .iter()
        .map(|s| s.to_string())
        .chain([
            format!("repos/{repo}/statuses/{sha}"),
            "-f".into(),
            format!("state={}", p.state),
            "-f".into(),
            format!("context={}", p.context),
            "-f".into(),
            format!("description={}", p.description),
        ])
        .collect()
}
