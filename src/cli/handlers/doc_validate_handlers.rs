//! Documentation link validation CLI handlers

use crate::services::doc_validator::{DocValidator, ValidationStatus, ValidatorConfig};
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use std::process::ExitCode;

pub use crate::contracts::OutputFormat;

/// Validate documentation links
#[derive(Parser, Debug)]
pub struct ValidateDocsCmd {
    /// Root directory to validate (defaults to current directory)
    #[arg(short, long)]
    pub root: Option<PathBuf>,

    /// Configuration file path
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Exit non-zero when a broken link is found (the default; bare
    /// `--fail-on-error` says so explicitly). Pass `--fail-on-error false`
    /// to report the links without failing the run.
    ///
    /// PMAT-688: this was a `bool` with `default_value = "true"`, a switch
    /// that could never change anything.
    #[arg(
        short,
        long,
        action = clap::ArgAction::Set,
        num_args = 0..=1,
        default_value_t = true,
        default_missing_value = "true"
    )]
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn execute(&self) -> Result<ExitCode> {
        let config = if let Some(config_path) = &self.config {
            self.load_config(config_path)?
        } else {
            self.build_config()
        };

        if self.verbose {
            crate::status_eprintln!("🔍 Validating documentation links...");
            crate::status_eprintln!("📁 Root: {}", config.root_dir.display());
            crate::status_eprintln!("⏱️  Timeout: {}ms", config.http_timeout_ms);
            crate::status_eprintln!("🔄 Max retries: {}", config.max_retries);
            crate::status_eprintln!("⚡ Max concurrent: {}", config.max_concurrent_requests);
        }

        // Only text/plain, json and junit are actually rendered. Every other
        // clap value used to fall through a catch-all to coloured human text,
        // so `validate-docs -o yaml | yq` was fed ANSI escapes. Refuse the
        // formats we cannot produce instead of silently downgrading them.
        check_output_format_supported(&self.output)?;

        let root = self.root.clone().unwrap_or_else(|| PathBuf::from("."));
        // A nonexistent --root used to walk to zero files and print
        // "All documentation links are valid!" with exit 0 even under
        // --fail-on-error: a gate that passes because it measured nothing.
        if !root.is_dir() {
            anyhow::bail!(
                "path does not exist or is not a directory: {}",
                root.display()
            );
        }

        let validator = DocValidator::new(config);
        let summary = validator.validate_directory(&root).await?;

        // Output results
        match self.output {
            OutputFormat::Text | OutputFormat::Plain => self.print_text_summary(&summary),
            OutputFormat::Json => self.print_json_summary(&summary)?,
            OutputFormat::Junit => self.print_junit_summary(&summary)?,
            ref other => {
                // Unreachable: check_output_format_supported rejects these above.
                anyhow::bail!("unsupported output format for validate-docs: {other}")
            }
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
        use crate::cli::colors as c;
        println!();
        println!("{}", c::header("Documentation Link Validation Summary"));
        println!();
        println!(
            "  Files scanned:    {}",
            c::number(&summary.total_files.to_string())
        );
        println!(
            "  Links found:      {}",
            c::number(&summary.total_links.to_string())
        );
        println!(
            "  Valid links:      {}{}{}",
            c::GREEN,
            summary.valid_links,
            c::RESET
        );
        println!(
            "  Broken links:     {}{}{}",
            if summary.broken_links > 0 {
                c::RED
            } else {
                c::GREEN
            },
            summary.broken_links,
            c::RESET
        );
        println!(
            "  Skipped links:    {}{}{}",
            c::DIM,
            summary.skipped_links,
            c::RESET
        );
        println!(
            "  Duration:         {}{}ms{}",
            c::DIM,
            summary.duration_ms,
            c::RESET
        );
        println!();

        if summary.broken_links > 0 {
            println!("{}", c::subheader("Broken Links:"));
            println!("{}", c::rule());
            for result in &summary.results {
                if matches!(
                    result.status,
                    ValidationStatus::NotFound | ValidationStatus::HttpError(_)
                ) {
                    println!(
                        "  {} {}{}{}:{}{}{}",
                        c::fail(""),
                        c::CYAN,
                        result.link.source_file.display(),
                        c::RESET,
                        c::YELLOW,
                        result.link.line_number,
                        c::RESET
                    );
                    println!("     Link: {}[{}]{}", c::DIM, result.link.text, c::RESET);
                    if let Some(msg) = &result.error_message {
                        println!("     Error: {}{}{}", c::RED, msg, c::RESET);
                    }
                    if let Some(code) = result.http_status_code {
                        println!("     HTTP Status: {}{}{}", c::RED, code, c::RESET);
                    }
                    println!();
                }
            }
        }

        if self.verbose {
            println!();
            println!("{}", c::subheader("All Links:"));
            println!("{}", c::rule());
            for result in &summary.results {
                let (status_icon, color) = match result.status {
                    ValidationStatus::Valid => ("✓", c::GREEN),
                    ValidationStatus::NotFound => ("✗", c::RED),
                    ValidationStatus::HttpError(_) => ("✗", c::RED),
                    ValidationStatus::NetworkError => ("⚠", c::YELLOW),
                    ValidationStatus::InvalidLink => ("⚠", c::YELLOW),
                    ValidationStatus::Skipped => ("⏭", c::DIM),
                };
                println!(
                    "  {color}{status_icon}{} {}{}{}:{}{}{} → {}",
                    c::RESET,
                    c::CYAN,
                    result.link.source_file.display(),
                    c::RESET,
                    c::YELLOW,
                    result.link.line_number,
                    c::RESET,
                    result.link.target
                );
            }
        }

        if summary.broken_links == 0 {
            println!("{}", c::pass("All documentation links are valid!"));
        } else {
            println!(
                "{}",
                c::fail(&format!("Found {} broken link(s)", summary.broken_links))
            );
        }
    }

    fn print_json_summary(
        &self,
        summary: &crate::services::doc_validator::ValidationSummary,
    ) -> Result<()> {
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

    fn print_junit_summary(
        &self,
        summary: &crate::services::doc_validator::ValidationSummary,
    ) -> Result<()> {
        println!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
        println!(
            "<testsuites name=\"Documentation Link Validation\" tests=\"{}\" failures=\"{}\" time=\"{:.3}\">",
            summary.total_links,
            summary.broken_links,
            summary.duration_ms as f64 / 1000.0
        );
        println!(
            "  <testsuite name=\"Link Validation\" tests=\"{}\" failures=\"{}\">",
            summary.total_links, summary.broken_links
        );

        for result in &summary.results {
            let test_name = format!(
                "{}:{} - {}",
                result.link.source_file.display(),
                result.link.line_number,
                result.link.target
            );

            print!(
                "    <testcase name=\"{}\" classname=\"LinkValidation\"",
                xml_escape(&test_name)
            );

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

/// Reject the `--output` values `validate-docs` has no renderer for.
///
/// The shared `OutputFormat` enum advertises nine values; this command
/// implements four. yaml/markdown/csv/summary/table used to be accepted and
/// rendered as coloured text, so machine consumers silently got ANSI escapes.
pub(crate) fn check_output_format_supported(format: &OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Text | OutputFormat::Plain | OutputFormat::Json | OutputFormat::Junit => {
            Ok(())
        }
        other => anyhow::bail!(
            "validate-docs does not support --output {other}; supported: text, plain, json, junit"
        ),
    }
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg_attr(coverage_nightly, coverage(off))]
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
        assert!(config
            .exclude_patterns
            .contains(&"node_modules".to_string()));
        assert!(config.exclude_patterns.contains(&".git".to_string()));
        assert!(config.exclude_patterns.contains(&"target".to_string()));
        assert!(config
            .exclude_patterns
            .contains(&"custom_exclude".to_string()));
        assert_eq!(config.exclude_patterns.len(), 5);
    }

    fn cmd_with(root: Option<PathBuf>, output: OutputFormat) -> ValidateDocsCmd {
        ValidateDocsCmd {
            root,
            config: None,
            fail_on_error: true,
            output,
            max_concurrent: 1,
            timeout: 1,
            max_retries: 0,
            exclude: vec![],
            verbose: false,
        }
    }

    // Regression: a nonexistent --root walked zero files and printed
    // "All documentation links are valid!" with exit 0.
    #[tokio::test]
    async fn test_execute_nonexistent_root_is_an_error() {
        let cmd = cmd_with(
            Some(PathBuf::from("/does/not/exist/pmat-doc-validate")),
            OutputFormat::Text,
        );
        let err = cmd.execute().await.expect_err("missing root must not pass");
        assert!(
            err.to_string().contains("does not exist"),
            "unexpected error: {err}"
        );
    }

    // Regression: yaml/markdown/csv/summary/table were accepted and rendered
    // as ANSI-coloured text by a catch-all match arm.
    #[tokio::test]
    async fn test_execute_rejects_unimplemented_output_format() {
        let dir = tempfile::tempdir().unwrap();
        let cmd = cmd_with(Some(dir.path().to_path_buf()), OutputFormat::Yaml);
        let err = cmd
            .execute()
            .await
            .expect_err("yaml has no renderer and must be rejected");
        assert!(
            err.to_string().contains("does not support --output yaml"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_check_output_format_supported_matrix() {
        for ok in [
            OutputFormat::Text,
            OutputFormat::Plain,
            OutputFormat::Json,
            OutputFormat::Junit,
        ] {
            assert!(
                check_output_format_supported(&ok).is_ok(),
                "{ok} must render"
            );
        }
        for bad in [
            OutputFormat::Yaml,
            OutputFormat::Markdown,
            OutputFormat::Csv,
            OutputFormat::Summary,
            OutputFormat::Table,
        ] {
            assert!(
                check_output_format_supported(&bad).is_err(),
                "{bad} has no renderer and must be refused, not downgraded to text"
            );
        }
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
        assert_eq!(
            config.exclude_patterns,
            vec![
                "archive".to_string(),
                "node_modules".to_string(),
                ".git".to_string(),
                "target".to_string(),
            ]
        );
    }

    /// PMAT-688: `--fail-on-error` was a `bool` with `default_value = "true"`,
    /// so the switch could never change anything — the flag-efficacy sweep
    /// booked it as a no-op on a corpus with a broken link. It is a real
    /// switch now: bare `--fail-on-error` and its absence both mean fail
    /// (the safe default), and `--fail-on-error false` opts out.
    #[test]
    fn fail_on_error_is_a_switch_that_can_be_turned_off() {
        let absent = ValidateDocsCmd::try_parse_from(["validate-docs"]).expect("parses");
        assert!(absent.fail_on_error, "fail by default");
        let bare = ValidateDocsCmd::try_parse_from(["validate-docs", "--fail-on-error"])
            .expect("bare switch parses");
        assert!(bare.fail_on_error);
        let off = ValidateDocsCmd::try_parse_from(["validate-docs", "--fail-on-error", "false"])
            .expect("`--fail-on-error false` parses");
        assert!(!off.fail_on_error, "the explicit value turns the gate off");
    }
}
