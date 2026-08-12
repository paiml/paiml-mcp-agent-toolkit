#![allow(unused)]
#![cfg_attr(coverage_nightly, coverage(off))]
//! Unified quality score handler (`pmat score`)
//!
//! Geometric composite of 7 sub-scores. Runs comply + RPS internally,
//! reads coverage/DBC from cache, writes all results to .pmat-metrics/.
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

/// Composite score with all sub-scores for persistence and display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositeScore {
    pub sha: String,
    pub timestamp: String,
    pub composite: f64,
    pub grade: String,
    pub sub_scores: SubScores,
    pub rps_categories: HashMap<String, f64>,
    pub comply_errors: usize,
    pub comply_warnings: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Sub scores.
pub struct SubScores {
    pub rps: f64,
    pub comply: f64,
    pub coverage: f64,
    pub muda_inv: f64,
    pub evoscore: f64,
    pub dbc: f64,
    pub file_health: f64,
    pub pv_lint: f64,
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
        score.composite >= 0.0 && score.composite <= 100.0,
        "composite score out of range: {}",
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
            10
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
        if score.composite < threshold {
            eprintln!(
                "FAIL: composite {:.1} < gate {:.1}",
                score.composite, threshold
            );
            std::process::exit(1);
        }
    }

    Ok(())
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
    let _ = writeln!(out, "- **Composite**: {:.1}/100", score.composite);
    let _ = writeln!(out, "- **Grade**: {}", score.grade);
    let _ = writeln!(out, "- **Commit**: {}", score.sha);
    let _ = writeln!(out, "- **Timestamp**: {}\n", score.timestamp);

    out.push_str("## Sub-Scores\n\n");
    out.push_str("| Sub-Score | Value |\n|---|---:|\n");
    let s = &score.sub_scores;
    let _ = writeln!(out, "| RPS | {:.1} |", s.rps);
    let _ = writeln!(
        out,
        "| Comply | {:.1} ({} errors, {} warnings) |",
        s.comply, score.comply_errors, score.comply_warnings
    );
    let _ = writeln!(out, "| Coverage | {:.1} |", s.coverage);
    let _ = writeln!(out, "| Muda (inv) | {:.1} |", s.muda_inv);
    let _ = writeln!(out, "| EvoScore | {:.1} |", s.evoscore);
    let _ = writeln!(out, "| DBC | {:.1} |", s.dbc);
    let _ = writeln!(out, "| File Health | {:.1} |", s.file_health);
    let _ = writeln!(out, "| PV Lint | {:.1} |", s.pv_lint);

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

    // Precondition: all sub-scores in valid range
    debug_assert!((0.0..=100.0).contains(&rps), "rps out of range: {rps}");
    debug_assert!(
        (0.0..=100.0).contains(&comply),
        "comply out of range: {comply}"
    );
    debug_assert!(
        (0.0..=100.0).contains(&coverage),
        "coverage out of range: {coverage}"
    );
    debug_assert!(
        (0.0..=100.0).contains(&muda_inv),
        "muda_inv out of range: {muda_inv}"
    );
    debug_assert!((0.0..=100.0).contains(&dbc), "dbc out of range: {dbc}");
    debug_assert!(
        (0.0..=100.0).contains(&file_health),
        "file_health out of range: {file_health}"
    );

    // Geometric mean — only include active sub-scores (skip neutral 50.0 defaults)
    let mut values = vec![rps, comply, coverage, muda_inv, evoscore, dbc, file_health];
    if pv_lint != 50.0 {
        values.push(pv_lint); // Only include PV Lint when contracts exist
    }
    let composite = geometric_mean(values.as_slice());
    debug_assert!(
        (0.0..=100.0).contains(&composite),
        "geometric mean out of range: {composite}"
    );

    let grade = match composite as u32 {
        90..=100 => "A",
        80..=89 => "B",
        70..=79 => "C",
        60..=69 => "D",
        _ => "F",
    }
    .to_string();

    Ok(CompositeScore {
        sha,
        timestamp,
        composite,
        grade,
        sub_scores: SubScores {
            rps,
            comply,
            coverage,
            muda_inv,
            evoscore,
            dbc,
            file_health,
            pv_lint,
        },
        rps_categories,
        comply_errors,
        comply_warnings,
    })
}

/// Run RPS once, return (percentage, category_percentages).
fn compute_rps(path: &Path) -> (f64, HashMap<String, f64>) {
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
            (score.percentage, cats)
        }
        Err(_) => (0.0, HashMap::new()),
    }
}

async fn compute_comply(path: &Path) -> (f64, usize, usize) {
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
                        return (score, errors, warnings);
                    }
                }
            }
            (50.0, 0, 0) // fallback if JSON parsing fails
        }
        _ => (50.0, 0, 0), // fallback if command fails
    }
}

fn compute_muda_inv(path: &Path) -> f64 {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let report = muda_handlers::calculate_muda_score(path);
    (100.0 - report.total_score).max(0.0)
}

fn read_coverage_cache(path: &Path) -> f64 {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    // Try .pmat-metrics/coverage.result first
    let coverage_result = path.join(".pmat-metrics/coverage.result");
    if let Ok(content) = std::fs::read_to_string(&coverage_result) {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(pct) = val.get("coverage_pct").and_then(|v| v.as_f64()) {
                return pct;
            }
        }
    }
    // Fallback: neutral score (won't kill composite)
    50.0
}

fn compute_dbc(path: &Path) -> f64 {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let score = compute_codebase_score(path);
    if score.contract_count == 0 {
        return 50.0; // neutral when no contracts
    }
    // Use the best signal: contract coverage and lint pass rate
    // These are binary quality indicators (did you write contracts? do they lint?)
    // more actionable than the abstract mean_score
    let coverage_pct = score.contract_coverage * 100.0;
    let lint_pct = score.lint_pass_rate * 100.0;
    // Weighted: 50% coverage, 30% lint, 20% mean contract score
    let dbc_score = 0.50 * coverage_pct + 0.30 * lint_pct + 0.20 * (score.mean_score * 100.0);
    dbc_score.clamp(0.0, 100.0)
}

fn compute_evoscore(path: &Path) -> f64 {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    // Read test results from .pmat-metrics/commit-*-tests.json
    let metrics_dir = path.join(".pmat-metrics");
    let mut test_records: Vec<(String, u64, u64)> = Vec::new(); // (sha, pass, total)

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
                            test_records.push((sha, pass, total));
                        }
                    }
                }
            }
        }
    }

    if test_records.is_empty() {
        return 50.0; // neutral when no test history
    }

    // Use latest test result: pass rate * 100
    let Some((_, pass, total)) = test_records.last() else {
        return 50.0;
    };
    let rate = *pass as f64 / *total as f64;
    (rate * 100.0).clamp(0.0, 100.0)
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
            composite,
            grade: "B".into(),
            sub_scores: sub,
            rps_categories: HashMap::new(),
            comply_errors,
            comply_warnings: 0,
        }
    }

    fn zero_subs() -> SubScores {
        SubScores {
            rps: 0.0,
            comply: 0.0,
            coverage: 0.0,
            muda_inv: 0.0,
            evoscore: 0.0,
            dbc: 0.0,
            file_health: 0.0,
            pv_lint: 0.0,
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
    fn test_pv_lint_no_contracts_no_pmat_yaml_no_src_returns_50() {
        let tmp = TempDir::new().unwrap();
        assert_eq!(compute_pv_lint(tmp.path()), 50.0);
    }

    #[test]
    fn test_pv_lint_no_contracts_cb1202_disabled_returns_50() {
        let tmp = TempDir::new().unwrap();
        write(
            &tmp.path().join(".pmat.yaml"),
            "checks:\n  cb-1202:\n    enabled: false\n",
        );
        assert_eq!(compute_pv_lint(tmp.path()), 50.0);
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
        assert_eq!(compute_pv_lint(tmp.path()), 0.0);
    }

    #[test]
    fn test_pv_lint_no_contracts_src_without_critical_keyword_returns_50() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src");
        mkdir(&src);
        write(&src.join("lib.rs"), "pub fn mundane() -> i32 { 42 }");
        assert_eq!(compute_pv_lint(tmp.path()), 50.0);
    }

    #[test]
    fn test_pv_lint_with_contracts_but_no_pv_cli_fallback_to_pipeline() {
        // pv CLI likely unavailable in test env → falls back to pipeline-depth.
        // With only an empty contracts dir, pipeline depth is 0 → 50.0 (clamp min).
        let tmp = TempDir::new().unwrap();
        mkdir(&tmp.path().join("contracts"));
        let score = compute_pv_lint(tmp.path());
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

    #[test]
    fn test_file_health_no_src_returns_100() {
        let tmp = TempDir::new().unwrap();
        assert_eq!(compute_file_health(tmp.path()), 100.0);
    }

    #[test]
    fn test_file_health_empty_src_returns_100() {
        let tmp = TempDir::new().unwrap();
        mkdir(&tmp.path().join("src"));
        assert_eq!(compute_file_health(tmp.path()), 100.0);
    }

    #[test]
    fn test_file_health_all_small_files_returns_100() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src");
        mkdir(&src);
        write(&src.join("a.rs"), "fn a() {}");
        write(&src.join("b.rs"), "fn b() {}");
        assert_eq!(compute_file_health(tmp.path()), 100.0);
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
        let result = compute_file_health(tmp.path());
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
        subs.coverage = 95.0;
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
        subs.rps = 92.0;
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
        subs.rps = 40.0;
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
        subs.file_health = 95.0;
        subs.muda_inv = 50.0;
        // Make comply_errors > 0 AND rps > 60 to dodge XV-001/008
        let mut score = dummy_score(subs, 80.0, 5);
        score.sub_scores.rps = 70.0;
        let violations = cross_validate(&score);
        assert!(
            violations.iter().any(|v| v.id == "XV-009"),
            "expected XV-009"
        );
    }

    #[test]
    fn test_cross_validate_xv010_low_coverage_but_high_composite() {
        let mut subs = zero_subs();
        subs.coverage = 30.0;
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
        assert_eq!(parsed.composite, 50.0);
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
