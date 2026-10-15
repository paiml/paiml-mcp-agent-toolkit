//! PMAT #1370 — `pmat roadmap sync` as the one writer of `docs/roadmaps/roadmap.yaml`.
//! `cargo test --lib -- roadmap_sync_route` runs them.

use super::*;
use crate::services::roadmap_service::RoadmapService;

const BASE: &str = "roadmap_version: '1.0'\ngithub_enabled: false\ngithub_repo: null\nroadmap:\n- id: PMAT-001\n  title: first\n  status: planned\n- id: PMAT-005\n  title: fifth\n  status: planned\n";

fn project(with_entries: bool) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let docs = dir.path().join("docs/roadmaps");
    std::fs::create_dir_all(&docs).expect("mkdir");
    if with_entries {
        std::fs::create_dir_all(docs.join("entries")).expect("mkdir entries");
    }
    let roadmap = docs.join("roadmap.yaml");
    std::fs::write(&roadmap, BASE).expect("write");
    (dir, roadmap)
}

fn add_ticket(roadmap: &Path) -> String {
    RoadmapService::new(roadmap)
        .add_item_with_next_id(|id| {
            let block = format!("- id: {id}\n  title: routed\n  status: planned\n");
            let mut items: Vec<crate::models::roadmap::RoadmapItem> =
                serde_yaml_ng::from_str(&block).expect("one item");
            items.pop().expect("one item")
        })
        .expect("add")
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).expect("read")
}

#[test]
fn roadmap_sync_route_targets_fragments_only_when_entries_exists() {
    let (with, roadmap) = project(true);
    assert_eq!(sync_target(with.path()), SyncTarget::Fragments(roadmap));
    let (without, _) = project(false);
    assert_eq!(sync_target(without.path()), SyncTarget::WorkStore);
}

#[test]
fn roadmap_sync_route_add_then_check_red_then_sync_then_check_green() {
    let (dir, roadmap) = project(true);
    // (a) the fragment writer leaves the aggregate byte-identical.
    let id = add_ticket(&roadmap);
    assert_eq!(read(&roadmap), BASE, "work add must not touch roadmap.yaml");

    // The aggregate now lags its fragments: --check is red and names the row.
    let red = run_fragment_sync(&roadmap, None, SyncMode::Check);
    assert_eq!(red.code, 1, "{}", red.stderr);
    assert!(red.stderr.contains(&id), "{}", red.stderr);

    // (b) sync writes it, and a second sync is a byte-identical no-op.
    let first = run_fragment_sync(&roadmap, None, SyncMode::Write);
    assert_eq!(first.code, 0, "{}", first.stderr);
    let once = read(&roadmap);
    assert!(once.contains(&format!("- id: {id}\n")), "{once}");
    assert_eq!(run_fragment_sync(&roadmap, None, SyncMode::Write).code, 0);
    assert_eq!(read(&roadmap), once, "sync must be deterministic");

    // (c) green after sync; a hand edit to a fragment-backed row turns it red,
    // and the next sync puts the fragment's bytes back.
    let green = run_fragment_sync(&roadmap, None, SyncMode::Check);
    assert_eq!(green.code, 0, "{}", green.stderr);
    std::fs::write(
        &roadmap,
        once.replace("title: routed", "title: hand-edited"),
    )
    .expect("hand edit");
    let edited = run_fragment_sync(&roadmap, None, SyncMode::Check);
    assert_eq!(edited.code, 1, "{}", edited.stderr);
    assert!(edited.stderr.contains(&id), "{}", edited.stderr);
    assert_eq!(run_fragment_sync(&roadmap, None, SyncMode::Write).code, 0);
    assert_eq!(read(&roadmap), once, "sync restores the fragment's row");
    assert_eq!(run_fragment_sync(&roadmap, None, SyncMode::Check).code, 0);
    drop(dir);
}

#[test]
fn roadmap_sync_route_dry_run_prints_and_writes_nothing() {
    let (_dir, roadmap) = project(true);
    let id = add_ticket(&roadmap);
    let printed = run_fragment_sync(&roadmap, None, SyncMode::DryRun);
    assert_eq!(printed.code, 0, "{}", printed.stderr);
    assert!(printed.stdout.contains(&format!("- id: {id}\n")));
    assert_eq!(read(&roadmap), BASE);
}

#[test]
fn roadmap_sync_route_refuses_a_snapshot_pin_in_fragment_mode() {
    let (_dir, roadmap) = project(true);
    let refused = run_fragment_sync(&roadmap, Some("abc123"), SyncMode::Write);
    assert_eq!(refused.code, 2, "{}", refused.stderr);
    assert_eq!(read(&roadmap), BASE);
}

#[test]
fn roadmap_sync_route_work_store_check_follows_the_rendered_file() {
    let (dir, _) = project(false);
    // Nothing rendered yet: an unreadable input is never a pass.
    assert_eq!(check_work_store(dir.path(), None).code, 2);

    super::super::sync::handle_roadmap_sync(dir.path(), None, false, "2026-09-25T00:00:00Z")
        .expect("render");
    let green = check_work_store(dir.path(), None);
    assert_eq!(green.code, 0, "{}", green.stderr);

    // The roadmap moves on; the rendered file is now stale.
    add_ticket(&dir.path().join(FRAGMENT_ROADMAP));
    let red = check_work_store(dir.path(), None);
    assert_eq!(red.code, 1, "{}", red.stderr);
}

#[test]
fn roadmap_sync_route_first_differing_line_is_one_based() {
    assert_eq!(first_differing_line("a\nb\n", "a\nb\n"), None);
    assert_eq!(first_differing_line("a\nb\n", "a\nc\n"), Some(2));
    assert_eq!(first_differing_line("a\n", "a\nb\n"), Some(2));
    assert_eq!(
        recorded_field("x: 1\ngenerated_at: \"T\"  # c\n", "generated_at:"),
        Some("T".to_string())
    );
    assert_eq!(recorded_field("x: 1\n", "gh_snapshot:"), None);
}

#[test]
fn roadmap_sync_route_check_cannot_see_an_edit_to_a_base_only_row() {
    // Stated, not implied: roadmap.yaml is its own base, so an edit to a row that
    // has no fragment IS the new base and `--check` passes it. Refusing that edit
    // in a pull request is the PR-diff guard's job (the aggregate is never touched
    // in a PR), not this check's — it would need the default branch's copy.
    let (_dir, roadmap) = project(true);
    std::fs::write(&roadmap, BASE.replace("title: first", "title: hand-edited")).expect("edit");
    assert_eq!(run_fragment_sync(&roadmap, None, SyncMode::Check).code, 0);
}
