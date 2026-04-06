#![cfg_attr(coverage_nightly, coverage(off))]
//! CLI handler for `pmat rust-project-score` command
//!
//! Calculates Rust project quality score (0-106 scale) across 6 categories.
//!
//! # Module Structure (include! pattern)
//!
//! - `rust_project_score_handlers_display.rs` — Text and Markdown formatters
//! - `rust_project_score_handlers_serialization.rs` — JSON and YAML formatters
//! - `rust_project_score_handlers_tests.rs` — Unit tests

use crate::cli::RepoScoreOutputFormat;
use crate::services::rust_project_score::models::ScoringMode;
use crate::services::rust_project_score::orchestrator::RustProjectScoreOrchestrator;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Handle the rust-project-score command
///
/// Analyzes a Rust project and calculates a comprehensive quality score (0-106 scale)
/// across six categories: Rust Tooling Compliance, Code Quality, Testing Excellence,
/// Documentation, Performance & Benchmarking, and Dependency Health.
///
/// # Arguments
///
/// * `path` - Path to the Rust project root (must contain Cargo.toml)
/// * `format` - Output format (Text, Json, Markdown, or Yaml)
/// * `verbose` - Include detailed breakdown in output
/// * `failures_only` - Show only failing checks (recommendations)
/// * `output` - Optional file path to write results to (stdout if None)
/// * `full` - Use full mode (comprehensive checks) vs fast mode (skips slow checks)
///
/// # Examples
///
/// ```no_run
/// use pmat::cli::handlers::rust_project_score_handlers::handle_rust_project_score;
/// use pmat::cli::RepoScoreOutputFormat;
/// use std::path::Path;
///
/// # async fn example() -> anyhow::Result<()> {
/// // Analyze current project in fast mode (default)
/// handle_rust_project_score(
///     Path::new("."),
///     &RepoScoreOutputFormat::Text,
///     false,  // verbose
///     false,  // failures_only
///     None,   // output to stdout
///     false,  // fast mode
/// ).await?;
///
/// // Full analysis with JSON output to file
/// handle_rust_project_score(
///     Path::new("/path/to/rust/project"),
///     &RepoScoreOutputFormat::Json,
///     true,   // verbose
///     false,  // show all checks
///     Some(Path::new("score.json")),  // write to file
///     true,   // full mode
/// ).await?;
/// # Ok(())
/// # }
/// ```
pub async fn handle_rust_project_score(
    path: &Path,
    format: &RepoScoreOutputFormat,
    verbose: bool,
    failures_only: bool,
    output: Option<&Path>,
    full: bool,
) -> Result<()> {
    // Validate path exists
    if !path.exists() {
        anyhow::bail!("Path not found: {}", path.display());
    }

    // Validate it's a directory
    if !path.is_dir() {
        anyhow::bail!("Path is not a directory: {}", path.display());
    }

    // Validate it has Cargo.toml or lakefile.lean (root or lean/ subdir)
    let is_rust = path.join("Cargo.toml").exists();
    let is_lean = path.join("lakefile.lean").exists()
        || path.join("lean-toolchain").exists()
        || path.join("lean").join("lakefile.lean").exists()
        || path.join("lean").join("lean-toolchain").exists();
    if !is_rust && !is_lean {
        anyhow::bail!(
            "Not a valid project (no Cargo.toml or lakefile.lean found): {}",
            path.display()
        );
    }

    // Create orchestrator and run scoring
    let orchestrator = RustProjectScoreOrchestrator::new();
    let mode = if full {
        ScoringMode::Full
    } else {
        ScoringMode::Fast
    };
    let project_score = orchestrator
        .score_with_mode(path, mode)
        .context("Failed to calculate Rust project score")?;
    debug_assert!(project_score.total_earned >= 0.0, "earned score negative");
    debug_assert!(
        project_score.total_possible > 0.0,
        "max score must be positive"
    );
    debug_assert!(
        project_score.total_earned <= project_score.total_possible,
        "earned {} > max {}",
        project_score.total_earned,
        project_score.total_possible
    );

    // Filter recommendations if failures_only
    let recommendations = if failures_only {
        project_score.recommendations.clone()
    } else {
        project_score.recommendations.clone()
    };

    // Format output
    let output_text = match format {
        RepoScoreOutputFormat::Text => format_text(&project_score, &recommendations, verbose),
        RepoScoreOutputFormat::Json => format_json(&project_score, &recommendations)?,
        RepoScoreOutputFormat::Markdown => {
            format_markdown(&project_score, &recommendations, verbose)
        }
        RepoScoreOutputFormat::Yaml => format_yaml(&project_score, &recommendations)?,
    };

    // Write output
    if let Some(output_path) = output {
        fs::write(output_path, output_text)
            .with_context(|| format!("Failed to write to {}", output_path.display()))?;
        println!("Rust project score written to: {}", output_path.display());
    } else {
        print!("{}", output_text);
    }

    Ok(())
}

// Display formatters: format_text, format_markdown
include!("rust_project_score_handlers_display.rs");

// Serialization formatters: format_json, format_yaml
include!("rust_project_score_handlers_serialization.rs");

// Unit tests
include!("rust_project_score_handlers_tests.rs");

// Design-by-contract specifications (Verus-style)
// #[requires(project_path.is_dir())]
// #[ensures(result.is_ok() ==> ret.len() > 0)]
