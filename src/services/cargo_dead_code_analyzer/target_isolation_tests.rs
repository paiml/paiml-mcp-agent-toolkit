//! #1305: the analyzer's `cargo check` builds into a directory no other
//! workspace root can build into.
//!
//! cargo fingerprints a workspace member by its path relative to the workspace
//! root and decides freshness by mtime, so a target directory shared by two
//! roots with a package name in common lets the newer root's fingerprint
//! answer for the older root's sources — an uncompilable crate exits 0. The
//! end-to-end plant of that is
//! `a_broken_crate_sharing_a_target_dir_with_a_same_named_crate_is_not_measured`;
//! these tests pin the properties it rests on, without compiling anything.

use super::{
    isolated_target_dir, isolated_target_dir_in, workspace_root_key, CargoDeadCodeAnalyzer,
    ISOLATED_TARGET_SUBDIR,
};
use std::path::{Path, PathBuf};

/// A dependency-free crate that is its own workspace root, at `dir`.
fn crate_at(dir: &Path) {
    std::fs::create_dir_all(dir.join("src")).expect("src");
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"isofx\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n[workspace]\n",
    )
    .expect("manifest");
    std::fs::write(dir.join("src/lib.rs"), "pub fn f() {}\n").expect("lib");
}

/// DCTI-OB-001, the whole of the fix: two roots under ONE target directory get
/// two isolated directories, both directly under `<target>/pmat-dead-code`.
#[test]
fn distinct_workspace_roots_never_share_an_isolated_target_dir() {
    let a = tempfile::tempdir().expect("tempdir");
    let b = tempfile::tempdir().expect("tempdir");
    let shared = Path::new("/shared/target");
    let dir_a = isolated_target_dir_in(shared, a.path());
    let dir_b = isolated_target_dir_in(shared, b.path());
    assert_ne!(
        dir_a, dir_b,
        "two roots must never build into one directory"
    );
    let base = shared.join(ISOLATED_TARGET_SUBDIR);
    assert_eq!(dir_a.parent(), Some(base.as_path()));
    assert_eq!(dir_b.parent(), Some(base.as_path()));
}

/// Worktrees of one repository share their last path component; the readable
/// prefix is therefore not the identity — the hash of the whole path is.
#[test]
fn roots_with_the_same_last_component_are_still_distinct() {
    let one = workspace_root_key(Path::new("/nonexistent/one/repo"));
    let two = workspace_root_key(Path::new("/nonexistent/two/repo"));
    assert_ne!(one, two);
    assert!(
        one.starts_with("repo-") && two.starts_with("repo-"),
        "{one} {two}"
    );
}

/// One directory reached through a symlink is ONE root: its fingerprints are
/// its own, and a second directory for it would only cost a cold build.
#[cfg(unix)]
#[test]
fn one_root_reached_through_a_symlink_is_one_key() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let real = tmp.path().join("real");
    std::fs::create_dir_all(&real).expect("mkdir");
    let link = tmp.path().join("link");
    std::os::unix::fs::symlink(&real, &link).expect("symlink");
    assert_eq!(workspace_root_key(&real), workspace_root_key(&link));
}

/// DCTI-OB-005: the key is FNV-1a over the canonical path, pinned to a value
/// computed outside Rust, so swapping in a hasher whose output may move
/// between toolchains fails here instead of orphaning every directory.
#[test]
fn the_key_is_pinned_across_builds() {
    assert_eq!(
        workspace_root_key(Path::new("/nonexistent/pmat-1305/root")),
        "root-132be0538c3d8309"
    );
}

/// DCTI-OB-002: the isolated directory lives inside the target directory cargo
/// resolves for the crate — the user's placement is honoured, not replaced.
#[test]
fn the_isolated_dir_is_inside_the_target_dir_cargo_resolves() {
    let parent = tempfile::tempdir().expect("tempdir");
    let configured = parent.path().join("configured-target");
    std::fs::create_dir_all(parent.path().join(".cargo")).expect(".cargo");
    std::fs::write(
        parent.path().join(".cargo/config.toml"),
        format!(
            "[build]\ntarget-dir = {:?}\n",
            configured.display().to_string()
        ),
    )
    .expect("config");
    let root = parent.path().join("crate");
    crate_at(&root);
    // An inherited environment variable outranks the config file; read, never
    // set. cargo resolves a relative one against its cwd, which is the crate.
    let expected_base = std::env::var_os("CARGO_TARGET_DIR")
        .or_else(|| std::env::var_os("CARGO_BUILD_TARGET_DIR"))
        .map_or(configured, |dir| root.join(dir));
    assert_eq!(
        isolated_target_dir(&root),
        isolated_target_dir_in(&expected_base, &root)
    );
}

/// DCTI-OB-003: the `cargo check` the analyzer runs names that directory. This
/// is the assertion the mutant (dropping `--target-dir`) fails without a
/// compile.
#[test]
fn the_cargo_check_command_names_the_isolated_target_dir() {
    let tmp = tempfile::tempdir().expect("tempdir");
    crate_at(tmp.path());
    let analyzer = CargoDeadCodeAnalyzer::new(tmp.path());
    let cmd = analyzer
        .build_cargo_check_command()
        .expect("PMAT_DEAD_CODE_SKIP is unset in this test");
    let args: Vec<String> = cmd
        .get_args()
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    let at = args.iter().position(|a| a == "--target-dir");
    assert!(at.is_some(), "no --target-dir in cargo {args:?}");
    assert_eq!(
        at.and_then(|i| args.get(i + 1)).map(PathBuf::from),
        Some(isolated_target_dir(analyzer.cargo_root()))
    );
}
