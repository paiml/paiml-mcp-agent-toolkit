//! PMAT-724 (goal-mode.md §11 step 5): the pure predicates behind CB-2112
//! (invariant A — every open item names an open issue numbered as the item's
//! numeric tail) and CB-2114 (invariants B/F1 — every open item carries a
//! `release:` naming an existing, open milestone its issue is on).
//!
//! Pure and deterministic: no I/O, no clock, findings ordered by item id so
//! two runs on one input render one message. The printer paints; nothing
//! here carries colour.

use super::{is_open, GithubSnapshot, IssueState};
use crate::models::roadmap::Roadmap;

/// The trailing ASCII-digit run of an item id, or `None` when there is none:
/// `PMAT-724` → 724, `PERF-001` → 1, `EPIC` → `None`, `PMAT-7a` → `None`.
pub fn numeric_tail(id: &str) -> Option<u64> {
    let digits = id.len() - id.trim_end_matches(|c: char| c.is_ascii_digit()).len();
    if digits == 0 {
        return None;
    }
    id[id.len() - digits..].parse().ok()
}

/// One CB-2112 finding: why an open item is not linked to its own open issue.
///
/// An open issue labelled `no-roadmap` is still an open issue here: the label
/// is §5.1's business (it removes the issue from G, and CB-2115 reports the
/// item naming it as ORPHAN-ROADMAP), not invariant A's — one place per
/// judgement (doctrine 5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkFinding {
    /// `github_issue` is null.
    NoIssue { id: String, title: String },
    /// The named issue is not in the snapshot.
    IssueAbsent { id: String, number: u64 },
    /// The named issue is closed.
    IssueClosed { id: String, number: u64 },
    /// The named issue's number is not the item's numeric tail.
    TailMismatch {
        id: String,
        number: u64,
        tail: Option<u64>,
    },
}

impl LinkFinding {
    /// The item the finding is about.
    pub fn id(&self) -> &str {
        match self {
            Self::NoIssue { id, .. }
            | Self::IssueAbsent { id, .. }
            | Self::IssueClosed { id, .. }
            | Self::TailMismatch { id, .. } => id,
        }
    }
    /// The finding class word.
    pub fn class(&self) -> &'static str {
        match self {
            Self::NoIssue { .. } => "NO-ISSUE",
            Self::IssueAbsent { .. } => "ISSUE-ABSENT",
            Self::IssueClosed { .. } => "ISSUE-CLOSED",
            Self::TailMismatch { .. } => "TAIL-MISMATCH",
        }
    }
    /// Plain-text rendering (the printer paints).
    pub fn render(&self) -> String {
        match self {
            Self::NoIssue { id, .. } => format!("NO-ISSUE {id}: github_issue is null"),
            Self::IssueAbsent { id, number } => {
                format!("ISSUE-ABSENT {id}: #{number} does not exist on GitHub")
            }
            Self::IssueClosed { id, number } => format!("ISSUE-CLOSED {id}: #{number} is closed"),
            Self::TailMismatch {
                id,
                number,
                tail: Some(t),
            } => {
                format!("TAIL-MISMATCH {id}: names #{number} but its numeric tail is {t}")
            }
            Self::TailMismatch {
                id,
                number,
                tail: None,
            } => {
                format!("TAIL-MISMATCH {id}: names #{number} but the id has no numeric tail")
            }
        }
    }
}

/// CB-2112's report.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LinkReport {
    pub open_items: usize,
    pub linked: usize,
    pub findings: Vec<LinkFinding>,
}

/// Judge invariant A over the roadmap and the snapshot: one finding per open
/// item, the first clause that fails (no issue; absent; closed; not the tail).
pub fn ticket_linkage(roadmap: &Roadmap, snapshot: &GithubSnapshot) -> LinkReport {
    let mut report = LinkReport::default();
    for item in roadmap.roadmap.iter().filter(|i| is_open(i)) {
        report.open_items += 1;
        let id = item.id.clone();
        let Some(number) = item.github_issue else {
            report.findings.push(LinkFinding::NoIssue {
                id,
                title: item.title.clone(),
            });
            continue;
        };
        let Some(issue) = snapshot.issue(number) else {
            report
                .findings
                .push(LinkFinding::IssueAbsent { id, number });
            continue;
        };
        if issue.state == IssueState::Closed {
            report
                .findings
                .push(LinkFinding::IssueClosed { id, number });
            continue;
        }
        let tail = numeric_tail(&item.id);
        if tail == Some(number) {
            report.linked += 1;
        } else {
            report
                .findings
                .push(LinkFinding::TailMismatch { id, number, tail });
        }
    }
    report.findings.sort_by(|a, b| a.id().cmp(b.id()));
    report
}

/// One CB-2114 finding: why an open item is not bound to its release.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReleaseFinding {
    /// `release` is null.
    NoRelease { id: String, title: String },
    /// `release` carries a `v` prefix (§4.1: the key is the bare string).
    Prefixed { id: String, release: String },
    /// No milestone with that exact title.
    NoMilestone { id: String, release: String },
    /// The milestone exists and is closed: a shipped release cannot hold
    /// open work (§4.1 RR-RELEASE — a tag is cut only when its milestone has
    /// zero open issues, and a milestone closes only when its tag exists).
    MilestoneClosed { id: String, release: String },
    /// No issue to check milestone membership against.
    NoIssue { id: String, release: String },
    /// The issue is on another milestone, or none.
    NotOnMilestone {
        id: String,
        number: u64,
        release: String,
        actual: Option<String>,
    },
}

impl ReleaseFinding {
    /// The item the finding is about.
    pub fn id(&self) -> &str {
        match self {
            Self::NoRelease { id, .. }
            | Self::Prefixed { id, .. }
            | Self::NoMilestone { id, .. }
            | Self::MilestoneClosed { id, .. }
            | Self::NoIssue { id, .. }
            | Self::NotOnMilestone { id, .. } => id,
        }
    }
    /// The finding class word.
    pub fn class(&self) -> &'static str {
        match self {
            Self::NoRelease { .. } => "NO-RELEASE",
            Self::Prefixed { .. } => "PREFIXED",
            Self::NoMilestone { .. } => "NO-MILESTONE",
            Self::MilestoneClosed { .. } => "MILESTONE-CLOSED",
            Self::NoIssue { .. } => "NO-ISSUE",
            Self::NotOnMilestone { .. } => "NOT-ON-MILESTONE",
        }
    }
    /// Plain-text rendering (the printer paints).
    pub fn render(&self) -> String {
        match self {
            Self::NoRelease { id, .. } => format!("NO-RELEASE {id}: release is null"),
            Self::Prefixed { id, release } => format!(
                "PREFIXED {id}: release {release:?} carries a v — the key is the bare semver string (goal-mode.md §4.1)"
            ),
            Self::NoMilestone { id, release } => format!("NO-MILESTONE {id}: no milestone titled {release:?}"),
            Self::MilestoneClosed { id, release } => format!(
                "MILESTONE-CLOSED {id}: milestone {release:?} is closed — a shipped release cannot hold open work (goal-mode.md §4.1: a tag is cut only when its milestone has zero open issues)"
            ),
            Self::NoIssue { id, release } => format!(
                "NO-ISSUE {id}: release {release:?} cannot be checked against a milestone — github_issue is null (CB-2112)"
            ),
            Self::NotOnMilestone { id, number, release, actual: Some(m) } => {
                format!("NOT-ON-MILESTONE {id}: #{number} is on milestone {m:?}, not {release:?}")
            }
            Self::NotOnMilestone { id, number, actual: None, .. } => {
                format!("NOT-ON-MILESTONE {id}: #{number} is on no milestone")
            }
        }
    }
}

/// CB-2114's report.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReleaseReport {
    pub open_items: usize,
    pub bound: usize,
    pub findings: Vec<ReleaseFinding>,
}

/// Judge invariants B/F1 over the roadmap and the snapshot: one finding per
/// open item, the first clause that fails (no release; prefixed; no such
/// milestone; milestone closed; no issue; issue not on it).
pub fn release_binding(roadmap: &Roadmap, snapshot: &GithubSnapshot) -> ReleaseReport {
    let mut report = ReleaseReport::default();
    for item in roadmap.roadmap.iter().filter(|i| is_open(i)) {
        report.open_items += 1;
        let id = item.id.clone();
        let Some(release) = item.release.clone() else {
            report.findings.push(ReleaseFinding::NoRelease {
                id,
                title: item.title.clone(),
            });
            continue;
        };
        if release.starts_with(['v', 'V']) {
            report
                .findings
                .push(ReleaseFinding::Prefixed { id, release });
            continue;
        }
        let Some(milestone) = snapshot.milestones.iter().find(|m| m.title == release) else {
            report
                .findings
                .push(ReleaseFinding::NoMilestone { id, release });
            continue;
        };
        if milestone.state == IssueState::Closed {
            report
                .findings
                .push(ReleaseFinding::MilestoneClosed { id, release });
            continue;
        }
        let Some(number) = item.github_issue else {
            report
                .findings
                .push(ReleaseFinding::NoIssue { id, release });
            continue;
        };
        let actual = snapshot.issue(number).and_then(|i| i.milestone.clone());
        if actual.as_deref() == Some(release.as_str()) {
            report.bound += 1;
        } else {
            report.findings.push(ReleaseFinding::NotOnMilestone {
                id,
                number,
                release,
                actual,
            });
        }
    }
    report.findings.sort_by(|a, b| a.id().cmp(b.id()));
    report
}
