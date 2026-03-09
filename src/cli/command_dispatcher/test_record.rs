//! Test Record Handler: `pmat test --record`
//!
//! Records cargo test results to .pmat-metrics/ for EvoScore (CB-142).
//! Implements the data pipeline described in swe-ci-evolution.md.

#![cfg_attr(coverage_nightly, coverage(off))]

use std::path::PathBuf;

/// Parsed cargo test output
pub(crate) struct CargoTestResult {
    pub passed: u64,
    pub failed: u64,
    pub ignored: u64,
    pub total: u64,
}

/// Record cargo test results to .pmat-metrics/ for EvoScore (CB-142)
pub(crate) async fn execute_test_record(from_stdin: bool, dry_run: bool) -> anyhow::Result<()> {
    let output = if from_stdin {
        read_stdin_cargo_output()?
    } else {
        run_cargo_test()?
    };

    let result = parse_cargo_test_output(&output)?;
    let short_sha = get_current_commit_sha()?;
    let timestamp = chrono::Utc::now().to_rfc3339();

    if dry_run {
        print_dry_run(&result, &short_sha, &timestamp);
        return Ok(());
    }

    write_test_record(&result, &short_sha, &timestamp)?;
    println!(
        "Recorded: {}/{} pass (commit {})",
        result.passed, result.total, short_sha
    );
    Ok(())
}

/// Run `cargo test --no-fail-fast` and capture output
fn run_cargo_test() -> anyhow::Result<String> {
    use std::process::Command;
    println!("Running cargo test --no-fail-fast ...");
    let output = Command::new("cargo")
        .args(["test", "--no-fail-fast"])
        .env("RUST_MIN_STACK", "8388608")
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    Ok(format!("{stdout}\n{stderr}"))
}

/// Read cargo test output from stdin
fn read_stdin_cargo_output() -> anyhow::Result<String> {
    use std::io::Read;
    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf)?;
    Ok(buf)
}

/// Parse "test result: ok. N passed; M failed; K ignored" summary lines
pub(crate) fn parse_cargo_test_output(output: &str) -> anyhow::Result<CargoTestResult> {
    let mut total_passed: u64 = 0;
    let mut total_failed: u64 = 0;
    let mut total_ignored: u64 = 0;
    let mut found = false;

    for line in output.lines() {
        if line.contains("test result:") {
            found = true;
            total_passed += extract_count(line, "passed");
            total_failed += extract_count(line, "failed");
            total_ignored += extract_count(line, "ignored");
        }
    }

    if !found {
        anyhow::bail!(
            "No 'test result:' lines found in cargo test output. \
             Ensure cargo test ran successfully."
        );
    }

    Ok(CargoTestResult {
        passed: total_passed,
        failed: total_failed,
        ignored: total_ignored,
        total: total_passed + total_failed,
    })
}

/// Extract count for a keyword like "passed", "failed", "ignored"
fn extract_count(line: &str, keyword: &str) -> u64 {
    for part in line.split(';') {
        let trimmed = part.trim().trim_start_matches("test result:");
        let trimmed = trimmed
            .trim()
            .trim_start_matches("ok.")
            .trim_start_matches("FAILED.");
        let trimmed = trimmed.trim();
        if trimmed.ends_with(keyword) {
            if let Some(num_str) = trimmed.strip_suffix(keyword) {
                if let Ok(n) = num_str.trim().parse::<u64>() {
                    return n;
                }
            }
        }
    }
    0
}

/// Get short commit SHA
fn get_current_commit_sha() -> anyhow::Result<String> {
    use std::process::Command;
    let output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()?;
    if !output.status.success() {
        anyhow::bail!("Failed to get git commit SHA");
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Write test record JSON to .pmat-metrics/
fn write_test_record(
    result: &CargoTestResult,
    short_sha: &str,
    timestamp: &str,
) -> anyhow::Result<()> {
    let metrics_dir = PathBuf::from(".pmat-metrics");
    std::fs::create_dir_all(&metrics_dir)?;

    let filename = format!("commit-{short_sha}-tests.json");
    let filepath = metrics_dir.join(&filename);

    let json = serde_json::json!({
        "commit": short_sha,
        "pass": result.passed,
        "total": result.total,
        "failed": result.failed,
        "ignored": result.ignored,
        "timestamp": timestamp,
    });

    std::fs::write(&filepath, serde_json::to_string_pretty(&json)?)?;
    Ok(())
}

/// Print what would be recorded (dry-run mode)
fn print_dry_run(result: &CargoTestResult, short_sha: &str, timestamp: &str) {
    println!("Dry run — would record:");
    println!("  File: .pmat-metrics/commit-{short_sha}-tests.json");
    println!("  Commit: {short_sha}");
    println!("  Passed: {}", result.passed);
    println!("  Failed: {}", result.failed);
    println!("  Ignored: {}", result.ignored);
    println!("  Total: {}", result.total);
    println!("  Timestamp: {timestamp}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_result_line() {
        let output = "test result: ok. 19795 passed; 0 failed; 167 ignored; 0 measured; 0 filtered out; finished in 45.32s";
        let result = parse_cargo_test_output(output).unwrap();
        assert_eq!(result.passed, 19795);
        assert_eq!(result.failed, 0);
        assert_eq!(result.ignored, 167);
        assert_eq!(result.total, 19795);
    }

    #[test]
    fn test_parse_multiple_result_lines() {
        let output = "\
test result: ok. 100 passed; 2 failed; 10 ignored; 0 measured; 0 filtered out
test result: ok. 50 passed; 1 failed; 5 ignored; 0 measured; 0 filtered out";
        let result = parse_cargo_test_output(output).unwrap();
        assert_eq!(result.passed, 150);
        assert_eq!(result.failed, 3);
        assert_eq!(result.ignored, 15);
        assert_eq!(result.total, 153);
    }

    #[test]
    fn test_parse_failed_test_result() {
        let output = "test result: FAILED. 18000 passed; 500 failed; 167 ignored";
        let result = parse_cargo_test_output(output).unwrap();
        assert_eq!(result.passed, 18000);
        assert_eq!(result.failed, 500);
        assert_eq!(result.total, 18500);
    }

    #[test]
    fn test_parse_no_result_line() {
        let output = "Running tests...\nAll done.";
        let result = parse_cargo_test_output(output);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_count_passed() {
        let line = "test result: ok. 19795 passed; 0 failed; 167 ignored";
        assert_eq!(extract_count(line, "passed"), 19795);
        assert_eq!(extract_count(line, "failed"), 0);
        assert_eq!(extract_count(line, "ignored"), 167);
    }
}
