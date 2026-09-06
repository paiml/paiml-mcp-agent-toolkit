#![cfg_attr(coverage_nightly, coverage(off))]
//! PMAT-680 (#1193 third half) — a ticket id is minted from ONE authority per
//! repository, not from whatever one checkout happens to hold.
//!
//! The defect measured on 3.38.0: `add_item_with_next_id` minted
//! `max(ids in THIS checkout's raw roadmap text, THIS checkout's
//! `roadmap.yaml.lock` high-water mark) + 1`, under a lock on that sibling
//! file. Two checkouts of the same repository — two worktrees, or two clones —
//! each see their own roadmap and their own lock, so they mint the SAME id by
//! construction, and an id already spent on another branch is minted a second
//! time. Neither is visible from inside one checkout, which is why no test
//! before this one could see it.
//!
//! The two facts pinned here:
//!
//! * S1 — an id spent on any ref of the repository is spent (`feature` holds
//!   PMAT-020, so the next mint on a branch whose roadmap stops at PMAT-010 is
//!   PMAT-021, not PMAT-011);
//! * S2 — two checkouts minting AT THE SAME TIME, in two OS processes, mint
//!   distinct ids, and each still only appends to its own roadmap.
//!
//! S2 needs real processes: `flock` is per-process, so two threads of one test
//! binary share the lock and cannot observe the defect at all. The parent
//! re-executes the test binary with `--exact <child path>` (the pattern of
//! `work_add_allocator_tests.rs`), once per checkout, at the same time.
//!
//! Registered from `cli/handlers/work_handlers/mod.rs` — `autotests = false`
//! and nothing reaches `src/tests/lib.rs`, so a file dropped in `src/tests/`
//! without a `mod` is never compiled (`docs/status/orphan-files-ledger.md`)
//! and its silence would read as a pass.

use crate::cli::commands::WorkPriority;
use crate::models::roadmap::{Roadmap, RoadmapItem};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

/// Env var that turns the child test below from a no-op into one real add.
const CHILD_PROJECT_ENV: &str = "PMAT_680_CHILD_PROJECT";
/// Env var carrying the unique title the child must add.
const CHILD_TITLE_ENV: &str = "PMAT_680_CHILD_TITLE";

// ── the temp repository ─────────────────────────────────────────────────────

/// A roadmap holding PMAT-001..PMAT-010, in the byte-for-byte block style a
/// hand-maintained roadmap is written in.
fn ten_row_roadmap() -> String {
    let mut text = String::from(
        "roadmap_version: '1.0'\ngithub_enabled: false\ngithub_repo: null\nroadmap:\n",
    );
    for number in 1..=10u32 {
        text.push_str(&format!(
            "- id: PMAT-{number:03}\n  title: ticket {number}\n  status: planned\n"
        ));
    }
    text
}

fn roadmap_path(checkout: &Path) -> PathBuf {
    checkout.join("docs/roadmaps/roadmap.yaml")
}

fn sibling_lock_path(checkout: &Path) -> PathBuf {
    checkout.join("docs/roadmaps/roadmap.yaml.lock")
}

fn roadmap_text(checkout: &Path) -> String {
    std::fs::read_to_string(roadmap_path(checkout)).expect("roadmap.yaml must be readable")
}

fn write_roadmap(checkout: &Path, body: &str) {
    let path = roadmap_path(checkout);
    std::fs::create_dir_all(path.parent().expect("roadmap.yaml has a parent"))
        .expect("docs/roadmaps must be creatable");
    std::fs::write(&path, body).expect("roadmap.yaml must be writable");
}

/// `git -C <dir> [-c identity] <args>`, asserted to succeed, stdout trimmed.
fn git(dir: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args([
            "-c",
            "user.email=t@t",
            "-c",
            "user.name=t",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .output()
        .expect("git must be runnable");
    assert!(
        output.status.success(),
        "git {args:?} failed in {dir:?} ({:?})\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

/// The repository root inside `tmp` — a subdirectory, so a linked worktree can
/// live beside it rather than inside the tree it is a checkout of.
fn repo_root(tmp: &TempDir) -> PathBuf {
    tmp.path().join("repo")
}

/// A fresh git repository at `<tmp>/repo` whose one commit carries a roadmap
/// of PMAT-001..PMAT-010.
fn repo_with_committed_roadmap() -> TempDir {
    let tmp = TempDir::new().expect("temp dir must be creatable");
    let root = repo_root(&tmp);
    std::fs::create_dir_all(&root).expect("repo dir must be creatable");
    git(&root, &["init", "-q"]);
    write_roadmap(&root, &ten_row_roadmap());
    git(&root, &["add", "docs/roadmaps/roadmap.yaml"]);
    git(&root, &["commit", "-q", "-m", "the roadmap"]);
    tmp
}

/// `<git-common-dir>/pmat/roadmap-id.lock` — the ONE authority, read here the
/// way an operator would, with git rather than with the code under test.
fn common_dir_lock(checkout: &Path) -> PathBuf {
    let common = git(checkout, &["rev-parse", "--git-common-dir"]);
    let common = PathBuf::from(&common);
    let common = if common.is_absolute() {
        common
    } else {
        checkout.join(common)
    };
    common.join("pmat").join("roadmap-id.lock")
}

// ── the command under test ──────────────────────────────────────────────────

async fn add(checkout: &Path, title: &str) -> anyhow::Result<()> {
    crate::cli::handlers::work_handlers::handle_work_add(
        title.to_string(),
        None,
        WorkPriority::Medium,
        None,
        Some(checkout.to_path_buf()),
        false,
        None,
    )
    .await
}

fn parse_roadmap(text: &str) -> Roadmap {
    let parsed = serde_yaml_ng::from_str::<Roadmap>(text);
    assert!(parsed.is_ok(), "the roadmap must parse: {parsed:?}\n{text}");
    parsed.expect("checked Ok immediately above")
}

fn ids(roadmap: &Roadmap) -> Vec<String> {
    roadmap.roadmap.iter().map(|i| i.id.clone()).collect()
}

/// The bytes `new` has that `old` did not — asserting first that every byte of
/// `old` survived, in place. The append-only guarantee of PMAT-679, restated
/// here because a mint from a shared authority must not cost it.
fn appended_tail(old: &str, new: &str) -> String {
    assert!(
        new.starts_with(old),
        "an add must leave every existing byte identical.\nbefore:\n{old}\nafter:\n{new}"
    );
    new[old.len()..].to_string()
}

fn parse_items(block: &str) -> Vec<RoadmapItem> {
    let parsed = serde_yaml_ng::from_str::<Vec<RoadmapItem>>(block);
    assert!(
        parsed.is_ok(),
        "the appended blocks must parse as items: {parsed:?}\n{block}"
    );
    parsed.expect("checked Ok immediately above")
}

// ── S1: an id spent on ANOTHER ref is spent ─────────────────────────────────

#[tokio::test]
async fn work_add_single_authority_counts_ids_on_other_refs() {
    let tmp = repo_with_committed_roadmap();
    let root = repo_root(&tmp);

    // A second branch spends PMAT-020 and commits it.
    git(&root, &["checkout", "-q", "-b", "feature"]);
    let mut on_feature = roadmap_text(&root);
    on_feature.push_str("- id: PMAT-020\n  title: minted on feature\n  status: planned\n");
    write_roadmap(&root, &on_feature);
    git(&root, &["add", "docs/roadmaps/roadmap.yaml"]);
    git(&root, &["commit", "-q", "-m", "a ticket on feature"]);

    // Back on the first branch, whose roadmap stops at PMAT-010.
    git(&root, &["checkout", "-q", "-"]);
    assert!(
        !roadmap_text(&root).contains("PMAT-020"),
        "the checkout under test must not itself hold PMAT-020"
    );

    add(&root, "after the other ref")
        .await
        .expect("a clean roadmap must accept an add");

    let roadmap = parse_roadmap(&roadmap_text(&root));
    assert!(
        roadmap.roadmap.iter().any(|i| i.id == "PMAT-021"),
        "an id spent on any ref of the repository is spent: the mint must be \
         PMAT-021, ids are {:?}",
        ids(&roadmap)
    );
    assert!(
        !roadmap.roadmap.iter().any(|i| i.id == "PMAT-011"),
        "PMAT-011 would re-use nothing this checkout can see, and that is the \
         defect: ids are {:?}",
        ids(&roadmap)
    );
}

// ── S2: two checkouts, at the same time, in two processes ───────────────────

/// The full libtest path of [`work_add_single_authority_child_mints_once`].
///
/// Derived from `module_path!()` (libtest drops the crate segment) so that
/// moving the module cannot silently turn the parent test into a test of an
/// empty filter.
fn child_test_path() -> String {
    let module = module_path!();
    let module = module.strip_prefix("pmat::").unwrap_or(module);
    format!("{module}::work_add_single_authority_child_mints_once")
}

/// One `handle_work_add` in a process of its own. A NO-OP unless
/// `PMAT_680_CHILD_PROJECT` is set: `cargo test --lib` runs it like any other
/// test, where there is no checkout to add to.
#[tokio::test]
async fn work_add_single_authority_child_mints_once() {
    let Ok(checkout) = std::env::var(CHILD_PROJECT_ENV) else {
        return;
    };
    let title = std::env::var(CHILD_TITLE_ENV).expect("the parent sets a title with the checkout");
    let checkout = PathBuf::from(checkout);

    add(&checkout, &title).await.expect("child add succeeds");

    let roadmap = parse_roadmap(&roadmap_text(&checkout));
    let item = roadmap.roadmap.iter().find(|i| i.title == title);
    assert!(
        item.is_some(),
        "the child's own ticket '{title}' is missing after its add, roadmap holds {:?}",
        ids(&roadmap)
    );
    println!("MINTED {}", item.expect("asserted present above").id);
}

/// Spawn one child per checkout, at the same time, and return the id each
/// minted in the order the checkouts were given.
fn mint_concurrently(checkouts: &[PathBuf], round: u32) -> Vec<String> {
    use std::process::Stdio;

    let exe = std::env::current_exe().expect("the test binary knows its own path");
    let child = child_test_path();
    let mut running = Vec::new();
    for (slot, checkout) in checkouts.iter().enumerate() {
        let title = format!("pmat-680 concurrent {round}-{slot}");
        let handle = Command::new(&exe)
            .args(["--exact", &child, "--nocapture"])
            .env(CHILD_PROJECT_ENV, checkout)
            .env(CHILD_TITLE_ENV, &title)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn the child test process");
        running.push((title, handle));
    }

    let mut minted = Vec::new();
    for (title, handle) in running {
        let out = handle.wait_with_output().expect("child process completes");
        let stdout = String::from_utf8_lossy(&out.stdout).to_string();
        let stderr = String::from_utf8_lossy(&out.stderr).to_string();
        assert!(
            out.status.success(),
            "child for '{title}' failed ({:?})\nstdout:\n{stdout}\nstderr:\n{stderr}",
            out.status
        );
        let id = stdout
            .lines()
            .find_map(|l| l.trim().strip_prefix("MINTED "));
        assert!(
            id.is_some(),
            "no MINTED line for '{title}'\nstdout:\n{stdout}\nstderr:\n{stderr}"
        );
        minted.push(id.expect("asserted present above").to_string());
    }
    minted
}

#[test]
fn work_add_single_authority_two_checkouts_mint_distinct_ids() {
    let tmp = repo_with_committed_roadmap();
    let root = repo_root(&tmp);
    let second = tmp.path().join("wt2");
    git(
        &root,
        &[
            "worktree",
            "add",
            "-q",
            second.to_str().expect("temp path is utf-8"),
            "-b",
            "second",
        ],
    );

    let checkouts = vec![root.clone(), second.clone()];
    let before: Vec<String> = checkouts.iter().map(|c| roadmap_text(c)).collect();
    let mut per_checkout: Vec<Vec<String>> = vec![Vec::new(), Vec::new()];
    for round in 0..3u32 {
        let minted = mint_concurrently(&checkouts, round);
        for (slot, id) in minted.into_iter().enumerate() {
            per_checkout[slot].push(id);
        }
    }

    // Every mint, in either checkout and any round, is a DIFFERENT id.
    let all: Vec<&str> = per_checkout
        .iter()
        .flat_map(|ids| ids.iter().map(String::as_str))
        .collect();
    let distinct: BTreeSet<&str> = all.iter().copied().collect();
    assert_eq!(
        distinct.len(),
        all.len(),
        "two checkouts of one repository must never mint the same id, got {all:?}"
    );

    // Each checkout appended exactly its own three rows, and touched nothing else.
    for (slot, checkout) in checkouts.iter().enumerate() {
        let after = roadmap_text(checkout);
        let tail = appended_tail(&before[slot], &after);
        let appended = parse_items(&tail);
        let appended_ids: Vec<String> = appended.iter().map(|i| i.id.clone()).collect();
        assert_eq!(
            appended_ids, per_checkout[slot],
            "{checkout:?} must hold exactly the rows it minted, in order"
        );
        let roadmap = parse_roadmap(&after);
        assert_eq!(
            roadmap.roadmap.len(),
            13,
            "{checkout:?} must hold its ten rows plus its own three, ids are {:?}",
            ids(&roadmap)
        );
        let other = 1 - slot;
        for foreign in &per_checkout[other] {
            assert!(
                !after.contains(foreign.as_str()),
                "{foreign} belongs to the other checkout and must not appear in {checkout:?}"
            );
        }
    }
}

// ── S3: the authority is one file, shared by every checkout ─────────────────

#[tokio::test]
async fn work_add_single_authority_lock_lives_in_the_git_common_dir() {
    let tmp = repo_with_committed_roadmap();
    let root = repo_root(&tmp);
    let second = tmp.path().join("wt3");
    git(
        &root,
        &[
            "worktree",
            "add",
            "-q",
            second.to_str().expect("temp path is utf-8"),
            "-b",
            "third",
        ],
    );

    add(&root, "from the main checkout")
        .await
        .expect("the main checkout accepts an add");
    add(&second, "from the linked worktree")
        .await
        .expect("the linked worktree accepts an add");

    // Both checkouts resolve the same authority file...
    assert_eq!(
        common_dir_lock(&root),
        common_dir_lock(&second),
        "every checkout of a repository must resolve ONE authority file"
    );
    let lock = common_dir_lock(&root);
    assert!(
        lock.exists(),
        "the authority must live at {lock:?} after two adds"
    );
    assert_eq!(
        std::fs::read_to_string(&lock)
            .expect("the authority file must be readable")
            .trim(),
        "12",
        "the authority must carry the last id minted through it"
    );

    // ...and neither wrote the per-checkout sibling the defect locked on.
    for checkout in [&root, &second] {
        assert!(
            !sibling_lock_path(checkout).exists(),
            "a per-checkout {:?} is exactly the state two checkouts cannot share",
            sibling_lock_path(checkout)
        );
    }
}

// ── S4: outside a repository, the sibling lock is still the authority ───────

#[tokio::test]
async fn work_add_single_authority_falls_back_to_the_sibling_lock_outside_git() {
    let tmp = TempDir::new().expect("temp dir must be creatable");
    let project = tmp.path().to_path_buf();
    write_roadmap(
        &project,
        "roadmap_version: '1.0'\ngithub_enabled: true\nroadmap: []\n",
    );
    assert!(
        !project.join(".git").exists(),
        "the control must not be a git repository"
    );

    add(&project, "the first ticket")
        .await
        .expect("a non-git project accepts an add");

    let roadmap = parse_roadmap(&roadmap_text(&project));
    assert_eq!(ids(&roadmap), vec!["PMAT-001"]);
    assert_eq!(
        std::fs::read_to_string(sibling_lock_path(&project))
            .expect("the sibling lock must be readable outside git")
            .trim(),
        "1",
        "outside a repository the sibling lock is still the high-water mark"
    );
}
