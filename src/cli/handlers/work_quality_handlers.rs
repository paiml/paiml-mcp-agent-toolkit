//! Work quality handlers for unified GitHub/YAML workflow
//!
//! Extracted from work_handlers.rs for file health compliance (CB-040).
//! Contains quality gates and Popper falsification validation.

#![cfg_attr(coverage_nightly, coverage(off))]

use crate::cli::colors as c;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Run git-aware tests for changed modules.
/// Returns true if tests passed or were skipped.
fn run_changed_module_tests(project_path: &PathBuf) -> Result<bool> {
    use std::process::Command;

    println!("   {}", c::dim("Running tests..."));
    let modules =
        crate::services::git_test_filter::extract_test_modules_from_changed_files(project_path)?;

    if modules.is_empty() {
        println!("      {}", c::skip("No Rust files changed, skipping tests"));
        return Ok(true);
    }

    let module_list = modules.join(", ");
    let display = if module_list.len() > 60 {
        format!("{}...", module_list.get(..60).unwrap_or(&module_list))
    } else {
        module_list
    };
    println!(
        "      {} {}",
        c::label("Testing changed modules:"),
        c::path(&display)
    );

    let test_cmd = crate::services::git_test_filter::build_test_command(&modules)
        .unwrap_or_else(|| vec!["test".into(), "--lib".into(), "--quiet".into()]);

    let status = Command::new("cargo")
        .args(&test_cmd)
        .arg("--quiet")
        .current_dir(project_path)
        .status()
        .context("Failed to run cargo test")?;

    if status.success() {
        println!("      {}", c::pass("Tests passed"));
        Ok(true)
    } else {
        println!("      {}", c::fail("Tests failed"));
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

    println!("   {}", c::dim("Rust project detected..."));
    let mut passed = true;

    // Check examples
    let examples_dir = project_path.join("examples");
    if examples_dir.exists() && examples_dir.is_dir() {
        println!("      {}", c::dim("Checking examples..."));
        let status = Command::new("cargo")
            .args(["test", "--examples", "--no-run"])
            .current_dir(project_path)
            .status()
            .context("Failed to run cargo test --examples")?;

        if status.success() {
            println!("      {}", c::pass("Examples compile"));
        } else {
            println!("      {}", c::fail("Examples failed to compile"));
            passed = false;
        }
    }

    // Capture rust-project-score
    println!("      {}", c::dim("Capturing rust-project-score..."));
    if let Ok(output) = Command::new("pmat")
        .args(["rust-project-score", "--format", "json"])
        .current_dir(project_path)
        .output()
    {
        if output.status.success() {
            if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                if let Some(score) = json.get("total_earned").and_then(|v| v.as_f64()) {
                    println!(
                        "      {}",
                        c::pass(&format!(
                            "Rust Project Score: {}",
                            c::score(score, 134.0, 80.0, 60.0)
                        ))
                    );
                }
            }
        } else {
            println!(
                "      {}",
                c::warn("Failed to capture rust-project-score (continuing)")
            );
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
        println!("   {}", c::skip("Golden traces config found, no baseline yet (run: renacer validate --generate golden_traces/baseline -- ./target/release/pmat --help)"));
        return Ok(true);
    }

    println!("   {}", c::dim("Golden traces detected..."));
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
            println!("      {}", c::pass("Golden traces match"));
            Ok(true)
        }
        Ok(status) if status.code() == Some(2) => {
            println!("      {}", c::skip("No golden baseline yet"));
            Ok(true)
        }
        Ok(_) => {
            println!("      {}", c::fail("Golden traces diverged"));
            Ok(false)
        }
        Err(_) => {
            println!(
                "      {}",
                c::warn("renacer not installed (skipping golden trace validation)")
            );
            Ok(true)
        }
    }
}

/// Run cargo clippy. Returns true if no warnings.
fn run_clippy_check(project_path: &PathBuf) -> Result<bool> {
    use std::process::Command;

    println!("   {}", c::dim("Running clippy..."));
    let status = Command::new("cargo")
        .args(["clippy", "--lib", "--quiet", "--", "-D", "warnings"])
        .current_dir(project_path)
        .status()
        .context("Failed to run cargo clippy")?;

    if status.success() {
        println!("      {}", c::pass("No clippy warnings"));
        Ok(true)
    } else {
        println!("      {}", c::fail("Clippy warnings found"));
        Ok(false)
    }
}

/// Run quality gates (tests, clippy, etc.)
///
/// Returns Ok(true) if all gates pass, Ok(false) if any fail, or Err on execution failure.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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
                eprintln!(
                    "   {}",
                    c::warn(&format!("Agent context index save failed: {}", e))
                );
            } else {
                let m = index.manifest();
                println!(
                    "   {} {} functions in {} files",
                    c::pass("Agent context index refreshed:"),
                    c::number(&format!("{}", m.function_count)),
                    c::number(&format!("{}", m.file_count))
                );
            }
        }
        Err(e) => {
            eprintln!(
                "   {}",
                c::warn(&format!("Agent context index build failed: {}", e))
            );
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
    /// Hypotheses that could not be measured (evidence missing), each entry
    /// naming the hypothesis and why. Distinct from `passed`/`falsified`: a
    /// hypothesis nobody could check did not "hold" (PMAT-1320).
    #[serde(default)]
    pub not_measured: Vec<String>,
}

impl FalsificationResult {
    /// True only when every hypothesis was actually checked (no missing
    /// evidence). Callers that must distinguish "all held" from "held, but
    /// N could not be measured" should check this alongside `passed`.
    pub fn fully_measured(&self) -> bool {
        self.not_measured.is_empty()
    }
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
            not_measured: Vec::new(),
        }
    }
}

/// Check test regression hypothesis. Returns (passed, validated_count).
fn falsify_test_regression(
    project_path: &PathBuf,
    step: usize,
    total: usize,
) -> Result<(bool, Vec<String>)> {
    use std::process::Command;

    println!(
        "   {} Hypothesis: No regressions introduced",
        c::label(&format!("[{}/{}]", step, total))
    );
    println!("      {}", c::dim("Falsification: Running tests..."));

    let status = Command::new("cargo")
        .args(["test", "--lib", "--quiet"])
        .current_dir(project_path)
        .status()
        .context("Failed to run cargo test")?;

    if status.success() {
        println!(
            "      {}",
            c::pass(&format!("Hypothesis holds ({}/{} validated)", step, total))
        );
        Ok((true, vec![]))
    } else {
        println!("      {}", c::fail("Hypothesis falsified: Tests fail"));
        Ok((false, vec!["Tests failed - regressions detected".into()]))
    }
}

/// Check coverage maintenance hypothesis from cached metrics.
fn falsify_coverage_regression(
    project_path: &PathBuf,
    result: &mut FalsificationResult,
    step: usize,
    total: usize,
) -> (bool, Vec<String>) {
    println!();
    println!(
        "   {} Hypothesis: Coverage maintained or improved",
        c::label(&format!("[{}/{}]", step, total))
    );
    println!(
        "      {}",
        c::dim("Falsification: Checking coverage trends...")
    );

    let trend_file = project_path.join(".pmat-metrics/trends/test-coverage.json");
    let coverage = parse_coverage_trend(&trend_file);

    match coverage {
        Some((previous, current)) => {
            result.coverage_before = Some(previous);
            result.coverage_after = Some(current);
            if current >= previous {
                result.coverage_maintained = true;
                let delta = current - previous;
                let msg = if delta > 0.0 {
                    format!("+{:.2}%", delta)
                } else {
                    format!("at {:.2}%", current)
                };
                println!(
                    "      {}",
                    c::pass(&format!(
                        "Hypothesis holds: Coverage {} ({}/{} validated)",
                        msg, step, total
                    ))
                );
                (true, vec![])
            } else {
                let delta = previous - current;
                println!(
                    "      {}",
                    c::fail(&format!("Hypothesis falsified: Coverage -{:.2}%", delta))
                );
                (false, vec![format!("Coverage dropped by {:.2}%", delta)])
            }
        }
        None => {
            let reason = format!("coverage: no trend history at {}", trend_file.display());
            result.not_measured.push(reason);
            println!(
                "      {}",
                c::warn(&format!(
                    "Not measured: no coverage history ({}/{} unmeasured)",
                    step, total
                ))
            );
            (true, vec![])
        }
    }
}

/// Parse coverage trend from JSON file. Returns (previous, current) if available.
fn parse_coverage_trend(path: &std::path::Path) -> Option<(f32, f32)> {
    let content = std::fs::read_to_string(path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    let entries = json.as_array()?;
    if entries.len() < 2 {
        return None;
    }

    let current = entries.last()?.get("value")?.as_f64()? as f32;
    let previous = entries.get(entries.len() - 2)?.get("value")?.as_f64()? as f32;
    Some((previous, current))
}

/// Check binary size hypothesis.
fn falsify_binary_bloat(
    project_path: &PathBuf,
    result: &mut FalsificationResult,
    step: usize,
    total: usize,
) -> (bool, Vec<String>) {
    println!();
    println!(
        "   {} Hypothesis: No dependency bloat",
        c::label(&format!("[{}/{}]", step, total))
    );

    let release_binary = project_path.join("target/release/pmat");
    if !release_binary.exists() {
        let reason = format!(
            "binary bloat: no release binary at {}",
            release_binary.display()
        );
        result.not_measured.push(reason);
        println!(
            "      {}",
            c::warn(&format!(
                "Not measured: no release binary ({}/{} unmeasured)",
                step, total
            ))
        );
        return (true, vec![]);
    }

    if let Ok(metadata) = std::fs::metadata(&release_binary) {
        let size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
        if size_mb <= 50.0 {
            println!(
                "      {}",
                c::pass(&format!(
                    "Hypothesis holds: {}MB < 50MB ({}/{} validated)",
                    c::number(&format!("{:.1}", size_mb)),
                    step,
                    total
                ))
            );
            (true, vec![])
        } else {
            println!(
                "      {}",
                c::fail(&format!(
                    "Hypothesis falsified: {}MB > 50MB limit",
                    c::number(&format!("{:.1}", size_mb))
                ))
            );
            (
                false,
                vec![format!("Binary size {:.1}MB exceeds 50MB limit", size_mb)],
            )
        }
    } else {
        (true, vec![])
    }
}

/// Run Karl Popper Falsification Validation
///
/// Scientific method: attempt to falsify work claims.
/// Pass only if all falsification attempts fail (work is valid).
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn run_popper_falsification(project_path: &PathBuf) -> Result<FalsificationResult> {
    let mut result = FalsificationResult::default();
    let total = 3;

    println!();
    println!(
        "{} (0/{} complete)",
        c::header("Karl Popper Falsification Validation"),
        total
    );
    println!(
        "   {}",
        c::dim("(Scientific method: attempting to falsify your work)")
    );
    println!();

    let (tests_ok, test_issues) = falsify_test_regression(project_path, 1, total)?;
    result.tests_passed = tests_ok;

    let (cov_ok, cov_issues) = falsify_coverage_regression(project_path, &mut result, 2, total);

    let (size_ok, size_issues) = falsify_binary_bloat(project_path, &mut result, 3, total);
    result.binary_size_ok = size_ok;

    result.passed = tests_ok && cov_ok && size_ok;
    let validated = [tests_ok, cov_ok, size_ok].iter().filter(|v| **v).count();
    let all_issues: Vec<String> = [test_issues, cov_issues, size_issues].concat();

    println!();
    if result.passed {
        if result.fully_measured() {
            result.summary = format!(
                "{}/{} hypotheses validated - work is valid",
                validated, total
            );
            println!(
                "   {}",
                c::pass(&format!(
                    "FALSIFICATION RESULT: PASSED ({}/{})",
                    validated, total
                ))
            );
        } else {
            result.summary = format!(
                "{}/{} validated, {} not measured - work holds but is not fully verified",
                validated,
                total,
                result.not_measured.len()
            );
            println!(
                "   {}",
                c::pass(&format!(
                    "FALSIFICATION RESULT: PASSED, WITH {} NOT MEASURED ({}/{})",
                    result.not_measured.len(),
                    validated,
                    total
                ))
            );
            for reason in &result.not_measured {
                println!("      {}", c::warn(&format!("Not measured: {}", reason)));
            }
        }
    } else {
        result.summary = format!(
            "{}/{} validated, {} falsified: {}",
            validated,
            total,
            total - validated,
            all_issues.join(", ")
        );
        println!(
            "   {}",
            c::fail(&format!(
                "FALSIFICATION RESULT: FAILED ({}/{} validated)",
                validated, total
            ))
        );
        for issue in &all_issues {
            println!("      - {}", c::fail(issue));
        }
    }
    println!();

    Ok(result)
}

/// PMAT-1317 — this file measured 0 of 263 lines covered. The subprocess
/// wrappers (`cargo test`, `cargo clippy`, `renacer`) stay untested here on
/// purpose: a nested cargo inside `cargo test` is the 75-second trap PMAT-1313
/// removed from `handle_localize`. The three functions below are pure
/// filesystem logic, and testing them found two falsifiers that pass on
/// MISSING evidence — recorded as PMAT-1320, pinned below so a fix changes a test.
#[cfg(test)]
mod tests {
    use super::*;

    fn trend(values: &[f64]) -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().expect("tempdir");
        let p = dir.path().join("test-coverage.json");
        let entries: Vec<serde_json::Value> = values
            .iter()
            .map(|v| serde_json::json!({ "value": v }))
            .collect();
        std::fs::write(&p, serde_json::to_string(&entries).expect("json")).expect("write");
        (dir, p)
    }

    #[test]
    fn coverage_trend_reads_the_last_two_entries_in_order() {
        let (_d, p) = trend(&[80.0, 85.5, 90.25]);
        assert_eq!(
            parse_coverage_trend(&p),
            Some((85.5, 90.25)),
            "(previous, current)"
        );
    }

    #[test]
    fn coverage_trend_needs_two_entries_and_a_numeric_value() {
        let (_d, one) = trend(&[80.0]);
        assert_eq!(parse_coverage_trend(&one), None, "one entry is not a trend");

        let dir = tempfile::tempdir().expect("tempdir");
        let bad = dir.path().join("t.json");
        std::fs::write(&bad, r#"[{"value":"eighty"},{"value":90}]"#).expect("write");
        assert_eq!(
            parse_coverage_trend(&bad),
            None,
            "a non-numeric value is refused"
        );
        std::fs::write(&bad, "not json").expect("write");
        assert_eq!(
            parse_coverage_trend(&bad),
            None,
            "unparsable is None, not a panic"
        );
        assert_eq!(
            parse_coverage_trend(&dir.path().join("absent.json")),
            None,
            "a missing file is None"
        );
    }

    fn project_with_trend(values: Option<&[f64]>) -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        if let Some(v) = values {
            let d = dir.path().join(".pmat-metrics/trends");
            std::fs::create_dir_all(&d).expect("mkdir");
            let entries: Vec<serde_json::Value> = v
                .iter()
                .map(|x| serde_json::json!({ "value": x }))
                .collect();
            std::fs::write(
                d.join("test-coverage.json"),
                serde_json::to_string(&entries).expect("json"),
            )
            .expect("write");
        }
        dir
    }

    #[test]
    fn coverage_regression_is_falsified_by_a_drop_and_holds_on_a_rise() {
        let dir = project_with_trend(Some(&[90.0, 85.0]));
        let mut r = FalsificationResult::default();
        let (ok, reasons) = falsify_coverage_regression(&dir.path().to_path_buf(), &mut r, 1, 1);
        assert!(!ok, "90 → 85 is a regression");
        assert_eq!(reasons, vec!["Coverage dropped by 5.00%".to_string()]);
        assert_eq!(
            (r.coverage_before, r.coverage_after),
            (Some(90.0), Some(85.0))
        );
        assert!(!r.coverage_maintained);

        let dir = project_with_trend(Some(&[85.0, 90.0]));
        let mut r = FalsificationResult::default();
        let (ok, reasons) = falsify_coverage_regression(&dir.path().to_path_buf(), &mut r, 1, 1);
        assert!(ok && reasons.is_empty(), "85 → 90 holds");
        assert!(r.coverage_maintained);

        let dir = project_with_trend(Some(&[90.0, 90.0]));
        let mut r = FalsificationResult::default();
        let (ok, _) = falsify_coverage_regression(&dir.path().to_path_buf(), &mut r, 1, 1);
        assert!(ok, "flat is maintained");
    }

    /// PMAT-1320: a falsifier that reports "Hypothesis holds" when the evidence
    /// is ABSENT cannot be falsified. No trend file → a third state
    /// (`not_measured`), not `coverage_maintained = true`.
    #[test]
    fn coverage_regression_with_no_history_is_not_measured_not_validated() {
        let dir = project_with_trend(None);
        let mut r = FalsificationResult::default();
        let (ok, reasons) = falsify_coverage_regression(&dir.path().to_path_buf(), &mut r, 1, 1);
        assert!(
            ok && reasons.is_empty(),
            "PMAT-1320: missing evidence must not read as falsified"
        );
        assert!(
            !r.coverage_maintained,
            "PMAT-1320: missing history must not be reported as maintained coverage"
        );
        assert_eq!(r.not_measured.len(), 1, "{:?}", r.not_measured);
        assert!(
            r.not_measured[0].contains("coverage"),
            "{:?}",
            r.not_measured
        );
        assert_eq!((r.coverage_before, r.coverage_after), (None, None));
    }

    #[test]
    fn binary_bloat_is_falsified_above_fifty_megabytes_and_holds_below() {
        let dir = tempfile::tempdir().expect("tempdir");
        let rel = dir.path().join("target/release");
        std::fs::create_dir_all(&rel).expect("mkdir");
        let bin = rel.join("pmat");

        // Sparse files: the metadata length is what is measured, not the bytes on disk.
        let f = std::fs::File::create(&bin).expect("create");
        f.set_len(51 * 1024 * 1024).expect("set_len");
        let mut r = FalsificationResult::default();
        let (ok, reasons) = falsify_binary_bloat(&dir.path().to_path_buf(), &mut r, 1, 1);
        assert!(!ok, "51MB exceeds the 50MB limit");
        assert_eq!(reasons.len(), 1);
        assert!(reasons[0].contains("exceeds 50MB"), "{reasons:?}");
        assert!(r.not_measured.is_empty(), "measured, just falsified");

        f.set_len(50 * 1024 * 1024).expect("set_len");
        let mut r = FalsificationResult::default();
        let (ok, reasons) = falsify_binary_bloat(&dir.path().to_path_buf(), &mut r, 1, 1);
        assert!(ok && reasons.is_empty(), "exactly 50MB is within the limit");
        assert!(r.not_measured.is_empty(), "measured, held");
    }

    /// PMAT-1320, second instance: no release binary is a third state, not a pass.
    #[test]
    fn binary_bloat_with_no_binary_is_not_measured_not_validated() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut r = FalsificationResult::default();
        let (ok, reasons) = falsify_binary_bloat(&dir.path().to_path_buf(), &mut r, 1, 1);
        assert!(
            ok && reasons.is_empty(),
            "PMAT-1320: missing evidence must not read as falsified"
        );
        assert_eq!(r.not_measured.len(), 1, "{:?}", r.not_measured);
        assert!(
            r.not_measured[0].contains("binary bloat") || r.not_measured[0].contains("binary"),
            "{:?}",
            r.not_measured
        );
    }
}
