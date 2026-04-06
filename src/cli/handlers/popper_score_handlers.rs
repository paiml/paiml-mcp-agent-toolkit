#![cfg_attr(coverage_nightly, coverage(off))]
//! CLI handler for `pmat popper-score` command
//!
//! Calculates Popper Falsifiability Score (0-100 scale) evaluating
//! scientific rigor and falsifiability of software repositories.
//!
//! This module is split into sub-files via include!():
//! - `popper_score_format_helpers.rs` — shared helper functions
//! - `popper_score_format_text.rs` — text output formatting
//! - `popper_score_format_markdown.rs` — markdown, JSON, YAML formatting
//! - `popper_score_tests.rs` — test fixtures and handler tests
//! - `popper_score_tests_format.rs` — format-specific tests (included by tests)

use crate::cli::RepoScoreOutputFormat;
use crate::services::popper_score::{score_project, PopperScore};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Handle the popper-score command
///
/// Analyzes a project and calculates a comprehensive Popper Falsifiability Score
/// (0-100 scale) across six categories: Falsifiability & Testability, Reproducibility
/// Infrastructure, Transparency & Openness, Statistical Rigor, Historical Integrity,
/// and ML/AI Reproducibility.
///
/// # Arguments
///
/// * `path` - Path to the project root
/// * `format` - Output format (Text, Json, Markdown, or Yaml)
/// * `verbose` - Include detailed breakdown in output
/// * `failures_only` - Show only failing checks (recommendations)
/// * `output` - Optional file path to write results to (stdout if None)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_popper_score(
    path: &Path,
    format: &RepoScoreOutputFormat,
    verbose: bool,
    failures_only: bool,
    output: Option<&Path>,
) -> Result<()> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    // PMAT-510: Deprecation notice — Popper categories B-F absorbed into RPS v3.0
    eprintln!(
        "Note: `pmat popper-score` is deprecated. Popper categories B-F are now \
         integrated into `pmat rust-project-score` as the Reproducibility category. \
         Category A (Falsifiability) remains as the gateway check in RPS v3.0."
    );

    // Validate path exists
    if !path.exists() {
        anyhow::bail!("Path not found: {}", path.display());
    }

    // Validate it's a directory
    if !path.is_dir() {
        anyhow::bail!("Path is not a directory: {}", path.display());
    }

    // Run Popper scoring
    let popper_score = score_project(path).context("Failed to calculate Popper score")?;

    // Format output
    let output_text = match format {
        RepoScoreOutputFormat::Text => format_text(&popper_score, verbose, failures_only),
        RepoScoreOutputFormat::Json => format_json(&popper_score)?,
        RepoScoreOutputFormat::Markdown => format_markdown(&popper_score, verbose, failures_only),
        RepoScoreOutputFormat::Yaml => format_yaml(&popper_score)?,
    };

    // Write output
    if let Some(output_path) = output {
        fs::write(output_path, &output_text)
            .with_context(|| format!("Failed to write to {}", output_path.display()))?;
        println!("Popper score written to: {}", output_path.display());
    } else {
        print!("{}", output_text);
    }

    Ok(())
}

// Shared helper functions (category entries, icons, priority labels)
include!("popper_score_format_helpers.rs");

// Text output formatting
include!("popper_score_format_text.rs");

// Markdown, JSON, and YAML output formatting
include!("popper_score_format_markdown.rs");

// Tests
include!("popper_score_tests.rs");
