#![cfg_attr(coverage_nightly, coverage(off))]
//! PMAT-673 — `pmat work add` must mint each ticket id exactly once.
//!
//! The defect (#1193, #1169): `handle_work_add` read the roadmap under a SHARED
//! lock, computed `max(id)+1` from the parsed items, and only then took the
//! EXCLUSIVE lock to write. Two processes both read `max = N`, both minted
//! `N+1`, and the second silently REPLACED the first ticket (`upsert_item`
//! matches on id). These tests pin the allocation to one exclusive lock, to the
//! RAW text of the roadmap (so nested subtask ids count too), and to a
//! high-water mark persisted in the lock file.
//!
//! Registered from `cli/handlers/work_handlers/mod.rs` — `autotests = false`
//! and nothing reaches `src/tests/lib.rs`, so a file dropped in `src/tests/`
//! without a `mod` is never compiled (see `docs/status/orphan-files-ledger.md`).

use crate::cli::commands::WorkPriority;
use crate::models::roadmap::Roadmap;
use crate::services::roadmap_service::{next_id_number, RoadmapService};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Env var that turns the child test below from a no-op into one real add.
const CHILD_PROJECT_ENV: &str = "PMAT_673_CHILD_PROJECT";
/// Env var carrying the unique title the child must add.
const CHILD_TITLE_ENV: &str = "PMAT_673_CHILD_TITLE";

const EMPTY_ROADMAP: &str = "roadmap_version: '1.0'\ngithub_enabled: true\nroadmap: []\n";

fn project_with_roadmap(body: &str) -> TempDir {
    let dir = TempDir::new().expect("temp project dir");
    write_roadmap(dir.path(), body);
    dir
}

fn roadmap_path(project: &Path) -> PathBuf {
    project.join("docs/roadmaps/roadmap.yaml")
}

fn lock_path(project: &Path) -> PathBuf {
    project.join("docs/roadmaps/roadmap.yaml.lock")
}

fn write_roadmap(project: &Path, body: &str) {
    let path = roadmap_path(project);
    std::fs::create_dir_all(path.parent().expect("roadmap has a parent"))
        .expect("create docs/roadmaps");
    std::fs::write(&path, body).expect("write roadmap.yaml");
}

fn load_roadmap(project: &Path) -> Roadmap {
    RoadmapService::new(roadmap_path(project))
        .load()
        .expect("roadmap loads")
}

fn lock_contents(project: &Path) -> String {
    std::fs::read_to_string(lock_path(project)).unwrap_or_default()
}

async fn add(project: &Path, title: &str) -> anyhow::Result<()> {
    crate::cli::handlers::work_handlers::handle_work_add(
        title.to_string(),
        None,
        WorkPriority::Medium,
        None,
        Some(project.to_path_buf()),
        false,
        None,
    )
    .await
}

fn ids(roadmap: &Roadmap) -> Vec<String> {
    roadmap.roadmap.iter().map(|i| i.id.clone()).collect()
}

// ── T1: two sequential adds mint two different ids ──────────────────────────

#[tokio::test]
async fn work_add_allocator_sequential_adds_mint_001_then_002() {
    let project = project_with_roadmap(EMPTY_ROADMAP);
    add(project.path(), "first ticket")
        .await
        .expect("first add succeeds");
    add(project.path(), "second ticket")
        .await
        .expect("second add succeeds");

    let roadmap = load_roadmap(project.path());
    assert_eq!(ids(&roadmap), vec!["PMAT-001", "PMAT-002"]);
    let titles: Vec<&str> = roadmap.roadmap.iter().map(|i| i.title.as_str()).collect();
    assert_eq!(titles, vec!["first ticket", "second ticket"]);
}

// ── T2: the lock file carries a high-water mark ─────────────────────────────

#[tokio::test]
async fn work_add_allocator_honours_the_lock_file_high_water_mark() {
    let project = project_with_roadmap(
        "roadmap_version: '1.0'\n\
         github_enabled: true\n\
         roadmap:\n\
         \x20 - id: PMAT-010\n\
         \x20   title: an existing ticket\n\
         \x20   status: planned\n",
    );
    std::fs::create_dir_all(lock_path(project.path()).parent().expect("parent"))
        .expect("create docs/roadmaps");
    std::fs::write(lock_path(project.path()), "5000").expect("seed the high-water mark");

    add(project.path(), "after the high-water mark")
        .await
        .expect("add succeeds");

    let roadmap = load_roadmap(project.path());
    assert!(
        roadmap.roadmap.iter().any(|i| i.id == "PMAT-5001"),
        "the lock file's 5000 must beat the roadmap's 10, ids were {:?}",
        ids(&roadmap)
    );
    assert_eq!(
        lock_contents(project.path()).trim(),
        "5001",
        "the mint must advance the high-water mark it just consumed"
    );
}

// ── T3: the RAW text beats the parsed model (nested subtask ids count) ──────

#[tokio::test]
async fn work_add_allocator_counts_nested_subtask_ids_from_the_raw_text() {
    let project = project_with_roadmap(
        "roadmap_version: '1.0'\n\
         github_enabled: true\n\
         roadmap:\n\
         \x20 - id: PMAT-010\n\
         \x20   title: an epic\n\
         \x20   status: planned\n\
         \x20   subtasks:\n\
         \x20     - id: PMAT-900\n\
         \x20       title: a subtask\n\
         \x20       status: planned\n",
    );

    add(project.path(), "after the subtask")
        .await
        .expect("add succeeds");

    let roadmap = load_roadmap(project.path());
    assert!(
        roadmap.roadmap.iter().any(|i| i.id == "PMAT-901"),
        "a subtask id is still an id in use, ids were {:?}",
        ids(&roadmap)
    );
}

// ── T4: an unparseable roadmap writes nothing ───────────────────────────────

#[tokio::test]
async fn work_add_allocator_writes_nothing_when_the_roadmap_does_not_parse() {
    let broken = "roadmap_version: '1.0'\n\
                  github_enabled: true\n\
                  roadmap:\n\
                  \x20 - id: PMAT-010\n\
                  \x20   title: a broken row\n\
                  \x20   status: bogus\n";
    let project = project_with_roadmap(broken);
    let before = std::fs::read(roadmap_path(project.path())).expect("read before");

    let err = add(project.path(), "must not land")
        .await
        .expect_err("an unparseable roadmap must refuse the add");
    let msg = format!("{err:#}");
    assert!(
        msg.contains("roadmap.yaml"),
        "error must name the file: {msg}"
    );
    assert!(msg.contains("line "), "error must locate the row: {msg}");

    let after = std::fs::read(roadmap_path(project.path())).expect("read after");
    assert_eq!(before, after, "the roadmap bytes must be untouched");
    assert!(
        lock_contents(project.path()).trim().parse::<u32>().is_err(),
        "a refused add must not bump the high-water mark, lock read {:?}",
        lock_contents(project.path())
    );
}

// ── T5: twelve concurrent PROCESSES mint twelve distinct ids ────────────────

/// The full libtest path of [`work_add_allocator_child_mints_once`].
///
/// Derived from `module_path!()` (libtest drops the crate segment) so that
/// moving the module cannot silently turn the parent test into a test of an
/// empty filter.
fn child_test_path() -> String {
    let module = module_path!();
    let module = module.strip_prefix("pmat::").unwrap_or(module);
    format!("{module}::work_add_allocator_child_mints_once")
}

/// One `handle_work_add` in a process of its own — the other half of
/// `work_add_allocator_twelve_processes_mint_twelve_distinct_ids`.
///
/// It is a NO-OP unless `PMAT_673_CHILD_PROJECT` is set, and that is deliberate:
/// `cargo test --lib` runs this like any other test, where there is no project
/// to add to. The parent re-executes the test binary with `--exact <this path>`
/// and the env var set, which is the only way to get two real OS processes
/// contending for the roadmap lock — threads would share the process-wide
/// `flock`, so an in-process test cannot observe the defect at all.
#[tokio::test]
async fn work_add_allocator_child_mints_once() {
    let Ok(project) = std::env::var(CHILD_PROJECT_ENV) else {
        return;
    };
    let title = std::env::var(CHILD_TITLE_ENV).expect("the parent sets a title with the project");
    let project = PathBuf::from(project);

    add(&project, &title).await.expect("child add succeeds");

    let roadmap = load_roadmap(&project);
    let item = roadmap.roadmap.iter().find(|i| i.title == title);
    assert!(
        item.is_some(),
        "the child's own ticket '{title}' is missing after its add, roadmap holds {:?}",
        ids(&roadmap)
    );
    println!("MINTED {}", item.expect("asserted present above").id);
}

#[test]
fn work_add_allocator_twelve_processes_mint_twelve_distinct_ids() {
    use std::process::{Command, Stdio};

    let project = project_with_roadmap(EMPTY_ROADMAP);
    let exe = std::env::current_exe().expect("the test binary knows its own path");
    let child = child_test_path();

    let mut minted: Vec<(String, String)> = Vec::new();
    for round in 0..2u32 {
        let mut running = Vec::new();
        for slot in 0..6u32 {
            let title = format!("pmat-673 concurrent {round}-{slot}");
            let handle = Command::new(&exe)
                .args(["--exact", &child, "--nocapture"])
                .env(CHILD_PROJECT_ENV, project.path())
                .env(CHILD_TITLE_ENV, &title)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("spawn the child test process");
            running.push((title, handle));
        }
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
            minted.push((title, id.expect("asserted present above").to_string()));
        }
    }

    assert_eq!(minted.len(), 12, "twelve children must each report one id");
    let distinct: BTreeSet<&str> = minted.iter().map(|(_, id)| id.as_str()).collect();
    assert_eq!(
        distinct.len(),
        12,
        "every child must mint a DIFFERENT id, got {:?}",
        minted.iter().map(|(_, id)| id.as_str()).collect::<Vec<_>>()
    );

    let roadmap = load_roadmap(project.path());
    assert_eq!(
        roadmap.roadmap.len(),
        12,
        "no add may be overwritten, roadmap holds {:?}",
        ids(&roadmap)
    );
    for (title, id) in &minted {
        let found = roadmap.roadmap.iter().find(|i| &i.id == id);
        assert!(
            found.is_some(),
            "{id} is gone from the roadmap: {:?}",
            ids(&roadmap)
        );
        assert_eq!(
            &found.expect("asserted present above").title,
            title,
            "{id} carries the wrong title"
        );
    }
}

// ── T6: the pure allocator ──────────────────────────────────────────────────

#[test]
fn work_add_allocator_next_id_number_starts_at_one() {
    assert_eq!(next_id_number("", None), 1);
    assert_eq!(
        next_id_number("roadmap_version: '1.0'\nroadmap: []\n", None),
        1
    );
}

#[test]
fn work_add_allocator_next_id_number_takes_the_max_over_every_prefix() {
    let raw = "roadmap:\n  - id: GH-7\n  - id: PMAT-3\n";
    assert_eq!(next_id_number(raw, None), 8);
}

#[test]
fn work_add_allocator_next_id_number_reads_quoted_ids() {
    assert_eq!(next_id_number("  - id: \"PMAT-12\"\n", None), 13);
    assert_eq!(next_id_number("  - id: 'PMAT-12'\n", None), 13);
}

#[test]
fn work_add_allocator_next_id_number_counts_nested_subtask_ids() {
    let raw = "roadmap:\n  - id: PMAT-010\n    subtasks:\n      - id: PMAT-900\n";
    assert_eq!(next_id_number(raw, None), 901);
}

#[test]
fn work_add_allocator_next_id_number_lock_high_water_can_win() {
    let raw = "  - id: PMAT-12\n";
    assert_eq!(next_id_number(raw, Some(20)), 21);
    assert_eq!(next_id_number(raw, Some(2)), 13);
}

#[test]
fn work_add_allocator_next_id_number_ignores_non_numeric_suffixes() {
    let raw = "  - id: PMAT-XX\n  - id: no-number-here\n  - id: PMAT-009\n";
    assert_eq!(next_id_number(raw, None), 10);
}
