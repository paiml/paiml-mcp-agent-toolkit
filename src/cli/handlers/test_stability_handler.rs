//! Test stability analysis handler
//!
//! Runs tests multiple times to detect flaky tests, tests whose duration varies
//! from run to run, and tests that are simply slow on every run.
//!
//! # What counts as a finding
//!
//! The rules are constants here (`VARIANCE_RATIO_THRESHOLD`,
//! `HIGH_VARIANCE_MIN_MEAN_MS`, `CONSISTENTLY_SLOW_MS`) and are also printed in
//! every report as `Criteria`, because a `timeout_sensitive: 0` is unreadable
//! without the rule that produced it. Variance alone used to be the only rule,
//! which made the report structurally blind to the worst case it exists to find
//! — a test that reliably burns ten seconds has variance ratio 1.0 (#950).
//!
//! # Where the durations come from
//!
//! Per-test durations used to be manufactured: the whole suite was timed once
//! and `elapsed / results.len()` was written into every test, so a test costing
//! microseconds was reported at `mean_ms 5022.92` next to a test that really
//! did sleep ten seconds, every test in a run shared one number, and the
//! `variance_ratio > 2.0` timeout-sensitive branch could only ever respond to
//! whole-suite wall-clock jitter (#950).
//!
//! Stable `libtest` has no per-test timing: `cargo test -- --report-time` is
//! rejected off nightly. `cargo nextest` does report it (`PASS [ 0.004s]`), so
//! that is the runner used when it is installed. When it is not, there is no
//! per-test timing to be had, and this handler says so — `timing.status =
//! "not_measured"`, empty `durations_ms`, `null` statistics and a `null`
//! `timeout_sensitive_count` — rather than dividing a number it does have by a
//! number it does not.

use anyhow::{bail, Result};
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;

/// Where per-test durations came from, or why there are none.
///
/// `absence` is a value here, not a zero: a consumer can tell "no test was
/// timeout-sensitive" from "nothing was timed".
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum Timing {
    /// Real per-test durations, reported by the named runner.
    Measured { runner: String },
    /// No per-test durations, and why.
    NotMeasured { reason: String },
}

impl Timing {
    fn is_measured(&self) -> bool {
        matches!(self, Timing::Measured { .. })
    }

    /// One-line description for the human reports.
    fn describe(&self) -> String {
        match self {
            Timing::Measured { runner } => format!("measured by {runner}"),
            Timing::NotMeasured { reason } => format!("not measured ({reason})"),
        }
    }
}

/// The runner used to execute the suite.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Runner {
    /// `cargo nextest run` — reports a real duration per test.
    Nextest,
    /// `cargo test` — reports pass/fail only on stable Rust.
    CargoTest,
}

impl Runner {
    fn timing(self) -> Timing {
        match self {
            Runner::Nextest => Timing::Measured {
                runner: "cargo-nextest".to_string(),
            },
            Runner::CargoTest => Timing::NotMeasured {
                reason: "cargo test reports no per-test timing on stable Rust \
                         (libtest's --report-time is nightly-only); \
                         install cargo-nextest for per-test durations"
                    .to_string(),
            },
        }
    }
}

/// Is a timing-capable runner installed?
fn detect_runner() -> Runner {
    let available = std::process::Command::new("cargo")
        .args(["nextest", "--version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if available {
        Runner::Nextest
    } else {
        Runner::CargoTest
    }
}

/// Per-test timing data
///
/// The statistics are `Option` because a runner that reports no timings must
/// not be papered over with a plausible-looking number.
#[derive(Debug, serde::Serialize)]
struct TestResult {
    name: String,
    pass_count: usize,
    fail_count: usize,
    pass_rate: f64,
    /// Empty when the runner reported no per-test timing.
    durations_ms: Vec<f64>,
    mean_ms: Option<f64>,
    p95_ms: Option<f64>,
    max_ms: Option<f64>,
    variance_ratio: Option<f64>,
    classification: String,
    recommendation: String,
}

/// Variance (max/mean) above which a test's duration is judged unstable.
const VARIANCE_RATIO_THRESHOLD: f64 = 2.0;

/// Mean below which a variance ratio is noise rather than a timeout risk:
/// a 0.1ms test that once took 0.4ms is 4x variable and threatens nothing.
const HIGH_VARIANCE_MIN_MEAN_MS: f64 = 100.0;

/// Mean at or above which a test is reported even when its timing is perfectly
/// consistent.
///
/// #950 residual: the only criterion was `variance_ratio > 2.0 && mean > 100`,
/// so a test that sleeps ten seconds on EVERY run has variance ratio 1.0,
/// classified Stable, and stable tests are not retained — the test most likely
/// to hit a CI timeout was the one the report could never mention. Consistency
/// is not safety: a test whose mean sits at 9.5s under a 10s timeout is
/// maximally timeout-sensitive at zero variance. The threshold is disclosed in
/// the report (see `Criteria`) so a `0` can be read against the rule that
/// produced it.
const CONSISTENTLY_SLOW_MS: f64 = 1000.0;

/// Why a test's duration puts it at risk of a timeout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TimeoutRisk {
    /// Duration swings from run to run.
    HighVariance,
    /// Reliably slow every run.
    ConsistentlySlow,
}

impl TimeoutRisk {
    fn classification(self) -> &'static str {
        match self {
            TimeoutRisk::HighVariance => "Timeout-Sensitive",
            TimeoutRisk::ConsistentlySlow => "Consistently-Slow",
        }
    }
}

/// Classify a test's durations, or `None` when they are no timeout risk.
///
/// `None` is also returned when there are no durations at all — but that case is
/// distinguished for the caller by `Timing`, which carries the reason nothing
/// was timed, so an unevaluated test is never counted as an evaluated-and-clean
/// one.
fn timeout_risk(stats: &DurationStats) -> Option<TimeoutRisk> {
    let (ratio, mean) = (stats.variance_ratio?, stats.mean?);
    if ratio > VARIANCE_RATIO_THRESHOLD && mean > HIGH_VARIANCE_MIN_MEAN_MS {
        Some(TimeoutRisk::HighVariance)
    } else if mean >= CONSISTENTLY_SLOW_MS {
        Some(TimeoutRisk::ConsistentlySlow)
    } else {
        None
    }
}

/// The rules this report applied, carried in the output.
///
/// Without them a `timeout_sensitive: 0` is unreadable: the reader cannot tell
/// a suite with no timeout risk from a criterion that could not fire.
#[derive(Debug, Clone, serde::Serialize)]
struct Criteria {
    flaky: String,
    timeout_sensitive: String,
    consistently_slow: String,
}

impl Criteria {
    fn current() -> Self {
        Self {
            flaky: "0 < pass rate < 1 across the runs".to_string(),
            timeout_sensitive: format!(
                "max/mean > {VARIANCE_RATIO_THRESHOLD:.1}x and mean > {HIGH_VARIANCE_MIN_MEAN_MS:.0}ms"
            ),
            consistently_slow: format!("mean >= {CONSISTENTLY_SLOW_MS:.0}ms at any variance"),
        }
    }

    /// One line for the human reports.
    fn describe(&self) -> String {
        format!(
            "flaky: {}; timeout-sensitive: {}; consistently-slow: {}",
            self.flaky, self.timeout_sensitive, self.consistently_slow
        )
    }
}

/// Full stability analysis result
#[derive(Debug, serde::Serialize)]
struct StabilityAnalysis {
    total_tests: usize,
    runs: usize,
    stable_count: usize,
    flaky_count: usize,
    /// `None` when no per-test timing was available, so timeout-sensitivity was
    /// not evaluated at all. A `0` here means "evaluated, none found".
    ///
    /// Counts both duration risks: high variance and consistent slowness. Each
    /// row's `classification` says which one it is.
    timeout_sensitive_count: Option<usize>,
    /// How many of `timeout_sensitive_count` are reliably slow rather than
    /// variable. `None` for the same reason as above.
    consistently_slow_count: Option<usize>,
    flaky_tests: Vec<TestResult>,
    timeout_sensitive_tests: Vec<TestResult>,
    total_duration_secs: f64,
    timing: Timing,
    /// The thresholds above, so the report states what it looked for.
    criteria: Criteria,
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
    use crate::cli::enums::OutputFormat;

    // The banner is part of the human report. Printing it above YAML, CSV or
    // JUnit would corrupt the document the caller asked for.
    let human_banner = matches!(format, OutputFormat::Table | OutputFormat::Text);

    if human_banner {
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
    }

    let start = Instant::now();
    let analysis = run_stability_analysis(path, runs, filter)?;
    let total_time = start.elapsed();

    let analysis = StabilityAnalysis {
        total_duration_secs: total_time.as_secs_f64(),
        ..analysis
    };

    let formatted = render(&analysis, *format)?;

    if let Some(output_path) = output {
        std::fs::write(output_path, &formatted)?;
        eprintln!(
            "{} Written to: {}",
            c::pass("✔"),
            c::path(&output_path.display().to_string())
        );
    } else {
        println!("{formatted}");
    }

    Ok(())
}

/// Render the analysis in the requested format.
///
/// Every advertised `-f` value gets an arm. Eight of the nine used to fall
/// through a `_ =>` into the colourised human table, so a CI job asking for
/// `junit` or `csv` received ANSI escape sequences and exit 0 (#951).
fn render(analysis: &StabilityAnalysis, format: crate::cli::enums::OutputFormat) -> Result<String> {
    use crate::cli::enums::OutputFormat;
    Ok(match format {
        OutputFormat::Json => serde_json::to_string_pretty(analysis)?,
        OutputFormat::Yaml => serde_yaml_ng::to_string(analysis)?,
        OutputFormat::Csv => format_csv(analysis),
        OutputFormat::Markdown => format_markdown(analysis),
        OutputFormat::Junit => format_junit(analysis),
        OutputFormat::Summary => format_summary(analysis),
        OutputFormat::Plain => format_plain(analysis),
        OutputFormat::Text => format_text(analysis),
        OutputFormat::Table => format_table(analysis),
    })
}

/// Statistics over one test's durations across the runs.
struct DurationStats {
    durations: Vec<f64>,
    mean: Option<f64>,
    p95: Option<f64>,
    max: Option<f64>,
    variance_ratio: Option<f64>,
}

/// Summarise a test's per-run durations.
///
/// A test is only summarised if EVERY run reported a duration for it; a partial
/// series would make the mean depend on which runs happened to be timed.
fn summarize_durations(samples: &[Option<f64>]) -> DurationStats {
    let durations: Vec<f64> = samples.iter().filter_map(|d| *d).collect();
    if durations.is_empty() || durations.len() != samples.len() {
        return DurationStats {
            durations: Vec::new(),
            mean: None,
            p95: None,
            max: None,
            variance_ratio: None,
        };
    }

    let mean = durations.iter().sum::<f64>() / durations.len() as f64;
    let max = durations.iter().cloned().fold(f64::MIN, f64::max);

    let mut sorted = durations.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let p95_idx = ((sorted.len() as f64 * 0.95) as usize).min(sorted.len() - 1);
    let p95 = sorted[p95_idx];

    let variance_ratio = if mean > 0.0 { max / mean } else { 1.0 };

    DurationStats {
        durations,
        mean: Some(mean),
        p95: Some(p95),
        max: Some(max),
        variance_ratio: Some(variance_ratio),
    }
}

/// Run the stability analysis
fn run_stability_analysis(
    path: &Path,
    runs: usize,
    filter: Option<&str>,
) -> Result<StabilityAnalysis> {
    use crate::cli::colors as c;

    let runner = detect_runner();
    let timing = runner.timing();

    // Collect per-test results across runs
    let mut test_results: HashMap<String, Vec<(bool, Option<f64>)>> = HashMap::new();

    for run in 1..=runs {
        eprintln!("  {}Run {}/{}{}...", c::BOLD, run, runs, c::RESET,);

        let results = run_test_suite(path, filter, runner)?;
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

        let samples: Vec<Option<f64>> = results.iter().map(|(_, d)| *d).collect();
        let stats = summarize_durations(&samples);

        let is_flaky = pass_rate < 1.0 && pass_rate > 0.0;
        // Timeout risk is a claim about per-test duration. Without one it is not
        // "false", it is unevaluated.
        let risk = timeout_risk(&stats);

        let (classification, recommendation) = if is_flaky {
            (
                "Flaky".to_string(),
                format!(
                    "Pass rate: {:.0}%. Consider adding retry logic or investigating non-determinism",
                    pass_rate * 100.0
                ),
            )
        } else {
            match risk {
                Some(TimeoutRisk::HighVariance) => (
                    TimeoutRisk::HighVariance.classification().to_string(),
                    format!(
                        "High variance ({:.1}x). Recommend adaptive timeout: {:.0}ms (2x P95)",
                        stats.variance_ratio.unwrap_or(1.0),
                        stats.p95.unwrap_or(0.0) * 2.0
                    ),
                ),
                Some(TimeoutRisk::ConsistentlySlow) => (
                    TimeoutRisk::ConsistentlySlow.classification().to_string(),
                    format!(
                        "Reliably slow: {:.0}ms mean at {:.1}x variance. Low variance is not \
                         safety — this test spends the timeout budget on every run. Recommend \
                         a timeout above {:.0}ms (2x max), or splitting the test",
                        stats.mean.unwrap_or(0.0),
                        stats.variance_ratio.unwrap_or(1.0),
                        stats.max.unwrap_or(0.0) * 2.0
                    ),
                ),
                None => ("Stable".to_string(), String::new()),
            }
        };

        let test_result = TestResult {
            name: name.clone(),
            pass_count,
            fail_count,
            pass_rate,
            durations_ms: stats.durations,
            mean_ms: stats.mean,
            p95_ms: stats.p95,
            max_ms: stats.max,
            variance_ratio: stats.variance_ratio,
            classification,
            recommendation,
        };

        if is_flaky {
            flaky_tests.push(test_result);
        } else if risk.is_some() {
            timeout_sensitive_tests.push(test_result);
        }
    }

    // Sort by severity, then by name so two runs over the same findings agree.
    flaky_tests.sort_by(|a, b| {
        a.pass_rate
            .partial_cmp(&b.pass_rate)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.name.cmp(&b.name))
    });
    timeout_sensitive_tests.sort_by(|a, b| {
        b.variance_ratio
            .partial_cmp(&a.variance_ratio)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.name.cmp(&b.name))
    });

    let timeout_sensitive_count = if timing.is_measured() {
        Some(timeout_sensitive_tests.len())
    } else {
        None
    };
    let consistently_slow_count = if timing.is_measured() {
        Some(
            timeout_sensitive_tests
                .iter()
                .filter(|t| t.classification == TimeoutRisk::ConsistentlySlow.classification())
                .count(),
        )
    } else {
        None
    };
    let stable_count = total_tests
        .saturating_sub(flaky_tests.len())
        .saturating_sub(timeout_sensitive_count.unwrap_or(0));

    Ok(StabilityAnalysis {
        total_tests,
        runs,
        stable_count,
        flaky_count: flaky_tests.len(),
        timeout_sensitive_count,
        consistently_slow_count,
        flaky_tests,
        timeout_sensitive_tests,
        total_duration_secs: 0.0, // filled in by caller
        timing,
        criteria: Criteria::current(),
    })
}

/// Last few lines of a command's stderr, for an error message.
fn stderr_tail(stderr: &str, lines: usize) -> String {
    let all: Vec<&str> = stderr.lines().filter(|l| !l.trim().is_empty()).collect();
    let start = all.len().saturating_sub(lines);
    all[start..].join("\n")
}

/// Did the runner get far enough to execute (or filter) tests?
///
/// This separates "the suite does not compile" from "the suite compiled and no
/// test matched", which need different errors.
fn suite_executed(stdout: &str, stderr: &str, runner: Runner) -> bool {
    match runner {
        // nextest announces the plan before running anything.
        Runner::Nextest => stderr.contains("tests across") || stderr.contains("test across"),
        // libtest prints a result line even for a zero-test run.
        Runner::CargoTest => stdout.contains("test result:") || stderr.contains("test result:"),
    }
}

/// Run the test suite once and parse results.
///
/// Returns `(name, passed, duration_ms)`; the duration is `None` when the
/// runner does not report one.
fn run_test_suite(
    path: &Path,
    filter: Option<&str>,
    runner: Runner,
) -> Result<Vec<(String, bool, Option<f64>)>> {
    let mut args: Vec<String> = match runner {
        Runner::Nextest => vec![
            "nextest".to_string(),
            "run".to_string(),
            "--lib".to_string(),
            "--no-fail-fast".to_string(),
            "--test-threads".to_string(),
            "4".to_string(),
        ],
        Runner::CargoTest => vec!["test".to_string(), "--lib".to_string()],
    };

    if let Some(f) = filter {
        args.push(f.to_string());
    }

    if runner == Runner::CargoTest {
        args.push("--".to_string());
        args.push("--test-threads=4".to_string());
    }

    let output = std::process::Command::new("cargo")
        .args(&args)
        .current_dir(path)
        .env("RUST_MIN_STACK", "8388608")
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let mut results = Vec::new();

    match runner {
        Runner::Nextest => {
            // nextest writes its per-test lines to stderr; stdout carries the
            // captured output of failing tests, which must not be re-parsed.
            parse_nextest_output(&stderr, &mut results);
        }
        Runner::CargoTest => {
            parse_text_test_output(&stdout, &mut results);
            // Also check stderr (cargo test puts some output there)
            if results.is_empty() {
                parse_text_test_output(&stderr, &mut results);
            }
        }
    }

    // A run that measured nothing is not a clean run. `Stable: 0, Flaky: 0`
    // with exit 0 used to be the answer for a crate whose test suite does not
    // compile, and for a `--filter` that matches nothing (#952).
    if results.is_empty() {
        if !suite_executed(&stdout, &stderr, runner) {
            bail!(
                "the test suite in {} did not build or run (cargo exited with {}), \
                 so nothing could be measured:\n{}",
                path.display(),
                output.status,
                stderr_tail(&stderr, 20)
            );
        }
        if let Some(f) = filter {
            bail!(
                "--filter '{f}' matched no tests in {}: there is nothing to measure \
                 the stability of. Check the filter, or drop it to analyse the whole suite.",
                path.display()
            );
        }
    }

    Ok(results)
}

/// Parse `cargo nextest run` output.
///
/// Lines look like `    PASS [   0.004s] (1/2) my-crate tests::my_test`; the
/// failure recap at the end repeats them, so the first sighting of a test wins.
fn parse_nextest_output(output: &str, results: &mut Vec<(String, bool, Option<f64>)>) {
    let mut seen: std::collections::HashSet<String> = results.iter().map(|r| r.0.clone()).collect();

    for line in output.lines() {
        let line = strip_ansi_owned(line);
        let line = line.trim();
        let passed = if line.starts_with("PASS ") {
            true
        } else if line.starts_with("FAIL ") {
            false
        } else {
            continue;
        };

        let Some(open) = line.find('[') else { continue };
        let Some(close) = line[open..].find(']').map(|i| i + open) else {
            continue;
        };
        // `SLOW [>60.000s]` is a progress notice with no final duration.
        let secs_text = line[open + 1..close].trim().trim_end_matches('s');
        let Ok(secs) = secs_text.parse::<f64>() else {
            continue;
        };

        // After `]`: an optional `(1/2)` progress counter, the binary id, then
        // the test name — which never contains whitespace.
        let Some(name) = line[close + 1..].split_whitespace().next_back() else {
            continue;
        };
        if seen.insert(name.to_string()) {
            results.push((name.to_string(), passed, Some(secs * 1000.0)));
        }
    }
}

fn strip_ansi_owned(line: &str) -> String {
    crate::cli::verify::strip_ansi(line)
}

/// Parser for `cargo test` text output.
///
/// Stable libtest reports no timing, so the duration is `None` — never an
/// estimate derived from the whole-suite wall clock.
fn parse_text_test_output(output: &str, results: &mut Vec<(String, bool, Option<f64>)>) {
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
            results.push((name, passed, None));
        }
    }
}

// ── Renderers ───────────────────────────────────────────────────────────────

fn opt_ms(value: Option<f64>) -> String {
    value.map_or_else(|| "n/a".to_string(), |v| format!("{v:.0}ms"))
}

fn opt_cell(value: Option<f64>) -> String {
    value.map_or_else(String::new, |v| format!("{v:.3}"))
}

fn reported_tests(analysis: &StabilityAnalysis) -> impl Iterator<Item = &TestResult> {
    analysis
        .flaky_tests
        .iter()
        .chain(analysis.timeout_sensitive_tests.iter())
}

/// Counts only, one `key=value` per line.
fn format_summary(analysis: &StabilityAnalysis) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    let _ = writeln!(out, "total_tests={}", analysis.total_tests);
    let _ = writeln!(out, "runs={}", analysis.runs);
    let _ = writeln!(out, "stable={}", analysis.stable_count);
    let _ = writeln!(out, "flaky={}", analysis.flaky_count);
    let _ = writeln!(
        out,
        "timeout_sensitive={}",
        analysis
            .timeout_sensitive_count
            .map_or_else(|| "not_measured".to_string(), |c| c.to_string())
    );
    let _ = writeln!(
        out,
        "consistently_slow={}",
        analysis
            .consistently_slow_count
            .map_or_else(|| "not_measured".to_string(), |c| c.to_string())
    );
    let _ = writeln!(out, "timing={}", analysis.timing.describe());
    let _ = writeln!(out, "criteria={}", analysis.criteria.describe());
    let _ = writeln!(out, "duration_secs={:.3}", analysis.total_duration_secs);
    out
}

/// One tab-separated line per reported test, no decoration and no colour.
fn format_plain(analysis: &StabilityAnalysis) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    for test in reported_tests(analysis) {
        let _ = writeln!(
            out,
            "{}\t{}\t{:.2}\t{}\t{}",
            test.classification,
            test.name,
            test.pass_rate,
            opt_cell(test.mean_ms),
            opt_cell(test.variance_ratio),
        );
    }
    if out.is_empty() {
        let _ = writeln!(out, "# no flaky or timeout-sensitive tests");
    }
    out
}

/// Column-aligned table of the reported tests (the `-f table` default).
///
/// `-f text` renders the same findings as prose; the two must not be the same
/// document, or one of them is an advertised format that does nothing.
fn format_table(analysis: &StabilityAnalysis) -> String {
    use crate::cli::colors as c;
    use std::fmt::Write;

    let mut out = String::new();
    let _ = writeln!(
        out,
        "  {}Tests:{} {}   {}Runs:{} {}   {}Stable:{} {}   {}Flaky:{} {}   \
         {}Timeout-sensitive:{} {}   {}Timing:{} {}",
        c::BOLD,
        c::RESET,
        analysis.total_tests,
        c::BOLD,
        c::RESET,
        analysis.runs,
        c::BOLD,
        c::RESET,
        analysis.stable_count,
        c::BOLD,
        c::RESET,
        analysis.flaky_count,
        c::BOLD,
        c::RESET,
        analysis
            .timeout_sensitive_count
            .map_or_else(|| "not measured".to_string(), |n| n.to_string()),
        c::BOLD,
        c::RESET,
        analysis.timing.describe(),
    );
    // Stated, not implied: a `0` above is only readable against the rules that
    // produced it (#950).
    let _ = writeln!(
        out,
        "  {}",
        c::dim(&format!("Criteria — {}", analysis.criteria.describe()))
    );
    let _ = writeln!(out);

    let rows: Vec<&TestResult> = reported_tests(analysis).collect();
    if rows.is_empty() {
        let _ = writeln!(
            out,
            "  {}",
            if analysis.total_tests == 0 {
                c::warn("No tests were found to analyse")
            } else {
                c::pass("No flaky or timeout-sensitive tests")
            }
        );
        return out;
    }

    let name_width = rows.iter().map(|t| t.name.len()).max().unwrap_or(4).max(4);
    let _ = writeln!(
        out,
        "  {:<name_width$}  {:<17}  {:>9}  {:>10}  {:>10}  {:>8}",
        "TEST", "CLASSIFICATION", "PASS RATE", "MEAN", "P95", "VAR"
    );
    let _ = writeln!(
        out,
        "  {}  {}  {}  {}  {}  {}",
        "-".repeat(name_width),
        "-".repeat(17),
        "-".repeat(9),
        "-".repeat(10),
        "-".repeat(10),
        "-".repeat(8),
    );
    for test in rows {
        let _ = writeln!(
            out,
            "  {:<name_width$}  {:<17}  {:>8.0}%  {:>10}  {:>10}  {:>8}",
            test.name,
            test.classification,
            test.pass_rate * 100.0,
            opt_ms(test.mean_ms),
            opt_ms(test.p95_ms),
            test.variance_ratio
                .map_or_else(|| "n/a".to_string(), |v| format!("{v:.2}x")),
        );
    }
    out
}

fn format_csv(analysis: &StabilityAnalysis) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    let _ = writeln!(
        out,
        "name,classification,runs,pass_count,fail_count,pass_rate,mean_ms,p95_ms,max_ms,variance_ratio"
    );
    for test in reported_tests(analysis) {
        let _ = writeln!(
            out,
            "\"{}\",{},{},{},{},{:.4},{},{},{},{}",
            test.name.replace('"', "\"\""),
            test.classification,
            analysis.runs,
            test.pass_count,
            test.fail_count,
            test.pass_rate,
            opt_cell(test.mean_ms),
            opt_cell(test.p95_ms),
            opt_cell(test.max_ms),
            opt_cell(test.variance_ratio),
        );
    }
    out
}

fn format_markdown(analysis: &StabilityAnalysis) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    let _ = writeln!(out, "# Test Stability Analysis\n");
    let _ = writeln!(out, "| Metric | Value |");
    let _ = writeln!(out, "|---|---|");
    let _ = writeln!(out, "| Total tests | {} |", analysis.total_tests);
    let _ = writeln!(out, "| Runs | {} |", analysis.runs);
    let _ = writeln!(out, "| Stable | {} |", analysis.stable_count);
    let _ = writeln!(out, "| Flaky | {} |", analysis.flaky_count);
    let _ = writeln!(
        out,
        "| Timeout-sensitive | {} |",
        analysis
            .timeout_sensitive_count
            .map_or_else(|| "not measured".to_string(), |c| c.to_string())
    );
    let _ = writeln!(
        out,
        "| Consistently slow | {} |",
        analysis
            .consistently_slow_count
            .map_or_else(|| "not measured".to_string(), |c| c.to_string())
    );
    let _ = writeln!(out, "| Per-test timing | {} |", analysis.timing.describe());
    let _ = writeln!(out, "| Duration | {:.1}s |", analysis.total_duration_secs);
    let _ = writeln!(out, "| Criteria | {} |\n", analysis.criteria.describe());

    if !analysis.flaky_tests.is_empty() {
        let _ = writeln!(out, "## Flaky Tests\n");
        let _ = writeln!(out, "| Test | Pass rate | Mean | Max |");
        let _ = writeln!(out, "|---|---|---|---|");
        for test in &analysis.flaky_tests {
            let _ = writeln!(
                out,
                "| `{}` | {:.0}% | {} | {} |",
                test.name,
                test.pass_rate * 100.0,
                opt_ms(test.mean_ms),
                opt_ms(test.max_ms),
            );
        }
        let _ = writeln!(out);
    }

    if !analysis.timeout_sensitive_tests.is_empty() {
        let _ = writeln!(out, "## Timeout-Sensitive and Consistently-Slow Tests\n");
        let _ = writeln!(
            out,
            "| Test | Classification | Variance | Mean | P95 | Max |"
        );
        let _ = writeln!(out, "|---|---|---|---|---|---|");
        for test in &analysis.timeout_sensitive_tests {
            let _ = writeln!(
                out,
                "| `{}` | {} | {} | {} | {} | {} |",
                test.name,
                test.classification,
                test.variance_ratio
                    .map_or_else(|| "n/a".to_string(), |v| format!("{v:.1}x")),
                opt_ms(test.mean_ms),
                opt_ms(test.p95_ms),
                opt_ms(test.max_ms),
            );
        }
        let _ = writeln!(out);
    }

    if analysis.flaky_tests.is_empty() && analysis.timeout_sensitive_tests.is_empty() {
        let _ = writeln!(out, "No flaky or timeout-sensitive tests found.");
    }

    out
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// JUnit XML: every flaky or timeout-sensitive test is a failing testcase, so a
/// CI job can gate on it.
fn format_junit(analysis: &StabilityAnalysis) -> String {
    use std::fmt::Write;
    let failures = analysis.flaky_tests.len() + analysis.timeout_sensitive_tests.len();
    let mut out = String::new();
    let _ = writeln!(out, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
    let _ = writeln!(
        out,
        "<testsuites name=\"Test Stability\" tests=\"{}\" failures=\"{failures}\" time=\"{:.3}\">",
        analysis.total_tests, analysis.total_duration_secs
    );
    let _ = writeln!(
        out,
        "  <testsuite name=\"stability\" tests=\"{}\" failures=\"{failures}\">",
        analysis.total_tests
    );
    let _ = writeln!(
        out,
        "    <properties>\n      <property name=\"runs\" value=\"{}\"/>\n      \
         <property name=\"timing\" value=\"{}\"/>\n    </properties>",
        analysis.runs,
        xml_escape(&analysis.timing.describe())
    );
    for test in reported_tests(analysis) {
        let _ = writeln!(
            out,
            "    <testcase name=\"{}\" classname=\"{}\">",
            xml_escape(&test.name),
            xml_escape(&test.classification)
        );
        let _ = writeln!(
            out,
            "      <failure message=\"{}\">{}</failure>",
            xml_escape(&test.classification),
            xml_escape(&test.recommendation)
        );
        let _ = writeln!(out, "    </testcase>");
    }
    let _ = writeln!(out, "  </testsuite>");
    let _ = writeln!(out, "</testsuites>");
    out
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
    match analysis.timeout_sensitive_count {
        Some(count) if count > 0 => {
            let _ = writeln!(
                out,
                "  {}Timeout-sensitive:{} {}{}{}",
                c::BOLD,
                c::RESET,
                c::YELLOW,
                count,
                c::RESET,
            );
        }
        Some(_) => {}
        None => {
            let _ = writeln!(
                out,
                "  {}Timeout-sensitive:{} {}",
                c::BOLD,
                c::RESET,
                c::dim(&analysis.timing.describe()),
            );
        }
    }
    if let Some(count) = analysis.consistently_slow_count {
        if count > 0 {
            let _ = writeln!(
                out,
                "  {}  of which consistently slow:{} {}{}{}",
                c::BOLD,
                c::RESET,
                c::YELLOW,
                count,
                c::RESET,
            );
        }
    }
    let _ = writeln!(
        out,
        "  {}Duration:{} {:.1}s",
        c::BOLD,
        c::RESET,
        analysis.total_duration_secs,
    );
    // #950: the counts above are only readable next to the rules that produced
    // them, so the rules are printed rather than left to the reader to guess.
    let _ = writeln!(
        out,
        "  {}\n",
        c::dim(&format!("Criteria — {}", analysis.criteria.describe()))
    );

    if !analysis.flaky_tests.is_empty() {
        let _ = writeln!(out, "{}\n", c::subheader("Flaky Tests"));
        for test in &analysis.flaky_tests {
            let _ = writeln!(out, "  {} {}", c::fail("✘"), c::label(&test.name),);
            let _ = writeln!(
                out,
                "     Pass rate: {}{:.0}%{}  Mean: {}  Max: {}",
                c::RED,
                test.pass_rate * 100.0,
                c::RESET,
                opt_ms(test.mean_ms),
                opt_ms(test.max_ms),
            );
            let _ = writeln!(out, "     {}", c::dim(&test.recommendation));
            let _ = writeln!(out);
        }
    }

    if !analysis.timeout_sensitive_tests.is_empty() {
        let _ = writeln!(
            out,
            "{}\n",
            c::subheader("Timeout-Sensitive and Consistently-Slow Tests")
        );
        for test in &analysis.timeout_sensitive_tests {
            let _ = writeln!(
                out,
                "  {} {} ({})",
                c::warn("!"),
                c::label(&test.name),
                test.classification
            );
            let _ = writeln!(
                out,
                "     Variance: {}{}{}  Mean: {}  P95: {}  Max: {}",
                c::YELLOW,
                test.variance_ratio
                    .map_or_else(|| "n/a".to_string(), |v| format!("{v:.1}x")),
                c::RESET,
                opt_ms(test.mean_ms),
                opt_ms(test.p95_ms),
                opt_ms(test.max_ms),
            );
            let _ = writeln!(out, "     {}", c::dim(&test.recommendation));
            let _ = writeln!(out);
        }
    }

    if analysis.total_tests == 0 {
        let _ = writeln!(out, "  {}", c::warn("No tests were found to analyse"));
    } else if analysis.flaky_count == 0 && analysis.timeout_sensitive_count.unwrap_or(0) == 0 {
        let _ = writeln!(out, "  {}", c::pass("All tests are stable"));
    }

    out
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::enums::OutputFormat;

    fn analysis_with(flaky: Vec<TestResult>, timing: Timing) -> StabilityAnalysis {
        let timeout_sensitive_count = if timing.is_measured() { Some(0) } else { None };
        StabilityAnalysis {
            total_tests: 100,
            runs: 3,
            stable_count: 100 - flaky.len(),
            flaky_count: flaky.len(),
            timeout_sensitive_count,
            consistently_slow_count: timeout_sensitive_count,
            flaky_tests: flaky,
            timeout_sensitive_tests: vec![],
            total_duration_secs: 10.0,
            timing,
            criteria: Criteria::current(),
        }
    }

    fn measured() -> Timing {
        Timing::Measured {
            runner: "cargo-nextest".to_string(),
        }
    }

    fn unmeasured() -> Timing {
        Timing::NotMeasured {
            reason: "cargo test reports no per-test timing".to_string(),
        }
    }

    fn flaky_test(name: &str) -> TestResult {
        TestResult {
            name: name.to_string(),
            pass_count: 2,
            fail_count: 1,
            pass_rate: 0.67,
            durations_ms: vec![100.0, 200.0, 150.0],
            mean_ms: Some(150.0),
            p95_ms: Some(200.0),
            max_ms: Some(200.0),
            variance_ratio: Some(1.33),
            classification: "Flaky".to_string(),
            recommendation: "Investigate non-determinism".to_string(),
        }
    }

    fn make_timeout_sensitive_test(name: &str) -> TestResult {
        TestResult {
            name: name.to_string(),
            pass_count: 3,
            fail_count: 0,
            pass_rate: 1.0,
            durations_ms: vec![50.0, 200.0, 600.0],
            mean_ms: Some(283.3),
            p95_ms: Some(580.0),
            max_ms: Some(600.0),
            variance_ratio: Some(2.12),
            classification: "Timeout-Sensitive".to_string(),
            recommendation: "Recommend adaptive timeout: 1160ms (2x P95)".to_string(),
        }
    }

    // ── #950: durations are measured or absent, never manufactured ──────────

    #[test]
    fn cargo_test_output_carries_no_duration() {
        // PIN: stable libtest reports no timing, so the parser must yield None
        // rather than a number for the caller to fill in from the wall clock.
        let mut results = Vec::new();
        parse_text_test_output("test my_test ... ok", &mut results);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "my_test");
        assert!(results[0].1);
        assert_eq!(results[0].2, None, "text mode must not invent a duration");
    }

    #[test]
    fn unmeasured_durations_do_not_become_statistics() {
        // Two runs, neither timed: every statistic stays absent and the empty
        // duration series is empty — not `[elapsed/N, elapsed/N]`.
        let stats = summarize_durations(&[None, None]);
        assert!(stats.durations.is_empty());
        assert_eq!(stats.mean, None);
        assert_eq!(stats.p95, None);
        assert_eq!(stats.max, None);
        assert_eq!(stats.variance_ratio, None);
    }

    #[test]
    fn a_partially_timed_series_is_not_averaged() {
        // Mixing a timed run with an untimed one would make the mean depend on
        // which runs happened to be measured.
        let stats = summarize_durations(&[Some(10.0), None]);
        assert_eq!(stats.mean, None);
        assert!(stats.durations.is_empty());
    }

    #[test]
    fn measured_durations_produce_real_statistics() {
        let stats = summarize_durations(&[Some(50.0), Some(200.0), Some(600.0)]);
        assert_eq!(stats.durations, vec![50.0, 200.0, 600.0]);
        assert!((stats.mean.unwrap() - 283.333).abs() < 0.01);
        assert_eq!(stats.max, Some(600.0));
        assert_eq!(stats.p95, Some(600.0));
        assert!((stats.variance_ratio.unwrap() - 2.117).abs() < 0.01);
    }

    #[test]
    fn nextest_output_yields_a_real_per_test_duration() {
        // The exact shape `cargo nextest run` prints, including the failure
        // recap that repeats the FAIL line.
        let output = "        FAIL [   0.004s] (1/2) durctl tests::test_instant_but_flaky\n\
                      \x20       PASS [  10.004s] (2/2) durctl tests::test_ten_second_sleep\n\
                      \x20    Summary [  10.004s] 2 tests run: 1 passed, 1 failed, 0 skipped\n\
                      \x20       FAIL [   0.004s] (1/2) durctl tests::test_instant_but_flaky\n";
        let mut results = Vec::new();
        parse_nextest_output(output, &mut results);
        assert_eq!(results.len(), 2, "the failure recap must not double-count");
        assert_eq!(results[0].0, "tests::test_instant_but_flaky");
        assert!(!results[0].1);
        assert!((results[0].2.unwrap() - 4.0).abs() < 0.001);
        assert_eq!(results[1].0, "tests::test_ten_second_sleep");
        assert!(results[1].1);
        // The 10-second test really is reported at ~10s, not at suite/N.
        assert!((results[1].2.unwrap() - 10004.0).abs() < 1.0);
        assert!(
            results[1].2.unwrap() > results[0].2.unwrap() * 1000.0,
            "a 10s test and a 4ms test must not share one number"
        );
    }

    #[test]
    fn nextest_progress_notices_are_not_results() {
        // `SLOW [>60.000s]` has no final duration; it is a progress line.
        let mut results = Vec::new();
        parse_nextest_output(
            "     SLOW [>60.000s] durctl tests::slow_one\n",
            &mut results,
        );
        assert!(results.is_empty());
    }

    #[test]
    fn timeout_sensitivity_is_unmeasured_not_zero_without_timings() {
        let analysis = analysis_with(vec![], unmeasured());
        assert_eq!(analysis.timeout_sensitive_count, None);
        let json = serde_json::to_string(&analysis).unwrap();
        assert!(json.contains("\"timeout_sensitive_count\":null"));
        assert!(json.contains("\"status\":\"not_measured\""));
        // And the human report says so rather than claiming all-clear.
        let text = crate::cli::verify::strip_ansi(&format_text(&analysis));
        assert!(
            text.contains("not measured"),
            "human report hid the absence: {text}"
        );
    }

    // ── #951: every advertised -f value renders its own format ──────────────

    #[test]
    fn every_advertised_format_renders_distinctly() {
        let analysis = analysis_with(vec![flaky_test("tests::flaky_one")], measured());
        let formats = [
            OutputFormat::Table,
            OutputFormat::Json,
            OutputFormat::Yaml,
            OutputFormat::Markdown,
            OutputFormat::Csv,
            OutputFormat::Summary,
            OutputFormat::Text,
            OutputFormat::Plain,
            OutputFormat::Junit,
        ];
        let mut rendered: Vec<(OutputFormat, String)> = Vec::new();
        for f in formats {
            rendered.push((f, render(&analysis, f).expect("render")));
        }
        for (fa, a) in &rendered {
            for (fb, b) in &rendered {
                if fa != fb {
                    assert_ne!(a, b, "{fa} and {fb} render identically");
                }
            }
        }
    }

    #[test]
    fn json_and_yaml_and_junit_and_csv_are_machine_parseable() {
        let analysis = analysis_with(vec![flaky_test("tests::flaky_one")], measured());

        let json = render(&analysis, OutputFormat::Json).unwrap();
        let _: serde_json::Value = serde_json::from_str(&json).expect("json must parse");

        let yaml = render(&analysis, OutputFormat::Yaml).unwrap();
        let value: serde_yaml_ng::Value = serde_yaml_ng::from_str(&yaml).expect("yaml must parse");
        assert_eq!(value["flaky_count"].as_u64(), Some(1));

        let junit = render(&analysis, OutputFormat::Junit).unwrap();
        assert!(junit.starts_with("<?xml version=\"1.0\""));
        assert!(junit.contains("<testsuites"));
        assert!(junit.contains("tests::flaky_one"));

        let csv = render(&analysis, OutputFormat::Csv).unwrap();
        let mut lines = csv.lines();
        assert_eq!(
            lines.next().unwrap(),
            "name,classification,runs,pass_count,fail_count,pass_rate,mean_ms,p95_ms,max_ms,variance_ratio"
        );
        assert_eq!(lines.next().unwrap().split(',').count(), 10);
    }

    #[test]
    fn machine_formats_carry_no_ansi_escapes() {
        let analysis = analysis_with(vec![flaky_test("tests::flaky_one")], measured());
        for f in [
            OutputFormat::Json,
            OutputFormat::Yaml,
            OutputFormat::Csv,
            OutputFormat::Markdown,
            OutputFormat::Junit,
            OutputFormat::Summary,
            OutputFormat::Plain,
        ] {
            let out = render(&analysis, f).unwrap();
            assert!(
                !out.contains('\u{1b}'),
                "{f} leaked an ANSI escape into a machine format"
            );
        }
    }

    #[test]
    fn the_table_and_the_prose_report_are_different_documents() {
        // Both are human formats, so both may carry colour; with colour off
        // (the piped case, which is how CI sees them) they must still differ,
        // or one of the two advertised formats does nothing.
        let analysis = analysis_with(vec![flaky_test("tests::flaky_one")], measured());
        let table = crate::cli::verify::strip_ansi(&format_table(&analysis));
        let text = crate::cli::verify::strip_ansi(&format_text(&analysis));
        assert_ne!(table, text);
        assert!(
            table.contains("CLASSIFICATION"),
            "table has columns: {table}"
        );
        assert!(text.contains("Flaky Tests"), "text has sections: {text}");
    }

    // ── #952: a run that measured nothing is not a pass ─────────────────────

    #[test]
    fn a_suite_that_did_not_build_is_distinguishable_from_a_zero_test_run() {
        // libtest prints a result line even for a zero-test run; a compile
        // failure never gets that far.
        assert!(suite_executed(
            "running 0 tests\n\ntest result: ok. 0 passed; 0 failed;\n",
            "",
            Runner::CargoTest
        ));
        assert!(!suite_executed(
            "",
            "error[E0425]: cannot find value\nerror: could not compile\n",
            Runner::CargoTest
        ));
        assert!(suite_executed(
            "",
            "    Starting 2 tests across 1 binary\n",
            Runner::Nextest
        ));
        assert!(!suite_executed(
            "",
            "error: could not compile `brokenctl`\n",
            Runner::Nextest
        ));
    }

    #[test]
    fn a_broken_test_suite_is_an_error_not_a_clean_zero() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname=\"brokenctl\"\nversion=\"0.1.0\"\nedition=\"2021\"\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("src/lib.rs"),
            "pub fn add(a:i32,b:i32)->i32{ this is not rust }\n",
        )
        .unwrap();

        let err = run_test_suite(dir.path(), None, Runner::CargoTest)
            .expect_err("a suite that does not compile must not report a clean zero");
        let msg = err.to_string();
        assert!(
            msg.contains("did not build or run"),
            "error must name the cause: {msg}"
        );
        assert!(
            msg.contains("error"),
            "error must carry cargo's stderr: {msg}"
        );
    }

    #[test]
    fn a_filter_that_matches_nothing_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname=\"filterctl\"\nversion=\"0.1.0\"\nedition=\"2021\"\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("src/lib.rs"),
            "pub fn add(a:i32,b:i32)->i32{a+b}\n#[cfg(test)]\nmod tests{\n#[test]\nfn t(){assert_eq!(super::add(1,1),2);}\n}\n",
        )
        .unwrap();

        // Sanity: the crate really does have a runnable test.
        let ok = run_test_suite(dir.path(), None, Runner::CargoTest).expect("suite runs");
        assert_eq!(ok.len(), 1, "fixture must have exactly one test");

        let err = run_test_suite(dir.path(), Some("nosuchtestname"), Runner::CargoTest)
            .expect_err("a filter matching zero tests must not report a clean zero");
        assert!(
            err.to_string().contains("matched no tests"),
            "error must name the filter: {err}"
        );
    }

    #[test]
    fn stderr_tail_keeps_the_last_lines_and_drops_blanks() {
        let tail = stderr_tail("a\n\nb\nc\n", 2);
        assert_eq!(tail, "b\nc");
    }

    // ── existing renderer coverage, ported to the Option-shaped statistics ──

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
        let analysis = analysis_with(vec![], measured());
        let text = format_text(&analysis);
        assert!(text.contains("All tests are stable"));
    }

    #[test]
    fn test_format_text_with_flaky() {
        let analysis = analysis_with(vec![flaky_test("test_flaky_one")], measured());
        let text = format_text(&analysis);
        assert!(text.contains("test_flaky_one"));
        assert!(text.contains("Flaky"));
    }

    #[test]
    fn test_format_text_with_timeout_sensitive_only() {
        let mut analysis = analysis_with(vec![], measured());
        analysis.timeout_sensitive_tests = vec![make_timeout_sensitive_test("slow_test")];
        analysis.timeout_sensitive_count = Some(1);
        let text = format_text(&analysis);
        assert!(text.contains("Timeout-Sensitive and Consistently-Slow Tests"));
        assert!(text.contains("slow_test"));
        assert!(text.contains("Variance"));
        // PIN: variance_ratio formatted with 1 decimal + "x" suffix.
        assert!(text.contains("2.1x"));
    }

    #[test]
    fn test_format_text_with_both_flaky_and_timeout_sensitive() {
        let mut analysis = analysis_with(vec![flaky_test("flaky_a")], measured());
        analysis.timeout_sensitive_tests = vec![make_timeout_sensitive_test("slow_b")];
        analysis.timeout_sensitive_count = Some(1);
        let text = format_text(&analysis);
        assert!(text.contains("flaky_a"));
        assert!(text.contains("slow_b"));
        assert!(text.contains("Flaky Tests"));
        assert!(text.contains("Timeout-Sensitive and Consistently-Slow Tests"));
        assert!(!text.contains("All tests are stable"));
    }

    #[test]
    fn test_format_text_summary_includes_run_counts_and_duration() {
        let mut analysis = analysis_with(vec![], measured());
        analysis.total_tests = 250;
        analysis.stable_count = 250;
        analysis.runs = 5;
        analysis.total_duration_secs = 47.3;
        let text = format_text(&analysis);
        assert!(text.contains("Total tests:"));
        assert!(text.contains("250"));
        assert!(text.contains("Stable:"));
        // PIN: Duration uses `.1` decimal format with "s" suffix.
        assert!(text.contains("47.3s"));
        // PIN: stable percent computed as `stable / total.max(1) * 100`.
        assert!(text.contains("100.0%"));
    }

    #[test]
    fn zero_tests_is_reported_as_zero_tests_not_as_all_stable() {
        let mut analysis = analysis_with(vec![], measured());
        analysis.total_tests = 0;
        analysis.stable_count = 0;
        let text = format_text(&analysis);
        assert!(text.contains("No tests were found"));
        assert!(!text.contains("All tests are stable"));
    }

    // ── parse_text_test_output edge cases ───────────────────────────────────

    #[test]
    fn test_parse_text_test_output_empty_input() {
        let mut results = Vec::new();
        parse_text_test_output("", &mut results);
        assert!(results.is_empty());
    }

    #[test]
    fn test_parse_text_test_output_no_test_lines() {
        let mut results = Vec::new();
        parse_text_test_output(
            "running 0 tests\ntest result: ok. 0 passed; 0 failed\n",
            &mut results,
        );
        assert!(results.is_empty());
    }

    #[test]
    fn test_parse_text_test_output_ignored_test_not_collected() {
        // PIN: the parser only matches `... ok` and `... FAILED`; ignored tests
        // (`... ignored`) are NOT collected.
        let mut results = Vec::new();
        parse_text_test_output("test ignored_one ... ignored", &mut results);
        assert!(results.is_empty());
    }

    #[test]
    fn test_parse_text_test_output_test_names_with_dots() {
        let mut results = Vec::new();
        parse_text_test_output("test foo::bar::test_baz ... ok", &mut results);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "foo::bar::test_baz");
        assert!(results[0].1);
    }

    #[test]
    fn test_parse_text_test_output_appends_to_existing_results() {
        // PIN: parse_text_test_output APPENDS to the &mut Vec (does not clear).
        let mut results = vec![("preexisting".to_string(), true, None)];
        parse_text_test_output("test new_test ... ok", &mut results);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, "preexisting");
        assert_eq!(results[1].0, "new_test");
    }

    // ── #950 residual: a CONSISTENTLY slow test was never reported ──────────

    /// RED on the old code: the only duration rule was
    /// `variance_ratio > 2.0 && mean > 100.0`, so a test that takes ten seconds
    /// on EVERY run has variance ratio 1.0, classified `Stable`, and stable
    /// tests are not retained in the report at all.
    #[test]
    fn a_test_that_is_slow_on_every_run_is_reported() {
        let ten_seconds_every_run =
            summarize_durations(&[Some(10_000.0), Some(10_010.0), Some(9_990.0)]);
        assert!(
            ten_seconds_every_run.variance_ratio.unwrap() < VARIANCE_RATIO_THRESHOLD,
            "the fixture must have LOW variance, or it would be caught by the old rule"
        );
        assert_eq!(
            timeout_risk(&ten_seconds_every_run),
            Some(TimeoutRisk::ConsistentlySlow),
            "a reliably 10s test is the one most likely to hit a CI timeout"
        );
    }

    /// The old rule must still fire: a variable test is still timeout-sensitive.
    #[test]
    fn a_high_variance_test_is_still_classified_by_variance() {
        let variable = summarize_durations(&[Some(150.0), Some(1500.0), Some(160.0)]);
        assert_eq!(
            timeout_risk(&variable),
            Some(TimeoutRisk::HighVariance),
            "variance detection must survive the added slow-test rule"
        );
    }

    /// And a fast, steady test must stay out of the report, or the new rule
    /// would flood it.
    #[test]
    fn a_fast_steady_test_is_no_timeout_risk() {
        assert_eq!(
            timeout_risk(&summarize_durations(&[Some(4.0), Some(5.0), Some(4.5)])),
            None
        );
        // Sub-threshold variance on a sub-threshold mean is noise, not risk.
        assert_eq!(
            timeout_risk(&summarize_durations(&[Some(0.1), Some(0.4), Some(0.1)])),
            None
        );
    }

    /// Without timings there is no risk claim to make either way.
    #[test]
    fn unmeasured_durations_yield_no_risk_classification() {
        assert_eq!(timeout_risk(&summarize_durations(&[None, None])), None);
        assert_eq!(
            timeout_risk(&summarize_durations(&[Some(10_000.0), None])),
            None,
            "a partial series must not be summarised into a claim"
        );
    }

    /// The report must state the rules it applied: a bare `timeout_sensitive: 0`
    /// cannot be read without them.
    #[test]
    fn every_format_discloses_the_criteria_it_applied() {
        let analysis = analysis_with(vec![], measured());

        let json: serde_json::Value =
            serde_json::from_str(&render(&analysis, OutputFormat::Json).unwrap()).unwrap();
        assert!(
            json["criteria"]["consistently_slow"]
                .as_str()
                .unwrap()
                .contains("1000"),
            "json must carry the slow threshold: {json}"
        );
        assert_eq!(json["consistently_slow_count"], 0);

        for format in [
            OutputFormat::Text,
            OutputFormat::Table,
            OutputFormat::Markdown,
            OutputFormat::Summary,
        ] {
            let rendered = render(&analysis, format).unwrap();
            assert!(
                rendered.contains("consistently-slow") || rendered.contains("Consistently slow"),
                "{format:?} must disclose the consistently-slow rule:\n{rendered}"
            );
        }
    }

    /// Unmeasured timing must keep saying "not measured" for the new count too,
    /// never `0`.
    #[test]
    fn an_unmeasured_run_reports_the_slow_count_as_not_measured() {
        let analysis = analysis_with(vec![], unmeasured());
        let json: serde_json::Value =
            serde_json::from_str(&render(&analysis, OutputFormat::Json).unwrap()).unwrap();
        assert!(
            json["consistently_slow_count"].is_null(),
            "an unevaluated criterion must be null, never 0: {json}"
        );
        let summary = render(&analysis, OutputFormat::Summary).unwrap();
        assert!(
            summary.contains("consistently_slow=not_measured"),
            "{summary}"
        );
    }
}
