//! Work quality handlers for unified GitHub/YAML workflow
//!
//! Extracted from work_handlers.rs for file health compliance (CB-040).
//! Contains quality gates and Popper falsification validation.

#![cfg_attr(coverage_nightly, coverage(off))]

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Run git-aware tests for changed modules.
/// Returns true if tests passed or were skipped.
fn run_changed_module_tests(project_path: &PathBuf) -> Result<bool> {
    use std::process::Command;

    println!("   🧪 Running tests...");
    let modules =
        crate::services::git_test_filter::extract_test_modules_from_changed_files(project_path)?;

    if modules.is_empty() {
        println!("      ℹ️  No Rust files changed, skipping tests");
        return Ok(true);
    }

    let module_list = modules.join(", ");
    let display = if module_list.len() > 60 {
        format!("{}...", &module_list[..60])
    } else {
        module_list
    };
    println!("      📋 Testing changed modules: {}", display);

    let test_cmd = crate::services::git_test_filter::build_test_command(&modules)
        .unwrap_or_else(|| vec!["test".into(), "--lib".into(), "--quiet".into()]);

    let status = Command::new("cargo")
        .args(&test_cmd)
        .arg("--quiet")
        .current_dir(project_path)
        .status()
        .context("Failed to run cargo test")?;

    if status.success() {
        println!("      ✅ Tests passed");
        Ok(true)
    } else {
        println!("      ❌ Tests failed");
        Ok(false)
    }
}

/// Run Rust-specific checks: examples compilation and project score.
/// Returns true if all checks passed.
fn run_rust_project_checks(project_path: &PathBuf) -> Result<bool> {
    use std::process::Command;

    if !project_path.join("Cargo.toml").exists() {
        return Ok(true);
    }

    println!("   🦀 Rust project detected...");
    let mut passed = true;

    // Check examples
    let examples_dir = project_path.join("examples");
    if examples_dir.exists() && examples_dir.is_dir() {
        println!("      📦 Checking examples...");
        let status = Command::new("cargo")
            .args(["test", "--examples", "--no-run"])
            .current_dir(project_path)
            .status()
            .context("Failed to run cargo test --examples")?;

        if status.success() {
            println!("      ✅ Examples compile");
        } else {
            println!("      ❌ Examples failed to compile");
            passed = false;
        }
    }

    // Capture rust-project-score
    println!("      📊 Capturing rust-project-score...");
    if let Ok(output) = Command::new("pmat")
        .args(["rust-project-score", "--format", "json"])
        .current_dir(project_path)
        .output()
    {
        if output.status.success() {
            if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                if let Some(score) = json.get("total_earned").and_then(|v| v.as_f64()) {
                    println!("      ✅ Rust Project Score: {:.1}/134", score);
                }
            }
        } else {
            println!("      ⚠️  Failed to capture rust-project-score (continuing)");
        }
    }

    Ok(passed)
}

/// Validate golden traces via renacer if baseline exists.
/// Returns true if validation passed or was skipped.
fn run_golden_trace_validation(project_path: &PathBuf) -> Result<bool> {
    use std::process::Command;

    if !project_path.join("renacer.toml").exists() {
        return Ok(true);
    }

    let baseline_dir = project_path.join("golden_traces").join("baseline");
    if !baseline_dir.exists() {
        println!("   🎯 Golden traces config found, no baseline yet (run: renacer validate --generate golden_traces/baseline -- ./target/release/pmat --help)");
        return Ok(true);
    }

    println!("   🎯 Golden traces detected...");
    match Command::new("renacer")
        .args([
            "validate",
            "--baseline",
            baseline_dir.to_str().unwrap_or("golden_traces/baseline"),
            "--ignore-timing",
            "--",
            "./target/release/pmat",
            "--help",
        ])
        .current_dir(project_path)
        .status()
    {
        Ok(status) if status.success() => {
            println!("      ✅ Golden traces match");
            Ok(true)
        }
        Ok(status) if status.code() == Some(2) => {
            println!("      ℹ️  No golden baseline yet");
            Ok(true)
        }
        Ok(_) => {
            println!("      ❌ Golden traces diverged");
            Ok(false)
        }
        Err(_) => {
            println!("      ⚠️  renacer not installed (skipping golden trace validation)");
            Ok(true)
        }
    }
}

/// Run cargo clippy. Returns true if no warnings.
fn run_clippy_check(project_path: &PathBuf) -> Result<bool> {
    use std::process::Command;

    println!("   📎 Running clippy...");
    let status = Command::new("cargo")
        .args(["clippy", "--lib", "--quiet", "--", "-D", "warnings"])
        .current_dir(project_path)
        .status()
        .context("Failed to run cargo clippy")?;

    if status.success() {
        println!("      ✅ No clippy warnings");
        Ok(true)
    } else {
        println!("      ❌ Clippy warnings found");
        Ok(false)
    }
}

/// Run quality gates (tests, clippy, etc.)
///
/// Returns Ok(true) if all gates pass, Ok(false) if any fail, or Err on execution failure.
pub async fn run_quality_gates(project_path: &PathBuf) -> Result<bool> {
    let tests_ok = run_changed_module_tests(project_path)?;
    let rust_ok = run_rust_project_checks(project_path)?;
    let traces_ok = run_golden_trace_validation(project_path)?;
    let clippy_ok = run_clippy_check(project_path)?;

    // Refresh agent context index for future searches (non-blocking)
    refresh_agent_context_index(project_path);

    println!();
    Ok(tests_ok && rust_ok && traces_ok && clippy_ok)
}

/// Refresh agent context index after quality gates pass.
/// Non-blocking: failures are logged but don't block quality gates.
fn refresh_agent_context_index(project_path: &PathBuf) {
    use crate::services::agent_context::AgentContextIndex;

    let index_path = project_path.join(".pmat/context.idx");
    match AgentContextIndex::build(project_path) {
        Ok(index) => {
            if let Err(e) = index.save(&index_path) {
                eprintln!("   ⚠  Agent context index save failed: {}", e);
            } else {
                let m = index.manifest();
                println!(
                    "   📚 Agent context index refreshed: {} functions in {} files",
                    m.function_count, m.file_count
                );
            }
        }
        Err(e) => {
            eprintln!("   ⚠  Agent context index build failed: {}", e);
        }
    }
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

/// Check test regression hypothesis. Returns (passed, validated_count).
fn falsify_test_regression(project_path: &PathBuf, step: usize, total: usize) -> Result<(bool, Vec<String>)> {
    use std::process::Command;

    println!("   📊 [{}/{}] Hypothesis: No regressions introduced", step, total);
    println!("      Falsification: Running tests...");

    let status = Command::new("cargo")
        .args(["test", "--lib", "--quiet"])
        .current_dir(project_path)
        .status()
        .context("Failed to run cargo test")?;

    if status.success() {
        println!("      ✅ Hypothesis holds ({}/{} validated)", step, total);
        Ok((true, vec![]))
    } else {
        println!("      ❌ Hypothesis falsified: Tests fail");
        Ok((false, vec!["Tests failed - regressions detected".into()]))
    }
}

/// Check coverage maintenance hypothesis from cached metrics.
fn falsify_coverage_regression(project_path: &PathBuf, result: &mut FalsificationResult, step: usize, total: usize) -> (bool, Vec<String>) {
    println!();
    println!("   📊 [{}/{}] Hypothesis: Coverage maintained or improved", step, total);
    println!("      Falsification: Checking coverage trends...");

    let trend_file = project_path.join(".pmat-metrics/trends/test-coverage.json");
    let coverage = parse_coverage_trend(&trend_file);

    match coverage {
        Some((previous, current)) => {
            result.coverage_before = Some(previous);
            result.coverage_after = Some(current);
            if current >= previous {
                result.coverage_maintained = true;
                let delta = current - previous;
                let msg = if delta > 0.0 { format!("+{:.2}%", delta) } else { format!("at {:.2}%", current) };
                println!("      ✅ Hypothesis holds: Coverage {} ({}/{} validated)", msg, step, total);
                (true, vec![])
            } else {
                let delta = previous - current;
                println!("      ❌ Hypothesis falsified: Coverage -{:.2}%", delta);
                (false, vec![format!("Coverage dropped by {:.2}%", delta)])
            }
        }
        None => {
            result.coverage_maintained = true;
            println!("      ⚠️  No coverage history ({}/{} validated)", step, total);
            (true, vec![])
        }
    }
}

/// Parse coverage trend from JSON file. Returns (previous, current) if available.
fn parse_coverage_trend(path: &std::path::Path) -> Option<(f32, f32)> {
    let content = std::fs::read_to_string(path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    let entries = json.as_array()?;
    if entries.len() < 2 { return None; }

    let current = entries.last()?.get("value")?.as_f64()? as f32;
    let previous = entries.get(entries.len() - 2)?.get("value")?.as_f64()? as f32;
    Some((previous, current))
}

/// Check binary size hypothesis.
fn falsify_binary_bloat(project_path: &PathBuf, step: usize, total: usize) -> (bool, Vec<String>) {
    println!();
    println!("   📊 [{}/{}] Hypothesis: No dependency bloat", step, total);

    let release_binary = project_path.join("target/release/pmat");
    if !release_binary.exists() {
        println!("      ⚠️  No release binary ({}/{} validated)", step, total);
        return (true, vec![]);
    }

    if let Ok(metadata) = std::fs::metadata(&release_binary) {
        let size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
        if size_mb <= 50.0 {
            println!("      ✅ Hypothesis holds: {:.1}MB < 50MB ({}/{} validated)", size_mb, step, total);
            (true, vec![])
        } else {
            println!("      ❌ Hypothesis falsified: {:.1}MB > 50MB limit", size_mb);
            (false, vec![format!("Binary size {:.1}MB exceeds 50MB limit", size_mb)])
        }
    } else {
        (true, vec![])
    }
}

/// Run Karl Popper Falsification Validation
///
/// Scientific method: attempt to falsify work claims.
/// Pass only if all falsification attempts fail (work is valid).
pub async fn run_popper_falsification(project_path: &PathBuf) -> Result<FalsificationResult> {
    let mut result = FalsificationResult::default();
    let total = 3;

    println!();
    println!("🔬 Karl Popper Falsification Validation (0/{} complete)", total);
    println!("   (Scientific method: attempting to falsify your work)");
    println!();

    let (tests_ok, test_issues) = falsify_test_regression(project_path, 1, total)?;
    result.tests_passed = tests_ok;

    let (cov_ok, cov_issues) = falsify_coverage_regression(project_path, &mut result, 2, total);

    let (size_ok, size_issues) = falsify_binary_bloat(project_path, 3, total);
    result.binary_size_ok = size_ok;

    result.passed = tests_ok && cov_ok && size_ok;
    let validated = [tests_ok, cov_ok, size_ok].iter().filter(|v| **v).count();
    let all_issues: Vec<String> = [test_issues, cov_issues, size_issues].concat();

    println!();
    if result.passed {
        result.summary = format!("{}/{} hypotheses validated - work is valid", validated, total);
        println!("   🎉 FALSIFICATION RESULT: PASSED ({}/{})", validated, total);
    } else {
        result.summary = format!("{}/{} validated, {} falsified: {}", validated, total, total - validated, all_issues.join(", "));
        println!("   ⚠️  FALSIFICATION RESULT: FAILED ({}/{} validated)", validated, total);
        for issue in &all_issues {
            println!("      - {}", issue);
        }
    }
    println!();

    Ok(result)
}
