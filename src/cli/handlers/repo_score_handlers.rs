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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_repo_score(
    path: &Path,
    format: RepoScoreOutputFormat,
    verbose: bool,
    failures_only: bool,
    output: Option<&Path>,
    update_badge: bool,
    deep: bool,
) -> Result<()> {
    // Validate path exists
    if !path.exists() {
        anyhow::bail!("Path not found: {}", path.display());
    }

    // Create configuration
    let config = build_scorer_config(verbose, failures_only, deep);

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

/// Build the configuration handed to the scorers.
///
/// `failures_only` is accepted and deliberately DISCARDED. It used to be wired
/// straight into `skip_slow_checks`, so a flag documented as "show only
/// failures and warnings" changed what was measured: the pre-commit hook
/// performance check was skipped, and a skipped check awards a full 10/10
/// "performance assumed" instead of the measured score. `repo-score` on this
/// repo scored 96.0 plain and 99.0 with `--failures-only` — the same repository,
/// two answers, and the higher one from the run that measured less. A
/// presentation flag must never reach the measurement; filtering belongs in the
/// formatters.
fn build_scorer_config(verbose: bool, _failures_only: bool, deep: bool) -> ScorerConfig {
    ScorerConfig {
        verbose,
        timeout_seconds: 300,
        skip_slow_checks: false,
        deep,
    }
}

// Display/formatting functions (format_text, format_category, format_json, format_yaml, format_markdown)
include!("repo_score_handlers_display.rs");

// Badge generation and README update (update_readme_badge, generate_badge_url, replace_badge_section, insert_badge_after_title)
include!("repo_score_handlers_badge.rs");

// Unit tests
include!("repo_score_handlers_tests.rs");

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod failures_only_is_a_display_filter_tests {
    use super::*;

    /// `--failures-only` was wired to `ScorerConfig.skip_slow_checks`, which
    /// skipped the pre-commit performance check and scored it 10/10 instead of
    /// the measured 7.0 — total 96.0 without the flag, 99.0 with it. The scorer
    /// configuration must not vary with a display flag.
    #[test]
    fn scorer_config_does_not_vary_with_failures_only() {
        let plain = build_scorer_config(false, false, false);
        let filtered = build_scorer_config(false, true, false);

        assert!(
            !filtered.skip_slow_checks,
            "--failures-only must not skip checks: a skipped check is scored as a pass"
        );
        assert_eq!(
            format!("{plain:?}"),
            format!("{filtered:?}"),
            "a display filter must not change what is measured"
        );
    }
}

// Design-by-contract specifications (Verus-style)
// #[requires(project_path.is_dir())]
// #[ensures(result.is_ok() ==> ret.len() > 0)]
