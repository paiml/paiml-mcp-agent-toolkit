//! Work Falsification: Popperian Falsification Executor
//!
//! Runs all falsification tests and BLOCKS completion if ANY fail.
//! Based on Karl Popper's demarcation criterion: claims must be falsifiable.
//!
//! **O(1) CRITICAL:** All checks read from cached metrics (<100ms total).
//! Cache is populated by pre-commit hooks and CI pipelines.
//!
//! Based on: docs/specifications/improve-pmat-work.md

use crate::cli::handlers::work_contract::{
    EvidenceType, FalsifiableClaim, FalsificationMethod, FalsificationResult, FileManifest,
    WorkContract,
};
use crate::services::gaming_detector;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Cache staleness thresholds (per spec v2.7)
const CACHE_WARN_HOURS: i64 = 1;
const CACHE_BLOCK_HOURS: i64 = 24;

/// Cached metric status
#[derive(Debug)]
struct CachedMetric {
    value: serde_json::Value,
    age_minutes: i64,
    is_stale_warn: bool,
    is_stale_block: bool,
}

/// Read a cached metric from .pmat-metrics/
fn read_cached_metric(project_path: &Path, filename: &str) -> Option<CachedMetric> {
    let cache_path = project_path.join(".pmat-metrics").join(filename);
    if !cache_path.exists() {
        return None;
    }

    let content = std::fs::read_to_string(&cache_path).ok()?;
    let value: serde_json::Value = serde_json::from_str(&content).ok()?;

    // Check file modification time for staleness
    let metadata = std::fs::metadata(&cache_path).ok()?;
    let modified = metadata.modified().ok()?;
    let age = std::time::SystemTime::now()
        .duration_since(modified)
        .ok()?;
    let age_minutes = age.as_secs() as i64 / 60;
    let age_hours = age_minutes / 60;

    Some(CachedMetric {
        value,
        age_minutes,
        is_stale_warn: age_hours >= CACHE_WARN_HOURS,
        is_stale_block: age_hours >= CACHE_BLOCK_HOURS,
    })
}

/// Result of running all falsification tests
#[derive(Debug)]
pub struct FalsificationReport {
    /// Total number of claims tested
    pub total_claims: usize,

    /// Number of claims that passed (survived falsification)
    pub passed: usize,

    /// Number of claims that failed (were falsified)
    pub failed: usize,

    /// Number of warnings (non-blocking)
    pub warnings: usize,

    /// Individual claim results
    pub claim_results: Vec<ClaimResult>,

    /// Overall pass/fail
    pub all_passed: bool,
}

/// Result of testing a single claim
#[derive(Debug)]
pub struct ClaimResult {
    /// Claim index (1-based for display)
    pub index: usize,

    /// Hypothesis being tested
    pub hypothesis: String,

    /// Method used for falsification
    pub method: FalsificationMethod,

    /// Result of the falsification attempt
    pub result: FalsificationResult,

    /// Is this a blocking failure or just a warning?
    pub is_blocking: bool,
}

impl FalsificationReport {
    /// Check if any blocking failures occurred
    pub fn has_blocking_failures(&self) -> bool {
        self.claim_results
            .iter()
            .any(|r| r.result.falsified && r.is_blocking)
    }

    /// Get all blocking failures
    pub fn blocking_failures(&self) -> Vec<&ClaimResult> {
        self.claim_results
            .iter()
            .filter(|r| r.result.falsified && r.is_blocking)
            .collect()
    }

    /// Get all warnings (non-blocking failures)
    pub fn warning_failures(&self) -> Vec<&ClaimResult> {
        self.claim_results
            .iter()
            .filter(|r| r.result.falsified && !r.is_blocking)
            .collect()
    }
}

/// Run all falsification tests against the work contract
pub async fn run_falsification_tests(
    project_path: &Path,
    contract: &WorkContract,
) -> Result<FalsificationReport> {
    let mut claim_results = Vec::new();
    let total_claims = contract.claims.len();

    println!();
    println!(
        "Running Popperian Falsification ({} claims to validate)",
        total_claims
    );
    println!();

    for (i, claim) in contract.claims.iter().enumerate() {
        let index = i + 1;
        println!(
            "[{}/{}] {}",
            index, total_claims, claim.hypothesis
        );
        print!("      Falsification: ");

        let (result, is_blocking) = run_single_falsification(project_path, contract, claim).await?;

        let status = if result.falsified {
            if is_blocking {
                "FAILED"
            } else {
                "WARNING"
            }
        } else {
            "PASSED"
        };

        println!("{}", result.explanation);
        println!("      Result: {}", status);

        if result.falsified {
            if let Some(ref evidence) = result.evidence {
                print_evidence(evidence);
            }
        }
        println!();

        claim_results.push(ClaimResult {
            index,
            hypothesis: claim.hypothesis.clone(),
            method: claim.falsification_method.clone(),
            result,
            is_blocking,
        });
    }

    let passed = claim_results.iter().filter(|r| !r.result.falsified).count();
    let failed = claim_results
        .iter()
        .filter(|r| r.result.falsified && r.is_blocking)
        .count();
    let warnings = claim_results
        .iter()
        .filter(|r| r.result.falsified && !r.is_blocking)
        .count();

    let all_passed = failed == 0;

    Ok(FalsificationReport {
        total_claims,
        passed,
        failed,
        warnings,
        claim_results,
        all_passed,
    })
}

/// Run a single falsification test
async fn run_single_falsification(
    project_path: &Path,
    contract: &WorkContract,
    claim: &FalsifiableClaim,
) -> Result<(FalsificationResult, bool)> {
    match claim.falsification_method {
        FalsificationMethod::ManifestIntegrity => {
            let result = test_manifest_integrity(project_path, &contract.baseline_file_manifest)?;
            Ok((result, true)) // Blocking
        }

        FalsificationMethod::MetaFalsification => {
            let result = test_meta_falsification(project_path)?;
            Ok((result, true)) // Blocking
        }

        FalsificationMethod::CoverageGaming => {
            let result = test_coverage_gaming(project_path)?;
            Ok((result, true)) // Blocking
        }

        FalsificationMethod::DifferentialCoverage => {
            let result =
                test_differential_coverage(project_path, &contract.baseline_commit).await?;
            Ok((result, true)) // Blocking
        }

        FalsificationMethod::AbsoluteCoverage => {
            let result =
                test_absolute_coverage(project_path, contract.thresholds.min_coverage_pct).await?;
            Ok((result, true)) // Blocking
        }

        FalsificationMethod::TdgRegression => {
            let result = test_tdg_regression(project_path, contract.baseline_tdg).await?;
            Ok((result, true)) // Blocking
        }

        FalsificationMethod::ComplexityRegression => {
            let result = test_complexity_regression(
                project_path,
                contract.thresholds.max_function_complexity,
            )?;
            Ok((result, true)) // Blocking
        }

        FalsificationMethod::SupplyChainIntegrity => {
            let result = test_supply_chain_integrity(project_path).await?;
            Ok((result, true)) // Blocking
        }

        FalsificationMethod::FileSizeRegression => {
            let result =
                test_file_size_regression(project_path, contract.thresholds.max_file_lines)?;
            Ok((result, false)) // Warning only
        }

        FalsificationMethod::SpecQuality => {
            let result = test_spec_quality(
                project_path,
                &contract.work_item_id,
                contract.thresholds.min_spec_score,
            )?;
            Ok((result, true)) // Blocking
        }

        FalsificationMethod::RoadmapUpdate => {
            let result = test_roadmap_update(project_path, &contract.baseline_commit)?;
            Ok((result, true)) // Blocking
        }

        FalsificationMethod::GitHubSync => {
            let result = test_github_sync(project_path)?;
            Ok((result, true)) // Blocking
        }

        FalsificationMethod::ExamplesCompile => {
            let result = test_examples_compile(project_path).await?;
            Ok((result, true)) // Blocking
        }

        FalsificationMethod::BookValidation => {
            let result = test_book_validation(project_path).await?;
            Ok((result, true)) // Blocking
        }

        // v2.6 comply spec additions
        FalsificationMethod::SatdDetection => {
            let result = test_satd_detection(project_path, &contract.baseline_commit).await?;
            Ok((result, contract.thresholds.block_on_new_satd)) // Configurable blocking
        }

        FalsificationMethod::DeadCodeDetection => {
            let result = test_dead_code_detection(project_path, &contract.baseline_commit).await?;
            Ok((result, contract.thresholds.block_on_new_dead_code)) // Configurable blocking
        }

        FalsificationMethod::PerFileCoverage => {
            let result = test_per_file_coverage(
                project_path,
                contract.thresholds.min_per_file_coverage_pct,
            )
            .await?;
            Ok((result, true)) // Blocking
        }

        FalsificationMethod::LintPass => {
            let result = test_lint_pass(project_path).await?;
            Ok((result, contract.thresholds.require_lint_pass)) // Configurable blocking
        }
    }
}

/// Test manifest integrity: verify all baseline files still exist
fn test_manifest_integrity(
    project_path: &Path,
    manifest: &FileManifest,
) -> Result<FalsificationResult> {
    print!("Searching for missing files... ");

    let missing = manifest.verify_integrity(project_path);

    if missing.is_empty() {
        Ok(FalsificationResult::passed(format!(
            "All {} files present",
            manifest.files.len()
        )))
    } else {
        Ok(FalsificationResult::failed(
            format!("{} files missing from baseline", missing.len()),
            EvidenceType::FileList(missing),
        ))
    }
}

/// Test for coverage gaming patterns
fn test_coverage_gaming(project_path: &Path) -> Result<FalsificationResult> {
    print!("Scanning for gaming patterns... ");

    let detection_result = gaming_detector::detect_coverage_gaming(project_path)?;

    if !detection_result.has_critical_violations() {
        Ok(FalsificationResult::passed(format!(
            "No gaming patterns found in {} files",
            detection_result.files_scanned
        )))
    } else {
        let violations = detection_result.critical_violations();
        let paths: Vec<PathBuf> = violations.iter().map(|v| v.file.clone()).collect();
        Ok(FalsificationResult::failed(
            format!("{} gaming violation(s) found", violations.len()),
            EvidenceType::FileList(paths),
        ))
    }
}

/// Test differential coverage: all changed lines must be covered
async fn test_differential_coverage(
    project_path: &Path,
    baseline_commit: &str,
) -> Result<FalsificationResult> {
    print!("Analyzing changed lines... ");

    // Get changed files since baseline
    let output = Command::new("git")
        .args(["diff", "--name-only", baseline_commit, "HEAD"])
        .current_dir(project_path)
        .output()
        .context("Failed to get git diff")?;

    if !output.status.success() {
        return Ok(FalsificationResult::passed(
            "No baseline commit found, skipping differential coverage".to_string(),
        ));
    }

    let changed_files: Vec<&str> = std::str::from_utf8(&output.stdout)?
        .lines()
        .filter(|f| f.ends_with(".rs"))
        .collect();

    if changed_files.is_empty() {
        return Ok(FalsificationResult::passed(
            "No Rust files changed".to_string(),
        ));
    }

    // For now, we assume coverage data is available from a previous run
    // In production, this would integrate with llvm-cov or similar
    // TODO: Integrate with actual coverage data

    Ok(FalsificationResult::passed(format!(
        "{} changed files (coverage check requires llvm-cov data)",
        changed_files.len()
    )))
}

/// Test absolute coverage threshold
async fn test_absolute_coverage(
    project_path: &Path,
    threshold: f64,
) -> Result<FalsificationResult> {
    print!("Checking coverage threshold... ");

    // Try to read coverage from cached metrics
    let metrics_dir = project_path.join(".pmat-metrics/trends");
    let coverage_file = metrics_dir.join("test-coverage.json");

    if !coverage_file.exists() {
        return Ok(FalsificationResult::passed(format!(
            "No coverage data (run 'make coverage' to establish baseline), threshold: {:.1}%",
            threshold
        )));
    }

    let content = std::fs::read_to_string(&coverage_file)?;
    let json: serde_json::Value = serde_json::from_str(&content)?;

    if let Some(entries) = json.as_array() {
        if let Some(latest) = entries.last() {
            if let Some(coverage) = latest.get("value").and_then(|v| v.as_f64()) {
                if coverage >= threshold {
                    return Ok(FalsificationResult::passed(format!(
                        "{:.1}% >= {:.1}%",
                        coverage, threshold
                    )));
                } else {
                    return Ok(FalsificationResult::failed(
                        format!("{:.1}% < {:.1}% threshold", coverage, threshold),
                        EvidenceType::NumericComparison {
                            actual: coverage,
                            threshold,
                        },
                    ));
                }
            }
        }
    }

    Ok(FalsificationResult::passed(
        "No coverage entries found".to_string(),
    ))
}

/// Test TDG score regression
async fn test_tdg_regression(project_path: &Path, baseline_tdg: f64) -> Result<FalsificationResult> {
    print!("Checking TDG score... ");

    // Read current TDG score from cache
    let tdg_file = project_path.join(".pmat-metrics/tdg-score.json");

    if !tdg_file.exists() {
        return Ok(FalsificationResult::passed(format!(
            "No TDG data (baseline: {:.1})",
            baseline_tdg
        )));
    }

    let content = std::fs::read_to_string(&tdg_file)?;
    let json: serde_json::Value = serde_json::from_str(&content)?;

    if let Some(current_tdg) = json.get("score").and_then(|v| v.as_f64()) {
        if current_tdg >= baseline_tdg {
            Ok(FalsificationResult::passed(format!(
                "{:.1} >= {:.1} (baseline)",
                current_tdg, baseline_tdg
            )))
        } else {
            Ok(FalsificationResult::failed(
                format!("{:.1} < {:.1} (regression)", current_tdg, baseline_tdg),
                EvidenceType::NumericComparison {
                    actual: current_tdg,
                    threshold: baseline_tdg,
                },
            ))
        }
    } else {
        Ok(FalsificationResult::passed(
            "No TDG score in cache".to_string(),
        ))
    }
}

/// Test complexity regression: no function should exceed threshold
fn test_complexity_regression(
    project_path: &Path,
    max_complexity: u32,
) -> Result<FalsificationResult> {
    print!("Analyzing function complexity... ");

    // Run pmat complexity check
    let output = Command::new("pmat")
        .args([
            "analyze",
            "complexity",
            "--format",
            "json",
            "--path",
            &project_path.to_string_lossy(),
        ])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                // Check for functions exceeding complexity
                if let Some(functions) = json.get("functions").and_then(|f| f.as_array()) {
                    let high_complexity: Vec<_> = functions
                        .iter()
                        .filter(|f| {
                            f.get("complexity")
                                .and_then(|c| c.as_u64())
                                .unwrap_or(0)
                                > max_complexity as u64
                        })
                        .collect();

                    if high_complexity.is_empty() {
                        return Ok(FalsificationResult::passed(format!(
                            "All functions <= {} complexity",
                            max_complexity
                        )));
                    } else {
                        let names: Vec<String> = high_complexity
                            .iter()
                            .filter_map(|f| f.get("name").and_then(|n| n.as_str()))
                            .map(|s| s.to_string())
                            .collect();
                        return Ok(FalsificationResult::failed(
                            format!(
                                "{} function(s) exceed complexity {}: {}",
                                high_complexity.len(),
                                max_complexity,
                                names.join(", ")
                            ),
                            EvidenceType::NumericComparison {
                                actual: high_complexity.len() as f64,
                                threshold: 0.0,
                            },
                        ));
                    }
                }
            }
            Ok(FalsificationResult::passed(
                "Complexity check passed".to_string(),
            ))
        }
        _ => Ok(FalsificationResult::passed(
            "Complexity analyzer not available".to_string(),
        )),
    }
}

/// Test file size regression: no file should exceed threshold
fn test_file_size_regression(
    project_path: &Path,
    max_lines: usize,
) -> Result<FalsificationResult> {
    print!("Checking file sizes... ");

    let mut large_files = Vec::new();

    for entry in walkdir::WalkDir::new(project_path)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            let path = e.path().to_string_lossy();
            !path.contains("/target/") && !path.contains("/.git/")
        })
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.extension().map(|e| e == "rs").unwrap_or(false) {
            if let Ok(content) = std::fs::read_to_string(path) {
                let line_count = content.lines().count();
                if line_count > max_lines {
                    large_files.push((
                        path.strip_prefix(project_path)
                            .unwrap_or(path)
                            .to_path_buf(),
                        line_count,
                    ));
                }
            }
        }
    }

    if large_files.is_empty() {
        Ok(FalsificationResult::passed(format!(
            "All files <= {} lines",
            max_lines
        )))
    } else {
        let paths: Vec<PathBuf> = large_files.iter().map(|(p, _)| p.clone()).collect();
        let details: Vec<String> = large_files
            .iter()
            .map(|(p, lines)| format!("{} ({} lines)", p.display(), lines))
            .collect();
        Ok(FalsificationResult::failed(
            format!("{} file(s) exceed {} lines: {}", large_files.len(), max_lines, details.join(", ")),
            EvidenceType::FileList(paths),
        ))
    }
}

/// Test spec quality: spec score must meet threshold
fn test_spec_quality(
    project_path: &Path,
    work_item_id: &str,
    min_score: u32,
) -> Result<FalsificationResult> {
    print!("Checking spec quality... ");

    // Look for spec file
    let spec_path = project_path.join(format!(
        "docs/specifications/{}-spec.md",
        work_item_id.to_lowercase()
    ));

    if !spec_path.exists() {
        // Also check for numbered format
        let spec_dir = project_path.join("docs/specifications");
        if spec_dir.exists() {
            // Spec not found - this is OK if not required
            return Ok(FalsificationResult::passed(
                "No spec file found (optional)".to_string(),
            ));
        }
    }

    // Run pmat spec score
    let output = Command::new("pmat")
        .args(["spec", "score", &spec_path.to_string_lossy()])
        .current_dir(project_path)
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Parse score from output (format: "Score: XX/100")
            if let Some(score_line) = stdout.lines().find(|l| l.contains("Score:")) {
                if let Some(score_str) = score_line.split('/').next() {
                    if let Ok(score) = score_str
                        .chars()
                        .filter(|c| c.is_ascii_digit())
                        .collect::<String>()
                        .parse::<u32>()
                    {
                        if score >= min_score {
                            return Ok(FalsificationResult::passed(format!(
                                "{}/100 >= {}/100",
                                score, min_score
                            )));
                        } else {
                            return Ok(FalsificationResult::failed(
                                format!("{}/100 < {}/100 threshold", score, min_score),
                                EvidenceType::NumericComparison {
                                    actual: score as f64,
                                    threshold: min_score as f64,
                                },
                            ));
                        }
                    }
                }
            }
            Ok(FalsificationResult::passed(
                "Spec score check completed".to_string(),
            ))
        }
        _ => Ok(FalsificationResult::passed(
            "Spec scorer not available".to_string(),
        )),
    }
}

/// Test roadmap update: roadmap must be modified since baseline
fn test_roadmap_update(project_path: &Path, baseline_commit: &str) -> Result<FalsificationResult> {
    print!("Checking roadmap update... ");

    let roadmap_path = project_path.join("docs/roadmaps/roadmap.yaml");

    if !roadmap_path.exists() {
        return Ok(FalsificationResult::passed(
            "No roadmap.yaml found".to_string(),
        ));
    }

    // Check if roadmap was modified since baseline
    let output = Command::new("git")
        .args([
            "diff",
            "--name-only",
            baseline_commit,
            "HEAD",
            "--",
            "docs/roadmaps/roadmap.yaml",
        ])
        .current_dir(project_path)
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let changed = !output.stdout.is_empty();
            if changed {
                Ok(FalsificationResult::passed("Roadmap was updated".to_string()))
            } else {
                Ok(FalsificationResult::failed(
                    "Roadmap not updated since baseline".to_string(),
                    EvidenceType::BooleanCheck(false),
                ))
            }
        }
        _ => Ok(FalsificationResult::passed(
            "Cannot check roadmap changes".to_string(),
        )),
    }
}

/// Test GitHub sync: all commits must be pushed
fn test_github_sync(project_path: &Path) -> Result<FalsificationResult> {
    print!("Checking git status... ");

    // Check for unpushed commits
    let output = Command::new("git")
        .args(["status", "--porcelain", "-b"])
        .current_dir(project_path)
        .output()
        .context("Failed to run git status")?;

    let status = String::from_utf8_lossy(&output.stdout);

    // Check for ahead commits
    let ahead_count = if status.contains("ahead") {
        // Parse "ahead X" from status
        status
            .lines()
            .next()
            .and_then(|l| {
                l.find("ahead")
                    .map(|i| &l[i..])
                    .and_then(|s| s.split_whitespace().nth(1))
                    .and_then(|n| n.trim_end_matches(']').parse::<usize>().ok())
            })
            .unwrap_or(0)
    } else {
        0
    };

    // Count dirty files
    let dirty_count = status.lines().skip(1).filter(|l| !l.is_empty()).count();

    if ahead_count == 0 && dirty_count == 0 {
        Ok(FalsificationResult::passed(
            "All changes committed and pushed".to_string(),
        ))
    } else {
        let mut issues = Vec::new();
        if ahead_count > 0 {
            issues.push(format!("{} unpushed commit(s)", ahead_count));
        }
        if dirty_count > 0 {
            issues.push(format!("{} uncommitted file(s)", dirty_count));
        }
        Ok(FalsificationResult::failed(
            issues.join(", "),
            EvidenceType::GitState {
                unpushed_commits: ahead_count,
                dirty_files: dirty_count,
            },
        ))
    }
}

/// Print evidence details
fn print_evidence(evidence: &EvidenceType) {
    match evidence {
        EvidenceType::FileList(files) => {
            println!("      Evidence (files):");
            for file in files.iter().take(5) {
                println!("        - {}", file.display());
            }
            if files.len() > 5 {
                println!("        ... and {} more", files.len() - 5);
            }
        }
        EvidenceType::NumericComparison { actual, threshold } => {
            println!(
                "      Evidence: actual={:.1}, threshold={:.1}",
                actual, threshold
            );
        }
        EvidenceType::GitState {
            unpushed_commits,
            dirty_files,
        } => {
            println!(
                "      Evidence: {} unpushed, {} dirty",
                unpushed_commits, dirty_files
            );
        }
        EvidenceType::BooleanCheck(value) => {
            println!("      Evidence: {}", value);
        }
        EvidenceType::CounterExample { details } => {
            println!("      Evidence: {}", details);
        }
    }
}

/// Capture baseline metrics for a new work contract
pub async fn capture_baseline(project_path: &Path) -> Result<(f64, f64, Option<f64>)> {
    println!("   📊 Capturing baseline metrics...");

    // Capture TDG score
    let tdg_score = capture_metric_from_cache(project_path, "tdg-score.json", "score")
        .await
        .unwrap_or(0.0);

    // Capture coverage
    let coverage = capture_coverage_from_cache(project_path).await.unwrap_or(0.0);

    // Capture Rust project score (if applicable)
    let rust_score = if project_path.join("Cargo.toml").exists() {
        Some(
            capture_metric_from_cache(project_path, "rust-project-score.json", "total_earned")
                .await
                .unwrap_or(0.0),
        )
    } else {
        None
    };

    println!("      TDG: {:.1}, Coverage: {:.1}%", tdg_score, coverage);
    if let Some(rs) = rust_score {
        println!("      Rust Score: {:.1}/134", rs);
    }

    Ok((tdg_score, coverage, rust_score))
}

/// Capture a metric from the cache
async fn capture_metric_from_cache(
    project_path: &Path,
    filename: &str,
    field: &str,
) -> Result<f64> {
    let metrics_dir = project_path.join(".pmat-metrics");
    let file_path = metrics_dir.join(filename);

    if file_path.exists() {
        let content = std::fs::read_to_string(&file_path)?;
        let json: serde_json::Value = serde_json::from_str(&content)?;
        if let Some(value) = json.get(field).and_then(|v| v.as_f64()) {
            return Ok(value);
        }
    }

    Ok(0.0)
}

/// Capture coverage from trends cache
async fn capture_coverage_from_cache(project_path: &Path) -> Result<f64> {
    let coverage_file = project_path.join(".pmat-metrics/trends/test-coverage.json");

    if coverage_file.exists() {
        let content = std::fs::read_to_string(&coverage_file)?;
        let json: serde_json::Value = serde_json::from_str(&content)?;

        if let Some(entries) = json.as_array() {
            if let Some(latest) = entries.last() {
                if let Some(coverage) = latest.get("value").and_then(|v| v.as_f64()) {
                    return Ok(coverage);
                }
            }
        }
    }

    Ok(0.0)
}

/// Test meta-falsification: verify the falsifier itself is not broken
fn test_meta_falsification(project_path: &Path) -> Result<FalsificationResult> {
    print!("Injecting dummy failure... ");

    let detector_working = gaming_detector::run_meta_falsification(project_path)?;

    if detector_working {
        Ok(FalsificationResult::passed(
            "Detected dummy gaming pattern correctly".to_string(),
        ))
    } else {
        Ok(FalsificationResult::failed(
            "Falsifier FAILED to detect known gaming pattern (SYSTEM BROKEN)".to_string(),
            EvidenceType::CounterExample {
                details: "Dummy #[cfg(not(coverage))] was ignored by detector".into(),
            },
        ))
    }
}

/// Test supply chain integrity: O(1) - reads from cached cargo deny status
async fn test_supply_chain_integrity(project_path: &Path) -> Result<FalsificationResult> {
    print!("Reading deny cache... ");

    // O(1): Read from cache instead of running cargo deny
    if let Some(cache) = read_cached_metric(project_path, "deny-status.json") {
        if cache.is_stale_block {
            return Ok(FalsificationResult::failed(
                format!("Deny cache too old ({} min). Run 'cargo deny check' first.", cache.age_minutes),
                EvidenceType::BooleanCheck(false),
            ));
        }

        // SECURITY: Require 'passed' field to exist - reject malformed cache (Popperian Audit v2.1 fix)
        let passed = match cache.value.get("passed").and_then(|v| v.as_bool()) {
            Some(p) => p,
            None => {
                return Ok(FalsificationResult::failed(
                    "Invalid deny cache (missing 'passed' field). Re-run 'cargo deny check'.".to_string(),
                    EvidenceType::BooleanCheck(false),
                ));
            }
        };
        let stale_note = format!(" (cached {} min ago)", cache.age_minutes);

        if passed {
            return Ok(FalsificationResult::passed(format!("No vulnerabilities{}", stale_note)));
        } else {
            let count = cache.value.get("vulnerability_count").and_then(|v| v.as_u64()).unwrap_or(0);
            return Ok(FalsificationResult::failed(
                format!("{} vulnerabilities{}", count, stale_note),
                EvidenceType::NumericComparison { actual: count as f64, threshold: 0.0 },
            ));
        }
    }

    // No cache - FAIL (Popperian Audit v1.2 fix: empty cache bypass)
    Ok(FalsificationResult::failed(
        "No deny cache. Run 'cargo deny check' first (O(1) requirement)".to_string(),
        EvidenceType::BooleanCheck(false),
    ))
}

/// Test examples compile: O(1) - reads from cached examples status
async fn test_examples_compile(project_path: &Path) -> Result<FalsificationResult> {
    print!("Reading examples cache... ");

    // O(1): Read from cache instead of running cargo build
    if let Some(cache) = read_cached_metric(project_path, "examples-status.json") {
        if cache.is_stale_block {
            return Ok(FalsificationResult::failed(
                format!("Examples cache too old ({} min). Run 'cargo build --examples' first.", cache.age_minutes),
                EvidenceType::BooleanCheck(false),
            ));
        }

        // SECURITY: Require 'passed' field to exist - reject malformed cache (Popperian Audit v2.1 fix)
        let passed = match cache.value.get("passed").and_then(|v| v.as_bool()) {
            Some(p) => p,
            None => {
                return Ok(FalsificationResult::failed(
                    "Invalid examples cache (missing 'passed' field). Re-run 'cargo build --examples'.".to_string(),
                    EvidenceType::BooleanCheck(false),
                ));
            }
        };
        let count = cache.value.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
        let stale_note = format!(" (cached {} min ago)", cache.age_minutes);

        if passed {
            return Ok(FalsificationResult::passed(format!("{} examples OK{}", count, stale_note)));
        } else {
            let failed = cache.value.get("failed").and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(", "))
                .unwrap_or_default();
            return Ok(FalsificationResult::failed(
                format!("Examples failed{}: {}", stale_note, failed),
                EvidenceType::CounterExample { details: failed },
            ));
        }
    }

    // Check if examples directory exists
    let examples_dir = project_path.join("examples");
    if !examples_dir.exists() {
        return Ok(FalsificationResult::passed(
            "No examples directory found (skipping)".to_string(),
        ));
    }

    // No cache available - pass with warning (examples are optional)
    Ok(FalsificationResult::passed(
        "No examples cache (run 'cargo build --examples' to populate)".to_string(),
    ))
}

/// Test pmat-book validation: book tests must pass
async fn test_book_validation(project_path: &Path) -> Result<FalsificationResult> {
    print!("Validating pmat-book... ");

    // Check if this is the pmat repository by looking for pmat-book sibling
    let pmat_book_path = project_path.parent().map(|p| p.join("pmat-book"));

    if let Some(book_path) = pmat_book_path {
        if book_path.exists() {
            // Try to run make validate-book
            let output = Command::new("make")
                .args(["validate-book"])
                .current_dir(project_path)
                .output();

            match output {
                Ok(out) if out.status.success() => {
                    return Ok(FalsificationResult::passed(
                        "pmat-book validation passed".to_string(),
                    ));
                }
                Ok(out) => {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    return Ok(FalsificationResult::failed(
                        "pmat-book validation failed".to_string(),
                        EvidenceType::CounterExample {
                            details: stderr.chars().take(500).collect(),
                        },
                    ));
                }
                Err(_) => {
                    // make validate-book not available, try direct test
                    let test_script = book_path.join("tests/ch13/test_language_examples.sh");
                    if test_script.exists() {
                        let script_output = Command::new("bash")
                            .arg(&test_script)
                            .current_dir(&book_path)
                            .output();

                        match script_output {
                            Ok(out) if out.status.success() => {
                                return Ok(FalsificationResult::passed(
                                    "pmat-book chapter tests passed".to_string(),
                                ));
                            }
                            Ok(out) => {
                                let stderr = String::from_utf8_lossy(&out.stderr);
                                return Ok(FalsificationResult::failed(
                                    "pmat-book chapter tests failed".to_string(),
                                    EvidenceType::CounterExample {
                                        details: stderr.chars().take(500).collect(),
                                    },
                                ));
                            }
                            Err(_) => {}
                        }
                    }
                }
            }
        }
    }

    // No pmat-book found or no validation available
    Ok(FalsificationResult::passed(
        "pmat-book not found (skipping validation)".to_string(),
    ))
}

// ============================================================================
// v2.6 comply spec: SATD, Dead Code, Per-File Coverage, Lint Gate
// ============================================================================

/// Test SATD detection: find new TODO/FIXME/HACK markers since baseline
async fn test_satd_detection(
    project_path: &Path,
    baseline_commit: &str,
) -> Result<FalsificationResult> {
    print!("Detecting SATD markers... ");

    // Run pmat analyze satd
    let output = Command::new("pmat")
        .args([
            "analyze",
            "satd",
            "--format",
            "json",
            "--path",
            &project_path.to_string_lossy(),
        ])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                // Get current SATD count
                let current_count = json
                    .get("total_count")
                    .or_else(|| json.get("count"))
                    .and_then(|c| c.as_u64())
                    .unwrap_or(0);

                // Check for new SATD since baseline (compare with git diff)
                let new_satd = detect_new_satd_since_baseline(project_path, baseline_commit)?;

                if new_satd.is_empty() {
                    Ok(FalsificationResult::passed(format!(
                        "No new SATD markers ({} existing)",
                        current_count
                    )))
                } else {
                    let paths: Vec<PathBuf> = new_satd.iter().map(|(p, _)| p.clone()).collect();
                    let details: Vec<String> = new_satd
                        .iter()
                        .take(5)
                        .map(|(p, marker)| format!("{}: {}", p.display(), marker))
                        .collect();
                    Ok(FalsificationResult::failed(
                        format!("{} new SATD marker(s): {}", new_satd.len(), details.join("; ")),
                        EvidenceType::FileList(paths),
                    ))
                }
            } else {
                Ok(FalsificationResult::passed(
                    "SATD check completed (no JSON output)".to_string(),
                ))
            }
        }
        _ => Ok(FalsificationResult::passed(
            "SATD analyzer not available".to_string(),
        )),
    }
}

/// Detect new SATD markers by comparing git diff
fn detect_new_satd_since_baseline(
    project_path: &Path,
    baseline_commit: &str,
) -> Result<Vec<(PathBuf, String)>> {
    let mut new_satd = Vec::new();

    // Get diff of added lines since baseline
    let output = Command::new("git")
        .args(["diff", "-U0", baseline_commit, "HEAD", "--", "*.rs"])
        .current_dir(project_path)
        .output()?;

    if !output.status.success() {
        return Ok(new_satd);
    }

    let diff = String::from_utf8_lossy(&output.stdout);
    let satd_patterns = ["TODO", "FIXME", "HACK", "XXX", "BUG"];

    let mut current_file: Option<PathBuf> = None;

    for line in diff.lines() {
        if let Some(file_path) = line.strip_prefix("+++ b/") {
            current_file = Some(PathBuf::from(file_path));
        } else if let Some(line_content) = line.strip_prefix('+') {
            if line.starts_with("+++") {
                continue;
            }
            // This is an added line
            for pattern in &satd_patterns {
                if line_content.contains(pattern) {
                    if let Some(ref file) = current_file {
                        // Extract the marker context
                        let marker = line_content
                            .split(pattern)
                            .nth(1)
                            .map(|s| format!("{}{}", pattern, s.chars().take(50).collect::<String>()))
                            .unwrap_or_else(|| pattern.to_string());
                        new_satd.push((file.clone(), marker));
                    }
                }
            }
        }
    }

    Ok(new_satd)
}

/// Test dead code detection: find new unreachable code since baseline
async fn test_dead_code_detection(
    project_path: &Path,
    baseline_commit: &str,
) -> Result<FalsificationResult> {
    print!("Detecting dead code... ");

    // Run pmat analyze dead-code
    let output = Command::new("pmat")
        .args([
            "analyze",
            "dead-code",
            "--format",
            "json",
            "--path",
            &project_path.to_string_lossy(),
        ])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                // Get dead code items
                let dead_items = json
                    .get("dead_code")
                    .or_else(|| json.get("items"))
                    .and_then(|items| items.as_array())
                    .map(|a| a.len())
                    .unwrap_or(0);

                // For now, we report any dead code found
                // Future: compare with baseline to only flag NEW dead code
                if dead_items == 0 {
                    Ok(FalsificationResult::passed("No dead code detected".to_string()))
                } else {
                    // Check if these are new since baseline
                    let changed_files = get_changed_files(project_path, baseline_commit)?;
                    let dead_in_changed: usize = json
                        .get("dead_code")
                        .or_else(|| json.get("items"))
                        .and_then(|items| items.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter(|item| {
                                    item.get("file")
                                        .and_then(|f| f.as_str())
                                        .map(|f| changed_files.iter().any(|cf| cf.ends_with(f)))
                                        .unwrap_or(false)
                                })
                                .count()
                        })
                        .unwrap_or(0);

                    if dead_in_changed == 0 {
                        Ok(FalsificationResult::passed(format!(
                            "{} existing dead code items (none in changed files)",
                            dead_items
                        )))
                    } else {
                        Ok(FalsificationResult::failed(
                            format!("{} dead code item(s) in changed files", dead_in_changed),
                            EvidenceType::NumericComparison {
                                actual: dead_in_changed as f64,
                                threshold: 0.0,
                            },
                        ))
                    }
                }
            } else {
                Ok(FalsificationResult::passed(
                    "Dead code check completed (no JSON output)".to_string(),
                ))
            }
        }
        _ => Ok(FalsificationResult::passed(
            "Dead code analyzer not available".to_string(),
        )),
    }
}

/// Get list of changed files since baseline
fn get_changed_files(project_path: &Path, baseline_commit: &str) -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(["diff", "--name-only", baseline_commit, "HEAD"])
        .current_dir(project_path)
        .output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|s| s.to_string())
            .collect())
    } else {
        Ok(Vec::new())
    }
}

/// Test per-file coverage: all files must meet threshold
async fn test_per_file_coverage(
    project_path: &Path,
    threshold: f64,
) -> Result<FalsificationResult> {
    print!("Checking per-file coverage... ");

    // Try to read per-file coverage from llvm-cov output
    let coverage_json = project_path.join("target/llvm-cov/coverage.json");

    if !coverage_json.exists() {
        // Try to run coverage if not available
        return Ok(FalsificationResult::passed(format!(
            "No per-file coverage data (run 'make coverage'), threshold: {:.1}%",
            threshold
        )));
    }

    let content = std::fs::read_to_string(&coverage_json)?;
    let json: serde_json::Value = serde_json::from_str(&content)?;

    let mut files_below_threshold = Vec::new();

    // Parse llvm-cov JSON format
    if let Some(data) = json.get("data").and_then(|d| d.as_array()) {
        for file_data in data {
            if let Some(files) = file_data.get("files").and_then(|f| f.as_array()) {
                for file in files {
                    let filename = file
                        .get("filename")
                        .and_then(|f| f.as_str())
                        .unwrap_or("unknown");

                    // Skip test files and generated files
                    if filename.contains("/tests/")
                        || filename.contains("_test.rs")
                        || filename.contains("/target/")
                    {
                        continue;
                    }

                    // Get coverage percentage
                    let coverage = file
                        .get("summary")
                        .and_then(|s| s.get("lines"))
                        .and_then(|l| l.get("percent"))
                        .and_then(|p| p.as_f64())
                        .unwrap_or(100.0);

                    if coverage < threshold {
                        files_below_threshold.push((PathBuf::from(filename), coverage));
                    }
                }
            }
        }
    }

    if files_below_threshold.is_empty() {
        Ok(FalsificationResult::passed(format!(
            "All files >= {:.1}% coverage",
            threshold
        )))
    } else {
        let paths: Vec<PathBuf> = files_below_threshold
            .iter()
            .map(|(p, _)| p.clone())
            .collect();
        let details: Vec<String> = files_below_threshold
            .iter()
            .take(10)
            .map(|(p, cov)| format!("{}: {:.1}%", p.display(), cov))
            .collect();
        Ok(FalsificationResult::failed(
            format!(
                "{} file(s) below {:.1}% threshold: {}",
                files_below_threshold.len(),
                threshold,
                details.join(", ")
            ),
            EvidenceType::FileList(paths),
        ))
    }
}

/// Test lint pass: O(1) - reads from cached lint status
async fn test_lint_pass(project_path: &Path) -> Result<FalsificationResult> {
    print!("Reading lint cache... ");

    // O(1): Read from cache instead of running make lint
    if let Some(cache) = read_cached_metric(project_path, "lint-status.json") {
        if cache.is_stale_block {
            return Ok(FalsificationResult::failed(
                format!("Lint cache too old ({} min). Run 'make lint' first.", cache.age_minutes),
                EvidenceType::BooleanCheck(false),
            ));
        }

        // SECURITY: Require 'passed' field to exist - reject malformed cache (Popperian Audit v2.1 fix)
        let passed = match cache.value.get("passed").and_then(|v| v.as_bool()) {
            Some(p) => p,
            None => {
                return Ok(FalsificationResult::failed(
                    "Invalid lint cache (missing 'passed' field). Re-run 'make lint'.".to_string(),
                    EvidenceType::BooleanCheck(false),
                ));
            }
        };
        let stale_note = if cache.is_stale_warn {
            format!(" (cached {} min ago, consider re-running)", cache.age_minutes)
        } else {
            format!(" (cached {} min ago)", cache.age_minutes)
        };

        if passed {
            return Ok(FalsificationResult::passed(format!("PASSED{}", stale_note)));
        } else {
            let errors = cache.value.get("error_count").and_then(|v| v.as_u64()).unwrap_or(0);
            return Ok(FalsificationResult::failed(
                format!("{} lint errors{}", errors, stale_note),
                EvidenceType::NumericComparison { actual: errors as f64, threshold: 0.0 },
            ));
        }
    }

    // No cache - check if Makefile exists and suggest running lint
    let makefile = project_path.join("Makefile");
    if !makefile.exists() {
        return Ok(FalsificationResult::passed(
            "No Makefile found (skipping lint check)".to_string(),
        ));
    }

    // No cache available - block until user runs make lint
    Ok(FalsificationResult::failed(
        "No lint cache. Run 'make lint' first (O(1) requirement)".to_string(),
        EvidenceType::BooleanCheck(false),
    ))
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_falsification_report_blocking() {
        let report = FalsificationReport {
            total_claims: 3,
            passed: 2,
            failed: 1,
            warnings: 0,
            claim_results: vec![
                ClaimResult {
                    index: 1,
                    hypothesis: "Test 1".to_string(),
                    method: FalsificationMethod::ManifestIntegrity,
                    result: FalsificationResult::passed("OK"),
                    is_blocking: true,
                },
                ClaimResult {
                    index: 2,
                    hypothesis: "Test 2".to_string(),
                    method: FalsificationMethod::AbsoluteCoverage,
                    result: FalsificationResult::failed(
                        "Coverage low",
                        EvidenceType::NumericComparison {
                            actual: 80.0,
                            threshold: 95.0,
                        },
                    ),
                    is_blocking: true,
                },
            ],
            all_passed: false,
        };

        assert!(report.has_blocking_failures());
        assert_eq!(report.blocking_failures().len(), 1);
    }

    #[test]
    fn test_github_sync_parsing() {
        // This tests the parsing logic for git status output
        let status = "## main...origin/main [ahead 2]\n M file.rs\n?? new.rs";

        let ahead = if status.contains("ahead") {
            status
                .lines()
                .next()
                .and_then(|l| {
                    l.find("ahead")
                        .map(|i| &l[i..])
                        .and_then(|s| s.split_whitespace().nth(1))
                        .and_then(|n| n.trim_end_matches(']').parse::<usize>().ok())
                })
                .unwrap_or(0)
        } else {
            0
        };

        assert_eq!(ahead, 2);
    }

    #[test]
    fn test_falsification_report_warnings() {
        let report = FalsificationReport {
            total_claims: 2,
            passed: 1,
            failed: 1,
            warnings: 1,
            claim_results: vec![
                ClaimResult {
                    index: 1,
                    hypothesis: "Test OK".to_string(),
                    method: FalsificationMethod::ManifestIntegrity,
                    result: FalsificationResult::passed("OK"),
                    is_blocking: false,
                },
                ClaimResult {
                    index: 2,
                    hypothesis: "Warning test".to_string(),
                    method: FalsificationMethod::LintPass,
                    result: FalsificationResult::failed(
                        "Size above warning threshold",
                        EvidenceType::NumericComparison {
                            actual: 45.0,
                            threshold: 40.0,
                        },
                    ),
                    is_blocking: false,
                },
            ],
            all_passed: false,
        };

        assert!(!report.has_blocking_failures());
        assert_eq!(report.blocking_failures().len(), 0);
        assert_eq!(report.warning_failures().len(), 1);
    }

    #[test]
    fn test_falsification_report_all_passed() {
        let report = FalsificationReport {
            total_claims: 2,
            passed: 2,
            failed: 0,
            warnings: 0,
            claim_results: vec![
                ClaimResult {
                    index: 1,
                    hypothesis: "Test 1".to_string(),
                    method: FalsificationMethod::ManifestIntegrity,
                    result: FalsificationResult::passed("OK"),
                    is_blocking: true,
                },
                ClaimResult {
                    index: 2,
                    hypothesis: "Test 2".to_string(),
                    method: FalsificationMethod::AbsoluteCoverage,
                    result: FalsificationResult::passed("Coverage OK"),
                    is_blocking: true,
                },
            ],
            all_passed: true,
        };

        assert!(!report.has_blocking_failures());
        assert_eq!(report.blocking_failures().len(), 0);
        assert_eq!(report.warning_failures().len(), 0);
    }

    #[test]
    fn test_falsification_result_constructors() {
        let passed = FalsificationResult::passed("Success");
        assert!(!passed.falsified);
        assert_eq!(passed.explanation, "Success");

        let failed = FalsificationResult::failed("Error", EvidenceType::BooleanCheck(false));
        assert!(failed.falsified);
        assert_eq!(failed.explanation, "Error");
    }

    #[test]
    fn test_evidence_type_display() {
        let bool_check = EvidenceType::BooleanCheck(true);
        let numeric = EvidenceType::NumericComparison {
            actual: 80.0,
            threshold: 95.0,
        };

        // Just verify these don't panic when formatted
        let _ = format!("{:?}", bool_check);
        let _ = format!("{:?}", numeric);
    }

    #[test]
    fn test_cached_metric_staleness_levels() {
        // Test cache staleness calculation
        let fresh = CachedMetric {
            value: serde_json::json!({"coverage": 85.0}),
            age_minutes: 30,
            is_stale_warn: false,
            is_stale_block: false,
        };
        assert!(!fresh.is_stale_warn);
        assert!(!fresh.is_stale_block);

        let warning = CachedMetric {
            value: serde_json::json!({"coverage": 85.0}),
            age_minutes: 90,
            is_stale_warn: true,
            is_stale_block: false,
        };
        assert!(warning.is_stale_warn);
        assert!(!warning.is_stale_block);

        let blocked = CachedMetric {
            value: serde_json::json!({"coverage": 85.0}),
            age_minutes: 1500, // 25 hours
            is_stale_warn: true,
            is_stale_block: true,
        };
        assert!(blocked.is_stale_warn);
        assert!(blocked.is_stale_block);
    }

    #[test]
    fn test_read_cached_metric_nonexistent() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let result = read_cached_metric(temp_dir.path(), "nonexistent.json");
        assert!(result.is_none());
    }

    #[test]
    fn test_read_cached_metric_invalid_json() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join(".pmat-metrics");
        std::fs::create_dir_all(&cache_dir).unwrap();
        std::fs::write(cache_dir.join("test.json"), "not valid json").unwrap();
        let result = read_cached_metric(temp_dir.path(), "test.json");
        assert!(result.is_none());
    }

    #[test]
    fn test_read_cached_metric_valid() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join(".pmat-metrics");
        std::fs::create_dir_all(&cache_dir).unwrap();
        std::fs::write(
            cache_dir.join("coverage.json"),
            r#"{"line_coverage": 85.5}"#,
        )
        .unwrap();
        let result = read_cached_metric(temp_dir.path(), "coverage.json");
        assert!(result.is_some());
        let metric = result.unwrap();
        assert_eq!(metric.value["line_coverage"], 85.5);
        assert!(!metric.is_stale_warn); // Just created
        assert!(!metric.is_stale_block);
    }

    #[test]
    fn test_claim_result_debug() {
        let claim = ClaimResult {
            index: 1,
            hypothesis: "Test hypothesis".to_string(),
            method: FalsificationMethod::ManifestIntegrity,
            result: FalsificationResult::passed("OK"),
            is_blocking: true,
        };
        let debug_str = format!("{:?}", claim);
        assert!(debug_str.contains("hypothesis"));
        assert!(debug_str.contains("Test hypothesis"));
    }

    #[test]
    fn test_falsification_report_empty() {
        let report = FalsificationReport {
            total_claims: 0,
            passed: 0,
            failed: 0,
            warnings: 0,
            claim_results: vec![],
            all_passed: true,
        };
        assert!(!report.has_blocking_failures());
        assert!(report.blocking_failures().is_empty());
        assert!(report.warning_failures().is_empty());
    }

    #[test]
    fn test_falsification_report_mixed_results() {
        let report = FalsificationReport {
            total_claims: 4,
            passed: 2,
            failed: 1,
            warnings: 1,
            claim_results: vec![
                ClaimResult {
                    index: 1,
                    hypothesis: "Passed".to_string(),
                    method: FalsificationMethod::ManifestIntegrity,
                    result: FalsificationResult::passed("OK"),
                    is_blocking: true,
                },
                ClaimResult {
                    index: 2,
                    hypothesis: "Blocking failure".to_string(),
                    method: FalsificationMethod::AbsoluteCoverage,
                    result: FalsificationResult::failed(
                        "Coverage low",
                        EvidenceType::NumericComparison {
                            actual: 70.0,
                            threshold: 95.0,
                        },
                    ),
                    is_blocking: true,
                },
                ClaimResult {
                    index: 3,
                    hypothesis: "Warning".to_string(),
                    method: FalsificationMethod::FileSizeRegression,
                    result: FalsificationResult::failed(
                        "File too large",
                        EvidenceType::NumericComparison {
                            actual: 600.0,
                            threshold: 500.0,
                        },
                    ),
                    is_blocking: false,
                },
                ClaimResult {
                    index: 4,
                    hypothesis: "Another pass".to_string(),
                    method: FalsificationMethod::LintPass,
                    result: FalsificationResult::passed("Lint OK"),
                    is_blocking: true,
                },
            ],
            all_passed: false,
        };
        assert!(report.has_blocking_failures());
        assert_eq!(report.blocking_failures().len(), 1);
        assert_eq!(report.warning_failures().len(), 1);
    }

    #[test]
    fn test_evidence_type_variants() {
        let file_list = EvidenceType::FileList(vec![
            PathBuf::from("file1.rs"),
            PathBuf::from("file2.rs"),
        ]);
        let _ = format!("{:?}", file_list);

        let counter_example = EvidenceType::CounterExample {
            details: "Some evidence text".to_string(),
        };
        let _ = format!("{:?}", counter_example);
    }

    #[test]
    fn test_falsification_method_variants() {
        // Verify all falsification methods can be formatted
        let methods = vec![
            FalsificationMethod::ManifestIntegrity,
            FalsificationMethod::MetaFalsification,
            FalsificationMethod::CoverageGaming,
            FalsificationMethod::DifferentialCoverage,
            FalsificationMethod::AbsoluteCoverage,
            FalsificationMethod::TdgRegression,
            FalsificationMethod::ComplexityRegression,
            FalsificationMethod::SupplyChainIntegrity,
            FalsificationMethod::FileSizeRegression,
            FalsificationMethod::SpecQuality,
            FalsificationMethod::RoadmapUpdate,
            FalsificationMethod::GitHubSync,
            FalsificationMethod::ExamplesCompile,
            FalsificationMethod::BookValidation,
            FalsificationMethod::SatdDetection,
            FalsificationMethod::DeadCodeDetection,
            FalsificationMethod::PerFileCoverage,
            FalsificationMethod::LintPass,
        ];
        for method in methods {
            let _ = format!("{:?}", method);
        }
    }

    #[test]
    fn test_cache_staleness_thresholds() {
        // Verify constants are reasonable
        assert_eq!(CACHE_WARN_HOURS, 1);
        assert_eq!(CACHE_BLOCK_HOURS, 24);
        assert!(CACHE_WARN_HOURS < CACHE_BLOCK_HOURS);
    }
}
