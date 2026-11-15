//! GH-1502: `pmat work start` and `pmat work delegate` refuse new work at the open-PR cap,
//! drafts included. The cap is `Policy::max_open_prs` (one place). A failed or rate-limited
//! read is `not_measured` (L23): said on stderr, never a refusal.
//!
//! Two predicates share that number: R2 counts non-draft, non-exempt PRs and is violated only
//! OVER the cap; this guard counts ALL open PRs (drafts, exempt) and refuses AT the cap.

use super::source::{current_repo, fetch_snapshot, GhGraphql, Graphql};
use super::{Policy, PrRec};
use anyhow::{bail, Result};
use chrono::Utc;
use std::path::Path;

/// Refuse when `prs` holds `cap` or more open PRs (drafts count): at 9 of 10 a start is
/// allowed, at 10 of 10 it is refused. The message names the count, the cap and the oldest PR.
pub fn judge_pr_cap(repo: &str, prs: &[PrRec], cap: usize) -> Result<()> {
    let open = prs.len();
    if open < cap {
        return Ok(());
    }
    let oldest = prs.iter().min_by_key(|p| p.created_at).map_or_else(
        || "none".to_string(),
        |p| format!("#{} \"{}\" (opened {})", p.number, p.title, p.created_at),
    );
    bail!(
        "refused: {repo} has {open} open PRs (drafts included), at its cap of {cap}; oldest is {oldest}. Merge or close one before starting new work"
    )
}

/// Judge `repo` against `cap`; the note is the `not_measured` text (verdict then `Ok`).
fn measure_pr_cap(g: &dyn Graphql, repo: &str, cap: usize) -> (Result<()>, Option<String>) {
    match fetch_snapshot(g, repo, Utc::now()) {
        Ok(snap) => (judge_pr_cap(repo, &snap.open_prs, cap), None),
        Err(e) => (
            Ok(()),
            Some(format!(
                "not_measured: open-PR cap of {repo} not read ({e:#}); not refusing"
            )),
        ),
    }
}

/// Read `repo` through `g` and judge it against `policy.max_open_prs`.
pub fn guard_pr_cap(g: &dyn Graphql, repo: &str, policy: &Policy) -> Result<()> {
    let (verdict, note) = measure_pr_cap(g, repo, policy.max_open_prs);
    if let Some(note) = note {
        eprintln!("{note}");
    }
    verdict
}

/// The entry-point guard for `work start` and `work delegate`, run before anything is written.
/// `work add` is not guarded here (it has its own R1/R3/R5 filing guard).
pub fn guard_pr_cap_at(project_path: &Path) -> Result<()> {
    let repo = match current_repo(project_path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("not_measured: repository not named ({e:#}); open-PR cap not checked");
            return Ok(());
        }
    };
    let policy = Policy::load(project_path, &repo)?;
    guard_pr_cap(&GhGraphql, &repo, &policy)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::github_comply::{evaluate, CheckState, RepoSnapshot, Rule};
    use serde_json::{json, Value};

    const T0: i64 = 1_793_491_200;

    fn pr(i: u64, draft: bool, hours: i64) -> PrRec {
        PrRec {
            number: 100 + i,
            title: format!("pr{i}"),
            is_draft: draft,
            created_at: chrono::DateTime::<chrono::Utc>::from_timestamp(T0, 0).unwrap_or_default()
                + chrono::Duration::hours(hours),
            head_ref: format!("b{i}"),
            labels: vec![],
            body: String::new(),
            checks: CheckState::None,
        }
    }

    fn prs(n: u64, drafts: u64) -> Vec<PrRec> {
        (0..n).map(|i| pr(i, i < drafts, i as i64)).collect()
    }

    fn node(p: &PrRec) -> Value {
        json!({"number": p.number, "title": p.title, "isDraft": p.is_draft,
               "createdAt": p.created_at.to_rfc3339(), "headRefName": p.head_ref,
               "body": "", "labels": {"nodes": []}, "commits": {"nodes": []}})
    }

    /// Fake GitHub: rateLimit, then one page of open PRs. No network.
    struct Fake(Vec<PrRec>);
    impl Graphql for Fake {
        fn query(&self, q: &str, _: &[(&str, &str)]) -> Result<Value> {
            if q.contains("rateLimit") {
                return Ok(json!({"data": {"rateLimit": {"remaining": 4000, "resetAt": "x"}}}));
            }
            let page = |field: &str, nodes: Vec<Value>| {
                json!({"data": {"repository": {field: {"totalCount": 0,
                    "pageInfo": {"hasNextPage": false, "endCursor": null}, "nodes": nodes}}}})
            };
            if q.contains("issues(") {
                return Ok(page("issues", vec![]));
            }
            let nodes: Vec<Value> = self.0.iter().map(node).collect();
            Ok(page("pullRequests", nodes))
        }
    }

    struct Down;
    impl Graphql for Down {
        fn query(&self, _: &str, _: &[(&str, &str)]) -> Result<Value> {
            bail!("rate limited")
        }
    }

    fn policy(cap: usize) -> Policy {
        Policy {
            max_open_prs: cap,
            ..Policy::default()
        }
    }

    #[test]
    fn guard_nine_open_of_ten_may_start() {
        assert!(guard_pr_cap(&Fake(prs(9, 0)), "o/r", &policy(10)).is_ok());
    }

    #[test]
    fn guard_ten_open_of_ten_is_refused_naming_count_cap_and_oldest() {
        let msg = guard_pr_cap(&Fake(prs(10, 0)), "o/r", &policy(10))
            .unwrap_err()
            .to_string();
        assert!(msg.contains("10 open PRs"), "{msg}");
        assert!(msg.contains("cap of 10"), "{msg}");
        assert!(msg.contains("#100"), "{msg}");
        assert!(msg.contains("drafts included"), "{msg}");
    }

    #[test]
    fn guard_counts_drafts_toward_the_cap() {
        assert!(guard_pr_cap(&Fake(prs(10, 10)), "o/r", &policy(10)).is_err());
    }

    #[test]
    fn guard_names_the_oldest_by_created_at_not_list_order() {
        let mut list = prs(3, 0);
        list[0].created_at += chrono::Duration::days(5);
        list[1].created_at -= chrono::Duration::days(5);
        let msg = guard_pr_cap(&Fake(list), "o/r", &policy(3))
            .unwrap_err()
            .to_string();
        assert!(msg.contains("#101"), "{msg}");
    }

    #[test]
    fn guard_read_failure_is_not_measured_and_does_not_refuse() {
        let (verdict, note) = measure_pr_cap(&Down, "o/r", 10);
        assert!(verdict.is_ok());
        let note = note.unwrap_or_default();
        assert!(note.contains("not_measured"), "{note}");
    }

    #[test]
    fn guard_and_r2_are_different_predicates_on_one_snapshot() {
        // cap 4: 3 ready + 1 draft. R2 passes; the guard refuses.
        let open = prs(4, 1);
        let snap = RepoSnapshot {
            repo: "o/r".into(),
            taken_at: open[0].created_at,
            open_issue_total: 0,
            open_issues: vec![],
            open_prs: open.clone(),
        };
        assert!(evaluate(&snap, &policy(4), &[Rule::R2]).ok());
        let msg = guard_pr_cap(&Fake(open), "o/r", &policy(4))
            .unwrap_err()
            .to_string();
        assert!(msg.contains("4 open PRs"), "{msg}");
    }
}
