//! CLI handler for `pmat repo-score` command
//!
//! Calculates repository health score (0-110 scale) across 6 categories + bonus points.

use crate::cli::RepoScoreOutputFormat;
use crate::services::repo_score::{aggregator::ScoreAggregator, models::Grade, scorers::ScorerConfig, RepoScore};
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

// ============================================================================
// Badge Generation (Phase 3: README Badge Maintenance)
// ============================================================================

/// Update README.md with repository health badge
fn update_readme_badge(repo_path: &Path, score: &RepoScore) -> Result<()> {
    let readme_path = repo_path.join("README.md");

    if !readme_path.exists() {
        println!("⚠️  README.md not found - skipping badge update");
        return Ok(());
    }

    let content = fs::read_to_string(&readme_path)
        .context("Failed to read README.md")?;

    let badge_url = generate_badge_url(score);
    let badge_markdown = format!(
        "<!-- PMAT-REPO-SCORE:START -->\n![Repository Health]({})\n<!-- PMAT-REPO-SCORE:END -->",
        badge_url
    );

    let updated = if content.contains("<!-- PMAT-REPO-SCORE:START -->") {
        // Replace existing badge
        replace_badge_section(&content, &badge_markdown)
    } else {
        // Insert badge after main heading
        insert_badge_after_title(&content, &badge_markdown)
    };

    fs::write(&readme_path, updated)
        .context("Failed to write updated README.md")?;

    println!("✅ Updated README.md with repository health badge");

    Ok(())
}

/// Generate shields.io badge URL from repository score
fn generate_badge_url(score: &RepoScore) -> String {
    let final_score = score.final_score.round() as u8;
    let max_score = 125; // 100 base + 25 future (Git History bonus)

    let color = match score.grade {
        Grade::APlus | Grade::A => "brightgreen",
        Grade::AMinus | Grade::BPlus => "green",
        Grade::B => "yellow",
        Grade::C => "orange",
        Grade::D | Grade::F => "red",
    };

    // URL encode the grade (e.g., "A+" -> "A%2B")
    let grade_str = score.grade.as_str();
    let encoded_grade = grade_str.replace('+', "%2B");

    format!(
        "https://img.shields.io/badge/repo%20health-{}%2F{}%20({})-{}?style=flat-square",
        final_score,
        max_score,
        encoded_grade,
        color
    )
}

/// Replace existing badge section in README
fn replace_badge_section(content: &str, new_badge: &str) -> String {
    let start_marker = "<!-- PMAT-REPO-SCORE:START -->";
    let end_marker = "<!-- PMAT-REPO-SCORE:END -->";

    if let Some(start) = content.find(start_marker) {
        if let Some(end) = content[start..].find(end_marker) {
            let end_pos = start + end + end_marker.len();
            let mut result = String::with_capacity(content.len());
            result.push_str(&content[..start]);
            result.push_str(new_badge);
            result.push_str(&content[end_pos..]);
            return result;
        }
    }

    // Fallback: append at end if markers found but parsing failed
    format!("{}\n\n{}", content, new_badge)
}

/// Insert badge after main title (first # heading)
fn insert_badge_after_title(content: &str, badge: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();

    // Find first heading line
    for (i, line) in lines.iter().enumerate() {
        if line.trim_start().starts_with("# ") {
            // Insert badge after heading and any immediate blank lines
            let mut insert_pos = i + 1;
            while insert_pos < lines.len() && lines[insert_pos].trim().is_empty() {
                insert_pos += 1;
            }

            let mut result = Vec::with_capacity(lines.len() + 3);
            result.extend_from_slice(&lines[..insert_pos]);
            result.push("");
            result.push(badge);
            result.push("");
            result.extend_from_slice(&lines[insert_pos..]);

            return result.join("\n");
        }
    }

    // No heading found - prepend badge
    format!("{}\n\n{}", badge, content)
}
