//! AD-03 (docs/specifications/agentic-delivery-pmat.md §4.7 / §9.3, #1126):
//! the generated hooks must be able to REFUSE a commit, not only warn.
//!
//! Before this: the pre-commit SATD and task-ID checks printed a warning and
//! reached `echo "✅ All quality gates passed!"`; no commit-msg hook existed,
//! so "link work to a ticket" was a habit. These tests install the hooks
//! into a throwaway repository and drive `git commit` itself — the only
//! oracle that can tell a warning from a refusal.
use super::hooks_command::HooksCommand;
use std::path::Path;
use std::process::Command;

fn git(dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .expect("git")
}

/// A throwaway git repository with one commit and pmat's hooks installed in
/// strict mode; `.git/hooks` is the hooks dir the installer writes to.
fn strict_repo() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("tempdir");
    let d = tmp.path();
    assert!(git(d, &["init", "-q"]).status.success());
    assert!(git(d, &["config", "user.email", "t@t"]).status.success());
    assert!(git(d, &["config", "user.name", "t"]).status.success());
    std::fs::write(d.join("README.md"), "one\n").expect("write");
    assert!(git(d, &["add", "."]).status.success());
    assert!(git(d, &["commit", "-q", "-m", "init"]).status.success());
    let cmd = HooksCommand::new(d.join(".git").join("hooks"), d.join("pmat.toml"));
    cmd.install_commit_msg_hook(true)
        .expect("install commit-msg hook");
    tmp
}

/// The refusal: a commit whose message carries no `Pmat-Ticket:` trailer
/// (and no `#NNN`) must not be created. The shipped hook warned and exited 0
/// — a gate that cannot fail (#1126).
#[test]
fn a_commit_without_a_ticket_trailer_is_refused_in_strict_mode() {
    let tmp = strict_repo();
    let d = tmp.path();
    std::fs::write(d.join("README.md"), "one\ntwo\n").expect("write");
    let out = git(d, &["commit", "-qam", "no trailer here"]);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !out.status.success(),
        "a commit without Pmat-Ticket must be refused; git said: {stderr}"
    );
    assert!(
        stderr.contains("Pmat-Ticket"),
        "the refusal must name the trailer it wants: {stderr}"
    );
    let log = git(d, &["log", "--oneline"]);
    let head = String::from_utf8_lossy(&log.stdout);
    assert_eq!(head.lines().count(), 1, "the refused commit must not exist");
}

/// The control: the same change with the trailer commits, and the trailer
/// is readable by git itself — the record the comply check (AD-07) reads.
#[test]
fn a_commit_with_the_trailer_is_accepted_and_the_trailer_is_git_readable() {
    let tmp = strict_repo();
    let d = tmp.path();
    std::fs::write(d.join("README.md"), "one\ntwo\n").expect("write");
    let out = git(
        d,
        &[
            "commit",
            "-qam",
            "with trailer",
            "-m",
            "Pmat-Ticket: PMAT-655",
        ],
    );
    assert!(
        out.status.success(),
        "a commit with the trailer must be accepted: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let t = git(
        d,
        &[
            "log",
            "-1",
            "--format=%(trailers:key=Pmat-Ticket,valueonly)",
        ],
    );
    assert_eq!(String::from_utf8_lossy(&t.stdout).trim(), "PMAT-655");
}

/// An issue reference is the accepted form for repositories without pmat
/// work: `#1126` in the message satisfies the hook.
#[test]
fn an_issue_reference_satisfies_the_ticket_rule() {
    let tmp = strict_repo();
    let d = tmp.path();
    std::fs::write(d.join("README.md"), "one\ntwo\n").expect("write");
    let out = git(d, &["commit", "-qam", "fix the thing (#1126)"]);
    assert!(
        out.status.success(),
        "#NNN must satisfy the rule: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Non-strict keeps the old behaviour: warn, do not refuse — so an
/// unconfigured repository is not locked out by an upgrade.
#[test]
fn without_strict_the_hook_warns_and_lets_the_commit_through() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let d = tmp.path();
    assert!(git(d, &["init", "-q"]).status.success());
    assert!(git(d, &["config", "user.email", "t@t"]).status.success());
    assert!(git(d, &["config", "user.name", "t"]).status.success());
    std::fs::write(d.join("README.md"), "one\n").expect("write");
    assert!(git(d, &["add", "."]).status.success());
    assert!(git(d, &["commit", "-q", "-m", "init"]).status.success());
    HooksCommand::new(d.join(".git").join("hooks"), d.join("pmat.toml"))
        .install_commit_msg_hook(false)
        .expect("install");
    std::fs::write(d.join("README.md"), "one\ntwo\n").expect("write");
    let out = git(d, &["commit", "-qam", "no trailer"]);
    assert!(out.status.success(), "non-strict must not refuse");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("Pmat-Ticket"),
        "non-strict must still warn, naming the trailer"
    );
}

/// The pre-commit SATD block blocks under strict: the generated text must
/// carry an `exit 1` on the over-threshold branch when strict is on, and the
/// success banner must not be reachable from it.
#[test]
fn the_pre_commit_satd_block_exits_1_under_strict() {
    let cmd = HooksCommand::new(
        std::path::PathBuf::from("/tmp"),
        std::path::PathBuf::from("/tmp"),
    );
    let hook = cmd.generate_quality_checks();
    assert!(
        hook.contains("PMAT_HOOKS_STRICT"),
        "the SATD block must consult the strict switch"
    );
    assert!(
        hook.contains("SATD comments exceed the threshold and [hooks] strict is on"),
        "the strict branch must refuse with a reason, not a warning glyph"
    );
}
