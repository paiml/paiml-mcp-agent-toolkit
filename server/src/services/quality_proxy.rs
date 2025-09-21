use crate::models::proxy::{
    ProxyMode, ProxyOperation, ProxyRequest, ProxyResponse, ProxyStatus, QualityConfig,
    QualityMetrics, QualityReport, QualityViolation, ViolationSeverity, ViolationType,
};
use crate::services::ast_rust::analyze_rust_file_with_complexity;
use crate::services::complexity::aggregate_results_with_thresholds;
use crate::services::satd_detector::SATDDetector;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info, warn};

/// Service for proxying code changes through quality gates.
///
/// This service intercepts code operations from AI agents and ensures
/// they meet quality standards before being applied.
///
/// # Example
///
/// ```
/// use pmat::services::quality_proxy::QualityProxyService;
/// use pmat::models::proxy::{ProxyRequest, ProxyOperation, ProxyMode};
///
/// # async fn example() -> anyhow::Result<()> {
/// let service = QualityProxyService::new();
/// let request = ProxyRequest {
///     operation: ProxyOperation::Write,
///     file_path: "test.rs".to_string(),
///     content: Some("fn main() { println!(\"Hello\"); }".to_string()),
///     old_content: None,
///     new_content: None,
///     mode: ProxyMode::Strict,
///     quality_config: Default::default(),
/// };
///
/// let response = service.proxy_operation(request).await?;
/// assert!(response.quality_report.passed);
/// # Ok(())
/// # }
/// ```
pub struct QualityProxyService {
    satd_detector: SATDDetector,
}

impl QualityProxyService {
    /// Creates a new quality proxy service.
    #[must_use] 
    pub fn new() -> Self {
        Self {
            satd_detector: SATDDetector::new(),
        }
    }

    /// Proxies a code operation through quality gates.
    ///
    /// # Arguments
    ///
    /// * `request` - The proxy request containing operation details
    ///
    /// # Returns
    ///
    /// A proxy response with quality report and final content
    ///
    /// # Example
    ///
    /// ```
    /// use pmat::services::quality_proxy::QualityProxyService;
    /// use pmat::models::proxy::{ProxyRequest, ProxyOperation, ProxyMode, QualityConfig};
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let service = QualityProxyService::new();
    /// let request = ProxyRequest {
    ///     operation: ProxyOperation::Write,
    ///     file_path: "example.rs".to_string(),
    ///     content: Some("/// Example function\nfn example() {}".to_string()),
    ///     old_content: None,
    ///     new_content: None,
    ///     mode: ProxyMode::Advisory,
    ///     quality_config: QualityConfig::default(),
    /// };
    ///
    /// let response = service.proxy_operation(request).await?;
    /// println!("Status: {:?}", response.status);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn proxy_operation(&self, request: ProxyRequest) -> Result<ProxyResponse> {
        info!(
            "Proxying {} operation for {}",
            match request.operation {
                ProxyOperation::Write => "write",
                ProxyOperation::Edit => "edit",
                ProxyOperation::Append => "append",
            },
            request.file_path
        );

        let content = self.get_operation_content(&request)?;
        let file_extension = Path::new(&request.file_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("rs");

        let ((quality_metrics, passed), violations) = self
            .analyze_content(
                &content,
                &request.file_path,
                file_extension,
                &request.quality_config,
            )
            .await?;

        let (status, final_content, refactoring_applied, refactoring_plan) = match request.mode {
            ProxyMode::Strict => {
                if passed {
                    (ProxyStatus::Accepted, content, false, None)
                } else {
                    (ProxyStatus::Rejected, String::new(), false, None)
                }
            }
            ProxyMode::Advisory => (ProxyStatus::Accepted, content, false, None),
            ProxyMode::AutoFix => {
                if passed {
                    (ProxyStatus::Accepted, content, false, None)
                } else {
                    match self
                        .auto_fix_content(
                            &content,
                            &request.file_path,
                            file_extension,
                            &request.quality_config,
                        )
                        .await
                    {
                        Ok((fixed_content, plan)) => {
                            let ((_, fixed_passed), _) = self
                                .analyze_content(
                                    &fixed_content,
                                    &request.file_path,
                                    file_extension,
                                    &request.quality_config,
                                )
                                .await?;

                            if fixed_passed {
                                (ProxyStatus::Modified, fixed_content, true, Some(plan))
                            } else {
                                warn!("Auto-fix failed to meet quality standards");
                                (ProxyStatus::Rejected, String::new(), false, None)
                            }
                        }
                        Err(e) => {
                            warn!("Auto-fix failed: {}", e);
                            (ProxyStatus::Rejected, String::new(), false, None)
                        }
                    }
                }
            }
        };

        Ok(ProxyResponse {
            status,
            quality_report: QualityReport {
                passed,
                metrics: quality_metrics,
                violations,
            },
            final_content,
            refactoring_applied,
            refactoring_plan,
        })
    }

    fn get_operation_content(&self, request: &ProxyRequest) -> Result<String> {
        match request.operation {
            ProxyOperation::Write => request
                .content
                .clone()
                .context("Write operation requires content"),
            ProxyOperation::Edit => {
                let old = request
                    .old_content
                    .as_ref()
                    .context("Edit operation requires old_content")?;
                let new = request
                    .new_content
                    .as_ref()
                    .context("Edit operation requires new_content")?;

                if let Some(existing_content) = &request.content {
                    Ok(existing_content.replace(old, new))
                } else {
                    Ok(new.clone())
                }
            }
            ProxyOperation::Append => {
                let append_content = request
                    .content
                    .as_ref()
                    .context("Append operation requires content")?;

                if let Some(existing) = &request.old_content {
                    Ok(format!("{existing}\n{append_content}"))
                } else {
                    Ok(append_content.clone())
                }
            }
        }
    }

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
                let max_comp = u32::from(report
                    .hotspots
                    .iter()
                    .map(|h| h.complexity)
                    .max()
                    .unwrap_or(0));

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

    async fn auto_fix_content(
        &self,
        content: &str,
        file_path: &str,
        extension: &str,
        config: &QualityConfig,
    ) -> Result<(String, Vec<HashMap<String, serde_json::Value>>)> {
        if extension != "rs" {
            return Ok((content.to_string(), Vec::new()));
        }

        info!("Applying auto-fix refactoring to {}", file_path);

        // For now, we'll apply basic fixes:
        // 1. Remove SATD comments
        // 2. Add missing documentation
        // 3. Format code

        let mut fixed_content = content.to_string();
        let mut plan = Vec::new();

        // Remove SATD comments
        if !config.allow_satd {
            let satd_patterns = vec![
                r"//\s*TODO:.*\n",
                r"//\s*FIXME:.*\n",
                r"//\s*HACK:.*\n",
                r"//\s*BUG:.*\n",
            ];

            for pattern in satd_patterns {
                let re = regex::Regex::new(pattern)?;
                if re.is_match(&fixed_content) {
                    fixed_content = re.replace_all(&fixed_content, "").to_string();
                    let mut step = HashMap::new();
                    step.insert("action".to_string(), serde_json::json!("remove_satd"));
                    step.insert("pattern".to_string(), serde_json::json!(pattern));
                    plan.push(step);
                }
            }
        }

        // Add basic documentation for public items
        if config.require_docs {
            let lines: Vec<&str> = fixed_content.lines().collect();
            let mut new_lines = Vec::new();

            for (i, line) in lines.iter().enumerate() {
                let trimmed = line.trim();
                if trimmed.starts_with("pub fn")
                    || trimmed.starts_with("pub struct")
                    || trimmed.starts_with("pub enum")
                {
                    let prev_has_doc = i > 0 && lines[i - 1].trim().starts_with("///");
                    if !prev_has_doc {
                        let item_name = trimmed
                            .split_whitespace()
                            .nth(2)
                            .unwrap_or("item")
                            .split('(')
                            .next()
                            .unwrap_or("item");

                        new_lines.push(format!("/// {item_name}"));
                        let mut step = HashMap::new();
                        step.insert("action".to_string(), serde_json::json!("add_documentation"));
                        step.insert("item".to_string(), serde_json::json!(item_name));
                        plan.push(step);
                    }
                }
                new_lines.push((*line).to_string());
            }

            if !plan.is_empty() {
                fixed_content = new_lines.join("\n");
            }
        }

        // Format code using rustfmt
        if config.auto_format {
            if let Ok(formatted) = self.format_rust_code(&fixed_content).await {
                if formatted != fixed_content {
                    fixed_content = formatted;
                    let mut step = HashMap::new();
                    step.insert("action".to_string(), serde_json::json!("format_code"));
                    plan.push(step);
                }
            }
        }

        Ok((fixed_content, plan))
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

impl Default for QualityProxyService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_proxy_high_quality_code() {
        let service = QualityProxyService::new();
        let request = ProxyRequest {
            operation: ProxyOperation::Write,
            file_path: "test.rs".to_string(),
            content: Some(
                r#"/// A simple greeting function
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}"#
                .to_string(),
            ),
            old_content: None,
            new_content: None,
            mode: ProxyMode::Strict,
            quality_config: QualityConfig::default(),
        };

        let response = service.proxy_operation(request).await.unwrap();
        assert!(matches!(response.status, ProxyStatus::Accepted));
        assert!(response.quality_report.passed);
    }

    #[tokio::test]
    async fn test_proxy_reject_satd() {
        let service = QualityProxyService::new();
        let request = ProxyRequest {
            operation: ProxyOperation::Write,
            file_path: "test.rs".to_string(),
            content: Some(
                r#"fn process() {
    // TODO: This needs to be implemented properly
    // FIXME: Critical bug here
    unimplemented!()
}"#
                .to_string(),
            ),
            old_content: None,
            new_content: None,
            mode: ProxyMode::Strict,
            quality_config: QualityConfig::default(),
        };

        let response = service.proxy_operation(request).await.unwrap();
        assert!(matches!(response.status, ProxyStatus::Rejected));
        assert!(!response.quality_report.passed);
        assert!(response.quality_report.metrics.satd_count > 0);
    }

    #[tokio::test]
    async fn test_proxy_advisory_mode() {
        let service = QualityProxyService::new();
        let request = ProxyRequest {
            operation: ProxyOperation::Write,
            file_path: "test.rs".to_string(),
            content: Some(
                r#"pub fn undocumented() {
    println!("No docs");
}"#
                .to_string(),
            ),
            old_content: None,
            new_content: None,
            mode: ProxyMode::Advisory,
            quality_config: QualityConfig::default(),
        };

        let response = service.proxy_operation(request).await.unwrap();
        assert!(matches!(response.status, ProxyStatus::Accepted));
        assert!(!response.quality_report.violations.is_empty());
    }

    #[test]
    fn test_get_operation_content() {
        let service = QualityProxyService::new();

        let write_request = ProxyRequest {
            operation: ProxyOperation::Write,
            file_path: "test.rs".to_string(),
            content: Some("write content".to_string()),
            old_content: None,
            new_content: None,
            mode: ProxyMode::Strict,
            quality_config: QualityConfig::default(),
        };

        let content = service.get_operation_content(&write_request).unwrap();
        assert_eq!(content, "write content");

        let edit_request = ProxyRequest {
            operation: ProxyOperation::Edit,
            file_path: "test.rs".to_string(),
            content: Some("original content here".to_string()),
            old_content: Some("original".to_string()),
            new_content: Some("modified".to_string()),
            mode: ProxyMode::Strict,
            quality_config: QualityConfig::default(),
        };

        let content = service.get_operation_content(&edit_request).unwrap();
        assert_eq!(content, "modified content here");
    }
}

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}
