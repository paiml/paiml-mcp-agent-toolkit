//! Regression tests: TDG must not depend on the caller's working directory.
//!
//! Round-3 defect: `is_file_git_tracked` ran `git log --oneline -1 -- <path>`
//! with no `-C`, so git resolved the repository from the PROCESS CWD. Running
//! `pmat analyze tdg -p <repo>` from anywhere outside `<repo>` made git exit
//! 128, the committed file was classified as untracked, and the
//! Known-Defects-v2.1 auto-fail was silently disabled. Same file, same commit:
//! 0.0 / grade F from inside the repo vs 100.0 / grade A+ from `/tmp`.

use super::*;
use std::process::Command;

/// Returns true when a usable `git` binary is present.
fn git_available() -> bool {
    Command::new("git")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Create a git repository containing one committed file; returns its path.
fn init_repo_with_commit(dir: &Path) -> PathBuf {
    let file = dir.join("tracked.rs");
    fs::write(&file, "pub fn f() -> i32 { 1 }\n").expect("write fixture");
    let run = |args: &[&str]| {
        Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .expect("git must run");
    };
    run(&["init", "-q"]);
    run(&["config", "user.email", "t@example.com"]);
    run(&["config", "user.name", "t"]);
    run(&["add", "-A"]);
    run(&["commit", "-qm", "init", "--no-verify"]);
    file
}

#[test]
fn test_git_tracked_does_not_depend_on_process_cwd() {
    if !git_available() {
        return;
    }
    let temp = tempfile::tempdir().expect("tempdir");
    let file = init_repo_with_commit(temp.path());

    // The test process CWD is the pmat repo, i.e. a DIFFERENT repository from
    // the fixture. Pre-fix this returned false (git: "not a git repository" /
    // "outside repository"), which is exactly how the grade came to depend on
    // the caller's CWD.
    assert!(
        is_file_git_tracked(&file),
        "a committed file must read as tracked no matter which directory pmat was invoked from"
    );
}

#[test]
fn test_untracked_file_still_reads_as_untracked() {
    if !git_available() {
        return;
    }
    let temp = tempfile::tempdir().expect("tempdir");
    init_repo_with_commit(temp.path());

    // Issue #279's escape hatch must survive the fix: a brand-new file with no
    // commit is still "untracked" so it is not auto-failed.
    let fresh = temp.path().join("fresh.rs");
    fs::write(&fresh, "pub fn g() -> i32 { 2 }\n").expect("write fixture");
    assert!(
        !is_file_git_tracked(&fresh),
        "an uncommitted file must still read as untracked (issue #279)"
    );
}

#[test]
fn test_critical_defect_grade_is_cwd_independent() {
    if !git_available() {
        return;
    }
    let temp = tempfile::tempdir().expect("tempdir");
    let dir = temp.path();
    let file = dir.join("bad.rs");
    fs::write(
        &file,
        "pub fn boom(v: Vec<i32>) -> i32 {\n    let x: Option<i32> = v.first().copied();\n    x.unwrap()\n}\n",
    )
    .expect("write fixture");
    let run = |args: &[&str]| {
        Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .expect("git must run");
    };
    run(&["init", "-q"]);
    run(&["config", "user.email", "t@example.com"]);
    run(&["config", "user.name", "t"]);
    run(&["add", "-A"]);
    run(&["commit", "-qm", "init", "--no-verify"]);

    let analyzer = TdgAnalyzerAst::new().expect("analyzer");
    let source = fs::read_to_string(&file).expect("read fixture");
    let score = analyzer
        .analyze_source(&source, Language::Rust, Some(file.clone()))
        .expect("analysis");

    // The unwrap() is a critical defect in a COMMITTED file, so it must be
    // detected and left un-waived even though the test process runs from a
    // different repository. The `0.0 / F` this used to assert was the old way
    // of expressing the auto-fail; the gate is now `CriticalDefectGate` and the
    // score carries a graduated penalty, so the CWD-independent property to pin
    // here is the DETECTION, not a particular magic score.
    assert_eq!(score.critical_defects_count, 1);
    assert!(
        score.has_critical_defects,
        "committed file with a critical defect must be flagged from any CWD"
    );
    assert!(
        score.critical_defects_suppressed.is_none(),
        "a committed file is not eligible for the #279 waiver from any CWD"
    );
    assert!(
        score.total < 70.0,
        "a critical defect must still cap the score below B-: got {}",
        score.total
    );
}
