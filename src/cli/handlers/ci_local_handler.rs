//! Local CI simulation handler
//!
//! Runs the same quality gate matrix as GitHub Actions locally
//! to eliminate push-wait-fix loops.

use crate::contracts::OutputFormat;
use anyhow::Result;
use std::path::Path;
use std::time::Instant;

/// CI check result
struct CiCheckResult {
    name: String,
    passed: bool,
    duration: std::time::Duration,
    output: String,
    fix_hint: Option<String>,
}

/// Is this format the human banner, or a machine-readable document?
///
/// `--format` used to be destructured as `format: _` in the dispatcher and
/// never reached this handler, so `-f json` printed the same progress banner as
/// `-f table` and `python3 -m json.tool` failed at character 0.
fn is_human_format(format: OutputFormat) -> bool {
    matches!(
        format,
        OutputFormat::Table | OutputFormat::Text | OutputFormat::Plain
    )
}

/// Run local CI simulation
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_ci_local(
    path: &Path,
    quick: bool,
    matrix: Option<&str>,
    fix: bool,
    verbose: bool,
    format: OutputFormat,
) -> Result<()> {
    use crate::cli::colors as c;

    // A path that does not exist is not a CI failure. Without this check every
    // check spawned into a missing cwd, failed with "No such file or directory
    // (os error 2)", and the run reported 3 failing stages with fix hints
    // ("Run `cargo fmt --all`") that address a problem the user does not have.
    if !path.exists() {
        anyhow::bail!("Path not found: {}", path.display());
    }

    let human = is_human_format(format);

    if human {
        println!("{}\n", c::header("PMAT Local CI Simulation"));
    }

    let checks = build_check_list(quick, matrix)?;
    let total = checks.len();
    let mut results: Vec<CiCheckResult> = Vec::new();

    for (i, check_name) in checks.iter().enumerate() {
        if human {
            print!("  [{}/{}] {} ... ", i + 1, total, c::label(check_name));
        }
        let start = Instant::now();

        let result = run_check(check_name, path, fix, verbose).await;
        let duration = start.elapsed();

        let (passed, output, fix_hint) = match result {
            Ok(out) => (true, out, None),
            Err(e) => {
                let hint = get_fix_hint(check_name);
                (false, e.to_string(), hint)
            }
        };

        if human {
            if passed {
                println!(
                    "{} {}",
                    c::pass(""),
                    c::dim(&format!("({:.1}s)", duration.as_secs_f64()))
                );
            } else {
                println!("{}", c::fail(""));
            }
        }

        if !passed && verbose && human {
            for line in output.lines().take(20) {
                println!("    {}", c::dim(line));
            }
        }

        results.push(CiCheckResult {
            name: check_name.to_string(),
            passed,
            duration,
            output,
            fix_hint,
        });
    }

    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.iter().filter(|r| !r.passed).count();

    if human {
        print_human_summary(&results, passed, failed);
    } else {
        println!("{}", render_results(&results, format)?);
    }

    if results.is_empty() {
        // A run that executed no checks has verified nothing. Printing the
        // strongest go-ahead ("safe to push") with exit 0 after 0 checks is how
        // an unrecognised --matrix used to sail through CI scripts.
        if human {
            println!(
                "\n{}",
                c::fail("CI simulation ran 0 checks — nothing was verified")
            );
        }
        std::process::exit(1);
    }

    if failed > 0 {
        if human {
            println!(
                "\n{}",
                c::fail("CI simulation FAILED — fix issues before pushing")
            );
        }
        std::process::exit(1);
    } else if human {
        println!("\n{}", c::pass("CI simulation PASSED — safe to push"));
    }

    Ok(())
}

/// The human banner: progress lines, a results line, and each failure's hint.
fn print_human_summary(results: &[CiCheckResult], passed: usize, failed: usize) {
    use crate::cli::colors as c;

    println!("\n{}", c::separator());
    let total_time: f64 = results.iter().map(|r| r.duration.as_secs_f64()).sum();
    println!(
        "\n{}",
        c::subheader(&format!(
            "Results: {} passed, {} failed ({:.1}s total)",
            passed, failed, total_time
        ))
    );

    for result in results {
        if !result.passed {
            println!("\n  {} {}", c::fail("FAIL"), c::label(&result.name));
            // Show first 10 lines of error output
            for line in result.output.lines().take(10) {
                println!("    {}", line);
            }
            if let Some(hint) = &result.fix_hint {
                println!("    {} {}", c::label("Fix:"), hint);
            }
        }
    }
}

/// Serializable view of one check, for the machine-readable formats.
#[derive(serde::Serialize)]
struct CiCheckRecord<'a> {
    name: &'a str,
    passed: bool,
    duration_secs: f64,
    output: &'a str,
    fix_hint: Option<&'a str>,
}

#[derive(serde::Serialize)]
struct CiRunRecord<'a> {
    total: usize,
    passed: usize,
    failed: usize,
    duration_secs: f64,
    checks: Vec<CiCheckRecord<'a>>,
}

fn to_record(results: &[CiCheckResult]) -> CiRunRecord<'_> {
    CiRunRecord {
        total: results.len(),
        passed: results.iter().filter(|r| r.passed).count(),
        failed: results.iter().filter(|r| !r.passed).count(),
        duration_secs: results.iter().map(|r| r.duration.as_secs_f64()).sum(),
        checks: results
            .iter()
            .map(|r| CiCheckRecord {
                name: &r.name,
                passed: r.passed,
                duration_secs: r.duration.as_secs_f64(),
                output: &r.output,
                fix_hint: r.fix_hint.as_deref(),
            })
            .collect(),
    }
}

/// Render the run in a machine-readable format. Every value clap advertises in
/// `--format` must produce that format; anything else is an advertised flag
/// that silently does nothing.
fn render_results(results: &[CiCheckResult], format: OutputFormat) -> Result<String> {
    let record = to_record(results);
    match format {
        OutputFormat::Json => Ok(serde_json::to_string_pretty(&record)?),
        OutputFormat::Yaml => Ok(serde_yaml_ng::to_string(&record)?),
        OutputFormat::Csv => Ok(render_csv(&record)),
        OutputFormat::Junit => Ok(render_junit(&record)),
        OutputFormat::Markdown => Ok(render_markdown(&record)),
        OutputFormat::Summary => Ok(format!(
            "{} passed, {} failed, {} total ({:.1}s)",
            record.passed, record.failed, record.total, record.duration_secs
        )),
        // is_human_format already routed these away.
        OutputFormat::Table | OutputFormat::Text | OutputFormat::Plain => Ok(String::new()),
    }
}

fn csv_escape(field: &str) -> String {
    format!("\"{}\"", field.replace('"', "\"\""))
}

fn render_csv(record: &CiRunRecord<'_>) -> String {
    let mut out = String::from("name,passed,duration_secs,fix_hint\n");
    for check in &record.checks {
        out.push_str(&format!(
            "{},{},{:.3},{}\n",
            csv_escape(check.name),
            check.passed,
            check.duration_secs,
            csv_escape(check.fix_hint.unwrap_or(""))
        ));
    }
    out
}

fn xml_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn render_junit(record: &CiRunRecord<'_>) -> String {
    let mut out = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str(&format!(
        "<testsuite name=\"pmat ci-local\" tests=\"{}\" failures=\"{}\" time=\"{:.3}\">\n",
        record.total, record.failed, record.duration_secs
    ));
    for check in &record.checks {
        out.push_str(&format!(
            "  <testcase name=\"{}\" time=\"{:.3}\"",
            xml_escape(check.name),
            check.duration_secs
        ));
        if check.passed {
            out.push_str("/>\n");
        } else {
            out.push_str(">\n");
            out.push_str(&format!(
                "    <failure message=\"check failed\">{}</failure>\n",
                xml_escape(check.output)
            ));
            out.push_str("  </testcase>\n");
        }
    }
    out.push_str("</testsuite>\n");
    out
}

fn render_markdown(record: &CiRunRecord<'_>) -> String {
    let mut out = String::from("# PMAT Local CI Simulation\n\n");
    out.push_str("| Check | Result | Seconds |\n|---|---|---|\n");
    for check in &record.checks {
        out.push_str(&format!(
            "| {} | {} | {:.1} |\n",
            check.name,
            if check.passed { "pass" } else { "FAIL" },
            check.duration_secs
        ));
    }
    out.push_str(&format!(
        "\n{} passed, {} failed, {} total ({:.1}s)\n",
        record.passed, record.failed, record.total, record.duration_secs
    ));
    out
}

/// Build the list of checks to run
///
/// An unrecognised `--matrix` is an error, not a warning: the catch-all arm used
/// to eprintln! a note and return an empty list, so 0 checks ran and the summary
/// still reported "CI simulation PASSED — safe to push" with exit 0.
fn build_check_list(quick: bool, matrix: Option<&str>) -> Result<Vec<&'static str>> {
    if let Some(m) = matrix {
        let checks = match m {
            "fmt" => vec!["cargo-fmt"],
            "clippy" => vec!["clippy-default", "clippy-all-features"],
            "test" => vec!["test-fast"],
            "cross" => vec!["cross-check-aarch64"],
            "bench" => vec!["bench-check"],
            "full" => full_checks(),
            _ => anyhow::bail!(
                "Unknown matrix: {}. Available: fmt, clippy, test, cross, bench, full",
                m
            ),
        };
        Ok(checks)
    } else if quick {
        Ok(vec!["cargo-fmt", "clippy-default", "test-fast"])
    } else {
        Ok(full_checks())
    }
}

fn full_checks() -> Vec<&'static str> {
    vec![
        "cargo-fmt",
        "clippy-default",
        "clippy-all-features",
        "test-fast",
        "test-lib",
        "cross-check-aarch64",
        "doc-check",
    ]
}

/// Run a single CI check
async fn run_check(check: &str, path: &Path, fix: bool, _verbose: bool) -> Result<String> {
    match check {
        "cargo-fmt" => {
            if fix {
                run_cmd(path, "cargo", &["fmt", "--all"])
            } else {
                run_cmd(path, "cargo", &["fmt", "--all", "--check"])
            }
        }
        "clippy-default" => {
            let mut args = vec!["clippy", "--lib", "--bins", "--examples"];
            if fix {
                args.push("--fix");
                args.push("--allow-dirty");
            }
            args.extend(&["--", "-D", "warnings"]);
            run_cmd(path, "cargo", &args)
        }
        "clippy-all-features" => {
            let mut args = vec!["clippy", "--lib", "--bins", "--all-features"];
            if fix {
                args.push("--fix");
                args.push("--allow-dirty");
            }
            args.extend(&["--", "-D", "warnings"]);
            run_cmd(path, "cargo", &args)
        }
        "test-fast" => run_cmd_with_env(
            path,
            "cargo",
            &["test", "--lib", "--", "--test-threads=4"],
            &[("RUST_MIN_STACK", "8388608")],
        ),
        "test-lib" => run_cmd_with_env(
            path,
            "cargo",
            &["test", "--lib"],
            &[("RUST_MIN_STACK", "8388608")],
        ),
        "cross-check-aarch64" => {
            // First ensure the target is installed
            let _ = run_cmd(
                path,
                "rustup",
                &["target", "add", "aarch64-unknown-linux-gnu"],
            );
            run_cmd(
                path,
                "cargo",
                &[
                    "check",
                    "--target",
                    "aarch64-unknown-linux-gnu",
                    "--no-default-features",
                ],
            )
        }
        "doc-check" => run_cmd_with_env(
            path,
            "cargo",
            &["doc", "--no-deps", "--document-private-items"],
            &[("RUSTDOCFLAGS", "-D warnings")],
        ),
        "bench-check" => run_cmd(path, "cargo", &["bench", "--no-run"]),
        _ => {
            anyhow::bail!("Unknown check: {}", check);
        }
    }
}

/// Run a command and capture output
fn run_cmd(path: &Path, cmd: &str, args: &[&str]) -> Result<String> {
    run_cmd_with_env(path, cmd, args, &[])
}

/// Run a command with environment variables
fn run_cmd_with_env(path: &Path, cmd: &str, args: &[&str], env: &[(&str, &str)]) -> Result<String> {
    let mut command = std::process::Command::new(cmd);
    command.args(args).current_dir(path);

    for (key, value) in env {
        command.env(key, value);
    }

    let output = command.output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let combined = if stderr.is_empty() { stdout } else { stderr };
        anyhow::bail!("{}", combined);
    }
}

/// Get fix hint for a failed check
fn get_fix_hint(check: &str) -> Option<String> {
    match check {
        "cargo-fmt" => Some("Run `cargo fmt --all` or use `pmat ci-local --fix`".to_string()),
        "clippy-default" | "clippy-all-features" => {
            Some("Run `cargo clippy --fix --allow-dirty` or use `pmat ci-local --fix`".to_string())
        }
        "test-fast" | "test-lib" => {
            Some("Run `RUST_MIN_STACK=8388608 cargo test --lib` to reproduce".to_string())
        }
        "cross-check-aarch64" => Some(
            "Run `rustup target add aarch64-unknown-linux-gnu` then `cargo check --target aarch64-unknown-linux-gnu --no-default-features`"
                .to_string(),
        ),
        "doc-check" => {
            Some("Run `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps`".to_string())
        }
        _ => None,
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    /// A missing path used to be reported as 3 failing CI stages, each with a
    /// fix hint ("Run `cargo fmt --all`") for a problem the user does not have.
    #[tokio::test]
    async fn test_missing_path_is_a_path_error_not_three_failing_stages() {
        let err = handle_ci_local(
            Path::new("/does/not/exist/pmat-ci-local-missing"),
            true,
            None,
            false,
            false,
            OutputFormat::Json,
        )
        .await
        .expect_err("a missing path must be an error, not a CI verdict");
        let msg = err.to_string();
        assert!(msg.contains("Path not found"), "{msg}");
        assert!(
            !msg.contains("cargo fmt"),
            "must not suggest a fix for a failure that did not happen: {msg}"
        );
    }

    #[test]
    fn test_build_check_list_quick() {
        let checks = build_check_list(true, None).unwrap();
        assert_eq!(checks.len(), 3);
        assert_eq!(checks[0], "cargo-fmt");
        assert_eq!(checks[1], "clippy-default");
        assert_eq!(checks[2], "test-fast");
    }

    #[test]
    fn test_build_check_list_full() {
        let checks = build_check_list(false, None).unwrap();
        assert!(checks.len() >= 5);
        assert!(checks.contains(&"cargo-fmt"));
        assert!(checks.contains(&"clippy-default"));
        assert!(checks.contains(&"test-lib"));
        assert!(checks.contains(&"cross-check-aarch64"));
    }

    #[test]
    fn test_build_check_list_matrix_fmt() {
        let checks = build_check_list(false, Some("fmt")).unwrap();
        assert_eq!(checks, vec!["cargo-fmt"]);
    }

    #[test]
    fn test_build_check_list_matrix_clippy() {
        let checks = build_check_list(false, Some("clippy")).unwrap();
        assert_eq!(checks, vec!["clippy-default", "clippy-all-features"]);
    }

    #[test]
    fn test_build_check_list_unknown_matrix_is_an_error() {
        // Regression: this used to return an empty Vec, so ci-local ran 0
        // checks and still printed "CI simulation PASSED — safe to push".
        let err = build_check_list(false, Some("nonexistent")).unwrap_err();
        assert!(err.to_string().contains("Unknown matrix"));
    }

    #[test]
    fn test_build_check_list_never_returns_an_empty_list_when_ok() {
        for matrix in [
            None,
            Some("fmt"),
            Some("clippy"),
            Some("test"),
            Some("cross"),
            Some("bench"),
            Some("full"),
        ] {
            for quick in [true, false] {
                let checks = build_check_list(quick, matrix).unwrap();
                assert!(!checks.is_empty(), "empty check list for {matrix:?}");
            }
        }
    }

    #[test]
    fn test_full_checks_length() {
        let checks = full_checks();
        assert!(checks.len() >= 5);
    }

    #[test]
    fn test_get_fix_hint_fmt() {
        let hint = get_fix_hint("cargo-fmt");
        assert!(hint.is_some());
        assert!(hint.unwrap().contains("cargo fmt"));
    }

    #[test]
    fn test_get_fix_hint_clippy() {
        let hint = get_fix_hint("clippy-default");
        assert!(hint.is_some());
        assert!(hint.unwrap().contains("clippy"));
    }

    #[test]
    fn test_get_fix_hint_unknown() {
        let hint = get_fix_hint("some-random-check");
        assert!(hint.is_none());
    }

    #[tokio::test]
    async fn test_run_check_fmt() {
        // Just verify it doesn't panic - actual formatting check may pass or fail
        let result = run_check("cargo-fmt", Path::new("."), false, false).await;
        assert!(result.is_ok() || result.is_err());
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod format_tests {
    use super::*;

    fn results() -> Vec<CiCheckResult> {
        vec![
            CiCheckResult {
                name: "cargo-fmt".to_string(),
                passed: true,
                duration: std::time::Duration::from_millis(1500),
                output: String::new(),
                fix_hint: None,
            },
            CiCheckResult {
                name: "clippy-default".to_string(),
                passed: false,
                duration: std::time::Duration::from_millis(500),
                output: "error: needless <borrow> & \"quote\"".to_string(),
                fix_hint: Some("cargo clippy --fix".to_string()),
            },
        ]
    }

    /// `-f json` used to print the human banner, so `python3 -m json.tool`
    /// failed at character 0. Every advertised format must produce that format.
    #[test]
    fn json_is_valid_json_and_carries_the_results() {
        let out = render_results(&results(), OutputFormat::Json).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
        assert_eq!(parsed["total"], 2);
        assert_eq!(parsed["failed"], 1);
        assert_eq!(parsed["checks"][0]["name"], "cargo-fmt");
        assert!(!out.contains("PMAT Local CI Simulation"));
    }

    /// `-f junit` emitted no XML at all.
    #[test]
    fn junit_is_xml_with_a_testcase_per_check() {
        let out = render_results(&results(), OutputFormat::Junit).unwrap();
        assert!(out.starts_with("<?xml"));
        assert!(out.contains("<testsuite"));
        assert_eq!(out.matches("<testcase").count(), 2);
        assert!(out.contains("<failure"));
        // Check output must be escaped, not injected raw.
        assert!(!out.contains("<borrow>"));
        assert!(out.contains("&lt;borrow&gt;"));
    }

    #[test]
    fn csv_has_a_header_and_one_row_per_check() {
        let out = render_results(&results(), OutputFormat::Csv).unwrap();
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines[0], "name,passed,duration_secs,fix_hint");
        assert_eq!(lines.len(), 3);
        assert!(lines[1].starts_with("\"cargo-fmt\",true"));
    }

    #[test]
    fn yaml_round_trips() {
        let out = render_results(&results(), OutputFormat::Yaml).unwrap();
        let parsed: serde_json::Value = serde_yaml_ng::from_str(&out).expect("valid YAML");
        assert_eq!(parsed["passed"], 1);
    }

    /// The three human formats keep the banner; the machine formats do not.
    #[test]
    fn only_table_text_plain_are_human() {
        assert!(is_human_format(OutputFormat::Table));
        assert!(is_human_format(OutputFormat::Text));
        assert!(is_human_format(OutputFormat::Plain));
        for f in [
            OutputFormat::Json,
            OutputFormat::Yaml,
            OutputFormat::Csv,
            OutputFormat::Junit,
            OutputFormat::Markdown,
            OutputFormat::Summary,
        ] {
            assert!(!is_human_format(f), "{f} must not print the human banner");
            assert!(!render_results(&results(), f).unwrap().is_empty());
        }
    }
}
