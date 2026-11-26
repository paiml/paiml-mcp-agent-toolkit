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
    run(
        Command::new("sh")
            .arg(".git/hooks/pre-push")
            .env("PATH", path),
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
