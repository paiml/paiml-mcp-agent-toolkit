//! Work sync engine (goal-mode.md §4.1, §5; PMAT-720).
//!
//! Judges a [`Roadmap`] against a [`GithubSnapshot`] and plans the fixers. It is
//! pure: nothing here touches the network or the file system. The handler fetches
//! the snapshot (or reads one from a file) and executes the plan, so every verdict
//! is reproducible from a JSON file and every fixer is a listed [`Action`] before
//! it is a side effect.
//!
//! The set predicate (§5.1) has tolerance ZERO and three finding classes, reported
//! separately because they have different fixes:
//!
//! * `ORPHAN-ROADMAP` — an open item with no issue, or naming a closed or absent
//!   one. Fixed by `--direction yaml-to-github` (no issue) or `github-to-yaml`
//!   (closed issue: GitHub is authoritative for state, §5.3).
//! * `ORPHAN-GITHUB` — an open issue, not labelled `no-roadmap`, that no open item
//!   names. Fixed by `--direction github-to-yaml`.
//! * `COLLISION` — one issue named by two or more open items. **Never fixed by
//!   the sync** (§5.4): which item is really the issue is a judgement, and a sync
//!   that guesses is how 13 items came to name #612.
//!
//! The field predicate (§5.2) has tolerance TIME: a matched pair whose title or
//! release disagrees is a finding only once `now − max(item.updated,
//! issue.updated_at)` exceeds the grace window. A bound in time, not a count —
//! "up to 3 may disagree" means three items may be wrong forever.
//!
//! "Open" here is every non-terminal status — `planned`, `inprogress`, `blocked`
//! and `review` — because a `blocked` item with no issue is exactly as invisible
//! to GitHub as a `planned` one. The specification names the first two; the enum
//! has four, and a bijection that ignores two of them has a hole.

use crate::models::roadmap::{ItemStatus, Roadmap, RoadmapItem};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Issues carrying this label are outside the roadmap's universe (§5.1, §9).
pub const NO_ROADMAP_LABEL: &str = "no-roadmap";
/// §5.2's default grace window.
pub const DEFAULT_GRACE_MINUTES: i64 = 60;

/// GitHub's open/closed state, as the snapshot records it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueState {
    Open,
    Closed,
}

/// Why a closed issue was closed (GitHub's `stateReason`). `NotPlanned` maps to a
/// `cancelled` item (§9: closed `wontfix` is terminal and legal); anything else
/// to `completed`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CloseReason {
    Completed,
    NotPlanned,
    Duplicate,
}

/// One issue as the snapshot saw it. Pull requests are never in a snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IssueSnapshot {
    pub number: u64,
    pub title: String,
    pub state: IssueState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_reason: Option<CloseReason>,
    #[serde(default)]
    pub labels: Vec<String>,
    /// The milestone's exact title — the release key (§4.1), never prefixed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub milestone: Option<String>,
    pub updated_at: DateTime<Utc>,
}

impl IssueSnapshot {
    /// Member of **G** (§5.1): open and not labelled `no-roadmap`.
    pub fn in_universe(&self) -> bool {
        self.state == IssueState::Open && !self.labels.iter().any(|l| l == NO_ROADMAP_LABEL)
    }
}

/// One milestone as the snapshot saw it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MilestoneSnapshot {
    pub title: String,
    pub state: IssueState,
}

/// Everything the engine needs from GitHub, taken at one instant. Written and
/// read as JSON so a run can be replayed without the network.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GithubSnapshot {
    pub repo: String,
    pub taken_at: DateTime<Utc>,
    pub issues: Vec<IssueSnapshot>,
    #[serde(default)]
    pub milestones: Vec<MilestoneSnapshot>,
}

impl GithubSnapshot {
    /// Parse a snapshot written by [`GithubSnapshot::to_json`] (or by hand, for a control).
    pub fn from_json(text: &str) -> anyhow::Result<Self> {
        serde_json::from_str(text).map_err(|e| anyhow::anyhow!("snapshot is not valid JSON: {e}"))
    }

    /// Serialize for replay.
    pub fn to_json(&self) -> anyhow::Result<String> {
        serde_json::to_string_pretty(self).map_err(Into::into)
    }

    /// The issue with this number, open or closed.
    pub fn issue(&self, number: u64) -> Option<&IssueSnapshot> {
        self.issues.iter().find(|i| i.number == number)
    }
}

/// The clock and the grace window (§5.2). Passed in, never read from the wall
/// inside the engine, so a verdict can be replayed.
#[derive(Debug, Clone, Copy)]
pub struct Settings {
    pub now: DateTime<Utc>,
    pub grace: Duration,
}

impl Settings {
    pub fn new(now: DateTime<Utc>, grace_minutes: i64) -> Self {
        Self {
            now,
            grace: Duration::minutes(grace_minutes),
        }
    }
}

/// Why an open item is an `ORPHAN-ROADMAP`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OrphanReason {
    /// `github_issue: null`.
    NoIssue,
    /// The issue exists and is closed.
    IssueClosed,
    /// No issue with that number is in the snapshot.
    IssueAbsent,
}

/// Which field of a matched pair disagrees (§5.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DriftField {
    Title,
    /// `item.release` vs the issue's milestone title — the projection is stale (§4.1).
    Release,
}

/// One finding. The JSON `class` tag is the spec's name for it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "class", rename_all = "SCREAMING-KEBAB-CASE")]
pub enum Finding {
    OrphanRoadmap {
        id: String,
        title: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        github_issue: Option<u64>,
        reason: OrphanReason,
    },
    OrphanGithub {
        number: u64,
        title: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        milestone: Option<String>,
    },
    Collision {
        number: u64,
        ids: Vec<String>,
    },
    Drift {
        id: String,
        number: u64,
        field: DriftField,
        roadmap: String,
        github: String,
        age_minutes: i64,
    },
}

impl Finding {
    /// The spec's class name: `ORPHAN-ROADMAP`, `ORPHAN-GITHUB`, `COLLISION`, `DRIFT`.
    pub fn class(&self) -> &'static str {
        match self {
            Finding::OrphanRoadmap { .. } => "ORPHAN-ROADMAP",
            Finding::OrphanGithub { .. } => "ORPHAN-GITHUB",
            Finding::Collision { .. } => "COLLISION",
            Finding::Drift { .. } => "DRIFT",
        }
    }
}

/// The verdict of one [`check`].
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SyncReport {
    /// |R|: items with a non-terminal status.
    pub open_items: usize,
    /// |G|: open issues not labelled `no-roadmap`.
    pub open_issues: usize,
    /// Pairs (open item, open issue) with exactly one owner.
    pub matched: usize,
    /// Field disagreements inside the grace window: counted, not findings.
    pub tolerated: usize,
    pub findings: Vec<Finding>,
}

impl SyncReport {
    /// §5.1 and §5.2 both hold.
    pub fn is_coherent(&self) -> bool {
        self.findings.is_empty()
    }

    /// How many findings carry this class (see [`Finding::class`]).
    pub fn count(&self, class: &str) -> usize {
        self.findings.iter().filter(|f| f.class() == class).count()
    }
}

/// Member of **R** (§5.1): every non-terminal status.
pub fn is_open(item: &RoadmapItem) -> bool {
    !matches!(item.status, ItemStatus::Completed | ItemStatus::Cancelled)
}

/// Judge the roadmap against the snapshot. Pure; deterministic order.
pub fn check(_roadmap: &Roadmap, _snapshot: &GithubSnapshot, _settings: &Settings) -> SyncReport {
    SyncReport::default()
}

/// Which side the fixers write to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    YamlToGithub,
    GithubToYaml,
    Full,
}

/// One fixer step. The engine lists them; the handler performs them, in order.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "kebab-case")]
pub enum Action {
    /// yaml→github: open an issue for an open item that has none. Performed by
    /// the handler (network); followed by a [`Action::LinkIssue`].
    CreateIssue { id: String, title: String },
    /// Record the number of an issue the handler just created.
    LinkIssue { id: String, number: u64 },
    /// github→yaml: an open issue no item names becomes a `GH-<n>` item.
    CreateItem {
        number: u64,
        title: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        release: Option<String>,
    },
    /// github→yaml: the issue is closed, so the item is terminal (§5.3).
    CloseItem {
        id: String,
        number: u64,
        status: ItemStatus,
    },
    /// github→yaml: project the milestone title into `release` (§4.1).
    SetRelease {
        id: String,
        number: u64,
        #[serde(skip_serializing_if = "Option::is_none")]
        release: Option<String>,
    },
    /// Nothing is done, and this is why. Every collided item gets one.
    Skip { id: String, reason: String },
}

/// Plan the fixers for `direction` from a report. Pure.
pub fn plan(
    _roadmap: &Roadmap,
    _snapshot: &GithubSnapshot,
    _report: &SyncReport,
    _direction: Direction,
) -> Vec<Action> {
    Vec::new()
}

/// Apply the roadmap-side actions (`LinkIssue`, `CreateItem`, `CloseItem`,
/// `SetRelease`) to `roadmap`; `CreateIssue` and `Skip` are no-ops here. Returns
/// how many actions changed something. Every touched item gets `updated = now`.
pub fn apply_to_roadmap(_roadmap: &mut Roadmap, _actions: &[Action], _now: DateTime<Utc>) -> usize {
    0
}

// Keep the collection imports live for the engine body that lands next.
#[allow(dead_code)]
type Groups = BTreeMap<u64, BTreeSet<String>>;

#[cfg(test)]
mod tests;
