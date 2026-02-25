#![cfg_attr(coverage_nightly, coverage(off))]
//! Drift Detector - Detect documentation/code drift
//!
//! Detects when documentation (README, help text) diverges from actual code,
//! preventing the issue reported in GitHub #118 where `pmat mcp` was documented
//! but didn't exist.
//!
//! # Architecture (Toyota Way - Poka-yoke)
//!
//! Error-proofing through automated validation:
//! - Pre-commit hook validates all documentation references
//! - CI/CD blocks PRs with drift
//! - Build-time validation of examples
//!
//! # References
//!
//! - Specification: docs/specifications/unified-cli-mcp-help-integration.md
//! - GitHub Issue: #118
//! - Toyota Way: Poka-yoke (error-proofing), Jidoka (built-in quality)

use crate::cli::registry::CommandRegistry;
use regex::Regex;
use std::collections::HashSet;
use std::path::Path;

/// Drift detection errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DriftError {
    /// Command referenced in docs doesn't exist
    NonExistentCommand {
        mentioned: String,
        file: String,
        line: usize,
        suggestion: Option<String>,
    },
    /// Example in docs doesn't work
    InvalidExample {
        example: String,
        file: String,
        line: usize,
        reason: String,
    },
    /// Command exists but not documented
    UndocumentedCommand { command: String },
    /// Deprecated command still documented without warning
    DeprecatedWithoutWarning {
        command: String,
        file: String,
        line: usize,
    },
    /// Broken link in documentation
    BrokenLink {
        url: String,
        file: String,
        line: usize,
    },
}

impl std::fmt::Display for DriftError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NonExistentCommand {
                mentioned,
                file,
                line,
                suggestion,
            } => {
                write!(
                    f,
                    "{}:{}: command '{}' doesn't exist",
                    file, line, mentioned
                )?;
                if let Some(s) = suggestion {
                    write!(f, " (did you mean '{}'?)", s)?;
                }
                Ok(())
            }
            Self::InvalidExample {
                example,
                file,
                line,
                reason,
            } => {
                write!(
                    f,
                    "{}:{}: invalid example '{}': {}",
                    file, line, example, reason
                )
            }
            Self::UndocumentedCommand { command } => {
                write!(f, "command '{}' is not documented", command)
            }
            Self::DeprecatedWithoutWarning {
                command,
                file,
                line,
            } => {
                write!(
                    f,
                    "{}:{}: deprecated command '{}' documented without deprecation notice",
                    file, line, command
                )
            }
            Self::BrokenLink { url, file, line } => {
                write!(f, "{}:{}: broken link '{}'", file, line, url)
            }
        }
    }
}

impl std::error::Error for DriftError {}

/// Drift detection result
#[derive(Debug)]
pub struct DriftReport {
    /// Detected errors
    pub errors: Vec<DriftError>,
    /// Commands found in documentation
    pub documented_commands: HashSet<String>,
    /// Commands in registry but not documented
    pub undocumented_commands: HashSet<String>,
    /// Total commands in registry
    pub total_commands: usize,
    /// Documentation coverage percentage
    pub coverage: f64,
}

impl DriftReport {
    /// Check if the report has any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Get error count
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Format as human-readable report
    pub fn to_string_report(&self) -> String {
        let mut report = String::new();

        report.push_str("Drift Detection Report\n");
        report.push_str("======================\n\n");

        report.push_str(&format!(
            "Commands: {} total, {} documented ({:.1}% coverage)\n",
            self.total_commands,
            self.documented_commands.len(),
            self.coverage
        ));

        if self.has_errors() {
            report.push_str(&format!("\n❌ {} errors detected:\n\n", self.errors.len()));
            for error in &self.errors {
                report.push_str(&format!("  • {}\n", error));
            }
        } else {
            report.push_str("\n✅ No drift detected\n");
        }

        if !self.undocumented_commands.is_empty() {
            report.push_str("\n⚠️ Undocumented commands:\n");
            for cmd in &self.undocumented_commands {
                report.push_str(&format!("  • {}\n", cmd));
            }
        }

        report
    }
}

/// Detects documentation drift
pub struct DriftDetector {
    registry: CommandRegistry,
    /// Regex to find pmat command references
    command_regex: Regex,
    /// Regex to find code blocks
    code_block_regex: Regex,
}

// Analysis methods (new, detect_in_file, detect_in_content, generate_report, helpers)
include!("drift_detector_analysis.rs");

// Tests
include!("drift_detector_tests.rs");
