//! `.pmat-ratchet.toml` and `.pmat-metrics.toml` readers, plus the pure
//! evaluation that turns (roster x config x measurements) into a verdict.
//!
//! No measuring happens here. [`evaluate_coherence`] and [`evaluate_ratchet`]
//! take the measurements as an argument so that both are total functions of
//! their inputs and can be falsified from a unit test without a source tree.
//!
//! Contracts: `contracts/comply-ratchet-v1.yaml`,
//! `contracts/comply-threshold-coherence-v1.yaml`.

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::kernel::{classify, ratchet_verdict, Classification, Direction, RatchetVerdict};

/// Filename of the ratchet baseline file, at the project root.
pub const RATCHET_FILE: &str = ".pmat-ratchet.toml";
/// Filename of the threshold file the coherence gate audits.
pub const METRICS_FILE: &str = ".pmat-metrics.toml";
/// Schema version this build understands. A file declaring anything else is
/// rejected rather than best-effort parsed — a ratchet that silently ignores
/// half its own config is worse than no ratchet.
pub const RATCHET_SCHEMA_VERSION: u32 = 1;

// ─────────────────────────── .pmat-ratchet.toml ───────────────────────────

/// Parsed `.pmat-ratchet.toml`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RatchetConfig {
    /// Schema version; must equal [`RATCHET_SCHEMA_VERSION`].
    pub version: u32,
    /// Provenance of the captured baselines.
    pub meta: RatchetMeta,
    /// Per-metric baselines, keyed by metric id.
    #[serde(default)]
    pub metric: BTreeMap<String, MetricBaseline>,
    /// Threshold-coherence declarations for `.pmat-metrics.toml`.
    pub coherence: CoherenceConfig,
}

/// Where the baselines came from. Recorded so a reader can re-measure the exact
/// tree that produced them instead of trusting the numbers.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RatchetMeta {
    /// Commit the baselines were measured at.
    pub captured_at_commit: String,
    /// ISO date of the capture.
    pub captured_at: String,
}

/// One ratcheted metric.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetricBaseline {
    /// The captured value. The gate fails when the observed value EXCEEDS it
    /// (`INV-2102-1`).
    ///
    /// Every ratcheted metric is normalised so that **bigger is worse** — you
    /// declare the debt, never the virtue. A coverage ratchet is stored as
    /// uncovered lines, not covered ones. Without that rule `verdict` would
    /// need a direction, and a direction on a comparator is one more thing that
    /// can be set the wrong way round and silently invert a gate.
    pub baseline: i64,
    /// Unit of `baseline` and of every measurement of this metric, e.g.
    /// `count` or `basis_points`. Free text, but it must match what the
    /// measurement emits; it exists so a reader cannot mistake 8500 for 85%.
    pub unit: String,
    /// Tolerance used by the coherence gate when this metric backs a threshold.
    #[serde(default)]
    pub band: u64,
    /// Whether test code is inside the measurement. Explicit because the same
    /// metric measured both ways differs by ~2x in this repo, and a number
    /// whose scope is unstated is not a measurement.
    pub includes_test_files: bool,
    /// A shell one-liner that reproduces `baseline` from a clean checkout.
    ///
    /// Executable, not illustrative: [`super::measure::measure_all`] RUNS this
    /// for every declared metric on every gate run, and the number it prints is
    /// the number the ratchet judges. There is no second, in-process
    /// measurement to disagree with it.
    ///
    /// (This doc previously named a verifier, `command_reproduces_measurement`,
    /// that has never existed in this tree — the field was a promise nothing
    /// kept for a whole release, which is the shape of defect this rule is
    /// about. `drive_tests::every_committed_metric_command_still_measures` is
    /// the test that now holds it.)
    pub command: String,
    /// What is being counted, in one sentence.
    pub description: String,
    /// Required only to RAISE a baseline (`FALSIFY-2102-3`). Lowering, which is
    /// the only move the nightly job makes, needs nothing.
    #[serde(default)]
    pub justification: Option<String>,
    /// May this metric legitimately measure zero while its baseline is above
    /// zero? Defaults to `false`, which is the fail-closed answer.
    ///
    /// A measurement of 0 against a non-zero baseline has exactly two
    /// explanations — the largest improvement in the project's history, or a
    /// predicate that has rotted — and nothing in the measurement can tell them
    /// apart. A rotted `git grep` pathspec and a genuine zero are byte-identical
    /// at the shell: both print `0`, both exit 1, neither writes to stderr. So
    /// the exit-code guard in [`super::measure`] cannot catch it, and a ratchet
    /// that only ever looks upward greets it as perfection.
    ///
    /// Setting this to `true` is the deliberate, auditable override: one word in
    /// a committed file, reviewed like any other change, rather than a flag on
    /// a command line. Set it when a metric is genuinely converging on zero.
    #[serde(default)]
    pub zero_is_reachable: bool,
}

/// Declarations that make the `.pmat-metrics.toml` audit total.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CoherenceConfig {
    /// Sections of `.pmat-metrics.toml` whose scalars are thresholds.
    pub threshold_sections: Vec<String>,
    /// Sections that carry no thresholds, mapped to the reason. Present so
    /// that a NEW section is in neither list and fails closed instead of
    /// being silently exempt.
    pub non_threshold_sections: BTreeMap<String, String>,
    /// One entry per threshold, keyed `"<section>.<key>"`.
    #[serde(default)]
    pub binding: BTreeMap<String, ThresholdBinding>,
}

/// What a configured threshold claims to be.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum BindingKind {
    /// Backed by a measurement and enforced.
    Gate,
    /// Deliberately not enforced anywhere — a recorded budget. Requires a
    /// `justification`, and always classifies `VACUOUS`.
    Budget,
    /// Enforced by named code OUTSIDE the ratchet, which the gate verifies
    /// exists. Requires `enforced_by` naming a file that is present.
    External,
}

/// Declaration for one `.pmat-metrics.toml` threshold.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ThresholdBinding {
    /// Gate, budget, or externally enforced.
    pub kind: BindingKind,
    /// For `kind = "gate"`: the `[metric.*]` id supplying the measurement.
    #[serde(default)]
    pub metric: Option<String>,
    /// For `kind = "gate"`: which side of the threshold is the bad side.
    /// Required — a `max_*` key that is really a floor is exactly the kind of
    /// mislabelling this gate exists to catch, so it is never inferred from
    /// the key's name.
    #[serde(default)]
    pub direction: Option<Direction>,
    /// Override for the metric's band.
    #[serde(default)]
    pub band: Option<u64>,
    /// For `kind = "budget"` / a vacuous gate: why the number stays.
    #[serde(default)]
    pub justification: Option<String>,
    /// For `kind = "external"`: repo-relative path of the enforcing code.
    #[serde(default)]
    pub enforced_by: Option<String>,
}

/// Why a ratchet/coherence config could not be used. Every variant is a hard
/// failure of the gate — there is no "carry on without it" path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    /// The file is not there.
    Missing(String),
    /// It could not be read.
    Unreadable(String),
    /// It did not parse.
    Malformed(String),
    /// Schema version mismatch.
    UnsupportedVersion(u32),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Missing(p) => write!(f, "{p} is missing"),
            ConfigError::Unreadable(e) => write!(f, "unreadable: {e}"),
            ConfigError::Malformed(e) => write!(f, "malformed: {e}"),
            ConfigError::UnsupportedVersion(v) => {
                write!(
                    f,
                    "unsupported schema version {v} (expected {RATCHET_SCHEMA_VERSION})"
                )
            }
        }
    }
}

impl RatchetConfig {
    /// Read and validate `<project>/.pmat-ratchet.toml`.
    pub fn load(project_path: &Path) -> Result<Self, ConfigError> {
        let path = project_path.join(RATCHET_FILE);
        if !path.exists() {
            return Err(ConfigError::Missing(RATCHET_FILE.to_string()));
        }
        let raw = std::fs::read_to_string(&path)
            .map_err(|e| ConfigError::Unreadable(format!("{RATCHET_FILE}: {e}")))?;
        Self::parse(&raw)
    }

    /// Parse from a string. Separated from [`Self::load`] so the falsification
    /// tests exercise the schema without touching the filesystem.
    pub fn parse(raw: &str) -> Result<Self, ConfigError> {
        let cfg: RatchetConfig = toml::from_str(raw)
            .map_err(|e| ConfigError::Malformed(format!("{RATCHET_FILE}: {e}")))?;
        if cfg.version != RATCHET_SCHEMA_VERSION {
            return Err(ConfigError::UnsupportedVersion(cfg.version));
        }
        Ok(cfg)
    }
}

// ─────────────────────────── .pmat-metrics.toml ───────────────────────────

/// A scalar leaf of `.pmat-metrics.toml`, as written.
#[derive(Debug, Clone, PartialEq)]
pub enum ThresholdValue {
    /// TOML integer.
    Int(i64),
    /// TOML float.
    Float(f64),
    /// TOML string (e.g. `min_tdg_grade = "A-"`).
    Text(String),
}

impl std::fmt::Display for ThresholdValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThresholdValue::Int(i) => write!(f, "{i}"),
            ThresholdValue::Float(x) => write!(f, "{x}"),
            ThresholdValue::Text(s) => write!(f, "{s:?}"),
        }
    }
}

/// One threshold found in `.pmat-metrics.toml`.
#[derive(Debug, Clone, PartialEq)]
pub struct RosterEntry {
    /// `"<section>.<key>"`.
    pub key: String,
    /// Section it came from.
    pub section: String,
    /// Value as written.
    pub value: ThresholdValue,
}

/// Every section and every scalar threshold in `.pmat-metrics.toml`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MetricsRoster {
    /// Section names, in file order, deduplicated.
    pub sections: Vec<String>,
    /// Non-boolean scalar leaves of every section.
    pub thresholds: Vec<RosterEntry>,
}

impl MetricsRoster {
    /// Read `<project>/.pmat-metrics.toml`.
    pub fn load(project_path: &Path) -> Result<Self, ConfigError> {
        let path = project_path.join(METRICS_FILE);
        if !path.exists() {
            return Err(ConfigError::Missing(METRICS_FILE.to_string()));
        }
        let raw = std::fs::read_to_string(&path)
            .map_err(|e| ConfigError::Unreadable(format!("{METRICS_FILE}: {e}")))?;
        Self::parse(&raw)
    }

    /// Collect sections and scalar thresholds from TOML text.
    ///
    /// Booleans are excluded: `fail_on_stale_metrics = false` is a switch, not
    /// a threshold, and classifying it would be noise. Everything else that is
    /// a scalar under a table IS collected, including strings, so a threshold
    /// cannot dodge the audit by being written as `"A-"`.
    pub fn parse(raw: &str) -> Result<Self, ConfigError> {
        let doc: toml::Value = toml::from_str(raw)
            .map_err(|e| ConfigError::Malformed(format!("{METRICS_FILE}: {e}")))?;
        let table = doc
            .as_table()
            .ok_or_else(|| ConfigError::Malformed(format!("{METRICS_FILE}: not a table")))?;

        let mut roster = MetricsRoster::default();
        for (section, body) in table {
            let Some(body) = body.as_table() else {
                continue;
            };
            roster.sections.push(section.clone());
            for (key, value) in body {
                let v = match value {
                    toml::Value::Integer(i) => ThresholdValue::Int(*i),
                    toml::Value::Float(x) => ThresholdValue::Float(*x),
                    toml::Value::String(s) => ThresholdValue::Text(s.clone()),
                    _ => continue,
                };
                roster.thresholds.push(RosterEntry {
                    key: format!("{section}.{key}"),
                    section: section.clone(),
                    value: v,
                });
            }
        }
        roster.sections.sort();
        roster.sections.dedup();
        roster.thresholds.sort_by(|a, b| a.key.cmp(&b.key));
        Ok(roster)
    }
}

// ───────────────────────────── measurements ─────────────────────────────

/// A metric measurement, or the reason there isn't one.
///
/// `Unavailable` is a first-class result, not an `Option` to be
/// `unwrap_or(0)`-ed away. Every consumer must decide what to do with it, and
/// the answer is always "fail" (`FALSIFY-2101-3`, `FALSIFY-2102-4`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Measurement {
    /// Measured value in the metric's declared unit.
    Value(i64),
    /// Not measurable in this run, and why.
    Unavailable(String),
}

/// Measurements keyed by metric id.
pub type Measurements = BTreeMap<String, Measurement>;

/// Which `enforced_by` paths actually exist, resolved once by the caller.
///
/// [`evaluate_coherence`] must stay pure — it is the function the falsification
/// tests drive without a source tree — so the filesystem question is answered
/// outside it and handed in. The fail-closed direction follows: an index that
/// does not contain a path rejects it, so "we never looked" reads as "it is not
/// there" rather than as a pass.
///
/// This exists because the doc comment on [`ThresholdBinding::enforced_by`]
/// promised, for an entire release, that "the gate verifies exists" — and no
/// code anywhere performed that check. A binding could name
/// `src/does/not/exist.rs` and be reported as externally enforced. That is the
/// same shape of defect as the threshold this rule audits: a claim in a config
/// that nothing re-derives.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EnforcerIndex(BTreeMap<String, bool>);

impl EnforcerIndex {
    /// Resolve every `enforced_by` path declared in `cfg` against `root`.
    pub fn resolve(root: &Path, cfg: &CoherenceConfig) -> Self {
        EnforcerIndex(
            cfg.binding
                .values()
                .filter_map(|b| b.enforced_by.as_deref())
                .map(|p| (p.to_string(), root.join(p).exists()))
                .collect(),
        )
    }

    /// Build one directly, for tests.
    pub fn from_existing<I: IntoIterator<Item = S>, S: Into<String>>(paths: I) -> Self {
        EnforcerIndex(paths.into_iter().map(|p| (p.into(), true)).collect())
    }

    /// Does this path exist? Unknown paths are absent, never assumed present.
    pub fn exists(&self, path: &str) -> bool {
        self.0.get(path).copied().unwrap_or(false)
    }
}

// ─────────────────────────── evaluation (pure) ───────────────────────────

/// Outcome severity of one evaluated item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Outcome {
    /// Nothing to report.
    Ok,
    /// Reportable but not blocking.
    Warn,
    /// Blocking.
    Fail,
}

impl Outcome {
    /// The worse of two outcomes.
    pub fn worst(self, other: Outcome) -> Outcome {
        match (self, other) {
            (Outcome::Fail, _) | (_, Outcome::Fail) => Outcome::Fail,
            (Outcome::Warn, _) | (_, Outcome::Warn) => Outcome::Warn,
            _ => Outcome::Ok,
        }
    }
}

/// Per-threshold result of the coherence audit (CB-2101).
#[derive(Debug, Clone, Serialize)]
pub struct ThresholdVerdict {
    /// `"<section>.<key>"`.
    pub key: String,
    /// Value as written in `.pmat-metrics.toml`.
    pub configured: String,
    /// `gate` / `budget` / `external` / `undeclared`.
    pub kind: String,
    /// Metric id backing a gate, when there is one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metric: Option<String>,
    /// Measured value in the metric's unit, when it was measurable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub measured: Option<i64>,
    /// Tolerance that decided FIRING vs VACUOUS.
    pub band: u64,
    /// FIRING / VIOLATED / VACUOUS — always present (`INV-2101-3`).
    pub classification: Classification,
    /// Blocking status of this threshold.
    pub outcome: Outcome,
    /// Human-readable reason for `outcome`.
    pub detail: String,
}

/// Whole-file result of the coherence audit.
#[derive(Debug, Clone, Serialize)]
pub struct CoherenceReport {
    /// One verdict per threshold, sorted by key.
    pub thresholds: Vec<ThresholdVerdict>,
    /// Sections declared in neither list — each one fails the gate.
    pub undeclared_sections: Vec<String>,
    /// Worst outcome across everything.
    pub outcome: Outcome,
}

/// Classify every threshold in `roster` against `cfg` and `measurements`.
///
/// Total by construction: every roster entry produces exactly one
/// [`ThresholdVerdict`] carrying exactly one [`Classification`], and an entry
/// nobody declared produces a failing one rather than being skipped.
pub fn evaluate_coherence(
    roster: &MetricsRoster,
    cfg: &CoherenceConfig,
    measurements: &Measurements,
    metrics: &BTreeMap<String, MetricBaseline>,
    enforcers: &EnforcerIndex,
) -> CoherenceReport {
    let undeclared_sections: Vec<String> = roster
        .sections
        .iter()
        .filter(|s| {
            !cfg.threshold_sections.contains(s) && !cfg.non_threshold_sections.contains_key(*s)
        })
        .cloned()
        .collect();

    let mut thresholds = Vec::new();
    for entry in &roster.thresholds {
        if !cfg.threshold_sections.contains(&entry.section) {
            continue;
        }
        thresholds.push(evaluate_one_threshold(
            entry,
            cfg,
            measurements,
            metrics,
            enforcers,
        ));
    }

    let mut outcome = thresholds
        .iter()
        .fold(Outcome::Ok, |acc, t| acc.worst(t.outcome));
    if !undeclared_sections.is_empty() {
        outcome = Outcome::Fail;
    }

    CoherenceReport {
        thresholds,
        undeclared_sections,
        outcome,
    }
}

/// Vacuous-with-no-measurement verdict builder — the fail-closed default.
fn unfirable(
    entry: &RosterEntry,
    kind: &str,
    metric: Option<String>,
    outcome: Outcome,
    detail: String,
) -> ThresholdVerdict {
    ThresholdVerdict {
        key: entry.key.clone(),
        configured: entry.value.to_string(),
        kind: kind.to_string(),
        metric,
        measured: None,
        band: 0,
        // A threshold nothing measures cannot fire. That IS vacuity; naming it
        // anything else would let an unenforceable number read as a gate.
        classification: Classification::Vacuous,
        outcome,
        detail,
    }
}

fn evaluate_one_threshold(
    entry: &RosterEntry,
    cfg: &CoherenceConfig,
    measurements: &Measurements,
    metrics: &BTreeMap<String, MetricBaseline>,
    enforcers: &EnforcerIndex,
) -> ThresholdVerdict {
    let Some(binding) = cfg.binding.get(&entry.key) else {
        return unfirable(
            entry,
            "undeclared",
            None,
            Outcome::Fail,
            format!(
                "no [coherence.binding.\"{}\"] entry — an undeclared threshold is \
                 enforced by nothing and can never fire",
                entry.key
            ),
        );
    };

    match binding.kind {
        BindingKind::Budget => {
            let Some(j) = binding
                .justification
                .as_deref()
                .filter(|s| !s.trim().is_empty())
            else {
                return unfirable(
                    entry,
                    "budget",
                    None,
                    Outcome::Fail,
                    "declared a budget but carries no justification".to_string(),
                );
            };
            unfirable(
                entry,
                "budget",
                None,
                Outcome::Warn,
                format!("recorded budget, enforced by nothing: {j}"),
            )
        }
        BindingKind::External => {
            let Some(by) = binding
                .enforced_by
                .as_deref()
                .filter(|s| !s.trim().is_empty())
            else {
                return unfirable(
                    entry,
                    "external",
                    None,
                    Outcome::Fail,
                    "declared externally enforced but names no enforcing code".to_string(),
                );
            };
            if !enforcers.exists(by) {
                return unfirable(
                    entry,
                    "external",
                    None,
                    Outcome::Fail,
                    format!(
                        "declared enforced by '{by}', which does not exist — a binding that \
                         names a missing file records a belief, not an enforcement"
                    ),
                );
            }
            unfirable(
                entry,
                "external",
                None,
                Outcome::Warn,
                format!(
                    "enforced outside the ratchet by {by}; WARN not Pass because this gate \
                     can check that the reader exists and cannot check that the bound it \
                     enforces is this one"
                ),
            )
        }
        BindingKind::Gate => evaluate_gate(entry, binding, measurements, metrics),
    }
}

fn evaluate_gate(
    entry: &RosterEntry,
    binding: &ThresholdBinding,
    measurements: &Measurements,
    metrics: &BTreeMap<String, MetricBaseline>,
) -> ThresholdVerdict {
    let Some(metric_id) = binding.metric.as_deref() else {
        return unfirable(
            entry,
            "gate",
            None,
            Outcome::Fail,
            "declared a gate but names no metric".to_string(),
        );
    };
    let Some(baseline) = metrics.get(metric_id) else {
        return unfirable(
            entry,
            "gate",
            Some(metric_id.to_string()),
            Outcome::Fail,
            format!("names metric '{metric_id}', which has no [metric.*] baseline"),
        );
    };
    let limit = match &entry.value {
        ThresholdValue::Int(i) => *i,
        other => {
            return unfirable(
                entry,
                "gate",
                Some(metric_id.to_string()),
                Outcome::Fail,
                format!(
                    "gate thresholds must be integers in the metric's unit ({}); found {other}",
                    baseline.unit
                ),
            )
        }
    };
    let measured = match measurements.get(metric_id) {
        Some(Measurement::Value(v)) => *v,
        Some(Measurement::Unavailable(why)) => {
            return unfirable(
                entry,
                "gate",
                Some(metric_id.to_string()),
                Outcome::Fail,
                format!(
                    "metric '{metric_id}' is unmeasurable in this run ({why}); \
                         unmeasurable is not compliant"
                ),
            )
        }
        None => {
            return unfirable(
                entry,
                "gate",
                Some(metric_id.to_string()),
                Outcome::Fail,
                format!(
                    "metric '{metric_id}' produced no measurement in this run; \
                         unmeasurable is not compliant"
                ),
            )
        }
    };

    let Some(direction) = binding.direction else {
        return unfirable(
            entry,
            "gate",
            Some(metric_id.to_string()),
            Outcome::Fail,
            "declared a gate but does not say which side of the limit is the bad side \
             (direction = \"max\" | \"min\")"
                .to_string(),
        );
    };
    let band = binding.band.unwrap_or(baseline.band);
    let classification = classify(limit, measured, band, direction);
    let (outcome, detail) = match classification {
        Classification::Violated => (
            Outcome::Fail,
            format!(
                "limit {limit} is breached by the measured {measured} {} — the config \
                 asserts a bound this tree does not meet, and nothing turned red",
                baseline.unit
            ),
        ),
        Classification::Vacuous => {
            match binding
                .justification
                .as_deref()
                .filter(|s| !s.trim().is_empty())
            {
                Some(j) => (
                    Outcome::Warn,
                    format!(
                        "limit {limit} is {} beyond the measured {measured} {} (band {band}) \
                         so it can never fire: {j}",
                        (limit - measured).abs(),
                        baseline.unit
                    ),
                ),
                None => (
                    Outcome::Fail,
                    format!(
                        "limit {limit} is {} beyond the measured {measured} {} (band {band}) \
                         so it can never fire, and no justification is recorded",
                        (limit - measured).abs(),
                        baseline.unit
                    ),
                ),
            }
        }
        Classification::Firing => (
            Outcome::Ok,
            format!(
                "measured {measured} {} against limit {limit} (band {band})",
                baseline.unit
            ),
        ),
    };

    ThresholdVerdict {
        key: entry.key.clone(),
        configured: entry.value.to_string(),
        kind: "gate".to_string(),
        metric: Some(metric_id.to_string()),
        measured: Some(measured),
        band,
        classification,
        outcome,
        detail,
    }
}

/// Per-metric result of the ratchet (CB-2102).
#[derive(Debug, Clone, Serialize)]
pub struct MetricVerdict {
    /// Metric id.
    pub metric: String,
    /// Captured baseline.
    pub baseline: i64,
    /// Observed value, when measurable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observed: Option<i64>,
    /// Baseline the nightly job would write.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_baseline: Option<i64>,
    /// Pass/Fail against the baseline, when measurable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verdict: Option<RatchetVerdict>,
    /// Blocking status.
    pub outcome: Outcome,
    /// Reason.
    pub detail: String,
}

/// Whole-file result of the ratchet.
#[derive(Debug, Clone, Serialize)]
pub struct RatchetReport {
    /// One verdict per declared metric.
    pub metrics: Vec<MetricVerdict>,
    /// Baselines raised relative to the previous commit without justification.
    pub unjustified_raises: Vec<String>,
    /// Things that could not be judged at all. Each one is a failure — an
    /// empty ratchet is a gate that cannot fail, which is the shape this
    /// whole rule exists to find.
    pub holes: Vec<String>,
    /// Worst outcome across everything.
    pub outcome: Outcome,
}

/// Compare every declared baseline against `measurements`.
///
/// `previous` is the same map as parsed from the PREVIOUS commit's ratchet
/// file, used to detect an upward edit (`FALSIFY-2102-3`). `None` means there
/// was no previous file — the initial capture — which is not a raise.
pub fn evaluate_ratchet(
    metrics: &BTreeMap<String, MetricBaseline>,
    measurements: &Measurements,
    previous: Option<&BTreeMap<String, MetricBaseline>>,
) -> RatchetReport {
    let mut out = Vec::new();
    let mut unjustified_raises = Vec::new();

    for (id, baseline) in metrics {
        if let Some(prev) = previous.and_then(|p| p.get(id)) {
            // Bigger is worse for every ratcheted metric (see `MetricBaseline`),
            // so a raise is unambiguously an increase.
            let raised = baseline.baseline > prev.baseline;
            let justified = baseline
                .justification
                .as_deref()
                .is_some_and(|s| !s.trim().is_empty());
            if raised && !justified {
                unjustified_raises.push(format!(
                    "{id}: {} -> {} with no justification",
                    prev.baseline, baseline.baseline
                ));
            }
        }

        let verdict = match measurements.get(id) {
            Some(Measurement::Value(v)) => {
                let verdict = ratchet_verdict(baseline.baseline, *v);
                let next = super::kernel::next_baseline(baseline.baseline, *v);
                let (outcome, detail) = match verdict {
                    RatchetVerdict::Fail => (
                        Outcome::Fail,
                        // The command, not the commit: a finding a reader
                        // cannot reproduce is a finding they will argue with.
                        format!(
                            "{v} {} exceeds the baseline {} — reproduce with: {}",
                            baseline.unit, baseline.baseline, baseline.command
                        ),
                    ),
                    RatchetVerdict::Pass if next < baseline.baseline => (
                        Outcome::Ok,
                        format!(
                            "{v} {} is below the baseline {}; nightly will lower it to {next}",
                            baseline.unit, baseline.baseline
                        ),
                    ),
                    RatchetVerdict::Pass => (
                        Outcome::Ok,
                        format!(
                            "{v} {} at the baseline {}",
                            baseline.unit, baseline.baseline
                        ),
                    ),
                };
                MetricVerdict {
                    metric: id.clone(),
                    baseline: baseline.baseline,
                    observed: Some(*v),
                    next_baseline: Some(next),
                    verdict: Some(verdict),
                    outcome,
                    detail,
                }
            }
            Some(Measurement::Unavailable(why)) => MetricVerdict {
                metric: id.clone(),
                baseline: baseline.baseline,
                observed: None,
                next_baseline: None,
                verdict: None,
                outcome: Outcome::Fail,
                detail: format!("unmeasurable in this run ({why}); unmeasurable is not compliant"),
            },
            None => MetricVerdict {
                metric: id.clone(),
                baseline: baseline.baseline,
                observed: None,
                next_baseline: None,
                verdict: None,
                outcome: Outcome::Fail,
                detail: "absent from the measurement run; unmeasurable is not compliant"
                    .to_string(),
            },
        };
        out.push(verdict);
    }

    // An empty metric set is not a clean sheet. A ratchet that declares
    // nothing passes every run, forever, while reading as a gate in the
    // report — the exact shape CB-2100 was written to find, and one this
    // function used to have: `fold(Ok, ..)` over an empty vector is `Ok`.
    let mut holes = Vec::new();
    if metrics.is_empty() {
        holes.push(format!(
            "{RATCHET_FILE} declares no [metric.*] entries; a ratchet with no metrics cannot \
             fail and is not a gate"
        ));
    }

    let mut outcome = out.iter().fold(Outcome::Ok, |acc, m| acc.worst(m.outcome));
    if !unjustified_raises.is_empty() || !holes.is_empty() {
        outcome = Outcome::Fail;
    }

    RatchetReport {
        metrics: out,
        unjustified_raises,
        holes,
        outcome,
    }
}
