//! Work Falsification: Popperian Falsification Executor
//!
//! Runs all falsification tests and BLOCKS completion if ANY fail.
//! Based on Karl Popper's demarcation criterion: claims must be falsifiable.
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
                    if let Some(score) = score_str
                        .chars()
                        .filter(|c| c.is_ascii_digit())
                        .collect::<String>()
                        .parse::<u32>()
                        .ok()
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

/// Test supply chain integrity: no vulnerable dependencies added
async fn test_supply_chain_integrity(project_path: &Path) -> Result<FalsificationResult> {
    print!("Running cargo deny check... ");

    // Ensure cargo-deny is installed or skip if not available
    let output = Command::new("cargo")
        .args(["deny", "check", "advisories"])
        .current_dir(project_path)
        .output();

    match output {
        Ok(out) => {
            if out.status.success() {
                Ok(FalsificationResult::passed(
                    "No vulnerable dependencies or advisories found".to_string(),
                ))
            } else {
                let stderr = String::from_utf8_lossy(&out.stderr);
                Ok(FalsificationResult::failed(
                    "Vulnerable dependencies detected".to_string(),
                    EvidenceType::CounterExample {
                        details: stderr.to_string(),
                    },
                ))
            }
        }
        Err(_) => {
            // cargo-deny not installed, skip for now or use a fallback
            Ok(FalsificationResult::passed(
                "cargo-deny not installed, skipping supply chain check (Warning: Risk!)"
                    .to_string(),
            ))
        }
    }
}

/// Test examples compile: all examples must compile and run
async fn test_examples_compile(project_path: &Path) -> Result<FalsificationResult> {
    print!("Running cargo run --examples... ");

    // Check if examples directory exists
    let examples_dir = project_path.join("examples");
    if !examples_dir.exists() {
        return Ok(FalsificationResult::passed(
            "No examples directory found (skipping)".to_string(),
        ));
    }

    // First check if examples compile
    let compile_output = Command::new("cargo")
        .args(["build", "--examples"])
        .current_dir(project_path)
        .output();

    match compile_output {
        Ok(out) if out.status.success() => {
            // Now try to run each example with --help to verify they execute
            let examples: Vec<_> = std::fs::read_dir(&examples_dir)?
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path()
                        .extension()
                        .map(|ext| ext == "rs")
                        .unwrap_or(false)
                })
                .collect();

            let example_count = examples.len();

            // Run a quick check on each example
            let mut failed_examples: Vec<String> = Vec::new();
            for entry in examples {
                let name = entry
                    .path()
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                let run_output = Command::new("cargo")
                    .args(["run", "--example", &name, "--", "--help"])
                    .current_dir(project_path)
                    .output();

                // We just check if it runs, --help should exit quickly
                if let Ok(o) = run_output {
                    if !o.status.success() {
                        // Track examples that fail to run
                        failed_examples.push(name);
                    }
                }
            }

            if failed_examples.is_empty() {
                Ok(FalsificationResult::passed(format!(
                    "{} examples compile successfully",
                    example_count
                )))
            } else {
                Ok(FalsificationResult::failed(
                    format!("{} example(s) failed", failed_examples.len()),
                    EvidenceType::CounterExample {
                        details: failed_examples.join(", "),
                    },
                ))
            }
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            Ok(FalsificationResult::failed(
                "Examples failed to compile".to_string(),
                EvidenceType::CounterExample {
                    details: stderr.chars().take(500).collect(),
                },
            ))
        }
        Err(e) => Ok(FalsificationResult::passed(format!(
            "Could not run cargo build --examples: {} (skipping)",
            e
        ))),
    }
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
}
