#![cfg_attr(coverage_nightly, coverage(off))]
//! PMAT-713 / #1240 — a merge that reuses a ticket id DELETES a ticket, and
//! every existing check passes afterwards because the survivor is unique.
//!
//! Measured before this module existed, on a fixture reproducing the reported
//! shape: `pmat work validate` printed `✓ Validation passed` on the tree where
//! `PMAT-002` had silently stopped meaning agent A's work and started meaning
//! agent BSE's. Uniqueness is preserved BY the loss, so uniqueness cannot be
//! the check.
//!
//! Registered from `cli/handlers/work_handlers/mod.rs` for the same reason its
//! siblings are: `autotests = false` and nothing reaches `src/tests/lib.rs`, so
//! a file dropped in `tests/` without a `mod` is never compiled and its silence
//! reads as a pass.

use crate::services::roadmap_text::{titles_by_id, titles_changed};

const BASE: &str = "\
roadmap_version: '1.0'
roadmap:
- id: PMAT-001
  item_type: task
  title: 'the pre-existing row'
  status: planned
- id: PMAT-002
  item_type: task
  title: 'L0-1a the CUDA row that is in the merge queue'
  status: planned
";

/// The merged tree exactly as reported: ONE entry for the id, and it is the
/// other agent's work.
const AFTER_MERGE: &str = "\
roadmap_version: '1.0'
roadmap:
- id: PMAT-001
  item_type: task
  title: 'the pre-existing row'
  status: planned
- id: PMAT-002
  item_type: task
  title: 'BSE-09b merge=union on GitHub merge engine'
  status: planned
";

#[test]
fn a_reused_id_is_a_collision_even_though_the_ids_are_unique() {
    // The property that made this invisible: the merged tree passes a
    // uniqueness check. Assert that, so the test cannot be mistaken for one
    // about duplicates.
    let ids: Vec<String> = crate::services::roadmap_text::id_lines(AFTER_MERGE)
        .into_iter()
        .map(|(_, id)| id)
        .collect();
    let mut unique = ids.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(ids.len(), unique.len(), "fixture must have unique ids");

    let changed = titles_changed(BASE, AFTER_MERGE);
    assert_eq!(
        changed.len(),
        1,
        "the reused id must be reported: {changed:?}"
    );
    assert_eq!(changed[0].id, "PMAT-002");
    assert!(changed[0].before.starts_with("L0-1a"));
    assert!(changed[0].after.starts_with("BSE-09b"));
}

#[test]
fn adding_a_row_is_not_a_collision() {
    let head = format!(
        "{BASE}- id: PMAT-003\n  item_type: task\n  title: 'a genuinely new row'\n  status: planned\n"
    );
    assert!(
        titles_changed(BASE, &head).is_empty(),
        "appending a row must not read as a collision"
    );
}

#[test]
fn deleting_a_row_is_not_a_collision() {
    let head = "\
roadmap_version: '1.0'
roadmap:
- id: PMAT-001
  item_type: task
  title: 'the pre-existing row'
  status: planned
";
    assert!(
        titles_changed(BASE, head).is_empty(),
        "a deleted row is `work delete`'s business, not a reused id"
    );
}

#[test]
fn an_unchanged_roadmap_reports_nothing() {
    assert!(titles_changed(BASE, BASE).is_empty());
}

#[test]
fn titles_are_read_per_row_not_from_the_nearest_title_line() {
    // A subtask carrying its own title must not be mistaken for the row's.
    let raw = "\
roadmap_version: '1.0'
roadmap:
- id: PMAT-001
  item_type: task
  title: 'the row title'
  subtasks:
  - id: PMAT-001a
    title: 'the subtask title'
";
    let titles = titles_by_id(raw);
    assert_eq!(
        titles.get("PMAT-001").map(String::as_str),
        Some("the row title")
    );
    assert_eq!(
        titles.get("PMAT-001a").map(String::as_str),
        Some("the subtask title"),
        "a subtask id is an id in use too, and carries its own title"
    );
}

// ── `work add --id`: the caller allocates, the allocator is not consulted.

fn roadmap_fixture() -> tempfile::TempDir {
    let dir = tempfile::TempDir::new().expect("fixture dir");
    let roadmaps = dir.path().join("docs/roadmaps");
    std::fs::create_dir_all(&roadmaps).expect("fixture roadmaps dir");
    std::fs::write(roadmaps.join("roadmap.yaml"), BASE).expect("fixture roadmap");
    dir
}

async fn add_with_id(
    project: &std::path::Path,
    title: &str,
    id: Option<&str>,
) -> anyhow::Result<()> {
    add_full(project, title, id, None).await
}

async fn add_from_issue(project: &std::path::Path, title: &str, issue: u64) -> anyhow::Result<()> {
    add_full(project, title, None, Some(issue)).await
}

async fn add_full(
    project: &std::path::Path,
    title: &str,
    id: Option<&str>,
    github_issue: Option<u64>,
) -> anyhow::Result<()> {
    crate::cli::handlers::work_handlers::handle_work_add(
        title.to_string(),
        None,
        crate::cli::commands::WorkPriority::Medium,
        None,
        Some(project.to_path_buf()),
        false,
        None,
        id.map(str::to_string),
        github_issue,
    )
    .await
}

#[tokio::test]
async fn an_explicit_id_is_minted_verbatim_and_the_allocator_is_not_consulted() {
    let dir = roadmap_fixture();
    // 1207 is far above max(id)+1, which is what an id from a collision-free
    // authority (a GitHub issue number) looks like.
    add_with_id(
        dir.path(),
        "allocated from the issue number",
        Some("PMAT-1207"),
    )
    .await
    .expect("an explicit free id must be accepted");
    let raw =
        std::fs::read_to_string(dir.path().join("docs/roadmaps/roadmap.yaml")).expect("read back");
    let titles = titles_by_id(&raw);
    assert_eq!(
        titles.get("PMAT-1207").map(String::as_str),
        Some("allocated from the issue number")
    );
    // The allocator would have said PMAT-003; it was not asked.
    assert!(
        !titles.contains_key("PMAT-003"),
        "allocator must not have run"
    );
}

#[tokio::test]
async fn an_explicit_id_already_in_use_is_refused_not_upserted() {
    let dir = roadmap_fixture();
    let before = std::fs::read_to_string(dir.path().join("docs/roadmaps/roadmap.yaml"))
        .expect("read before");

    let err = add_with_id(
        dir.path(),
        "a second meaning for an id in use",
        Some("PMAT-002"),
    )
    .await
    .expect_err("reusing an id must be refused");
    assert!(
        format!("{err}").contains("already in use"),
        "the refusal must say why: {err}"
    );

    let after =
        std::fs::read_to_string(dir.path().join("docs/roadmaps/roadmap.yaml")).expect("read after");
    assert_eq!(
        before, after,
        "a refused --id must leave the roadmap byte-identical — overwriting the row \
         is the very failure this flag exists to prevent"
    );
}

#[tokio::test]
async fn an_explicit_id_pushes_the_high_water_mark_so_the_allocator_cannot_reissue_it() {
    let dir = roadmap_fixture();
    add_with_id(dir.path(), "explicitly allocated", Some("PMAT-1207"))
        .await
        .expect("explicit id accepted");
    add_with_id(dir.path(), "then an allocated one", None)
        .await
        .expect("allocator still works");
    let raw =
        std::fs::read_to_string(dir.path().join("docs/roadmaps/roadmap.yaml")).expect("read back");
    let ids: Vec<String> = crate::services::roadmap_text::id_lines(&raw)
        .into_iter()
        .map(|(_, id)| id)
        .collect();
    let mut unique = ids.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(
        ids.len(),
        unique.len(),
        "the allocator reissued a spent id: {ids:?}"
    );
    assert_eq!(
        ids.iter().filter(|i| *i == "PMAT-1207").count(),
        1,
        "PMAT-1207 must appear exactly once, got {ids:?}"
    );
}

// ── Ask 1: the id space is collision-proof when the number comes from an
// authority both branches must go through.

#[tokio::test]
async fn the_id_is_derived_from_the_github_issue_number() {
    let dir = roadmap_fixture();
    add_from_issue(dir.path(), "the P0 CUDA row", 1065)
        .await
        .expect("an issue-derived id must be accepted");
    let raw =
        std::fs::read_to_string(dir.path().join("docs/roadmaps/roadmap.yaml")).expect("read back");
    assert_eq!(
        titles_by_id(&raw).get("PMAT-1065").map(String::as_str),
        Some("the P0 CUDA row"),
        "the id must be PMAT-<issue>, not the allocator's next number"
    );
    // The allocator would have said PMAT-003; it was not consulted.
    assert!(
        !titles_by_id(&raw).contains_key("PMAT-003"),
        "the sequential allocator must not run when an issue supplies the number"
    );
    assert!(
        raw.contains("github_issue: 1065"),
        "the ticket must record the issue that authorised its id, so the two stay joined"
    );
}

/// The property that makes this collision-PROOF rather than collision-unlikely.
///
/// The sequential allocator's defect is not that it is careless, it is that two
/// branches deterministically AGREE: both read the same roadmap, both compute
/// `max + 1`, both are right locally, and the merge deletes one ticket. An issue
/// number inverts that — GitHub hands out each number once, so two agents cannot
/// hold the same one, and no amount of what-can-this-branch-see is involved.
///
/// Simulated as two checkouts that cannot see each other at all: each starts from
/// the same base roadmap, and neither's write is visible to the other.
#[tokio::test]
async fn two_branches_that_cannot_see_each_other_still_cannot_collide() {
    let agent_a = roadmap_fixture();
    let agent_bse = roadmap_fixture();

    // Distinct issues, as GitHub would allocate them.
    add_from_issue(agent_a.path(), "L0-1a the CUDA row", 1065)
        .await
        .expect("agent A");
    add_from_issue(agent_bse.path(), "BSE-09b merge=union", 1074)
        .await
        .expect("agent BSE");

    let a =
        std::fs::read_to_string(agent_a.path().join("docs/roadmaps/roadmap.yaml")).expect("read A");
    let bse = std::fs::read_to_string(agent_bse.path().join("docs/roadmaps/roadmap.yaml"))
        .expect("read BSE");

    let new_in_a: Vec<String> = titles_by_id(&a)
        .into_keys()
        .filter(|id| !titles_by_id(BASE).contains_key(id))
        .collect();
    let new_in_bse: Vec<String> = titles_by_id(&bse)
        .into_keys()
        .filter(|id| !titles_by_id(BASE).contains_key(id))
        .collect();

    assert_eq!(new_in_a, vec!["PMAT-1065".to_string()]);
    assert_eq!(new_in_bse, vec!["PMAT-1074".to_string()]);
    assert!(
        new_in_a.iter().all(|id| !new_in_bse.contains(id)),
        "two isolated branches minted the same id: {new_in_a:?} vs {new_in_bse:?}"
    );

    // And the control: the SEQUENTIAL allocator, given the same two isolated
    // checkouts, does collide — which is the whole defect.
    let seq_a = roadmap_fixture();
    let seq_bse = roadmap_fixture();
    add_with_id(seq_a.path(), "L0-1a the CUDA row", None)
        .await
        .expect("agent A, allocator");
    add_with_id(seq_bse.path(), "BSE-09b merge=union", None)
        .await
        .expect("agent BSE, allocator");
    let sa = std::fs::read_to_string(seq_a.path().join("docs/roadmaps/roadmap.yaml"))
        .expect("read seq A");
    let sb = std::fs::read_to_string(seq_bse.path().join("docs/roadmaps/roadmap.yaml"))
        .expect("read seq BSE");
    let seq_new_a: Vec<String> = titles_by_id(&sa)
        .into_keys()
        .filter(|id| !titles_by_id(BASE).contains_key(id))
        .collect();
    let seq_new_b: Vec<String> = titles_by_id(&sb)
        .into_keys()
        .filter(|id| !titles_by_id(BASE).contains_key(id))
        .collect();
    assert_eq!(
        seq_new_a, seq_new_b,
        "the control must reproduce the defect: two isolated checkouts both mint the \
         same id from max+1. If this ever stops being equal, the allocator changed \
         and this test's premise needs re-reading."
    );

    // The two ids the issue-derived path produced are exactly what the
    // sequential path could not: different.
    assert_ne!(new_in_a, new_in_bse);
}

#[tokio::test]
async fn an_issue_number_whose_id_is_taken_is_refused_not_upserted() {
    let dir = roadmap_fixture();
    add_from_issue(dir.path(), "first claim on the id", 2)
        .await
        .expect_err("PMAT-002 already exists in the fixture, so this must be refused");
    let raw =
        std::fs::read_to_string(dir.path().join("docs/roadmaps/roadmap.yaml")).expect("read back");
    assert_eq!(
        titles_by_id(&raw).get("PMAT-002").map(String::as_str),
        Some("L0-1a the CUDA row that is in the merge queue"),
        "the existing row must be untouched"
    );
}

// ── The YAML shapes `titles_by_id` gets wrong (quorum on PMAT-713, 3/3 lanes).
//
// `id_lines` already tracks block scalars, because a reviewer's note quoting
// `id: PMAT-001` must not be read as a declaration. `titles_by_id` inherited none
// of that, so it reads a `title:` out of prose. These pin all three shapes.

#[test]
fn a_title_inside_a_block_scalar_is_prose_not_the_row_title() {
    let raw = "\
roadmap_version: '1.0'
roadmap:
- id: PMAT-001
  notes: |
    A reviewer wrote this, quoting another row:
    title: 'the WRONG title, it is inside a block scalar'
  title: 'the real title'
";
    assert_eq!(
        titles_by_id(raw).get("PMAT-001").map(String::as_str),
        Some("the real title"),
        "a title: inside a block scalar was read as the row's title"
    );
}

#[test]
fn a_flow_style_row_still_has_a_title() {
    let raw = "\
roadmap_version: '1.0'
roadmap:
- {id: PMAT-001, title: 'written inline'}
";
    assert_eq!(
        titles_by_id(raw).get("PMAT-001").map(String::as_str),
        Some("written inline"),
        "a flow-style row yielded no title, so titles_changed silently ignores that id"
    );
}

#[test]
fn a_subtask_before_the_parents_title_does_not_leave_the_parent_titleless() {
    let raw = "\
roadmap_version: '1.0'
roadmap:
- id: PMAT-001
  subtasks:
  - id: PMAT-001a
    title: 'the subtask title'
  title: 'the parent title'
";
    let titles = titles_by_id(raw);
    assert_eq!(
        titles.get("PMAT-001").map(String::as_str),
        Some("the parent title"),
        "the parent lost its title because a subtask was declared first"
    );
    assert_eq!(
        titles.get("PMAT-001a").map(String::as_str),
        Some("the subtask title")
    );
}

/// `max_id_number` reads `PMAT-7` and `PMAT-007` as the same integer 7, but
/// `duplicate_ids` compares the id STRINGS, so the two spellings coexist without
/// a duplicate error while the allocator counts past both. Two rows that are the
/// same ticket by number and different tickets by string is the exact ambiguity
/// this ticket exists to remove.
#[test]
fn two_spellings_of_one_number_are_a_duplicate() {
    let raw = "\
roadmap_version: '1.0'
roadmap:
- id: PMAT-7
  title: 'minted unpadded'
- id: PMAT-007
  title: 'minted padded'
";
    let dupes = crate::services::roadmap_text::duplicate_ids(raw);
    assert!(
        !dupes.is_empty(),
        "PMAT-7 and PMAT-007 are the same number and were not reported as a duplicate"
    );
}
