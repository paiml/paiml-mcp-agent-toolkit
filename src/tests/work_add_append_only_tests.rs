#![cfg_attr(coverage_nightly, coverage(off))]
//! PMAT-679 (#1193, #1169 second half; aprender #2874) — `pmat work add` must
//! APPEND the row it mints, and `pmat work edit` must replace ONLY the row it
//! edits. Every byte neither command touched stays identical.
//!
//! The defect measured on 3.38.0: both commands round-tripped the whole
//! roadmap through serde — `add_item_with_next_id` ended in
//! `write_roadmap_unlocked(&roadmap)` and `handle_work_edit` in a whole-model
//! save — so adding one ticket rewrote all 2,532 lines of aprender's roadmap.
//! Two consequences, both observed: every concurrent branch conflicted on the
//! roadmap, and every byte the serde model does not carry (a comment, an
//! unknown key, a flow-style row, the choice of block scalar) was silently
//! reformatted or dropped.
//!
//! These tests read the BYTES, not the parsed model: a model comparison is
//! exactly the check that cannot see this defect.
//!
//! Registered from `cli/handlers/work_handlers/mod.rs` — `autotests = false`
//! and nothing reaches `src/tests/lib.rs`, so a file dropped in `src/tests/`
//! without a `mod` is never compiled (`docs/status/orphan-files-ledger.md`)
//! and its silence would read as a pass.

use crate::cli::commands::WorkPriority;
use crate::models::roadmap::RoadmapItem;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Everything the serde model cannot carry, in one roadmap that parses:
/// a leading comment, an unknown key (`zeta`), a flow-style row, a block
/// scalar whose body quotes an id, and a trailing comment.
const LOSSY_FIXTURE: &str = "# a leading comment nobody may rewrite
roadmap_version: '1.0'
github_enabled: false
github_repo: null
roadmap:
- id: PMAT-001
  title: first
  status: planned
  zeta: 1
- {id: PMAT-002, title: b, status: planned}
- id: PMAT-003
  title: third
  status: planned
  notes: |
    the reviewer wrote id: PMAT-900
    and a second body line
# end
";

/// The empty-sequence spelling `pmat work init` writes. `roadmap: []` is the
/// one existing line an append may rewrite, because a sequence element cannot
/// follow a flow-style empty sequence.
const EMPTY_SEQUENCE_FIXTURE: &str = "roadmap_version: '1.0'
github_enabled: false
github_repo: null
roadmap: []
";

fn roadmap_path(project: &Path) -> PathBuf {
    project.join("docs/roadmaps/roadmap.yaml")
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

fn roadmap_text(project: &Path) -> String {
    std::fs::read_to_string(roadmap_path(project)).expect("roadmap.yaml must be readable")
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

/// How many lines the write moved: positional differences plus the length
/// change. The number the defect makes large — 2,532 on aprender — and that
/// an append keeps equal to the size of the row it appended.
fn changed_line_count(old: &str, new: &str) -> usize {
    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();
    let common = old_lines.len().min(new_lines.len());
    let differing = (0..common)
        .filter(|&index| old_lines[index] != new_lines[index])
        .count();
    differing + new_lines.len().abs_diff(old_lines.len())
}

/// The one item a block of raw YAML declares.
fn parse_one_item(block: &str) -> RoadmapItem {
    let parsed = serde_yaml_ng::from_str::<Vec<RoadmapItem>>(block);
    assert!(
        parsed.is_ok(),
        "the block must parse as one item: {parsed:?}\n{block}"
    );
    let mut items = parsed.expect("checked Ok immediately above");
    assert_eq!(
        items.len(),
        1,
        "the append must add exactly one row: {block}"
    );
    items.remove(0)
}

/// The whole roadmap a file declares — asserted, not unwrapped, so a failure
/// prints the bytes that failed to parse.
fn parse_roadmap(text: &str) -> crate::models::roadmap::Roadmap {
    let parsed = serde_yaml_ng::from_str::<crate::models::roadmap::Roadmap>(text);
    assert!(parsed.is_ok(), "the roadmap must parse: {parsed:?}\n{text}");
    parsed.expect("checked Ok immediately above")
}

// ── A1: add appends, and touches nothing else ───────────────────────────────

/// The whole defect in one test. After `work add`, the file is the old file
/// plus one rendered row: every comment, the unknown key, the flow-style row
/// and the block scalar are byte-identical, the result still strict-parses,
/// and `work validate` is still happy.
#[tokio::test]
async fn work_add_append_only_add_appends_the_row_and_rewrites_nothing() {
    let project = project_with_roadmap(LOSSY_FIXTURE);
    let old = roadmap_text(project.path());

    add(project.path(), "new")
        .await
        .expect("a clean roadmap must accept an add");

    let new = roadmap_text(project.path());
    assert!(
        new.starts_with(&old),
        "an add must leave every existing byte identical.\nbefore:\n{old}\nafter:\n{new}"
    );

    let appended = &new[old.len()..];
    let item = parse_one_item(appended);
    assert_eq!(item.id, "PMAT-004", "the minted id is one past PMAT-003");
    assert_eq!(item.title, "new");

    // The 2,532-line class is dead: the only lines that moved are the row's.
    assert_eq!(
        changed_line_count(&old, &new),
        appended.lines().count(),
        "an add must change exactly the lines of the row it appended.\nafter:\n{new}"
    );

    // And what it wrote is still a roadmap both readers accept.
    let reloaded = parse_roadmap(&new);
    assert_eq!(reloaded.roadmap.len(), 4);
    crate::cli::handlers::work_handlers::handle_work_validate(
        Some(project.path().to_path_buf()),
        false,
        false,
    )
    .await
    .expect("what `add` wrote must validate");
}

// ── A2: the one existing line an append may rewrite ─────────────────────────

/// `roadmap: []` is a flow-style empty sequence; a block sequence element
/// cannot follow it. That one line becomes `roadmap:`, the row follows it, and
/// the result parses.
#[tokio::test]
async fn work_add_append_only_add_opens_an_empty_flow_sequence() {
    let project = project_with_roadmap(EMPTY_SEQUENCE_FIXTURE);

    add(project.path(), "the first ticket")
        .await
        .expect("an empty roadmap must accept an add");

    let new = roadmap_text(project.path());
    assert!(
        new.contains("roadmap:\n- id: PMAT-001"),
        "the empty sequence must open into a block sequence:\n{new}"
    );
    assert!(
        !new.contains("roadmap: []"),
        "the empty flow sequence must be gone:\n{new}"
    );
    let reloaded = parse_roadmap(&new);
    assert_eq!(reloaded.roadmap.len(), 1);
    assert_eq!(reloaded.roadmap[0].title, "the first ticket");
    // The three lines above `roadmap:` are untouched.
    assert!(new.starts_with("roadmap_version: '1.0'\ngithub_enabled: false\ngithub_repo: null\n"));
}

// ── A3: edit replaces exactly one block ─────────────────────────────────────

/// `work edit` rewrote the whole file too. Editing PMAT-001's title must leave
/// every other line — including the comments, the flow row and the block
/// scalar of rows it did not touch — present verbatim and in order.
#[tokio::test]
async fn work_add_append_only_edit_replaces_only_the_edited_block() {
    let project = project_with_roadmap(LOSSY_FIXTURE);
    let old = roadmap_text(project.path());

    edit_title(project.path(), "PMAT-001", "an edited title")
        .await
        .expect("a clean roadmap must accept an edit");

    let new = roadmap_text(project.path());
    // Every line of the fixture except PMAT-001's own block survives verbatim,
    // in order. PMAT-001's block is lines 6..9 (`- id: PMAT-001` .. `zeta: 1`).
    let untouched: Vec<&str> = old
        .lines()
        .filter(|line| {
            !matches!(
                line.trim(),
                "- id: PMAT-001" | "title: first" | "status: planned" | "zeta: 1"
            )
        })
        .collect();
    let mut remaining = new.lines();
    for line in &untouched {
        assert!(
            remaining.any(|candidate| candidate == *line),
            "an edit must leave {line:?} verbatim and in order:\n{new}"
        );
    }
    assert!(
        new.contains("- {id: PMAT-002, title: b, status: planned}"),
        "the flow-style row must be byte-identical:\n{new}"
    );
    assert!(
        new.contains("    the reviewer wrote id: PMAT-900\n"),
        "the block scalar body must be byte-identical:\n{new}"
    );

    let reloaded = parse_roadmap(&new);
    assert_eq!(reloaded.roadmap.len(), 3, "no row gained or lost:\n{new}");
    assert_eq!(reloaded.roadmap[0].id, "PMAT-001");
    assert_eq!(reloaded.roadmap[0].title, "an edited title");
    assert_eq!(reloaded.roadmap[1].id, "PMAT-002", "order is preserved");
    assert_eq!(reloaded.roadmap[2].title, "third");
}
