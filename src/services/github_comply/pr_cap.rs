//! GH-1502: `pmat work start` and `pmat work delegate` refuse new work when the repo is at
//! its open-PR cap, drafts included. The cap is `Policy::max_open_prs` (one place, no second
//! list). A failed or rate-limited read is `not_measured` (L23): said on stderr, never a refusal.

use super::source::{current_repo, fetch_snapshot, GhGraphql, Graphql};
use super::{Policy, RepoSnapshot};
use anyhow::{bail, Result};
use chrono::Utc;
use std::path::Path;

/// Refuse when `snap` holds `cap` or more open PRs (drafts count): at 9 of 10 a start is
/// allowed, at 10 of 10 it is refused. The message names the count, the cap and the oldest PR.
pub fn judge_pr_cap(snap: &RepoSnapshot, cap: usize) -> Result<()> {
    let open = snap.open_prs.len();
    if open < cap {
        return Ok(());
    }
    let oldest = snap
        .open_prs
        .iter()
        .min_by_key(|p| p.created_at)
        .map_or_else(
            || "none".to_string(),
            |p| format!("#{} \"{}\" (opened {})", p.number, p.title, p.created_at),
        );
    bail!(
        "refused: {} has {open} open PRs (drafts included), at its cap of {cap}; oldest is {oldest}. Merge or close one before starting new work",
        snap.repo
    )
}

/// Read `repo` through `g` and judge it against `policy.max_open_prs`.
pub fn guard_pr_cap(g: &dyn Graphql, repo: &str, policy: &Policy) -> Result<()> {
    match fetch_snapshot(g, repo, Utc::now()) {
        Ok(snap) => judge_pr_cap(&snap, policy.max_open_prs),
        Err(e) => {
            eprintln!("not_measured: open-PR cap of {repo} not read ({e:#}); not refusing");
            Ok(())
        }
    }
}

/// The entry-point guard for `work start` and `work delegate`, run before anything is written.
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
    use crate::services::github_comply::{CheckState, PrRec};
    use serde_json::Value;

    fn snap(n: u64, drafts: u64) -> RepoSnapshot {
        let t0 = chrono::DateTime::<Utc>::from_timestamp(1_793_491_200, 0).unwrap_or_default();
        let open_prs = (0..n)
            .map(|i| PrRec {
                number: 100 + i,
                title: format!("pr{i}"),
                is_draft: i < drafts,
                created_at: t0 + chrono::Duration::hours(i as i64),
                head_ref: format!("b{i}"),
                labels: vec![],
                body: String::new(),
                checks: CheckState::None,
            })
            .collect();
        RepoSnapshot {
            repo: "o/r".into(),
            taken_at: t0,
            open_issue_total: 0,
            open_issues: vec![],
            open_prs,
        }
    }

    struct Down;
    impl Graphql for Down {
        fn query(&self, _: &str, _: &[(&str, &str)]) -> Result<Value> {
            bail!("rate limited")
        }
    }

    #[test]
    fn nine_open_of_ten_may_start() {
        assert!(judge_pr_cap(&snap(9, 0), 10).is_ok());
    }

    #[test]
    fn ten_open_of_ten_is_refused_naming_count_cap_and_oldest() {
        let msg = judge_pr_cap(&snap(10, 0), 10).unwrap_err().to_string();
        assert!(msg.contains("10 open PRs"), "{msg}");
        assert!(msg.contains("cap of 10"), "{msg}");
        assert!(msg.contains("#100"), "{msg}");
    }

    #[test]
    fn drafts_count_toward_the_cap() {
        assert!(judge_pr_cap(&snap(10, 10), 10).is_err());
    }

    #[test]
    fn unreadable_repo_is_not_measured_and_does_not_refuse() {
        assert!(guard_pr_cap(&Down, "o/r", &Policy::default()).is_ok());
    }
}
