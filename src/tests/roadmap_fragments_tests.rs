#![cfg_attr(coverage_nightly, coverage(off))]
//! PMAT-1363 (#1363; paiml/aprender#3294, #3296) — `pmat work add/start/complete`
//! write `docs/roadmaps/entries/<id>.yaml` and never open `docs/roadmaps/roadmap.yaml`
//! for write. `roadmap.yaml` becomes a generated aggregate.
//!
//! THE DEFECT. Every consumer repo's pull requests contend on one file, however
//! disjoint their code is — Amdahl serial fraction 1 on the merge path. Measured in
//! paiml/aprender 2026-09-15: 146 PRs opened over 10 days against 88 merged, and of
//! the last 25 `merge_group` CI runs 7 succeeded and 15 were cancelled. PMAT-679
//! already made `work add` APPEND rather than rewrite, which bounds the damage to the
//! file's tail; it does not remove the contention, because two branches still append
//! to the same tail.
//!
//! A unique filename per ticket makes pull requests pairwise disjoint on the roadmap,
//! so the conflict rate is 0 by proof rather than by luck.
//!
//! IDEMPOTENCE IS LOAD-BEARING. The aggregate is regenerated post-merge on the
//! default branch, so a generator that is not a pure function of (base, fragments)
//! churns a commit on every merge. aprender's first cut read the live roadmap.yaml as
//! its base and raised `duplicate id` on its own output — it failed on run ONE.
//!
//! Registered from `cli/handlers/work_handlers/mod.rs`: `autotests = false` and
//! nothing reaches `src/tests/lib.rs`, so a file dropped in `src/tests/` without a
//! `mod` is never compiled and its silence would read as a pass.

use crate::services::roadmap_fragments::{aggregate, fragment_filename};

const BASE: &str = "roadmap_version: '1.0'\nroadmap:\n- id: PMAT-100\n  title: a\n- id: PMAT-300\n  title: c\n- id: LEGACY-THING\n  title: legacy\n";

fn ids(s: &str) -> Vec<String> {
    s.lines()
        .filter_map(|l| l.strip_prefix("- id: "))
        .map(|v| v.trim().trim_matches('\'').to_string())
        .collect()
}

#[test]
fn roadmap_fragments_zero_fragments_reproduce_the_base_byte_for_byte() {
    // The lossless row. If this is not byte-exact the migration is not a migration.
    assert_eq!(aggregate(BASE, &[]).unwrap(), BASE);
}

#[test]
fn roadmap_fragments_land_at_the_sorted_slot_numerically() {
    let out = aggregate(
        BASE,
        &[("PMAT-200".into(), "- id: PMAT-200\n  title: b\n".into())],
    )
    .unwrap();
    assert_eq!(
        ids(&out),
        ["PMAT-100", "PMAT-200", "PMAT-300", "LEGACY-THING"]
    );

    // 90 < 100 as numbers, "100" < "90" as strings. A string compare passes the
    // row above and fails this one, which is why both exist.
    let out = aggregate(
        BASE,
        &[("PMAT-90".into(), "- id: PMAT-90\n  title: n\n".into())],
    )
    .unwrap();
    assert_eq!(ids(&out)[0], "PMAT-90");
}

#[test]
fn roadmap_fragments_aggregate_is_idempotent_and_deterministic() {
    let f = vec![(
        "PMAT-200".to_string(),
        "- id: PMAT-200\n  title: b\n".to_string(),
    )];
    let once = aggregate(BASE, &f).unwrap();
    assert_eq!(
        aggregate(&once, &f).unwrap(),
        once,
        "post-merge regeneration must be a no-op"
    );

    let two = vec![
        (
            "PMAT-150".to_string(),
            "- id: PMAT-150\n  title: x\n".to_string(),
        ),
        (
            "PMAT-250".to_string(),
            "- id: PMAT-250\n  title: y\n".to_string(),
        ),
    ];
    let mut rev = two.clone();
    rev.reverse();
    assert_eq!(
        aggregate(BASE, &two).unwrap(),
        aggregate(BASE, &rev).unwrap(),
        "readdir order must not reach the output"
    );
}

#[test]
fn roadmap_fragments_supersede_a_base_row_rather_than_colliding() {
    // This is what makes entries/ the ONLY edit path for an existing id, which is
    // in turn what lets the gate forbid every write to roadmap.yaml. Refusing it as
    // a duplicate re-opens the base as an edit surface.
    let out = aggregate(
        BASE,
        &[(
            "PMAT-100".into(),
            "- id: PMAT-100\n  title: superseded\n".into(),
        )],
    )
    .unwrap();
    assert_eq!(ids(&out), ["PMAT-100", "PMAT-300", "LEGACY-THING"]);
    assert!(out.contains("superseded"));
}

#[test]
fn roadmap_fragments_refuse_an_id_that_cannot_be_a_filename() {
    // Real data, paiml/aprender's roadmap: of 879 entries, 52 cannot be filenames.
    // One id is literally `Push completed work to origin/main (5 commits)` — it
    // contains a path separator.
    assert!(fragment_filename("PMAT-1363").is_some());
    assert!(fragment_filename("APR-FORMAT-002").is_some());
    assert!(fragment_filename("Push completed work to origin/main (5 commits)").is_none());
    assert!(fragment_filename("../escape").is_none());
    assert!(fragment_filename("").is_none());
}

/// Real data, when it is reachable. A fixture proves the rule; 879 real entries
/// prove the PARSER — block scalars whose bodies quote an id, flow-style rows,
/// quoted ids, and a `created: &id001` anchor aliased by 17 later entries.
///
/// NOT-RUN when the file is absent, said out loud: a result that was not measured
/// must never read as a pass.
#[test]
fn roadmap_fragments_round_trip_a_real_roadmap_byte_for_byte() {
    // THIS REPO's own roadmap, never another checkout's working tree. An earlier
    // cut reached into /home/noah/src/aprender and silently measured a stale branch
    // — 820 entries where that repo's origin/main carries 879. A test whose input
    // depends on which branch some unrelated checkout happens to be on measures
    // nothing repeatable, and in clean-room CI would not be there at all.
    let path = "docs/roadmaps/roadmap.yaml";
    let Ok(raw) = std::fs::read_to_string(path) else {
        eprintln!("NOT-RUN: {path} absent — the real-data row measured nothing");
        return;
    };

    let (preamble, entries) = crate::services::roadmap_fragments::split_entries(&raw);
    assert!(
        entries.len() > 100,
        "{path}: split found {} entries — a parser that finds none would pass every other row",
        entries.len()
    );

    // Byte-exact reassembly. This is the migration's whole safety argument.
    let rebuilt: String = preamble + &entries.iter().map(|(_, b)| b.as_str()).collect::<String>();
    assert_eq!(rebuilt, raw, "{path}: split/join is not byte-exact");

    // And zero fragments must be a no-op over the same real file.
    assert_eq!(
        aggregate(&raw, &[]).unwrap(),
        raw,
        "{path}: aggregate(x, []) != x"
    );

    eprintln!(
        "real-data row: {path}, {} entries, byte-exact",
        entries.len()
    );
}

// ---------------------------------------------------------------- the wiring
//
// The contract predicate for PMAT-1363, stated so it holds however many write
// sites there turn out to be: after the command, `git diff --name-only` excludes
// docs/roadmaps/roadmap.yaml. At this level that is "the file's bytes did not
// change", which is the same claim without needing a git repo.

use crate::services::roadmap_service::RoadmapService;

fn fixture(with_entries: bool) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let docs = dir.path().join("docs/roadmaps");
    std::fs::create_dir_all(&docs).expect("mkdir");
    if with_entries {
        std::fs::create_dir_all(docs.join("entries")).expect("mkdir entries");
    }
    let path = docs.join("roadmap.yaml");
    std::fs::write(
        &path,
        "roadmap_version: '1.0'\ngithub_enabled: false\ngithub_repo: null\nroadmap:\n- id: PMAT-001\n  title: first\n  status: planned\n",
    )
    .expect("write");
    (dir, path)
}

fn an_item(id: String) -> crate::models::roadmap::RoadmapItem {
    // Built from YAML like the neighbouring suite does: RoadmapItem has no
    // Default, and hand-filling it would drift from the real shape.
    let block = format!("- id: {id}\n  title: wired\n  status: planned\n");
    let mut items: Vec<crate::models::roadmap::RoadmapItem> =
        serde_yaml_ng::from_str(&block).expect("the block must parse as one item");
    items.pop().expect("exactly one item")
}

#[test]
fn roadmap_fragments_work_add_writes_a_fragment_and_leaves_the_aggregate_untouched() {
    let (_dir, path) = fixture(true);
    let before = std::fs::read_to_string(&path).expect("read");

    let svc = RoadmapService::new(&path);
    let id = svc.add_item_with_next_id(an_item).expect("add");

    let after = std::fs::read_to_string(&path).expect("read");
    assert_eq!(
        before, after,
        "docs/roadmaps/roadmap.yaml must be byte-identical — it is a GENERATED aggregate"
    );

    let frag = path
        .parent()
        .expect("parent")
        .join("entries")
        .join(format!("{id}.yaml"));
    assert!(frag.exists(), "expected a fragment at {}", frag.display());
    let body = std::fs::read_to_string(&frag).expect("read fragment");
    assert!(
        body.contains(&format!("- id: {id}")),
        "fragment must carry its own row: {body}"
    );
}

#[test]
fn roadmap_fragments_work_add_still_appends_when_the_repo_has_not_migrated() {
    // OPT-IN ON entries/. pmat ships to every consumer repo; a repo that has not
    // created docs/roadmaps/entries/ keeps the PMAT-679 append behaviour exactly.
    // Changing that silently on a version bump would stop their roadmap updating
    // with nothing to read as an error.
    let (_dir, path) = fixture(false);
    let before = std::fs::read_to_string(&path).expect("read");

    let svc = RoadmapService::new(&path);
    let id = svc.add_item_with_next_id(an_item).expect("add");

    let after = std::fs::read_to_string(&path).expect("read");
    assert_ne!(
        before, after,
        "an un-migrated repo still appends to roadmap.yaml"
    );
    assert!(after.contains(&format!("- id: {id}")));
    assert!(
        after.starts_with(&before),
        "and the append must still preserve every prior byte (PMAT-679)"
    );
}

#[test]
fn roadmap_fragments_work_add_with_a_caller_supplied_id_also_writes_a_fragment() {
    // `pmat work add --github-issue N` takes add_item_with_id, NOT the allocator
    // path. Patching only the allocator would have made the migrated behaviour
    // depend on which flag the caller used — green on one seam, silently
    // appending on the other. This is the seam the ticket for this very change
    // was filed through.
    let (_dir, path) = fixture(true);
    let before = std::fs::read_to_string(&path).expect("read");

    let svc = RoadmapService::new(&path);
    svc.add_item_with_id("PMAT-1363", an_item)
        .expect("add with id");

    assert_eq!(
        before,
        std::fs::read_to_string(&path).expect("read"),
        "the caller-supplied-id path must leave the aggregate byte-identical too"
    );
    let frag = path
        .parent()
        .expect("parent")
        .join("entries/PMAT-1363.yaml");
    assert!(frag.exists(), "expected {}", frag.display());
}

#[test]
fn roadmap_fragments_refuse_to_write_a_fragment_for_an_unfilenameable_id() {
    // The refusal is not theoretical: it is what stops a sanitised name silently
    // breaking trailer-to-filename parity.
    let dir = tempfile::TempDir::new().expect("tempdir");
    let err = crate::services::roadmap_fragments::write_fragment(
        dir.path(),
        "Push completed work to origin/main (5 commits)",
        "- id: x\n",
    )
    .expect_err("an id with a path separator cannot be a fragment");
    assert!(err.contains("cannot be a filename"), "{err}");
    assert_eq!(
        std::fs::read_dir(dir.path()).expect("readdir").count(),
        0,
        "a refused fragment must write nothing at all"
    );
}
