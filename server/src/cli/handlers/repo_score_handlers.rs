//! CLI handler for `pmat repo-score` command
//!
//! Calculates repository health score (0-110 scale) across 6 categories + bonus points.

use crate::cli::RepoScoreOutputFormat;
use crate::services::repo_score::{aggregator::ScoreAggregator, scorers::ScorerConfig, RepoScore};
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
) -> Result<()> {
    // Validate path exists
    if !path.exists() {
        anyhow::bail!("Path not found: {}", path.display());
    }

    // Create configuration
    let config = ScorerConfig {
        verbose,
        timeout_seconds: 300,
        skip_slow_checks: failures_only,
    };

    // Run scoring
    let aggregator = ScoreAggregator::new();
    let score = aggregator
        .aggregate(path, &config)
        .await
        .context("Failed to calculate repository score")?;

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

/// Format score as human-readable text
fn format_text(score: &RepoScore, verbose: bool) -> String {
    let mut output = String::new();

    // Header
    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    output.push_str("📊  Repository Health Score\n");
    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    output.push('\n');

    // Summary
    output.push_str("📌  Summary\n");
    output.push_str(&format!("  Total Score:  {:.1}/100\n", score.total_score));
    output.push_str(&format!("  Bonus Points: {:.1}/10\n", score.bonus_points));
    output.push_str(&format!("  Final Score:  {:.1}/110\n", score.final_score));
    output.push_str(&format!("  Grade:        {}\n", score.grade.as_str()));
    output.push('\n');

    // Categories
    output.push_str("📂  Categories\n");
    output.push_str(&format_category(
        "Documentation",
        &score.categories.documentation,
        verbose,
    ));
    output.push_str(&format_category(
        "Pre-commit Hooks",
        &score.categories.precommit_hooks,
        verbose,
    ));
    output.push_str(&format_category(
        "Repository Hygiene",
        &score.categories.repository_hygiene,
        verbose,
    ));
    output.push_str(&format_category(
        "Build/Test Automation",
        &score.categories.build_test_automation,
        verbose,
    ));
    output.push_str(&format_category(
        "Continuous Integration",
        &score.categories.continuous_integration,
        verbose,
    ));
    output.push_str(&format_category(
        "PMAT Compliance",
        &score.categories.pmat_compliance,
        verbose,
    ));
    output.push('\n');

    // Bonus
    if score.bonus_points > 0.0 {
        output.push_str("⭐  Bonus Points\n");
        if score.bonus.property_tests.points > 0.0 {
            output.push_str(&format!(
                "  Property Testing:  {:.1}/3.0\n",
                score.bonus.property_tests.points
            ));
        }
        if score.bonus.fuzzing.points > 0.0 {
            output.push_str(&format!(
                "  Fuzzing:           {:.1}/2.0\n",
                score.bonus.fuzzing.points
            ));
        }
        if score.bonus.mutation_testing.points > 0.0 {
            output.push_str(&format!(
                "  Mutation Testing:  {:.1}/2.0\n",
                score.bonus.mutation_testing.points
            ));
        }
        if score.bonus.living_docs.points > 0.0 {
            output.push_str(&format!(
                "  Living Docs:       {:.1}/3.0\n",
                score.bonus.living_docs.points
            ));
        }
        output.push('\n');
    }

    // Recommendations
    if !score.recommendations.is_empty() {
        output.push_str("💡  Recommendations\n");
        for rec in &score.recommendations {
            use crate::services::repo_score::Priority;
            output.push_str(&format!(
                "  {} {}: {}\n",
                match rec.priority {
                    Priority::Critical => "🔴",
                    Priority::High => "🔴",
                    Priority::Medium => "🟡",
                    Priority::Low => "🟢",
                },
                rec.category,
                rec.description
            ));
        }
        output.push('\n');
    }

    // Footer
    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    output
}

/// Format a single category
fn format_category(name: &str, category: &crate::services::repo_score::CategoryScore, verbose: bool) -> String {
    let mut output = String::new();
    let status_icon = match category.status {
        crate::services::repo_score::ScoreStatus::Pass => "✅",
        crate::services::repo_score::ScoreStatus::Warning => "⚠️",
        crate::services::repo_score::ScoreStatus::Fail => "❌",
    };

    output.push_str(&format!(
        "  {} {:<25} {:.1}/{:.1} ({:.1}%)\n",
        status_icon,
        name,
        category.score,
        category.max_score,
        category.percentage
    ));

    if verbose && !category.findings.is_empty() {
        for finding in &category.findings {
            output.push_str(&format!("     • {}\n", finding.message));
        }
    }

    output
}

/// Format score as JSON
fn format_json(score: &RepoScore) -> Result<String> {
    serde_json::to_string_pretty(score).context("Failed to serialize to JSON")
}

/// Format score as YAML
fn format_yaml(score: &RepoScore) -> Result<String> {
    serde_yaml::to_string(score).context("Failed to serialize to YAML")
}

/// Format score as Markdown
fn format_markdown(score: &RepoScore) -> String {
    let mut output = String::new();

    output.push_str("# Repository Health Score\n\n");

    output.push_str("## Summary\n\n");
    output.push_str(&format!("- **Total Score**: {:.1}/100\n", score.total_score));
    output.push_str(&format!("- **Bonus Points**: {:.1}/10\n", score.bonus_points));
    output.push_str(&format!("- **Final Score**: {:.1}/110\n", score.final_score));
    output.push_str(&format!("- **Grade**: {}\n\n", score.grade.as_str()));

    output.push_str("## Category Scores\n\n");
    output.push_str("| Category | Score | Max | Percentage | Status |\n");
    output.push_str("|----------|-------|-----|------------|--------|\n");

    let categories = [
        ("Documentation", &score.categories.documentation),
        ("Pre-commit Hooks", &score.categories.precommit_hooks),
        ("Repository Hygiene", &score.categories.repository_hygiene),
        (
            "Build/Test Automation",
            &score.categories.build_test_automation,
        ),
        (
            "Continuous Integration",
            &score.categories.continuous_integration,
        ),
        ("PMAT Compliance", &score.categories.pmat_compliance),
    ];

    for (name, cat) in &categories {
        let status = match cat.status {
            crate::services::repo_score::ScoreStatus::Pass => "✅ Pass",
            crate::services::repo_score::ScoreStatus::Warning => "⚠️ Warning",
            crate::services::repo_score::ScoreStatus::Fail => "❌ Fail",
        };
        output.push_str(&format!(
            "| {} | {:.1} | {:.1} | {:.1}% | {} |\n",
            name, cat.score, cat.max_score, cat.percentage, status
        ));
    }

    output.push_str("\n## Recommendations\n\n");
    if score.recommendations.is_empty() {
        output.push_str("No recommendations - excellent work! 🎉\n");
    } else {
        for rec in &score.recommendations {
            output.push_str(&format!("- **{}**: {}\n", rec.category, rec.description));
        }
    }

    output
}
