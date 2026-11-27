//! GH-1502: `pmat work start` and `pmat work delegate` refuse new work at the open-PR cap,
//! drafts included. The cap is `Policy::max_open_prs` (one place). A failed or rate-limited
//! read is `not_measured` (L23): said on stderr, never a refusal.
//!
//! CAP-VALVE (paiml/infra#5489, [`super::cap_grant`]): at or over the cap, a fresh PR
//! CAP-GRANT for this repo that covers the open count admits the start.
//!
//! Two predicates share that number: R2 counts non-draft, non-exempt PRs and is violated only
//! OVER the cap; this guard counts ALL open PRs (drafts, exempt) and refuses AT the cap.

use super::cap_grant::{self, Grant, GRANT_FILE};
use super::source::{current_repo, fetch_snapshot, GhGraphql, Graphql};
use super::{Policy, PrRec};
use anyhow::{anyhow, bail, Result};
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

/// [`judge_pr_cap`] with the CAP-VALVE (paiml/infra#5489): at or over the cap, a fresh PR
/// CAP-GRANT in `grant_file` that covers the open count admits, and the note says which one.
/// A refusal names the way to ask for a grant. Under the cap the file is not read.
fn judge_with_valve(
    repo: &str,
    prs: &[PrRec],
    cap: usize,
    grant_file: &Path,
    now: i64,
) -> (Result<()>, Option<String>) {
    let Err(refusal) = judge_pr_cap(repo, prs, cap) else {
        return (Ok(()), None);
    };
    let open = prs.len();
    let grant = cap_grant::read(grant_file, repo, open, now);
    if let Grant::Covers(line) = &grant {
        let note = format!("CAP-GRANT honoured: {repo} at {open} open PRs (cap {cap}): {line}");
        return (Ok(()), Some(note));
    }
    match cap_grant::escalation(grant_file, &grant, repo, open, cap) {
        Some(way_on) => (Err(anyhow!("{refusal}. {way_on}")), None),
        None => (Err(refusal), None),
    }
}

/// Judge `repo` against `cap`. The note is said on stderr: `not_measured` (verdict then
/// `Ok`) or the grant that admitted.
fn measure_pr_cap(
    g: &dyn Graphql,
    repo: &str,
    cap: usize,
    grant_file: &Path,
    now: i64,
) -> (Result<()>, Option<String>) {
    match fetch_snapshot(g, repo, Utc::now()) {
        Ok(snap) => judge_with_valve(repo, &snap.open_prs, cap, grant_file, now),
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
    let now = Utc::now().timestamp();
    guard_pr_cap_valve(g, repo, policy, Path::new(GRANT_FILE), now)
}

/// [`guard_pr_cap`] with the CAP-VALVE grant file and the clock (Unix time) as arguments.
pub fn guard_pr_cap_valve(
    g: &dyn Graphql,
    repo: &str,
    policy: &Policy,
    grant_file: &Path,
    now: i64,
) -> Result<()> {
    let (verdict, note) = measure_pr_cap(g, repo, policy.max_open_prs, grant_file, now);
    if let Some(note) = note {
        eprintln!("{note}");
    }
    verdict
}

/// The entry-point guard for `work start` and `work delegate`, run before anything is written.
/// `work add` is not guarded here (it has its own R1/R3/R5 filing guard).
pub fn guard_pr_cap_at(project_path: &Path) -> Result<()> {
    #[cfg(test)]
    if let Some((repo, g)) = tests::injected() {
        return guard_with(project_path, Ok(repo), &*g, &tests::grant_file());
    }
    let repo = current_repo(project_path);
    guard_with(project_path, repo, &GhGraphql, Path::new(GRANT_FILE))
}

/// Seam under `guard_pr_cap_at`: the repo name, the reader and the grant file are arguments.
fn guard_with(
    project_path: &Path,
    repo: Result<String>,
    g: &dyn Graphql,
    grant_file: &Path,
) -> Result<()> {
    let repo = match repo {
        Ok(r) => r,
        Err(e) => {
            eprintln!("not_measured: repository not named ({e:#}); open-PR cap not checked");
            return Ok(());
        }
    };
    let policy = Policy::load(project_path, &repo)?;
    let now = Utc::now().timestamp();
    guard_pr_cap_valve(g, &repo, &policy, grant_file, now)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::github_comply::{evaluate, CheckState, RepoSnapshot, Rule};
    use serde_json::{json, Value};

    use std::{cell::RefCell, rc::Rc};

    thread_local! {
        static INJECTED: RefCell<Option<(String, Rc<dyn Graphql>)>> = const { RefCell::new(None) };
    }

    thread_local! {
        static GRANTS: RefCell<Option<std::path::PathBuf>> = const { RefCell::new(None) };
    }

    /// The grant file of a test: never the host's real one.
    pub(super) fn grant_file() -> std::path::PathBuf {
        GRANTS
            .with(|c| c.borrow().clone())
            .unwrap_or_else(|| "/nonexistent-valve-dir/grants".into())
    }

    pub(super) fn injected() -> Option<(String, Rc<dyn Graphql>)> {
        INJECTED.with(|c| c.borrow().clone())
    }

    fn inject(g: impl Graphql + 'static) {
        INJECTED.with(|c| *c.borrow_mut() = Some(("o/r".to_string(), Rc::new(g))));
    }

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
        let (verdict, note) = measure_pr_cap(
            &Down,
            "o/r",
            10,
            Path::new("/nonexistent-valve-dir/grants"),
            T0,
        );
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

    #[test]
    fn guard_unnamed_repo_is_not_measured_and_does_not_refuse() -> Result<()> {
        // Pins current behaviour: an unnamed repo is "not checked" (stderr note, Ok), the
        // same fail-open as a failed read (L23); it is not a refusal.
        let dir = tempfile::tempdir()?;
        let r = guard_with(
            dir.path(),
            Err(anyhow::anyhow!("no remote")),
            &Fake(prs(99, 0)),
            &grant_file(),
        );
        assert!(r.is_ok());
        Ok(())
    }

    async fn start(dir: &Path) -> Result<()> {
        crate::cli::handlers::work_handlers::handle_work_start(
            "GH-1".into(),
            false,
            false,
            Some(dir.to_path_buf()),
            false,
            None,
            vec![],
            0,
            vec![],
            Default::default(),
            None,
        )
        .await
    }

    async fn delegate(dir: &Path) -> Result<()> {
        crate::cli::handlers::work_handlers::handle_work_delegate(
            "GH-1".into(),
            false,
            Some(dir.to_path_buf()),
        )
        .await
    }

    #[tokio::test]
    async fn work_start_and_delegate_refuse_at_cap_and_write_nothing() -> Result<()> {
        let dir = tempfile::tempdir()?;
        inject(Fake(prs(Policy::default().max_open_prs as u64, 0)));
        for r in [start(dir.path()).await, delegate(dir.path()).await] {
            let msg = r.err().map(|e| e.to_string()).unwrap_or_default();
            assert!(msg.contains("refused") && msg.contains("open PRs"), "{msg}");
        }
        assert_eq!(std::fs::read_dir(dir.path())?.count(), 0);
        Ok(())
    }

    #[tokio::test]
    async fn work_start_and_delegate_pass_the_guard_under_cap() -> Result<()> {
        let dir = tempfile::tempdir()?;
        inject(Fake(prs(Policy::default().max_open_prs as u64 - 1, 0)));
        for r in [start(dir.path()).await, delegate(dir.path()).await] {
            let msg = r.err().map(|e| e.to_string()).unwrap_or_default();
            assert!(!msg.contains("open PRs"), "{msg}");
        }
        Ok(())
    }

    /// A grant file in `dir` with one PR grant for `repo`, `age` seconds before `now`.
    fn grant(
        dir: &Path,
        repo: &str,
        count: usize,
        now: i64,
        age: i64,
    ) -> Result<std::path::PathBuf> {
        let ts = chrono::DateTime::<Utc>::from_timestamp(now - age, 0).unwrap_or_default();
        let file = dir.join("CAP-GRANTS.md");
        std::fs::write(
            &file,
            format!(
                "{} CAP-GRANT repo={repo} count={count}/4 cop=x-cop reason=PR: nightly fix | req: x\n",
                ts.format("%Y-%m-%dT%H:%M:%SZ")
            ),
        )?;
        Ok(file)
    }

    #[test]
    fn valve_fresh_grant_admits_at_and_over_the_cap() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let file = grant(dir.path(), "o/r", 6, T0, 60)?;
        for n in [4, 6] {
            let r = guard_pr_cap_valve(&Fake(prs(n, 1)), "o/r", &policy(4), &file, T0);
            assert!(r.is_ok(), "{n}: {r:?}");
        }
        Ok(())
    }

    #[test]
    fn valve_refusal_keeps_the_cap_text_and_names_the_escalation_line() -> Result<()> {
        let dir = tempfile::tempdir()?;
        for (repo, count, age) in [("o/other", 9, 60), ("o/r", 3, 60), ("o/r", 9, 7201)] {
            let file = grant(dir.path(), repo, count, T0, age)?;
            let msg = guard_pr_cap_valve(&Fake(prs(4, 0)), "o/r", &policy(4), &file, T0)
                .err()
                .map(|e| e.to_string())
                .unwrap_or_default();
            assert!(
                msg.contains("at its cap of 4"),
                "{repo} {count} {age}: {msg}"
            );
            assert!(msg.contains("CAP-ESCALATE o/r 4/4 PR:"), "{msg}");
        }
        Ok(())
    }

    #[test]
    fn valve_is_not_read_under_the_cap() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let file = dir.path().join("absent");
        let r = guard_pr_cap_valve(&Fake(prs(3, 0)), "o/r", &policy(4), &file, T0);
        assert!(r.is_ok(), "{r:?}");
        Ok(())
    }

    #[tokio::test]
    async fn work_start_and_delegate_honour_a_fresh_grant_at_cap() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let cap = Policy::default().max_open_prs;
        let file = grant(dir.path(), "o/r", cap, Utc::now().timestamp(), 60)?;
        GRANTS.with(|c| *c.borrow_mut() = Some(file));
        inject(Fake(prs(cap as u64, 0)));
        for r in [start(dir.path()).await, delegate(dir.path()).await] {
            let msg = r.err().map(|e| e.to_string()).unwrap_or_default();
            assert!(!msg.contains("open PRs"), "{msg}");
        }
        Ok(())
    }
}
