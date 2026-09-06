#![cfg_attr(coverage_nightly, coverage(off))]
//! PMAT-676 — `pmat work add` and `pmat work edit` must refuse a roadmap that
//! `pmat work validate` rejects, before writing anything.
//!
//! The defect measured on 3.38.0: `add` refused only an UNPARSEABLE roadmap.
//! A roadmap declaring the same id twice parses (a sequence of two well-formed
//! rows is well-formed), so `work validate` failed it with exit 1 while
//! `work add` accepted it, minted an id from it and rewrote the whole file
//! from the lossy serde model — the duplicate stayed, and every field the
//! model drops was silently discarded.
//!
//! Root cause: two raw-text id scanners and no shared validator. PMAT-673's
//! allocator (`id_key_value`/`next_id_number`) and PMAT-674's validator
//! (`collect_id_lines`/`duplicate_ids`) read the same text with different
//! rules and neither asked the other. `services::roadmap_text` is now the one
//! scanner and `check_roadmap_text` the one validator; these tests pin that
//! `add`, `edit` and `validate` agree on what a valid roadmap is.
//!
//! Registered from `cli/handlers/work_handlers/mod.rs` — `autotests = false`
//! and nothing reaches `src/tests/lib.rs`, so a file dropped in `src/tests/`
//! without a `mod` is never compiled (`docs/status/orphan-files-ledger.md`)
//! and its silence would read as a pass.

use crate::cli::commands::WorkPriority;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// A roadmap that PARSES and that `work validate` rejects: PMAT-011 twice.
/// PMAT-010 is a distinct row, so `edit` has something legitimate to change.
const DUPLICATE_FIXTURE: &str = "roadmap_version: '1.0'
github_enabled: false
github_repo: null
roadmap:
- id: PMAT-010
  title: A row with an id of its own
  status: planned
  acceptance_criteria:
  - it parses
- id: PMAT-011
  title: The first clash
  status: planned
- id: PMAT-011
  title: The second clash
  status: planned
";

/// The same shape with distinct ids: the control every refusal is measured
/// against, and the 3.38.0 allocator behaviour that must survive the fix.
const CLEAN_FIXTURE: &str = "roadmap_version: '1.0'
github_enabled: false
github_repo: null
roadmap:
- id: PMAT-010
  title: A row with an id of its own
  status: planned
  acceptance_criteria:
  - it parses
- id: PMAT-011
  title: Another row
  status: planned
";

fn roadmap_path(project: &Path) -> PathBuf {
    project.join("docs/roadmaps/roadmap.yaml")
}

fn lock_path(project: &Path) -> PathBuf {
    project.join("docs/roadmaps/roadmap.yaml.lock")
}

/// Write `yaml` as `docs/roadmaps/roadmap.yaml` of a fresh project directory.
fn project_with_roadmap(yaml: &str) -> TempDir {
    let dir = tempfile::tempdir().expect("a temp dir must be creatable");
    let path = roadmap_path(dir.path());
    std::fs::create_dir_all(path.parent().expect("roadmap.yaml has a parent"))
        .expect("docs/roadmaps must be creatable");
    std::fs::write(&path, yaml).expect("roadmap.yaml must be writable");
    dir
}

/// The 1-based line number of the `nth` (0-based) line reading exactly
/// `needle`. Computed from the fixture rather than written down, so editing a
/// fixture cannot silently make an assertion vacuous.
fn line_of_nth_occurrence(text: &str, needle: &str, nth: usize) -> usize {
    let hits: Vec<usize> = text
        .lines()
        .enumerate()
        .filter(|(_, line)| line.trim() == needle)
        .map(|(index, _)| index + 1)
        .collect();
    assert!(
        hits.len() > nth,
        "fixture must contain at least {} lines {needle:?}, found {hits:?}",
        nth + 1
    );
    hits[nth]
}

fn roadmap_bytes(project: &Path) -> Vec<u8> {
    std::fs::read(roadmap_path(project)).expect("roadmap.yaml must be readable")
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

async fn edit_title(project: &Path, id: &str, title: &str) -> anyhow::Result<()> {
    crate::cli::handlers::work_handlers::handle_work_edit(
        id.to_string(),
        Some(title.to_string()),
        None,
        None,
        None,
        None,
        Some(project.to_path_buf()),
        None,
        vec![],
    )
    .await
}

/// Every assertion the refusal wording owes a reader: the id, and BOTH lines
/// it was declared on, located in the file `validate` would name.
fn assert_locates_the_clash(error: &str, first: usize, second: usize) {
    assert!(
        error.contains("PMAT-011"),
        "the refusal must name the duplicated id: {error}"
    );
    assert!(
        error.contains(&format!("roadmap.yaml:{first}")),
        "the refusal must locate the first occurrence (line {first}): {error}"
    );
    assert!(
        error.contains(&format!("roadmap.yaml:{second}")),
        "the refusal must locate the second occurrence (line {second}): {error}"
    );
}

// ── R1: `work add` refuses a duplicated-id roadmap, writing nothing ──────────

/// The whole defect in one test. `add` must fail with `validate`'s wording,
/// and the two things it would otherwise have consumed — the roadmap bytes and
/// the lock file's id high-water mark — must both be untouched. Checking the
/// bytes and not only the parsed model matters: the old `add` rewrote the file
/// through serde, which is lossy.
#[tokio::test]
async fn work_add_refuses_invalid_add_refuses_a_duplicated_id_and_writes_nothing() {
    let project = project_with_roadmap(DUPLICATE_FIXTURE);
    let first = line_of_nth_occurrence(DUPLICATE_FIXTURE, "- id: PMAT-011", 0);
    let second = line_of_nth_occurrence(DUPLICATE_FIXTURE, "- id: PMAT-011", 1);
    std::fs::write(lock_path(project.path()), "7").expect("seed the high-water mark");
    let before = roadmap_bytes(project.path());

    let result = add(project.path(), "a ticket nobody may mint").await;

    let error = result.err().map_or_else(String::new, |e| format!("{e:#}"));
    assert!(
        !error.is_empty(),
        "a roadmap declaring PMAT-011 on lines {first} and {second} must not be added to"
    );
    assert_locates_the_clash(&error, first, second);
    assert_eq!(
        roadmap_bytes(project.path()),
        before,
        "a refused add must leave the roadmap byte-identical"
    );
    assert_eq!(
        std::fs::read_to_string(lock_path(project.path())).unwrap_or_default(),
        "7",
        "a refused add must not advance the lock file's id high-water mark"
    );
}

// ── R2: `work edit` refuses the same roadmap ────────────────────────────────

/// `edit` saved through the serde model with no text check at all, so it was
/// the second way to launder a roadmap `validate` rejects. Editing a row that
/// is itself blameless (PMAT-010) must still be refused: the file, not the
/// row, is what is invalid.
#[tokio::test]
async fn work_add_refuses_invalid_edit_refuses_a_duplicated_id_and_writes_nothing() {
    let project = project_with_roadmap(DUPLICATE_FIXTURE);
    let first = line_of_nth_occurrence(DUPLICATE_FIXTURE, "- id: PMAT-011", 0);
    let second = line_of_nth_occurrence(DUPLICATE_FIXTURE, "- id: PMAT-011", 1);
    let before = roadmap_bytes(project.path());

    let result = edit_title(project.path(), "PMAT-010", "a title nobody may save").await;

    let error = result.err().map_or_else(String::new, |e| format!("{e:#}"));
    assert!(
        !error.is_empty(),
        "editing a row of a roadmap that declares PMAT-011 twice (lines {first}, {second}) \
         must not be saved"
    );
    assert_locates_the_clash(&error, first, second);
    assert_eq!(
        roadmap_bytes(project.path()),
        before,
        "a refused edit must leave the roadmap byte-identical"
    );
}

// ── R3: the control — a valid roadmap still mints and still validates ───────

/// The refusal must be about the duplicate and nothing else. On a clean
/// roadmap `add` keeps the 3.38.0 allocator behaviour (one past the highest id
/// in the raw text), the edit lands, and `validate` is happy with the result.
#[tokio::test]
async fn work_add_refuses_invalid_clean_roadmap_still_mints_and_still_validates() {
    let project = project_with_roadmap(CLEAN_FIXTURE);

    add(project.path(), "a ticket that may be minted")
        .await
        .expect("a clean roadmap must still accept an add");

    let after = std::fs::read_to_string(roadmap_path(project.path())).expect("roadmap is readable");
    assert!(
        after.contains("PMAT-012"),
        "the next id past PMAT-011 is PMAT-012, roadmap was:\n{after}"
    );

    edit_title(project.path(), "PMAT-010", "an edited title")
        .await
        .expect("a clean roadmap must still accept an edit");
    let edited =
        std::fs::read_to_string(roadmap_path(project.path())).expect("roadmap is readable");
    assert!(
        edited.contains("an edited title"),
        "the edit must have been saved, roadmap was:\n{edited}"
    );

    crate::cli::handlers::work_handlers::handle_work_validate(
        Some(project.path().to_path_buf()),
        false,
        false,
    )
    .await
    .expect("what `add` and `edit` wrote must validate");
}
