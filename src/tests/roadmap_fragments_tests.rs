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
