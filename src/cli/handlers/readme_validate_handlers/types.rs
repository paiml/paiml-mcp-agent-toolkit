#![cfg_attr(coverage_nightly, coverage(off))]
//! Types and command definitions for README validation

use clap::{Parser, ValueEnum};
use std::path::PathBuf;

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

/// Validate README and documentation for hallucinations
///
/// # Example
///
/// ```bash
/// # Generate deep context first
/// pmat context --output deep_context.md --format llm-optimized
///
/// # Validate README against codebase facts
/// pmat validate-readme \
///     --targets README.md CLAUDE.md \
///     --deep-context deep_context.md \
///     --fail-on-contradiction
/// ```
#[derive(Parser, Debug)]
pub struct ValidateReadmeCmd {
    /// Documentation files to validate (e.g., README.md, CLAUDE.md)
    #[arg(short, long, num_args = 1.., required = true)]
    pub targets: Vec<PathBuf>,

    /// Deep context markdown file (output from `pmat context`)
    #[arg(short, long, required = true)]
    pub deep_context: PathBuf,

    /// Confidence threshold for verification (0.0-1.0)
    #[arg(long, default_value = "0.9")]
    pub verified_threshold: f32,

    /// Confidence threshold for contradictions (0.0-1.0)
    #[arg(long, default_value = "0.3")]
    pub contradiction_threshold: f32,

    /// Fail if contradictions found
    #[arg(long, default_value = "true")]
    pub fail_on_contradiction: bool,

    /// Fail if unverified claims found
    #[arg(long, default_value = "false")]
    pub fail_on_unverified: bool,

    /// Output format (text, json, junit)
    #[arg(short, long, default_value = "text")]
    pub output: OutputFormat,

    /// Show only failures (contradictions and unverified)
    #[arg(long)]
    pub failures_only: bool,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,
}
