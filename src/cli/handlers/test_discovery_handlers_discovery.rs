/// Phase 1: Discovery - Run tests and capture ALL failures
async fn handle_discovery_run(
    project_path: &Path,
    output_path: &Path,
    use_nextest: bool,
    timeout: u64,
) -> Result<()> {
    println!("🔍 Discovering test failures in {}", project_path.display());
    println!(
        "   Using: {}",
        if use_nextest {
            "cargo nextest"
        } else {
            "cargo test"
        }
    );
    println!("   Timeout: {}s", timeout);
    println!();

    // Build the command
    // cargo test --format json requires nightly; nextest uses --message-format libtest-json (experimental).
    // Use the standard human-readable output and parse "test result:" summary lines instead.
    let mut cmd = if use_nextest {
        let mut c = Command::new("cargo");
        c.arg("nextest")
            .arg("run")
            .arg("--workspace")
            .arg("--no-fail-fast")
            .current_dir(project_path);
        c
    } else {
        let mut c = Command::new("cargo");
        c.arg("test")
            .arg("--workspace")
            .arg("--no-fail-fast")
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

    // Create discovery report (check both stdout and stderr for summary lines)
    let combined_for_count = format!("{}\n{}", stdout, stderr);

    // A runner that never started (missing `cargo nextest`, build failure) used
    // to be indistinguishable from a clean run: `output.status` was discarded,
    // the empty output parsed to zero tests, and the command printed
    // "✅ Discovery complete: Total tests: 0" and exited 0. Zero tests executed
    // is not a green run — it is a run that produced no evidence at all.
    check_runner_actually_ran(&output.status, &combined_for_count, &stderr)?;
    let report = DiscoveryReport {
        total_tests: count_total_tests(&combined_for_count)?,
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

/// True when the captured output contains a runner summary line, i.e. proof
/// that a test binary actually ran to completion.
fn has_test_summary_line(output: &str) -> bool {
    output.lines().map(str::trim).any(|line| {
        line.starts_with("test result:")
            || (line.starts_with("Summary") && line.contains("tests run"))
    })
}

/// Distinguish "tests ran and some failed" from "the runner never ran".
///
/// A failing test suite exits non-zero *and* prints a summary line; a missing
/// `cargo nextest` or a build failure exits non-zero with no summary at all.
/// Only the second case is an error here — reporting it as a zero-test success
/// is what let a broken runner masquerade as a green discovery.
fn check_runner_actually_ran(
    status: &std::process::ExitStatus,
    combined_output: &str,
    stderr: &str,
) -> Result<()> {
    if status.success() || has_test_summary_line(combined_output) {
        return Ok(());
    }

    let detail = stderr.trim();
    let detail = if detail.is_empty() {
        "no output captured".to_string()
    } else {
        // Keep the tail: cargo puts the actual error last.
        detail
            .lines()
            .rev()
            .take(20)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n")
    };

    anyhow::bail!(
        "test runner did not run any tests (exit status: {status}); no test summary line was produced.\n{detail}"
    )
}

/// Parse test output to extract failures from human-readable cargo test / nextest output.
/// Matches lines like:
///   "test some::module::test_name ... FAILED"       (cargo test)
///   "    FAIL [   0.123s] crate test_name"           (nextest)
fn parse_test_output(stdout: &str, stderr: &str) -> Result<Vec<TestFailure>> {
    let mut failures = Vec::new();
    let combined = format!("{}\n{}", stdout, stderr);

    for line in combined.lines() {
        let trimmed = line.trim();

        // cargo test format: "test path::to::test ... FAILED"
        if trimmed.starts_with("test ") && trimmed.ends_with("FAILED") {
            let name = trimmed
                .strip_prefix("test ")
                .unwrap_or(trimmed)
                .split(" ... ")
                .next()
                .unwrap_or("unknown")
                .trim()
                .to_string();
            failures.push(TestFailure {
                name,
                file: PathBuf::from("unknown"),
                line: None,
                reason: "FAILED".to_string(),
                category: FailureCategory::Unknown,
                duration_ms: None,
            });
            continue;
        }

        // nextest format: "    FAIL [   0.123s] crate::binary test_name"
        // Also: "        (N/M) FAIL [   0.123s] crate::binary test_name" (with progress prefix)
        let nextest_line = trimmed
            .strip_prefix('(')
            .and_then(|s| s.split(')').nth(1))
            .map(str::trim)
            .unwrap_or(trimmed);
        if nextest_line.starts_with("FAIL") {
            if let Some(after_bracket) = nextest_line.split(']').nth(1) {
                let name = after_bracket.trim().to_string();
                // Deduplicate: nextest may print FAIL lines in both stdout and stderr
                if !failures.iter().any(|f| f.name == name) {
                    failures.push(TestFailure {
                        name,
                        file: PathBuf::from("unknown"),
                        line: None,
                        reason: "FAILED".to_string(),
                        category: FailureCategory::Unknown,
                        duration_ms: None,
                    });
                }
            }
        }
    }

    // Try to refine failure reasons from the "failures:" section at the bottom
    refine_failure_reasons(&combined, &mut failures);

    Ok(failures)
}

/// Extract detailed failure reasons from the "failures:" block printed by cargo test
fn refine_failure_reasons(output: &str, failures: &mut [TestFailure]) {
    let mut current_test: Option<String> = None;
    let mut current_reason = String::new();

    for trimmed in output
        .lines()
        .map(str::trim)
        .skip_while(|l| *l != "failures:")
        .skip(1)
    {
        // End of failures section
        if trimmed.starts_with("test result:") || trimmed == "failures:" {
            flush_failure_reason(&current_test, &current_reason, failures);
            break;
        }
        // New test failure header: "---- test_name stdout ----"
        if trimmed.starts_with("---- ") && trimmed.ends_with(" ----") {
            flush_failure_reason(&current_test, &current_reason, failures);
            let inner = trimmed
                .strip_prefix("---- ")
                .and_then(|s| s.strip_suffix(" ----"))
                .unwrap_or("")
                .replace(" stdout", "");
            current_test = Some(inner);
            current_reason.clear();
        } else if current_test.is_some() {
            current_reason.push_str(trimmed);
            current_reason.push('\n');
        }
    }
}

fn flush_failure_reason(
    current_test: &Option<String>,
    current_reason: &str,
    failures: &mut [TestFailure],
) {
    let name = match current_test {
        Some(n) => n,
        None => return,
    };
    let reason = current_reason.trim();
    if reason.is_empty() {
        return;
    }
    if let Some(f) = failures.iter_mut().find(|f| f.name == *name) {
        f.reason = reason.to_string();
        f.category = categorize_failure(&f.reason);
    }
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

/// Count total tests from human-readable output.
/// Parses "test result: ok. N passed; M failed; I ignored" summary lines.
fn count_total_tests(stdout: &str) -> Result<usize> {
    let mut total = 0usize;
    for line in stdout.lines() {
        let trimmed = line.trim();
        // cargo test: "test result: ok. 123 passed; 0 failed; 5 ignored; 0 measured; 0 filtered out"
        // nextest:    "Summary [   1.234s] 123 tests run: 120 passed, 3 failed, 5 skipped"
        if trimmed.starts_with("test result:") {
            // Extract numbers: passed + failed + ignored
            let passed = extract_number_before(trimmed, " passed").unwrap_or(0);
            let failed = extract_number_before(trimmed, " failed").unwrap_or(0);
            let ignored = extract_number_before(trimmed, " ignored").unwrap_or(0);
            total += passed + failed + ignored;
        } else if trimmed.starts_with("Summary") && trimmed.contains("tests run") {
            // nextest summary: "Summary [   1.234s] 123 tests run: 120 passed, 3 failed, 5 skipped"
            if let Some(count) = extract_number_before(trimmed, " tests run") {
                total += count;
            }
        }
    }
    Ok(total)
}

/// Extract the number immediately before a suffix in a string.
/// e.g. extract_number_before("123 passed; 4 failed", " passed") => Some(123)
fn extract_number_before(s: &str, suffix: &str) -> Option<usize> {
    let idx = s.find(suffix)?;
    let before = &s[..idx];
    // Walk backwards to find the start of the number
    let num_str: String = before.chars().rev().take_while(|c| c.is_ascii_digit()).collect();
    let num_str: String = num_str.chars().rev().collect();
    num_str.parse().ok()
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(all(test, unix))]
mod runner_status_tests {
    use super::*;

    fn status(code_is_zero: bool) -> std::process::ExitStatus {
        let program = if code_is_zero { "true" } else { "false" };
        std::process::Command::new(program)
            .status()
            .expect("spawn /bin/true or /bin/false")
    }

    #[test]
    fn runner_that_never_started_is_an_error() {
        // `cargo nextest` missing: non-zero exit, no summary line anywhere.
        let stderr = "error: no such command: nextest\n";
        let err = check_runner_actually_ran(&status(false), stderr, stderr)
            .expect_err("a runner that produced no test summary must not be a success");
        let msg = err.to_string();
        assert!(msg.contains("did not run any tests"), "{msg}");
        assert!(msg.contains("no such command: nextest"), "{msg}");
    }

    #[test]
    fn failing_tests_with_a_cargo_summary_are_not_a_runner_error() {
        let out = "test foo ... FAILED\ntest result: FAILED. 1 passed; 2 failed; 0 ignored\n";
        assert!(check_runner_actually_ran(&status(false), out, "").is_ok());
    }

    #[test]
    fn failing_tests_with_a_nextest_summary_are_not_a_runner_error() {
        let out = "Summary [   1.234s] 3 tests run: 1 passed, 2 failed, 0 skipped\n";
        assert!(check_runner_actually_ran(&status(false), out, "").is_ok());
    }

    #[test]
    fn successful_run_is_never_an_error() {
        assert!(check_runner_actually_ran(&status(true), "", "").is_ok());
    }

    #[test]
    fn has_test_summary_line_needs_a_real_summary() {
        assert!(!has_test_summary_line(""));
        assert!(!has_test_summary_line("error: could not compile `foo`"));
        assert!(has_test_summary_line("   test result: ok. 3 passed; 0 failed"));
        assert!(has_test_summary_line(
            "Summary [   0.1s] 3 tests run: 3 passed, 0 failed, 0 skipped"
        ));
    }
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
