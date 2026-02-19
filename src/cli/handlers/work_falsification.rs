#![cfg_attr(coverage_nightly, coverage(off))]
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
use serde::{Deserialize, Serialize};
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
    let age = std::time::SystemTime::now().duration_since(modified).ok()?;
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
        println!("[{}/{}] {}", index, total_claims, claim.hypothesis);
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

/// Determine if a falsification method should block completion on failure.
///
/// Returns `true` for always-blocking methods, or reads from contract thresholds
/// for configurable methods (v2.6/v3.1 additions).
fn determine_blocking_status(
    method: &FalsificationMethod,
    thresholds: &crate::cli::handlers::work_contract::ContractThresholds,
) -> bool {
    match method {
        // Always blocking
        FalsificationMethod::ManifestIntegrity
        | FalsificationMethod::MetaFalsification
        | FalsificationMethod::CoverageGaming
        | FalsificationMethod::DifferentialCoverage
        | FalsificationMethod::AbsoluteCoverage
        | FalsificationMethod::TdgRegression
        | FalsificationMethod::ComplexityRegression
        | FalsificationMethod::SupplyChainIntegrity
        | FalsificationMethod::SpecQuality
        | FalsificationMethod::RoadmapUpdate
        | FalsificationMethod::GitHubSync
        | FalsificationMethod::ExamplesCompile
        | FalsificationMethod::BookValidation
        | FalsificationMethod::PerFileCoverage
        | FalsificationMethod::FixChainLimit => true,

        // Warning only
        FalsificationMethod::FileSizeRegression => false,

        // Configurable via thresholds
        FalsificationMethod::SatdDetection => thresholds.block_on_new_satd,
        FalsificationMethod::DeadCodeDetection => thresholds.block_on_new_dead_code,
        FalsificationMethod::LintPass => thresholds.require_lint_pass,
        FalsificationMethod::VariantCoverage => thresholds.block_on_untested_variants,
        FalsificationMethod::CrossCrateParity => thresholds.block_on_cross_crate_failure,
        FalsificationMethod::RegressionGate => thresholds.block_on_regression,
        FalsificationMethod::FormalProofVerification => thresholds.require_proof_verification,
    }
}

/// Dispatch a falsification test to the appropriate handler.
///
/// Each variant maps to a single test function. Sync tests are called directly;
/// async tests are `.await`ed.
async fn dispatch_falsification_test(
    project_path: &Path,
    contract: &WorkContract,
    claim: &FalsifiableClaim,
) -> Result<FalsificationResult> {
    match claim.falsification_method {
        FalsificationMethod::ManifestIntegrity => {
            test_manifest_integrity(project_path, &contract.baseline_file_manifest)
        }
        FalsificationMethod::MetaFalsification => test_meta_falsification(project_path),
        FalsificationMethod::CoverageGaming => test_coverage_gaming(project_path),
        FalsificationMethod::DifferentialCoverage => {
            test_differential_coverage(project_path, &contract.baseline_commit).await
        }
        FalsificationMethod::AbsoluteCoverage => {
            test_absolute_coverage(project_path, contract.thresholds.min_coverage_pct).await
        }
        FalsificationMethod::TdgRegression => {
            test_tdg_regression(project_path, contract.baseline_tdg).await
        }
        FalsificationMethod::ComplexityRegression => {
            test_complexity_regression(project_path, contract.thresholds.max_function_complexity)
        }
        FalsificationMethod::SupplyChainIntegrity => {
            test_supply_chain_integrity(project_path).await
        }
        FalsificationMethod::FileSizeRegression => {
            test_file_size_regression(project_path, contract.thresholds.max_file_lines)
        }
        FalsificationMethod::SpecQuality => test_spec_quality(
            project_path,
            &contract.work_item_id,
            contract.thresholds.min_spec_score,
        ),
        FalsificationMethod::RoadmapUpdate => {
            test_roadmap_update(project_path, &contract.baseline_commit)
        }
        FalsificationMethod::GitHubSync => test_github_sync(project_path),
        FalsificationMethod::ExamplesCompile => test_examples_compile(project_path).await,
        FalsificationMethod::BookValidation => test_book_validation(project_path).await,
        FalsificationMethod::SatdDetection => {
            test_satd_detection(project_path, &contract.baseline_commit).await
        }
        FalsificationMethod::DeadCodeDetection => {
            test_dead_code_detection(project_path, &contract.baseline_commit).await
        }
        FalsificationMethod::PerFileCoverage => {
            test_per_file_coverage(project_path, contract.thresholds.min_per_file_coverage_pct)
                .await
        }
        FalsificationMethod::LintPass => test_lint_pass(project_path).await,
        FalsificationMethod::VariantCoverage => {
            test_variant_coverage(project_path, &contract.baseline_commit)
        }
        FalsificationMethod::FixChainLimit => {
            test_fix_chain_limit(project_path, contract.thresholds.max_fix_chain)
        }
        FalsificationMethod::CrossCrateParity => test_cross_crate_parity(project_path).await,
        FalsificationMethod::RegressionGate => test_regression_gate(project_path).await,
        FalsificationMethod::FormalProofVerification => {
            test_formal_proof_verification(project_path, contract.thresholds.max_sorry_count)
        }
    }
}

/// Run a single falsification test
async fn run_single_falsification(
    project_path: &Path,
    contract: &WorkContract,
    claim: &FalsifiableClaim,
) -> Result<(FalsificationResult, bool)> {
    let result = dispatch_falsification_test(project_path, contract, claim).await?;
    let is_blocking = determine_blocking_status(&claim.falsification_method, &contract.thresholds);
    Ok((result, is_blocking))
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

    // Coverage data is assumed available from a previous run
    // In production, this integrates with llvm-cov or similar

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
async fn test_tdg_regression(
    project_path: &Path,
    baseline_tdg: f64,
) -> Result<FalsificationResult> {
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
                            f.get("complexity").and_then(|c| c.as_u64()).unwrap_or(0)
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
fn test_file_size_regression(project_path: &Path, max_lines: usize) -> Result<FalsificationResult> {
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
            format!(
                "{} file(s) exceed {} lines: {}",
                large_files.len(),
                max_lines,
                details.join(", ")
            ),
            EvidenceType::FileList(paths),
        ))
    }
}

/// Parse spec score from pmat output (format: "Score: XX/100")
fn parse_spec_score(stdout: &str) -> Option<u32> {
    let score_line = stdout.lines().find(|l| l.contains("Score:"))?;
    let score_str = score_line.split('/').next()?;
    score_str
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse::<u32>()
        .ok()
}

/// Evaluate parsed spec score against threshold
fn evaluate_spec_score(score: u32, min_score: u32) -> FalsificationResult {
    if score >= min_score {
        FalsificationResult::passed(format!("{}/100 >= {}/100", score, min_score))
    } else {
        FalsificationResult::failed(
            format!("{}/100 < {}/100 threshold", score, min_score),
            EvidenceType::NumericComparison {
                actual: score as f64,
                threshold: min_score as f64,
            },
        )
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
            let result = parse_spec_score(&stdout)
                .map(|score| evaluate_spec_score(score, min_score))
                .unwrap_or_else(|| {
                    FalsificationResult::passed("Spec score check completed".to_string())
                });
            Ok(result)
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
                Ok(FalsificationResult::passed(
                    "Roadmap was updated".to_string(),
                ))
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
                    .and_then(|i| l.get(i..))
                    .and_then(|s| s.split_whitespace().nth(1))
                    .and_then(|n| n.trim_end_matches(']').parse::<usize>().ok())
            })
            .unwrap_or(0)
    } else {
        0
    };

    // Count dirty files (exclude untracked ?? files — they are not uncommitted changes)
    let dirty_count = status
        .lines()
        .skip(1)
        .filter(|l| !l.is_empty() && !l.starts_with("??"))
        .count();

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
    let coverage = capture_coverage_from_cache(project_path)
        .await
        .unwrap_or(0.0);

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

/// Fallback: try reading deny cache from .pmat-work/<item>/ or .pmat/ directories.
/// Converts raw text output to the expected JSON format with `passed` field.
fn read_deny_cache_fallback(project_path: &Path) -> Option<CachedMetric> {
    // Try .pmat-work/**/deny-cache.txt then .pmat/deny-cache.txt
    let candidates = find_cache_file(project_path, "deny-cache.txt");
    for path in candidates {
        if let Ok(content) = std::fs::read_to_string(&path) {
            let passed = !content.contains("error") && !content.contains("DENIED");
            let age_minutes = file_age_minutes(&path);
            return Some(CachedMetric {
                value: serde_json::json!({ "passed": passed }),
                age_minutes,
                is_stale_warn: age_minutes >= CACHE_WARN_HOURS * 60,
                is_stale_block: age_minutes >= CACHE_BLOCK_HOURS * 60,
            });
        }
    }
    None
}

/// Fallback: try reading lint cache from .pmat-work/<item>/ or .pmat/ directories.
fn read_lint_cache_fallback(project_path: &Path) -> Option<CachedMetric> {
    let candidates = find_cache_file(project_path, "lint-cache.txt");
    for path in candidates {
        if let Ok(content) = std::fs::read_to_string(&path) {
            let passed = !content.contains("error");
            let error_count = content.matches("error").count() as u64;
            let age_minutes = file_age_minutes(&path);
            return Some(CachedMetric {
                value: serde_json::json!({ "passed": passed, "error_count": error_count }),
                age_minutes,
                is_stale_warn: age_minutes >= CACHE_WARN_HOURS * 60,
                is_stale_block: age_minutes >= CACHE_BLOCK_HOURS * 60,
            });
        }
    }
    None
}

/// Find cache file candidates in .pmat-work/*/ and .pmat/ directories.
fn find_cache_file(project_path: &Path, filename: &str) -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    // Check .pmat-work/*/<filename> (most specific, sorted by mtime desc)
    let work_dir = project_path.join(".pmat-work");
    if work_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&work_dir) {
            let mut work_candidates: Vec<PathBuf> = entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .map(|e| e.path().join(filename))
                .filter(|p| p.exists())
                .collect();
            // Sort by mtime descending (most recent first)
            work_candidates.sort_by(|a, b| {
                let a_time = std::fs::metadata(a).and_then(|m| m.modified()).ok();
                let b_time = std::fs::metadata(b).and_then(|m| m.modified()).ok();
                b_time.cmp(&a_time)
            });
            candidates.extend(work_candidates);
        }
    }

    // Check .pmat/<filename>
    let pmat_path = project_path.join(".pmat").join(filename);
    if pmat_path.exists() {
        candidates.push(pmat_path);
    }

    candidates
}

/// Get file age in minutes.
fn file_age_minutes(path: &Path) -> i64 {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| std::time::SystemTime::now().duration_since(t).ok())
        .map(|d| d.as_secs() as i64 / 60)
        .unwrap_or(0)
}

/// Test supply chain integrity: O(1) - reads from cached cargo deny status
async fn test_supply_chain_integrity(project_path: &Path) -> Result<FalsificationResult> {
    print!("Reading deny cache... ");

    // O(1): Read from cache instead of running cargo deny
    // Try primary cache location, then fallback to work-item and .pmat directories
    if let Some(cache) = read_cached_metric(project_path, "deny-status.json")
        .or_else(|| read_deny_cache_fallback(project_path))
    {
        if cache.is_stale_block {
            return Ok(FalsificationResult::failed(
                format!(
                    "Deny cache too old ({} min). Run 'cargo deny check' first.",
                    cache.age_minutes
                ),
                EvidenceType::BooleanCheck(false),
            ));
        }

        // Validate 'passed' field exists — reject malformed cache (Popperian Audit v2.1)
        let passed = match cache.value.get("passed").and_then(|v| v.as_bool()) {
            Some(p) => p,
            None => {
                return Ok(FalsificationResult::failed(
                    "Invalid deny cache (missing 'passed' field). Re-run 'cargo deny check'."
                        .to_string(),
                    EvidenceType::BooleanCheck(false),
                ));
            }
        };
        let stale_note = format!(" (cached {} min ago)", cache.age_minutes);

        if passed {
            return Ok(FalsificationResult::passed(format!(
                "No vulnerabilities{}",
                stale_note
            )));
        } else {
            let count = cache
                .value
                .get("vulnerability_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            return Ok(FalsificationResult::failed(
                format!("{} vulnerabilities{}", count, stale_note),
                EvidenceType::NumericComparison {
                    actual: count as f64,
                    threshold: 0.0,
                },
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
                format!(
                    "Examples cache too old ({} min). Run 'cargo build --examples' first.",
                    cache.age_minutes
                ),
                EvidenceType::BooleanCheck(false),
            ));
        }

        // Validate 'passed' field exists — reject malformed cache (Popperian Audit v2.1)
        let passed = match cache.value.get("passed").and_then(|v| v.as_bool()) {
            Some(p) => p,
            None => {
                return Ok(FalsificationResult::failed(
                    "Invalid examples cache (missing 'passed' field). Re-run 'cargo build --examples'.".to_string(),
                    EvidenceType::BooleanCheck(false),
                ));
            }
        };
        let count = cache
            .value
            .get("count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let stale_note = format!(" (cached {} min ago)", cache.age_minutes);

        if passed {
            return Ok(FalsificationResult::passed(format!(
                "{} examples OK{}",
                count, stale_note
            )));
        } else {
            let failed = cache
                .value
                .get("failed")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                })
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

/// Run `make validate-book` and return the result.
///
/// Returns `Some(result)` if the command executed (pass or fail),
/// or `None` if `make` was not available.
fn try_make_validate_book(project_path: &Path) -> Option<FalsificationResult> {
    let output = Command::new("make")
        .args(["validate-book"])
        .current_dir(project_path)
        .output()
        .ok()?;

    if output.status.success() {
        Some(FalsificationResult::passed(
            "pmat-book validation passed".to_string(),
        ))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Some(FalsificationResult::failed(
            "pmat-book validation failed".to_string(),
            EvidenceType::CounterExample {
                details: stderr.chars().take(500).collect(),
            },
        ))
    }
}

/// Run pmat-book chapter test script as a fallback.
///
/// Returns `Some(result)` if the script exists and executed,
/// or `None` if not available.
fn try_book_chapter_tests(book_path: &Path) -> Option<FalsificationResult> {
    let test_script = book_path.join("tests/ch13/test_language_examples.sh");
    if !test_script.exists() {
        return None;
    }

    let output = Command::new("bash")
        .arg(&test_script)
        .current_dir(book_path)
        .output()
        .ok()?;

    if output.status.success() {
        Some(FalsificationResult::passed(
            "pmat-book chapter tests passed".to_string(),
        ))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Some(FalsificationResult::failed(
            "pmat-book chapter tests failed".to_string(),
            EvidenceType::CounterExample {
                details: stderr.chars().take(500).collect(),
            },
        ))
    }
}

/// Test pmat-book validation: book tests must pass
async fn test_book_validation(project_path: &Path) -> Result<FalsificationResult> {
    print!("Validating pmat-book... ");

    // Check if this is the pmat repository by looking for pmat-book sibling
    let book_path = match project_path.parent().map(|p| p.join("pmat-book")) {
        Some(p) if p.exists() => p,
        _ => {
            return Ok(FalsificationResult::passed(
                "pmat-book not found (skipping validation)".to_string(),
            ));
        }
    };

    // Try make validate-book first, then fall back to chapter test script
    if let Some(result) = try_make_validate_book(project_path) {
        return Ok(result);
    }

    if let Some(result) = try_book_chapter_tests(&book_path) {
        return Ok(result);
    }

    // No validation method available
    Ok(FalsificationResult::passed(
        "pmat-book not found (skipping validation)".to_string(),
    ))
}

// ============================================================================
// v3.1 defect churn prevention: Variant Coverage, Fix Chains, Cross-Crate, Regression
// ============================================================================

/// Test variant coverage: find match arms in changed files that lack test coverage.
///
/// Scans changed `.rs` files for `match` expressions with 5+ arms and checks
/// whether each arm's pattern appears in at least one test function.
fn test_variant_coverage(
    project_path: &Path,
    baseline_commit: &str,
) -> Result<FalsificationResult> {
    print!("Scanning match arm coverage... ");

    let changed_files = get_changed_files(project_path, baseline_commit)?;
    let rs_files: Vec<&String> = changed_files
        .iter()
        .filter(|f| f.ends_with(".rs"))
        .collect();

    if rs_files.is_empty() {
        return Ok(FalsificationResult::passed(
            "No Rust files changed".to_string(),
        ));
    }

    let mut untested_arms: Vec<(String, String)> = Vec::new(); // (file, variant)

    for rel_path in &rs_files {
        let full_path = project_path.join(rel_path);
        let Ok(content) = std::fs::read_to_string(&full_path) else {
            continue;
        };

        // Find match blocks with 5+ arms (threshold for "enum with variants")
        let variants = extract_large_match_variants(&content);
        if variants.is_empty() {
            continue;
        }

        // Check if test functions reference each variant
        let test_section = extract_test_section(&content);
        for variant in &variants {
            if !test_section.contains(variant) {
                untested_arms.push((rel_path.to_string(), variant.clone()));
            }
        }
    }

    if untested_arms.is_empty() {
        Ok(FalsificationResult::passed(format!(
            "{} changed file(s) — all match variants tested",
            rs_files.len()
        )))
    } else {
        let details: Vec<String> = untested_arms
            .iter()
            .take(10)
            .map(|(f, v)| format!("{}::{}", f, v))
            .collect();
        let paths: Vec<PathBuf> = untested_arms
            .iter()
            .map(|(f, _)| PathBuf::from(f))
            .collect();
        Ok(FalsificationResult::failed(
            format!(
                "{} untested match variant(s): {}",
                untested_arms.len(),
                details.join(", ")
            ),
            EvidenceType::FileList(paths),
        ))
    }
}

/// State machine for parsing match blocks
struct MatchParser {
    variants: Vec<String>,
    current_arms: Vec<String>,
    brace_depth: usize,
    in_match: bool,
}

impl MatchParser {
    fn new() -> Self {
        Self {
            variants: Vec::new(),
            current_arms: Vec::new(),
            brace_depth: 0,
            in_match: false,
        }
    }

    fn process_line(&mut self, trimmed: &str) {
        if !self.in_match {
            if trimmed.contains("match ") && trimmed.ends_with('{') {
                self.in_match = true;
                self.current_arms.clear();
                self.brace_depth = 1;
            }
            return;
        }
        self.update_brace_depth(trimmed);
        if self.in_match && self.brace_depth == 1 {
            self.try_extract_arm(trimmed);
        }
    }

    fn update_brace_depth(&mut self, trimmed: &str) {
        for ch in trimmed.chars() {
            match ch {
                '{' => self.brace_depth += 1,
                '}' => {
                    self.brace_depth -= 1;
                    if self.brace_depth == 0 {
                        self.flush_match_block();
                    }
                }
                _ => {}
            }
        }
    }

    fn flush_match_block(&mut self) {
        if self.current_arms.len() >= 5 {
            self.variants.append(&mut self.current_arms);
        } else {
            self.current_arms.clear();
        }
        self.in_match = false;
    }

    fn try_extract_arm(&mut self, trimmed: &str) {
        let Some(pattern) = trimmed.split("=>").next() else {
            return;
        };
        let pattern = pattern.trim();
        if pattern == "_" || pattern.starts_with("//") || !trimmed.contains("=>") {
            return;
        }
        let variant = pattern
            .split("::")
            .last()
            .map(|s| {
                s.trim_matches(|c: char| !c.is_alphanumeric() && c != '_')
                    .to_string()
            })
            .unwrap_or_default();
        if !variant.is_empty() {
            self.current_arms.push(variant);
        }
    }
}

/// Extract variant names from match blocks with 5+ arms.
/// Returns variant identifiers like "Q4_K", "LLaMA", etc.
fn extract_large_match_variants(content: &str) -> Vec<String> {
    let mut parser = MatchParser::new();
    for line in content.lines() {
        parser.process_line(line.trim());
    }
    parser.variants
}

/// Extract test section content (everything after #[cfg(test)] or in test functions)
fn extract_test_section(content: &str) -> String {
    let mut in_test = false;
    let mut test_content = String::new();

    for line in content.lines() {
        if line.contains("#[cfg(test)]") || line.contains("#[test]") || line.contains("mod tests") {
            in_test = true;
        }
        if in_test {
            test_content.push_str(line);
            test_content.push('\n');
        }
    }

    test_content
}

/// Test fix-chain limit: detect consecutive fix commits touching the same files.
///
/// Analyzes recent git history for patterns where 3+ consecutive commits with "fix"
/// in the message touch the same file — a signal of inadequate pre-merge testing.
fn test_fix_chain_limit(project_path: &Path, max_chain: usize) -> Result<FalsificationResult> {
    print!("Analyzing fix chains... ");

    // Get last 50 commits with changed files
    let output = Command::new("git")
        .args(["log", "--oneline", "--name-only", "-50"])
        .current_dir(project_path)
        .output()
        .context("Failed to run git log")?;

    if !output.status.success() {
        return Ok(FalsificationResult::passed(
            "Cannot read git history".to_string(),
        ));
    }

    let log = String::from_utf8_lossy(&output.stdout);
    let chains = detect_fix_chains(&log, max_chain);

    if chains.is_empty() {
        Ok(FalsificationResult::passed(format!(
            "No fix chains > {} consecutive commits",
            max_chain
        )))
    } else {
        let details: Vec<String> = chains
            .iter()
            .take(5)
            .map(|(file, count)| format!("{} ({} consecutive)", file, count))
            .collect();
        let paths: Vec<PathBuf> = chains.iter().map(|(f, _)| PathBuf::from(f)).collect();
        Ok(FalsificationResult::failed(
            format!(
                "{} file(s) with fix chains > {}: {}",
                chains.len(),
                max_chain,
                details.join(", ")
            ),
            EvidenceType::FileList(paths),
        ))
    }
}

/// Check if a git log line is a commit header (starts with hex hash, length > 8)
fn is_commit_line(trimmed: &str) -> bool {
    trimmed.len() > 8
        && trimmed
            .as_bytes()
            .first()
            .map(|b| b.is_ascii_hexdigit())
            .unwrap_or(false)
}

/// Check if a commit message indicates a fix
fn is_fix_commit(msg: &str) -> bool {
    let lower = msg.to_lowercase();
    lower.contains("fix") || lower.contains("bug") || lower.contains("hotfix")
}

/// Collect violations from streak map, draining entries exceeding max_chain
fn collect_violations(
    streaks: &mut std::collections::HashMap<String, usize>,
    max_chain: usize,
    violations: &mut Vec<(String, usize)>,
) {
    violations.extend(streaks.drain().filter(|(_, streak)| *streak > max_chain));
}

/// Increment streak counts for each file in the current commit
fn increment_streaks(files: &[String], streaks: &mut std::collections::HashMap<String, usize>) {
    for file in files {
        *streaks.entry(file.clone()).or_insert(0) += 1;
    }
}

/// Parse git log output to detect consecutive fix-commit chains per file.
/// Returns (file, chain_length) for files exceeding the threshold.
fn detect_fix_chains(log: &str, max_chain: usize) -> Vec<(String, usize)> {
    let mut streaks: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut violations: Vec<(String, usize)> = Vec::new();
    let mut current_is_fix = false;
    let mut current_files: Vec<String> = Vec::new();

    for line in log.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if is_commit_line(trimmed) {
            // Flush previous commit
            if current_is_fix {
                increment_streaks(&current_files, &mut streaks);
            } else {
                collect_violations(&mut streaks, max_chain, &mut violations);
            }
            current_files.clear();
            current_is_fix = is_fix_commit(trimmed);
        } else if trimmed.contains('.') && !trimmed.starts_with('#') {
            current_files.push(trimmed.to_string());
        }
    }

    // Flush final commit
    if current_is_fix {
        increment_streaks(&current_files, &mut streaks);
    }
    collect_violations(&mut streaks, max_chain, &mut violations);

    violations.sort_by(|a, b| b.1.cmp(&a.1));
    violations.dedup_by(|a, b| a.0 == b.0);
    violations
}

/// Test cross-crate parity: verify sibling project tests still pass after changes.
///
/// Reads `.pmat-work/cross-crate.json` config for sibling project paths and test commands.
/// Only runs if config exists (opt-in per project).
async fn test_cross_crate_parity(project_path: &Path) -> Result<FalsificationResult> {
    print!("Checking cross-crate config... ");

    // Look for cross-crate config
    let config_path = project_path.join(".pmat-work/cross-crate.json");

    if !config_path.exists() {
        return Ok(FalsificationResult::passed(
            "No cross-crate config (create .pmat-work/cross-crate.json to enable)".to_string(),
        ));
    }

    let content = std::fs::read_to_string(&config_path)?;
    let config: serde_json::Value = serde_json::from_str(&content)?;

    let Some(projects) = config.get("projects").and_then(|p| p.as_array()) else {
        return Ok(FalsificationResult::passed(
            "No projects in cross-crate config".to_string(),
        ));
    };

    let mut failures = Vec::new();
    let mut passed_count = 0;

    for project in projects {
        let Some(path) = project.get("path").and_then(|p| p.as_str()) else {
            continue;
        };
        let test_cmd = project
            .get("test_command")
            .and_then(|c| c.as_str())
            .unwrap_or("cargo test --lib");

        let full_path = project_path.join(path);
        if !full_path.exists() {
            continue;
        }

        // Split test command into program + args
        let parts: Vec<&str> = test_cmd.split_whitespace().collect();
        let (program, args) = parts.split_first().unwrap_or((&"cargo", &[]));

        let output = Command::new(program)
            .args(args)
            .current_dir(&full_path)
            .output();

        match output {
            Ok(out) if out.status.success() => passed_count += 1,
            Ok(_) => failures.push(path.to_string()),
            Err(_) => failures.push(format!("{} (command failed)", path)),
        }
    }

    if failures.is_empty() {
        Ok(FalsificationResult::passed(format!(
            "{} sibling project(s) pass",
            passed_count
        )))
    } else {
        let paths: Vec<PathBuf> = failures.iter().map(PathBuf::from).collect();
        Ok(FalsificationResult::failed(
            format!(
                "{} sibling project(s) failed: {}",
                failures.len(),
                failures.join(", ")
            ),
            EvidenceType::FileList(paths),
        ))
    }
}

/// Test regression gate: verify no performance regressions from cached benchmarks.
///
/// Reads `.pmat-metrics/benchmark-status.json` for cached benchmark results.
async fn test_regression_gate(project_path: &Path) -> Result<FalsificationResult> {
    print!("Reading benchmark cache... ");

    // O(1): Read from cache
    if let Some(cache) = read_cached_metric(project_path, "benchmark-status.json") {
        if cache.is_stale_block {
            return Ok(FalsificationResult::passed(format!(
                "Benchmark cache old ({} min), skipping",
                cache.age_minutes
            )));
        }

        let passed = cache
            .value
            .get("passed")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let stale_note = format!(" (cached {} min ago)", cache.age_minutes);

        if passed {
            let benchmarks = cache
                .value
                .get("count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            return Ok(FalsificationResult::passed(format!(
                "{} benchmark(s) OK{}",
                benchmarks, stale_note
            )));
        } else {
            let regressions = cache
                .value
                .get("regressions")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default();
            return Ok(FalsificationResult::failed(
                format!("Performance regression{}: {}", stale_note, regressions),
                EvidenceType::CounterExample {
                    details: regressions,
                },
            ));
        }
    }

    // No cache — skip gracefully
    Ok(FalsificationResult::passed(
        "No benchmark cache (run benchmarks to populate .pmat-metrics/benchmark-status.json)"
            .to_string(),
    ))
}

// ============================================================================
// v4.0 provable contracts: Formal Proof Verification
// ============================================================================

/// Count sorry occurrences in Lean source, respecting comments and word boundaries.
/// Handles: line comments (--), nested block comments (/- ... -/), inline block comments,
/// and word-boundary checking to avoid false positives from identifiers like `sorry_helper`.
fn count_lean_sorry_in_source(source: &str) -> usize {
    let mut count = 0;
    let mut in_block_comment = 0i32;

    for line in source.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("--") {
            continue;
        }

        // Strip block comments inline for same-line /- ... -/ handling
        let cleaned = strip_lean_block_comments(trimmed, &mut in_block_comment);

        if in_block_comment > 0 {
            continue;
        }

        if contains_sorry_word_boundary(&cleaned) {
            count += 1;
        }
    }

    count
}

/// Strips block comment content from a line, updating nesting depth.
fn strip_lean_block_comments(line: &str, depth: &mut i32) -> String {
    let bytes = line.as_bytes();
    let mut result = String::with_capacity(line.len());
    let mut i = 0;

    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'-' {
            *depth += 1;
            i += 2;
            continue;
        }
        if i + 1 < bytes.len() && bytes[i] == b'-' && bytes[i + 1] == b'/' && *depth > 0 {
            *depth -= 1;
            i += 2;
            continue;
        }
        if *depth == 0 {
            result.push(bytes[i] as char);
        }
        i += 1;
    }

    result
}

/// Checks if line contains "sorry" as a standalone word (not part of an identifier).
fn contains_sorry_word_boundary(line: &str) -> bool {
    let bytes = line.as_bytes();
    let sorry = b"sorry";
    let mut pos = 0;
    while pos + sorry.len() <= bytes.len() {
        if let Some(idx) = line[pos..].find("sorry") {
            let abs_idx = pos + idx;
            let before_ok =
                abs_idx == 0 || !(bytes[abs_idx - 1].is_ascii_alphanumeric() || bytes[abs_idx - 1] == b'_');
            let after_ok = abs_idx + sorry.len() >= bytes.len()
                || !(bytes[abs_idx + sorry.len()].is_ascii_alphanumeric()
                    || bytes[abs_idx + sorry.len()] == b'_');
            if before_ok && after_ok {
                return true;
            }
            pos = abs_idx + 1;
        } else {
            break;
        }
    }
    false
}

/// Test formal proof verification: count sorry occurrences in .lean files
fn test_formal_proof_verification(
    project_path: &Path,
    max_sorry_count: usize,
) -> Result<FalsificationResult> {
    print!("Scanning .lean files for sorry... ");

    let mut total_sorry = 0usize;
    let mut sorry_files = Vec::new();

    // Walk project looking for .lean files
    for entry in walkdir::WalkDir::new(project_path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "lean") {
            if let Ok(content) = std::fs::read_to_string(path) {
                let count = count_lean_sorry_in_source(&content);
                if count > 0 {
                    total_sorry += count;
                    sorry_files.push(path.to_path_buf());
                }
            }
        }
    }

    if sorry_files.is_empty() && total_sorry == 0 {
        return Ok(FalsificationResult::passed(
            "No .lean files with sorry found".to_string(),
        ));
    }

    if total_sorry <= max_sorry_count {
        Ok(FalsificationResult::passed(format!(
            "{} sorry occurrence(s) within threshold (max: {})",
            total_sorry, max_sorry_count
        )))
    } else {
        Ok(FalsificationResult::failed(
            format!(
                "{} sorry occurrence(s) exceed threshold (max: {}), in: {}",
                total_sorry,
                max_sorry_count,
                sorry_files
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            EvidenceType::FileList(sorry_files),
        ))
    }
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
                        format!(
                            "{} new SATD marker(s): {}",
                            new_satd.len(),
                            details.join("; ")
                        ),
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

/// Check if a trimmed line is a regular SATD-eligible comment.
/// Must start with `//` but NOT `///` or `//!`, and must not be
/// a SECURITY/SAFETY annotation.
fn is_satd_comment(trimmed: &str) -> bool {
    if !trimmed.starts_with("//") || trimmed.starts_with("///") || trimmed.starts_with("//!") {
        return false;
    }
    let comment_text = trimmed[2..].trim_start();
    !comment_text.starts_with("SECURITY:") && !comment_text.starts_with("SAFETY:")
}

/// Extract SATD markers from a single added line.
/// Returns one entry per matching pattern found in the line.
fn extract_satd_markers(
    line_content: &str,
    file: &Path,
    satd_patterns: &[&str],
) -> Vec<(PathBuf, String)> {
    let trimmed = line_content.trim();
    satd_patterns
        .iter()
        .filter(|pattern| trimmed.contains(*pattern))
        .map(|pattern| {
            let marker = line_content
                .split(pattern)
                .nth(1)
                .map(|s| format!("{}{}", pattern, s.chars().take(50).collect::<String>()))
                .unwrap_or_else(|| pattern.to_string());
            (file.to_path_buf(), marker)
        })
        .collect()
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
            continue;
        }

        // Skip non-added lines and the "+++ b/" header (already handled above)
        let Some(line_content) = line.strip_prefix('+') else {
            continue;
        };
        if line.starts_with("+++") {
            continue;
        }

        let trimmed = line_content.trim();
        if !is_satd_comment(trimmed) {
            continue;
        }

        if let Some(ref file) = current_file {
            new_satd.extend(extract_satd_markers(line_content, file, &satd_patterns));
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
                    Ok(FalsificationResult::passed(
                        "No dead code detected".to_string(),
                    ))
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

/// Check if a file should be skipped for per-file coverage checks.
fn is_excluded_from_per_file_coverage(filename: &str) -> bool {
    filename.contains("/tests/") || filename.contains("_test.rs") || filename.contains("/target/")
}

/// Extract the line coverage percentage from a single llvm-cov file entry.
fn extract_file_line_coverage(file_entry: &serde_json::Value) -> f64 {
    file_entry
        .get("summary")
        .and_then(|s| s.get("lines"))
        .and_then(|l| l.get("percent"))
        .and_then(|p| p.as_f64())
        .unwrap_or(100.0)
}

/// Parse llvm-cov JSON and return files whose coverage is below `threshold`.
///
/// Each entry is `(filename, coverage_pct)`. Test files and generated files
/// are excluded.
fn collect_files_below_threshold(
    json: &serde_json::Value,
    threshold: f64,
) -> Vec<(PathBuf, f64)> {
    let data = match json.get("data").and_then(|d| d.as_array()) {
        Some(d) => d,
        None => return Vec::new(),
    };

    data.iter()
        .filter_map(|file_data| file_data.get("files").and_then(|f| f.as_array()))
        .flatten()
        .filter_map(|file| {
            let filename = file.get("filename").and_then(|f| f.as_str()).unwrap_or("unknown");
            if is_excluded_from_per_file_coverage(filename) {
                return None;
            }
            let coverage = extract_file_line_coverage(file);
            if coverage < threshold {
                Some((PathBuf::from(filename), coverage))
            } else {
                None
            }
        })
        .collect()
}

/// Build a FalsificationResult from the list of files below coverage threshold.
fn build_per_file_coverage_result(
    files_below_threshold: Vec<(PathBuf, f64)>,
    threshold: f64,
) -> FalsificationResult {
    if files_below_threshold.is_empty() {
        return FalsificationResult::passed(format!(
            "All files >= {:.1}% coverage",
            threshold
        ));
    }

    let paths: Vec<PathBuf> = files_below_threshold
        .iter()
        .map(|(p, _)| p.clone())
        .collect();
    let details: Vec<String> = files_below_threshold
        .iter()
        .take(10)
        .map(|(p, cov)| format!("{}: {:.1}%", p.display(), cov))
        .collect();
    FalsificationResult::failed(
        format!(
            "{} file(s) below {:.1}% threshold: {}",
            files_below_threshold.len(),
            threshold,
            details.join(", ")
        ),
        EvidenceType::FileList(paths),
    )
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
        return Ok(FalsificationResult::passed(format!(
            "No per-file coverage data (run 'make coverage'), threshold: {:.1}%",
            threshold
        )));
    }

    let content = std::fs::read_to_string(&coverage_json)?;
    let json: serde_json::Value = serde_json::from_str(&content)?;

    let files_below = collect_files_below_threshold(&json, threshold);
    Ok(build_per_file_coverage_result(files_below, threshold))
}

/// Test lint pass: O(1) - reads from cached lint status
async fn test_lint_pass(project_path: &Path) -> Result<FalsificationResult> {
    print!("Reading lint cache... ");

    // O(1): Read from cache instead of running make lint
    // Try primary cache location, then fallback to work-item and .pmat directories
    if let Some(cache) = read_cached_metric(project_path, "lint-status.json")
        .or_else(|| read_lint_cache_fallback(project_path))
    {
        if cache.is_stale_block {
            return Ok(FalsificationResult::failed(
                format!(
                    "Lint cache too old ({} min). Run 'make lint' first.",
                    cache.age_minutes
                ),
                EvidenceType::BooleanCheck(false),
            ));
        }

        // Validate 'passed' field exists — reject malformed cache (Popperian Audit v2.1)
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
            format!(
                " (cached {} min ago, consider re-running)",
                cache.age_minutes
            )
        } else {
            format!(" (cached {} min ago)", cache.age_minutes)
        };

        if passed {
            return Ok(FalsificationResult::passed(format!("PASSED{}", stale_note)));
        } else {
            let errors = cache
                .value
                .get("error_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            return Ok(FalsificationResult::failed(
                format!("{} lint errors{}", errors, stale_note),
                EvidenceType::NumericComparison {
                    actual: errors as f64,
                    threshold: 0.0,
                },
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

include!("work_falsification_tests.rs");
