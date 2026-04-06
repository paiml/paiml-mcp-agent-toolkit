#![cfg_attr(coverage_nightly, coverage(off))]
//! CLI handler for `pmat repo-score` command
//!
//! Calculates repository health score (0-100 scale) across 6 categories.
//!
//! ## Module Structure
//! - `repo_score_handlers_display.rs` — text/JSON/YAML/Markdown formatting
//! - `repo_score_handlers_badge.rs` — README badge generation and update
//! - `repo_score_handlers_tests.rs` — unit tests

use crate::cli::RepoScoreOutputFormat;
use crate::services::repo_score::{
    aggregator::ScoreAggregator, models::Grade, scorers::ScorerConfig, RepoScore,
};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Handle the repo-score command
pub async fn handle_repo_score(
    path: &Path,
    format: RepoScoreOutputFormat,
    verbose: bool,
    failures_only: bool,
    output: Option<&Path>,
    update_badge: bool,
    deep: bool,
) -> Result<()> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    // Validate path exists
    if !path.exists() {
        anyhow::bail!("Path not found: {}", path.display());
    }

    // Create configuration
    let config = ScorerConfig {
        verbose,
        timeout_seconds: 300,
        skip_slow_checks: failures_only,
        deep,
    };

    // Run scoring
    let aggregator = ScoreAggregator::new();
    let score = aggregator
        .aggregate(path, &config)
        .await
        .context("Failed to calculate repository score")?;

    // Update README badge if requested
    if update_badge {
        update_readme_badge(path, &score)?;
    }

    // Format output
    let output_text = match format {
        RepoScoreOutputFormat::Text => format_text(&score, verbose),
        RepoScoreOutputFormat::Json => format_json(&score)?,
        RepoScoreOutputFormat::Markdown => format_markdown(&score),
        RepoScoreOutputFormat::Yaml => format_yaml(&score)?,
    };

    // Write output
    if let Some(output_path) = output {
        fs::write(output_path, output_text)
            .with_context(|| format!("Failed to write to {}", output_path.display()))?;
        println!("Repository score written to: {}", output_path.display());
    } else {
        print!("{}", output_text);
    }

    Ok(())
}

// Display/formatting functions (format_text, format_category, format_json, format_yaml, format_markdown)
include!("repo_score_handlers_display.rs");

// Badge generation and README update (update_readme_badge, generate_badge_url, replace_badge_section, insert_badge_after_title)
include!("repo_score_handlers_badge.rs");

// Unit tests
include!("repo_score_handlers_tests.rs");

// Design-by-contract specifications (Verus-style)
// #[requires(project_path.is_dir())]
// #[ensures(result.is_ok() ==> ret.len() > 0)]
