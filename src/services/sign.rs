//! `pmat sign` — FLOW-011 rev7.2 T1 signer (PMAT-1529).
//!
//! Reads one receipt per commit (`<root>/<repo>/<pr>/<patch-id>.json`), refuses
//! unless every rule below holds, then writes `sign.json` LAST. Lines marked
//! PROPOSAL come from `T1-signer-shape.md` (infra-res), not from the spec text.
//!
//! Refusals: <2 distinct models (§1a); seat receipt not from a lane identity
//! (§1a); rerun-as-pass / RED->GREEN with no diff (§1a); missed planted control;
//! lane model == author model incl. Co-Authored-By; head != PR current head
//! (PROPOSAL). Everything external (head lookup, patch-id generator, clock) is
//! injected, so tests need no GitHub, git or keys.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// One lane attempt. Every attempt is kept (§1a): reruns are extra entries.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Lane {
    pub lane: String, // "q1".."q3" or "seat:<domain>"
    pub verdict: String,
    pub model: String,
    pub ts: String,
    /// PROPOSAL: the lane identity that wrote the receipt (checked for seats).
    #[serde(default)]
    pub by: String,
    /// The head this attempt reviewed; two attempts on one head = no diff.
    #[serde(default)]
    pub head: String,
    #[serde(default)]
    pub control_caught: Option<bool>,
}

/// Per-commit receipt (PROPOSAL layout, T1-signer-shape.md §3).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Receipt {
    pub patch_id: String,
    pub head: String,
    pub pr: u64,
    pub repo: String,
    pub lanes: Vec<Lane>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct SignFile {
    pub repo: String,
    pub pr: u64,
    pub head: String,
    pub patch_ids: Vec<String>,
    pub receipt_sha256: std::collections::BTreeMap<String, String>,
    pub signer: String,
    pub ts: String,
}

#[derive(Debug, PartialEq)]
pub enum Refusal {
    FewerThanTwoModels(usize),
    SeatNotFromLaneIdentity(String),
    RerunAsPass(String),
    MissedControl(String),
    AuthorModelInLane(String),
    StaleHead { receipt: String, current: String },
    MissingLane(String),
    NotPass(String),
    ReceiptMissing(String),
    AlreadySigned,
}

#[derive(Debug, PartialEq)]
pub enum SignError {
    Refused(Refusal),
    /// Exit 3: a value could not be measured; never substitute another key.
    NotMeasured(String),
}

/// Inputs injected by the caller.
pub struct Inputs<'a> {
    pub repo: &'a str,
    pub pr: u64,
    pub root: &'a Path,
    /// Ordered patch-ids of `origin/main..head`.
    pub patch_ids: &'a [String],
    pub head_lookup: &'a dyn Fn(&str, u64) -> Result<String, String>,
    pub lane_roster: &'a [String],
    /// PROPOSAL: seat domains the PR requires, e.g. ["rust"].
    pub required_seats: &'a [String],
    /// Author models, explicit plus parsed from Co-Authored-By trailers.
    pub author_models: &'a [String],
    pub now: &'a str,
    pub rearm: bool,
}

/// "Claude Sonnet 5.5" and "claude-sonnet-5-5" compare equal.
pub fn norm_model(m: &str) -> String {
    m.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Models named in `Co-Authored-By: <name> <email>` trailers.
pub fn co_authored_models(messages: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for line in messages.iter().flat_map(|m| m.lines()) {
        if let Some(rest) = line.trim().to_lowercase().strip_prefix("co-authored-by:") {
            let name = rest.split('<').next().unwrap_or("").trim();
            if !name.is_empty() {
                out.push(norm_model(name));
            }
        }
    }
    out
}

pub fn receipt_dir(root: &Path, repo: &str, pr: u64) -> PathBuf {
    root.join(repo).join(pr.to_string())
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Rule check over one receipt. Pure.
pub fn verify_receipt(
    r: &Receipt,
    current_head: &str,
    roster: &[String],
    seats: &[String],
    authors: &[String],
) -> Result<(), Refusal> {
    let id = r.patch_id.clone();
    if r.head != current_head {
        return Err(Refusal::StaleHead {
            receipt: r.head.clone(),
            current: current_head.into(),
        });
    }
    // Rerun-as-pass: a non-PASS attempt followed by PASS on the same head.
    for l in &r.lanes {
        let same: Vec<&Lane> = r.lanes.iter().filter(|o| o.lane == l.lane).collect();
        let red_first = same
            .iter()
            .any(|a| a.verdict != "PASS" && a.head == l.head && a.ts < l.ts);
        if l.verdict == "PASS" && red_first {
            return Err(Refusal::RerunAsPass(format!("{id}/{}", l.lane)));
        }
    }
    // Judge the LATEST attempt of each lane.
    let mut latest: Vec<&Lane> = Vec::new();
    for l in &r.lanes {
        match latest.iter().position(|o| o.lane == l.lane) {
            Some(i) if latest[i].ts < l.ts => latest[i] = l,
            Some(_) => {}
            None => latest.push(l),
        }
    }
    for need in ["q1", "q2", "q3"]
        .iter()
        .map(|s| s.to_string())
        .chain(seats.iter().map(|d| format!("seat:{d}")))
    {
        if !latest.iter().any(|l| l.lane == need) {
            return Err(Refusal::MissingLane(format!("{id}/{need}")));
        }
    }
    for l in &latest {
        if l.verdict != "PASS" {
            return Err(Refusal::NotPass(format!("{id}/{}", l.lane)));
        }
        if l.lane.starts_with("seat:") && !roster.contains(&l.by) {
            return Err(Refusal::SeatNotFromLaneIdentity(format!("{id}/{}", l.lane)));
        }
        if l.control_caught == Some(false) {
            return Err(Refusal::MissedControl(format!("{id}/{}", l.lane)));
        }
        if authors.iter().any(|a| *a == norm_model(&l.model)) {
            return Err(Refusal::AuthorModelInLane(format!("{id}/{}", l.lane)));
        }
    }
    let models: BTreeSet<String> = latest.iter().map(|l| norm_model(&l.model)).collect();
    if models.len() < 2 {
        return Err(Refusal::FewerThanTwoModels(models.len()));
    }
    Ok(())
}

/// Verify every receipt, then write `sign.json` last. Returns what was written.
pub fn sign(i: &Inputs) -> Result<SignFile, SignError> {
    let bad = |r| SignError::Refused(r);
    let dir = receipt_dir(i.root, i.repo, i.pr);
    let out = dir.join("sign.json");
    if out.exists() && !i.rearm {
        return Err(bad(Refusal::AlreadySigned));
    }
    let head = (i.head_lookup)(i.repo, i.pr).map_err(SignError::NotMeasured)?;
    let mut shas = std::collections::BTreeMap::new();
    for pid in i.patch_ids {
        let p = dir.join(format!("{pid}.json"));
        let bytes = std::fs::read(&p).map_err(|_| bad(Refusal::ReceiptMissing(pid.clone())))?;
        let r: Receipt = serde_json::from_slice(&bytes)
            .map_err(|e| SignError::NotMeasured(format!("{}: {e}", p.display())))?;
        let authors: Vec<String> = i.author_models.iter().map(|a| norm_model(a)).collect();
        verify_receipt(&r, &head, i.lane_roster, i.required_seats, &authors).map_err(bad)?;
        shas.insert(pid.clone(), sha256_hex(&bytes));
    }
    if i.patch_ids.is_empty() {
        return Err(SignError::NotMeasured("no patch-ids to sign".into()));
    }
    let sf = SignFile {
        repo: i.repo.into(),
        pr: i.pr,
        head,
        patch_ids: i.patch_ids.to_vec(),
        receipt_sha256: shas,
        signer: "pmat".into(),
        ts: i.now.into(),
    };
    if out.exists() {
        // PROPOSAL: re-arm keeps the old file (reversible), never deletes it.
        let kept = dir.join(format!("sign.json.disarmed-{}", i.now.replace(':', "")));
        std::fs::rename(&out, kept).map_err(|e| SignError::NotMeasured(e.to_string()))?;
    }
    let body = serde_json::to_vec_pretty(&sf).map_err(|e| SignError::NotMeasured(e.to_string()))?;
    std::fs::write(&out, body).map_err(|e| SignError::NotMeasured(e.to_string()))?;
    Ok(sf)
}

/// `--rearm <pr>` needs an `ALFREDO <UTC> — …` line naming that PR with a
/// reversible verb. PROPOSAL: verb allow-list (arm/rearm) and irreversible deny-list.
pub fn rearm_authorized(rulings: &Path, pr: u64) -> bool {
    let Ok(text) = std::fs::read_to_string(rulings) else {
        return false;
    };
    let re = regex::Regex::new(&format!(r"(?i)(#|\bpr\s*){pr}\b")).expect("static regex");
    text.lines().any(|l| {
        let low = l.to_lowercase();
        l.starts_with("ALFREDO ")
            && l.contains(" — ")
            && re.is_match(l)
            && (low.contains("rearm") || low.contains("re-arm"))
            && !["merge", "force", "delete", "yank", "close"]
                .iter()
                .any(|v| low.contains(v))
    })
}

/// PluggableGenerator: one function builds the key (conflict (a): FLOW-011 §4
/// full-context diff-tree vs EMR-001 `-U0`). Default is `-U0` (operator ruling).
pub type PatchIdFn = fn(&Path, &str) -> Result<String, SignError>;

/// Default key command (`-U0` diff-tree, myers); `{c}` is the commit.
pub const PATCH_ID_CMD: &str =
    "git diff-tree -p -U0 --no-color --no-ext-diff --no-renames --diff-algorithm=myers {c}^ {c} | git patch-id --verbatim";

pub fn patch_id_u0(repo_dir: &Path, commit: &str) -> Result<String, SignError> {
    let nm = |m: String| SignError::NotMeasured(m);
    let v = run(repo_dir, "git", &["--version"]).map_err(nm)?;
    if !git_at_least(&v, (2, 39)) {
        // Conflict (c): --verbatim needs git >= 2.39. Never compute another key.
        return Err(nm(format!(
            "git patch-id --verbatim needs >= 2.39, have: {}",
            v.trim()
        )));
    }
    let script = PATCH_ID_CMD.replace("{c}", commit);
    let o = run(repo_dir, "sh", &["-c", &script]).map_err(nm)?;
    o.split_whitespace()
        .next()
        .map(str::to_string)
        .ok_or_else(|| SignError::NotMeasured("empty patch-id".into()))
}

pub fn git_at_least(version_line: &str, min: (u32, u32)) -> bool {
    let nums: Vec<u32> = version_line
        .split_whitespace()
        .nth(2)
        .unwrap_or("")
        .split('.')
        .filter_map(|p| p.parse().ok())
        .collect();
    matches!(nums.as_slice(), [a, b, ..] if (*a, *b) >= min)
}

fn run(dir: &Path, prog: &str, args: &[&str]) -> Result<String, String> {
    let o = std::process::Command::new(prog)
        .args(args)
        .current_dir(dir)
        .output()
        .map_err(|e| e.to_string())?;
    if o.status.success() {
        Ok(String::from_utf8_lossy(&o.stdout).into_owned())
    } else {
        Err(String::from_utf8_lossy(&o.stderr).into_owned())
    }
}
