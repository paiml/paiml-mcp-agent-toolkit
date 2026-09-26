//! #1285: `pmat hooks install` in a linked worktree failed with "Not a directory
//! (os error 20)". There `.git` is a FILE (`gitdir: …`), so joining `.git/hooks`
//! names a path under a file. These tests build a real `git worktree add` and
//! resolve the hooks dir the way git does. Every git call runs with the user's
//! global and system config switched off, because a global `core.hooksPath` would
//! otherwise decide the answer.
use super::hooks_command::{git_hooks_dir, HooksCommand};
use std::path::{Path, PathBuf};
use std::process::Command;

const ISOLATED: &[(&str, &str)] = &[
    ("GIT_CONFIG_GLOBAL", "/dev/null"),
    ("GIT_CONFIG_NOSYSTEM", "1"),
];

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .current_dir(dir)
        .envs(ISOLATED.iter().copied())
        .args([
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@t",
            "-c",
            "init.defaultBranch=main",
        ])
        .args(args)
        .output()
        .expect("git");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// `<tmp>/main` (a repo with one commit) and `<tmp>/wt` (its linked worktree).
fn repo_with_worktree() -> (tempfile::TempDir, PathBuf, PathBuf) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let main = tmp.path().join("main");
    std::fs::create_dir(&main).expect("mkdir main");
    git(&main, &["init", "-q"]);
    git(&main, &["commit", "-q", "--allow-empty", "-m", "init"]);
    git(&main, &["worktree", "add", "-q", "../wt"]);
    let wt = tmp.path().join("wt");
    (tmp, main, wt)
}

fn canon(p: &Path) -> PathBuf {
    std::fs::canonicalize(p).unwrap_or_else(|e| panic!("canonicalize {}: {e}", p.display()))
}

#[test]
fn the_fixture_is_a_linked_worktree_where_the_old_join_fails() {
    // The control: without it a fixture that is secretly a plain repo passes the
    // test below for the wrong reason.
    let (_tmp, _main, wt) = repo_with_worktree();
    assert!(wt.join(".git").is_file(), "linked worktree: .git is a file");
    let err = std::fs::create_dir_all(wt.join(".git").join("hooks"))
        .expect_err("the pre-#1285 path must be uncreatable");
    assert_eq!(err.raw_os_error(), Some(20), "ENOTDIR, as in #1285: {err}");
}

#[test]
fn a_linked_worktree_resolves_to_the_common_hooks_dir() {
    let (_tmp, main, wt) = repo_with_worktree();
    let hooks = git_hooks_dir(&wt, ISOLATED).expect("git answers inside a worktree");
    std::fs::create_dir_all(&hooks).expect("the resolved dir is creatable");
    assert_eq!(canon(&hooks), canon(&main.join(".git").join("hooks")));
}

#[test]
fn install_from_a_linked_worktree_writes_the_hook_git_runs() {
    let (_tmp, main, wt) = repo_with_worktree();
    let hooks = git_hooks_dir(&wt, ISOLATED).expect("hooks dir");
    let cmd = HooksCommand::new(hooks, wt.join("pmat.toml"));
    let rt = tokio::runtime::Runtime::new().expect("runtime");
    let result = rt
        .block_on(cmd.install(true, false, false, false))
        .expect("install from a linked worktree");
    assert!(result.success, "{}", result.message);
    assert!(main.join(".git/hooks/pre-commit").is_file());
}

#[test]
fn core_hooks_path_is_honoured() {
    let (_tmp, _main, wt) = repo_with_worktree();
    git(&wt, &["config", "core.hooksPath", "custom-hooks"]);
    let hooks = git_hooks_dir(&wt, ISOLATED).expect("hooks dir");
    assert_eq!(hooks, wt.join("custom-hooks"));
}

#[test]
fn a_plain_repo_still_resolves_to_dot_git_hooks() {
    let (_tmp, main, _wt) = repo_with_worktree();
    let hooks = git_hooks_dir(&main, ISOLATED).expect("hooks dir");
    std::fs::create_dir_all(&hooks).expect("mkdir");
    assert_eq!(canon(&hooks), canon(&main.join(".git").join("hooks")));
}

#[test]
fn outside_a_repo_git_gives_no_answer() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let ceiling = tmp.path().to_str().expect("utf-8 tmp path");
    let mut envs = ISOLATED.to_vec();
    envs.push(("GIT_CEILING_DIRECTORIES", ceiling));
    let inner = tmp.path().join("x");
    std::fs::create_dir(&inner).expect("mkdir");
    assert_eq!(git_hooks_dir(&inner, &envs), None);
}
