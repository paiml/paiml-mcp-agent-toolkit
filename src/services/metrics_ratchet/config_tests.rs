//! Falsification tests for the threshold-coherence evaluator (CB-2101) and the
//! two config readers.
//!
//! Contract: `contracts/comply-threshold-coherence-v1.yaml`.

use super::config::*;
use super::kernel::{Classification, Direction};
use std::collections::BTreeMap;

const METRICS_MINI: &str = r#"
[thresholds]
lint_max_ms = 150_000

[quality_gates]
max_unwrap_calls = 100
min_tdg_grade = "A-"

[enforcement]
fail_on_threshold_violation = true
"#;

fn unwrap_metric(band: u64) -> BTreeMap<String, MetricBaseline> {
    let mut m = BTreeMap::new();
    m.insert(
        "unwrap_calls".to_string(),
        MetricBaseline {
            baseline: 11_056,
            unit: "count".into(),
            band,
            includes_test_files: false,
            command: "true".into(),
            description: "test".into(),
            justification: None,
        },
    );
    m
}

fn gate_binding(band: Option<u64>) -> ThresholdBinding {
    ThresholdBinding {
        kind: BindingKind::Gate,
        metric: Some("unwrap_calls".into()),
        direction: Some(Direction::Max),
        band,
        justification: None,
        enforced_by: None,
    }
}

fn budget(justification: Option<&str>) -> ThresholdBinding {
    ThresholdBinding {
        kind: BindingKind::Budget,
        metric: None,
        direction: None,
        band: None,
        justification: justification.map(str::to_string),
        enforced_by: None,
    }
}

fn coherence(bindings: Vec<(&str, ThresholdBinding)>) -> CoherenceConfig {
    CoherenceConfig {
        threshold_sections: vec!["thresholds".into(), "quality_gates".into()],
        non_threshold_sections: BTreeMap::from([(
            "enforcement".to_string(),
            "switches, not thresholds".to_string(),
        )]),
        binding: bindings
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect(),
    }
}

fn measured(v: i64) -> Measurements {
    Measurements::from([("unwrap_calls".to_string(), Measurement::Value(v))])
}

fn verdict_for<'a>(r: &'a CoherenceReport, key: &str) -> &'a ThresholdVerdict {
    r.thresholds
        .iter()
        .find(|t| t.key == key)
        .unwrap_or_else(|| panic!("no verdict for {key}; totality broken"))
}

// ─────────────────────────── roster parsing ───────────────────────────

/// Every non-boolean scalar in a threshold section is on the roster — including
/// the string-valued `min_tdg_grade`, which must not be able to dodge the audit
/// by not being a number.
#[test]
fn roster_collects_every_scalar_including_strings_and_excludes_bools() {
    let r = MetricsRoster::parse(METRICS_MINI).expect("parses");
    let keys: Vec<&str> = r.thresholds.iter().map(|t| t.key.as_str()).collect();
    assert!(keys.contains(&"thresholds.lint_max_ms"));
    assert!(keys.contains(&"quality_gates.max_unwrap_calls"));
    assert!(keys.contains(&"quality_gates.min_tdg_grade"));
    assert!(
        !keys.contains(&"enforcement.fail_on_threshold_violation"),
        "booleans are switches, not thresholds"
    );
    assert_eq!(
        r.sections,
        vec!["enforcement", "quality_gates", "thresholds"],
        "every section is enumerated, including non-threshold ones"
    );
}

// ─────────────────────────── arm (a): VIOLATED ───────────────────────────

/// CB-2101 arm (a), the live defect: `max_unwrap_calls = 100` against 11,056
/// measured. A max-threshold exceeded at HEAD while the build is green is a
/// FAIL, not a warning — the config is asserting something false.
#[test]
fn arm_a_violated_threshold_on_a_green_build_fails() {
    let roster = MetricsRoster::parse(METRICS_MINI).unwrap();
    let cfg = coherence(vec![
        ("quality_gates.max_unwrap_calls", gate_binding(Some(200))),
        ("thresholds.lint_max_ms", budget(Some("timing budget"))),
        ("quality_gates.min_tdg_grade", budget(Some("aspiration"))),
    ]);
    let report = evaluate_coherence(&roster, &cfg, &measured(11_056), &unwrap_metric(200));

    let v = verdict_for(&report, "quality_gates.max_unwrap_calls");
    assert_eq!(v.classification, Classification::Violated);
    assert_eq!(v.outcome, Outcome::Fail);
    assert_eq!(report.outcome, Outcome::Fail);
    assert!(v.detail.contains("11056"));
}

// ─────────────────────────── arm (b): VACUOUS ───────────────────────────

/// CB-2101 arm (b): a limit further from the measurement than the band can
/// never fire. With a justification that is a WARN; without one it is a FAIL,
/// so decoration cannot masquerade as enforcement silently.
#[test]
fn arm_b_vacuous_threshold_warns_with_justification_and_fails_without() {
    let roster = MetricsRoster::parse(METRICS_MINI).unwrap();

    let mut vacuous = gate_binding(Some(10));
    vacuous.justification = Some("headroom for the vendored tree".into());
    let cfg = coherence(vec![
        ("quality_gates.max_unwrap_calls", vacuous),
        ("thresholds.lint_max_ms", budget(Some("timing budget"))),
        ("quality_gates.min_tdg_grade", budget(Some("aspiration"))),
    ]);
    // limit 100 vs measured 20, band 10 -> slack 80 > 10 -> vacuous.
    let report = evaluate_coherence(&roster, &cfg, &measured(20), &unwrap_metric(10));
    let v = verdict_for(&report, "quality_gates.max_unwrap_calls");
    assert_eq!(v.classification, Classification::Vacuous);
    assert_eq!(v.outcome, Outcome::Warn);
    assert_eq!(report.outcome, Outcome::Warn);

    let cfg = coherence(vec![
        ("quality_gates.max_unwrap_calls", gate_binding(Some(10))),
        ("thresholds.lint_max_ms", budget(Some("timing budget"))),
        ("quality_gates.min_tdg_grade", budget(Some("aspiration"))),
    ]);
    let report = evaluate_coherence(&roster, &cfg, &measured(20), &unwrap_metric(10));
    let v = verdict_for(&report, "quality_gates.max_unwrap_calls");
    assert_eq!(v.classification, Classification::Vacuous);
    assert_eq!(v.outcome, Outcome::Fail, "vacuous without justification");
}

/// The discriminating mutation for CB-2101: set `max_unwrap_calls = 100000`
/// and the classification must move to VACUOUS. Pinned as a test so the
/// mutation stays reproducible after the config is fixed.
#[test]
fn mutation_max_unwrap_calls_100000_classifies_vacuous() {
    let roster = MetricsRoster::parse(
        "[quality_gates]\nmax_unwrap_calls = 100000\n[enforcement]\nx = true\n",
    )
    .unwrap();
    let mut cfg = coherence(vec![(
        "quality_gates.max_unwrap_calls",
        gate_binding(Some(200)),
    )]);
    cfg.non_threshold_sections
        .insert("enforcement".into(), "switches".into());
    let report = evaluate_coherence(&roster, &cfg, &measured(11_056), &unwrap_metric(200));
    let v = verdict_for(&report, "quality_gates.max_unwrap_calls");
    assert_eq!(v.classification, Classification::Vacuous);
    assert_eq!(v.outcome, Outcome::Fail);
}

// ───────────────────────── FIRING (the fixed state) ─────────────────────────

/// After the fix — limit at the measurement plus the band — the threshold is
/// FIRING, and one more unwrap keeps it FIRING while the ratchet's own band-0
/// baseline is what goes red. That separation is what makes the CB-2102
/// mutation discriminating.
#[test]
fn firing_threshold_absorbs_one_unwrap_so_only_the_ratchet_moves() {
    let roster = MetricsRoster::parse("[quality_gates]\nmax_unwrap_calls = 11256\n").unwrap();
    let cfg = CoherenceConfig {
        threshold_sections: vec!["quality_gates".into()],
        non_threshold_sections: BTreeMap::new(),
        binding: BTreeMap::from([(
            "quality_gates.max_unwrap_calls".to_string(),
            gate_binding(Some(200)),
        )]),
    };
    for observed in [11_056, 11_057] {
        let report = evaluate_coherence(&roster, &cfg, &measured(observed), &unwrap_metric(200));
        let v = verdict_for(&report, "quality_gates.max_unwrap_calls");
        assert_eq!(
            v.classification,
            Classification::Firing,
            "observed {observed} should stay FIRING"
        );
        assert_eq!(v.outcome, Outcome::Ok);
    }
}

// ─────────────────────── FALSIFY-2101-3: unmeasurable ───────────────────────

/// FALSIFY-2101-3: a gate whose metric produced no measurement is a FAIL.
/// Unmeasurable is not compliant, and it is not a warning either.
#[test]
fn falsify_2101_3_unmeasurable_metric_fails() {
    let roster = MetricsRoster::parse("[quality_gates]\nmax_unwrap_calls = 11256\n").unwrap();
    let cfg = CoherenceConfig {
        threshold_sections: vec!["quality_gates".into()],
        non_threshold_sections: BTreeMap::new(),
        binding: BTreeMap::from([(
            "quality_gates.max_unwrap_calls".to_string(),
            gate_binding(Some(200)),
        )]),
    };

    // No measurement at all.
    let report = evaluate_coherence(&roster, &cfg, &Measurements::new(), &unwrap_metric(200));
    let v = verdict_for(&report, "quality_gates.max_unwrap_calls");
    assert_eq!(v.outcome, Outcome::Fail);
    assert_eq!(v.classification, Classification::Vacuous);
    assert!(v.detail.contains("unmeasurable is not compliant"));

    // Explicitly unavailable, with a reason.
    let m = Measurements::from([(
        "unwrap_calls".to_string(),
        Measurement::Unavailable("no src/ directory".into()),
    )]);
    let report = evaluate_coherence(&roster, &cfg, &m, &unwrap_metric(200));
    assert_eq!(report.outcome, Outcome::Fail);
    assert!(verdict_for(&report, "quality_gates.max_unwrap_calls")
        .detail
        .contains("no src/ directory"));

    // A justification must NOT buy an unmeasurable gate a pass.
    let mut justified = gate_binding(Some(200));
    justified.justification = Some("we will wire it up later".into());
    let cfg = CoherenceConfig {
        threshold_sections: vec!["quality_gates".into()],
        non_threshold_sections: BTreeMap::new(),
        binding: BTreeMap::from([("quality_gates.max_unwrap_calls".to_string(), justified)]),
    };
    let report = evaluate_coherence(&roster, &cfg, &Measurements::new(), &unwrap_metric(200));
    assert_eq!(
        report.outcome,
        Outcome::Fail,
        "a justification cannot make an unmeasured gate compliant"
    );
}

// ───────────────────────── totality of the audit ─────────────────────────

/// INV-2101-3 at the file level: every threshold in a threshold section gets
/// exactly one verdict carrying exactly one classification — including the ones
/// nobody declared, which fail rather than vanish.
#[test]
fn inv_2101_3_every_threshold_gets_exactly_one_classification() {
    let roster = MetricsRoster::parse(METRICS_MINI).unwrap();
    let cfg = coherence(vec![(
        "quality_gates.max_unwrap_calls",
        gate_binding(Some(200)),
    )]);
    let report = evaluate_coherence(&roster, &cfg, &measured(11_000), &unwrap_metric(200));

    let in_threshold_sections = roster
        .thresholds
        .iter()
        .filter(|t| cfg.threshold_sections.contains(&t.section))
        .count();
    assert_eq!(report.thresholds.len(), in_threshold_sections);

    let undeclared = verdict_for(&report, "thresholds.lint_max_ms");
    assert_eq!(undeclared.kind, "undeclared");
    assert_eq!(undeclared.outcome, Outcome::Fail);
    assert_eq!(undeclared.classification, Classification::Vacuous);
}

/// A NEW section of `.pmat-metrics.toml` is in neither list, so it fails closed
/// instead of silently exempting every threshold inside it.
#[test]
fn a_new_section_fails_closed() {
    let roster =
        MetricsRoster::parse("[quality_gates]\nmax_unwrap_calls = 11256\n[brand_new]\nx = 1\n")
            .unwrap();
    let cfg = CoherenceConfig {
        threshold_sections: vec!["quality_gates".into()],
        non_threshold_sections: BTreeMap::new(),
        binding: BTreeMap::from([(
            "quality_gates.max_unwrap_calls".to_string(),
            gate_binding(Some(200)),
        )]),
    };
    let report = evaluate_coherence(&roster, &cfg, &measured(11_056), &unwrap_metric(200));
    assert_eq!(report.undeclared_sections, vec!["brand_new".to_string()]);
    assert_eq!(report.outcome, Outcome::Fail);
}

/// A budget with no justification, and an "external" declaration naming no
/// enforcing code, both fail. Neither is allowed to be a place to file a
/// number and forget it.
#[test]
fn unjustified_budget_and_unnamed_external_both_fail() {
    let roster = MetricsRoster::parse("[quality_gates]\nmax_unwrap_calls = 100\n").unwrap();
    let mk = |b: ThresholdBinding| CoherenceConfig {
        threshold_sections: vec!["quality_gates".into()],
        non_threshold_sections: BTreeMap::new(),
        binding: BTreeMap::from([("quality_gates.max_unwrap_calls".to_string(), b)]),
    };

    let report = evaluate_coherence(
        &roster,
        &mk(budget(None)),
        &measured(11_056),
        &unwrap_metric(0),
    );
    assert_eq!(report.outcome, Outcome::Fail);

    let report = evaluate_coherence(
        &roster,
        &mk(budget(Some("   "))),
        &measured(11_056),
        &unwrap_metric(0),
    );
    assert_eq!(report.outcome, Outcome::Fail, "whitespace is not a reason");

    let external = ThresholdBinding {
        kind: BindingKind::External,
        metric: None,
        direction: None,
        band: None,
        justification: None,
        enforced_by: None,
    };
    let report = evaluate_coherence(&roster, &mk(external), &measured(11_056), &unwrap_metric(0));
    assert_eq!(report.outcome, Outcome::Fail);
}

/// A gate that names a metric with no baseline, or omits the direction, fails
/// rather than defaulting to something plausible.
#[test]
fn gate_with_missing_metric_or_direction_fails() {
    let roster = MetricsRoster::parse("[quality_gates]\nmax_unwrap_calls = 11256\n").unwrap();
    let mk = |b: ThresholdBinding| CoherenceConfig {
        threshold_sections: vec!["quality_gates".into()],
        non_threshold_sections: BTreeMap::new(),
        binding: BTreeMap::from([("quality_gates.max_unwrap_calls".to_string(), b)]),
    };

    let mut no_metric = gate_binding(Some(200));
    no_metric.metric = None;
    let report = evaluate_coherence(
        &roster,
        &mk(no_metric),
        &measured(11_056),
        &unwrap_metric(200),
    );
    assert_eq!(report.outcome, Outcome::Fail);

    let mut no_direction = gate_binding(Some(200));
    no_direction.direction = None;
    let report = evaluate_coherence(
        &roster,
        &mk(no_direction),
        &measured(11_056),
        &unwrap_metric(200),
    );
    assert_eq!(report.outcome, Outcome::Fail);

    let unknown_metric = ThresholdBinding {
        metric: Some("nope".into()),
        ..gate_binding(Some(200))
    };
    let report = evaluate_coherence(
        &roster,
        &mk(unknown_metric),
        &measured(11_056),
        &unwrap_metric(200),
    );
    assert_eq!(report.outcome, Outcome::Fail);
}

/// A non-integer threshold cannot be a gate: the comparator is integer-only by
/// design, and silently rounding `0.60` would make the verdict depend on which
/// way the rounding went.
#[test]
fn float_or_string_threshold_cannot_be_a_gate() {
    let roster = MetricsRoster::parse("[quality_gates]\nmin_tdg_grade = \"A-\"\n").unwrap();
    let cfg = CoherenceConfig {
        threshold_sections: vec!["quality_gates".into()],
        non_threshold_sections: BTreeMap::new(),
        binding: BTreeMap::from([(
            "quality_gates.min_tdg_grade".to_string(),
            gate_binding(Some(200)),
        )]),
    };
    let report = evaluate_coherence(&roster, &cfg, &measured(11_056), &unwrap_metric(200));
    assert_eq!(report.outcome, Outcome::Fail);
    assert!(verdict_for(&report, "quality_gates.min_tdg_grade")
        .detail
        .contains("must be integers"));
}

// ─────────────────────────── config schema ───────────────────────────

/// A ratchet file the build does not understand is rejected outright. A
/// best-effort parse that drops half the metrics is a ratchet that quietly
/// stops ratcheting.
#[test]
fn unsupported_schema_version_is_rejected() {
    let err = RatchetConfig::parse(
        "version = 99\n[meta]\ncaptured_at_commit = \"x\"\ncaptured_at = \"2026-01-01\"\n\
         [coherence]\nthreshold_sections = []\n[coherence.non_threshold_sections]\n",
    )
    .unwrap_err();
    assert_eq!(err, ConfigError::UnsupportedVersion(99));
}

/// A missing ratchet file is a failure to measure, not an exemption.
#[test]
fn a_missing_ratchet_file_is_an_error_not_a_default() {
    let dir = tempfile::tempdir().expect("tempdir");
    let err = RatchetConfig::load(dir.path()).unwrap_err();
    assert_eq!(err, ConfigError::Missing(RATCHET_FILE.to_string()));
    let err = MetricsRoster::load(dir.path()).unwrap_err();
    assert_eq!(err, ConfigError::Missing(METRICS_FILE.to_string()));
}
