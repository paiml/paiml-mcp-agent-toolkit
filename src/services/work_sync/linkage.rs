//! PMAT-724 (goal-mode.md §11 step 5): the pure predicates behind CB-2112
//! (invariant A — every open item names an open issue numbered as the item's
//! numeric tail) and CB-2114 (invariants B/F1 — every open item carries a
//! `release:` naming an existing milestone its issue is on).

use super::{is_open, GithubSnapshot};
use crate::models::roadmap::Roadmap;

/// The trailing ASCII-digit run of an item id, or `None` when there is none.
pub fn numeric_tail(id: &str) -> Option<u64> {
    if id.is_empty() {
        return None;
    }
    let mut i = id.len();
    while i > 0 && id.as_bytes()[i - 1].is_ascii_digit() {
        i -= 1;
    }
    if i == id.len() {
        None
    } else {
        id[i..].parse().ok()
    }
}

/// One CB-2112 finding: why an open item is not linked to its own open issue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkFinding {
    /// `github_issue` is null.
    NoIssue { id: String, title: String },
    /// The named issue is not in the snapshot.
    IssueAbsent { id: String, number: u64 },
    /// The named issue is closed.
    IssueClosed { id: String, number: u64 },
    /// The named issue carries the `no-roadmap` label.
    IssueExcluded { id: String, number: u64 },
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
            | Self::IssueExcluded { id, .. }
            | Self::TailMismatch { id, .. } => id,
        }
    }
    /// The finding class word.
    pub fn class(&self) -> &'static str {
        match self {
            Self::NoIssue { .. } => "NO-ISSUE",
            Self::IssueAbsent { .. } => "ISSUE-ABSENT",
            Self::IssueClosed { .. } => "ISSUE-CLOSED",
            Self::IssueExcluded { .. } => "ISSUE-EXCLUDED",
            Self::TailMismatch { .. } => "TAIL-MISMATCH",
        }
    }
    /// Plain-text rendering (the printer paints).
    pub fn render(&self) -> String {
        match self {
            Self::NoIssue { id, .. } => format!("NO-ISSUE {id}: github_issue is null"),
            Self::IssueClosed { id, number } => format!("ISSUE-CLOSED {id}: #{number} is closed"),
            Self::IssueAbsent { id, number } => {
                format!("ISSUE-ABSENT {id}: #{number} does not exist on GitHub")
            }
            Self::IssueExcluded { id, number } => {
                format!("ISSUE-EXCLUDED {id}: #{number} is labelled no-roadmap")
            }
            Self::TailMismatch { id, number, tail } => {
                if let Some(t) = tail {
                    format!("TAIL-MISMATCH {id}: names #{number} but its numeric tail is {t}")
                } else {
                    format!("TAIL-MISMATCH {id}: names #{number} but the id has no numeric tail")
                }
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

/// Judge invariant A over the roadmap and the snapshot.
pub fn ticket_linkage(roadmap: &Roadmap, snapshot: &GithubSnapshot) -> LinkReport {
    let mut report = LinkReport::default();
    for item in &roadmap.roadmap {
        if !is_open(item) {
            continue;
        }
        report.open_items += 1;
        if let Some(n) = item.github_issue {
            if let Some(issue) = snapshot.issue(n) {
                if issue.state == super::IssueState::Closed {
                    report.findings.push(LinkFinding::IssueClosed {
                        id: item.id.clone(),
                        number: n,
                    });
                } else if !issue.in_universe() {
                    report.findings.push(LinkFinding::IssueExcluded {
                        id: item.id.clone(),
                        number: n,
                    });
                } else {
                    let tail = numeric_tail(&item.id);
                    if tail == Some(n) {
                        report.linked += 1;
                    } else {
                        report.findings.push(LinkFinding::TailMismatch {
                            id: item.id.clone(),
                            number: n,
                            tail,
                        });
                    }
                }
            } else {
                report.findings.push(LinkFinding::IssueAbsent {
                    id: item.id.clone(),
                    number: n,
                });
            }
        } else {
            report.findings.push(LinkFinding::NoIssue {
                id: item.id.clone(),
                title: item.title.clone(),
            });
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
            Self::NoIssue { .. } => "NO-ISSUE",
            Self::NotOnMilestone { .. } => "NOT-ON-MILESTONE",
        }
    }
    /// Plain-text rendering (the printer paints).
    pub fn render(&self) -> String {
        match self {
            Self::NoRelease { id, .. } => format!("NO-RELEASE {id}: release is null"),
            Self::Prefixed { id, release } => format!("PREFIXED {id}: release {release:?} carries a v — the key is the bare semver string (goal-mode.md §4.1)"),
            Self::NoMilestone { id, release } => format!("NO-MILESTONE {id}: no milestone titled {release:?}"),
            Self::NoIssue { id, release } => format!("NO-ISSUE {id}: release {release:?} cannot be checked against a milestone — github_issue is null (CB-2112)"),
            Self::NotOnMilestone { id, number, release, actual } => {
                if let Some(m) = actual {
                    format!("NOT-ON-MILESTONE {id}: #{number} is on milestone {m:?}, not {release:?}")
                } else {
                    format!("NOT-ON-MILESTONE {id}: #{number} is on no milestone")
                }
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

/// Judge invariants B/F1 over the roadmap and the snapshot.
pub fn release_binding(roadmap: &Roadmap, snapshot: &GithubSnapshot) -> ReleaseReport {
    let mut report = ReleaseReport::default();
    for item in &roadmap.roadmap {
        if !is_open(item) {
            continue;
        }
        report.open_items += 1;
        if let Some(release) = &item.release {
            if release.starts_with('v') || release.starts_with('V') {
                report.findings.push(ReleaseFinding::Prefixed {
                    id: item.id.clone(),
                    release: release.clone(),
                });
            } else if !snapshot.milestones.iter().any(|m| m.title == *release) {
                report.findings.push(ReleaseFinding::NoMilestone {
                    id: item.id.clone(),
                    release: release.clone(),
                });
            } else if let Some(n) = item.github_issue {
                let issue = snapshot.issue(n);
                let actual = issue.and_then(|i| i.milestone.clone());
                if issue.is_none() || actual.as_deref() != Some(release) {
                    report.findings.push(ReleaseFinding::NotOnMilestone {
                        id: item.id.clone(),
                        number: n,
                        release: release.clone(),
                        actual,
                    });
                } else {
                    report.bound += 1;
                }
            } else {
                report.findings.push(ReleaseFinding::NoIssue {
                    id: item.id.clone(),
                    release: release.clone(),
                });
            }
        } else {
            report.findings.push(ReleaseFinding::NoRelease {
                id: item.id.clone(),
                title: item.title.clone(),
            });
        }
    }
    report.findings.sort_by(|a, b| a.id().cmp(b.id()));
    report
}
