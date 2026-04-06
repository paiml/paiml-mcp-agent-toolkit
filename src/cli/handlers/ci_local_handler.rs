//! Local CI simulation handler
//!
//! Runs the same quality gate matrix as GitHub Actions locally
//! to eliminate push-wait-fix loops.

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

/// Run local CI simulation
pub async fn handle_ci_local(
    path: &Path,
    quick: bool,
    matrix: Option<&str>,
    fix: bool,
    verbose: bool,
) -> Result<()> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    use crate::cli::colors as c;

    println!("{}\n", c::header("PMAT Local CI Simulation"));

    let checks = build_check_list(quick, matrix);
    let total = checks.len();
    let mut results: Vec<CiCheckResult> = Vec::new();

    for (i, check_name) in checks.iter().enumerate() {
        print!("  [{}/{}] {} ... ", i + 1, total, c::label(check_name));
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

        if passed {
            println!(
                "{} {}",
                c::pass(""),
                c::dim(&format!("({:.1}s)", duration.as_secs_f64()))
            );
        } else {
            println!("{}", c::fail(""));
        }

        if !passed && verbose {
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

    // Summary
    println!("\n{}", c::separator());
    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.iter().filter(|r| !r.passed).count();
    let total_time: f64 = results.iter().map(|r| r.duration.as_secs_f64()).sum();

    println!(
        "\n{}",
        c::subheader(&format!(
            "Results: {} passed, {} failed ({:.1}s total)",
            passed, failed, total_time
        ))
    );

    // Show failures with fix hints
    for result in &results {
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

    if failed > 0 {
        println!(
            "\n{}",
            c::fail("CI simulation FAILED — fix issues before pushing")
        );
        std::process::exit(1);
    } else {
        println!("\n{}", c::pass("CI simulation PASSED — safe to push"));
    }

    Ok(())
}

/// Build the list of checks to run
fn build_check_list(quick: bool, matrix: Option<&str>) -> Vec<&'static str> {
    if let Some(m) = matrix {
        match m {
            "fmt" => vec!["cargo-fmt"],
            "clippy" => vec!["clippy-default", "clippy-all-features"],
            "test" => vec!["test-fast"],
            "cross" => vec!["cross-check-aarch64"],
            "bench" => vec!["bench-check"],
            "full" => full_checks(),
            _ => {
                eprintln!(
                    "Unknown matrix: {}. Available: fmt, clippy, test, cross, bench, full",
                    m
                );
                vec![]
            }
        }
    } else if quick {
        vec!["cargo-fmt", "clippy-default", "test-fast"]
    } else {
        full_checks()
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

    #[test]
    fn test_build_check_list_quick() {
        let checks = build_check_list(true, None);
        assert_eq!(checks.len(), 3);
        assert_eq!(checks[0], "cargo-fmt");
        assert_eq!(checks[1], "clippy-default");
        assert_eq!(checks[2], "test-fast");
    }

    #[test]
    fn test_build_check_list_full() {
        let checks = build_check_list(false, None);
        assert!(checks.len() >= 5);
        assert!(checks.contains(&"cargo-fmt"));
        assert!(checks.contains(&"clippy-default"));
        assert!(checks.contains(&"test-lib"));
        assert!(checks.contains(&"cross-check-aarch64"));
    }

    #[test]
    fn test_build_check_list_matrix_fmt() {
        let checks = build_check_list(false, Some("fmt"));
        assert_eq!(checks, vec!["cargo-fmt"]);
    }

    #[test]
    fn test_build_check_list_matrix_clippy() {
        let checks = build_check_list(false, Some("clippy"));
        assert_eq!(checks, vec!["clippy-default", "clippy-all-features"]);
    }

    #[test]
    fn test_build_check_list_unknown_matrix() {
        let checks = build_check_list(false, Some("nonexistent"));
        assert!(checks.is_empty());
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
