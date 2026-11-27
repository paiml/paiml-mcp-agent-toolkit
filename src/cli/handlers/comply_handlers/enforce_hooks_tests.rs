//! CB-2113 must be caught before CI: `pmat comply enforce` installs the strict
//! commit-msg hook and a pre-push hook that runs the same judge CI runs.
//! These tests execute the installed hooks; only their exit codes are asserted.
#![cfg(unix)]
use super::{handle_enforce, ComplyOutputFormat};
use std::path::Path;
use std::process::Command;

fn run(cmd: &mut Command, dir: &Path) -> bool {
    cmd.current_dir(dir)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .output()
        .expect("spawn")
        .status
        .success()
}

/// A repository with a roadmap holding one open ticket and no template hooks.
fn roadmap_repo() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("tempdir");
    let d = tmp.path();
    assert!(run(
        Command::new("git").args(["init", "-q", "--template="]),
        d
    ));
    std::fs::create_dir_all(d.join(".git/hooks")).expect("hooks dir");
    std::fs::create_dir_all(d.join("docs/roadmaps")).expect("roadmap dir");
    std::fs::write(
        d.join("docs/roadmaps/roadmap.yaml"),
        "roadmap:\n- id: PMAT-1\n  status: planned\n",
    )
    .expect("roadmap");
    tmp
}

async fn enforce(dir: &Path, disable: bool) {
    handle_enforce(dir, true, disable, ComplyOutputFormat::Json)
        .await
        .expect("enforce");
}

fn commit_msg_accepts(dir: &Path, message: &str) -> bool {
    std::fs::write(dir.join("MSG"), message).expect("message");
    run(
        Command::new("bash").args([".git/hooks/commit-msg", "MSG"]),
        dir,
    )
}

#[tokio::test]
async fn enforce_installs_a_commit_msg_hook_that_refuses_what_cb2113_refuses() {
    let tmp = roadmap_repo();
    let d = tmp.path();
    enforce(d, false).await;
    assert!(
        !commit_msg_accepts(d, "fix: no trailer\n"),
        "a commit without Pmat-Ticket must be refused"
    );
    assert!(
        !commit_msg_accepts(d, "fix: absent\n\nPmat-Ticket: PMAT-2\n"),
        "a ticket the roadmap does not have must be refused"
    );
    assert!(
        commit_msg_accepts(d, "fix: ok\n\nPmat-Ticket: PMAT-1\n"),
        "an open roadmap ticket must be accepted"
    );
}

#[tokio::test]
async fn enforce_leaves_a_foreign_commit_msg_hook_alone() {
    let tmp = roadmap_repo();
    let d = tmp.path();
    let hook = d.join(".git/hooks/commit-msg");
    let foreign = "#!/bin/sh\nexit 0\n";
    std::fs::write(&hook, foreign).expect("foreign hook");
    enforce(d, false).await;
    assert_eq!(std::fs::read_to_string(&hook).expect("read"), foreign);
    enforce(d, true).await;
    assert_eq!(std::fs::read_to_string(&hook).expect("read"), foreign);
}

#[tokio::test]
async fn enforce_disable_removes_the_commit_msg_hook_it_installed() {
    let tmp = roadmap_repo();
    let d = tmp.path();
    enforce(d, false).await;
    assert!(d.join(".git/hooks/commit-msg").exists());
    enforce(d, true).await;
    assert!(!d.join(".git/hooks/commit-msg").exists());
}

#[tokio::test]
async fn work_init_does_not_downgrade_the_enforced_commit_msg_hook() {
    let tmp = roadmap_repo();
    let d = tmp.path();
    enforce(d, false).await;
    crate::services::hook_manager::install_commit_msg_hook(&d.to_path_buf()).expect("work init");
    assert!(
        !commit_msg_accepts(d, "fix: Refs PMAT-1\n"),
        "the trailer is still required after `pmat work init`"
    );
}

/// The pre-push hook run against a stand-in `pmat` whose CB-2113 verdict is `rc`.
fn pre_push_passes(dir: &Path, rc: i32) -> bool {
    pre_push_passes_with(dir, rc, "")
}

/// The same, with `refs` as the lines git writes on the hook's stdin.
fn pre_push_passes_with(dir: &Path, rc: i32, refs: &str) -> bool {
    use std::os::unix::fs::PermissionsExt;
    let bin = dir.join("stub-bin");
    std::fs::create_dir_all(&bin).expect("bin");
    let pmat = bin.join("pmat");
    std::fs::write(
        &pmat,
        format!("#!/bin/sh\ncase \"$*\" in *'--checks CB-2113'*) exit {rc} ;; esac\nexit 0\n"),
    )
    .expect("stub");
    std::fs::set_permissions(&pmat, std::fs::Permissions::from_mode(0o755)).expect("chmod");
    let path = format!(
        "{}:{}",
        bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let stdin = dir.join("refs");
    std::fs::write(&stdin, refs).expect("refs");
    run(
        Command::new("sh")
            .arg(".git/hooks/pre-push")
            .env("PATH", path)
            .stdin(std::fs::File::open(&stdin).expect("stdin")),
        dir,
    )
}

#[tokio::test]
async fn enforce_pre_push_refuses_a_push_cb2113_refuses() {
    let tmp = roadmap_repo();
    let d = tmp.path();
    enforce(d, false).await;
    assert!(
        !pre_push_passes(d, 1),
        "a CB-2113 failure must stop the push"
    );
    assert!(
        pre_push_passes(d, 0),
        "a CB-2113 pass must not stop the push"
    );
}

#[tokio::test]
async fn enforce_pre_push_lets_a_delete_only_push_through() {
    let tmp = roadmap_repo();
    let d = tmp.path();
    enforce(d, false).await;
    let (zero, oid) = ("0".repeat(40), "0a".repeat(20));
    let delete = format!("(delete) {zero} refs/heads/old {oid}\n");
    assert!(
        pre_push_passes_with(d, 1, &delete),
        "a delete sends no commit to judge"
    );
    let mixed = format!("{delete}refs/heads/new {oid} refs/heads/new {zero}\n");
    assert!(
        !pre_push_passes_with(d, 1, &mixed),
        "one pushed commit is enough to judge"
    );
}

#[tokio::test]
async fn enforce_keeps_a_dangling_commit_msg_symlink() {
    let tmp = roadmap_repo();
    let d = tmp.path();
    let link = d.join(".git/hooks/commit-msg");
    std::os::unix::fs::symlink(d.join("elsewhere/commit-msg"), &link).expect("symlink");
    enforce(d, false).await;
    let meta = std::fs::symlink_metadata(&link).expect("link");
    assert!(meta.file_type().is_symlink(), "the user's link stays");
    assert!(!d.join("elsewhere/commit-msg").exists());
}

/// `git` in `dir` with an identity, running that repository's hooks.
fn git(dir: &Path, args: &[&str]) -> bool {
    run(
        Command::new("git")
            .args(["-c", "user.name=t", "-c", "user.email=t@example.invalid"])
            .args([
                "-c",
                "commit.gpgsign=false",
                "-c",
                "core.hooksPath=.git/hooks",
            ])
            .args(args),
        dir,
    )
}

fn commit(dir: &Path, message: &str) -> bool {
    git(dir, &["commit", "-q", "--allow-empty", "-m", message])
}

/// `main` and `feat`, each one commit past their base, `feat` checked out under
/// the enforced hooks. The pre-commit hook is removed: it asks a live `pmat` for
/// an active ticket and is not under test.
async fn diverged_repo() -> tempfile::TempDir {
    let tmp = roadmap_repo();
    let d = tmp.path();
    assert!(git(d, &["symbolic-ref", "HEAD", "refs/heads/main"]));
    assert!(commit(d, "base"));
    assert!(git(d, &["checkout", "-q", "-b", "feat"]));
    assert!(commit(d, "feature"));
    assert!(git(d, &["checkout", "-q", "main"]));
    assert!(commit(d, "mainline"));
    assert!(git(d, &["checkout", "-q", "feat"]));
    enforce(d, false).await;
    std::fs::remove_file(d.join(".git/hooks/pre-commit")).expect("pre-commit");
    tmp
}

#[tokio::test]
async fn the_enforced_hook_passes_the_merges_cb2113_does_not_judge() {
    let tmp = diverged_repo().await;
    let d = tmp.path();
    assert!(!commit(d, "no trailer"), "a normal commit is still judged");
    assert!(git(d, &["merge", "--no-edit", "main"]), "git merge");
    assert!(git(d, &["reset", "-q", "--hard", "HEAD^1"]));
    assert!(git(d, &["merge", "--no-commit", "main"]));
    assert!(
        git(d, &["commit", "--no-edit"]),
        "git commit ending a merge"
    );
    assert!(git(d, &["checkout", "-q", "-b", "topic"]));
    assert!(commit(d, "topic\n\nPmat-Ticket: PMAT-1"));
    assert!(git(d, &["checkout", "-q", "feat"]));
    assert!(
        git(d, &["merge", "--no-ff", "--no-edit", "topic"]),
        "a --no-ff merge of a branch that could fast-forward"
    );
}

#[tokio::test]
async fn a_merge_head_that_adds_no_parent_does_not_exempt_a_commit() {
    let tmp = diverged_repo().await;
    let d = tmp.path();
    assert!(git(d, &["checkout", "-q", "-b", "ahead"]));
    assert!(commit(d, "ahead\n\nPmat-Ticket: PMAT-1"));
    assert!(git(d, &["checkout", "-q", "feat"]));
    // git drops a parent that is an ancestor or descendant of the other
    for rev in ["HEAD~1", "HEAD", "ahead"] {
        let oid = Command::new("git")
            .args(["rev-parse", rev])
            .current_dir(d)
            .output()
            .expect("rev-parse");
        std::fs::write(d.join(".git/MERGE_HEAD"), oid.stdout).expect("MERGE_HEAD");
        assert!(!commit(d, "no trailer"), "MERGE_HEAD={rev}");
    }
    for junk in ["", "not-an-object\n"] {
        std::fs::write(d.join(".git/MERGE_HEAD"), junk).expect("MERGE_HEAD");
        assert!(
            !commit_msg_accepts(d, "no trailer\n"),
            "MERGE_HEAD={junk:?}"
        );
    }
}

#[tokio::test]
async fn a_forged_no_ff_state_naming_no_other_commit_is_refused() {
    let tmp = diverged_repo().await;
    let d = tmp.path();
    let head = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(d)
        .output()
        .expect("rev-parse");
    std::fs::write(d.join(".git/MERGE_MODE"), "no-ff").expect("MERGE_MODE");
    for forged in [head.stdout.as_slice(), b"not-an-object\n"] {
        std::fs::write(d.join(".git/MERGE_HEAD"), forged).expect("MERGE_HEAD");
        assert!(!commit_msg_accepts(d, "no trailer\n"));
    }
}
