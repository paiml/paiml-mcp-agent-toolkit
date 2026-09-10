//! PMAT-724 (goal-mode.md §11 step 5): the pure predicates behind CB-2112
//! (invariant A — every open item names an open issue numbered as the item's
//! numeric tail) and CB-2114 (invariants B/F1 — every open item carries a
//! `release:` naming an existing milestone its issue is on).
//!
//! RED stub: the surface the tests name, with bodies that judge nothing. The
//! implementation phase replaces the bodies.

use super::GithubSnapshot;
use crate::models::roadmap::Roadmap;

/// The trailing ASCII-digit run of an item id, or `None` when there is none.
pub fn numeric_tail(_id: &str) -> Option<u64> {
    None
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
        format!("{} {}", self.class(), self.id())
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
pub fn ticket_linkage(_roadmap: &Roadmap, _snapshot: &GithubSnapshot) -> LinkReport {
    LinkReport::default()
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
        format!("{} {}", self.class(), self.id())
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
pub fn release_binding(_roadmap: &Roadmap, _snapshot: &GithubSnapshot) -> ReleaseReport {
    ReleaseReport::default()
}
