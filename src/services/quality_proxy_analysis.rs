/// Maximum cyclomatic complexity over every function in the file, including
/// methods on impls/classes.
///
/// Deliberately unfiltered: this is the measurement the quality report
/// publishes as `metrics.max_complexity`, and it must not depend on the
/// threshold the caller happened to configure.
pub(crate) fn max_function_cyclomatic(
    file_metrics: &crate::services::complexity::FileComplexityMetrics,
) -> u16 {
    file_metrics
        .functions
        .iter()
        .map(|f| f.metrics.cyclomatic)
        .chain(
            file_metrics
                .classes
                .iter()
                .flat_map(|c| c.methods.iter().map(|m| m.metrics.cyclomatic)),
        )
        .max()
        .unwrap_or(0)
}

/// Severity for one line of `cargo clippy` output.
///
/// Every lint finding used to be filed as a `Warning`, and `passed` is "all
/// violations are warnings", so content that does not even compile
/// ("error: could not compile … due to 1 previous error") was *accepted* by
/// strict mode while a TODO comment rejected it. rustc's own level prefix
/// decides: `error:`/`error[E0433]:` is an error.
pub(crate) fn lint_line_severity(line: &str) -> ViolationSeverity {
    let trimmed = line.trim_start();
    if trimmed.starts_with("error:") || trimmed.starts_with("error[") {
        ViolationSeverity::Error
    } else {
        ViolationSeverity::Warning
    }
}

/// Stderr fragments that mean **clippy never ran**, as opposed to clippy
/// having run and disliked the code.
///
/// Each is emitted by cargo/rustup *before* any code is compiled.
const CLIPPY_UNAVAILABLE_MARKERS: &[&str] = &[
    // rustup: the shim exists, the component does not (rust:*-slim, and any
    // minimal-profile toolchain):
    //   error: 'cargo-clippy' is not installed for the toolchain '1.95.0-...'
    "is not installed for the toolchain",
    // rustup: the whole toolchain is missing:
    //   error: toolchain '1.95.0-x86_64-unknown-linux-gnu' is not installed
    // Anchored on rustup's closing quote so it cannot match a rustc diagnostic
    // that merely contains the words.
    "' is not installed",
    // cargo: no `cargo-clippy` on PATH at all.
    "no such command:",
    // cargo (pre-1.54 wording).
    "no such subcommand:",
];

/// Why `cargo clippy` could not deliver a verdict — `None` when it did.
///
/// This exists because a tool that did not run was being read as a judgement
/// about the code. `rust:1.95-slim` installs the MINIMAL rustup profile, so
/// `/usr/local/cargo/bin/cargo-clippy` is present as a *shim* while the clippy
/// component is not installed. `cargo clippy` then writes, to stderr, exit 1:
///
/// ```text
/// error: 'cargo-clippy' is not installed for the toolchain '1.95.0-x86_64-unknown-linux-gnu'.
/// help: run `rustup component add clippy` to install it
/// ```
///
/// Both lines begin with a rustc-shaped level prefix, so `lint_line_severity`
/// filed the first as [`ViolationSeverity::Error`], `passed` became false, and
/// `ProxyMode::Strict` answered `Rejected`. The caller cannot tell that
/// rejection apart from "your code does not compile" — and the property test
/// `test_high_quality_code_accepted` duly reported a minimal failing input
/// (`file_path = "a.rs"`, `fn_name = "a"`), i.e. a logic bug that does not
/// exist. Measured in paiml/infra run 33091353601, `clean-room (pmat)` GATE B2:
/// 21024 passed, 9 failed, every failure this string.
///
/// A missing verifier is a NO-GO, never a verdict. Returning the reason here
/// lets the lint stage fail loudly and name the tool.
pub(crate) fn clippy_unavailable_reason(stderr: &str) -> Option<String> {
    stderr.lines().find_map(|line| {
        let trimmed = line.trim();
        // Only cargo's/rustup's own diagnostics, not source spans that might
        // quote one of these phrases inside the code being linted.
        if !(trimmed.starts_with("error:") || trimmed.starts_with("error[")) {
            return None;
        }
        CLIPPY_UNAVAILABLE_MARKERS
            .iter()
            .any(|marker| trimmed.contains(marker))
            .then(|| trimmed.to_string())
    })
}

/// Turn one completed `cargo clippy` run into lint findings, or into a loud
/// error when the run produced no verdict to read.
///
/// Split out from [`QualityProxyService::run_lint_checks`] so the
/// did-the-tool-even-run decision is a pure function over the process output
/// and can be tested against the exact bytes the clean room saw.
pub(crate) fn interpret_clippy_output(
    output: &std::process::Output,
) -> Result<Vec<(usize, String)>> {
    let stderr = String::from_utf8_lossy(&output.stderr);

    if let Some(reason) = clippy_unavailable_reason(&stderr) {
        anyhow::bail!(
            "the quality proxy's lint stage did not run: {reason}\n\
             This is a missing tool, not a finding about the code under review. \
             Install it with `rustup component add clippy`. No lint verdict was \
             produced, so none is reported."
        );
    }

    let mut violations = Vec::new();
    // Warnings are reported on a *successful* run too, so the findings are
    // collected regardless of exit status.
    for line in stderr.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("warning:")
            || trimmed.starts_with("error:")
            || trimmed.starts_with("error[")
        {
            // Extract line number if possible
            let line_num = 1; // Default line number
            let message = line.to_string();
            violations.push((line_num, message));
        }
    }

    Ok(violations)
}

impl QualityProxyService {
    async fn analyze_content(
        &self,
        content: &str,
        file_path: &str,
        extension: &str,
        config: &QualityConfig,
    ) -> Result<((QualityMetrics, bool), Vec<QualityViolation>)> {
        let mut violations = Vec::new();

        if extension != "rs" {
            debug!("Skipping Rust-specific analysis for non-Rust file");
            return Ok((
                (
                    QualityMetrics {
                        max_complexity: 0,
                        satd_count: 0,
                        lint_violations: 0,
                        coverage_percentage: None,
                    },
                    true,
                ),
                violations,
            ));
        }

        let temp_file = self.create_temp_file(content, extension)?;
        let temp_path = temp_file.path();

        // Analyze complexity
        let max_complexity = match analyze_rust_file_with_complexity(temp_path).await {
            Ok(file_metrics) => {
                // `report.hotspots` is threshold-filtered, so reading the maximum
                // from it published a 0 meaning "nothing exceeded the threshold"
                // as if it were the measured maximum: the same content reported
                // max_complexity 0 under the default threshold and 9 under
                // max_complexity=1. The measurement comes from the unfiltered
                // function list; the threshold only decides the violation.
                let max_comp = u32::from(max_function_cyclomatic(&file_metrics));

                // The result is already FileComplexityMetrics, use it directly
                let report = aggregate_results_with_thresholds(
                    vec![file_metrics],
                    Some(config.max_complexity as u16),
                    Some(config.max_complexity as u16 + 5),
                );

                if max_comp > config.max_complexity {
                    if let Some(hotspot) = report.hotspots.first() {
                        violations.push(QualityViolation {
                            violation_type: ViolationType::Complexity,
                            severity: ViolationSeverity::Error,
                            location: format!("{}:{}", file_path, hotspot.line),
                            message: format!(
                                "Function '{}' complexity {} exceeds maximum {}",
                                hotspot.function.as_ref().unwrap_or(&"unknown".to_string()),
                                hotspot.complexity,
                                config.max_complexity
                            ),
                            suggestion: Some(
                                "Consider splitting this function into smaller functions"
                                    .to_string(),
                            ),
                        });
                    }
                }

                max_comp
            }
            Err(e) => {
                warn!("Failed to analyze complexity: {}", e);
                0
            }
        };

        // Detect SATD
        let satd_instances = self
            .satd_detector
            .extract_from_content(content, Path::new(file_path))?;
        let satd_count = satd_instances.len();

        if !config.allow_satd && satd_count > 0 {
            for instance in &satd_instances {
                violations.push(QualityViolation {
                    violation_type: ViolationType::Satd,
                    severity: ViolationSeverity::Error,
                    location: format!("{}:{}", file_path, instance.line),
                    message: format!("SATD detected: {}", instance.text),
                    suggestion: Some(
                        "Remove TODO/FIXME comments and implement the functionality".to_string(),
                    ),
                });
            }
        }

        // Run lint checks using cargo clippy directly.
        //
        // A lint stage that did not run is NOT "zero lint violations". This arm
        // used to `warn!` and substitute 0, publishing `metrics.lint_violations
        // = 0` — a measurement nobody took — into the same QualityReport that
        // callers read as evidence. The failure is propagated instead: the
        // report either carries a lint measurement or it does not exist.
        let violations_found = self
            .run_lint_checks(content)
            .await
            .context("quality proxy lint stage failed; no lint measurement was produced")?;
        for (line, message) in &violations_found {
            violations.push(QualityViolation {
                violation_type: ViolationType::Lint,
                severity: lint_line_severity(message),
                location: format!("{file_path}:{line}"),
                message: message.clone(),
                suggestion: Some("Fix lint issue".to_string()),
            });
        }
        let lint_violations = violations_found.len();

        // Check documentation
        if config.require_docs {
            let doc_violations = self.check_documentation(content, file_path);
            violations.extend(doc_violations);
        }

        let passed = violations
            .iter()
            .all(|v| matches!(v.severity, ViolationSeverity::Warning));

        Ok((
            (
                QualityMetrics {
                    max_complexity,
                    satd_count,
                    lint_violations,
                    coverage_percentage: None,
                },
                passed,
            ),
            violations,
        ))
    }

    fn check_documentation(&self, content: &str, file_path: &str) -> Vec<QualityViolation> {
        let mut violations = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("pub fn")
                || trimmed.starts_with("pub struct")
                || trimmed.starts_with("pub enum")
            {
                // Check previous lines for documentation
                let has_doc = if line_num > 0 {
                    // Check up to 5 lines before for doc comments
                    let start = line_num.saturating_sub(5);
                    lines[start..line_num]
                        .iter()
                        .any(|l| l.trim().starts_with("///"))
                } else {
                    false
                };

                if !has_doc {
                    violations.push(QualityViolation {
                        violation_type: ViolationType::Docs,
                        severity: ViolationSeverity::Warning,
                        location: format!("{}:{}", file_path, line_num + 1),
                        message: "Public item missing documentation".to_string(),
                        suggestion: Some("Add /// documentation comment".to_string()),
                    });
                }
            }
        }

        violations
    }

    fn create_temp_file(&self, content: &str, extension: &str) -> Result<tempfile::NamedTempFile> {
        use std::io::Write;

        let mut temp_file = tempfile::Builder::new()
            .suffix(&format!(".{extension}"))
            .tempfile()?;

        temp_file.write_all(content.as_bytes())?;
        temp_file.flush()?;

        Ok(temp_file)
    }

    async fn run_lint_checks(&self, content: &str) -> Result<Vec<(usize, String)>> {
        use std::fs;
        use std::io::Write;
        use std::process::Command;

        // Create a temporary Rust project
        let temp_dir = tempfile::TempDir::new()?;
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir)?;

        let lib_path = src_dir.join("lib.rs");
        let mut lib_file = fs::File::create(&lib_path)?;
        lib_file.write_all(content.as_bytes())?;
        lib_file.flush()?;

        let cargo_toml = r#"[package]
name = "temp_quality_check"
version = "0.1.0"
edition = "2021"

[dependencies]
"#;

        let cargo_path = temp_dir.path().join("Cargo.toml");
        let mut cargo_file = fs::File::create(&cargo_path)?;
        cargo_file.write_all(cargo_toml.as_bytes())?;
        cargo_file.flush()?;

        // Run cargo clippy.
        //
        // `-D warnings` used to be passed here, which relabelled every lint as
        // "error:" and made rustc's own level prefix useless for telling a
        // style lint apart from content that does not compile. Levels are left
        // as rustc reports them so `lint_line_severity` can be trusted; the
        // caller decides what fails the gate.
        let output = match Command::new("cargo")
            .arg("clippy")
            .current_dir(temp_dir.path())
            .output()
        {
            Ok(output) => output,
            // `cargo` itself is not on PATH. Spawning failed, so there is no
            // stderr to classify — say so here rather than let a caller read
            // an io error as a lint finding.
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => anyhow::bail!(
                "the quality proxy's lint stage did not run: `cargo` is not on PATH ({e}). \
                 This is a missing tool, not a finding about the code under review."
            ),
            Err(e) => {
                return Err(e).context("failed to spawn `cargo clippy` for the quality proxy")
            }
        };

        interpret_clippy_output(&output)
    }

    async fn format_rust_code(&self, content: &str) -> Result<String> {
        use std::process::Command;

        let temp_file = self.create_temp_file(content, "rs")?;

        let output = Command::new("rustfmt")
            .arg("--edition")
            .arg("2021")
            .arg(temp_file.path())
            .output()?;

        if output.status.success() {
            std::fs::read_to_string(temp_file.path()).context("Failed to read formatted file")
        } else {
            Err(anyhow::anyhow!(
                "rustfmt failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }
}

#[cfg(test)]
mod analysis_measurement_tests {
    //! Regressions for two fabricated values in the quality_proxy report:
    //! a threshold-filtered `max_complexity`, and lint findings that were all
    //! filed as warnings so non-compiling content passed strict mode.
    use super::*;
    use crate::services::complexity::{
        ComplexityMetrics, FileComplexityMetrics, FunctionComplexity,
    };

    fn func(name: &str, cyclomatic: u16) -> FunctionComplexity {
        FunctionComplexity {
            name: name.to_string(),
            line_start: 1,
            line_end: 10,
            metrics: ComplexityMetrics {
                cyclomatic,
                cognitive: cyclomatic,
                nesting_max: 1,
                lines: 10,
                halstead: None,
            },
        }
    }

    fn file_with(functions: Vec<FunctionComplexity>) -> FileComplexityMetrics {
        FileComplexityMetrics {
            path: "qpa.rs".to_string(),
            total_complexity: ComplexityMetrics {
                cyclomatic: 0,
                cognitive: 0,
                nesting_max: 0,
                lines: 0,
                halstead: None,
            },
            functions,
            classes: vec![],
        }
    }

    #[test]
    fn test_max_complexity_is_measured_not_threshold_filtered() {
        // The reported maximum must not depend on the configured threshold:
        // under the default 20 the hotspot list is empty, and reading the max
        // from it published 0 for a function measured at 9.
        let metrics = file_with(vec![func("nasty", 9), func("tidy", 2)]);

        let report = aggregate_results_with_thresholds(vec![metrics.clone()], Some(20), Some(25));
        assert!(
            report.hotspots.iter().all(|h| h.complexity < 9),
            "the 9-complexity function is filtered out at threshold 20; \
             that filtered list must not be the source of the measurement"
        );

        assert_eq!(max_function_cyclomatic(&metrics), 9);
    }

    #[test]
    fn test_max_complexity_includes_methods() {
        let mut metrics = file_with(vec![func("plain", 3)]);
        metrics.classes.push(crate::services::complexity::ClassComplexity {
            name: "Impl".to_string(),
            line_start: 1,
            line_end: 20,
            metrics: ComplexityMetrics {
                cyclomatic: 0,
                cognitive: 0,
                nesting_max: 0,
                lines: 0,
                halstead: None,
            },
            methods: vec![func("method", 12)],
        });
        assert_eq!(max_function_cyclomatic(&metrics), 12);
    }

    #[test]
    fn test_max_complexity_of_empty_file_is_zero() {
        assert_eq!(max_function_cyclomatic(&file_with(vec![])), 0);
    }

    #[test]
    fn test_compile_errors_are_errors_not_warnings() {
        // These two lines are exactly what the shipped binary reported (as
        // severity "warning") while ACCEPTING "this is not rust at all !!!".
        for line in [
            "error: expected one of `!` or `::`, found `is`",
            "error: could not compile `temp_quality_check` (lib) due to 1 previous error",
            "error[E0433]: failed to resolve: use of undeclared crate or module `foo`",
        ] {
            assert!(
                matches!(lint_line_severity(line), ViolationSeverity::Error),
                "{line} must fail a strict gate"
            );
        }
    }

    #[test]
    fn test_style_lints_stay_warnings() {
        for line in [
            "warning: function `simple` is never used",
            "warning: `temp_quality_check` (lib) generated 1 warning",
        ] {
            assert!(matches!(
                lint_line_severity(line),
                ViolationSeverity::Warning
            ));
        }
    }
}

#[cfg(test)]
mod lint_stage_availability_tests {
    //! A missing tool must never be reported as a verdict about the code.
    //!
    //! These are not synthetic strings. `MISSING_COMPONENT_STDERR` is what
    //! `cargo clippy` printed inside the `rust:1.95-slim` clean-room container
    //! in paiml/infra run 33091353601, `clean-room (pmat)` GATE B2, where it
    //! cost nine tests — including a proptest that reported a "minimal failing
    //! input" for a logic bug that does not exist.
    use super::*;

    /// Verbatim from the clean-room job log (2026-08-27T17:07:27Z).
    const MISSING_COMPONENT_LINE: &str =
        "error: 'cargo-clippy' is not installed for the toolchain '1.95.0-x86_64-unknown-linux-gnu'.";
    const MISSING_COMPONENT_STDERR: &str = concat!(
        "error: 'cargo-clippy' is not installed for the toolchain '1.95.0-x86_64-unknown-linux-gnu'.\n",
        "help: run `rustup component add clippy` to install it\n",
    );

    /// What clippy prints when it HAS run and found something.
    const REAL_FINDINGS_STDERR: &str = "\
    Checking temp_quality_check v0.1.0 (/tmp/.tmpXYZ)
warning: function `simple` is never used
 --> src/lib.rs:1:4
warning: `temp_quality_check` (lib) generated 1 warning
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.31s
";

    /// What clippy prints when it HAS run and the content does not compile.
    const REAL_COMPILE_ERROR_STDERR: &str = "\
    Checking temp_quality_check v0.1.0 (/tmp/.tmpXYZ)
error: expected one of `!` or `::`, found `is`
 --> src/lib.rs:1:6
error: could not compile `temp_quality_check` (lib) due to 1 previous error
";

    fn output(stderr: &str) -> std::process::Output {
        std::process::Output {
            // Exit status is deliberately not consulted: the missing-component
            // run and a genuine compile error both exit non-zero.
            status: Default::default(),
            stdout: Vec::new(),
            stderr: stderr.as_bytes().to_vec(),
        }
    }

    /// THE regression. Falsification first: the same bytes, run through the
    /// severity classifier the proxy actually uses, are an Error — which is how
    /// "clippy is not installed" became `ProxyStatus::Rejected`. The guard is
    /// therefore load-bearing, not decoration.
    #[test]
    fn a_missing_clippy_component_is_not_a_code_verdict() {
        assert!(
            MISSING_COMPONENT_STDERR.starts_with(MISSING_COMPONENT_LINE),
            "the two constants must describe the same captured stderr"
        );
        assert!(
            matches!(
                lint_line_severity(MISSING_COMPONENT_LINE),
                ViolationSeverity::Error
            ),
            "precondition: this line is what the proxy used to file as an Error \
             about the code under review"
        );

        let reason = clippy_unavailable_reason(MISSING_COMPONENT_STDERR)
            .expect("a toolchain without the clippy component ran nothing");
        assert!(reason.contains("cargo-clippy"), "{reason}");

        let err = interpret_clippy_output(&output(MISSING_COMPONENT_STDERR))
            .expect_err("no verdict was produced, so none may be returned");
        let msg = err.to_string();
        assert!(msg.contains("did not run"), "{msg}");
        assert!(
            msg.contains("rustup component add clippy"),
            "the failure has to name the tool and how to get it; got: {msg}"
        );
    }

    #[test]
    fn a_missing_toolchain_is_not_a_code_verdict() {
        let stderr = "error: toolchain '1.95.0-x86_64-unknown-linux-gnu' is not installed\n";
        assert!(clippy_unavailable_reason(stderr).is_some(), "{stderr}");
        assert!(interpret_clippy_output(&output(stderr)).is_err());
    }

    #[test]
    fn a_cargo_without_the_clippy_subcommand_is_not_a_code_verdict() {
        for stderr in [
            "error: no such command: `clippy`\n",
            "error: no such subcommand: `clippy`\n",
        ] {
            assert!(clippy_unavailable_reason(stderr).is_some(), "{stderr}");
            assert!(
                interpret_clippy_output(&output(stderr)).is_err(),
                "{stderr}"
            );
        }
    }

    /// The counter-test. A guard that refuses every run would hide real lint
    /// findings, which is the same defect pointed the other way.
    #[test]
    fn a_clippy_run_that_actually_happened_still_reports_its_findings() {
        assert!(clippy_unavailable_reason(REAL_FINDINGS_STDERR).is_none());
        let found = interpret_clippy_output(&output(REAL_FINDINGS_STDERR))
            .expect("clippy ran; its findings are a verdict");
        assert_eq!(found.len(), 2, "{found:?}");
        assert!(found
            .iter()
            .all(|(_, m)| m.trim_start().starts_with("warning:")));
    }

    /// Content that does not compile is a verdict about the content, and must
    /// keep reaching strict mode as an Error.
    #[test]
    fn a_compile_error_is_still_a_verdict_about_the_code() {
        assert!(clippy_unavailable_reason(REAL_COMPILE_ERROR_STDERR).is_none());
        let found = interpret_clippy_output(&output(REAL_COMPILE_ERROR_STDERR))
            .expect("clippy ran; a compile error is its answer");
        assert!(
            found
                .iter()
                .any(|(_, m)| matches!(lint_line_severity(m), ViolationSeverity::Error)),
            "{found:?}"
        );
    }

    /// The markers are matched only on cargo's own diagnostic lines, so a
    /// source span that happens to quote one is still a finding.
    #[test]
    fn a_source_span_quoting_the_marker_is_not_mistaken_for_a_missing_tool() {
        let stderr = "\
warning: unused variable: `x`
 --> src/lib.rs:2:9
  |
2 |     let x = \"no such command: nope\";
";
        assert!(clippy_unavailable_reason(stderr).is_none(), "{stderr}");
    }
}
