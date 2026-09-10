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
pub mod github;

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
    /// The issue is open but carries the `no-roadmap` label, so it is outside
    /// **G** (§5.1): the item points out of the universe. Quorum finding, PMAT-720.
    IssueExcluded,
}

/// Which field of a matched pair disagrees (§5.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DriftField {
    Title,
    /// `item.release` vs the issue's milestone title — the projection is stale (§4.1).
    Release,
}

impl DriftField {
    /// The field as the spec and the roadmap spell it.
    pub fn name(&self) -> &'static str {
        match self {
            DriftField::Title => "title",
            DriftField::Release => "release",
        }
    }
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
    /// One line of plain text, the same wording wherever the finding is shown:
    /// the class word first, then the item id and/or `#number`, then the reason.
    /// No colour and no padding here — this engine is pure, and a comply
    /// message or a JSON report must never carry a terminal escape; the
    /// `pmat work sync` printer paints the class word itself.
    pub fn render(&self) -> String {
        match self {
            Finding::Collision { number, ids } => format!(
                "COLLISION #{number} is named by {} ({} items)",
                ids.join(", "),
                ids.len()
            ),
            Finding::OrphanRoadmap {
                id,
                github_issue,
                reason,
                ..
            } => {
                let issue = github_issue.map_or_else(|| "?".to_string(), |n| format!("#{n}"));
                let why = match reason {
                    OrphanReason::NoIssue => "no issue".to_string(),
                    OrphanReason::IssueClosed => format!("{issue} is closed"),
                    OrphanReason::IssueAbsent => format!("{issue} does not exist on GitHub"),
                    OrphanReason::IssueExcluded => format!("{issue} is labelled no-roadmap"),
                };
                format!("ORPHAN-ROADMAP {id}: {why}")
            }
            Finding::OrphanGithub { number, title, .. } => {
                format!("ORPHAN-GITHUB #{number}: {title}")
            }
            Finding::Drift {
                id,
                number,
                field,
                roadmap,
                github,
                age_minutes,
            } => format!(
                "DRIFT {id} <-> #{number} {}: roadmap {roadmap:?} vs GitHub {github:?} ({age_minutes} min, past the grace window)",
                field.name()
            ),
        }
    }

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
pub fn check(roadmap: &Roadmap, snapshot: &GithubSnapshot, settings: &Settings) -> SyncReport {
    let mut report = SyncReport::default();

    let mut open_items = Vec::new();
    for item in &roadmap.roadmap {
        if is_open(item) {
            open_items.push(item);
        }
    }
    report.open_items = open_items.len();

    let mut g_issues = Vec::new();
    for issue in &snapshot.issues {
        if issue.in_universe() {
            g_issues.push(issue);
        }
    }
    report.open_issues = g_issues.len();

    let mut named_issues: BTreeMap<u64, BTreeSet<String>> = BTreeMap::new();
    for item in &open_items {
        if let Some(n) = item.github_issue {
            named_issues.entry(n).or_default().insert(item.id.clone());
        }
    }

    let mut collided_ids: BTreeSet<String> = BTreeSet::new();
    let mut collisions = Vec::new();
    for (n, ids) in &named_issues {
        if ids.len() >= 2 {
            collisions.push(Finding::Collision {
                number: *n,
                ids: ids.iter().cloned().collect(),
            });
            for id in ids {
                collided_ids.insert(id.clone());
            }
        }
    }
    collisions.sort_by_key(|f| match f {
        Finding::Collision { number, .. } => *number,
        _ => 0,
    });
    report.findings.extend(collisions);

    let mut orphan_roadmap = Vec::new();
    let mut matched_pairs = Vec::new(); // (item, issue)

    for item in &open_items {
        if collided_ids.contains(&item.id) {
            continue;
        }
        match item.github_issue {
            None => {
                orphan_roadmap.push(Finding::OrphanRoadmap {
                    id: item.id.clone(),
                    title: item.title.clone(),
                    github_issue: None,
                    reason: OrphanReason::NoIssue,
                });
            }
            Some(n) => {
                if let Some(issue) = snapshot.issue(n) {
                    if issue.state == IssueState::Closed {
                        orphan_roadmap.push(Finding::OrphanRoadmap {
                            id: item.id.clone(),
                            title: item.title.clone(),
                            github_issue: Some(n),
                            reason: OrphanReason::IssueClosed,
                        });
                    } else if !issue.in_universe() {
                        orphan_roadmap.push(Finding::OrphanRoadmap {
                            id: item.id.clone(),
                            title: item.title.clone(),
                            github_issue: Some(n),
                            reason: OrphanReason::IssueExcluded,
                        });
                    } else {
                        matched_pairs.push((item, issue));
                    }
                } else {
                    orphan_roadmap.push(Finding::OrphanRoadmap {
                        id: item.id.clone(),
                        title: item.title.clone(),
                        github_issue: Some(n),
                        reason: OrphanReason::IssueAbsent,
                    });
                }
            }
        }
    }
    orphan_roadmap.sort_by_key(|f| match f {
        Finding::OrphanRoadmap { id, .. } => id.clone(),
        _ => String::new(),
    });
    report.findings.extend(orphan_roadmap);

    report.matched = matched_pairs.len();

    let mut orphan_github = Vec::new();
    for issue in &g_issues {
        if !named_issues.contains_key(&issue.number) {
            orphan_github.push(Finding::OrphanGithub {
                number: issue.number,
                title: issue.title.clone(),
                milestone: issue.milestone.clone(),
            });
        }
    }
    orphan_github.sort_by_key(|f| match f {
        Finding::OrphanGithub { number, .. } => *number,
        _ => 0,
    });
    report.findings.extend(orphan_github);

    let mut drift_findings = Vec::new();
    for (item, issue) in matched_pairs {
        let title_differs = item.title.trim() != issue.title.trim();
        let release_differs = item.release != issue.milestone;

        if title_differs || release_differs {
            let item_dt = chrono::DateTime::parse_from_rfc3339(&item.updated)
                .map(|dt| dt.with_timezone(&Utc));
            let age_minutes = match item_dt {
                Ok(dt) => {
                    let max_dt = if dt > issue.updated_at {
                        dt
                    } else {
                        issue.updated_at
                    };
                    (settings.now - max_dt).num_minutes()
                }
                Err(_) => i64::MAX,
            };

            if age_minutes < settings.grace.num_minutes() {
                if title_differs {
                    report.tolerated += 1;
                }
                if release_differs {
                    report.tolerated += 1;
                }
            } else {
                if title_differs {
                    drift_findings.push(Finding::Drift {
                        id: item.id.clone(),
                        number: issue.number,
                        field: DriftField::Title,
                        roadmap: item.title.clone(),
                        github: issue.title.clone(),
                        age_minutes,
                    });
                }
                if release_differs {
                    drift_findings.push(Finding::Drift {
                        id: item.id.clone(),
                        number: issue.number,
                        field: DriftField::Release,
                        roadmap: item.release.clone().unwrap_or_default(),
                        github: issue.milestone.clone().unwrap_or_default(),
                        age_minutes,
                    });
                }
            }
        }
    }

    drift_findings.sort_by(|a, b| {
        let (id_a, field_a) = match a {
            Finding::Drift { id, field, .. } => (id, field),
            _ => unreachable!(),
        };
        let (id_b, field_b) = match b {
            Finding::Drift { id, field, .. } => (id, field),
            _ => unreachable!(),
        };
        let f_a = match field_a {
            DriftField::Title => 0,
            DriftField::Release => 1,
        };
        let f_b = match field_b {
            DriftField::Title => 0,
            DriftField::Release => 1,
        };
        id_a.cmp(id_b).then(f_a.cmp(&f_b))
    });
    report.findings.extend(drift_findings);

    report
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
    snapshot: &GithubSnapshot,
    report: &SyncReport,
    direction: Direction,
) -> Vec<Action> {
    let mut actions = Vec::new();
    for finding in &report.findings {
        match finding {
            Finding::Collision { ids, number } => {
                for id in ids {
                    let other_ids: Vec<String> = ids.iter().filter(|x| *x != id).cloned().collect();
                    actions.push(Action::Skip {
                        id: id.clone(),
                        reason: format!(
                            "COLLISION with #{} (§5.4): also named by {}",
                            number,
                            other_ids.join(", ")
                        ),
                    });
                }
            }
            Finding::OrphanRoadmap {
                id,
                title,
                github_issue,
                reason,
            } => match reason {
                OrphanReason::NoIssue => match direction {
                    Direction::YamlToGithub | Direction::Full => {
                        actions.push(Action::CreateIssue {
                            id: id.clone(),
                            title: title.clone(),
                        });
                    }
                    Direction::GithubToYaml => {
                        actions.push(Action::Skip {
                            id: id.clone(),
                            reason: "no issue — run --direction yaml-to-github to open one"
                                .to_string(),
                        });
                    }
                },
                OrphanReason::IssueClosed => {
                    let n = github_issue.expect("issue closed must have a number");
                    match direction {
                        Direction::GithubToYaml | Direction::Full => {
                            let status = if let Some(issue) = snapshot.issue(n) {
                                if issue.state_reason == Some(CloseReason::NotPlanned) {
                                    ItemStatus::Cancelled
                                } else {
                                    ItemStatus::Completed
                                }
                            } else {
                                ItemStatus::Completed
                            };
                            actions.push(Action::CloseItem {
                                id: id.clone(),
                                number: n,
                                status,
                            });
                        }
                        Direction::YamlToGithub => {
                            actions.push(Action::Skip {
                            id: id.clone(),
                            reason: format!("#{n} is closed and GitHub is authoritative for state (§5.3) — run --direction github-to-yaml"),
                        });
                        }
                    }
                }
                OrphanReason::IssueAbsent => {
                    actions.push(Action::Skip {
                        id: id.clone(),
                        reason: format!(
                            "#{} does not exist on GitHub — a human decides whether the number is a typo or the item is stale",
                            github_issue.expect("absent must have number")
                        ),
                    });
                }
                OrphanReason::IssueExcluded => {
                    actions.push(Action::Skip {
                        id: id.clone(),
                        reason: format!(
                            "#{} carries the no-roadmap label, so it is outside the universe (§5.1) — a human removes the label or cancels the item",
                            github_issue.expect("excluded must have number")
                        ),
                    });
                }
            },
            Finding::OrphanGithub {
                number,
                title,
                milestone,
            } => match direction {
                Direction::GithubToYaml | Direction::Full => {
                    actions.push(Action::CreateItem {
                        number: *number,
                        title: title.clone(),
                        release: milestone.clone(),
                    });
                }
                Direction::YamlToGithub => {
                    actions.push(Action::Skip {
                        id: format!("#{}", number),
                        reason: "the fix is on the roadmap side — run --direction github-to-yaml"
                            .to_string(),
                    });
                }
            },
            Finding::Drift {
                id,
                number,
                field,
                roadmap: _,
                github: _,
                age_minutes: _,
            } => {
                match field {
                    DriftField::Release => match direction {
                        Direction::GithubToYaml | Direction::Full => {
                            let issue = snapshot.issue(*number).expect("issue exists");
                            actions.push(Action::SetRelease {
                                id: id.clone(),
                                number: *number,
                                release: issue.milestone.clone(),
                            });
                        }
                        Direction::YamlToGithub => {
                            actions.push(Action::Skip {
                            id: id.clone(),
                            reason: "the fix is on the roadmap side — run --direction github-to-yaml".to_string(),
                        });
                        }
                    },
                    DriftField::Title => {
                        actions.push(Action::Skip {
                        id: id.clone(),
                        reason: "the title differs and no side is authoritative for it (§5.3): reported, never written".to_string(),
                    });
                    }
                }
            }
        }
    }
    actions
}

/// Apply the roadmap-side actions (`LinkIssue`, `CreateItem`, `CloseItem`,
/// `SetRelease`) to `roadmap`; `CreateIssue` and `Skip` are no-ops here. Returns
/// how many actions changed something. Every touched item gets `updated = now`.
pub fn apply_to_roadmap(roadmap: &mut Roadmap, actions: &[Action], now: DateTime<Utc>) -> usize {
    let mut count = 0;
    let now_str = now.to_rfc3339();

    for action in actions {
        match action {
            Action::LinkIssue { id, number } => {
                if let Some(item) = roadmap.roadmap.iter_mut().find(|i| &i.id == id) {
                    item.github_issue = Some(*number);
                    item.updated = now_str.clone();
                    count += 1;
                }
            }
            Action::CreateItem {
                number,
                title,
                release,
            } => {
                let id = format!("GH-{}", number);
                if !roadmap.roadmap.iter().any(|i| i.id == id) {
                    let mut item = RoadmapItem::from_github_issue(*number, title.clone());
                    item.release = release.clone();
                    item.created = now_str.clone();
                    item.updated = now_str.clone();
                    roadmap.roadmap.push(item);
                    count += 1;
                }
            }
            Action::CloseItem {
                id,
                number: _,
                status,
            } => {
                if let Some(item) = roadmap.roadmap.iter_mut().find(|i| &i.id == id) {
                    item.status = *status;
                    item.updated = now_str.clone();
                    count += 1;
                }
            }
            Action::SetRelease {
                id,
                number: _,
                release,
            } => {
                if let Some(item) = roadmap.roadmap.iter_mut().find(|i| &i.id == id) {
                    item.release = release.clone();
                    item.updated = now_str.clone();
                    count += 1;
                }
            }
            Action::CreateIssue { .. } | Action::Skip { .. } => {}
        }
    }
    count
}

#[cfg(test)]
mod tests;
