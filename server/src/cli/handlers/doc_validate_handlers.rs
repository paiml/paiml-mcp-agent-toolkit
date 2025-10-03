//! Documentation link validation CLI handlers

use crate::services::doc_validator::{
    DocValidator, ValidationStatus, ValidatorConfig,
};
use anyhow::Result;
use clap::{Parser, ValueEnum};
use std::path::PathBuf;
use std::process::ExitCode;

/// Output format for validation results
#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    /// Human-readable text output
    Text,
    /// JSON output for programmatic consumption
    Json,
    /// JUnit XML for CI integration
    Junit,
}

/// Validate documentation links
#[derive(Parser, Debug)]
pub struct ValidateDocsCmd {
    /// Root directory to validate (defaults to current directory)
    #[arg(short, long)]
    pub root: Option<PathBuf>,

    /// Configuration file path
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Fail on broken links
    #[arg(short, long, default_value = "true")]
    pub fail_on_error: bool,

    /// Output format (text, json, junit)
    #[arg(short, long, default_value = "text")]
    pub output: OutputFormat,

    /// Maximum concurrent HTTP requests
    #[arg(long, default_value = "10")]
    pub max_concurrent: usize,

    /// HTTP request timeout in seconds
    #[arg(long, default_value = "30")]
    pub timeout: u64,

    /// Maximum retries for failed requests
    #[arg(long, default_value = "3")]
    pub max_retries: u32,

    /// Exclude patterns (can be specified multiple times)
    #[arg(long)]
    pub exclude: Vec<String>,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

impl ValidateDocsCmd {
    /// Execute the validate-docs command
    pub async fn execute(&self) -> Result<ExitCode> {
        let config = if let Some(config_path) = &self.config {
            self.load_config(config_path)?
        } else {
            self.build_config()
        };

        if self.verbose {
            eprintln!("🔍 Validating documentation links...");
            eprintln!("📁 Root: {}", config.root_dir.display());
            eprintln!("⏱️  Timeout: {}ms", config.http_timeout_ms);
            eprintln!("🔄 Max retries: {}", config.max_retries);
            eprintln!("⚡ Max concurrent: {}", config.max_concurrent_requests);
        }

        let validator = DocValidator::new(config);
        let root = self.root.clone().unwrap_or_else(|| PathBuf::from("."));
        let summary = validator.validate_directory(&root).await?;

        // Output results
        match self.output {
            OutputFormat::Text => self.print_text_summary(&summary),
            OutputFormat::Json => self.print_json_summary(&summary)?,
            OutputFormat::Junit => self.print_junit_summary(&summary)?,
        }

        // Exit with error code if broken links found and fail_on_error is true
        if self.fail_on_error && summary.broken_links > 0 {
            Ok(ExitCode::FAILURE)
        } else {
            Ok(ExitCode::SUCCESS)
        }
    }

    fn build_config(&self) -> ValidatorConfig {
        // Start with default excludes, then add CLI-provided ones
        let mut exclude_patterns = vec![
            "archive".to_string(),
            "node_modules".to_string(),
            ".git".to_string(),
            "target".to_string(),
        ];
        exclude_patterns.extend(self.exclude.clone());

        ValidatorConfig {
            root_dir: self.root.clone().unwrap_or_else(|| PathBuf::from(".")),
            http_timeout_ms: self.timeout * 1000,
            max_retries: self.max_retries,
            retry_delay_ms: 1000,
            max_concurrent_requests: self.max_concurrent,
            exclude_patterns,
            follow_redirects: true,
            user_agent: format!("pmat-doc-validator/{}", env!("CARGO_PKG_VERSION")),
        }
    }

    fn load_config(&self, path: &PathBuf) -> Result<ValidatorConfig> {
        // Load from TOML config file
        let content = std::fs::read_to_string(path)?;
        let config: ValidatorConfig = toml::from_str(&content)?;
        Ok(config)
    }

    fn print_text_summary(&self, summary: &crate::services::doc_validator::ValidationSummary) {
        println!("\n📊 Documentation Link Validation Summary");
        println!("========================================");
        println!();
        println!("📁 Files scanned:    {}", summary.total_files);
        println!("🔗 Links found:      {}", summary.total_links);
        println!("✅ Valid links:      {}", summary.valid_links);
        println!("❌ Broken links:     {}", summary.broken_links);
        println!("⏭️  Skipped links:    {}", summary.skipped_links);
        println!("⏱️  Duration:         {}ms", summary.duration_ms);
        println!();

        if summary.broken_links > 0 {
            println!("🔴 Broken Links:");
            println!("================");
            for result in &summary.results {
                if matches!(
                    result.status,
                    ValidationStatus::NotFound | ValidationStatus::HttpError(_)
                ) {
                    println!(
                        "  ❌ {}:{}",
                        result.link.source_file.display(),
                        result.link.line_number
                    );
                    println!("     Link: [{}]({})", result.link.text, result.link.target);
                    if let Some(msg) = &result.error_message {
                        println!("     Error: {}", msg);
                    }
                    if let Some(code) = result.http_status_code {
                        println!("     HTTP Status: {}", code);
                    }
                    println!();
                }
            }
        }

        if self.verbose {
            println!("\n📋 All Links:");
            println!("=============");
            for result in &summary.results {
                let status_icon = match result.status {
                    ValidationStatus::Valid => "✅",
                    ValidationStatus::NotFound => "❌",
                    ValidationStatus::HttpError(_) => "❌",
                    ValidationStatus::NetworkError => "⚠️",
                    ValidationStatus::InvalidLink => "⚠️",
                    ValidationStatus::Skipped => "⏭️",
                };
                println!(
                    "  {} {}:{} -> {}",
                    status_icon,
                    result.link.source_file.display(),
                    result.link.line_number,
                    result.link.target
                );
            }
        }

        if summary.broken_links == 0 {
            println!("🎉 All documentation links are valid!");
        } else {
            println!("💥 Found {} broken link(s)", summary.broken_links);
        }
    }

    fn print_json_summary(&self, summary: &crate::services::doc_validator::ValidationSummary) -> Result<()> {
        use serde_json::json;

        let results_json: Vec<_> = summary
            .results
            .iter()
            .map(|r| {
                json!({
                    "source_file": r.link.source_file.to_string_lossy(),
                    "line_number": r.link.line_number,
                    "link_text": r.link.text,
                    "link_target": r.link.target,
                    "link_type": format!("{:?}", r.link.link_type),
                    "status": format!("{:?}", r.status),
                    "error_message": r.error_message,
                    "http_status_code": r.http_status_code,
                    "response_time_ms": r.response_time_ms,
                })
            })
            .collect();

        let output = json!({
            "total_files": summary.total_files,
            "total_links": summary.total_links,
            "valid_links": summary.valid_links,
            "broken_links": summary.broken_links,
            "skipped_links": summary.skipped_links,
            "duration_ms": summary.duration_ms,
            "results": results_json,
        });

        println!("{}", serde_json::to_string_pretty(&output)?);
        Ok(())
    }

    fn print_junit_summary(&self, summary: &crate::services::doc_validator::ValidationSummary) -> Result<()> {
        println!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
        println!(
            "<testsuites name=\"Documentation Link Validation\" tests=\"{}\" failures=\"{}\" time=\"{:.3}\">",
            summary.total_links,
            summary.broken_links,
            summary.duration_ms as f64 / 1000.0
        );
        println!("  <testsuite name=\"Link Validation\" tests=\"{}\" failures=\"{}\">",
            summary.total_links, summary.broken_links);

        for result in &summary.results {
            let test_name = format!(
                "{}:{} - {}",
                result.link.source_file.display(),
                result.link.line_number,
                result.link.target
            );

            print!("    <testcase name=\"{}\" classname=\"LinkValidation\"",
                xml_escape(&test_name));

            if let Some(time_ms) = result.response_time_ms {
                print!(" time=\"{:.3}\"", time_ms as f64 / 1000.0);
            }

            if matches!(
                result.status,
                ValidationStatus::NotFound | ValidationStatus::HttpError(_)
            ) {
                println!(">");
                println!(
                    "      <failure message=\"{}\">",
                    xml_escape(&result.error_message.clone().unwrap_or_default())
                );
                println!("Link: {}", xml_escape(&result.link.target));
                println!("      </failure>");
                println!("    </testcase>");
            } else {
                println!(" />");
            }
        }

        println!("  </testsuite>");
        println!("</testsuites>");
        Ok(())
    }
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xml_escape() {
        assert_eq!(xml_escape("hello"), "hello");
        assert_eq!(xml_escape("<tag>"), "&lt;tag&gt;");
        assert_eq!(xml_escape("a & b"), "a &amp; b");
        assert_eq!(xml_escape("\"quoted\""), "&quot;quoted&quot;");
    }

    #[test]
    fn test_build_config() {
        let cmd = ValidateDocsCmd {
            root: Some(PathBuf::from("docs")),
            config: None,
            fail_on_error: true,
            output: OutputFormat::Text,
            max_concurrent: 20,
            timeout: 60,
            max_retries: 5,
            exclude: vec!["custom_exclude".to_string()],
            verbose: false,
        };

        let config = cmd.build_config();
        assert_eq!(config.root_dir, PathBuf::from("docs"));
        assert_eq!(config.http_timeout_ms, 60000);
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.max_concurrent_requests, 20);

        // Should have defaults + CLI excludes
        assert!(config.exclude_patterns.contains(&"archive".to_string()));
        assert!(config.exclude_patterns.contains(&"node_modules".to_string()));
        assert!(config.exclude_patterns.contains(&".git".to_string()));
        assert!(config.exclude_patterns.contains(&"target".to_string()));
        assert!(config.exclude_patterns.contains(&"custom_exclude".to_string()));
        assert_eq!(config.exclude_patterns.len(), 5);
    }

    #[test]
    fn test_build_config_without_custom_excludes() {
        let cmd = ValidateDocsCmd {
            root: Some(PathBuf::from("docs")),
            config: None,
            fail_on_error: true,
            output: OutputFormat::Text,
            max_concurrent: 10,
            timeout: 30,
            max_retries: 3,
            exclude: vec![], // No custom excludes
            verbose: false,
        };

        let config = cmd.build_config();

        // Should still have defaults
        assert_eq!(config.exclude_patterns, vec![
            "archive".to_string(),
            "node_modules".to_string(),
            ".git".to_string(),
            "target".to_string(),
        ]);
    }
}
