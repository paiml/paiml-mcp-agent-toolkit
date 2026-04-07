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

    eprintln!("Computing unified quality score...");

    let score = compute_composite(path).await?;
    debug_assert!(
        score.composite >= 0.0 && score.composite <= 100.0,
        "composite score out of range: {}",
        score.composite
    );

    // Persist to .pmat-metrics/
    persist_score(path, &score);

    // Format output
    let output_text = match format {
        RepoScoreOutputFormat::Json => serde_json::to_string_pretty(&score)?,
        _ => format_text(&score),
    };

    if let Some(output_path) = output {
        std::fs::write(output_path, &output_text)?;
        eprintln!("Score written to: {}", output_path.display());
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
