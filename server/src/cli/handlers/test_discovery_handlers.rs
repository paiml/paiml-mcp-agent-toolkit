//! Test discovery and fixing handlers for GH-98
//!
//! Systematic test fixing agent with 5-phase automation:
//! 1. Discovery: Run tests, capture ALL failures
//! 2. Categorization: Group by root cause
//! 3. Bulk Marking: Add #[ignore] with reasons
//! 4. Verification: Ensure all tests pass
//! 5. Tracking: Create GitHub issues

use crate::cli::commands::TestDiscoveryCommands;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Test failure information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestFailure {
    /// Full test name (module::function)
    pub name: String,
    /// File path where test is defined
    pub file: PathBuf,
    /// Line number in file
    pub line: Option<u32>,
    /// Failure reason/message
    pub reason: String,
    /// Failure category
    pub category: FailureCategory,
    /// Test duration (if available)
    pub duration_ms: Option<u64>,
}

/// Failure categories for triage
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FailureCategory {
    /// Test timed out
    Timeout,
    /// Compilation error
    CompileError,
    /// Runtime panic/error
    RuntimeError,
    /// Assertion failure
    AssertionFailure,
    /// Unknown/uncategorized
    Unknown,
}

/// Test discovery report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryReport {
    /// Total tests discovered
    pub total_tests: usize,
    /// Number of failures
    pub failures: usize,
    /// List of all failures
    pub test_failures: Vec<TestFailure>,
    /// Discovery timestamp
    pub timestamp: String,
    /// Command used
    pub command: String,
}

/// Handle test-discovery command
pub async fn handle_test_discovery_command(command: TestDiscoveryCommands) -> Result<()> {
    match command {
        TestDiscoveryCommands::Run {
            path,
            output,
            use_nextest,
            timeout,
        } => handle_discovery_run(&path, &output, use_nextest, timeout).await,

        TestDiscoveryCommands::Categorize { input, output } => {
            handle_categorization(&input, &output).await
        }

        TestDiscoveryCommands::Mark { input, apply } => handle_mark(&input, apply).await,

        TestDiscoveryCommands::Verify { path } => handle_verify(&path).await,
    }
}

/// Phase 1: Discovery - Run tests and capture ALL failures
async fn handle_discovery_run(
    project_path: &Path,
    output_path: &Path,
    use_nextest: bool,
    timeout: u64,
) -> Result<()> {
    println!("🔍 Discovering test failures in {}", project_path.display());
    println!("   Using: {}", if use_nextest { "cargo nextest" } else { "cargo test" });
    println!("   Timeout: {}s", timeout);
    println!();

    // Build the command
    let mut cmd = if use_nextest {
        let mut c = Command::new("cargo");
        c.arg("nextest")
            .arg("run")
            .arg("--workspace")
            .arg("--no-fail-fast")
            .arg("--message-format")
            .arg("json")
            .current_dir(project_path);
        c
    } else {
        let mut c = Command::new("cargo");
        c.arg("test")
            .arg("--workspace")
            .arg("--no-fail-fast")
            .arg("--")
            .arg("--format")
            .arg("json")
            .current_dir(project_path);
        c
    };

    // Run the command and capture output
    println!("📊 Running tests (this may take a while)...");
    let output = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("Failed to run test command")?;

    // Parse the output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("\n📈 Parsing test results...");
    let failures = parse_test_output(&stdout, &stderr)?;

    // Create discovery report
    let report = DiscoveryReport {
        total_tests: count_total_tests(&stdout)?,
        failures: failures.len(),
        test_failures: failures.clone(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        command: format!("{:?}", cmd),
    };

    // Write to output file
    let json = serde_json::to_string_pretty(&report)?;
    std::fs::write(output_path, json)?;

    // Print summary
    println!("\n✅ Discovery complete:");
    println!("   Total tests: {}", report.total_tests);
    println!("   Failures: {}", report.failures);
    println!("   Output: {}", output_path.display());
    println!();

    // Print categorized summary
    print_category_summary(&failures);

    Ok(())
}

/// Parse test output to extract failures
fn parse_test_output(stdout: &str, _stderr: &str) -> Result<Vec<TestFailure>> {
    let mut failures = Vec::new();

    // Parse JSON lines from nextest/cargo test
    for line in stdout.lines() {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
            // Check if this is a test failure event
            if json.get("type").and_then(|t| t.as_str()) == Some("test")
                && json.get("event").and_then(|e| e.as_str()) == Some("failed")
            {
                let name = json
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("unknown")
                    .to_string();

                let reason = json
                    .get("stdout")
                    .and_then(|s| s.as_str())
                    .or_else(|| json.get("message").and_then(|m| m.as_str()))
                    .unwrap_or("Unknown failure")
                    .to_string();

                let category = categorize_failure(&reason);

                failures.push(TestFailure {
                    name,
                    file: PathBuf::from("unknown"), // Will be resolved later
                    line: None,
                    reason,
                    category,
                    duration_ms: json.get("exec_time").and_then(|d| d.as_u64()),
                });
            }
        }
    }

    Ok(failures)
}

/// Categorize failure by examining the error message
fn categorize_failure(reason: &str) -> FailureCategory {
    if reason.contains("timed out") || reason.contains("Timeout") {
        FailureCategory::Timeout
    } else if reason.contains("failed to compile") || reason.contains("unresolved import") {
        FailureCategory::CompileError
    } else if reason.contains("panicked at") || reason.contains("thread panicked") {
        FailureCategory::RuntimeError
    } else if reason.contains("assert") || reason.contains("expected") {
        FailureCategory::AssertionFailure
    } else {
        FailureCategory::Unknown
    }
}

/// Count total tests from output
fn count_total_tests(_stdout: &str) -> Result<usize> {
    // Simple implementation - count test events
    // TODO: Improve this to get accurate count
    Ok(0)
}

/// Print categorized summary
fn print_category_summary(failures: &[TestFailure]) {
    use std::collections::HashMap;

    let mut by_category: HashMap<String, usize> = HashMap::new();
    for failure in failures {
        let cat = format!("{:?}", failure.category);
        *by_category.entry(cat).or_insert(0) += 1;
    }

    println!("📊 Failures by category:");
    for (category, count) in by_category {
        println!("   {}: {}", category, count);
    }
}

/// Phase 2: Categorization - Group failures by root cause
async fn handle_categorization(_input: &Path, _output: &Path) -> Result<()> {
    println!("📋 Categorization not yet implemented (GH-98 Phase 2)");
    println!("   Use: pmat test-discovery categorize -i failures.json");
    Ok(())
}

/// Phase 3: Mark - Add #[ignore] attributes
async fn handle_mark(_input: &Path, _apply: bool) -> Result<()> {
    println!("🏷️  Mark not yet implemented (GH-98 Phase 3)");
    println!("   Use: pmat test-discovery mark -i categories.json --apply");
    Ok(())
}

/// Phase 4: Verify - Ensure all tests pass
async fn handle_verify(_path: &Path) -> Result<()> {
    println!("✅ Verify not yet implemented (GH-98 Phase 4)");
    println!("   Use: pmat test-discovery verify");
    Ok(())
}
