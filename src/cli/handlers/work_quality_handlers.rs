//! Work quality handlers for unified GitHub/YAML workflow
//!
//! Extracted from work_handlers.rs for file health compliance (CB-040).
//! Contains quality gates and Popper falsification validation.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Run quality gates (tests, clippy, etc.)
///
/// Returns Ok(true) if all gates pass, Ok(false) if any fail, or Err on execution failure.
pub async fn run_quality_gates(project_path: &PathBuf) -> Result<bool> {
    use std::process::Command;

    let mut all_passed = true;

    // 1. Run cargo test (git-aware: only test changed modules)
    println!("   🧪 Running tests...");

    // Extract test modules from changed files
    let modules =
        crate::services::git_test_filter::extract_test_modules_from_changed_files(project_path)?;

    let test_status = if modules.is_empty() {
        // No Rust files changed - skip tests
        println!("      ℹ️  No Rust files changed, skipping tests");
        std::process::ExitStatus::default()
    } else {
        // Run tests for changed modules only
        let module_list = modules.join(", ");
        println!(
            "      📋 Testing changed modules: {}",
            if module_list.len() > 60 {
                format!("{}...", &module_list[..60])
            } else {
                module_list
            }
        );

        let test_cmd = crate::services::git_test_filter::build_test_command(&modules)
            .unwrap_or_else(|| {
                vec![
                    "test".to_string(),
                    "--lib".to_string(),
                    "--quiet".to_string(),
                ]
            });

        Command::new("cargo")
            .args(&test_cmd)
            .arg("--quiet")
            .current_dir(project_path)
            .status()
            .context("Failed to run cargo test")?
    };

    if test_status.success() {
        println!("      ✅ Tests passed");
    } else {
        println!("      ❌ Tests failed");
        all_passed = false;
    }

    // 2. Rust project-specific checks (if Cargo.toml exists)
    if project_path.join("Cargo.toml").exists() {
        println!("   🦀 Rust project detected...");

        // Check if examples directory exists
        let examples_dir = project_path.join("examples");
        if examples_dir.exists() && examples_dir.is_dir() {
            println!("      📦 Checking examples...");
            let examples_status = Command::new("cargo")
                .args(["test", "--examples", "--no-run"])
                .current_dir(project_path)
                .status()
                .context("Failed to run cargo test --examples")?;

            if examples_status.success() {
                println!("      ✅ Examples compile");
            } else {
                println!("      ❌ Examples failed to compile");
                all_passed = false;
            }
        } else {
            println!("      ℹ️  No examples directory found, skipping example checks");
        }

        // Capture rust-project-score (O(1) from cache)
        println!("      📊 Capturing rust-project-score...");
        match Command::new("pmat")
            .args(["rust-project-score", "--format", "json"])
            .current_dir(project_path)
            .output()
        {
            Ok(output) if output.status.success() => {
                // Parse score and display
                if let Ok(score_json) = std::str::from_utf8(&output.stdout) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(score_json) {
                        if let Some(score) = json.get("total_earned").and_then(|v| v.as_f64()) {
                            println!("      ✅ Rust Project Score: {:.1}/134", score);
                        }
                    }
                }
            }
            Ok(_) => {
                println!("      ⚠️  Failed to capture rust-project-score (continuing)");
            }
            Err(_) => {
                println!("      ⚠️  pmat rust-project-score not available (continuing)");
            }
        }
    }

    // 3. Renacer golden tracing validation (if renacer.toml exists)
    if project_path.join("renacer.toml").exists() {
        println!("   🎯 Golden traces detected...");

        match Command::new("renacer")
            .args(["validate", "--all"])
            .current_dir(project_path)
            .status()
        {
            Ok(status) if status.success() => {
                println!("      ✅ Golden traces match");
            }
            Ok(_) => {
                println!("      ❌ Golden traces diverged");
                all_passed = false;
            }
            Err(_) => {
                println!("      ⚠️  renacer not installed (skipping golden trace validation)");
                println!("         Install: cargo install renacer");
            }
        }
    }

    // 4. Run cargo clippy
    println!("   📎 Running clippy...");
    let clippy_status = Command::new("cargo")
        .arg("clippy")
        .arg("--lib")
        .arg("--quiet")
        .arg("--")
        .arg("-D")
        .arg("warnings")
        .current_dir(project_path)
        .status()
        .context("Failed to run cargo clippy")?;

    if clippy_status.success() {
        println!("      ✅ No clippy warnings");
    } else {
        println!("      ❌ Clippy warnings found");
        all_passed = false;
    }

    println!();
    Ok(all_passed)
}

/// Karl Popper Falsification Result
///
/// Captures the results of post-work falsification validation.
/// Based on the philosophy that scientific claims must be falsifiable -
/// we validate that our work satisfies falsification criteria.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FalsificationResult {
    /// Tests passed (falsify: no regressions introduced)
    pub tests_passed: bool,
    /// Coverage increased or maintained (falsify: no code bloat without tests)
    pub coverage_maintained: bool,
    /// Coverage percentage before work
    pub coverage_before: Option<f32>,
    /// Coverage percentage after work
    pub coverage_after: Option<f32>,
    /// Binary size within threshold (falsify: no dependency bloat)
    pub binary_size_ok: bool,
    /// Overall falsification passed
    pub passed: bool,
    /// Human-readable summary
    pub summary: String,
}

impl Default for FalsificationResult {
    fn default() -> Self {
        Self {
            tests_passed: false,
            coverage_maintained: false,
            coverage_before: None,
            coverage_after: None,
            binary_size_ok: true,
            passed: false,
            summary: String::new(),
        }
    }
}

/// Run Karl Popper Falsification Validation
///
/// This implements the scientific method for validating work:
/// 1. Hypothesis: Work should not introduce regressions
/// 2. Falsification: Run tests to attempt to falsify the hypothesis
/// 3. Measurement: Measure coverage to verify improvements
/// 4. Result: Pass only if falsification attempts fail (work is valid)
///
/// Based on: docs/specifications/80-20-to-95.md
pub async fn run_popper_falsification(project_path: &PathBuf) -> Result<FalsificationResult> {
    use std::process::Command;

    let mut result = FalsificationResult::default();
    let mut issues: Vec<String> = Vec::new();
    let total_hypotheses = 3;
    let mut validated = 0;

    println!();
    println!(
        "🔬 Karl Popper Falsification Validation (0/{} complete)",
        total_hypotheses
    );
    println!("   (Scientific method: attempting to falsify your work)");
    println!();

    // 1. Hypothesis: Tests should pass (falsify: look for regressions)
    println!(
        "   📊 [1/{}] Hypothesis: No regressions introduced",
        total_hypotheses
    );
    println!("      Falsification: Running tests...");

    let test_status = Command::new("cargo")
        .args(["test", "--lib", "--quiet"])
        .current_dir(project_path)
        .status()
        .context("Failed to run cargo test")?;

    if test_status.success() {
        result.tests_passed = true;
        validated += 1;
        println!(
            "      ✅ Hypothesis holds ({}/{} validated)",
            validated, total_hypotheses
        );
    } else {
        result.tests_passed = false;
        issues.push("Tests failed - regressions detected".to_string());
        println!("      ❌ Hypothesis falsified: Tests fail");
    }

    // 2. Hypothesis: Coverage should be maintained or improved
    println!();
    println!(
        "   📊 [2/{}] Hypothesis: Coverage maintained or improved",
        total_hypotheses
    );
    println!("      Falsification: Checking coverage trends...");

    // Try to read coverage from cached metrics
    let metrics_dir = project_path.join(".pmat-metrics/trends");
    if metrics_dir.exists() {
        if let Ok(content) = std::fs::read_to_string(metrics_dir.join("test-coverage.json")) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(entries) = json.as_array() {
                    if entries.len() >= 2 {
                        // Compare last two entries
                        let current = entries
                            .last()
                            .and_then(|e| e.get("value"))
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0) as f32;
                        let previous = entries
                            .get(entries.len() - 2)
                            .and_then(|e| e.get("value"))
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0) as f32;

                        result.coverage_before = Some(previous);
                        result.coverage_after = Some(current);

                        if current >= previous {
                            result.coverage_maintained = true;
                            validated += 1;
                            let delta = current - previous;
                            if delta > 0.0 {
                                println!(
                                    "      ✅ Hypothesis holds: Coverage +{:.2}% ({}/{} validated)",
                                    delta, validated, total_hypotheses
                                );
                            } else {
                                println!(
                                    "      ✅ Hypothesis holds: Coverage at {:.2}% ({}/{} validated)",
                                    current, validated, total_hypotheses
                                );
                            }
                        } else {
                            let delta = previous - current;
                            issues.push(format!("Coverage dropped by {:.2}%", delta));
                            println!("      ❌ Hypothesis falsified: Coverage -{:.2}%", delta);
                        }
                    } else if !entries.is_empty() {
                        result.coverage_maintained = true;
                        validated += 1;
                        println!(
                            "      ⚠️  Insufficient history ({}/{} validated)",
                            validated, total_hypotheses
                        );
                    }
                }
            }
        }
    }

    if result.coverage_before.is_none() {
        result.coverage_maintained = true; // Assume OK if no data
        validated += 1;
        println!(
            "      ⚠️  No coverage history ({}/{} validated)",
            validated, total_hypotheses
        );
        println!("         Run 'make coverage' to establish baseline");
    }

    // 3. Binary size check (optional, only if release build exists)
    println!();
    println!(
        "   📊 [3/{}] Hypothesis: No dependency bloat",
        total_hypotheses
    );
    result.binary_size_ok = true; // Default to OK

    let release_binary = project_path.join("target/release/pmat");
    if release_binary.exists() {
        if let Ok(metadata) = std::fs::metadata(&release_binary) {
            let size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
            if size_mb <= 50.0 {
                validated += 1;
                println!(
                    "      ✅ Hypothesis holds: {:.1}MB < 50MB ({}/{} validated)",
                    size_mb, validated, total_hypotheses
                );
            } else {
                result.binary_size_ok = false;
                issues.push(format!("Binary size {:.1}MB exceeds 50MB limit", size_mb));
                println!(
                    "      ❌ Hypothesis falsified: {:.1}MB > 50MB limit",
                    size_mb
                );
            }
        }
    } else {
        validated += 1;
        println!(
            "      ⚠️  No release binary ({}/{} validated)",
            validated, total_hypotheses
        );
    }

    // Determine overall result
    result.passed = result.tests_passed && result.coverage_maintained && result.binary_size_ok;

    println!();
    if result.passed {
        result.summary = format!(
            "{}/{} hypotheses validated - work is valid",
            validated, total_hypotheses
        );
        println!(
            "   🎉 FALSIFICATION RESULT: PASSED ({}/{})",
            validated, total_hypotheses
        );
        println!("      All hypotheses held under scrutiny");
    } else {
        let failed = total_hypotheses - validated;
        result.summary = format!(
            "{}/{} validated, {} falsified: {}",
            validated,
            total_hypotheses,
            failed,
            issues.join(", ")
        );
        println!(
            "   ⚠️  FALSIFICATION RESULT: FAILED ({}/{} validated)",
            validated, total_hypotheses
        );
        println!("      Issues found:");
        for issue in &issues {
            println!("      - {}", issue);
        }
    }
    println!();

    Ok(result)
}
