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

    // Format output.
    //
    // `failures_only` reaches the formatters and nothing else. It used to reach
    // neither: it was discarded at `build_scorer_config` (correctly, see below)
    // and the four formatters were never given it, so the flag had no consumer
    // at all and `--failures-only` was byte-identical to the plain run on a
    // repo with a passing category to drop.
    let output_text = match format {
        RepoScoreOutputFormat::Text => format_text(&score, verbose, failures_only),
        RepoScoreOutputFormat::Json => format_json(&score, failures_only)?,
        RepoScoreOutputFormat::Markdown => format_markdown(&score, failures_only),
        RepoScoreOutputFormat::Yaml => format_yaml(&score, failures_only)?,
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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod failures_only_filters_the_report_tests {
    //! `--failures-only` must remove passing rows from every format.
    //!
    //! The flag was discarded at `build_scorer_config` (correctly — see the
    //! doc there) and was never handed to a formatter, so it had NO consumer:
    //! on a repo with 5 failing categories and 1 passing one, `repo-score` and
    //! `repo-score --failures-only` were byte-identical and the ✓ row survived
    //! in both.
    use super::*;
    use crate::services::repo_score::{
        models::{CategoryScore, CategoryScores, ScoreMetadata, ScoreStatus},
        RepoScore,
    };

    fn category(score: f64, max: f64, status: ScoreStatus) -> CategoryScore {
        CategoryScore {
            score,
            max_score: max,
            percentage: score / max * 100.0,
            status,
            subcategories: vec![],
            findings: vec![],
        }
    }

    /// One passing category among five failing ones — the mixed state the flag
    /// exists to filter.
    fn mixed_score() -> RepoScore {
        RepoScore {
            total_score: 42.0,
            grade: Grade::F,
            categories: CategoryScores {
                documentation: category(2.0, 20.0, ScoreStatus::Fail),
                precommit_hooks: category(2.0, 20.0, ScoreStatus::Fail),
                repository_hygiene: category(10.0, 10.0, ScoreStatus::Pass),
                build_test_automation: category(2.0, 25.0, ScoreStatus::Fail),
                continuous_integration: category(2.0, 20.0, ScoreStatus::Fail),
                pmat_compliance: category(1.0, 5.0, ScoreStatus::Warning),
            },
            recommendations: vec![],
            metadata: ScoreMetadata::new(std::path::PathBuf::from(".")),
        }
    }

    #[test]
    fn text_drops_the_passing_row_and_keeps_the_failing_ones() {
        let score = mixed_score();
        let plain = format_text(&score, false, false);
        let filtered = format_text(&score, false, true);

        assert_ne!(plain, filtered, "--failures-only changed nothing");
        assert!(
            plain.contains("Repository Hygiene"),
            "control: the passing row is in the plain report:\n{plain}"
        );
        assert!(
            !filtered.contains("Repository Hygiene"),
            "the passing row survived --failures-only:\n{filtered}"
        );
        assert!(
            filtered.contains("Documentation"),
            "a failing row must survive:\n{filtered}"
        );
    }

    #[test]
    fn markdown_drops_the_passing_row() {
        let score = mixed_score();
        let plain = format_markdown(&score, false);
        let filtered = format_markdown(&score, true);

        assert_ne!(plain, filtered);
        assert!(plain.contains("Repository Hygiene"));
        assert!(!filtered.contains("Repository Hygiene"), "{filtered}");
    }

    #[test]
    fn json_and_yaml_drop_the_passing_category() {
        let score = mixed_score();
        let doc: serde_json::Value =
            serde_json::from_str(&format_json(&score, true).expect("json")).expect("parse");

        assert!(
            doc["categories"].get("repository_hygiene").is_none(),
            "the passing category survived in JSON: {doc}"
        );
        assert!(
            doc["categories"].get("documentation").is_some(),
            "a failing category must survive in JSON: {doc}"
        );
        assert_eq!(doc["failures_only"], serde_json::Value::Bool(true));

        let yaml = format_yaml(&score, true).expect("yaml");
        assert!(!yaml.contains("repository_hygiene"), "{yaml}");
        assert!(yaml.contains("documentation"), "{yaml}");
    }

    /// A display filter must never move a number. The flag used to reach
    /// `ScorerConfig.skip_slow_checks`, which scored a skipped check as a full
    /// pass: 96.0 plain, 99.0 with the flag, on the same repository.
    #[test]
    fn the_total_is_the_measured_one_in_every_format() {
        let score = mixed_score();
        for report in [
            format_text(&score, false, true),
            format_markdown(&score, true),
        ] {
            assert!(
                report.contains("42.0"),
                "the filtered report must still show the measured total:\n{report}"
            );
        }
        let doc: serde_json::Value =
            serde_json::from_str(&format_json(&score, true).expect("json")).expect("parse");
        assert_eq!(doc["total_score"], 42.0);
    }
}
