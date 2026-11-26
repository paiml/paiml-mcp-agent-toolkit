//! Ticket findings (FLOW-OWNER-001 §4): pure checks over the open tickets of one
//! repository. Nothing here reads or writes GitHub; the loop that acts on a finding
//! lives elsewhere.

use super::rubric::{self, Ticket};
use crate::services::github_comply::{self, find_fitting_issue, IssueRec, RepoSnapshot, Rule};
use chrono::{DateTime, Duration, Utc};
use regex::Regex;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::LazyLock;

/// The report's schema id.
pub const SCHEMA: &str = "pmat-work-findings/v1";
/// Every finding id, in the order of the spec's table.
pub const IDS: [&str; 9] = ["Q1", "V1", "V2", "V3", "V4", "P1", "P2", "P3", "P4"];

/// The state of a pull request linked to a ticket.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrState {
    Open,
    Merged,
    Closed,
}

/// One open ticket, with everything the findings read.
#[derive(Debug, Clone, PartialEq)]
pub struct TicketRec {
    pub rec: IssueRec,
    pub body: String,
    pub updated_at: DateTime<Utc>,
    /// `Some(true)` when the parent is open; `None` when there is no parent.
    pub parent_open: Option<bool>,
    /// The pull requests whose closing keyword names this ticket.
    pub linked_prs: Vec<(u64, PrState)>,
}

impl TicketRec {
    /// The rubric's view of the ticket.
    pub fn ticket(&self) -> Ticket {
        Ticket {
            title: self.rec.title.clone(),
            body: self.body.clone(),
            labels: self.rec.labels.clone(),
            parent_open: self.parent_open == Some(true),
            has_milestone: self.rec.milestone.is_some(),
        }
    }

    /// An epic or master: labelled so, or it has sub-issues.
    pub fn is_container(&self) -> bool {
        self.rec.sub_issues > 0
            || self
                .rec
                .labels
                .iter()
                .any(|l| l.eq_ignore_ascii_case("epic") || l.eq_ignore_ascii_case("master"))
    }
}

/// The most days `--stale-ticket-days` takes: a century, far inside what a duration holds.
pub const MAX_STALE_DAYS: i64 = 36_500;

/// A `--duplicate-threshold`: a number from 0 to 1. Outside it (or NaN) V2 would
/// silently report nothing or everything.
pub fn parse_similarity(text: &str) -> Result<f64, String> {
    text.parse::<f64>()
        .ok()
        .filter(|v| (0.0..=1.0).contains(v))
        .ok_or_else(|| "expected a number from 0.0 to 1.0".to_string())
}

/// The policy numbers the findings use.
#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    pub pass_mark: u32,
    pub stale_ticket_days: i64,
    pub duplicate_threshold: f64,
    pub ticket_cap: usize,
    pub max_open_children: u64,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            pass_mark: rubric::PASS_MARK,
            stale_ticket_days: 30,
            duplicate_threshold: 0.8,
            ticket_cap: 50,
            max_open_children: 8,
        }
    }
}

/// What the loop does with a finding. A close is only ever proposed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Action {
    LabelComment,
    Label,
    Link,
    CloseProposal,
    None,
}

/// One finding about one ticket (or, for P4, about the repository).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Finding {
    pub id: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue: Option<u64>,
    pub action: Action,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<&'static str>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<u32>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub missing: Vec<String>,
    /// V2: the older ticket that survives. P1: the parent to link under.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

fn finding(id: &'static str, issue: u64, action: Action, message: String) -> Finding {
    Finding {
        id,
        issue: Some(issue),
        action,
        label: None,
        message,
        score: None,
        missing: Vec::new(),
        target: None,
        path: None,
    }
}

/// The verdict of one `pmat work findings` run.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FindingsReport {
    pub schema: &'static str,
    pub repo: String,
    pub taken_at: DateTime<Utc>,
    pub open_issues: usize,
    pub pass_mark: u32,
    pub counts: BTreeMap<&'static str, usize>,
    pub findings: Vec<Finding>,
    /// What the run could not judge. Never read as "no finding".
    pub not_measured: Vec<String>,
}

impl FindingsReport {
    /// The human report: the counts, then one line per finding.
    pub fn render_text(&self) -> String {
        let mut out = format!(
            "ticket findings — {} ({} open, pass mark {})\n",
            self.repo, self.open_issues, self.pass_mark
        );
        for (id, n) in &self.counts {
            out.push_str(&format!("  {id}: {n}\n"));
        }
        for f in &self.findings {
            let at = f.issue.map(|n| format!(" #{n}")).unwrap_or_default();
            out.push_str(&format!("  {}{at}: {}\n", f.id, f.message));
        }
        for m in &self.not_measured {
            out.push_str(&format!("  NOT MEASURED: {m}\n"));
        }
        out
    }
}

/// What was read about the repository, and when.
pub struct Evidence<'a> {
    pub repo: &'a str,
    pub now: DateTime<Utc>,
    /// GitHub's count of open issues (authoritative for P4).
    pub open_total: usize,
    pub tickets: &'a [TicketRec],
    /// Files a ticket names that were on the default branch and are no longer.
    pub gone_paths: &'a BTreeSet<String>,
    pub not_measured: Vec<String>,
}

/// Judge every finding over the evidence.
pub fn evaluate(ev: Evidence<'_>, s: &Settings) -> FindingsReport {
    let mut sorted: Vec<&TicketRec> = ev.tickets.iter().collect();
    sorted.sort_by_key(|t| t.rec.number);
    let recs: Vec<IssueRec> = sorted.iter().map(|t| t.rec.clone()).collect();
    let mut findings = Vec::new();
    for (k, t) in sorted.iter().enumerate() {
        findings.extend(q1(t, s));
        findings.extend(v1(t));
        findings.extend(v2(t, &recs[..k], s));
        findings.extend(v3(t, ev.gone_paths));
        findings.extend(v4(t, ev.now, s));
        findings.extend(p1(t, &sorted));
        findings.extend(p3(t, s));
    }
    let (p2s, mut not_measured) = p2(ev.repo, ev.now, &recs);
    findings.extend(p2s);
    findings.extend(p4(ev.open_total.max(ev.tickets.len()), s));
    findings.sort_by_key(|f| (order_of(f.id), f.issue));
    not_measured.extend(ev.not_measured);
    let mut counts: BTreeMap<&'static str, usize> = IDS.iter().map(|id| (*id, 0)).collect();
    for f in &findings {
        *counts.entry(f.id).or_default() += 1;
    }
    FindingsReport {
        schema: SCHEMA,
        repo: ev.repo.to_string(),
        taken_at: ev.now,
        open_issues: ev.open_total.max(ev.tickets.len()),
        pass_mark: s.pass_mark,
        counts,
        findings,
        not_measured,
    }
}

fn order_of(id: &str) -> usize {
    IDS.iter().position(|i| *i == id).unwrap_or(IDS.len())
}

/// Q1: the ticket scores under the pass mark.
fn q1(t: &TicketRec, s: &Settings) -> Option<Finding> {
    let score = rubric::score(&t.ticket(), s.pass_mark);
    if score.passed {
        return None;
    }
    let mut f = finding(
        "Q1",
        t.rec.number,
        Action::LabelComment,
        format!(
            "scores {}/100, under the pass mark {}",
            score.total, s.pass_mark
        ),
    );
    f.label = Some("needs-rewrite");
    f.score = Some(score.total);
    f.missing = score.missing;
    Some(f)
}

/// V1: every linked pull request is merged and the ticket is still open.
fn v1(t: &TicketRec) -> Option<Finding> {
    let all_merged = t.linked_prs.iter().all(|(_, st)| *st == PrState::Merged);
    if t.linked_prs.is_empty() || !all_merged {
        return None;
    }
    let prs: Vec<String> = t.linked_prs.iter().map(|(n, _)| format!("#{n}")).collect();
    Some(finding(
        "V1",
        t.rec.number,
        Action::CloseProposal,
        format!(
            "every linked PR is merged ({}) and the ticket is open",
            prs.join(" ")
        ),
    ))
}

/// V2: the title duplicates an older open ticket (R5's matcher, policy threshold).
fn v2(t: &TicketRec, older: &[IssueRec], s: &Settings) -> Option<Finding> {
    let (fit, sim) = find_fitting_issue(&t.rec.title, older, None, s.duplicate_threshold)?;
    let mut f = finding(
        "V2",
        t.rec.number,
        Action::CloseProposal,
        format!(
            "duplicates older open ticket #{} \"{}\" (title similarity {sim:.2}); survivor #{}",
            fit.number, fit.title, fit.number
        ),
    );
    f.target = Some(fit.number);
    Some(f)
}

/// V3: one finding per file the body names that is gone from the default branch.
fn v3(t: &TicketRec, gone: &BTreeSet<String>) -> Vec<Finding> {
    rubric::named_files(&t.body)
        .into_iter()
        .filter(|p| gone.contains(p))
        .map(|p| {
            let mut f = finding(
                "V3",
                t.rec.number,
                Action::LabelComment,
                format!("names `{p}`, which is no longer on the default branch"),
            );
            f.label = Some("needs-revalidate");
            f.path = Some(p);
            f
        })
        .collect()
}

/// V4: no activity for `stale_ticket_days` and no open linked pull request.
fn v4(t: &TicketRec, now: DateTime<Utc>, s: &Settings) -> Option<Finding> {
    let idle = now - t.updated_at;
    let open_pr = t.linked_prs.iter().any(|(_, st)| *st == PrState::Open);
    if open_pr || idle <= Duration::days(s.stale_ticket_days) {
        return None;
    }
    let mut f = finding(
        "V4",
        t.rec.number,
        Action::Label,
        format!(
            "no activity for {} days (limit {}) and no open linked PR",
            idle.num_days(),
            s.stale_ticket_days
        ),
    );
    f.label = Some("needs-revalidate");
    Some(f)
}

static PARENT_LINE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?mi)^[\s>*_`-]*parent[*_`]*:[*_`\s]*#([0-9]+)\b")
        .expect("PARENT_LINE is a valid regex")
});

/// The `Parent: #N` the body states, if any.
pub fn stated_parent(body: &str) -> Option<u64> {
    PARENT_LINE
        .captures(body)
        .and_then(|c| c[1].parse::<u64>().ok())
}

/// P1: an open ticket with no parent. An epic or master is its own top and is exempt.
fn p1(t: &TicketRec, all: &[&TicketRec]) -> Option<Finding> {
    if t.rec.parent.is_some() || t.is_container() {
        return None;
    }
    let home = stated_parent(&t.body).filter(|n| {
        *n != t.rec.number && all.iter().any(|o| o.rec.number == *n && o.is_container())
    });
    Some(match home {
        Some(n) => {
            let mut f = finding(
                "P1",
                t.rec.number,
                Action::Link,
                format!("has no parent; its body names open epic or master #{n}"),
            );
            f.target = Some(n);
            f
        }
        None => {
            let mut f = finding(
                "P1",
                t.rec.number,
                Action::Label,
                "has no parent and names no open epic or master in a `Parent: #N` line".into(),
            );
            f.label = Some("needs-parent");
            f
        }
    })
}

/// P2: an epic or master whose children are all closed (GHC-001 R6b, reused). The
/// second value is what R6b could not judge.
fn p2(repo: &str, now: DateTime<Utc>, recs: &[IssueRec]) -> (Vec<Finding>, Vec<String>) {
    let snap = RepoSnapshot {
        repo: repo.to_string(),
        taken_at: now,
        open_issue_total: recs.len(),
        open_issues: recs.to_vec(),
        open_prs: Vec::new(),
    };
    let report = github_comply::evaluate(&snap, &github_comply::Policy::default(), &[Rule::R6b]);
    let found = report
        .violations
        .into_iter()
        .filter_map(|v| Some(finding("P2", v.subject?, Action::CloseProposal, v.message)))
        .collect();
    (found, report.not_measured)
}

/// P3: a master with more open children than one train carries. A finding only.
fn p3(t: &TicketRec, s: &Settings) -> Option<Finding> {
    let open = t.rec.sub_issues.saturating_sub(t.rec.sub_issues_closed?);
    (open > s.max_open_children).then(|| {
        finding(
            "P3",
            t.rec.number,
            Action::None,
            format!(
                "has {open} open children (more than {})",
                s.max_open_children
            ),
        )
    })
}

/// P4: the repository's open count is over its cap. A finding only in v1.
fn p4(open: usize, s: &Settings) -> Option<Finding> {
    (open > s.ticket_cap).then(|| Finding {
        issue: None,
        ..finding(
            "P4",
            0,
            Action::None,
            format!("{open} open tickets, over the cap of {}", s.ticket_cap),
        )
    })
}
