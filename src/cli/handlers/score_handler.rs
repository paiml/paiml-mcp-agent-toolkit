#![allow(unused)]
#![cfg_attr(coverage_nightly, coverage(off))]
//! Unified quality score handler (`pmat score`)
//!
//! Geometric composite of the sub-scores pmat could actually measure. Runs
//! comply + RPS internally, reads coverage/DBC from cache, writes all results
//! to .pmat-metrics/. A dimension with nothing to measure is reported as
//! `null` with a reason in `not_measured`, and is left out of the composite.
//! See docs/specifications/components/scoring-convergence.md

use crate::cli::handlers::comply_handlers::muda_handlers;
use crate::cli::handlers::work_contract::compute_codebase_score;
use crate::cli::RepoScoreOutputFormat;
use crate::services::rust_project_score::models::ScoringMode;
use crate::services::rust_project_score::orchestrator::RustProjectScoreOrchestrator;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// The outcome of one sub-score.
///
/// `Ok(v)` means pmat measured this dimension of this project and got `v`.
/// `Err(reason)` means there was nothing to measure — it is the *absence* of a
/// score, not a low score. Four dimensions used to answer the literal `50.0`
/// for every project on earth (coverage, evoscore, dbc, pv_lint) because a
/// mid-range number was used to mean "unmeasured"; a reader could not tell that
/// apart from a measurement, and three of the four were folded into the
/// geometric mean as if they were one.
pub type Dimension = std::result::Result<f64, String>;

/// A dimension the composite does not cover, and why.
///
/// Emitted in the report so a reader can see what the number does not include.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotMeasured {
    /// Sub-score name — the same key used in `sub_scores`.
    pub dimension: String,
    /// Why nothing was measured, naming the artifact that would make it
    /// measurable.
    pub reason: String,
}

/// Every dimension of the composite, in report order.
pub const DIMENSIONS: [&str; 8] = [
    "rps",
    "comply",
    "coverage",
    "muda_inv",
    "evoscore",
    "dbc",
    "file_health",
    "pv_lint",
];

/// Composite score with all sub-scores for persistence and display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositeScore {
    pub sha: String,
    pub timestamp: String,
    /// Geometric mean of the *measured* sub-scores, or `None` when no
    /// dimension could be measured (in which case there is no score to report,
    /// rather than a 0.0 that reads as "measured, terrible").
    #[serde(default)]
    pub composite: Option<f64>,
    pub grade: String,
    pub sub_scores: SubScores,
    /// Dimensions excluded from `composite` because they were not measured.
    #[serde(default)]
    pub not_measured: Vec<NotMeasured>,
    /// How many of the [`DIMENSIONS`] the composite actually covers.
    #[serde(default)]
    pub dimensions_measured: usize,
    /// How many dimensions exist in total (`DIMENSIONS.len()`).
    #[serde(default)]
    pub dimensions_total: usize,
    pub rps_categories: HashMap<String, f64>,
    pub comply_errors: usize,
    pub comply_warnings: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Sub scores. `None` (JSON `null`) means the dimension was not measured — see
/// `CompositeScore::not_measured` for the reason.
pub struct SubScores {
    #[serde(default)]
    pub rps: Option<f64>,
    #[serde(default)]
    pub comply: Option<f64>,
    #[serde(default)]
    pub coverage: Option<f64>,
    #[serde(default)]
    pub muda_inv: Option<f64>,
    #[serde(default)]
    pub evoscore: Option<f64>,
    #[serde(default)]
    pub dbc: Option<f64>,
    #[serde(default)]
    pub file_health: Option<f64>,
    #[serde(default)]
    pub pv_lint: Option<f64>,
}

/// Handle the `pmat score` command.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_score(
    path: &Path,
    gate: Option<f64>,
    format: &RepoScoreOutputFormat,
    output: Option<&Path>,
    trend: bool,
    regression_check: bool,
    stack: bool,
) -> Result<()> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    if !path.exists() || !path.is_dir() {
        anyhow::bail!("Path is not a valid directory: {}", path.display());
    }

    // Trend mode: show history without running a new score
    if trend {
        print_trend(path);
        return Ok(());
    }

    crate::status_eprintln!("Computing unified quality score...");

    let score = compute_composite(path).await?;
    debug_assert!(
        score.composite.is_none_or(|c| (0.0..=100.0).contains(&c)),
        "composite score out of range: {:?}",
        score.composite
    );

    // Persist to .pmat-metrics/
    persist_score(path, &score);

    // Format output
    let output_text = render_score(&score, format)?;

    if let Some(output_path) = output {
        std::fs::write(output_path, &output_text)?;
        crate::status_eprintln!("Score written to: {}", output_path.display());
    } else {
        println!("{}", output_text);
    }

    // Stack quality (CB-150)
    if stack {
        print_stack_quality(path);
    }

    // Cross-validation (CB-146)
    let violations = cross_validate(&score);
    if !violations.is_empty() {
        eprintln!(
            "\nCross-validation ({}/{} invariants violated):",
            violations.len(),
            CROSS_VALIDATION_INVARIANTS
        );
        for v in &violations {
            eprintln!("  {} {}", v.id, v.message);
        }
        if violations.len() >= 3 {
            eprintln!("WARNING: 3+ invariants violated — systemic inconsistency");
        }
    }

    // Regression check (CB-145)
    if regression_check {
        if let Some(delta) = check_regression(path, &score) {
            if delta < -5.0 {
                eprintln!(
                    "REGRESSION: composite dropped {:.1} pts (threshold: -5.0)",
                    delta
                );
                std::process::exit(1);
            }
        }
    }

    // Gate check (CB-147)
    if let Some(threshold) = gate {
        // An unmeasured dimension is *excluded* from the composite, which can
        // only push the mean up. Disclose the coverage of the number the gate
        // is about to accept, so "we never measured it" cannot read as a pass.
        if !score.not_measured.is_empty() {
            eprintln!(
                "GATE SCOPE: composite covers {}/{} dimensions; not measured: {}",
                score.dimensions_measured,
                score.dimensions_total,
                score.not_measured_summary()
            );
        }
        match score.composite {
            Some(composite) if composite >= threshold => {}
            Some(composite) => {
                eprintln!("FAIL: composite {composite:.1} < gate {threshold:.1}");
                std::process::exit(1);
            }
            None => {
                eprintln!(
                    "FAIL: composite is not measured (0/{} dimensions), so gate {:.1} cannot pass",
                    score.dimensions_total, threshold
                );
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

impl CompositeScore {
    /// `dimension (reason), dimension (reason)` — for one-line disclosure.
    fn not_measured_summary(&self) -> String {
        self.not_measured
            .iter()
            .map(|n| format!("{} ({})", n.dimension, n.reason))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// Render a computed score in the requested output format.
///
/// Every non-JSON variant used to fall through to the ANSI text renderer, so
/// `-f yaml` and `-f markdown` emitted byte-identical banner text despite --help
/// promising "YAML format" and "Markdown format with tables". Each advertised
/// format now has its own arm, and the match is exhaustive so a new variant
/// cannot silently inherit the text renderer again.
fn render_score(score: &CompositeScore, format: &RepoScoreOutputFormat) -> Result<String> {
    Ok(match format {
        RepoScoreOutputFormat::Json => serde_json::to_string_pretty(score)?,
        RepoScoreOutputFormat::Yaml => serde_yaml_ng::to_string(score)?,
        RepoScoreOutputFormat::Markdown => format_markdown(score),
        RepoScoreOutputFormat::Text => format_text(score),
    })
}

/// Render the composite score as Markdown tables (`-f markdown`).
fn format_markdown(score: &CompositeScore) -> String {
    use std::fmt::Write as _;
    let mut out = String::new();
    out.push_str("# PMAT Unified Score\n\n");
    let _ = match score.composite {
        Some(c) => writeln!(out, "- **Composite**: {c:.1}/100"),
        None => writeln!(out, "- **Composite**: not measured"),
    };
    let _ = writeln!(out, "- **Grade**: {}", score.grade);
    let _ = writeln!(
        out,
        "- **Dimensions measured**: {}/{}",
        score.dimensions_measured, score.dimensions_total
    );
    let _ = writeln!(out, "- **Commit**: {}", score.sha);
    let _ = writeln!(out, "- **Timestamp**: {}\n", score.timestamp);

    out.push_str("## Sub-Scores\n\n");
    out.push_str("| Sub-Score | Value |\n|---|---:|\n");
    let s = &score.sub_scores;
    let _ = writeln!(out, "| RPS | {} |", md_value(s.rps));
    let _ = writeln!(
        out,
        "| Comply | {} ({} errors, {} warnings) |",
        md_value(s.comply),
        score.comply_errors,
        score.comply_warnings
    );
    let _ = writeln!(out, "| Coverage | {} |", md_value(s.coverage));
    let _ = writeln!(out, "| Muda (inv) | {} |", md_value(s.muda_inv));
    let _ = writeln!(out, "| EvoScore | {} |", md_value(s.evoscore));
    let _ = writeln!(out, "| DBC | {} |", md_value(s.dbc));
    let _ = writeln!(out, "| File Health | {} |", md_value(s.file_health));
    let _ = writeln!(out, "| PV Lint | {} |", md_value(s.pv_lint));

    if !score.not_measured.is_empty() {
        out.push_str("\n## Not Measured (excluded from the composite)\n\n");
        out.push_str("| Dimension | Reason |\n|---|---|\n");
        for n in &score.not_measured {
            let _ = writeln!(out, "| {} | {} |", n.dimension, n.reason);
        }
    }

    if !score.rps_categories.is_empty() {
        out.push_str("\n## RPS Categories\n\n");
        out.push_str("| Category | Percent |\n|---|---:|\n");
        let mut cats: Vec<_> = score.rps_categories.iter().collect();
        cats.sort_by(|a, b| a.0.cmp(b.0));
        for (name, pct) in cats {
            let _ = writeln!(out, "| {name} | {pct:.1} |");
        }
    }
    out
}

/// A sub-score cell: the number, or an explicit "not measured".
fn md_value(value: Option<f64>) -> String {
    match value {
        Some(v) => format!("{v:.1}"),
        None => "not measured".to_string(),
    }
}

/// Compute the geometric composite from all sub-scores.
async fn compute_composite(path: &Path) -> Result<CompositeScore> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let sha = get_head_sha(path);
    let timestamp = chrono::Utc::now().to_rfc3339();

    // 1. RPS — run once, extract percentage + categories
    let (rps, rps_categories) = compute_rps(path);

    // 2. Comply (error/warning penalty)
    let (comply, comply_errors, comply_warnings) = compute_comply(path).await;

    // 3. Muda inverse (100 - waste score)
    let muda_inv = compute_muda_inv(path);

    // 4. Coverage from cache
    let coverage = read_coverage_cache(path);

    // 5. EvoScore from test history
    let evoscore = compute_evoscore(path);

    // 6. DBC portfolio score
    let dbc = compute_dbc(path);

    // 7. File health
    let file_health = compute_file_health(path);

    // 8. PV Lint (provable contracts)
    let pv_lint = compute_pv_lint(path);

    // One list, in DIMENSIONS order — the single place that decides what the
    // composite covers.
    let dimensions: [Dimension; 8] = [
        rps,
        comply,
        coverage,
        muda_inv,
        evoscore,
        dbc,
        file_health,
        pv_lint,
    ];

    Ok(assemble_score(
        sha,
        timestamp,
        dimensions,
        rps_categories,
        comply_errors,
        comply_warnings,
    ))
}

/// Fold the eight dimensions into a report.
///
/// Pure, and the only place that decides what the composite covers: a
/// dimension with no value cannot enter the mean, because `Err` carries no
/// value to enter it with.
fn assemble_score(
    sha: String,
    timestamp: String,
    dimensions: [Dimension; 8],
    rps_categories: HashMap<String, f64>,
    comply_errors: usize,
    comply_warnings: usize,
) -> CompositeScore {
    debug_assert_eq!(dimensions.len(), DIMENSIONS.len());

    // Precondition: every *measured* sub-score is in range. An unmeasured one
    // has no value to be in range.
    for (name, dim) in DIMENSIONS.iter().zip(dimensions.iter()) {
        debug_assert!(
            dim.as_ref().is_ok_and(|v| (0.0..=100.0).contains(v)) || dim.is_err(),
            "{name} out of range: {dim:?}"
        );
    }

    // Geometric mean over the measured dimensions only. Exclusion is by
    // construction (`Err` carries no value) — not by comparing a float against
    // a magic sentinel, which is how coverage/evoscore/dbc used to sneak their
    // "unmeasured" 50.0 into the mean while pv_lint's identical 50.0 was
    // skipped.
    let values: Vec<f64> = dimensions
        .iter()
        .filter_map(|d| d.as_ref().ok().copied())
        .collect();
    let not_measured: Vec<NotMeasured> = DIMENSIONS
        .iter()
        .zip(dimensions.iter())
        .filter_map(|(name, d)| {
            d.as_ref().err().map(|reason| NotMeasured {
                dimension: (*name).to_string(),
                reason: reason.clone(),
            })
        })
        .collect();

    let composite = if values.is_empty() {
        None // nothing measured — there is no score, and 0.0 would be a lie
    } else {
        Some(geometric_mean(values.as_slice()))
    };
    debug_assert!(
        composite.is_none_or(|c| (0.0..=100.0).contains(&c)),
        "geometric mean out of range: {composite:?}"
    );

    let grade = grade_for(composite);
    let [rps, comply, coverage, muda_inv, evoscore, dbc, file_health, pv_lint] = dimensions;

    CompositeScore {
        sha,
        timestamp,
        composite,
        grade,
        sub_scores: SubScores {
            rps: rps.ok(),
            comply: comply.ok(),
            coverage: coverage.ok(),
            muda_inv: muda_inv.ok(),
            evoscore: evoscore.ok(),
            dbc: dbc.ok(),
            file_health: file_health.ok(),
            pv_lint: pv_lint.ok(),
        },
        dimensions_measured: values.len(),
        dimensions_total: DIMENSIONS.len(),
        not_measured,
        rps_categories,
        comply_errors,
        comply_warnings,
    }
}

/// Letter grade for a composite — `n/a` when there is no composite to grade.
fn grade_for(composite: Option<f64>) -> String {
    let Some(composite) = composite else {
        return "n/a".to_string();
    };
    match composite as u32 {
        90..=100 => "A",
        80..=89 => "B",
        70..=79 => "C",
        60..=69 => "D",
        _ => "F",
    }
    .to_string()
}

/// Run RPS once, return (percentage, category_percentages).
///
/// A failed orchestrator run used to report `0.0`, which zero-absorbs the
/// geometric mean and turns "we could not run RPS" into a composite of 0.
fn compute_rps(path: &Path) -> (Dimension, HashMap<String, f64>) {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let orchestrator = RustProjectScoreOrchestrator::new();
    match orchestrator.score_with_mode(path, ScoringMode::Fast) {
        Ok(score) => {
            let cats = score
                .categories
                .iter()
                .map(|(k, v)| {
                    let pct = if v.max > 0.0 {
                        v.earned / v.max * 100.0
                    } else {
                        0.0
                    };
                    (k.clone(), pct)
                })
                .collect();
            (Ok(score.percentage), cats)
        }
        Err(e) => (
            Err(format!("the Rust Project Score run failed: {e}")),
            HashMap::new(),
        ),
    }
}

async fn compute_comply(path: &Path) -> (Dimension, usize, usize) {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    // Run pmat comply check --format json as subprocess to avoid internal coupling
    let output = std::process::Command::new("pmat")
        .args(["comply", "check", "--format", "json"])
        .current_dir(path)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output();

    match output {
        Ok(o) if o.status.success() || o.status.code() == Some(1) => {
            // Both exit 0 (compliant) and exit 1 (non-compliant) produce valid JSON
            if let Ok(content) = String::from_utf8(o.stdout) {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(checks) = val.get("checks").and_then(|c| c.as_array()) {
                        let errors = checks
                            .iter()
                            .filter(|c| c.get("status").and_then(|s| s.as_str()) == Some("Fail"))
                            .count();
                        let warnings = checks
                            .iter()
                            .filter(|c| c.get("status").and_then(|s| s.as_str()) == Some("Warn"))
                            .count();
                        let score =
                            (100.0_f64 - (errors as f64 * 10.0 + warnings as f64 * 3.0)).max(0.0);
                        return (Ok(score), errors, warnings);
                    }
                }
            }
            (Err(COMPLY_UNMEASURED.to_string()), 0, 0)
        }
        _ => (Err(COMPLY_UNMEASURED.to_string()), 0, 0),
    }
}

const COMPLY_UNMEASURED: &str =
    "`pmat comply check --format json` produced no parsable report (is `pmat` on PATH?)";

fn compute_muda_inv(path: &Path) -> Dimension {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let report = muda_handlers::calculate_muda_score(path);
    Ok((100.0 - report.total_score).max(0.0))
}

/// Coverage is *read*, never computed: `pmat score` does not run a coverage
/// tool. When the cache is absent there is no coverage number for this
/// project, and saying so beats inventing a mid-range one.
fn read_coverage_cache(path: &Path) -> Dimension {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let coverage_result = path.join(".pmat-metrics/coverage.result");
    let Ok(content) = std::fs::read_to_string(&coverage_result) else {
        return Err(format!(
            "no coverage run recorded: {} is absent (`pmat score` does not run \
             coverage; `make coverage` / scripts/record-metric.sh writes it)",
            coverage_result.display()
        ));
    };
    let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) else {
        return Err(format!("{} is not valid JSON", coverage_result.display()));
    };
    let Some(pct) = val.get("coverage_pct").and_then(|v| v.as_f64()) else {
        return Err(format!(
            "{} has no numeric `coverage_pct` key",
            coverage_result.display()
        ));
    };
    if !(0.0..=100.0).contains(&pct) {
        return Err(format!(
            "{} reports coverage_pct {pct}, which is not a percentage",
            coverage_result.display()
        ));
    }
    Ok(pct)
}

fn compute_dbc(path: &Path) -> Dimension {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let score = compute_codebase_score(path);
    if score.contract_count == 0 {
        return Err(format!(
            "no design-by-contract work items: {} holds no <id>/contract.json",
            path.join(".pmat-work").display()
        ));
    }
    // Use the best signal: contract coverage and lint pass rate
    // These are binary quality indicators (did you write contracts? do they lint?)
    // more actionable than the abstract mean_score
    let coverage_pct = score.contract_coverage * 100.0;
    let lint_pct = score.lint_pass_rate * 100.0;
    // Weighted: 50% coverage, 30% lint, 20% mean contract score
    let dbc_score = 0.50 * coverage_pct + 0.30 * lint_pct + 0.20 * (score.mean_score * 100.0);
    Ok(dbc_score.clamp(0.0, 100.0))
}

/// Pass rate of the most recently recorded test run.
///
/// Records come from `pmat test-record`; a project that has never recorded one
/// has no test history to score. The record is chosen by file name rather than
/// by `read_dir` order, which is not defined and made the result depend on the
/// filesystem when several commits were recorded.
fn compute_evoscore(path: &Path) -> Dimension {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    // Read test results from .pmat-metrics/commit-*-tests.json
    let metrics_dir = path.join(".pmat-metrics");
    let mut test_records: Vec<(String, String, u64, u64)> = Vec::new(); // (file, sha, pass, total)

    if let Ok(entries) = std::fs::read_dir(&metrics_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with("commit-") && name_str.ends_with("-tests.json") {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                        let pass = val.get("pass").and_then(|v| v.as_u64()).unwrap_or(0);
                        let total = val.get("total").and_then(|v| v.as_u64()).unwrap_or(pass);
                        let sha = val
                            .get("commit")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        if total > 0 {
                            test_records.push((name_str.to_string(), sha, pass, total));
                        }
                    }
                }
            }
        }
    }

    if test_records.is_empty() {
        return Err(format!(
            "no test history: {} holds no commit-<sha>-tests.json (written by `pmat test-record`)",
            metrics_dir.display()
        ));
    }

    // Prefer the record for the commit being scored; otherwise the
    // lexicographically last file name, so the answer is deterministic.
    test_records.sort_by(|a, b| a.0.cmp(&b.0));
    let head = get_head_sha(path);
    let (_, _, pass, total) = test_records
        .iter()
        .find(|(_, sha, _, _)| !sha.is_empty() && head.starts_with(sha.as_str()))
        .or_else(|| test_records.last())
        .expect("test_records is non-empty");
    let rate = *pass as f64 / *total as f64;
    Ok((rate * 100.0).clamp(0.0, 100.0))
}

// Computation functions (PV lint, coverage, DBC, etc.) extracted for CB-040
include!("score_handler_compute.rs");

// Display, trend, stack quality, and history functions extracted for CB-040
include!("score_handler_display.rs");

/// `pmat score --quiet` was byte-identical to `pmat score`.
///
/// The small fixture used during triage showed no stderr at all, so `score`
/// was first filed as "emits only its report". On the flag-efficacy gate's
/// ~120-file corpus it emits 55 bytes: "Computing unified quality score..."
/// from this handler and "✅ Analysis complete" from the RPS orchestrator's
/// progress bar. Both are chatter and are now suppressible.
///
/// The other direction matters more: the gate failure, the regression trip and
/// the cross-validation violations are *results*, and this test pins that they
/// keep using an unguarded stderr macro. `--quiet` is documented as "errors
/// only"; a suppressed `FAIL: composite ... < gate ...` would be a far worse
/// defect than a surviving banner.
#[cfg(test)]
mod score_quiet_chatter_tests {
    use crate::cli::handlers::bottleneck_handler::quiet_chatter_tests::unguarded_stderr_lines;

    const SOURCE: &str = include_str!("score_handler.rs");

    /// The stderr call that emits `needle`.
    ///
    /// The message is often on a later line than the macro that prints it, so
    /// looking only at the message's own line reads every multi-line call as
    /// "no stderr macro here" and the test passes vacuously in both
    /// directions.
    fn call_line_for(needle: &str) -> &'static str {
        let lines: Vec<&str> = SOURCE.lines().collect();
        let at = lines
            .iter()
            .position(|l| l.contains(needle))
            .unwrap_or_else(|| panic!("no line carries {needle:?}"));
        let macro_needle = concat!("eprint", "ln!");
        lines[..=at]
            .iter()
            .rev()
            .take(4)
            .find(|l| l.contains(macro_needle))
            .unwrap_or_else(|| panic!("{needle:?} is not printed to stderr at all"))
    }

    #[test]
    fn score_progress_is_suppressible() {
        for chatter in ["Computing unified quality score", "Score written to:"] {
            let line = call_line_for(chatter);
            assert!(
                unguarded_stderr_lines(line).is_empty(),
                "{chatter:?} is progress chatter and must route through the \
                 quiet rule, but its call is unguarded: {line:?}"
            );
        }
    }

    #[test]
    fn score_failures_stay_loud() {
        for verdict in [
            "FAIL: composite",
            "REGRESSION: composite dropped",
            "WARNING: 3+ invariants violated",
        ] {
            let line = call_line_for(verdict);
            assert!(
                !unguarded_stderr_lines(line).is_empty(),
                "{verdict:?} is a result, not chatter: --quiet must not \
                 suppress it, but its call is guarded: {line:?}"
            );
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    //! Tests for include!'d score_handler_compute.rs — PMAT-642 Phase-1 pivot.
    //! Cover pure fs + compute functions (no async) to move broad coverage.
    use super::*;
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn write(path: &std::path::Path, content: &str) {
        std::fs::write(path, content).expect("write");
    }

    fn mkdir(path: &std::path::Path) {
        std::fs::create_dir_all(path).expect("mkdir");
    }

    fn dummy_score(sub: SubScores, composite: f64, comply_errors: usize) -> CompositeScore {
        CompositeScore {
            sha: "abc1234".into(),
            timestamp: "2026-04-24T08:00:00Z".into(),
            composite: Some(composite),
            grade: "B".into(),
            sub_scores: sub,
            not_measured: Vec::new(),
            dimensions_measured: DIMENSIONS.len(),
            dimensions_total: DIMENSIONS.len(),
            rps_categories: HashMap::new(),
            comply_errors,
            comply_warnings: 0,
        }
    }

    fn zero_subs() -> SubScores {
        SubScores {
            rps: Some(0.0),
            comply: Some(0.0),
            coverage: Some(0.0),
            muda_inv: Some(0.0),
            evoscore: Some(0.0),
            dbc: Some(0.0),
            file_health: Some(0.0),
            pv_lint: Some(0.0),
        }
    }

    // --- geometric_mean ---

    #[test]
    fn test_geometric_mean_empty_returns_zero() {
        assert_eq!(geometric_mean(&[]), 0.0);
    }

    #[test]
    fn test_geometric_mean_single_value_returns_that_value() {
        let result = geometric_mean(&[42.0]);
        assert!((result - 42.0).abs() < 1e-10, "got {result}");
    }

    #[test]
    fn test_geometric_mean_all_positive() {
        // Geometric mean of 2, 8 = sqrt(16) = 4
        let result = geometric_mean(&[2.0, 8.0]);
        assert!((result - 4.0).abs() < 1e-10, "got {result}");
    }

    #[test]
    fn test_geometric_mean_with_zero_is_zero() {
        assert_eq!(geometric_mean(&[0.0, 100.0]), 0.0);
        assert_eq!(geometric_mean(&[50.0, 0.0, 50.0]), 0.0);
    }

    #[test]
    fn test_geometric_mean_three_values() {
        // cube_root(8 * 27 * 64) = cube_root(13824) = 24
        let result = geometric_mean(&[8.0, 27.0, 64.0]);
        assert!((result - 24.0).abs() < 1e-6, "got {result}");
    }

    // --- compute_pv_lint (no contracts dir branches) ---

    #[test]
    fn test_pv_lint_no_contracts_no_pmat_yaml_no_src_is_not_measured() {
        let tmp = TempDir::new().unwrap();
        let reason = compute_pv_lint(tmp.path()).expect_err("no contracts/ ⇒ nothing to measure");
        assert!(reason.contains("no provable contracts"), "reason: {reason}");
    }

    #[test]
    fn test_pv_lint_no_contracts_cb1202_disabled_is_not_measured() {
        let tmp = TempDir::new().unwrap();
        write(
            &tmp.path().join(".pmat.yaml"),
            "checks:\n  cb-1202:\n    enabled: false\n",
        );
        let reason = compute_pv_lint(tmp.path()).expect_err("cb-1202 off ⇒ nothing to measure");
        assert!(reason.contains("cb-1202"), "reason: {reason}");
    }

    #[test]
    fn test_pv_lint_no_contracts_src_with_critical_keyword_returns_zero() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src");
        mkdir(&src);
        write(
            &src.join("lib.rs"),
            "pub fn forward(x: f64) -> f64 { x * 2.0 }",
        );
        // Has "pub fn forward" — one of the critical ML/compiler keywords.
        assert_eq!(compute_pv_lint(tmp.path()), Ok(0.0));
    }

    #[test]
    fn test_pv_lint_no_contracts_src_without_critical_keyword_is_not_measured() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src");
        mkdir(&src);
        write(&src.join("lib.rs"), "pub fn mundane() -> i32 { 42 }");
        assert!(compute_pv_lint(tmp.path()).is_err());
    }

    #[test]
    fn test_pv_lint_with_contracts_but_no_pv_cli_fallback_to_pipeline() {
        // pv CLI likely unavailable in test env → falls back to pipeline-depth.
        // With only an empty contracts dir, pipeline depth is 0 → 50.0 (clamp min).
        let tmp = TempDir::new().unwrap();
        mkdir(&tmp.path().join("contracts"));
        let score = compute_pv_lint(tmp.path()).expect("contracts/ exists ⇒ measured");
        assert!(
            (50.0..=95.0).contains(&score),
            "expected clamp [50,95], got {score}"
        );
    }

    // --- compute_pipeline_depth ---

    #[test]
    fn test_pipeline_depth_empty_project_is_zero() {
        let tmp = TempDir::new().unwrap();
        mkdir(&tmp.path().join("contracts"));
        assert_eq!(compute_pipeline_depth(tmp.path()), 0.0);
    }

    #[test]
    fn test_pipeline_depth_yaml_contracts_adds_five() {
        let tmp = TempDir::new().unwrap();
        let cd = tmp.path().join("contracts");
        mkdir(&cd);
        write(&cd.join("a.yaml"), "name: a\n");
        // Only YAML contract contributor hits → 5pts
        assert!((compute_pipeline_depth(tmp.path()) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_pipeline_depth_build_rs_pre_count_adds_five() {
        let tmp = TempDir::new().unwrap();
        mkdir(&tmp.path().join("contracts"));
        write(&tmp.path().join("build.rs"), "// PRE_COUNT=1\n");
        // 5pts from build.rs (no YAML, no src, no lean, no kani, no proof-status)
        assert!((compute_pipeline_depth(tmp.path()) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_pipeline_depth_contract_macro_in_src_adds_five() {
        let tmp = TempDir::new().unwrap();
        mkdir(&tmp.path().join("contracts"));
        let src = tmp.path().join("src");
        mkdir(&src);
        write(
            &src.join("lib.rs"),
            "#[contract(\"a\", equation = \"b\")]\npub fn f() {}",
        );
        // 5pts from contract macro only (no YAML, no build.rs, etc.)
        assert!((compute_pipeline_depth(tmp.path()) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_pipeline_depth_yaml_lean_theorem_and_kani_stacks_fifteen() {
        let tmp = TempDir::new().unwrap();
        let cd = tmp.path().join("contracts");
        mkdir(&cd);
        write(
            &cd.join("a.yaml"),
            "name: a\nlean_theorem: foo\nkani_harnesses:\n  - bar\n",
        );
        // 5 (yaml exists) + 5 (lean_theorem ref) + 5 (kani ref) = 15
        assert!((compute_pipeline_depth(tmp.path()) - 15.0).abs() < 1e-10);
    }

    // --- compute_contract_drift ---

    #[test]
    fn test_contract_drift_no_contracts_dir_returns_zeros() {
        let tmp = TempDir::new().unwrap();
        let (stale, total, ratio) = compute_contract_drift(tmp.path());
        assert_eq!((stale, total, ratio), (0, 0, 0.0));
    }

    #[test]
    fn test_contract_drift_yaml_without_matching_src_not_stale() {
        let tmp = TempDir::new().unwrap();
        let cd = tmp.path().join("contracts");
        mkdir(&cd);
        write(&cd.join("orphan.yaml"), "name: orphan\n");
        // No src referring to "orphan" → no mtime comparison → stale=0.
        let (stale, total, _) = compute_contract_drift(tmp.path());
        assert_eq!(stale, 0);
        assert_eq!(total, 1);
    }

    #[test]
    fn test_contract_drift_skips_binding_yaml() {
        let tmp = TempDir::new().unwrap();
        let cd = tmp.path().join("contracts");
        mkdir(&cd);
        write(&cd.join("binding-spec.yaml"), "name: bind\n");
        write(&cd.join("real.yaml"), "name: real\n");
        // binding yaml is skipped → total counts only "real.yaml".
        let (_, total, _) = compute_contract_drift(tmp.path());
        assert_eq!(total, 1);
    }

    // --- compute_file_health ---

    /// A project with no sources has no file health. Reporting 100.0 there
    /// scored a codebase that does not exist, and made the empty and the
    /// healthy project indistinguishable.
    #[test]
    fn test_file_health_no_src_is_not_measured() {
        let tmp = TempDir::new().unwrap();
        let reason = compute_file_health(tmp.path()).expect_err("no src/ ⇒ nothing to measure");
        assert!(reason.contains("no Rust sources"), "reason: {reason}");
    }

    #[test]
    fn test_file_health_empty_src_is_not_measured() {
        let tmp = TempDir::new().unwrap();
        mkdir(&tmp.path().join("src"));
        assert!(compute_file_health(tmp.path()).is_err());
    }

    #[test]
    fn test_file_health_all_small_files_returns_100() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src");
        mkdir(&src);
        write(&src.join("a.rs"), "fn a() {}");
        write(&src.join("b.rs"), "fn b() {}");
        assert_eq!(compute_file_health(tmp.path()), Ok(100.0));
    }

    #[test]
    fn test_file_health_one_huge_file_in_two_drops_to_fifty() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src");
        mkdir(&src);
        write(&src.join("small.rs"), "fn a() {}");
        let big: String = (0..1100).map(|i| format!("// line {i}\n")).collect();
        write(&src.join("big.rs"), &big);
        // 1 of 2 files is >1000 lines → 50%.
        let result = compute_file_health(tmp.path()).expect("2 .rs files ⇒ measured");
        assert!((result - 50.0).abs() < 1e-10, "got {result}");
    }

    // --- cross_validate ---

    #[test]
    fn test_cross_validate_no_violations_when_scores_are_consistent() {
        let score = dummy_score(zero_subs(), 50.0, 3);
        let violations = cross_validate(&score);
        // comply_errors > 0 so XV-001/XV-008 do NOT trigger. coverage=0 so XV-003 not triggered.
        // rps=0, s.file_health=0, s.muda_inv=0, coverage=0 composite=50 so XV-007/009/010 not triggered.
        assert!(violations.is_empty(), "violations: {}", violations.len());
    }

    #[test]
    fn test_cross_validate_xv001_clean_comply_but_low_rps_code_quality() {
        let mut score = dummy_score(zero_subs(), 70.0, 0);
        score.rps_categories.insert("Code Quality".into(), 30.0);
        let violations = cross_validate(&score);
        assert!(
            violations.iter().any(|v| v.id == "XV-001"),
            "expected XV-001"
        );
    }

    #[test]
    fn test_cross_validate_xv003_high_coverage_low_testing_score() {
        let mut subs = zero_subs();
        subs.coverage = Some(95.0);
        let mut score = dummy_score(subs, 70.0, 5);
        score
            .rps_categories
            .insert("Testing Excellence".into(), 40.0);
        let violations = cross_validate(&score);
        assert!(
            violations.iter().any(|v| v.id == "XV-003"),
            "expected XV-003"
        );
    }

    #[test]
    fn test_cross_validate_xv007_rps_grade_a_but_low_composite() {
        let mut subs = zero_subs();
        subs.rps = Some(92.0);
        let score = dummy_score(subs, 70.0, 5);
        let violations = cross_validate(&score);
        assert!(
            violations.iter().any(|v| v.id == "XV-007"),
            "expected XV-007"
        );
    }

    #[test]
    fn test_cross_validate_xv008_clean_comply_but_low_rps() {
        let mut subs = zero_subs();
        subs.rps = Some(40.0);
        let score = dummy_score(subs, 50.0, 0);
        let violations = cross_validate(&score);
        assert!(
            violations.iter().any(|v| v.id == "XV-008"),
            "expected XV-008"
        );
    }

    #[test]
    fn test_cross_validate_xv009_good_file_health_but_low_muda() {
        let mut subs = zero_subs();
        subs.file_health = Some(95.0);
        subs.muda_inv = Some(50.0);
        // Make comply_errors > 0 AND rps > 60 to dodge XV-001/008
        let mut score = dummy_score(subs, 80.0, 5);
        score.sub_scores.rps = Some(70.0);
        let violations = cross_validate(&score);
        assert!(
            violations.iter().any(|v| v.id == "XV-009"),
            "expected XV-009"
        );
    }

    #[test]
    fn test_cross_validate_xv010_low_coverage_but_high_composite() {
        let mut subs = zero_subs();
        subs.coverage = Some(30.0);
        let score = dummy_score(subs, 85.0, 5);
        let violations = cross_validate(&score);
        assert!(
            violations.iter().any(|v| v.id == "XV-010"),
            "expected XV-010"
        );
    }

    // --- persist_score ---

    #[test]
    fn test_persist_score_writes_json_in_pmat_metrics() {
        let tmp = TempDir::new().unwrap();
        let score = dummy_score(zero_subs(), 50.0, 0);
        persist_score(tmp.path(), &score);
        let out = tmp
            .path()
            .join(".pmat-metrics")
            .join("commit-abc1234-meta.json");
        assert!(out.exists(), "persist file missing: {}", out.display());
        let content = std::fs::read_to_string(&out).unwrap();
        // Valid JSON round-trip.
        let parsed: CompositeScore = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.sha, "abc1234");
        assert_eq!(parsed.composite, Some(50.0));
    }

    /// `--trend` and `--stack` read score files written by earlier pmat
    /// versions, where every sub-score was a bare number and `not_measured`
    /// did not exist. Those must still load, or history silently empties.
    #[test]
    fn test_pre_existing_score_files_still_deserialize() {
        let legacy = r#"{
            "sha": "deadbee",
            "timestamp": "2026-04-01T00:00:00Z",
            "composite": 52.86,
            "grade": "F",
            "sub_scores": {
                "rps": 43.5, "comply": 28.0, "coverage": 50.0, "muda_inv": 75.7,
                "evoscore": 50.0, "dbc": 50.0, "file_health": 100.0, "pv_lint": 50.0
            },
            "rps_categories": {},
            "comply_errors": 3,
            "comply_warnings": 14
        }"#;
        let parsed: CompositeScore = serde_json::from_str(legacy).expect("legacy score file");
        assert_eq!(parsed.composite, Some(52.86));
        assert_eq!(parsed.sub_scores.coverage, Some(50.0));
        assert!(parsed.not_measured.is_empty());
        assert_eq!(parsed.dimensions_total, 0, "absent in the legacy shape");
    }

    // --- get_head_sha ---

    #[test]
    fn test_get_head_sha_returns_unknown_outside_git_repo() {
        let tmp = TempDir::new().unwrap();
        // TempDir is not a git repo → git rev-parse fails → "unknown".
        let sha = get_head_sha(tmp.path());
        // It may be "unknown" OR a parent-repo SHA if TempDir happens to be
        // inside a git worktree. Accept either shape.
        assert!(
            sha == "unknown" || sha.chars().all(|c| c.is_ascii_hexdigit()),
            "got: {sha:?}"
        );
    }

    // --- check_pv_lint_gates ---

    #[test]
    fn test_check_pv_lint_gates_returns_true_when_pv_unavailable() {
        // In CI pv CLI is very likely not installed → Err branch → returns true
        // (don't penalize). Just exercise the code path.
        let tmp = TempDir::new().unwrap();
        let _ = check_pv_lint_gates(tmp.path());
        // No assertion on return — the behavior depends on host. We only need to
        // hit the function for coverage.
    }

    // --- "unmeasured" must not be a number (the 50.0 sentinel) ---

    /// Reasons carried by the four dimensions that used to answer 50.0.
    fn unmeasured_subject() -> ([Dimension; 8], Vec<f64>) {
        let dims: [Dimension; 8] = [
            Ok(40.0),                        // rps
            Ok(60.0),                        // comply
            Err("no coverage run".into()),   // coverage
            Ok(90.0),                        // muda_inv
            Err("no test history".into()),   // evoscore
            Err("no contracts".into()),      // dbc
            Ok(100.0),                       // file_health
            Err("no contracts/ dir".into()), // pv_lint
        ];
        (dims, vec![40.0, 60.0, 90.0, 100.0])
    }

    /// The headline defect: coverage, evoscore and dbc were folded into the
    /// geometric mean as the literal 50.0 whenever they had nothing to measure,
    /// while pv_lint's identical 50.0 was skipped by `if pv_lint != 50.0`.
    #[test]
    fn unmeasured_dimensions_are_excluded_from_the_composite() {
        let (dims, measured) = unmeasured_subject();
        let score = assemble_score("sha".into(), "ts".into(), dims, HashMap::new(), 0, 0);

        let expected = geometric_mean(&measured);
        let composite = score.composite.expect("four dimensions were measured");
        assert!(
            (composite - expected).abs() < 1e-9,
            "composite {composite} must be the geometric mean of the {} measured \
             dimensions ({expected}), not of a slate padded with 50.0",
            measured.len()
        );

        // And the padded mean must be a *different* number, or the assertion
        // above would hold for the buggy behaviour too.
        let padded = geometric_mean(&[40.0, 60.0, 50.0, 90.0, 50.0, 50.0, 100.0]);
        assert!(
            (composite - padded).abs() > 1.0,
            "composite {composite} is indistinguishable from the 50.0-padded \
             mean {padded}"
        );

        assert_eq!(score.dimensions_measured, 4);
        assert_eq!(score.dimensions_total, 8);
    }

    /// A dimension with nothing to measure reports no value at all, and says
    /// why — 50.0 was indistinguishable from a measurement.
    #[test]
    fn unmeasured_dimensions_are_null_and_disclosed() {
        let (dims, _) = unmeasured_subject();
        let score = assemble_score("sha".into(), "ts".into(), dims, HashMap::new(), 0, 0);

        assert_eq!(score.sub_scores.coverage, None);
        assert_eq!(score.sub_scores.evoscore, None);
        assert_eq!(score.sub_scores.dbc, None);
        assert_eq!(score.sub_scores.pv_lint, None);
        assert_eq!(score.sub_scores.rps, Some(40.0));

        let disclosed: Vec<&str> = score
            .not_measured
            .iter()
            .map(|n| n.dimension.as_str())
            .collect();
        assert_eq!(disclosed, ["coverage", "evoscore", "dbc", "pv_lint"]);
        for n in &score.not_measured {
            assert!(
                !n.reason.trim().is_empty(),
                "{} is undisclosed: no reason given",
                n.dimension
            );
        }

        // JSON consumers see null, never a plausible-looking number.
        let json = serde_json::to_value(&score).unwrap();
        assert!(json["sub_scores"]["coverage"].is_null(), "{json}");
        assert_eq!(json["dimensions_measured"], 4);
    }

    /// Nothing measured is not a score of zero (which grades F, i.e. "measured,
    /// terrible") — it is the absence of a score.
    #[test]
    fn nothing_measured_yields_no_composite_and_no_grade() {
        let dims: [Dimension; 8] = std::array::from_fn(|i| Err(format!("dim {i} unmeasurable")));
        let score = assemble_score("sha".into(), "ts".into(), dims, HashMap::new(), 0, 0);
        assert_eq!(score.composite, None);
        assert_eq!(score.grade, "n/a");
        assert_eq!(score.dimensions_measured, 0);
        assert_eq!(score.not_measured.len(), 8);
    }

    // --- each fallback path reports "not measured", with the artifact named ---

    #[test]
    fn test_coverage_without_cache_is_not_measured_and_with_cache_is() {
        let tmp = TempDir::new().unwrap();
        let reason = read_coverage_cache(tmp.path()).expect_err("no cache ⇒ no coverage");
        assert!(reason.contains("coverage.result"), "reason: {reason}");

        mkdir(&tmp.path().join(".pmat-metrics"));
        write(
            &tmp.path().join(".pmat-metrics/coverage.result"),
            r#"{"coverage_pct": 12.5}"#,
        );
        assert_eq!(read_coverage_cache(tmp.path()), Ok(12.5));
    }

    #[test]
    fn test_evoscore_without_history_is_not_measured_and_with_history_is() {
        let tmp = TempDir::new().unwrap();
        let reason = compute_evoscore(tmp.path()).expect_err("no records ⇒ no test history");
        assert!(reason.contains("test-record"), "reason: {reason}");

        mkdir(&tmp.path().join(".pmat-metrics"));
        write(
            &tmp.path().join(".pmat-metrics/commit-deadbee-tests.json"),
            r#"{"commit":"deadbee","pass":7,"total":10}"#,
        );
        assert_eq!(compute_evoscore(tmp.path()), Ok(70.0));
    }

    #[test]
    fn test_dbc_without_work_contracts_is_not_measured() {
        let tmp = TempDir::new().unwrap();
        let reason = compute_dbc(tmp.path()).expect_err("no .pmat-work ⇒ no contracts");
        assert!(reason.contains("contract.json"), "reason: {reason}");
    }

    /// Several commit records used to be picked by `read_dir` order, which the
    /// filesystem chooses.
    #[test]
    fn test_evoscore_prefers_the_recorded_commit_over_directory_order() {
        let tmp = TempDir::new().unwrap();
        mkdir(&tmp.path().join(".pmat-metrics"));
        for (sha, pass) in [("aaaaaaa", 1u32), ("bbbbbbb", 5), ("ccccccc", 9)] {
            write(
                &tmp.path()
                    .join(format!(".pmat-metrics/commit-{sha}-tests.json")),
                &format!(r#"{{"commit":"{sha}","pass":{pass},"total":10}}"#),
            );
        }
        // No git repo here, so HEAD is unknown → deterministic fallback is the
        // last file name, never "whatever read_dir returned last".
        assert_eq!(compute_evoscore(tmp.path()), Ok(90.0));
    }

    // --- the composite's blind spots reach the reader ---

    #[test]
    fn test_renderers_say_not_measured_instead_of_a_number() {
        let (dims, _) = unmeasured_subject();
        let score = assemble_score("sha".into(), "ts".into(), dims, HashMap::new(), 0, 0);

        let text = render_score(&score, &RepoScoreOutputFormat::Text).unwrap();
        assert!(text.contains("not measured"), "text: {text}");
        assert!(text.contains("no coverage run"), "text: {text}");
        assert!(text.contains("4/8"), "text must state the coverage: {text}");

        let md = render_score(&score, &RepoScoreOutputFormat::Markdown).unwrap();
        assert!(md.contains("| Coverage | not measured |"), "md: {md}");
        assert!(md.contains("Not Measured"), "md: {md}");
        assert!(md.contains("**Dimensions measured**: 4/8"), "md: {md}");
    }

    /// Dropping an unmeasured dimension can only raise the geometric mean, so a
    /// project can climb into B territory by never recording coverage. XV-011
    /// makes that visible instead of letting it read as a pass.
    #[test]
    fn test_cross_validate_xv011_high_composite_on_partial_measurement() {
        let (dims, _) = unmeasured_subject();
        let mut score = assemble_score("sha".into(), "ts".into(), dims, HashMap::new(), 1, 0);
        score.composite = Some(85.0);
        let violations = cross_validate(&score);
        assert!(
            violations.iter().any(|v| v.id == "XV-011"),
            "expected XV-011 when a composite of 85 covers only 4/8 dimensions"
        );
    }

    /// An unmeasured coverage is not a low coverage: XV-010 must not fire on
    /// the absence of a number.
    #[test]
    fn test_cross_validate_xv010_needs_a_measured_coverage() {
        let mut subs = zero_subs();
        subs.coverage = None;
        let score = dummy_score(subs, 85.0, 5);
        let violations = cross_validate(&score);
        assert!(
            !violations.iter().any(|v| v.id == "XV-010"),
            "XV-010 claims 'coverage < 50' about a coverage that was never measured"
        );
    }

    /// Source-level pin. This one compiles against the pre-fix code too, so it
    /// demonstrates the defect rather than only the new API: the production
    /// half of both files must not use 50.0 as a stand-in for "unmeasured",
    /// nor exclude a dimension by comparing a float to that sentinel.
    #[test]
    fn no_sub_score_uses_50_as_an_unmeasured_sentinel() {
        const HANDLER: &str = include_str!("score_handler.rs");
        const COMPUTE: &str = include_str!("score_handler_compute.rs");

        for (name, source) in [
            ("score_handler.rs", HANDLER),
            ("score_handler_compute.rs", COMPUTE),
        ] {
            // Only the production half — the test modules below quote these
            // very needles.
            let production = source.split("#[cfg(test)]").next().unwrap_or(source);

            assert!(
                !production.contains("!= 50.0"),
                "{name} excludes a dimension from the composite by comparing it \
                 against the magic float 50.0; unmeasured must be representable"
            );
            for line in production.lines() {
                let code = line.split("//").next().unwrap_or(line);
                assert!(
                    !code.contains("return 50.0"),
                    "{name} returns the literal 50.0 as a sub-score fallback, \
                     which is indistinguishable from a measurement: {line:?}"
                );
            }
        }
    }

    // --- render_score: each advertised format must be its own format ---

    #[test]
    fn test_render_score_yaml_and_markdown_are_not_the_text_banner() {
        let score = dummy_score(zero_subs(), 58.3, 2);

        let text = render_score(&score, &RepoScoreOutputFormat::Text).unwrap();
        let yaml = render_score(&score, &RepoScoreOutputFormat::Yaml).unwrap();
        let markdown = render_score(&score, &RepoScoreOutputFormat::Markdown).unwrap();

        assert_ne!(
            yaml, text,
            "-f yaml must not fall through to the text renderer"
        );
        assert_ne!(
            markdown, text,
            "-f markdown must not fall through to the text renderer"
        );

        // YAML must actually parse as YAML and carry the composite.
        let parsed: serde_yaml_ng::Value = serde_yaml_ng::from_str(&yaml).expect("valid YAML");
        assert!(parsed.get("composite").is_some(), "yaml: {yaml}");

        // Markdown must carry a table, as --help promises.
        assert!(
            markdown.starts_with("# PMAT Unified Score"),
            "md: {markdown}"
        );
        assert!(markdown.contains("| RPS |"), "md: {markdown}");
    }
}
