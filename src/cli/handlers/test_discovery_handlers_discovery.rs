/// Phase 1: Discovery - Run tests and capture ALL failures
async fn handle_discovery_run(
    project_path: &Path,
    output_path: &Path,
    use_nextest: bool,
    timeout: u64,
) -> Result<()> {
    crate::status_println!("🔍 Discovering test failures in {}", project_path.display());
    crate::status_println!(
        "   Using: {}",
        if use_nextest {
            "cargo nextest"
        } else {
            "cargo test"
        }
    );
    crate::status_println!("   Timeout: {}s", timeout);
    crate::status_println!();

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
    crate::status_println!("📊 Running tests (this may take a while)...");
    let output = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("Failed to run test command")?;

    // Parse the output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    crate::status_println!("\n📈 Parsing test results...");
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

    // nextest re-emits each failing test's captured libtest output, which
    // contains "test <name> ... FAILED" too. Parsing both arms turned three
    // real failures into six entries (one prefixed "(3/4) tiny ", one bare)
    // that the name-equality dedup could not collapse. When the runner is
    // nextest, its own FAIL lines are the only authority.
    let nextest = is_nextest_output(&combined);

    for line in combined.lines() {
        let trimmed = line.trim();

        // cargo test format: "test path::to::test ... FAILED"
        if !nextest && trimmed.starts_with("test ") && trimmed.ends_with("FAILED") {
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

        if let Some(name) = nextest_fail_name(trimmed) {
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

    // Try to refine failure reasons from the "failures:" section at the bottom
    refine_failure_reasons(&combined, &mut failures);
    // ...and from nextest's per-test captured output, which has no such section.
    refine_nextest_reasons(&combined, &mut failures);

    Ok(failures)
}

/// True when the captured output came from `cargo nextest` rather than libtest.
fn is_nextest_output(output: &str) -> bool {
    output.lines().map(str::trim).any(|line| {
        (line.starts_with("Summary") && line.contains("tests run"))
            || (line.starts_with("Starting") && line.contains("tests across"))
            || nextest_fail_name(line).is_some()
    })
}

/// Extract the test name from a nextest FAIL line, if this is one.
///
/// nextest prints `FAIL [   0.123s] (1/4) tiny fail_mod::always_fails_assert`
/// (and older builds put the `(n/m)` counter before `FAIL`). Everything after
/// the `]` used to be taken verbatim as the name, so reports carried names
/// like `(3/4) tiny fail_mod::always_panics` — a progress counter and a binary
/// id that are not part of any test's name.
fn nextest_fail_name(line: &str) -> Option<String> {
    let line = line
        .strip_prefix('(')
        .and_then(|s| s.split(')').nth(1))
        .map(str::trim)
        .unwrap_or(line);
    if !line.starts_with("FAIL") {
        return None;
    }
    let after_bracket = line.split(']').nth(1)?.trim();
    // Drop a leading "(n/m)" progress counter, then the binary id: what is
    // left is the test path, the last whitespace-separated token.
    let rest = after_bracket
        .strip_prefix('(')
        .and_then(|s| s.split_once(')'))
        .map_or(after_bracket, |(_, tail)| tail.trim());
    let name = rest.split_whitespace().next_back()?;
    (!name.is_empty()).then(|| name.to_string())
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

/// Extract failure reasons from nextest's per-test captured output.
///
/// `refine_failure_reasons` only understands libtest's `failures:` section
/// with its `---- <name> stdout ----` headers, which nextest never prints — so
/// every real nextest failure kept the placeholder reason "FAILED" and
/// `categorize_failure` was never reached, leaving `pmat test-discovery
/// categorize` with a single "Unknown / priority 4" bucket for a run whose
/// failures were an assertion, a panic and a flake. nextest labels each
/// captured block with the failing test's name (`--- STDERR: tiny my::test ---`
/// in older builds, a `stderr ───` block under the FAIL line in newer ones),
/// so evidence is attributed to the most recent test named above it.
fn refine_nextest_reasons(output: &str, failures: &mut [TestFailure]) {
    let lines: Vec<&str> = output.lines().map(str::trim).collect();
    let mut current: Option<usize> = None;

    for (i, line) in lines.iter().enumerate() {
        if let Some(idx) = failures
            .iter()
            .position(|f| !f.name.is_empty() && line.contains(f.name.as_str()))
        {
            // Not `continue`: a panic line names its own test *and* is the
            // evidence for it ("thread 'my::test' panicked at …").
            current = Some(idx);
        }
        let Some(idx) = current else { continue };
        if failures[idx].reason != "FAILED" || !is_failure_evidence(line) {
            continue;
        }
        let mut reason = (*line).to_string();
        // "panicked at src/lib.rs:12:9:" carries its message on the next line.
        if reason.ends_with(':') {
            if let Some(next) = lines.get(i + 1).filter(|l| !l.is_empty()) {
                reason.push(' ');
                reason.push_str(next);
            }
        }
        failures[idx].category = categorize_failure(&reason);
        failures[idx].reason = reason;
    }
}

/// Lines that actually say why a test failed (as opposed to nextest framing).
fn is_failure_evidence(line: &str) -> bool {
    const MARKERS: &[&str] = &[
        "panicked at",
        "assertion",
        "timed out",
        "overflowed its stack",
        "SIGSEGV",
        "SIGABRT",
        "SIGILL",
        "No such file or directory",
        "onnection refused",
        "Address already in use",
    ];
    line.starts_with("error:") || MARKERS.iter().any(|m| line.contains(m))
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
    } else if reason.contains("panicked at")
        || reason.contains("thread panicked")
        // A crashed test binary reports no panic message at all; leaving these
        // in Unknown put real stack overflows and signals in the
        // "needs triage" bucket alongside genuinely unparsed failures.
        || reason.contains("overflowed its stack")
        || reason.contains("SIGSEGV")
        || reason.contains("SIGABRT")
        || reason.contains("SIGILL")
    {
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
    // nextest prints exactly ONE "Summary [..] N tests run" line for the whole
    // run, but it also re-emits each failing test's captured libtest output —
    // whose "test result:" lines were summed on top of the summary, reporting
    // 7 tests for a 4-test crate. When a nextest summary is present it is the
    // only count; libtest summaries are then just echoed fragments of it.
    let mut nextest_total: Option<usize> = None;
    let mut libtest_total = 0usize;

    for line in stdout.lines() {
        let trimmed = line.trim();
        // cargo test: "test result: ok. 123 passed; 0 failed; 5 ignored; 0 measured; 0 filtered out"
        // nextest:    "Summary [   1.234s] 123 tests run: 120 passed, 3 failed, 5 skipped"
        if trimmed.starts_with("test result:") {
            // Extract numbers: passed + failed + ignored
            let passed = extract_number_before(trimmed, " passed").unwrap_or(0);
            let failed = extract_number_before(trimmed, " failed").unwrap_or(0);
            let ignored = extract_number_before(trimmed, " ignored").unwrap_or(0);
            libtest_total += passed + failed + ignored;
        } else if trimmed.starts_with("Summary") && trimmed.contains("tests run") {
            if let Some(count) = extract_number_before(trimmed, " tests run") {
                nextest_total = Some(nextest_total.map_or(count, |t: usize| t.max(count)));
            }
        }
    }
    Ok(nextest_total.unwrap_or(libtest_total))
}

/// Extract the number immediately before a suffix in a string.
/// e.g. extract_number_before("123 passed; 4 failed", " passed") => Some(123)
fn extract_number_before(s: &str, suffix: &str) -> Option<usize> {
    let idx = s.find(suffix)?;
    let before = &s[..idx];
    // Walk backwards to find the start of the number
    let num_str: String = before
        .chars()
        .rev()
        .take_while(|c| c.is_ascii_digit())
        .collect();
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
        assert!(has_test_summary_line(
            "   test result: ok. 3 passed; 0 failed"
        ));
        assert!(has_test_summary_line(
            "Summary [   0.1s] 3 tests run: 3 passed, 0 failed, 0 skipped"
        ));
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod nextest_parsing_tests {
    //! Regression tests for the nextest output shape: a real run of a 4-test
    //! crate with 3 failures used to report 7 tests and 6 failures, with
    //! names carrying nextest's "(n/m) <binary>" progress prefix and every
    //! reason stuck on the placeholder "FAILED".
    use super::*;

    /// Real `cargo nextest run --workspace --no-fail-fast` shape: nextest's
    /// own FAIL lines plus the libtest output it re-emits per failing test.
    const NEXTEST_OUTPUT: &str = r#"
    Starting 4 tests across 1 binary (run ID: abc, nextest profile: default)
        PASS [   0.002s] (1/4) tiny ok_mod::passes
        FAIL [   0.004s] (2/4) tiny fail_mod::always_fails_assert

--- STDOUT:              tiny fail_mod::always_fails_assert ---

running 1 test
test fail_mod::always_fails_assert ... FAILED

failures:

failures:
    fail_mod::always_fails_assert

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

--- STDERR:              tiny fail_mod::always_fails_assert ---
thread 'fail_mod::always_fails_assert' panicked at src/lib.rs:12:9:
assertion `left == right` failed
  left: 1
 right: 2

        FAIL [   0.005s] (3/4) tiny slow_mod::takes_forever

--- STDERR:              tiny slow_mod::takes_forever ---
error: test timed out after 60s

        FAIL [   0.006s] (4/4) tiny deep_mod::recurses

--- STDERR:              tiny deep_mod::recurses ---
thread 'deep_mod::recurses' has overflowed its stack

------------
     Summary [   0.010s] 4 tests run: 1 passed, 3 failed, 0 skipped
"#;

    #[test]
    fn nextest_failures_are_not_double_counted_and_names_have_no_progress_prefix() {
        let failures = parse_test_output(NEXTEST_OUTPUT, "").expect("parse");
        let names: Vec<&str> = failures.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(
            names,
            vec![
                "fail_mod::always_fails_assert",
                "slow_mod::takes_forever",
                "deep_mod::recurses"
            ],
            "one entry per failing test, named as the test is named"
        );
    }

    #[test]
    fn nextest_total_is_the_summary_line_not_the_re_emitted_libtest_lines() {
        assert_eq!(
            count_total_tests(NEXTEST_OUTPUT).expect("count"),
            4,
            "a 4-test crate has 4 tests, however many libtest summaries nextest echoes"
        );
    }

    #[test]
    fn nextest_failures_are_categorized_from_their_captured_output() {
        let failures = parse_test_output(NEXTEST_OUTPUT, "").expect("parse");
        let by_name = |n: &str| {
            failures
                .iter()
                .find(|f| f.name == n)
                .unwrap_or_else(|| panic!("missing {n}"))
        };
        assert_ne!(
            by_name("fail_mod::always_fails_assert").category,
            FailureCategory::Unknown,
            "a panicking assertion must not stay uncategorized: {:?}",
            by_name("fail_mod::always_fails_assert").reason
        );
        assert!(by_name("fail_mod::always_fails_assert")
            .reason
            .contains("assertion"));
        assert_eq!(
            by_name("slow_mod::takes_forever").category,
            FailureCategory::Timeout
        );
        assert_ne!(
            by_name("deep_mod::recurses").category,
            FailureCategory::Unknown
        );
    }

    #[test]
    fn plain_cargo_test_output_is_still_parsed() {
        let out = "running 2 tests\ntest a::b ... FAILED\ntest c::d ... ok\n\n\
                   failures:\n\n---- a::b stdout ----\nthread 'a::b' panicked at src/l.rs:1:1:\n\
                   boom\n\ntest result: FAILED. 1 passed; 1 failed; 0 ignored\n";
        let failures = parse_test_output(out, "").expect("parse");
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].name, "a::b");
        assert_eq!(count_total_tests(out).expect("count"), 2);
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
