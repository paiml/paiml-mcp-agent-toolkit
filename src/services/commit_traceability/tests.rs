//! Falsification suite for CB-2113 (goal-mode.md §7: "plant this → RED").
//!
//! Every fixture is a real git repository built with the `git` binary, so what
//! is measured is what `git log --format=%(trailers…)` reports, not a parser
//! of ours. The control comes first: without a fixture that PASSES, every red
//! below could be an engine that always fails.

use super::{inputs, measure_against, Inputs, Range, Violation, ROADMAP_PATH};
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

const ROADMAP: &str = r#"roadmap_version: "1.0"
github_enabled: false
github_repo: null
roadmap:
  - id: PMAT-001
    title: planned work
    status: planned
  - id: PMAT-002
    title: finished work
    status: completed
  - id: PMAT-003
    title: work under way
    status: inprogress
  - id: PMAT-004
    title: abandoned work
    status: cancelled
"#;

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args([
            "-c",
            "user.name=cb2113",
            "-c",
            "user.email=cb2113@example.invalid",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .env("LC_ALL", "C")
        .output()
        .expect("git runs");
    assert!(
        out.status.success(),
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// A repository on `master` with the roadmap committed and tagged `v0.1.0`.
fn repo() -> TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    git(dir.path(), &["init", "-q", "-b", "master"]);
    let rm = dir.path().join(ROADMAP_PATH);
    std::fs::create_dir_all(rm.parent().expect("parent")).expect("mkdir");
    std::fs::write(&rm, ROADMAP).expect("write roadmap");
    git(dir.path(), &["add", "."]);
    git(dir.path(), &["commit", "-q", "-m", "roadmap"]);
    git(dir.path(), &["tag", "v0.1.0"]);
    dir
}

/// An empty commit with `subject`, plus an optional second paragraph (where a
/// trailer lives).
fn commit(dir: &Path, subject: &str, body: Option<&str>) -> String {
    let mut args = vec!["commit", "-q", "--allow-empty", "-m", subject];
    if let Some(b) = body {
        args.extend(["-m", b]);
    }
    git(dir, &args);
    git(dir, &["rev-parse", "HEAD"])
}

fn on_feature(dir: &Path) {
    git(dir, &["switch", "-q", "-c", "feature"]);
}

// ── control ─────────────────────────────────────────────────────────────────

#[test]
fn control_a_trailered_feature_commit_measures_clean() {
    let dir = repo();
    on_feature(dir.path());
    commit(dir.path(), "feat: x", Some("Pmat-Ticket: PMAT-001"));
    let m = measure_against(dir.path(), None).expect("measured");
    assert!(
        matches!(&m.range, Range::PullRequest { base, .. } if base == "master"),
        "{:?}",
        m.range
    );
    assert_eq!((m.commits, m.trailered), (1, 1), "{m:?}");
    assert!(m.findings.is_empty(), "{:?}", m.findings);
    assert!(m.clean());
}

// ── the §7 falsifier: a commit with no trailer ──────────────────────────────

#[test]
fn a_commit_with_no_trailer_is_a_finding_naming_the_commit() {
    let dir = repo();
    on_feature(dir.path());
    let bad = commit(dir.path(), "no trailer here", None);
    let m = measure_against(dir.path(), None).expect("measured");
    assert_eq!((m.commits, m.trailered), (1, 0), "{m:?}");
    assert_eq!(m.findings.len(), 1, "{:?}", m.findings);
    assert_eq!(m.findings[0].hash, bad);
    assert_eq!(m.findings[0].subject, "no trailer here");
    assert_eq!(m.findings[0].violation, Violation::NoTrailer);
    assert!(!m.clean());
}

#[test]
fn a_ticket_id_in_the_subject_is_not_a_trailer() {
    // The commit-msg hook accepts `PMAT-001` anywhere in the message as a
    // fallback; this rule does not. The spec says TRAILER, and a subject
    // mention is prose that `git interpret-trailers` cannot see.
    let dir = repo();
    on_feature(dir.path());
    commit(dir.path(), "fix(PMAT-001): mentioned, not trailed", None);
    let m = measure_against(dir.path(), None).expect("measured");
    assert_eq!(m.trailered, 0, "{m:?}");
    assert_eq!(m.findings[0].violation, Violation::NoTrailer);
}

// ── leg 2 (T6): the id must be real and non-terminal ────────────────────────

#[test]
fn a_trailer_naming_an_id_absent_from_the_roadmap_is_a_finding() {
    let dir = repo();
    on_feature(dir.path());
    commit(dir.path(), "feat: y", Some("Pmat-Ticket: PMAT-999"));
    let m = measure_against(dir.path(), None).expect("measured");
    assert_eq!((m.commits, m.trailered), (1, 1), "{m:?}");
    assert_eq!(
        m.findings.iter().map(|f| &f.violation).collect::<Vec<_>>(),
        vec![&Violation::UnknownTicket("PMAT-999".into())]
    );
}

#[test]
fn a_trailer_naming_a_completed_item_is_a_finding() {
    let dir = repo();
    on_feature(dir.path());
    commit(dir.path(), "feat: z", Some("Pmat-Ticket: PMAT-002"));
    let m = measure_against(dir.path(), None).expect("measured");
    assert_eq!(
        m.findings.iter().map(|f| &f.violation).collect::<Vec<_>>(),
        vec![&Violation::TerminalTicket {
            id: "PMAT-002".into(),
            status: "completed".into()
        }]
    );
}

#[test]
fn a_trailer_naming_a_cancelled_item_is_a_finding() {
    let dir = repo();
    on_feature(dir.path());
    commit(dir.path(), "feat: w", Some("Pmat-Ticket: PMAT-004"));
    let m = measure_against(dir.path(), None).expect("measured");
    assert_eq!(m.findings.len(), 1, "{:?}", m.findings);
    assert!(matches!(
        &m.findings[0].violation,
        Violation::TerminalTicket { id, status } if id == "PMAT-004" && status == "cancelled"
    ));
}

#[test]
fn an_inprogress_item_is_not_terminal() {
    let dir = repo();
    on_feature(dir.path());
    commit(dir.path(), "feat: v", Some("Pmat-Ticket: PMAT-003"));
    let m = measure_against(dir.path(), None).expect("measured");
    assert!(m.clean(), "{:?}", m.findings);
}

#[test]
fn every_bad_trailer_on_one_commit_is_its_own_finding() {
    let dir = repo();
    on_feature(dir.path());
    commit(
        dir.path(),
        "feat: two trailers",
        Some("Pmat-Ticket: PMAT-999\nPmat-Ticket: PMAT-002"),
    );
    let m = measure_against(dir.path(), None).expect("measured");
    assert_eq!(m.trailered, 1);
    assert_eq!(m.findings.len(), 2, "{:?}", m.findings);
}

// ── range: merges excluded, base chosen, default branch counted not judged ──

#[test]
fn merge_commits_are_excluded_from_the_range() {
    let dir = repo();
    on_feature(dir.path());
    commit(dir.path(), "feat: a", Some("Pmat-Ticket: PMAT-001"));
    git(dir.path(), &["switch", "-q", "-c", "topic"]);
    commit(dir.path(), "feat: b", Some("Pmat-Ticket: PMAT-003"));
    git(dir.path(), &["switch", "-q", "feature"]);
    // The merge commit itself carries no trailer. It must not be judged.
    git(
        dir.path(),
        &["merge", "-q", "--no-ff", "-m", "merge topic", "topic"],
    );
    let m = measure_against(dir.path(), None).expect("measured");
    assert_eq!((m.commits, m.trailered), (2, 2), "{m:?}");
    assert!(m.clean(), "{:?}", m.findings);
}

#[test]
fn only_the_commits_the_pr_adds_are_judged() {
    // An untrailered commit on master, BEFORE the branch point, is master's
    // business (§8.4), not this branch's.
    let dir = repo();
    commit(dir.path(), "master drift, no trailer", None);
    on_feature(dir.path());
    commit(dir.path(), "feat: mine", Some("Pmat-Ticket: PMAT-001"));
    let m = measure_against(dir.path(), None).expect("measured");
    assert_eq!(m.commits, 1, "{m:?}");
    assert!(m.clean(), "{:?}", m.findings);
}

#[test]
fn an_explicit_base_wins_over_discovery() {
    let dir = repo();
    git(dir.path(), &["switch", "-q", "-c", "release"]);
    commit(dir.path(), "release prep, no trailer", None);
    on_feature(dir.path());
    commit(
        dir.path(),
        "feat: on top of release",
        Some("Pmat-Ticket: PMAT-001"),
    );
    // Against master the untrailered release commit is in range and red…
    let vs_master = measure_against(dir.path(), None).expect("measured");
    assert_eq!(vs_master.commits, 2, "{vs_master:?}");
    assert!(!vs_master.clean());
    // …against the named base only the PR's own commit is judged.
    let vs_release = measure_against(dir.path(), Some("release")).expect("measured");
    assert!(
        matches!(&vs_release.range, Range::PullRequest { base, .. } if base == "release"),
        "{:?}",
        vs_release.range
    );
    assert_eq!(vs_release.commits, 1, "{vs_release:?}");
    assert!(vs_release.clean());
}

#[test]
fn a_remote_default_branch_is_preferred_over_a_local_one() {
    // On a CI checkout `origin/master` is the truth and a local `master` may
    // not exist at all; locally both exist and the remote one is what the PR
    // is measured against.
    let dir = repo();
    git(
        dir.path(),
        &["update-ref", "refs/remotes/origin/master", "HEAD"],
    );
    git(
        dir.path(),
        &[
            "symbolic-ref",
            "refs/remotes/origin/HEAD",
            "refs/remotes/origin/master",
        ],
    );
    on_feature(dir.path());
    commit(dir.path(), "feat: q", Some("Pmat-Ticket: PMAT-001"));
    let m = measure_against(dir.path(), None).expect("measured");
    assert!(
        matches!(&m.range, Range::PullRequest { base, .. } if base == "origin/master"),
        "{:?}",
        m.range
    );
}

#[test]
fn the_default_branch_is_counted_not_judged() {
    let dir = repo();
    commit(dir.path(), "on master, no trailer", None);
    commit(
        dir.path(),
        "on master, trailed",
        Some("Pmat-Ticket: PMAT-001"),
    );
    let m = measure_against(dir.path(), None).expect("measured");
    assert_eq!(
        m.range,
        Range::DefaultBranch {
            since: Some("v0.1.0".into())
        }
    );
    assert_eq!((m.commits, m.trailered), (2, 1), "{m:?}");
    assert!(
        m.findings.is_empty(),
        "default-branch commits are not judged: {:?}",
        m.findings
    );
    assert!(
        !m.clean(),
        "counted is not judged; a default-branch measurement is never `clean`"
    );
}

#[test]
fn a_default_branch_with_no_tag_counts_from_the_root() {
    let dir = tempfile::tempdir().expect("tempdir");
    git(dir.path(), &["init", "-q", "-b", "master"]);
    let rm = dir.path().join(ROADMAP_PATH);
    std::fs::create_dir_all(rm.parent().expect("parent")).expect("mkdir");
    std::fs::write(&rm, ROADMAP).expect("write");
    git(dir.path(), &["add", "."]);
    git(dir.path(), &["commit", "-q", "-m", "roadmap"]);
    let m = measure_against(dir.path(), None).expect("measured");
    assert_eq!(m.range, Range::DefaultBranch { since: None });
    assert_eq!(m.commits, 1, "{m:?}");
}

// ── not measured is a failure, not a pass ───────────────────────────────────

#[test]
fn no_resolvable_base_is_not_measured() {
    let dir = tempfile::tempdir().expect("tempdir");
    git(dir.path(), &["init", "-q", "-b", "feature"]);
    let rm = dir.path().join(ROADMAP_PATH);
    std::fs::create_dir_all(rm.parent().expect("parent")).expect("mkdir");
    std::fs::write(&rm, ROADMAP).expect("write");
    git(dir.path(), &["add", "."]);
    git(dir.path(), &["commit", "-q", "-m", "only branch"]);
    let err = measure_against(dir.path(), None).expect_err("no base to measure against");
    assert!(err.contains("base"), "{err}");
}

#[test]
fn an_unparsable_roadmap_is_not_measured() {
    let dir = repo();
    on_feature(dir.path());
    std::fs::write(dir.path().join(ROADMAP_PATH), "roadmap: [not: {valid").expect("write");
    commit(dir.path(), "feat: r", Some("Pmat-Ticket: PMAT-001"));
    let err = measure_against(dir.path(), None).expect_err("garbage roadmap");
    assert!(err.contains("roadmap"), "{err}");
}

#[test]
fn a_nonexistent_explicit_base_is_not_measured() {
    let dir = repo();
    on_feature(dir.path());
    commit(dir.path(), "feat: s", Some("Pmat-Ticket: PMAT-001"));
    let err = measure_against(dir.path(), Some("no-such-branch")).expect_err("bad base");
    assert!(err.contains("no-such-branch"), "{err}");
}

// ── inputs ──────────────────────────────────────────────────────────────────

#[test]
fn inputs_distinguish_no_git_from_no_roadmap_from_ready() {
    let plain = tempfile::tempdir().expect("tempdir");
    assert_eq!(inputs(plain.path()), Inputs::NotGit);
    let bare = tempfile::tempdir().expect("tempdir");
    git(bare.path(), &["init", "-q", "-b", "master"]);
    assert_eq!(inputs(bare.path()), Inputs::NoRoadmap);
    assert_eq!(inputs(repo().path()), Inputs::Ready);
}

// ── registration: the rule exists under its id, at error severity ───────────

#[test]
fn cb_2113_is_registered_at_error_severity_with_its_title() {
    use crate::models::comply_config::{CheckSeverity, ComplyConfig};
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let ids =
        crate::cli::handlers::comply_handlers::check_evidence_gates::enumerate_comply_rule_ids(
            root,
        )
        .expect("registry enumerable");
    assert!(
        ids.contains("cb-2113"),
        "cb-2113 is not registered by any check builder"
    );
    let rule = crate::services::gate_effect::roster::collect(root)
        .into_iter()
        .find(|r| r.id == "CB-2113")
        .expect("CB-2113 in the roster");
    assert_eq!(rule.title, "Commit Traceability", "{rule:?}");
    assert!(rule.has_citation(), "{rule:?}");
    let cfg = ComplyConfig::default();
    let declared = cfg
        .checks
        .get("cb-2113")
        .expect("cb-2113 severity declared by default");
    assert!(declared.enabled);
    assert_eq!(declared.severity, CheckSeverity::Error);
}

#[test]
fn a_roadmap_that_was_committed_and_is_now_gone_is_not_an_absence() {
    // Quorum lanes 1-3: `NoRoadmap` → Skip let `git rm docs/roadmaps/roadmap.yaml`
    // pass the gate. CB-2102 draws the same line for its own input: never
    // committed is a structural absence; committed-then-deleted is a deleted
    // gate input, and deleting a gate's input is not a way of passing it.
    let dir = repo();
    on_feature(dir.path());
    git(dir.path(), &["rm", "-q", ROADMAP_PATH]);
    git(
        dir.path(),
        &[
            "commit",
            "-q",
            "-m",
            "drop the roadmap",
            "-m",
            "Pmat-Ticket: PMAT-001",
        ],
    );
    assert_eq!(inputs(dir.path()), Inputs::RoadmapDeleted);
    // Deleted only in the working tree, still in history: the same verdict.
    let dir2 = repo();
    std::fs::remove_file(dir2.path().join(ROADMAP_PATH)).expect("rm");
    assert_eq!(inputs(dir2.path()), Inputs::RoadmapDeleted);
}
