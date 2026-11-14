//! GHC-001: GitHub comply — `pmat comply github` and the `pmat work add` guards.
//!
//! Six per-repository rules (OUT-204), judged over a [`RepoSnapshot`] — the
//! open issues and open pull requests of ONE repository — so the rules are pure
//! and the same code serves the live GraphQL read, a snapshot file, and a test:
//!
//! | rule | violated when                                                          | where it runs                |
//! |------|------------------------------------------------------------------------|------------------------------|
//! | R1   | more than `max_open_issues` (100) open issues                          | reconciler, `work add`       |
//! | R2   | more than `max_open_prs` (4) open non-draft PRs (exempt train/*, fix-set/expedite) | reconciler        |
//! | R3   | an open ticket with neither a milestone nor an epic parent             | PR check, `work add`         |
//! | R4   | a non-draft PR older than `pr_max_age_hours` (4) with no PASS          | reconciler                   |
//! | R5   | `work add` while an open ticket already fits, or the repo is at the R1 cap | `work add`               |
//! | R6   | a PR body with no `Closes|Fixes|Resolves owner/repo#N` / `Part of owner/repo#N` | PR check            |
//! | R4d  | a DRAFT PR older than `draft_ttl_hours` (72) — OUT-281, reported apart from R4 | reconciler          |
//! | R6b  | an open epic whose sub-issues are all closed — OUT-281                 | reconciler                   |
//!
//! R5 is advisory in v1 (OUT-281): a title match warns and names the existing ticket but
//! does not refuse; blocking needs body similarity (v2).
//!
//! Policy lives in `.pmat/github-comply.toml` (defaults below, per-repo
//! overrides under `[repo."owner/name"]`). An unknown key is an error: a typo
//! that silently selected the default would be a gate that cannot fail.
//!
//! A read that failed is *not measured* and is never a pass — the live source
//! returns an error rather than an empty snapshot.

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Where the policy is read from, relative to the project root.
pub const POLICY_PATH: &str = ".pmat/github-comply.toml";

/// The six rules of GHC-001.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Rule {
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    /// R4 for drafts: a draft open longer than `draft_ttl_hours` (72). Reported apart from R4.
    R4d,
    /// R6b: an open epic all of whose sub-issues are closed.
    R6b,
}

impl Rule {
    /// The rules `pmat comply github` judges from a repository snapshot. R5 is
    /// a creation-time rule (`pmat work add`) and has no repository verdict.
    pub const SNAPSHOT: [Rule; 7] = [
        Rule::R1,
        Rule::R2,
        Rule::R3,
        Rule::R4,
        Rule::R4d,
        Rule::R6,
        Rule::R6b,
    ];
    /// The reconciler's set (per repo): R1, R2, R4, plus the two repo-level checks
    /// OUT-281 added, the draft TTL (R4d) and the finished-epic check (R6b).
    pub const RECONCILER: [Rule; 5] = [Rule::R1, Rule::R2, Rule::R4, Rule::R4d, Rule::R6b];

    /// The id of each variant, in declaration order (`self as usize` indexes this).
    const IDS: [&'static str; 8] = ["R1", "R2", "R3", "R4", "R5", "R6", "R4d", "R6b"];

    pub fn id(self) -> &'static str {
        Rule::IDS[self as usize]
    }

    pub fn parse(s: &str) -> Result<Rule> {
        Rule::SNAPSHOT
            .iter()
            .copied()
            .find(|r| r.id().eq_ignore_ascii_case(s.trim()))
            .with_context(|| {
                format!("unknown rule `{s}` — `pmat comply github` judges R1, R2, R3, R4, R4d, R6, R6b (R5 is `pmat work add`)")
            })
    }
}

/// Per-repo policy. Defaults are the OUT-204 numbers.
#[derive(Debug, Clone, PartialEq)]
pub struct Policy {
    /// R1 cap.
    pub max_open_issues: usize,
    /// R1 target (reported, not enforced).
    pub target_open_issues: usize,
    /// R2 cap on open non-draft PRs.
    pub max_open_prs: usize,
    /// R2 exemptions: a glob (`*` only) matched against the head branch and
    /// each label name.
    pub pr_exempt: Vec<String>,
    /// R4 max age, hours, of a non-draft PR without a PASS.
    pub pr_max_age_hours: i64,
    /// R4d: max age, hours, of a DRAFT PR (OUT-281: 72 h TTL).
    pub draft_ttl_hours: i64,
    /// R5: Jaccard similarity of title tokens at or above which an open ticket "fits".
    pub fit_similarity: f64,
}

impl Default for Policy {
    fn default() -> Self {
        Policy {
            max_open_issues: 100,
            target_open_issues: 50,
            max_open_prs: 4,
            pr_exempt: vec!["train/*".into(), "fix-set/expedite".into()],
            pr_max_age_hours: 4,
            draft_ttl_hours: 72,
            fit_similarity: 0.6,
        }
    }
}

/// One `[defaults]` / `[repo."o/r"]` table; every key optional.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct PolicyTable {
    max_open_issues: Option<usize>,
    target_open_issues: Option<usize>,
    max_open_prs: Option<usize>,
    pr_exempt: Option<Vec<String>>,
    pr_max_age_hours: Option<i64>,
    draft_ttl_hours: Option<i64>,
    fit_similarity: Option<f64>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct PolicyFile {
    #[serde(default)]
    defaults: PolicyTable,
    #[serde(default)]
    repo: std::collections::BTreeMap<String, PolicyTable>,
}

impl Policy {
    /// Parse `text`, layering `[defaults]` then `[repo."<repo>"]` over the built-ins.
    pub fn from_toml(text: &str, repo: &str) -> Result<Policy> {
        let file: PolicyFile = toml::from_str(text).context("invalid github-comply policy")?;
        let mut p = Policy::default();
        p.apply(&file.defaults);
        if let Some(t) = file.repo.get(repo) {
            p.apply(t);
        }
        p.validate()?;
        Ok(p)
    }

    /// Read `<root>/.pmat/github-comply.toml`; an absent file is the defaults.
    pub fn load(root: &Path, repo: &str) -> Result<Policy> {
        let path = root.join(POLICY_PATH);
        match std::fs::read_to_string(&path) {
            Ok(text) => {
                Policy::from_toml(&text, repo).with_context(|| format!("{}", path.display()))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Policy::default()),
            Err(e) => Err(e).with_context(|| format!("reading {}", path.display())),
        }
    }

    /// Layer `t` over `self`: a key present in `t` wins, an absent one keeps the value.
    fn apply(&mut self, t: &PolicyTable) {
        self.max_open_issues = t.max_open_issues.unwrap_or(self.max_open_issues);
        self.target_open_issues = t.target_open_issues.unwrap_or(self.target_open_issues);
        self.max_open_prs = t.max_open_prs.unwrap_or(self.max_open_prs);
        self.pr_exempt = t
            .pr_exempt
            .clone()
            .unwrap_or_else(|| self.pr_exempt.clone());
        self.pr_max_age_hours = t.pr_max_age_hours.unwrap_or(self.pr_max_age_hours);
        self.draft_ttl_hours = t.draft_ttl_hours.unwrap_or(self.draft_ttl_hours);
        self.fit_similarity = t.fit_similarity.unwrap_or(self.fit_similarity);
    }

    fn validate(&self) -> Result<()> {
        if self.pr_max_age_hours < 0 {
            bail!("pr_max_age_hours must not be negative");
        }
        if self.draft_ttl_hours < 0 {
            bail!("draft_ttl_hours must not be negative");
        }
        if !(0.0..=1.0).contains(&self.fit_similarity) {
            bail!("fit_similarity must be within 0.0..=1.0");
        }
        Ok(())
    }
}

/// The CI verdict on a PR's head commit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckState {
    /// Every required check passed.
    Pass,
    /// Checks are running or expected.
    Pending,
    /// A check failed or errored.
    Fail,
    /// No check has reported.
    None,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IssueRec {
    pub number: u64,
    pub title: String,
    #[serde(default)]
    pub milestone: Option<String>,
    /// The parent (epic) issue number, when this is a sub-issue.
    #[serde(default)]
    pub parent: Option<u64>,
    #[serde(default)]
    pub labels: Vec<String>,
    /// How many sub-issues this issue has (an epic has at least one).
    #[serde(default)]
    pub sub_issues: u64,
    /// How many of those sub-issues are closed. `None` = the read did not say (R6b is then
    /// not measured, never a pass).
    #[serde(default)]
    pub sub_issues_closed: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrRec {
    pub number: u64,
    pub title: String,
    #[serde(default)]
    pub is_draft: bool,
    pub created_at: DateTime<Utc>,
    pub head_ref: String,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub body: String,
    pub checks: CheckState,
}

/// The open issues and open PRs of one repository, at one instant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepoSnapshot {
    pub repo: String,
    pub taken_at: DateTime<Utc>,
    /// GitHub's `totalCount` — authoritative for R1 even if `open_issues` was truncated.
    pub open_issue_total: usize,
    pub open_issues: Vec<IssueRec>,
    pub open_prs: Vec<PrRec>,
}

/// One broken rule.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Violation {
    pub rule: Rule,
    /// The issue or PR number, when the rule is about one.
    pub subject: Option<u64>,
    pub message: String,
}

/// The verdict of one `pmat comply github` run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Report {
    pub repo: String,
    /// `github-graphql` or `snapshot-file` — a file is not evidence for the repo.
    pub source: String,
    pub rules_run: Vec<Rule>,
    pub violations: Vec<Violation>,
    /// Things the run could not judge. Never read as a pass.
    pub not_measured: Vec<String>,
}

pub mod backfill;
mod rules;
pub mod source;
pub use rules::*;

#[cfg(test)]
#[path = "github_comply_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "github_comply_source_tests.rs"]
mod source_tests;

#[cfg(test)]
#[path = "github_comply_backfill_tests.rs"]
mod backfill_tests;
