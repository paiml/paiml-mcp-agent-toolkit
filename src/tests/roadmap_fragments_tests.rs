#![cfg_attr(coverage_nightly, coverage(off))]
//! PMAT-1363 (#1363; paiml/aprender#3294, #3296) — the aggregator itself.
//!
//! `docs/roadmaps/roadmap.yaml` becomes a GENERATED aggregate of
//! `docs/roadmaps/entries/<id>.yaml`, so pull requests are pairwise disjoint on the
//! roadmap. Measured in paiml/aprender 2026-09-15: 146 PRs opened over 10 days against
//! 88 merged, and of the last 25 `merge_group` CI runs 7 succeeded and 15 were
//! cancelled, because every PR wrote one shared file.
//!
//! This file is the PORT'S case table. Rows `case_01`..`case_14` are the fourteen rows
//! of `python3 scripts/lib/roadmap_fragments.py --selftest` in paiml/aprender, in
//! order, with the same fixture and the same expectations — so "the Rust aggregator
//! agrees with the reference" is a claim a failing row can refute, not a sentence.
//! Rows after them pin the parser against the reference's regexes, and the
//! divergences the module header lists.
//!
//! Registered from `cli/handlers/work_handlers/mod.rs`: `autotests = false` and
//! nothing reaches `src/tests/lib.rs`, so a file dropped in `src/tests/` without a
//! `mod` is never compiled and its silence would read as a pass.

use crate::services::roadmap_fragments::{
    aggregate, aggregate_with, census, check_fragment, first_difference, fragment_filename,
    insertion_index, is_filename_safe, is_ticket_id, parse_id, parse_id_value, split_entries,
    FragmentError, MAX_ID_LEN,
};

/// The reference selftest's `BASE`, byte for byte.
const BASE: &str = "roadmap_version: '1.0'\nroadmap:\n- id: PMAT-100\n  title: a\n- id: PMAT-300\n  title: c\n- id: LEGACY-THING\n  title: legacy\n";

fn ids(text: &str) -> Vec<String> {
    split_entries(text)
        .1
        .into_iter()
        .map(|(id, _)| id)
        .collect()
}

fn frag(id: &str, body: &str) -> (String, String) {
    (id.to_string(), format!("- id: {id}\n  {body}\n"))
}

// ------------------------------------------------------------ the case table

#[test]
fn roadmap_fragments_case_01_no_fragments_reproduce_the_base_byte_for_byte() {
    // THE LOSSLESS ROW. If this is not byte-exact the migration is not a migration.
    assert_eq!(
        aggregate(BASE, &[]).expect("aggregate accepts these fragments"),
        BASE
    );
}

#[test]
fn roadmap_fragments_case_02_a_numeral_between_two_lands_at_its_sorted_slot() {
    let out = aggregate(BASE, &[frag("PMAT-200", "title: b")])
        .expect("aggregate accepts these fragments");
    assert_eq!(
        ids(&out),
        ["PMAT-100", "PMAT-200", "PMAT-300", "LEGACY-THING"]
    );
}

#[test]
fn roadmap_fragments_case_03_a_numeral_after_all_of_its_prefix_is_appended() {
    let out = aggregate(BASE, &[frag("PMAT-400", "title: d")])
        .expect("aggregate accepts these fragments");
    assert_eq!(ids(&out).last().map(String::as_str), Some("PMAT-400"));
}

#[test]
fn roadmap_fragments_case_04_an_unseen_prefix_is_appended() {
    let out = aggregate(BASE, &[frag("APEX-1", "title: new prefix")])
        .expect("aggregate accepts these fragments");
    assert_eq!(ids(&out).last().map(String::as_str), Some("APEX-1"));
}

#[test]
fn roadmap_fragments_case_05_a_legacy_id_is_appended() {
    let out = aggregate(BASE, &[frag("FREEFORM", "title: legacy")])
        .expect("aggregate accepts these fragments");
    assert_eq!(ids(&out).last().map(String::as_str), Some("FREEFORM"));
}

#[test]
fn roadmap_fragments_case_06_numerals_compare_numerically_not_as_strings() {
    // 90 < 100 as numbers, "100" < "90" as strings. A string compare passes case 02
    // and fails this one, which is why both exist.
    let out = aggregate(BASE, &[frag("PMAT-90", "title: ninety")])
        .expect("aggregate accepts these fragments");
    assert_eq!(ids(&out)[0], "PMAT-90");
}

#[test]
fn roadmap_fragments_case_07_a_fragment_supersedes_the_base_row_of_its_id() {
    // This is what makes entries/ the ONLY edit path for an existing id, which is in
    // turn what lets a gate forbid every write to roadmap.yaml. Refusing it as a
    // duplicate re-opens the base as an edit surface.
    let out = aggregate(BASE, &[frag("PMAT-100", "title: superseded")])
        .expect("aggregate accepts these fragments");
    assert_eq!(ids(&out), ["PMAT-100", "PMAT-300", "LEGACY-THING"]);
    assert!(
        out.contains("superseded") && !out.contains("title: a\n"),
        "{out}"
    );
}

#[test]
fn roadmap_fragments_case_08_two_fragments_cannot_claim_one_id() {
    let err = aggregate(BASE, &[frag("PMAT-9", "t: a"), frag("PMAT-9", "t: b")])
        .expect_err("a duplicate id among fragments must be refused");
    assert_eq!(err, FragmentError::DuplicateAmongFragments("PMAT-9".into()));
    assert!(
        err.to_string().contains("duplicate id among fragments"),
        "{err}"
    );
}

#[test]
fn roadmap_fragments_case_09_aggregate_is_idempotent() {
    // The post-merge aggregation runs on the default branch; a generator that is not
    // a pure function of (base, fragments) churns a commit on every merge.
    let f = vec![frag("PMAT-200", "title: b")];
    let once = aggregate(BASE, &f).expect("aggregate accepts these fragments");
    assert_eq!(
        aggregate(&once, &f).expect("aggregate accepts these fragments"),
        once
    );
}

#[test]
fn roadmap_fragments_case_10_fragment_input_order_does_not_change_the_bytes() {
    let two = vec![frag("PMAT-150", "t: x"), frag("PMAT-250", "t: y")];
    let reversed: Vec<_> = two.iter().rev().cloned().collect();
    assert_eq!(
        aggregate(BASE, &two).expect("aggregate accepts these fragments"),
        aggregate(BASE, &reversed).expect("aggregate accepts these fragments")
    );

    // Stronger than the reference's row, and deliberately: two APPENDED ids are
    // the case where arrival order would otherwise reach the output.
    let legacy = vec![frag("ZULU", "t: z"), frag("ALPHA", "t: a")];
    let legacy_rev: Vec<_> = legacy.iter().rev().cloned().collect();
    assert_eq!(
        aggregate(BASE, &legacy).expect("aggregate accepts these fragments"),
        aggregate(BASE, &legacy_rev).expect("aggregate accepts these fragments")
    );
}

#[test]
fn roadmap_fragments_case_11_two_fragments_each_land_at_their_own_slot() {
    let out = aggregate(
        BASE,
        &[frag("PMAT-150", "title: x"), frag("PMAT-250", "title: y")],
    )
    .expect("aggregate accepts these fragments");
    assert_eq!(
        ids(&out),
        [
            "PMAT-100",
            "PMAT-150",
            "PMAT-250",
            "PMAT-300",
            "LEGACY-THING"
        ]
    );
}

#[test]
fn roadmap_fragments_case_12_mutation_append_only_placement_is_caught() {
    // A guard whose failure mode is untested is decoration (#3294). The reference
    // monkeypatches insertion_index; here the placement rule is injected, so the
    // REAL aggregation runs with the mutant and case 02's expectation must fail.
    let fragments = [frag("PMAT-200", "title: b")];
    let mutated = aggregate_with(BASE, &fragments, |entries, _| entries.len())
        .expect("aggregate accepts these fragments");
    assert_ne!(
        ids(&mutated),
        ["PMAT-100", "PMAT-200", "PMAT-300", "LEGACY-THING"],
        "mutant survived — case 02 proves nothing"
    );
    let real = aggregate_with(BASE, &fragments, insertion_index)
        .expect("aggregate accepts these fragments");
    assert_eq!(
        real,
        aggregate(BASE, &fragments).expect("aggregate accepts these fragments")
    );
}

#[test]
fn roadmap_fragments_case_13_census_accounts_for_every_row_of_a_real_roadmap() {
    // The reference runs this over aprender's live roadmap. Here it runs over THIS
    // repository's, which CI always has: an input that depends on which branch some
    // unrelated checkout sits on measures nothing repeatable.
    let raw = std::fs::read_to_string("docs/roadmaps/roadmap.yaml")
        .expect("this repository's roadmap is committed; a missing input is a failure, not a skip");
    let c = census(&raw);
    assert!(c.total() > 100, "census found {} rows", c.total());
    assert_eq!(c.total(), ids(&raw).len(), "every row counted exactly once");
    eprintln!(
        "census: fragmentable={} safe-legacy={} NOT-filename-safe={} total={}",
        c.safe_prefix_n.len(),
        c.safe_legacy.len(),
        c.not_filename_safe(),
        c.total()
    );
}

#[test]
fn roadmap_fragments_case_14_a_real_id_contains_a_path_separator() {
    // Real rows from paiml/aprender's roadmap at origin/main: one id is literally a
    // commit message with a slash in it. Filename safety is load-bearing, not
    // defensive — relax it to admit '/' and this id becomes a would-be path.
    let raw = "roadmap:\n\
               - id: PMAT-3294\n  title: a\n\
               - id: ROADMAP-RECONCILE-2026-07-04\n  title: b\n\
               - id: Push completed work to origin/main (5 commits)\n  title: c\n\
               - id: PMAT-12 (superseded)\n  title: d\n";
    let c = census(raw);
    assert_eq!(c.safe_prefix_n, ["PMAT-3294"]);
    assert_eq!(c.safe_legacy, ["ROADMAP-RECONCILE-2026-07-04"]);
    assert_eq!(
        c.unsafe_legacy,
        ["Push completed work to origin/main (5 commits)"]
    );
    assert_eq!(c.unsafe_prefix_n, ["PMAT-12 (superseded)"]);
    assert!(c.unsafe_legacy.iter().any(|id| id.contains('/')));
    assert!(fragment_filename("Push completed work to origin/main (5 commits)").is_none());
    assert!(fragment_filename("../escape").is_none());
    assert!(fragment_filename("").is_none());
}

// ------------------------------------------------- parity with the reference

#[test]
fn roadmap_fragments_parse_id_is_the_reference_regex() {
    // ID_RE = ^([A-Za-z][A-Za-z0-9_]*)-([0-9]+)([^0-9A-Za-z_].*)?$
    let parsed = |id: &str| parse_id(id).map(|(p, n)| (p.to_string(), format!("{n:?}")));
    let numbered = |p: &str, n: &str| Some((p.to_string(), format!("Numeral({n:?})")));
    assert_eq!(parsed("PMAT-1363"), numbered("PMAT", "1363"));
    assert_eq!(parsed("A_B9-007"), numbered("A_B9", "7"));
    assert_eq!(parsed("PMAT-0"), numbered("PMAT", "0"));
    assert_eq!(parsed("PMAT-12 (notes)"), numbered("PMAT", "12"));
    assert_eq!(parsed("PMAT-12-3"), numbered("PMAT", "12"));
    assert_eq!(
        parsed("PMAT-12é"),
        numbered("PMAT", "12"),
        "é is not in [0-9A-Za-z_]"
    );
    // The prefix runs to the FIRST dash. An rsplit on the last dash reads this as
    // prefix `ROADMAP-RECONCILE-2026-07` numbered 4 and sorts it among strangers.
    assert_eq!(parsed("ROADMAP-RECONCILE-2026-07-04"), None);
    assert_eq!(parsed("PMAT-12a"), None);
    assert_eq!(parsed("PMAT-12_"), None);
    assert_eq!(parsed("1PMAT-3"), None);
    assert_eq!(parsed("PMAT-"), None);
    assert_eq!(parsed("PMAT"), None);
    assert_eq!(parsed("A.B-1"), None);
    assert_eq!(parsed(""), None);
}

#[test]
fn roadmap_fragments_numerals_have_no_ceiling() {
    // Python's int() has none, so a u64 would disagree with the reference here.
    let big = "PMAT-123456789012345678901234567890";
    let bigger = "PMAT-123456789012345678901234567891";
    let base = format!("roadmap:\n- id: {bigger}\n  t: b\n");
    let out = aggregate(&base, &[frag(big, "t: a")]).expect("aggregate accepts these fragments");
    assert_eq!(ids(&out), [big, bigger]);
    let (_, zero_padded) = parse_id("PMAT-0090").expect("a PREFIX-N id");
    let (_, plain) = parse_id("PMAT-90").expect("a PREFIX-N id");
    assert_eq!(zero_padded, plain);
}

#[test]
fn roadmap_fragments_filename_safety_is_the_reference_regex() {
    // FILENAME_SAFE = ^[A-Za-z0-9][A-Za-z0-9._-]{0,110}$ — 111 characters at most.
    assert_eq!(MAX_ID_LEN, 111);
    assert!(is_filename_safe(&format!("A{}", "b".repeat(110))));
    assert!(!is_filename_safe(&format!("A{}", "b".repeat(111))));
    assert!(is_filename_safe("9.x_y-z"));
    assert!(!is_filename_safe(".hidden"));
    assert!(!is_filename_safe("-dash"));
    assert!(!is_filename_safe("a b"));
    assert!(!is_filename_safe("a/b"));
    assert!(!is_filename_safe("é"));
}

#[test]
fn roadmap_fragments_a_ticket_id_is_filename_safe_prefix_n_exactly() {
    for good in ["PMAT-1363", "GH-7", "A_B-001"] {
        assert!(is_ticket_id(good), "{good} must be accepted");
    }
    for bad in [
        "PMAT-12 (notes)",
        "PMAT-12-3",
        "PMAT-12a",
        "ROADMAP-RECONCILE-2026-07-04",
        "Push completed work to origin/main (5 commits)",
        "1PMAT-3",
        "PMAT-",
        "PMAT",
        "",
    ] {
        assert!(!is_ticket_id(bad), "{bad:?} must be refused");
    }
    assert!(
        !is_ticket_id(&format!("P-{}", "1".repeat(110))),
        "112 characters"
    );
}

#[test]
fn roadmap_fragments_parse_id_value_undoes_the_reference_quoting() {
    assert_eq!(parse_id_value("  PMAT-1 \r"), "PMAT-1");
    assert_eq!(parse_id_value("'it''s'"), "it's");
    assert_eq!(parse_id_value("\"say \\\"hi\\\"\""), "say \"hi\"");
    assert_eq!(parse_id_value("'"), "'");
}

#[test]
fn roadmap_fragments_split_reads_rows_at_the_row_indent_and_nothing_nested() {
    let col0 = "roadmap:\n- id: PMAT-1\n  subtasks:\n  - id: PMAT-1.1\n- id: 'PMAT-2'\n";
    let (pre, rows) = split_entries(col0);
    assert_eq!(pre, "roadmap:\n");
    assert_eq!(
        rows.iter().map(|(id, _)| id.as_str()).collect::<Vec<_>>(),
        ["PMAT-1", "PMAT-2"],
        "a subtask's id is not a row"
    );

    // pmat writes indent-2 roadmaps too; the reference's column-0 regex would read
    // this as having no rows at all, and every fragment would land in the preamble.
    let col2 = "roadmap:\n  - id: PMAT-1\n    title: a\n  - id: PMAT-3\n    title: c\n";
    let out = aggregate(
        col2,
        &[("PMAT-2".into(), "  - id: PMAT-2\n    title: b\n".into())],
    )
    .expect("aggregate accepts these fragments");
    assert_eq!(ids(&out), ["PMAT-1", "PMAT-2", "PMAT-3"]);
    assert_eq!(
        serde_yaml_ng::from_str::<serde_yaml_ng::Value>(&out)
            .expect("the aggregate parses as YAML")["roadmap"]
            .as_sequence()
            .map(Vec::len),
        Some(3)
    );
}

// ---------------------------------------------- the divergences, each proved

#[test]
fn roadmap_fragments_refuse_a_fragment_that_would_break_idempotence() {
    // Each of these is accepted by the reference and produces an aggregate that
    // changes on its own re-aggregation.
    let refused = |block: &str| check_fragment("PMAT-5", block, 0).expect_err(block);
    assert!(matches!(
        refused("# note\n- id: PMAT-5\n"),
        FragmentError::Malformed { .. }
    ));
    assert!(matches!(
        refused("- id: PMAT-5\n  t: x"),
        FragmentError::Malformed { .. }
    ));
    assert!(matches!(
        refused("- id: PMAT-6\n"),
        FragmentError::Malformed { .. }
    ));
    assert!(matches!(
        refused("- id: PMAT-5\n- id: PMAT-7\n"),
        FragmentError::Malformed { .. }
    ));
    assert!(matches!(
        refused("  - id: PMAT-5\n"),
        FragmentError::Malformed { .. }
    ));
    assert!(matches!(
        check_fragment("a b", "- id: a b\n", 0),
        Err(FragmentError::NotAFilename(_))
    ));
    assert_eq!(
        check_fragment("PMAT-5", "- id: PMAT-5\n  t: x\n", 0),
        Ok(())
    );

    // Why the leading comment is refused, shown rather than asserted: glued onto the
    // previous row, it is repeated by every aggregation.
    let glued = format!("{BASE}# note\n- id: PMAT-500\n  t: x\n");
    let (_, rows) = split_entries(&glued);
    assert!(
        rows[2].1.ends_with("# note\n"),
        "the comment belongs to LEGACY-THING's block now"
    );
}

#[test]
fn roadmap_fragments_an_empty_flow_sequence_base_still_yields_valid_yaml() {
    // The reference emits `roadmap: []\n- id: …`, which does not parse.
    let base = "roadmap_version: '1.0'\nroadmap: []\n";
    let f = vec![frag("PMAT-1", "title: first")];
    let once = aggregate(base, &f).expect("aggregate accepts these fragments");
    let parsed: serde_yaml_ng::Value = serde_yaml_ng::from_str(&once).expect("must parse");
    assert_eq!(parsed["roadmap"].as_sequence().map(Vec::len), Some(1));
    assert_eq!(
        aggregate(&once, &f).expect("aggregate accepts these fragments"),
        once,
        "and it is idempotent"
    );
}

#[test]
fn roadmap_fragments_first_difference_names_the_row() {
    let f = vec![frag("PMAT-200", "title: b")];
    let out = aggregate(BASE, &f).expect("aggregate accepts these fragments");
    assert_eq!(first_difference(&out, &out), None);
    assert_eq!(first_difference(&out, BASE).as_deref(), Some("PMAT-200"));
    let edited = BASE.replace("title: c", "title: C");
    assert_eq!(first_difference(BASE, &edited).as_deref(), Some("PMAT-300"));
    let header = BASE.replace("'1.0'", "'2.0'");
    assert_eq!(first_difference(BASE, &header).as_deref(), Some("PREAMBLE"));
}

// ------------------------------------------------------------- real data

/// This repository's own roadmap: always present, in CI too. 300-plus real rows prove
/// the PARSER — block scalars whose bodies quote ids, quoted ids, prose.
fn this_repos_roadmap() -> String {
    std::fs::read_to_string("docs/roadmaps/roadmap.yaml")
        .expect("this repository's roadmap is committed; a missing input is a failure, not a skip")
}

#[test]
fn roadmap_fragments_round_trip_a_real_roadmap_byte_for_byte() {
    let raw = this_repos_roadmap();
    let (preamble, entries) = split_entries(&raw);
    assert!(entries.len() > 100, "split found {} rows", entries.len());
    let rebuilt: String = preamble + &entries.iter().map(|(_, b)| b.as_str()).collect::<String>();
    assert_eq!(rebuilt, raw, "split/join is not byte-exact");
    assert_eq!(
        aggregate(&raw, &[]).expect("aggregate accepts these fragments"),
        raw,
        "aggregate(x, []) != x"
    );
}

#[test]
fn roadmap_fragments_three_consecutive_aggregations_of_real_data_are_byte_identical() {
    // Every 7th fragmentable row is lifted out of the base into a fragment — the
    // shape of a repository mid-migration — with its `updated:` changed, so
    // supersession is exercised, not only insertion.
    let raw = this_repos_roadmap();
    let (_, rows) = split_entries(&raw);
    let fragments: Vec<(String, String)> = rows
        .iter()
        .filter(|(id, _)| is_ticket_id(id))
        .step_by(7)
        .map(|(id, block)| (id.clone(), restamp(block)))
        .collect();
    assert!(fragments.len() > 10, "{} fragments", fragments.len());

    let first = aggregate(&raw, &fragments).expect("aggregate accepts these fragments");
    let second = aggregate(&raw, &fragments).expect("aggregate accepts these fragments");
    let third = aggregate(&raw, &fragments).expect("aggregate accepts these fragments");
    assert!(
        first == second && second == third,
        "three runs over one input differ"
    );
    assert_eq!(
        aggregate(&first, &fragments).expect("aggregate accepts these fragments"),
        first,
        "not idempotent on real data"
    );
    assert_ne!(first, raw, "the fragments must have changed something");

    let before: crate::models::roadmap::Roadmap =
        serde_yaml_ng::from_str(&raw).expect("this repository's roadmap parses");
    let after: crate::models::roadmap::Roadmap = serde_yaml_ng::from_str(&first)
        .expect("the aggregate of real data must parse as a roadmap");
    assert_eq!(
        after.roadmap.len(),
        before.roadmap.len(),
        "a row was lost or duplicated"
    );
    for (id, _) in &fragments {
        let item = after
            .find_item(id)
            .expect("every fragment's row is present");
        assert_eq!(item.updated, STAMP, "{id} was not superseded");
    }
}

const STAMP: &str = "2099-01-01T00:00:00Z";

/// The row with its own `updated:` value replaced — a change that cannot make the
/// YAML invalid, whatever the title's quoting.
fn restamp(block: &str) -> String {
    block
        .split_inclusive('\n')
        .map(|line| {
            if line.starts_with("  updated: ") {
                format!("  updated: {STAMP}\n")
            } else {
                line.to_string()
            }
        })
        .collect()
}
