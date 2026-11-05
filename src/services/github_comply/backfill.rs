//! GHC-001 backfill (OUT-204 step 4): open issues that already have >= 1 merged
//! referencing PR, classified so the tri-* windows can close with merged-PR evidence.
//!
//! This only classifies. It never writes to GitHub and never closes anything.
//!
//! * DONE: a merged PR closes the issue (`Closes|Fixes|Resolves owner/repo#N`) and its
//!   title shares at least one word with the issue's title.
//! * WRONG-LINK: a merged PR closes the issue but no closing PR shares any word with the
//!   issue's title, so the keyword probably points at the wrong number.
//! * PARTIAL: the only merged references are `Part of owner/repo#N`.
//!
//! Precedence when an issue has several PRs: DONE, then WRONG-LINK, then PARTIAL.

use super::{IssueRec, RepoSnapshot};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

/// A merged pull request, as far as the backfill needs it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MergedPr {
    pub number: u64,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Verdict {
    Done,
    Partial,
    WrongLink,
}

impl Verdict {
    pub fn label(self) -> &'static str {
        match self {
            Verdict::Done => "DONE",
            Verdict::Partial => "PARTIAL",
            Verdict::WrongLink => "WRONG-LINK",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BackfillRow {
    pub issue: u64,
    pub title: String,
    pub verdict: Verdict,
    /// Every merged PR that references the issue, ascending.
    pub prs: Vec<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BackfillReport {
    pub repo: String,
    pub merged_prs_read: usize,
    pub rows: Vec<BackfillRow>,
}

impl BackfillReport {
    pub fn render_text(&self) -> String {
        let mut out = format!(
            "GHC-001 backfill  {}  ({} merged PRs read, {} issues with a merged reference)\n",
            self.repo,
            self.merged_prs_read,
            self.rows.len()
        );
        for r in &self.rows {
            let prs: Vec<String> = r.prs.iter().map(|p| format!("#{p}")).collect();
            out.push_str(&format!(
                "  {:<10} #{}  {}  <- {}\n",
                r.verdict.label(),
                r.issue,
                r.title,
                prs.join(", ")
            ));
        }
        out
    }
}

/// `(keyword, owner/repo, N)`; the keyword decides closing versus "Part of".
static KEYED_REF: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)(?:^|[^A-Za-z0-9_])(close[sd]?|fix(?:e[sd])?|resolve[sd]?|part\s+of):?\s+([A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+)#([0-9]+)",
    )
    .expect("KEYED_REF is a valid regex")
});

/// `(closing?, issue)` for every reference in `body` into `repo`.
fn refs_into(body: &str, repo: &str) -> Vec<(bool, u64)> {
    KEYED_REF
        .captures_iter(body)
        .filter(|c| c[2].eq_ignore_ascii_case(repo))
        .filter_map(|c| {
            let n = c[3].parse::<u64>().ok()?;
            Some((!c[1].to_lowercase().starts_with("part"), n))
        })
        .collect()
}

fn classify(issue: &IssueRec, hits: &[(&MergedPr, bool)]) -> Verdict {
    let closers: Vec<_> = hits.iter().filter(|(_, closing)| *closing).collect();
    if closers
        .iter()
        .any(|(p, _)| super::titles_share_a_word(&p.title, &issue.title))
    {
        Verdict::Done
    } else if !closers.is_empty() {
        Verdict::WrongLink
    } else {
        Verdict::Partial
    }
}

/// Classify every open issue of `snap` that a merged PR in `merged` references.
pub fn backfill(snap: &RepoSnapshot, merged: &[MergedPr]) -> BackfillReport {
    let refs: Vec<(&MergedPr, Vec<(bool, u64)>)> = merged
        .iter()
        .map(|p| (p, refs_into(&p.body, &snap.repo)))
        .collect();
    let mut rows = Vec::new();
    for issue in &snap.open_issues {
        let mut hits: Vec<(&MergedPr, bool)> = Vec::new();
        for (p, rs) in &refs {
            let closing = rs.iter().any(|(c, n)| *c && *n == issue.number);
            if closing || rs.iter().any(|(_, n)| *n == issue.number) {
                hits.push((p, closing));
            }
        }
        if hits.is_empty() {
            continue;
        }
        let mut prs: Vec<u64> = hits.iter().map(|(p, _)| p.number).collect();
        prs.sort_unstable();
        prs.dedup();
        rows.push(BackfillRow {
            issue: issue.number,
            title: issue.title.clone(),
            verdict: classify(issue, &hits),
            prs,
        });
    }
    BackfillReport {
        repo: snap.repo.clone(),
        merged_prs_read: merged.len(),
        rows,
    }
}
