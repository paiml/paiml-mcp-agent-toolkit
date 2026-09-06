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

// ── R4: the one scanner, against both implementations it replaces ───────────

/// The merged contract, fixture by fixture, against what the two scanners it
/// replaces returned. Where they disagreed, the validator's reading wins —
/// that is the whole point of having one scanner, and both differences move
/// the allocator towards the stricter reading rather than away from it.
#[test]
fn work_add_refuses_invalid_one_scanner_settles_both_readings() {
    use crate::services::roadmap_text::{duplicate_ids, next_id_number};

    // (1) A block-scalar body is text, not YAML. The validator has always
    // skipped it; the old allocator counted it and would have minted PMAT-901
    // because a reviewer's note quoted an id. Nobody declared PMAT-900 here.
    let block = "roadmap:\n\
                 \x20 - id: PMAT-010\n\
                 \x20   notes: |\n\
                 \x20     the reviewer wrote:\n\
                 \x20     id: PMAT-900\n\
                 \x20   status: planned\n";
    assert_eq!(
        next_id_number(block, None),
        11,
        "an id quoted inside a block scalar is not an id in use (the old \
         allocator read 901 here; the validator has always read none)"
    );
    assert!(
        duplicate_ids(block).is_empty(),
        "and it is not a duplicate either: {:?}",
        duplicate_ids(block)
    );

    // (2) A flow-style row IS a row. The validator read it; the old allocator
    // stopped at the `{` and read no id at all — a false LOW, and a false LOW
    // is the one direction that mints a duplicate.
    let flow = "roadmap:\n  - {id: PMAT-030, title: three, status: planned}\n";
    assert_eq!(
        next_id_number(flow, None),
        31,
        "a flow-style row is an id in use (the old allocator read 1 here)"
    );

    // (3) A subtask's id is an id in use, at any depth. Both implementations
    // agreed, and so does this one.
    let subtask = "roadmap:\n  - id: PMAT-010\n    subtasks:\n      - id: PMAT-900\n";
    assert_eq!(next_id_number(subtask, None), 901);
    assert!(duplicate_ids(subtask).is_empty());
    assert_eq!(
        duplicate_ids("roadmap:\n- id: PMAT-1\n  subtasks:\n  - id: PMAT-1\n"),
        vec![("PMAT-1".to_string(), vec![2, 4])],
        "a subtask reusing its parent's id is a clash at lines 2 and 4"
    );

    // (4) The lock file's high-water mark still beats the text, and still
    // loses to a higher id in the text.
    let raw = "  - id: PMAT-12\n";
    assert_eq!(next_id_number(raw, Some(20)), 21);
    assert_eq!(next_id_number(raw, Some(2)), 13);

    // (5) Every spelling of the key, and nothing that merely looks like one —
    // the PMAT-673 quorum fixture, read by the PMAT-674 scanner.
    let spellings = "roadmap:\n\
                     \x20 -   id: PMAT-021\n\
                     \x20 - \"id\": PMAT-022\n\
                     \x20 - 'id': \"PMAT-023\"\n\
                     \x20 - id:PMAT-024\n\
                     \x20 - id: PMAT-025   # trailing comment\n\
                     \x20 - identity: PMAT-990\n\
                     \x20 # - id: PMAT-991\n\
                     \x20 -id: PMAT-992\n\
                     \x20   github_issue: 993\n";
    assert_eq!(
        next_id_number(spellings, None),
        26,
        "PMAT-025 is the highest id actually declared"
    );
}

/// The validator itself: `Ok` on a clean roadmap, and on a broken one the
/// exact line `pmat work validate` prints, one per duplicated id.
#[test]
fn work_add_refuses_invalid_check_roadmap_text_renders_validates_wording() {
    use crate::services::roadmap_text::check_roadmap_text;

    let path = Path::new("docs/roadmaps/roadmap.yaml");
    assert!(
        check_roadmap_text(CLEAN_FIXTURE, path).is_ok(),
        "a roadmap with distinct ids is one `work add` may write to"
    );

    let first = line_of_nth_occurrence(DUPLICATE_FIXTURE, "- id: PMAT-011", 0);
    let second = line_of_nth_occurrence(DUPLICATE_FIXTURE, "- id: PMAT-011", 1);
    let error = check_roadmap_text(DUPLICATE_FIXTURE, path)
        .expect_err("a roadmap declaring PMAT-011 twice must be refused");
    assert_eq!(
        error.to_string(),
        format!(
            "duplicate id PMAT-011 at docs/roadmaps/roadmap.yaml:{first}, \
             docs/roadmaps/roadmap.yaml:{second}"
        )
    );
    assert_eq!(
        error.duplicates(),
        [("PMAT-011".to_string(), vec![first, second])]
    );
}
