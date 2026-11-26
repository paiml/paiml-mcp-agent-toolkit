//! FLOW-011 T1: the `sign.json` writer behind `pmat sign` (#1529).
//!
//! For each commit of a PR, writes `<root>/<repo>/<pr>/<patch-id>.json`, then
//! writes `<root>/<repo>/<pr>/sign.json` last. `sign.json` binds the ordered
//! patch-id list, the head sha and the sha256 of every receipt file.
//! Refusals follow FLOW-011 rev 7.2 section 1a and the T1 signer shape. A refused
//! sign writes nothing. `--rearm` is not part of this module.

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

/// One attempt by one lane. Every attempt is kept; a rerun never replaces one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attempt {
    pub verdict: String,
    pub head: String,
}

/// One review lane's record for one commit.
#[derive(Debug, Clone)]
pub struct Lane {
    /// `q1`..`q3` for quorum lanes, `seat:<domain>` for seat lanes.
    pub name: String,
    pub model: String,
    /// Lane identity. Seat lanes must carry one.
    pub identity: Option<String>,
    /// True when this lane is a coding worker, which may never seat.
    pub coding_worker: bool,
    /// False when the lane missed the planted control, which makes the round invalid.
    pub control_caught: bool,
    pub attempts: Vec<Attempt>,
}

/// One commit of the PR, in `origin/main..<head>` order.
#[derive(Debug, Clone)]
pub struct Commit {
    pub patch_id: String,
    pub lanes: Vec<Lane>,
}

/// Why a sign was refused. A refusal writes nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    StaleHead { signed: String, pr_head: String },
    QuorumNotThreeOfThree(String),
    FewerThanTwoModels(String),
    SeatWithoutIdentity(String),
    CodingWorkerSeat(String),
    AuthorModel { patch_id: String, model: String },
    ControlMissed(String),
    RerunAsPass(String),
    NotGreen(String),
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "refused: {}", self.reason())
    }
}

impl Refusal {
    /// The refusal's sentence, without the `refused: ` prefix.
    fn reason(&self) -> String {
        match self {
            Self::StaleHead { signed, pr_head } => {
                format!("head {signed} is not the PR head {pr_head}")
            }
            Self::AuthorModel { patch_id, model } => {
                format!("lane model {model} is the author model in {patch_id}")
            }
            other => other.receipt_reason(),
        }
    }

    /// The sentence for the refusals that name one commit's receipt.
    fn receipt_reason(&self) -> String {
        match self {
            Self::QuorumNotThreeOfThree(p) => format!("{p} is not 3/3 quorum"),
            Self::FewerThanTwoModels(p) => format!("{p} has fewer than 2 distinct models"),
            Self::SeatWithoutIdentity(p) => format!("seat receipt without identity in {p}"),
            Self::NotGreen(p) => format!("a lane is not green in {p}"),
            other => other.lane_reason(),
        }
    }

    /// The sentence for the refusals that name one lane's receipt.
    fn lane_reason(&self) -> String {
        match self {
            Self::CodingWorkerSeat(p) => format!("coding worker seated in {p}"),
            Self::ControlMissed(p) => format!("a lane missed the control in {p}"),
            Self::RerunAsPass(p) => format!("red then green with no diff in {p}"),
            other => format!("{other:?}"),
        }
    }
}

fn is_pass(verdict: &str) -> bool {
    matches!(verdict, "PASS" | "APPROVE" | "APPROVED")
}

/// Judge one commit's lanes against the sign refusals.
///
/// `author_models` holds the author's model and every model named in a
/// Co-Authored-By trailer. `head` is the commit's head; `pr_head` is the PR's
/// current head (a GraphQL read at sign time).
pub fn check(
    commit: &Commit,
    author_models: &[String],
    head: &str,
    pr_head: &str,
) -> Result<(), Refusal> {
    let p = commit.patch_id.as_str();
    if head != pr_head {
        return Err(Refusal::StaleHead {
            signed: head.into(),
            pr_head: pr_head.into(),
        });
    }
    if commit.lanes.iter().filter(|l| !is_seat(l)).count() != 3 {
        return Err(Refusal::QuorumNotThreeOfThree(p.into()));
    }
    let models: std::collections::BTreeSet<&str> =
        commit.lanes.iter().map(|l| l.model.as_str()).collect();
    if models.len() < 2 {
        return Err(Refusal::FewerThanTwoModels(p.into()));
    }
    commit
        .lanes
        .iter()
        .try_for_each(|lane| check_lane(lane, p, author_models))
}

fn is_seat(lane: &Lane) -> bool {
    lane.name.starts_with("seat:")
}

/// A seat lane needs an identity and must not be a coding worker.
fn check_seat(lane: &Lane, p: &str) -> Result<(), Refusal> {
    if !is_seat(lane) {
        return Ok(());
    }
    if lane.identity.as_deref().is_none_or(str::is_empty) {
        return Err(Refusal::SeatWithoutIdentity(p.into()));
    }
    if lane.coding_worker {
        return Err(Refusal::CodingWorkerSeat(p.into()));
    }
    Ok(())
}

/// A lane may not run on a model that authored the commit.
fn check_author(lane: &Lane, p: &str, author_models: &[String]) -> Result<(), Refusal> {
    if author_models.iter().any(|a| a == &lane.model) {
        return Err(Refusal::AuthorModel {
            patch_id: p.into(),
            model: lane.model.clone(),
        });
    }
    Ok(())
}

/// Judge one lane: seat identity, author model, control, reruns, final verdict.
fn check_lane(lane: &Lane, p: &str, author_models: &[String]) -> Result<(), Refusal> {
    check_seat(lane, p)?;
    check_author(lane, p, author_models)?;
    if !lane.control_caught {
        return Err(Refusal::ControlMissed(p.into()));
    }
    if has_rerun_green(&lane.attempts) {
        return Err(Refusal::RerunAsPass(p.into()));
    }
    match lane.attempts.last() {
        Some(last) if is_pass(&last.verdict) => Ok(()),
        _ => Err(Refusal::NotGreen(p.into())),
    }
}

/// True when a red attempt is followed by a green one at the same head (no diff).
fn has_rerun_green(attempts: &[Attempt]) -> bool {
    attempts.iter().enumerate().any(|(i, earlier)| {
        attempts[i + 1..].iter().any(|later| {
            !is_pass(&earlier.verdict) && is_pass(&later.verdict) && later.head == earlier.head
        })
    })
}

/// Serialize one commit's lanes into its receipt body.
fn receipt_body(commit: &Commit, repo: &str, pr: u64, head: &str) -> Value {
    let lanes: Vec<Value> = commit
        .lanes
        .iter()
        .map(|l| {
            json!({
                "lane": l.name,
                "model": l.model,
                "identity": l.identity,
                "attempts": l.attempts.iter().map(|a| json!({"verdict": a.verdict, "head": a.head})).collect::<Vec<_>>(),
            })
        })
        .collect();
    json!({ "patch_id": commit.patch_id, "repo": repo, "pr": pr, "head": head, "lanes": lanes })
}

/// The PR-level inputs to a sign.
pub struct SignRequest<'a> {
    pub root: &'a Path,
    pub repo: &'a str,
    pub pr: u64,
    /// Head the receipts were judged at.
    pub head: &'a str,
    /// The PR's current head.
    pub pr_head: &'a str,
    pub author_models: &'a [String],
    pub ts: &'a str,
}

/// Write the receipts and `sign.json` for a PR. Returns the `sign.json` path.
///
/// Every commit is checked before anything is written, so a refusal leaves
/// `<root>/<repo>/<pr>/` untouched.
pub fn sign_pr(req: &SignRequest<'_>, commits: &[Commit]) -> Result<PathBuf> {
    if commits.is_empty() {
        bail!("refused: no commits to sign");
    }
    for commit in commits {
        check(commit, req.author_models, req.head, req.pr_head)
            .map_err(|r| anyhow::anyhow!("{r}"))?;
    }
    let dir = req.root.join(req.repo).join(req.pr.to_string());
    std::fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;

    let mut receipt_sha256 = BTreeMap::new();
    for commit in commits {
        let path = dir.join(format!("{}.json", commit.patch_id));
        let body = receipt_body(commit, req.repo, req.pr, req.head);
        let bytes = serde_json::to_vec_pretty(&body)?;
        std::fs::write(&path, &bytes).with_context(|| format!("write {}", path.display()))?;
        receipt_sha256.insert(commit.patch_id.clone(), hex_sha256(&bytes));
    }

    let patch_ids: Vec<&str> = commits.iter().map(|c| c.patch_id.as_str()).collect();
    let sign = json!({
        "repo": req.repo,
        "pr": req.pr,
        "head": req.head,
        "patch_ids": patch_ids,
        "receipt_sha256": receipt_sha256,
        "signer": "pmat",
        "ts": req.ts,
    });
    let out = dir.join("sign.json");
    let tmp = dir.join("sign.json.tmp");
    std::fs::write(&tmp, serde_json::to_vec_pretty(&sign)?)
        .with_context(|| format!("write {}", tmp.display()))?;
    std::fs::rename(&tmp, &out).with_context(|| format!("rename to {}", out.display()))?;
    Ok(out)
}

fn hex_sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lane(name: &str, model: &str) -> Lane {
        Lane {
            name: name.into(),
            model: model.into(),
            identity: None,
            coding_worker: false,
            control_caught: true,
            attempts: vec![Attempt {
                verdict: "PASS".into(),
                head: "h1".into(),
            }],
        }
    }

    fn green_commit() -> Commit {
        Commit {
            patch_id: "p1".into(),
            lanes: vec![lane("q1", "opus"), lane("q2", "sonnet"), lane("q3", "opus")],
        }
    }

    fn authors() -> Vec<String> {
        vec!["haiku".into()]
    }

    /// PR `paiml/x#7`; `pr_head` is "h1", so `head` "h2" is stale.
    fn req<'a>(root: &'a Path, head: &'a str, authors: &'a [String]) -> SignRequest<'a> {
        SignRequest {
            root,
            repo: "paiml/x",
            pr: 7,
            head,
            pr_head: "h1",
            author_models: authors,
            ts: "2026-11-25T00:00:00Z",
        }
    }

    #[test]
    fn green_commit_passes() {
        assert_eq!(check(&green_commit(), &authors(), "h1", "h1"), Ok(()));
    }

    #[test]
    fn stale_head_refused() {
        let r = check(&green_commit(), &authors(), "h1", "h2");
        assert!(matches!(r, Err(Refusal::StaleHead { .. })));
    }

    #[test]
    fn quorum_must_be_three() {
        let mut c = green_commit();
        c.lanes.pop();
        assert!(matches!(
            check(&c, &authors(), "h1", "h1"),
            Err(Refusal::QuorumNotThreeOfThree(_))
        ));
    }

    #[test]
    fn one_model_refused() {
        let mut c = green_commit();
        for l in &mut c.lanes {
            l.model = "opus".into();
        }
        assert!(matches!(
            check(&c, &authors(), "h1", "h1"),
            Err(Refusal::FewerThanTwoModels(_))
        ));
    }

    #[test]
    fn seat_needs_identity_and_no_coding_worker() -> Result<()> {
        let mut c = green_commit();
        c.lanes.push(lane("seat:storage", "sonnet"));
        assert!(matches!(
            check(&c, &authors(), "h1", "h1"),
            Err(Refusal::SeatWithoutIdentity(_))
        ));
        let seat = c.lanes.last_mut().context("commit has no seat lane")?;
        seat.identity = Some("infra-9".into());
        seat.coding_worker = true;
        assert!(matches!(
            check(&c, &authors(), "h1", "h1"),
            Err(Refusal::CodingWorkerSeat(_))
        ));
        Ok(())
    }

    #[test]
    fn author_model_refused() {
        let mut c = green_commit();
        c.lanes[0].model = "haiku".into();
        assert!(matches!(
            check(&c, &authors(), "h1", "h1"),
            Err(Refusal::AuthorModel { .. })
        ));
    }

    #[test]
    fn missed_control_refused() {
        let mut c = green_commit();
        c.lanes[1].control_caught = false;
        assert!(matches!(
            check(&c, &authors(), "h1", "h1"),
            Err(Refusal::ControlMissed(_))
        ));
    }

    #[test]
    fn red_then_green_same_head_refused() {
        let mut c = green_commit();
        c.lanes[0].attempts = vec![
            Attempt {
                verdict: "FAIL".into(),
                head: "h1".into(),
            },
            Attempt {
                verdict: "PASS".into(),
                head: "h1".into(),
            },
        ];
        assert!(matches!(
            check(&c, &authors(), "h1", "h1"),
            Err(Refusal::RerunAsPass(_))
        ));
        c.lanes[0].attempts[1].head = "h0".into();
        assert_eq!(check(&c, &authors(), "h1", "h1"), Ok(()));
    }

    #[test]
    fn last_attempt_must_pass() {
        let mut c = green_commit();
        c.lanes[2].attempts.push(Attempt {
            verdict: "FAIL".into(),
            head: "h1".into(),
        });
        assert!(matches!(
            check(&c, &authors(), "h1", "h1"),
            Err(Refusal::NotGreen(_))
        ));
    }

    #[test]
    fn sign_writes_receipts_then_sign_json() -> Result<()> {
        let root = tempfile::tempdir().context("create temp dir")?;
        let commits = vec![green_commit()];
        let out = sign_pr(&req(root.path(), "h1", &authors()), &commits)
            .context("sign_pr should succeed for a green commit")?;
        let dir = root.path().join("paiml/x/7");
        assert_eq!(out, dir.join("sign.json"));
        let receipt = std::fs::read(dir.join("p1.json")).context("read p1.json receipt")?;
        let sign_bytes = std::fs::read(&out).context("read sign.json")?;
        let sign: Value = serde_json::from_slice(&sign_bytes).context("parse sign.json")?;
        assert_eq!(sign["receipt_sha256"]["p1"], hex_sha256(&receipt));
        assert_eq!(sign["patch_ids"], json!(["p1"]));
        assert!(!dir.join("sign.json.tmp").exists());
        Ok(())
    }

    #[test]
    fn lane_reason_falls_back_to_debug_for_head_refusals() {
        let r = Refusal::StaleHead {
            signed: "a".into(),
            pr_head: "b".into(),
        };
        assert!(r.lane_reason().starts_with("StaleHead"));
    }

    #[test]
    fn refusal_writes_nothing() -> Result<()> {
        let root = tempfile::tempdir().context("create temp dir")?;
        let commits = vec![green_commit()];
        assert!(sign_pr(&req(root.path(), "h2", &authors()), &commits).is_err());
        assert!(!root.path().join("paiml/x/7").exists());
        Ok(())
    }
}
