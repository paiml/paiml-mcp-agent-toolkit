//! Test stability analysis handler
//!
//! Runs tests multiple times to detect flaky tests and
//! timeout-sensitive tests with high duration variance.

use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;

/// Per-test timing data
#[derive(Debug, serde::Serialize)]
struct TestResult {
    name: String,
    pass_count: usize,
    fail_count: usize,
    pass_rate: f64,
    durations_ms: Vec<f64>,
    mean_ms: f64,
    p95_ms: f64,
    max_ms: f64,
    variance_ratio: f64,
    classification: String,
    recommendation: String,
}

/// Full stability analysis result
#[derive(Debug, serde::Serialize)]
struct StabilityAnalysis {
    total_tests: usize,
    runs: usize,
    stable_count: usize,
    flaky_count: usize,
    timeout_sensitive_count: usize,
    flaky_tests: Vec<TestResult>,
    timeout_sensitive_tests: Vec<TestResult>,
    total_duration_secs: f64,
}

/// Handle the test-stability command
pub async fn handle_test_stability(
    path: &Path,
    runs: usize,
    filter: Option<&str>,
    format: &crate::cli::enums::OutputFormat,
    output: Option<&Path>,
) -> Result<()> {
    use crate::cli::colors as c;

    println!("{}\n", c::header("Test Stability Analysis"));
    println!(
        "  {}Runs:{} {}  {}Filter:{} {}\n",
        c::BOLD,
        c::RESET,
        c::number(&runs.to_string()),
        c::BOLD,
        c::RESET,
        c::dim(filter.unwrap_or("(all)")),
    );

    let start = Instant::now();
    let analysis = run_stability_analysis(path, runs, filter)?;
    let total_time = start.elapsed();

    let analysis = StabilityAnalysis {
        total_duration_secs: total_time.as_secs_f64(),
        ..analysis
    };

    let formatted = match format {
        crate::cli::enums::OutputFormat::Json => serde_json::to_string_pretty(&analysis)?,
        _ => format_text(&analysis),
    };

    if let Some(output_path) = output {
        std::fs::write(output_path, &formatted)?;
        eprintln!(
            "{} Written to: {}",
            c::pass(""),
            c::path(&output_path.display().to_string())
        );
    } else {
        println!("{formatted}");
    }

    Ok(())
}

/// Run the stability analysis
fn run_stability_analysis(
    path: &Path,
    runs: usize,
    filter: Option<&str>,
) -> Result<StabilityAnalysis> {
    use crate::cli::colors as c;

    // Collect per-test results across runs
    let mut test_results: HashMap<String, Vec<(bool, f64)>> = HashMap::new();

    for run in 1..=runs {
        eprintln!("  {}Run {}/{}{}...", c::BOLD, run, runs, c::RESET,);

        let results = run_test_suite(path, filter)?;
        for (name, passed, duration_ms) in results {
            test_results
                .entry(name)
                .or_default()
                .push((passed, duration_ms));
        }
    }

    // Analyze results
    let total_tests = test_results.len();
    let mut flaky_tests = Vec::new();
    let mut timeout_sensitive_tests = Vec::new();

    for (name, results) in &test_results {
        let pass_count = results.iter().filter(|(p, _)| *p).count();
        let fail_count = results.len() - pass_count;
        let pass_rate = pass_count as f64 / results.len() as f64;

        let durations: Vec<f64> = results.iter().map(|(_, d)| *d).collect();
        let mean = durations.iter().sum::<f64>() / durations.len() as f64;
        let max = durations.iter().cloned().fold(0.0f64, f64::max);

        let mut sorted_durations = durations.clone();
        sorted_durations.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let p95_idx =
            ((sorted_durations.len() as f64 * 0.95) as usize).min(sorted_durations.len() - 1);
        let p95 = sorted_durations[p95_idx];

        let variance_ratio = if mean > 0.0 { max / mean } else { 1.0 };

        let is_flaky = pass_rate < 1.0 && pass_rate > 0.0;
        let is_timeout_sensitive = variance_ratio > 2.0 && mean > 100.0; // >100ms mean, >2x variance

        let (classification, recommendation) = if is_flaky {
            (
                "Flaky".to_string(),
                format!(
                    "Pass rate: {:.0}%. Consider adding retry logic or investigating non-determinism",
                    pass_rate * 100.0
                ),
            )
        } else if is_timeout_sensitive {
            (
                "Timeout-Sensitive".to_string(),
                format!(
                    "High variance ({:.1}x). Recommend adaptive timeout: {:.0}ms (2x P95)",
                    variance_ratio,
                    p95 * 2.0
                ),
            )
        } else {
            ("Stable".to_string(), String::new())
        };

        let test_result = TestResult {
            name: name.clone(),
            pass_count,
            fail_count,
            pass_rate,
            durations_ms: durations,
            mean_ms: mean,
            p95_ms: p95,
            max_ms: max,
            variance_ratio,
            classification,
            recommendation,
        };

        if is_flaky {
            flaky_tests.push(test_result);
        } else if is_timeout_sensitive {
            timeout_sensitive_tests.push(test_result);
        }
    }

    // Sort by severity
    flaky_tests.sort_by(|a, b| {
        a.pass_rate
            .partial_cmp(&b.pass_rate)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    timeout_sensitive_tests.sort_by(|a, b| {
        b.variance_ratio
            .partial_cmp(&a.variance_ratio)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let stable_count = total_tests - flaky_tests.len() - timeout_sensitive_tests.len();

    Ok(StabilityAnalysis {
        total_tests,
        runs,
        stable_count,
        flaky_count: flaky_tests.len(),
        timeout_sensitive_count: timeout_sensitive_tests.len(),
        flaky_tests,
        timeout_sensitive_tests,
        total_duration_secs: 0.0, // filled in by caller
    })
}

/// Run the test suite once and parse results
fn run_test_suite(path: &Path, filter: Option<&str>) -> Result<Vec<(String, bool, f64)>> {
    let mut args = vec![
        "test".to_string(),
        "--lib".to_string(),
        "--".to_string(),
        "--test-threads=4".to_string(),
        "-Z".to_string(),
        "unstable-options".to_string(),
        "--format=json".to_string(),
    ];

    if let Some(f) = filter {
        // Insert filter before --
        args.insert(2, f.to_string());
    }

    let output = std::process::Command::new("cargo")
        .args(&args)
        .current_dir(path)
        .env("RUST_MIN_STACK", "8388608")
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut results = Vec::new();

    for line in stdout.lines() {
        // Parse JSON test events
        if let Ok(event) = serde_json::from_str::<serde_json::Value>(line) {
            if event.get("type").and_then(|t| t.as_str()) == Some("test")
                && event.get("event").and_then(|e| e.as_str()).is_some()
            {
                let name = event
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("")
                    .to_string();
                let event_type = event.get("event").and_then(|e| e.as_str()).unwrap_or("");
                let exec_time = event
                    .get("exec_time")
                    .and_then(|t| t.as_f64())
                    .unwrap_or(0.0)
                    * 1000.0; // convert to ms

                if event_type == "ok" || event_type == "failed" {
                    results.push((name, event_type == "ok", exec_time));
                }
            }
        }
    }

    // Fallback: if JSON parsing yielded nothing, parse text output
    if results.is_empty() {
        parse_text_test_output(&stdout, &mut results);
    }

    Ok(results)
}

/// Fallback parser for text test output
fn parse_text_test_output(output: &str, results: &mut Vec<(String, bool, f64)>) {
    for line in output.lines() {
        let line = line.trim();
        if line.starts_with("test ") && (line.ends_with("... ok") || line.ends_with("... FAILED")) {
            let passed = line.ends_with("... ok");
            let name = line
                .strip_prefix("test ")
                .unwrap_or(line)
                .trim_end_matches(" ... ok")
                .trim_end_matches(" ... FAILED")
                .to_string();
            // No timing info available in text mode
            results.push((name, passed, 0.0));
        }
    }
}

/// Format results as colorized text
fn format_text(analysis: &StabilityAnalysis) -> String {
    use crate::cli::colors as c;
    use std::fmt::Write;

    let mut out = String::new();

    let _ = writeln!(out, "{}", c::subheader("Summary"));
    let _ = writeln!(
        out,
        "  {}Total tests:{} {}",
        c::BOLD,
        c::RESET,
        c::number(&analysis.total_tests.to_string())
    );
    let _ = writeln!(
        out,
        "  {}Stable:{} {} ({:.1}%)",
        c::BOLD,
        c::RESET,
        c::number(&analysis.stable_count.to_string()),
        analysis.stable_count as f64 / analysis.total_tests.max(1) as f64 * 100.0,
    );

    if analysis.flaky_count > 0 {
        let _ = writeln!(
            out,
            "  {}Flaky:{} {}{}{}",
            c::BOLD,
            c::RESET,
            c::RED,
            analysis.flaky_count,
            c::RESET,
        );
    }
    if analysis.timeout_sensitive_count > 0 {
        let _ = writeln!(
            out,
            "  {}Timeout-sensitive:{} {}{}{}",
            c::BOLD,
            c::RESET,
            c::YELLOW,
            analysis.timeout_sensitive_count,
            c::RESET,
        );
    }
    let _ = writeln!(
        out,
        "  {}Duration:{} {:.1}s\n",
        c::BOLD,
        c::RESET,
        analysis.total_duration_secs,
    );

    if !analysis.flaky_tests.is_empty() {
        let _ = writeln!(out, "{}\n", c::subheader("Flaky Tests"));
        for test in &analysis.flaky_tests {
            let _ = writeln!(out, "  {} {}", c::fail(""), c::label(&test.name),);
            let _ = writeln!(
                out,
                "     Pass rate: {}{:.0}%{}  Mean: {:.0}ms  Max: {:.0}ms",
                c::RED,
                test.pass_rate * 100.0,
                c::RESET,
                test.mean_ms,
                test.max_ms,
            );
            let _ = writeln!(out, "     {}", c::dim(&test.recommendation));
            let _ = writeln!(out);
        }
    }

    if !analysis.timeout_sensitive_tests.is_empty() {
        let _ = writeln!(out, "{}\n", c::subheader("Timeout-Sensitive Tests"));
        for test in &analysis.timeout_sensitive_tests {
            let _ = writeln!(out, "  {} {}", c::warn(""), c::label(&test.name),);
            let _ = writeln!(
                out,
                "     Variance: {}{:.1}x{}  Mean: {:.0}ms  P95: {:.0}ms  Max: {:.0}ms",
                c::YELLOW,
                test.variance_ratio,
                c::RESET,
                test.mean_ms,
                test.p95_ms,
                test.max_ms,
            );
            let _ = writeln!(out, "     {}", c::dim(&test.recommendation));
            let _ = writeln!(out);
        }
    }

    if analysis.flaky_count == 0 && analysis.timeout_sensitive_count == 0 {
        let _ = writeln!(out, "  {}", c::pass("All tests are stable"));
    }

    out
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_text_test_output_ok() {
        let mut results = Vec::new();
        parse_text_test_output("test my_test ... ok", &mut results);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "my_test");
        assert!(results[0].1);
    }

    #[test]
    fn test_parse_text_test_output_failed() {
        let mut results = Vec::new();
        parse_text_test_output("test my_test ... FAILED", &mut results);
        assert_eq!(results.len(), 1);
        assert!(!results[0].1);
    }

    #[test]
    fn test_parse_text_test_output_mixed() {
        let output = "test a ... ok\ntest b ... FAILED\ntest c ... ok\n";
        let mut results = Vec::new();
        parse_text_test_output(output, &mut results);
        assert_eq!(results.len(), 3);
        assert!(results[0].1);
        assert!(!results[1].1);
        assert!(results[2].1);
    }

    #[test]
    fn test_format_text_empty() {
        let analysis = StabilityAnalysis {
            total_tests: 100,
            runs: 3,
            stable_count: 100,
            flaky_count: 0,
            timeout_sensitive_count: 0,
            flaky_tests: vec![],
            timeout_sensitive_tests: vec![],
            total_duration_secs: 10.0,
        };
        let text = format_text(&analysis);
        assert!(text.contains("All tests are stable"));
    }

    #[test]
    fn test_format_text_with_flaky() {
        let analysis = StabilityAnalysis {
            total_tests: 100,
            runs: 3,
            stable_count: 99,
            flaky_count: 1,
            timeout_sensitive_count: 0,
            flaky_tests: vec![TestResult {
                name: "test_flaky_one".to_string(),
                pass_count: 2,
                fail_count: 1,
                pass_rate: 0.67,
                durations_ms: vec![100.0, 200.0, 150.0],
                mean_ms: 150.0,
                p95_ms: 200.0,
                max_ms: 200.0,
                variance_ratio: 1.33,
                classification: "Flaky".to_string(),
                recommendation: "Investigate non-determinism".to_string(),
            }],
            timeout_sensitive_tests: vec![],
            total_duration_secs: 30.0,
        };
        let text = format_text(&analysis);
        assert!(text.contains("test_flaky_one"));
        assert!(text.contains("Flaky"));
    }
}
