//! Unit tests for the annotation gate and the repaired derivation guard.
//!
//! The evasion corpus below is the one the research pass built by hand: seven
//! one-line rewrites of the same defect, asking which of them a maintainer
//! could use to make the check look clean. Six of the seven silence it, and
//! that is the correct answer for six of them — an approximation, a quotation
//! and a narration genuinely are not assertions. `v4` is the one that silences
//! it for no good reason, and it is pinned here as a known blind spot rather
//! than papered over.

use super::annotate::{
    assertive, classify, has_rationale, headroom, is_derivation, observations, restatements,
    strip_observation_clauses, xrefs, Role, MAX_ASSERTION_CHARS,
};

#[test]
fn reachability_probe_annotate() {}

/// Only a short, unhedged, unquoted, same-line comment asserts.
#[test]
fn the_annotation_gate_admits_only_assertions() {
    // The live positive: the brief's headline defect, 38 characters.
    let tp = "Current: 570 (CRITICAL - must reduce!)";
    assert_eq!(tp.chars().count(), 38);
    assert_eq!(classify(tp), Role::Assertive);

    assert_eq!(classify(""), Role::Empty);
    assert_eq!(
        classify("Current: ~570 (CRITICAL - must reduce!)"),
        Role::Approximate,
        "a hedged number cannot contradict a precise one"
    );
    assert_eq!(
        classify("Current: 570 baseline"),
        Role::Narration,
        "a baseline is a record of a past measurement"
    );
    assert_eq!(
        classify("Current: 570 for `src/`"),
        Role::Quoted,
        "a quoted number is someone else's"
    );
    assert_eq!(
        classify("10min (user requirement: \"under 10 min coverage\")"),
        Role::Quoted
    );
}

/// The 72-character bound, exactly.
///
/// Measured, not chosen: the longest true-positive annotation across both audit
/// repositories is 38 characters, and the shortest narration false positive is
/// 74. Anything in between is a coin toss, so the boundary sits where the data
/// separates.
#[test]
fn the_length_bound_is_where_the_data_separates() {
    assert_eq!(MAX_ASSERTION_CHARS, 72);
    let at_bound = "x".repeat(MAX_ASSERTION_CHARS);
    let over_bound = "x".repeat(MAX_ASSERTION_CHARS + 1);
    assert_eq!(classify(&at_bound), Role::Assertive);
    assert_eq!(
        classify(&over_bound),
        Role::Context,
        "a paragraph explains a number; it does not assert one"
    );
    // Counter-control: length alone is doing the work here, not content.
    assert_eq!(at_bound.chars().count() + 1, over_bound.chars().count());
}

/// The evasion corpus, verbatim, with the verdict for each rewrite.
#[test]
fn known_evasions_are_classified_deliberately() {
    let annot = "Current: 570 (CRITICAL - must reduce!)";
    assert!(
        assertive(annot) && !observations(annot).is_empty(),
        "v0 fires"
    );

    // v1 moves the comment to the line above. Handled in the extractor: the
    // preceding block never reaches this gate.
    // v2 hedges, v5 calls it a baseline, v6 quotes a path.
    assert!(!assertive("Current: ~570 (CRITICAL - must reduce!)"));
    assert!(!assertive("Current: 570 baseline"));
    assert!(!assertive("Current: 570 for `src/`"));

    // v3 is longer prose but still under the bound, and still fires.
    let v3 = "Current: 570 unwrap calls in shipped code today, must reduce";
    assert!(v3.chars().count() <= MAX_ASSERTION_CHARS);
    assert!(
        assertive(v3) && !observations(v3).is_empty(),
        "v3 still fires"
    );

    // v4 is the known blind spot: rephrasing `Current: 570` as `Current count
    // is 570` slips past the observation pattern. It is recorded, not fixed —
    // widening the pattern to catch it is exactly the change that would have to
    // be re-measured against the reference corpus first.
    let v4 = "Current count is 570, must reduce";
    assert!(assertive(v4), "v4 passes the gate");
    assert!(
        observations(v4).is_empty(),
        "v4 escapes the observation pattern — a recorded blind spot, not a pass"
    );
}

/// Observation clauses belong to C1, not C2.
///
/// Without stripping them, `50 MB (current: 42 MB, 16% headroom)` reads as a
/// comment claiming the limit is 42 MB, and C2 fires on every healthy budget
/// line in the tree.
#[test]
fn observation_clauses_are_stripped_before_restatement() {
    let annot = "50 MB (current: 42 MB, 16% headroom)";
    let obs = observations(annot);
    assert_eq!(obs.len(), 1);
    assert_eq!(obs[0].value, 42.0);
    assert_eq!(obs[0].unit, "MB");

    let head = strip_observation_clauses(annot);
    assert!(
        !head.contains("42"),
        "the observation must not survive into the restatement pass: {head:?}"
    );
    let restated = restatements(&head);
    assert_eq!(restated.len(), 1);
    assert_eq!(restated[0].value, 50.0);
    assert_eq!(restated[0].unit, "MB");

    assert_eq!(headroom(annot).map(|(p, _)| p), Some(16.0));
}

/// The ten hand-audited correct derivations, at the guard's own level.
#[test]
fn the_derivation_guard_accepts_every_audited_derivation() {
    let cases: [(&str, f64); 9] = [
        ("64 rows * 17 bytes", 1088.0),
        ("64 * 17 + 16 bytes header", 1104.0),
        ("4 lanes * 8 regs * 10 bytes", 320.0),
        ("1000 ms / 4 workers", 250.0),
        ("3 * 1024 bytes", 3072.0),
        ("30 ms * 3 retries", 90.0),
        ("128 bytes payload + 4 bytes crc", 132.0),
        ("500 ms budget, 3 phases", 1500.0),
        ("(64 + 1) * 32 bytes", 2080.0),
    ];
    for (annot, target) in cases {
        assert!(
            is_derivation(annot, &[target]),
            "{annot:?} derives {target} and must not be read as a restatement"
        );
    }
    // The spec's own example.
    assert!(is_derivation("(2 bytes scale + 16 bytes quants)", &[18.0]));
    // The tenth case has one number: a unit conversion, not a derivation. It is
    // acquitted by the unit tables instead, and the guard must not claim it.
    assert!(!is_derivation("2 hours", &[7_200_000.0]));
}

/// A specimen the repaired guard missed, found by running R2 over aprender.
///
/// `memory_saved_bytes: 75776  # intermediate_dim × 4 bytes (18944 × 4 for 7B)`
/// at `crates/apr-cli/contracts/kernel-fusion-v1.yaml:89` derives its value
/// exactly, and the first cut of the repair reported it as a contradiction —
/// twice over. The multiplication is written U+00D7, which the tokenizer read
/// as punctuation; and the parenthesised aside ends in `7B`, whose stray
/// numeral made the whole run unparseable. Both are how a repository actually
/// writes arithmetic, so both are fixed rather than tolerated.
///
/// This is the reason a rule change is re-measured on the reference corpus
/// before it ships: the unit tests were green and the regression was one
/// finding wide.
#[test]
fn prose_arithmetic_is_still_arithmetic() {
    assert!(
        is_derivation("intermediate_dim × 4 bytes (18944 × 4 for 7B)", &[75_776.0]),
        "18944 × 4 is 75,776 — U+00D7 is a multiplication sign, not punctuation"
    );
    assert!(is_derivation(
        "hidden_dim × 4 bytes (3584 × 4 for 7B) — avoids intermediate buffer",
        &[14_336.0]
    ));
    // A chain stops at the first numeral no operator connects, so the trailing
    // `7B` neither breaks the chain nor joins it.
    assert!(is_derivation("(18944 × 4 for 7B)", &[75_776.0]));
    assert!(!is_derivation("(18944 × 4 for 7B)", &[7.0]));
    // The other Unicode operators, and the counter-control that the tokenizer
    // is not simply treating every symbol as a multiply.
    assert!(is_derivation("60 ÷ 4 workers", &[15.0]));
    assert!(!is_derivation("60 ~ 4 workers", &[15.0]));
}

/// The repair mandated by the spec's own attack, at the guard's own level.
///
/// The researched guard searched every pairwise combination of `+ - * /`,
/// `/100` and `/1000` — plus every single number on its own — for a 2%-tolerant
/// hit. Measured against a real annotation distribution that declared an
/// *unrelated* target a derivation 21.6% of the time at two numbers and 84.4%
/// at ten, silently suppressing about one genuine contradiction in five on the
/// commonest annotated shape.
///
/// Only three forms are accepted now: a sum or product over every number, a
/// word-stripped arithmetic run, or a sum or product of at most three numbers
/// each adjacent to a word. Subtraction of two unrelated numbers is not one of
/// them.
#[test]
fn coincidence_is_not_derivation() {
    // 100 - 16 == 84. The old guard called that a derivation and suppressed the
    // finding; there is no `-` anywhere in the annotation.
    assert!(
        !is_derivation("100 ms at 16 ms overhead", &[84.0]),
        "a difference nobody wrote down is a coincidence, not a derivation"
    );
    // 900 / 100 == 9, via the old `/100` scaling guess.
    assert!(!is_derivation("900 ms over 100 runs", &[9.0]));
    // A bare number equal to the target: the old contiguous-sum-of-one rule.
    assert!(!is_derivation("see PMAT-2104 and 512 elsewhere", &[512.0]));
    // Counter-control: the same shape *with* the operator written down is a
    // derivation, and must still be suppressed.
    assert!(is_derivation("900 ms / 100 runs", &[9.0]));
}

/// Cross-references carry a file, a key, or both.
#[test]
fn xrefs_name_a_file_and_a_key() {
    let found = xrefs("50MB (aligned with .pmat-metrics.toml binary_max_bytes)");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].file.as_deref(), Some(".pmat-metrics.toml"));
    assert_eq!(found[0].key.as_deref(), Some("binary_max_bytes"));

    assert!(
        xrefs("50MB, roughly the same ballpark").is_empty(),
        "vague similarity is not an assertion of equality"
    );
}

/// A divergent number with a reason is policy; one with only a restatement is
/// anchored to nothing.
#[test]
fn rationale_requires_more_than_restating_the_key() {
    assert!(!has_rationale("", "threshold"));
    assert!(!has_rationale("target: 95", "threshold"));
    assert!(
        !has_rationale("max threshold value", "threshold"),
        "restating the key is not a reason"
    );
    assert!(has_rationale(
        "codecov needs the drop allowance wide during the migration",
        "threshold"
    ));
}
