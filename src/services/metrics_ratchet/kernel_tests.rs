//! Falsification tests for the ratchet/coherence kernel.
//!
//! Named against `contracts/comply-ratchet-v1.yaml` and
//! `contracts/comply-threshold-coherence-v1.yaml`. Each test states the rule it
//! would break, so a failure names the invariant rather than a line number.
//!
//! The sweeps are exhaustive over a small window plus the `i64` extremes rather
//! than random: for a three-way integer comparison the interesting inputs are
//! the boundaries, and a boundary you enumerate is a boundary you tested.

use super::config::*;
use super::kernel::*;
use std::collections::BTreeMap;

const EDGES: [i64; 9] = [
    i64::MIN,
    i64::MIN + 1,
    -1_000_000,
    -1,
    0,
    1,
    1_000_000,
    i64::MAX - 1,
    i64::MAX,
];

// ─────────────────────────── INV-2102-1..3 ───────────────────────────

/// INV-2102-1: `verdict(b, o) = Fail  <->  o > b`. Anything else means a
/// regression can pass, or a clean tree can be blocked.
#[test]
fn inv_2102_1_verdict_fails_exactly_when_observed_exceeds_baseline() {
    for b in -50..=50i64 {
        for o in -50..=50i64 {
            let expected = if o > b {
                RatchetVerdict::Fail
            } else {
                RatchetVerdict::Pass
            };
            assert_eq!(
                ratchet_verdict(b, o),
                expected,
                "baseline {b}, observed {o}"
            );
        }
    }
    for &b in &EDGES {
        for &o in &EDGES {
            assert_eq!(
                ratchet_verdict(b, o) == RatchetVerdict::Fail,
                o > b,
                "baseline {b}, observed {o}"
            );
        }
    }
}

/// INV-2102-2: `next(b, o) = o` when `o <= b`, else `b` — the baseline is
/// monotone non-increasing. A rising baseline is a ratchet that ratchets the
/// wrong way, which is how debt limits historically drifted.
#[test]
fn inv_2102_2_next_baseline_is_monotone_non_increasing() {
    for b in -50..=50i64 {
        for o in -50..=50i64 {
            let n = next_baseline(b, o);
            assert!(n <= b, "baseline {b}, observed {o} produced {n} > {b}");
            assert_eq!(n, if o <= b { o } else { b });
        }
    }
    for &b in &EDGES {
        for &o in &EDGES {
            assert!(next_baseline(b, o) <= b);
        }
    }
}

/// INV-2102-3: `next` is idempotent — running the nightly job twice on the same
/// measurement must not walk the baseline anywhere.
#[test]
fn inv_2102_3_next_baseline_is_idempotent() {
    for b in -50..=50i64 {
        for o in -50..=50i64 {
            let once = next_baseline(b, o);
            assert_eq!(next_baseline(once, o), once, "baseline {b}, observed {o}");
        }
    }
    for &b in &EDGES {
        for &o in &EDGES {
            let once = next_baseline(b, o);
            assert_eq!(next_baseline(once, o), once);
        }
    }
}

// ─────────────────────────── INV-2101-1..3 ───────────────────────────

/// INV-2101-3: classification is TOTAL — every (limit, measured, band,
/// direction) lands on exactly one of FIRING / VIOLATED / VACUOUS. A fourth
/// state, or an unreachable input, is how "unmeasurable" gets to look like
/// "fine".
#[test]
fn inv_2101_3_classification_is_total_and_disjoint() {
    let dirs = [Direction::Max, Direction::Min];
    for limit in -20..=20i64 {
        for measured in -20..=20i64 {
            for band in [0u64, 1, 5, 40, u64::MAX] {
                for dir in dirs {
                    let c = classify(limit, measured, band, dir);
                    let n = u8::from(c == Classification::Firing)
                        + u8::from(c == Classification::Violated)
                        + u8::from(c == Classification::Vacuous);
                    assert_eq!(
                        n, 1,
                        "limit {limit} measured {measured} band {band} {dir:?}"
                    );
                }
            }
        }
    }
}

/// INV-2101-2: VIOLATED means, and only means, that the bound is breached.
#[test]
fn inv_2101_2_violated_iff_bound_breached() {
    for limit in -20..=20i64 {
        for measured in -20..=20i64 {
            for band in [0u64, 3, 1000] {
                assert_eq!(
                    classify(limit, measured, band, Direction::Max) == Classification::Violated,
                    measured > limit
                );
                assert_eq!(
                    classify(limit, measured, band, Direction::Min) == Classification::Violated,
                    measured < limit
                );
            }
        }
    }
}

/// INV-2101-1: FIRING means the limit is reachable — inside the band — and
/// VACUOUS means it is not. The band boundary is inclusive on the FIRING side.
#[test]
fn inv_2101_1_firing_is_exactly_the_reachable_band() {
    // Max, measured 100, band 10: limits 100..=110 fire, 111+ are vacuous,
    // 99 and below are violated.
    assert_eq!(
        classify(99, 100, 10, Direction::Max),
        Classification::Violated
    );
    assert_eq!(
        classify(100, 100, 10, Direction::Max),
        Classification::Firing
    );
    assert_eq!(
        classify(110, 100, 10, Direction::Max),
        Classification::Firing
    );
    assert_eq!(
        classify(111, 100, 10, Direction::Max),
        Classification::Vacuous
    );
    // Min mirrors it.
    assert_eq!(
        classify(100, 99, 10, Direction::Min),
        Classification::Violated
    );
    assert_eq!(
        classify(100, 100, 10, Direction::Min),
        Classification::Firing
    );
    assert_eq!(
        classify(100, 110, 10, Direction::Min),
        Classification::Firing
    );
    assert_eq!(
        classify(100, 111, 10, Direction::Min),
        Classification::Vacuous
    );
}

/// The `i64` extremes must not overflow the slack computation. An overflow here
/// silently inverts the verdict, which is the worst failure a gate can have:
/// green for the wrong reason.
#[test]
fn classify_does_not_overflow_at_i64_extremes() {
    assert_eq!(
        classify(i64::MAX, i64::MIN, 0, Direction::Max),
        Classification::Vacuous
    );
    assert_eq!(
        classify(i64::MIN, i64::MAX, u64::MAX, Direction::Max),
        Classification::Violated
    );
    assert_eq!(
        classify(i64::MIN, i64::MAX, u64::MAX, Direction::Min),
        Classification::Firing
    );
    assert_eq!(
        slack(i64::MAX, i64::MIN, Direction::Max),
        (i64::MAX as i128) - (i64::MIN as i128)
    );
}

/// KANI-2101-2 in test form: loosening a FIRING threshold never makes it
/// VIOLATED. Without this, "relax the limit" could be a route from a live gate
/// to a failing one, and every author would learn to skip the gate instead.
#[test]
fn classify_is_monotone_in_limit() {
    for limit in -20..=20i64 {
        for looser in limit..=25i64 {
            for measured in -20..=20i64 {
                for band in [0u64, 4, 100] {
                    if classify(limit, measured, band, Direction::Max) == Classification::Firing {
                        assert_ne!(
                            classify(looser, measured, band, Direction::Max),
                            Classification::Violated,
                            "limit {limit} -> {looser}, measured {measured}, band {band}"
                        );
                    }
                }
            }
        }
    }
}

// ─────────────────────── the four named FALSIFY-2102 cases ───────────────────

fn baseline(value: i64) -> MetricBaseline {
    MetricBaseline {
        baseline: value,
        unit: "count".into(),
        band: 0,
        includes_test_files: false,
        command: "true".into(),
        description: "test metric".into(),
        justification: None,
        zero_is_reachable: false,
        analyzer: None,
    }
}

fn metrics_with(id: &str, b: MetricBaseline) -> BTreeMap<String, MetricBaseline> {
    let mut m = BTreeMap::new();
    m.insert(id.to_string(), b);
    m
}

/// FALSIFY-2102-1: baseline at the MEASURED value, a PR adds one `.unwrap()`
/// -> Fail. Uses the real captured figure (11056), not the 570 the source
/// document and `.pmat-metrics.toml`'s own comment both claimed.
#[test]
fn falsify_2102_1_one_added_unwrap_fails_against_the_measured_baseline() {
    let metrics = metrics_with("unwrap_calls", baseline(11_056));
    let mut measured = Measurements::new();
    measured.insert("unwrap_calls".into(), Measurement::Value(11_057));

    let report = evaluate_ratchet(&metrics, &measured, None);
    assert_eq!(report.outcome, Outcome::Fail);
    assert_eq!(report.metrics[0].verdict, Some(RatchetVerdict::Fail));
    // And the failing run must NOT rewrite the baseline.
    assert_eq!(report.metrics[0].next_baseline, Some(11_056));
}

/// FALSIFY-2102-2: a PR removes three -> Pass, and the nightly job's rewrite
/// target is the new, lower figure.
#[test]
fn falsify_2102_2_removing_three_passes_and_lowers_the_baseline() {
    let metrics = metrics_with("unwrap_calls", baseline(11_056));
    let mut measured = Measurements::new();
    measured.insert("unwrap_calls".into(), Measurement::Value(11_053));

    let report = evaluate_ratchet(&metrics, &measured, None);
    assert_eq!(report.outcome, Outcome::Ok);
    assert_eq!(report.metrics[0].verdict, Some(RatchetVerdict::Pass));
    assert_eq!(report.metrics[0].next_baseline, Some(11_053));
}

/// FALSIFY-2102-3: `.pmat-ratchet.toml` edited upward with no justification
/// -> Fail, even though the observed value is inside the new baseline. This is
/// the only failure mode the measurement alone cannot catch: the tree agrees
/// with the file because the file was moved to agree with the tree.
#[test]
fn falsify_2102_3_raising_a_baseline_without_justification_fails() {
    let previous = metrics_with("unwrap_calls", baseline(11_056));
    let raised = metrics_with("unwrap_calls", baseline(11_500));
    let mut measured = Measurements::new();
    measured.insert("unwrap_calls".into(), Measurement::Value(11_400));

    let report = evaluate_ratchet(&raised, &measured, Some(&previous));
    assert_eq!(report.outcome, Outcome::Fail);
    assert_eq!(report.unjustified_raises.len(), 1);
    assert!(report.unjustified_raises[0].contains("11056 -> 11500"));

    // With a justification the same edit is allowed, and says so.
    let mut justified_b = baseline(11_500);
    justified_b.justification = Some("vendored module imported wholesale, see #1234".into());
    let justified = metrics_with("unwrap_calls", justified_b);
    let report = evaluate_ratchet(&justified, &measured, Some(&previous));
    assert_eq!(report.outcome, Outcome::Ok);
    assert!(report.unjustified_raises.is_empty());
}

/// FALSIFY-2102-4: a declared metric absent from the measurement run -> Fail,
/// not Pass. An empty result set is a failure to measure, never a clean bill.
#[test]
fn falsify_2102_4_missing_measurement_fails_it_does_not_pass() {
    let metrics = metrics_with("unwrap_calls", baseline(11_056));

    let report = evaluate_ratchet(&metrics, &Measurements::new(), None);
    assert_eq!(report.outcome, Outcome::Fail, "absent metric must not pass");
    assert!(report.metrics[0].verdict.is_none());
    assert!(report.metrics[0]
        .detail
        .contains("unmeasurable is not compliant"));

    let mut unavailable = Measurements::new();
    unavailable.insert(
        "unwrap_calls".into(),
        Measurement::Unavailable("src/ not found".into()),
    );
    let report = evaluate_ratchet(&metrics, &unavailable, None);
    assert_eq!(report.outcome, Outcome::Fail);
    assert!(report.metrics[0].detail.contains("src/ not found"));
}

/// Every ratcheted metric is normalised so that BIGGER IS WORSE, so a coverage
/// ratchet is stored as *uncovered* basis points. This test pins the reason:
/// with the virtue stored instead of the debt, a 3.17-point coverage DROP
/// reads as a Pass under `INV-2102-1`, and the nightly job then writes the
/// drop in as the new baseline — a ratchet running backwards.
#[test]
fn ratchet_metrics_store_the_debt_so_a_drop_cannot_read_as_an_improvement() {
    // Stored as debt: uncovered = 10000 - covered. Coverage falling 73.17% ->
    // 70.00% is uncovered rising 2683 -> 3000, which exceeds the baseline.
    let metrics = metrics_with("uncovered_bp", baseline(2_683));
    let mut measured = Measurements::new();
    measured.insert("uncovered_bp".into(), Measurement::Value(3_000));
    let report = evaluate_ratchet(&metrics, &measured, None);
    assert_eq!(report.outcome, Outcome::Fail);
    assert_eq!(report.metrics[0].verdict, Some(RatchetVerdict::Fail));

    // The same movement stored as the virtue (covered_bp 7317 -> 7000) would
    // have passed, which is the trap the normalisation rule removes.
    assert_eq!(ratchet_verdict(7_317, 7_000), RatchetVerdict::Pass);
}

/// A raise is an increase, full stop — no direction to get backwards.
#[test]
fn falsify_2102_3_raise_detection_is_direction_free() {
    let previous = metrics_with("uncovered_bp", baseline(2_683));
    let raised = metrics_with("uncovered_bp", baseline(3_000));
    let mut measured = Measurements::new();
    measured.insert("uncovered_bp".into(), Measurement::Value(2_900));
    let report = evaluate_ratchet(&raised, &measured, Some(&previous));
    assert_eq!(report.outcome, Outcome::Fail);
    assert_eq!(report.unjustified_raises.len(), 1);
}
