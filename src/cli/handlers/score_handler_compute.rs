/// Compute PV Lint sub-score using pv score D1-D5 dimensions (pv-compatibility spec §2.1).
///
/// Aligns pmat scoring with the provable-contracts scoring model:
///   D1: Specification Depth (20%)
///   D2: Falsification Coverage (25%)
///   D3: Kani Proof Coverage (25%)
///   D4: Lean Proof Coverage (10%)
///   D5: Binding Coverage (20%)
///
/// Falls back to pipeline-depth heuristic if pv CLI is unavailable.
///
/// A project with no `contracts/` directory has no provable contracts to score,
/// so this returns `Err` (not measured) rather than a mid-range number. That is
/// the same conclusion `compute_composite` used to reach with `if pv_lint !=
/// 50.0`, expressed in the type instead of by comparing a float to a sentinel.
/// `compute_pv_lint`'s answer when `contracts/` does not exist: honor an
/// explicit opt-out, then look for safety-critical kernels shipped with no
/// contracts at all (scored 0.0), else "not measured". Extracted from the
/// `if !contracts_dir.exists()` branch of `compute_pv_lint`; every path
/// through it still returns the same value it did inline.
fn score_pv_lint_when_no_contracts(path: &Path, contracts_dir: &Path) -> Dimension {
    let pmat_yaml = path.join(".pmat.yaml");
    if pmat_yaml.exists() {
        if let Ok(content) = std::fs::read_to_string(&pmat_yaml) {
            if content.contains("cb-1202") && content.contains("enabled: false") {
                return Err("PV lint is switched off: .pmat.yaml disables check cb-1202".to_string());
            }
        }
    }
    let src_dir = path.join("src");
    if src_dir.exists() && has_critical_kernel_without_contracts(&src_dir) {
        // Measured, and deliberately harsh: safety-critical kernels
        // with no contracts at all score zero.
        return Ok(0.0);
    }
    Err(format!(
        "no provable contracts: {} does not exist",
        contracts_dir.display()
    ))
}

/// Whether `src_dir` contains a `pub fn` named after one of the safety-critical
/// kernel keywords `compute_pv_lint` scores harshly when contracts are absent.
fn has_critical_kernel_without_contracts(src_dir: &Path) -> bool {
    let critical = [
        "forward",
        "backward",
        "optimizer",
        "checkpoint",
        "sampling",
        "kv_cache",
        "quantize",
        "kernel",
        "matmul",
        "gemm",
    ];
    critical.iter().any(|kw| {
        walkdir::WalkDir::new(src_dir)
            .into_iter()
            .flatten()
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            .any(|e| {
                std::fs::read_to_string(e.path())
                    .map(|c| c.contains(&format!("pub fn {kw}")))
                    .unwrap_or(false)
            })
    })
}

fn compute_pv_lint(path: &Path) -> Dimension {
    let contracts_dir = path.join("contracts");
    if !contracts_dir.exists() {
        return score_pv_lint_when_no_contracts(path, &contracts_dir);
    }

    // Strategy 1: Use `pv score` for D1-D5 dimensions (pv-compatibility spec §2.1)
    let pv_score = score_via_pv_score(path);
    if let Some(score) = pv_score {
        // Gate check: pv lint must also pass
        let lint_ok = check_pv_lint_gates(path);
        if !lint_ok {
            return Ok(score * 0.5); // Halve score if lint gates fail
        }
        return Ok(score);
    }

    // Strategy 2: Fallback to pipeline depth heuristic (pv CLI unavailable)
    let pipeline = compute_pipeline_depth(path);
    // Scale pipeline (0-30) to (50-95) range
    Ok((50.0 + pipeline * 1.5).clamp(50.0, 95.0))
}

/// Call `pv score <contracts_dir> --format json` and parse D1-D5 dimensions.
/// Returns score on 0-100 scale, or None if pv unavailable.
fn score_via_pv_score(path: &Path) -> Option<f64> {
    let contracts_dir = path.join("contracts");
    let output = std::process::Command::new("pv")
        .args([
            "score",
            &contracts_dir.display().to_string(),
            "--format",
            "json",
        ])
        .current_dir(path)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;

    if !output.status.success() && output.status.code() != Some(1) {
        return None;
    }

    let content = String::from_utf8(output.stdout).ok()?;
    let val: serde_json::Value = serde_json::from_str(&content).ok()?;

    // Parse mean_score (0.0-1.0) from pv score output
    let contract_count = val.get("contracts").and_then(|c| c.as_u64()).unwrap_or(0);
    if contract_count == 0 {
        return None; // No contracts — fall through to heuristic
    }
    let mean_score = val.get("mean_score").and_then(|s| s.as_f64())?;

    // Also extract per-dimension averages for diagnostics
    if let Some(scores) = val.get("scores").and_then(|s| s.as_array()) {
        if !scores.is_empty() {
            let n = scores.len() as f64;
            let avg_d3 = scores
                .iter()
                .filter_map(|s| s.get("kani_coverage").and_then(|v| v.as_f64()))
                .sum::<f64>()
                / n;
            let avg_d4 = scores
                .iter()
                .filter_map(|s| s.get("lean_coverage").and_then(|v| v.as_f64()))
                .sum::<f64>()
                / n;
            if avg_d3 > 0.0 || avg_d4 > 0.0 {
                crate::status_eprintln!(
                    "  PV Score: {:.1}% (Kani avg: {:.0}%, Lean avg: {:.0}%)",
                    mean_score * 100.0,
                    avg_d3 * 100.0,
                    avg_d4 * 100.0
                );
            }
        }
    }

    // Blend pv score (contract quality) with pipeline depth (integration quality)
    // pv score measures D1-D5 of the contracts themselves
    // pipeline depth measures how well the codebase integrates them
    let pipeline = compute_pipeline_depth(path);
    let pipeline_norm = pipeline / 30.0; // 0.0-1.0

    // 40% contract quality (pv score D1-D5) + 60% integration depth (pipeline)
    // pmat measures the integration quality; pv score measures the contract quality.
    // Projects with full pipeline (build.rs + macros + Lean refs) deserve high scores
    // even if the contracts themselves still have gaps (lean_coverage, binding_coverage).
    let blended = mean_score * 0.4 + pipeline_norm * 0.6;
    Some((blended * 100.0).clamp(0.0, 100.0))
}

/// Quick check: does `pv lint` pass all gates?
fn check_pv_lint_gates(path: &Path) -> bool {
    let output = std::process::Command::new("pv")
        .args(["lint", "--format", "json"])
        .current_dir(path)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output();

    match output {
        Ok(o) => {
            if let Ok(content) = String::from_utf8(o.stdout) {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                    return val.get("passed").and_then(|p| p.as_bool()).unwrap_or(false);
                }
            }
            o.status.success()
        }
        Err(_) => true, // pv unavailable — don't penalize
    }
}

/// Count of `.yaml` files directly under `contracts_dir` (pipeline check #1).
fn pipeline_yaml_count(contracts_dir: &Path) -> usize {
    std::fs::read_dir(contracts_dir)
        .map(|entries| {
            entries
                .flatten()
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "yaml"))
                .count()
        })
        .unwrap_or(0)
}

/// Pipeline check #2: does `build.rs` read the contracts at build time?
fn pipeline_build_rs_reads_contracts(path: &Path) -> bool {
    let build_rs = path.join("build.rs");
    build_rs.exists()
        && std::fs::read_to_string(&build_rs)
            .map(|content| {
                content.contains("PRE_COUNT") || content.contains("emit_contract_assertions")
            })
            .unwrap_or(false)
}

/// Pipeline check #3: any `.rs` file under `src_dir` carrying a `#[contract]` macro.
fn pipeline_has_contract_macro(src_dir: &Path) -> bool {
    walkdir::WalkDir::new(src_dir)
        .into_iter()
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
        .any(|e| {
            std::fs::read_to_string(e.path())
                .map(|c| c.contains("::contract(") || c.contains("#[contract("))
                .unwrap_or(false)
        })
}

/// Pipeline checks #4/#5: does any `.yaml` under `contracts_dir` contain `needle`?
fn pipeline_any_yaml_contains(contracts_dir: &Path, needle: &str) -> bool {
    std::fs::read_dir(contracts_dir)
        .map(|entries| {
            entries.flatten().any(|e| {
                e.path().extension().is_some_and(|ext| ext == "yaml")
                    && std::fs::read_to_string(e.path())
                        .map(|c| c.contains(needle))
                        .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

/// Pipeline check #6: 0-5 pts from `provable-contracts/proof-status.json`'s
/// L4 (Kani) / L5 (Lean) verification-level totals.
fn pipeline_proof_status_score(path: &Path) -> f64 {
    let abs_path = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let Some(ps_path) = abs_path
        .parent()
        .map(|p| p.join("provable-contracts").join("proof-status.json"))
    else {
        return 0.0;
    };
    let Ok(content) = std::fs::read_to_string(&ps_path) else {
        return 0.0;
    };
    let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) else {
        return 0.0;
    };
    let totals = val.get("totals");
    let obligations = totals
        .and_then(|t| t.get("obligations"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let kani = totals
        .and_then(|t| t.get("kani_harnesses"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let lean = totals
        .and_then(|t| t.get("lean_proved"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    if obligations > 0 {
        // Score based on L4+L5 coverage: kani/obligations + lean/obligations
        let l4_ratio = (kani as f64 / obligations as f64).min(1.0);
        let l5_ratio = (lean as f64 / obligations as f64).min(1.0);
        // L4 (Kani) worth 3 pts, L5 (Lean) worth 2 pts
        l4_ratio * 3.0 + l5_ratio * 2.0
    } else {
        2.0 // File exists but no obligations — partial credit
    }
}

/// Score the contract pipeline depth (0-30 points).
///
/// Checks how deeply integrated the provable-contracts pipeline is:
/// - YAML contracts exist (5 pts)
/// - build.rs reads YAML and emits PRE/POST env vars (5 pts)
/// - #[contract] macros on production functions (5 pts)
/// - lean_theorem references in YAML (5 pts)
/// - Kani harnesses defined (5 pts)
/// - proof-status.json available from provable-contracts (5 pts)
fn compute_pipeline_depth(path: &Path) -> f64 {
    let mut score = 0.0;
    let contracts_dir = path.join("contracts");

    // 1. YAML contracts exist (5 pts)
    let yaml_count = pipeline_yaml_count(&contracts_dir);
    if yaml_count > 0 {
        score += 5.0;
    }

    // 2. build.rs reads contracts (5 pts)
    if pipeline_build_rs_reads_contracts(path) {
        score += 5.0;
    }

    // 3. #[contract] macros in source (5 pts)
    let src_dir = path.join("src");
    if src_dir.exists() && pipeline_has_contract_macro(&src_dir) {
        score += 5.0;
    }

    // 4. lean_theorem references in YAML (5 pts)
    if yaml_count > 0 && pipeline_any_yaml_contains(&contracts_dir, "lean_theorem:") {
        score += 5.0;
    }

    // 5. Kani harnesses defined (5 pts)
    if yaml_count > 0 && pipeline_any_yaml_contains(&contracts_dir, "kani_harnesses:") {
        score += 5.0;
    }

    // 6. proof-status.json — parse L4/L5 verification levels (0-5 pts)
    score += pipeline_proof_status_score(path);

    score
}

/// CD5: Contract drift detection. Compares contract YAML mtimes vs source file mtimes.
// Available for future CD5 scoring integration
/// Returns (stale_count, total_count, drift_ratio).
/// A contract is "stale" if the YAML is >30 days older than the most recently modified
/// source file that references it (via #[contract] annotation).
/// Every `src/` directory `compute_contract_drift` should search for a
/// referencing source file: the crate root's `src/`, or (workspace layout)
/// every `crates/*/src/` that exists.
fn drift_src_dirs(path: &Path) -> Vec<std::path::PathBuf> {
    if path.join("src").exists() {
        vec![path.join("src")]
    } else if path.join("crates").exists() {
        std::fs::read_dir(path.join("crates"))
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|e| {
                let s = e.path().join("src");
                s.exists().then_some(s)
            })
            .collect()
    } else {
        vec![]
    }
}

/// The mtime of the newest `.rs` file under any of `src_dirs` whose content
/// contains `search_pattern`, or `None` if no such file exists.
fn newest_referencing_source_mtime(
    src_dirs: &[std::path::PathBuf],
    search_pattern: &str,
) -> Option<std::time::SystemTime> {
    let mut newest_src = None;
    for sdir in src_dirs {
        for e in walkdir::WalkDir::new(sdir).into_iter().flatten() {
            if e.path().extension().is_none_or(|ext| ext != "rs") {
                continue;
            }
            let Ok(content) = std::fs::read_to_string(e.path()) else {
                continue;
            };
            if !content.contains(search_pattern) {
                continue;
            }
            let Ok(meta) = std::fs::metadata(e.path()) else {
                continue;
            };
            let Ok(mtime) = meta.modified() else {
                continue;
            };
            if newest_src.is_none_or(|n| mtime > n) {
                newest_src = Some(mtime);
            }
        }
    }
    newest_src
}

/// A contract is stale if its newest referencing source file is more than
/// `threshold` newer than the contract YAML's own mtime.
fn contract_is_stale(
    yaml_mtime: std::time::SystemTime,
    newest_src: Option<std::time::SystemTime>,
    threshold: std::time::Duration,
) -> bool {
    let Some(src_mtime) = newest_src else {
        return false;
    };
    if src_mtime <= yaml_mtime {
        return false;
    }
    let Ok(diff) = src_mtime.duration_since(yaml_mtime) else {
        return false;
    };
    diff > threshold
}

fn compute_contract_drift(path: &Path) -> (usize, usize, f64) {
    let contracts_dir = path.join("contracts");
    if !contracts_dir.exists() {
        return (0, 0, 0.0);
    }

    let mut stale = 0usize;
    let mut total = 0usize;
    let thirty_days = std::time::Duration::from_secs(30 * 24 * 3600);

    let Ok(entries) = std::fs::read_dir(&contracts_dir) else {
        return (0, 0, 0.0);
    };

    for entry in entries.flatten() {
        let p = entry.path();
        if p.extension().is_none_or(|e| e != "yaml") {
            continue;
        }
        if p.file_name()
            .is_some_and(|n| n.to_string_lossy().contains("binding"))
        {
            continue;
        }

        let Ok(yaml_meta) = std::fs::metadata(&p) else {
            continue;
        };
        let Ok(yaml_mtime) = yaml_meta.modified() else {
            continue;
        };

        total += 1;

        // Find the newest source file that references this contract
        let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let search_pattern = format!("\"{stem}\"");
        let src_dirs = drift_src_dirs(path);
        let newest_src = newest_referencing_source_mtime(&src_dirs, &search_pattern);

        if contract_is_stale(yaml_mtime, newest_src, thirty_days) {
            stale += 1;
        }
    }

    let drift = if total > 0 {
        stale as f64 / total as f64
    } else {
        0.0
    };
    (stale, total, drift)
}

/// Share of `src/**/*.rs` files at or under 1000 lines.
///
/// A project with no Rust sources has no file health: it used to report a
/// perfect 100.0, which is a score for a codebase that does not exist.
fn compute_file_health(path: &Path) -> Dimension {
    // Count files by size category for a density-based score
    let src_dir = path.join("src");
    if !src_dir.exists() {
        return Err(format!(
            "no Rust sources to size: {} does not exist",
            src_dir.display()
        ));
    }
    let mut total = 0usize;
    let mut over_1000 = 0usize;

    for entry in walkdir::WalkDir::new(&src_dir).into_iter().flatten() {
        if entry.path().extension().is_none_or(|e| e != "rs") {
            continue;
        }
        total += 1;
        if let Ok(content) = std::fs::read_to_string(entry.path()) {
            if content.lines().count() > 1000 {
                over_1000 += 1;
            }
        }
    }

    if total == 0 {
        return Err(format!(
            "no Rust sources to size: {} holds no .rs files",
            src_dir.display()
        ));
    }

    // Score based on percentage of files under 1000 lines
    let pct_healthy = (1.0 - (over_1000 as f64 / total as f64)) * 100.0;
    Ok(pct_healthy.clamp(0.0, 100.0))
}

fn geometric_mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    // Use log-space to avoid overflow
    let n = values.len() as f64;
    let log_sum: f64 = values
        .iter()
        .map(|v| if *v > 0.0 { v.ln() } else { f64::NEG_INFINITY })
        .sum();
    if log_sum == f64::NEG_INFINITY {
        return 0.0;
    }
    (log_sum / n).exp()
}

/// Kani bounded model checking proofs for scoring functions.
/// Run: `cargo kani --harness verify_geometric_mean_bounded`
#[cfg(kani)]
mod kani_proofs {
    use super::geometric_mean;

    /// Prove: geometric_mean always returns a value in [0, 100] for inputs in [0, 100].
    /// Exhaustively checks all f64 values within the bound (Kani explores symbolically).
    #[kani::proof]
    #[kani::unwind(8)]
    fn verify_geometric_mean_bounded() {
        let n: usize = kani::any();
        kani::assume(n > 0 && n <= 7);
        let mut values = Vec::with_capacity(n);
        for _ in 0..n {
            let v: f64 = kani::any();
            kani::assume(v >= 0.0 && v <= 100.0 && v.is_finite());
            values.push(v);
        }
        let result = geometric_mean(&values);
        assert!(result >= 0.0, "geometric_mean must be non-negative");
        assert!(result <= 100.0, "geometric_mean must not exceed max input");
        assert!(
            result.is_finite() || result == 0.0,
            "geometric_mean must be finite or zero"
        );
    }

    /// Prove: geometric_mean of empty slice returns 0.
    #[kani::proof]
    fn verify_geometric_mean_empty() {
        let result = geometric_mean(&[]);
        assert_eq!(result, 0.0);
    }

    /// Prove: geometric_mean of single value returns that value.
    #[kani::proof]
    fn verify_geometric_mean_identity() {
        let v: f64 = kani::any();
        kani::assume(v > 0.0 && v <= 100.0 && v.is_finite());
        let result = geometric_mean(&[v]);
        // Allow small floating-point epsilon
        assert!(
            (result - v).abs() < 1e-10,
            "single-value geometric mean must equal the value"
        );
    }

    /// Prove: geometric_mean with any zero input returns 0.
    #[kani::proof]
    fn verify_geometric_mean_zero_absorbing() {
        let a: f64 = kani::any();
        kani::assume(a >= 0.0 && a <= 100.0 && a.is_finite());
        let result = geometric_mean(&[a, 0.0]);
        assert_eq!(result, 0.0, "any zero input must make geometric mean zero");
    }
}

fn get_head_sha(path: &Path) -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(path)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Write the score record under `.pmat-metrics/`.
///
/// Returns the failure rather than dropping it: every arm here used to be
/// discarded (`if let Ok(..)`, `let _ = write`), so a full disk or a read-only
/// checkout produced a score run that reported success and persisted nothing,
/// and the next run that expected the record found none with no trace of why.
fn persist_score(path: &Path, score: &CompositeScore) -> Result<std::path::PathBuf, String> {
    // #1070: `pmat score` on a clean checkout left `?? .pmat-metrics/` behind.
    // The directory carries its own ignore rule now — see `utils::pmat_cache_dir`.
    let metrics_dir = crate::utils::pmat_cache_dir::ensure_metrics_dir(path);
    let filename = format!("commit-{}-meta.json", score.sha);
    let filepath = metrics_dir.join(filename);
    let json = serde_json::to_string_pretty(score)
        .map_err(|e| format!("could not serialize score: {e}"))?;
    std::fs::write(&filepath, json)
        .map_err(|e| format!("could not save score to {}: {e}", filepath.display()))?;
    Ok(filepath)
}

struct Violation {
    id: &'static str,
    message: String,
}

/// How many invariants `cross_validate` can report, for the "n/N violated"
/// line. It used to print a hard-coded 10 while implementing seven
/// (XV-001/003/007/008/009/010), which named four checks that do not exist.
const CROSS_VALIDATION_INVARIANTS: usize = 7;

// Every invariant below reads sub-scores. An unmeasured dimension has no
// value, so it can neither satisfy nor violate an invariant — `?`/`let Some`
// is what keeps "we never measured it" from reading as a good number.

/// XV-001: TDG grade gate pass should correlate with decent code quality.
/// CB-200 passes when comply has no TDG failures, but RPS Code Quality can
/// still be low. `Some(0)`, not `0`: an UNMEASURED comply must not read as a
/// clean one. While these were `usize`, a run with no `pmat` on PATH reported
/// 0 errors and fired every invariant that means "comply is clean".
fn check_xv_001(score: &CompositeScore) -> Option<Violation> {
    if score.comply_errors != Some(0) {
        return None;
    }
    let cq = *score.rps_categories.get("Code Quality")?;
    (cq < 40.0).then(|| Violation {
        id: "XV-001",
        message: format!("Comply 0 errors but RPS Code Quality {cq:.0}% < 40%"),
    })
}

/// XV-003: High coverage should mean decent testing score.
fn check_xv_003(score: &CompositeScore) -> Option<Violation> {
    let cov = score.sub_scores.coverage.filter(|c| *c >= 90.0)?;
    let ts = *score.rps_categories.get("Testing Excellence")?;
    (ts < 60.0).then(|| Violation {
        id: "XV-003",
        message: format!("Coverage {cov:.0}% but RPS Testing {ts:.0}% < 60%"),
    })
}

/// XV-007: RPS Grade A should mean composite >= 75.
fn check_xv_007(score: &CompositeScore) -> Option<Violation> {
    let rps = score.sub_scores.rps?;
    let composite = score.composite?;
    (rps >= 90.0 && composite < 75.0).then(|| Violation {
        id: "XV-007",
        message: format!("RPS {rps:.0} (A-level) but composite {composite:.1} < 75"),
    })
}

/// XV-008: Clean comply should correlate with decent RPS.
fn check_xv_008(score: &CompositeScore) -> Option<Violation> {
    let rps = score.sub_scores.rps?;
    (score.comply_errors == Some(0) && rps < 60.0).then(|| Violation {
        id: "XV-008",
        message: format!("Comply 0 errors but RPS {rps:.0} < 60"),
    })
}

/// XV-009: Good file health should mean low muda over-processing.
fn check_xv_009(score: &CompositeScore) -> Option<Violation> {
    let file_health = score.sub_scores.file_health?;
    let muda_inv = score.sub_scores.muda_inv?;
    (file_health >= 90.0 && muda_inv < 70.0).then(|| Violation {
        id: "XV-009",
        message: format!("File health {file_health:.0} (A) but Muda inv {muda_inv:.0} < 70"),
    })
}

/// XV-010: Low coverage must cap composite.
fn check_xv_010(score: &CompositeScore) -> Option<Violation> {
    let coverage = score.sub_scores.coverage?;
    let composite = score.composite?;
    (coverage < 50.0 && composite >= 80.0).then(|| Violation {
        id: "XV-010",
        message: format!("Coverage {coverage:.0}% < 50 but composite {composite:.1} >= 80"),
    })
}

/// XV-011: a high composite must not rest on dimensions nobody measured.
/// Excluding an unmeasured dimension can only raise the geometric mean, so
/// without this a project earns a B by never recording coverage.
fn check_xv_011(score: &CompositeScore) -> Option<Violation> {
    let composite = score.composite?;
    (composite >= 80.0 && !score.not_measured.is_empty()).then(|| Violation {
        id: "XV-011",
        message: format!(
            "composite {composite:.1} >= 80 but only {}/{} dimensions were measured ({} not measured)",
            score.dimensions_measured,
            score.dimensions_total,
            score
                .not_measured
                .iter()
                .map(|n| n.dimension.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ),
    })
}

/// CB-146: Cross-validation invariants. Detect contradictions between sub-scores.
fn cross_validate(score: &CompositeScore) -> Vec<Violation> {
    [
        check_xv_001(score),
        check_xv_003(score),
        check_xv_007(score),
        check_xv_008(score),
        check_xv_009(score),
        check_xv_010(score),
        check_xv_011(score),
    ]
    .into_iter()
    .flatten()
    .collect()
}
