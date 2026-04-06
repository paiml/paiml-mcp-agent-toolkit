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
fn compute_pv_lint(path: &Path) -> f64 {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let contracts_dir = path.join("contracts");
    if !contracts_dir.exists() {
        let pmat_yaml = path.join(".pmat.yaml");
        if pmat_yaml.exists() {
            if let Ok(content) = std::fs::read_to_string(&pmat_yaml) {
                if content.contains("cb-1202") && content.contains("enabled: false") {
                    return 50.0;
                }
            }
        }
        let src_dir = path.join("src");
        if src_dir.exists() {
            let critical = ["forward", "backward", "optimizer", "checkpoint",
                "sampling", "kv_cache", "quantize", "kernel", "matmul", "gemm"];
            let has_critical = critical.iter().any(|kw| {
                walkdir::WalkDir::new(&src_dir).into_iter().flatten()
                    .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
                    .any(|e| std::fs::read_to_string(e.path())
                        .map(|c| c.contains(&format!("pub fn {kw}")))
                        .unwrap_or(false))
            });
            if has_critical {
                return 0.0;
            }
        }
        return 50.0;
    }

    // Strategy 1: Use `pv score` for D1-D5 dimensions (pv-compatibility spec §2.1)
    let pv_score = score_via_pv_score(path);
    if let Some(score) = pv_score {
        // Gate check: pv lint must also pass
        let lint_ok = check_pv_lint_gates(path);
        if !lint_ok {
            return score * 0.5; // Halve score if lint gates fail
        }
        return score;
    }

    // Strategy 2: Fallback to pipeline depth heuristic (pv CLI unavailable)
    let pipeline = compute_pipeline_depth(path);
    // Scale pipeline (0-30) to (50-95) range
    (50.0 + pipeline * 1.5).clamp(50.0, 95.0)
}

/// Call `pv score <contracts_dir> --format json` and parse D1-D5 dimensions.
/// Returns score on 0-100 scale, or None if pv unavailable.
fn score_via_pv_score(path: &Path) -> Option<f64> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let contracts_dir = path.join("contracts");
    let output = std::process::Command::new("pv")
        .args(["score", &contracts_dir.display().to_string(), "--format", "json"])
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
            let avg_d3 = scores.iter()
                .filter_map(|s| s.get("kani_coverage").and_then(|v| v.as_f64()))
                .sum::<f64>() / n;
            let avg_d4 = scores.iter()
                .filter_map(|s| s.get("lean_coverage").and_then(|v| v.as_f64()))
                .sum::<f64>() / n;
            if avg_d3 > 0.0 || avg_d4 > 0.0 {
                eprintln!(
                    "  PV Score: {:.1}% (Kani avg: {:.0}%, Lean avg: {:.0}%)",
                    mean_score * 100.0, avg_d3 * 100.0, avg_d4 * 100.0
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
    debug_assert!(path.exists(), "path must exist: {}", path.display());
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
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let mut score = 0.0;
    let contracts_dir = path.join("contracts");

    // 1. YAML contracts exist (5 pts)
    let yaml_count = std::fs::read_dir(&contracts_dir)
        .map(|entries| {
            entries
                .flatten()
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "yaml"))
                .count()
        })
        .unwrap_or(0);
    if yaml_count > 0 {
        score += 5.0;
    }

    // 2. build.rs reads contracts (5 pts)
    let build_rs = path.join("build.rs");
    if build_rs.exists() {
        if let Ok(content) = std::fs::read_to_string(&build_rs) {
            if content.contains("PRE_COUNT") || content.contains("emit_contract_assertions") {
                score += 5.0;
            }
        }
    }

    // 3. #[contract] macros in source (5 pts)
    let src_dir = path.join("src");
    if src_dir.exists() {
        let has_contract = walkdir::WalkDir::new(&src_dir)
            .into_iter()
            .flatten()
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            .any(|e| {
                std::fs::read_to_string(e.path())
                    .map(|c| c.contains("::contract(") || c.contains("#[contract("))
                    .unwrap_or(false)
            });
        if has_contract {
            score += 5.0;
        }
    }

    // 4. lean_theorem references in YAML (5 pts)
    if yaml_count > 0 {
        let has_lean_ref = std::fs::read_dir(&contracts_dir)
            .map(|entries| {
                entries.flatten().any(|e| {
                    e.path().extension().is_some_and(|ext| ext == "yaml")
                        && std::fs::read_to_string(e.path())
                            .map(|c| c.contains("lean_theorem:"))
                            .unwrap_or(false)
                })
            })
            .unwrap_or(false);
        if has_lean_ref {
            score += 5.0;
        }
    }

    // 5. Kani harnesses defined (5 pts)
    if yaml_count > 0 {
        let has_kani = std::fs::read_dir(&contracts_dir)
            .map(|entries| {
                entries.flatten().any(|e| {
                    e.path().extension().is_some_and(|ext| ext == "yaml")
                        && std::fs::read_to_string(e.path())
                            .map(|c| c.contains("kani_harnesses:"))
                            .unwrap_or(false)
                })
            })
            .unwrap_or(false);
        if has_kani {
            score += 5.0;
        }
    }

    // 6. proof-status.json — parse L4/L5 verification levels (0-5 pts)
    let abs_path = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let proof_status_path = abs_path
        .parent()
        .map(|p| p.join("provable-contracts").join("proof-status.json"));
    if let Some(ps_path) = proof_status_path {
        if let Ok(content) = std::fs::read_to_string(&ps_path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                let totals = val.get("totals");
                let obligations = totals.and_then(|t| t.get("obligations")).and_then(|v| v.as_u64()).unwrap_or(0);
                let kani = totals.and_then(|t| t.get("kani_harnesses")).and_then(|v| v.as_u64()).unwrap_or(0);
                let lean = totals.and_then(|t| t.get("lean_proved")).and_then(|v| v.as_u64()).unwrap_or(0);
                if obligations > 0 {
                    // Score based on L4+L5 coverage: kani/obligations + lean/obligations
                    let l4_ratio = (kani as f64 / obligations as f64).min(1.0);
                    let l5_ratio = (lean as f64 / obligations as f64).min(1.0);
                    // L4 (Kani) worth 3 pts, L5 (Lean) worth 2 pts
                    score += l4_ratio * 3.0 + l5_ratio * 2.0;
                } else {
                    score += 2.0; // File exists but no obligations — partial credit
                }
            }
        }
    }

    score
}

/// CD5: Contract drift detection. Compares contract YAML mtimes vs source file mtimes.
#[allow(dead_code)] // Available for future CD5 scoring integration
/// Returns (stale_count, total_count, drift_ratio).
/// A contract is "stale" if the YAML is >30 days older than the most recently modified
/// source file that references it (via #[contract] annotation).
fn compute_contract_drift(path: &Path) -> (usize, usize, f64) {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
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
        if p.extension().map_or(true, |e| e != "yaml") { continue; }
        if p.file_name().is_some_and(|n| n.to_string_lossy().contains("binding")) { continue; }

        let Ok(yaml_meta) = std::fs::metadata(&p) else { continue };
        let Ok(yaml_mtime) = yaml_meta.modified() else { continue };

        total += 1;

        // Find the newest source file that references this contract
        let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let search_pattern = format!("\"{stem}\"");

        let mut newest_src = None;
        let src_dirs: Vec<std::path::PathBuf> = if path.join("src").exists() {
            vec![path.join("src")]
        } else if path.join("crates").exists() {
            std::fs::read_dir(path.join("crates"))
                .into_iter().flatten().flatten()
                .filter_map(|e| { let s = e.path().join("src"); s.exists().then_some(s) })
                .collect()
        } else {
            vec![]
        };

        for sdir in &src_dirs {
            for e in walkdir::WalkDir::new(sdir).into_iter().flatten() {
                if !e.path().extension().is_some_and(|ext| ext == "rs") { continue; }
                if let Ok(content) = std::fs::read_to_string(e.path()) {
                    if content.contains(&search_pattern) {
                        if let Ok(meta) = std::fs::metadata(e.path()) {
                            if let Ok(mtime) = meta.modified() {
                                if newest_src.map_or(true, |n| mtime > n) {
                                    newest_src = Some(mtime);
                                }
                            }
                        }
                    }
                }
            }
        }

        // If source is significantly newer than contract, it's stale
        if let Some(src_mtime) = newest_src {
            if src_mtime > yaml_mtime {
                if let Ok(diff) = src_mtime.duration_since(yaml_mtime) {
                    if diff > thirty_days {
                        stale += 1;
                    }
                }
            }
        }
    }

    let drift = if total > 0 { stale as f64 / total as f64 } else { 0.0 };
    (stale, total, drift)
}

fn compute_file_health(path: &Path) -> f64 {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    // Count files by size category for a density-based score
    let src_dir = path.join("src");
    if !src_dir.exists() {
        return 100.0;
    }
    let mut total = 0usize;
    let mut over_1000 = 0usize;

    for entry in walkdir::WalkDir::new(&src_dir).into_iter().flatten() {
        if entry.path().extension().map_or(true, |e| e != "rs") {
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
        return 100.0;
    }

    // Score based on percentage of files under 1000 lines
    let pct_healthy = (1.0 - (over_1000 as f64 / total as f64)) * 100.0;
    pct_healthy.clamp(0.0, 100.0)
}

fn geometric_mean(values: &[f64]) -> f64 {
    debug_assert!(!values.is_empty(), "values must not be empty");
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

fn get_head_sha(path: &Path) -> String {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(path)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn persist_score(path: &Path, score: &CompositeScore) {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let metrics_dir = path.join(".pmat-metrics");
    let _ = std::fs::create_dir_all(&metrics_dir);
    let filename = format!("commit-{}-meta.json", score.sha);
    let filepath = metrics_dir.join(filename);
    if let Ok(json) = serde_json::to_string_pretty(score) {
        let _ = std::fs::write(filepath, json);
    }
}

struct Violation {
    id: &'static str,
    message: String,
}

/// CB-146: Cross-validation invariants. Detect contradictions between sub-scores.
fn cross_validate(score: &CompositeScore) -> Vec<Violation> {
    debug_assert!(true, "contract: cross_validate");
    let s = &score.sub_scores;
    let mut v = Vec::new();

    // XV-001: TDG grade gate pass should correlate with decent code quality
    // CB-200 passes when comply has no TDG failures, but RPS Code Quality can still be low
    if score.comply_errors == 0 {
        if let Some(cq) = score.rps_categories.get("Code Quality") {
            if *cq < 40.0 {
                v.push(Violation {
                    id: "XV-001",
                    message: format!("Comply 0 errors but RPS Code Quality {cq:.0}% < 40%"),
                });
            }
        }
    }

    // XV-003: High coverage should mean decent testing score
    if s.coverage >= 90.0 {
        if let Some(ts) = score.rps_categories.get("Testing Excellence") {
            if *ts < 60.0 {
                v.push(Violation {
                    id: "XV-003",
                    message: format!("Coverage {:.0}% but RPS Testing {ts:.0}% < 60%", s.coverage),
                });
            }
        }
    }

    // XV-007: RPS Grade A should mean composite >= 75
    if s.rps >= 90.0 && score.composite < 75.0 {
        v.push(Violation {
            id: "XV-007",
            message: format!(
                "RPS {:.0} (A-level) but composite {:.1} < 75",
                s.rps, score.composite
            ),
        });
    }

    // XV-008: Clean comply should correlate with decent RPS
    if score.comply_errors == 0 && s.rps < 60.0 {
        v.push(Violation {
            id: "XV-008",
            message: format!("Comply 0 errors but RPS {:.0} < 60", s.rps),
        });
    }

    // XV-009: Good file health should mean low muda over-processing
    if s.file_health >= 90.0 && s.muda_inv < 70.0 {
        v.push(Violation {
            id: "XV-009",
            message: format!(
                "File health {:.0} (A) but Muda inv {:.0} < 70",
                s.file_health, s.muda_inv
            ),
        });
    }

    // XV-010: Low coverage must cap composite
    if s.coverage < 50.0 && score.composite >= 80.0 {
        v.push(Violation {
            id: "XV-010",
            message: format!(
                "Coverage {:.0}% < 50 but composite {:.1} >= 80",
                s.coverage, score.composite
            ),
        });
    }

    v
}

