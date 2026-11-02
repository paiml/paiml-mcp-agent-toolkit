//! FLOW-012 P1 (GH-1591) — the branch is the ticket.
//!
//! Falsifier, verbatim: "A fresh worktree on work/PMAT-123 with no setup:
//! pmat work status reports PMAT-123".
//!
//! Principle 2, "state is derived, never declared": with no explicit id, the
//! active ticket is read from the current branch name (`^work/PMAT-[0-9]+$`).
//! An explicit id still wins. Off such a branch there is no guess: status keeps
//! its old behaviour, continue/complete refuse with the fix.

#[path = "support/pmat_cmd.rs"]
mod pmat_cmd;

use std::path::{Path, PathBuf};
use std::process::Command;

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .current_dir(dir)
        .args(args)
        .env("GIT_AUTHOR_NAME", "t")
        .env("GIT_AUTHOR_EMAIL", "t@example.com")
        .env("GIT_COMMITTER_NAME", "t")
        .env("GIT_COMMITTER_EMAIL", "t@example.com")
        .output()
        .expect("git runs");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// A temp repo on `main` with one empty commit, plus a fresh worktree on `branch`.
fn fixture(branch: &str) -> (tempfile::TempDir, PathBuf, PathBuf) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let main = tmp.path().join("main");
    std::fs::create_dir(&main).unwrap();
    git(&main, &["init", "-q", "-b", "main"]);
    git(&main, &["commit", "-q", "--allow-empty", "-m", "init"]);
    let wt = tmp.path().join("wt");
    git(
        &main,
        &["worktree", "add", "-q", "-b", branch, wt.to_str().unwrap()],
    );
    (tmp, main, wt)
}

fn run(dir: &Path, args: &[&str]) -> (bool, String, String) {
    let out = pmat_cmd::pmat()
        .current_dir(dir)
        .args(args)
        .output()
        .expect("spawn pmat");
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
}

/// The falsifier itself: no roadmap, no declaration, no key file.
#[test]
fn status_on_a_work_branch_reports_the_ticket_with_no_setup() {
    let (_tmp, _main, wt) = fixture("work/PMAT-123");
    let (ok, out, err) = run(&wt, &["work", "status"]);
    assert!(ok, "status failed: {err}");
    assert!(out.contains("PMAT-123"), "stdout lacked PMAT-123: {out}");
    assert!(
        out.contains("work/PMAT-123"),
        "should name its source: {out}"
    );
}

/// Explicit argument is first priority.
#[test]
fn an_explicit_id_beats_the_branch() {
    let (_tmp, _main, wt) = fixture("work/PMAT-123");
    let (_ok, out, err) = run(&wt, &["work", "status", "PMAT-999"]);
    let all = format!("{out}{err}");
    assert!(all.contains("PMAT-999"), "{all}");
    assert!(!out.contains("Active ticket: PMAT-123"), "{out}");
}

/// A branch that does not match exactly is never guessed at.
#[test]
fn a_near_miss_branch_is_not_a_ticket() {
    for b in ["work/PMAT-123-extra", "feature/PMAT-123", "work/PMAT-"] {
        let (_tmp, _main, wt) = fixture(b);
        let (_ok, out, _err) = run(&wt, &["work", "status"]);
        assert!(!out.contains("Active ticket"), "{b} was guessed: {out}");
    }
}

/// Negative: on main with no id and no file, no ticket is invented.
#[test]
fn status_on_main_reports_no_active_ticket() {
    let (_tmp, main, _wt) = fixture("work/PMAT-123");
    let (_ok, out, _err) = run(&main, &["work", "status"]);
    assert!(!out.contains("Active ticket"), "guessed a ticket: {out}");
    assert!(!out.contains("PMAT-123"), "guessed a ticket: {out}");
}

/// Negative: continue/complete off a work branch refuse with the fix, not a guess.
#[test]
fn continue_and_complete_on_main_refuse_with_the_fix() {
    let (_tmp, main, _wt) = fixture("work/PMAT-123");
    for verb in ["continue", "complete"] {
        let (ok, _out, err) = run(&main, &["work", verb]);
        assert!(!ok, "{verb} succeeded with no ticket");
        assert!(err.contains("work/PMAT-"), "{verb} gave no fix: {err}");
    }
}

/// continue with no id resolves from the branch (the item is in the roadmap, so it
/// succeeds and must name the derived ticket).
#[test]
fn continue_on_a_work_branch_uses_the_branch_ticket() {
    let (_tmp, _main, wt) = fixture("work/PMAT-123");
    write_roadmap(&wt, true);
    let (ok, out, err) = run(&wt, &["work", "continue"]);
    assert!(ok, "continue failed: {out}{err}");
    let all = format!("{out}{err}");
    assert!(all.contains("PMAT-123"), "{all}");
}

fn write_roadmap(wt: &Path, with_branch_ticket: bool) {
    let dir = wt.join("docs/roadmaps");
    std::fs::create_dir_all(&dir).unwrap();
    let mut y = String::from("roadmap_version: '1.0'\ngithub_enabled: false\nroadmap:\n");
    let mut ids = vec!["OTHER-1"];
    if with_branch_ticket {
        ids.push("PMAT-123");
    }
    for id in ids {
        y.push_str(&format!(
            "- id: {id}\n  github_issue: null\n  item_type: task\n  title: 't {id}'\n  status: planned\n  priority: medium\n  assigned_to: null\n  created: 2026-01-01T00:00:00Z\n  updated: 2026-01-01T00:00:00Z\n  spec: null\n  acceptance_criteria: []\n  phases: []\n  subtasks: []\n  estimated_effort: null\n  labels: []\n  notes: null\n"
        ));
    }
    std::fs::write(dir.join("roadmap.yaml"), y).unwrap();
}

/// B3: the branch ticket is shown IN ADDITION to the roadmap listing.
#[test]
fn status_on_a_work_branch_still_lists_the_roadmap() {
    for known in [false, true] {
        let (_tmp, _main, wt) = fixture("work/PMAT-123");
        write_roadmap(&wt, known);
        let (ok, out, err) = run(&wt, &["work", "status"]);
        assert!(ok, "status failed: {err}");
        assert!(out.contains("Active ticket: "), "no active ticket: {out}");
        assert!(
            out.contains("OTHER-1"),
            "roadmap listing lost (known={known}): {out}"
        );
    }
}

/// B4: status with an explicit id that differs from the branch prints the note.
#[test]
fn status_with_a_differing_explicit_id_prints_the_note() {
    let (_tmp, _main, wt) = fixture("work/PMAT-123");
    write_roadmap(&wt, false);
    let (_ok, _out, err) = run(&wt, &["work", "status", "OTHER-1"]);
    assert!(
        err.contains("note: explicit id OTHER-1 used; branch work/PMAT-123 names PMAT-123"),
        "no note: {err}"
    );
    let (_ok, _out, err) = run(&wt, &["work", "status", "PMAT-123"]);
    assert!(!err.contains("note: explicit id"), "spurious note: {err}");
}
