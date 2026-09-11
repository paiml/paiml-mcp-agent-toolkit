//! `pmat spec review --record <json>` (goal-mode.md §6.3): validate, then
//! stage. It does not produce. Production is a quorum's job or a human's —
//! the artifact is JSON, and a hand-written review with real findings is a
//! legitimate one.

use super::ReviewFinding;
use std::path::{Path, PathBuf};

/// What `--record` staged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recorded {
    /// Project-relative: `docs/audits/spec-<slug>-review.json`.
    pub artifact: String,
    pub spec: String,
    pub spec_sha256: String,
}

/// Why `--record` refused. Every refusal before [`RecordRefusal::Unwritable`]
/// writes nothing and stages nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordRefusal {
    Unreadable { file: PathBuf, reason: String },
    BadReview { file: PathBuf, reason: String },
    NotASpec { named: String },
    SpecUnreadable { spec: String, reason: String },
    Unjudgeable { spec: String, why: String },
    Findings(Vec<ReviewFinding>),
    Unwritable { artifact: String, reason: String },
    NotStaged { artifact: String, reason: String },
}

impl RecordRefusal {
    /// One line a reader can act on.
    pub fn render(&self) -> String {
        match self {
            RecordRefusal::Unreadable { file, reason } => format!("{} cannot be read ({reason}); nothing recorded", file.display()),
            RecordRefusal::BadReview { file, reason } => format!("BAD-REVIEW {}: does not parse as a review (goal-mode.md §6.1): {reason}; nothing recorded", file.display()),
            RecordRefusal::NotASpec { named } => format!("the review names {named}, which is not a spec under docs/specifications/ (a relative path ending .md, no . or .. segment); nothing recorded"),
            RecordRefusal::SpecUnreadable { spec, reason } => format!("the review names {spec}, which cannot be read ({reason}); nothing recorded"),
            RecordRefusal::Unjudgeable { spec, why } => format!("UNJUDGEABLE {spec}: its front-matter does not parse, so the roles its review needs cannot be read ({why}); nothing recorded"),
            RecordRefusal::Findings(findings) => format!("{} finding(s) — the review would fail CB-2111 as recorded; nothing recorded", findings.len()),
            RecordRefusal::Unwritable { artifact, reason } => format!("{artifact} cannot be written ({reason}); nothing staged"),
            RecordRefusal::NotStaged { artifact, reason } => format!("{artifact} is written but NOT staged: git add failed ({reason})"),
        }
    }
}

/// Validate the review at `file` against the spec it names under `project`,
/// exactly as CB-2111 will judge it; on success write it, byte for byte, to
/// its artifact path and `git add` it.
pub fn record(_project: &Path, _file: &Path) -> Result<Recorded, RecordRefusal> {
    Err(RecordRefusal::Unreadable {
        file: PathBuf::new(),
        reason: "RED stub".to_string(),
    })
}
