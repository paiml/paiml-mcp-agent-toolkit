//! #1285: `pmat hooks install` in a linked worktree failed with "Not a directory
//! (os error 20)". There `.git` is a FILE (`gitdir: …`), so joining `.git/hooks`
//! names a path under a file. These tests build a real `git worktree add` and
//! resolve the hooks dir the way git does. Every git call runs with the user's
//! global and system config switched off, because a global `core.hooksPath` would
//! otherwise decide the answer, and without the `GIT_DIR` family a git hook exports,
//! because inherited they would point every call at the repository running the hook.
use super::hooks_command::{
    fallback_hooks_dir, git_hooks_dir, git_hooks_dir_command, HooksCommand,
};
use super::tdg_hooks::install_tdg_hooks;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

const ISOLATED: &[(&str, &str)] = &[
    ("GIT_CONFIG_GLOBAL", "/dev/null"),
    ("GIT_CONFIG_NOSYSTEM", "1"),
];

/// Variables that name a repository outright; git exports them to its hooks.
const REPO_VARS: &[&str] = &[
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_COMMON_DIR",
    "GIT_INDEX_FILE",
];

fn git<S: AsRef<OsStr> + std::fmt::Debug>(dir: &Path, args: &[S]) {
    let mut cmd = Command::new("git");
    for var in REPO_VARS {
        cmd.env_remove(var);
    }
    let out = cmd
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
    let what = format!("canonicalize {}", p.display());
    std::fs::canonicalize(p).expect(&what)
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

#[test]
fn tdg_install_from_a_linked_worktree_writes_where_git_runs_hooks() {
    // install_tdg_hooks resolves with the caller's own git config. Pinning
    // core.hooksPath in the repository (to git's own default) keeps a global value on
    // the test host from sending the hook outside the temp dir.
    let (_tmp, main, wt) = repo_with_worktree();
    let common_hooks = main.join(".git").join("hooks");
    git(
        &wt,
        &[
            OsStr::new("config"),
            OsStr::new("core.hooksPath"),
            common_hooks.as_os_str(),
        ],
    );
    let rt = tokio::runtime::Runtime::new().expect("runtime");
    rt.block_on(install_tdg_hooks(&wt))
        .expect("tdg install from a linked worktree");
    assert!(common_hooks.join("pre-commit").is_file());
}

#[cfg(unix)]
#[test]
fn a_hooks_path_that_is_not_utf8_still_resolves() {
    use std::os::unix::ffi::OsStrExt;
    let (_tmp, _main, wt) = repo_with_worktree();
    let name = OsStr::from_bytes(b"hooks-\xff");
    git(
        &wt,
        &[OsStr::new("config"), OsStr::new("core.hooksPath"), name],
    );
    assert_eq!(git_hooks_dir(&wt, ISOLATED), Some(wt.join(name)));
}

#[test]
fn without_git_a_linked_worktree_falls_back_to_the_common_hooks_dir() {
    let (_tmp, main, wt) = repo_with_worktree();
    let common_hooks = main.join(".git").join("hooks");
    std::fs::create_dir_all(&common_hooks).expect("mkdir hooks");
    assert_eq!(canon(&fallback_hooks_dir(&wt)), canon(&common_hooks));
    assert_eq!(fallback_hooks_dir(&main), common_hooks);
}

#[test]
fn the_git_query_drops_an_inherited_repository() {
    // Inside a git hook GIT_DIR names the repository running the hook, not `dir`.
    let cmd = git_hooks_dir_command(Path::new("."), &[]);
    let removed: Vec<&OsStr> = cmd
        .get_envs()
        .filter(|(_, value)| value.is_none())
        .map(|(key, _)| key)
        .collect();
    for var in REPO_VARS {
        assert!(removed.contains(&OsStr::new(var)), "{var} must be removed");
    }
}

#[test]
fn the_backup_is_reported_where_it_was_written() {
    // The success message names `backup_path()`; it must be the file install wrote
    // in the common hooks dir, not a literal `.git/hooks/...` that a worktree lacks.
    let (_tmp, main, wt) = repo_with_worktree();
    let hooks = git_hooks_dir(&wt, ISOLATED).expect("hooks dir");
    std::fs::create_dir_all(&hooks).expect("mkdir");
    std::fs::write(hooks.join("pre-commit"), "#!/bin/sh\nexit 0\n").expect("old hook");
    let cmd = HooksCommand::new(hooks, wt.join("pmat.toml"));
    let rt = tokio::runtime::Runtime::new().expect("runtime");
    let result = rt
        .block_on(cmd.install(true, true, false, false))
        .expect("install with backup");
    assert!(result.backup_created, "{}", result.message);
    assert!(cmd.backup_path().is_file());
    assert_eq!(
        canon(&cmd.backup_path()),
        canon(&main.join(".git/hooks/pre-commit.pmat-backup"))
    );
}
