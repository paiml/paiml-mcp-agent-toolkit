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
                // The result is already FileComplexityMetrics, use it directly
                let report = aggregate_results_with_thresholds(
                    vec![file_metrics],
                    Some(config.max_complexity as u16),
                    Some(config.max_complexity as u16 + 5),
                );

                // Find max complexity from hotspots
                let max_comp = u32::from(
                    report
                        .hotspots
                        .iter()
                        .map(|h| h.complexity)
                        .max()
                        .unwrap_or(0),
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
                        severity: ViolationSeverity::Warning,
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

        // Run cargo clippy
        let output = Command::new("cargo")
            .arg("clippy")
            .arg("--")
            .arg("-D")
            .arg("warnings")
            .current_dir(temp_dir.path())
            .output()?;

        let mut violations = Vec::new();

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Parse basic clippy output
            for line in stderr.lines() {
                if line.contains("warning:") || line.contains("error:") {
                    // Extract line number if possible
                    let line_num = 1; // Default line number
                    let message = line.to_string();
                    violations.push((line_num, message));
                }
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
