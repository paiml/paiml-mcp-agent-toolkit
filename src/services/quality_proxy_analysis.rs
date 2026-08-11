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

        // Run lint checks using cargo clippy directly
        let lint_violations = match self.run_lint_checks(content).await {
            Ok(violations_found) => {
                for (line, message) in &violations_found {
                    violations.push(QualityViolation {
                        violation_type: ViolationType::Lint,
                        severity: lint_line_severity(message),
                        location: format!("{file_path}:{line}"),
                        message: message.clone(),
                        suggestion: Some("Fix lint issue".to_string()),
                    });
                }
                violations_found.len()
            }
            Err(e) => {
                warn!("Failed to run lint checks: {}", e);
                0
            }
        };

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
        let output = Command::new("cargo")
            .arg("clippy")
            .current_dir(temp_dir.path())
            .output()?;

        let mut violations = Vec::new();

        // Warnings are reported on a *successful* run too, so the findings are
        // collected regardless of exit status.
        let stderr = String::from_utf8_lossy(&output.stderr);
        for line in stderr.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("warning:") || trimmed.starts_with("error:") || trimmed.starts_with("error[") {
                // Extract line number if possible
                let line_num = 1; // Default line number
                let message = line.to_string();
                violations.push((line_num, message));
            }
        }

        Ok(violations)
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
