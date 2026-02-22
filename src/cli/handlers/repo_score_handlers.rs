#![cfg_attr(coverage_nightly, coverage(off))]
//! CLI handler for `pmat repo-score` command
//!
//! Calculates repository health score (0-100 scale) across 6 categories.

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
    output.push_str(&format!("  Score: {:.1}/100\n", score.total_score));
    output.push_str(&format!("  Grade: {}\n", score.grade.as_str()));
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
fn format_category(
    name: &str,
    category: &crate::services::repo_score::CategoryScore,
    verbose: bool,
) -> String {
    let mut output = String::new();
    let status_icon = match category.status {
        crate::services::repo_score::ScoreStatus::Pass => "✅",
        crate::services::repo_score::ScoreStatus::Warning => "⚠️",
        crate::services::repo_score::ScoreStatus::Fail => "❌",
    };

    output.push_str(&format!(
        "  {} {:<25} {:.1}/{:.1} ({:.1}%)\n",
        status_icon, name, category.score, category.max_score, category.percentage
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
    serde_yaml_ng::to_string(score).context("Failed to serialize to YAML")
}

/// Format score as Markdown
fn format_markdown(score: &RepoScore) -> String {
    let mut output = String::new();

    output.push_str("# Repository Health Score\n\n");

    output.push_str("## Summary\n\n");
    output.push_str(&format!("- **Score**: {:.1}/100\n", score.total_score));
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

    let content = fs::read_to_string(&readme_path).context("Failed to read README.md")?;

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

    fs::write(&readme_path, updated).context("Failed to write updated README.md")?;

    println!("✅ Updated README.md with repository health badge");

    Ok(())
}

/// Generate shields.io badge URL from repository score
fn generate_badge_url(score: &RepoScore) -> String {
    let final_score = score.total_score.round() as u8;
    let max_score = 100;

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
        final_score, max_score, encoded_grade, color
    )
}

/// Replace existing badge section in README
fn replace_badge_section(content: &str, new_badge: &str) -> String {
    let start_marker = "<!-- PMAT-REPO-SCORE:START -->";
    let end_marker = "<!-- PMAT-REPO-SCORE:END -->";

    if let Some(start) = content.find(start_marker) {
        if let Some(end) = content.get(start..).unwrap_or_default().find(end_marker) {
            let end_pos = start + end + end_marker.len();
            let mut result = String::with_capacity(content.len());
            result.push_str(content.get(..start).unwrap_or_default());
            result.push_str(new_badge);
            result.push_str(content.get(end_pos..).unwrap_or_default());
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

// ============================================================================
// Integration Tests (Phase 3: README Badge Maintenance)
// ============================================================================

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::repo_score::{CategoryScore, CategoryScores, ScoreMetadata, ScoreStatus};
    use std::path::PathBuf;
    use tempfile::TempDir;

    /// Helper to create test RepoScore
    fn create_test_score(total_score: f64, grade: Grade) -> RepoScore {
        RepoScore {
            total_score,
            grade,
            categories: CategoryScores {
                documentation: CategoryScore {
                    score: 15.0,
                    max_score: 15.0,
                    percentage: 100.0,
                    status: ScoreStatus::Pass,
                    subcategories: vec![],
                    findings: vec![],
                },
                precommit_hooks: CategoryScore {
                    score: 20.0,
                    max_score: 20.0,
                    percentage: 100.0,
                    status: ScoreStatus::Pass,
                    subcategories: vec![],
                    findings: vec![],
                },
                repository_hygiene: CategoryScore {
                    score: 15.0,
                    max_score: 15.0,
                    percentage: 100.0,
                    status: ScoreStatus::Pass,
                    subcategories: vec![],
                    findings: vec![],
                },
                build_test_automation: CategoryScore {
                    score: 25.0,
                    max_score: 25.0,
                    percentage: 100.0,
                    status: ScoreStatus::Pass,
                    subcategories: vec![],
                    findings: vec![],
                },
                continuous_integration: CategoryScore {
                    score: 20.0,
                    max_score: 20.0,
                    percentage: 100.0,
                    status: ScoreStatus::Pass,
                    subcategories: vec![],
                    findings: vec![],
                },
                pmat_compliance: CategoryScore {
                    score: 5.0,
                    max_score: 5.0,
                    percentage: 100.0,
                    status: ScoreStatus::Pass,
                    subcategories: vec![],
                    findings: vec![],
                },
            },
            recommendations: vec![],
            metadata: ScoreMetadata {
                timestamp: chrono::Utc::now().to_rfc3339(),
                repository_path: PathBuf::from("/tmp/test"),
                git_branch: Some("master".to_string()),
                git_commit: Some("test123".to_string()),
                pmat_version: env!("CARGO_PKG_VERSION").to_string(),
                spec_version: "1.0.0".to_string(),
                execution_time_ms: 0,
            },
        }
    }

    // ========================================================================
    // RED TEST 1: Badge Insertion in New README
    // ========================================================================

    #[test]
    fn test_badge_insertion_in_new_readme() {
        let temp_dir = TempDir::new().unwrap();
        let readme = temp_dir.path().join("README.md");
        fs::write(&readme, "# My Project\n\nDescription here.").unwrap();

        let score = create_test_score(104.0, Grade::APlus);
        update_readme_badge(temp_dir.path(), &score).unwrap();

        let content = fs::read_to_string(&readme).unwrap();

        // RED TEST ASSERTIONS (will FAIL until implementation correct)
        assert!(
            content.contains("<!-- PMAT-REPO-SCORE:START -->"),
            "Badge start marker not found"
        );
        assert!(
            content.contains("<!-- PMAT-REPO-SCORE:END -->"),
            "Badge end marker not found"
        );
        assert!(
            content.contains("repo%20health-104"),
            "Badge score not found in URL"
        );
        assert!(
            content.contains("brightgreen"),
            "Badge color not correct for A+ grade"
        );
        assert!(
            content.contains("A%2B"),
            "Badge grade not encoded properly (A+ → A%2B)"
        );
    }

    // ========================================================================
    // RED TEST 2: Badge Replacement in Existing README
    // ========================================================================

    #[test]
    fn test_badge_replacement_in_existing_readme() {
        let temp_dir = TempDir::new().unwrap();
        let readme = temp_dir.path().join("README.md");
        let initial = "# My Project\n\n<!-- PMAT-REPO-SCORE:START -->\n![Old Badge](https://img.shields.io/badge/repo%20health-50%2F125%20(F)-red)\n<!-- PMAT-REPO-SCORE:END -->\n\nDescription text.";
        fs::write(&readme, initial).unwrap();

        let score = create_test_score(99.0, Grade::A);
        update_readme_badge(temp_dir.path(), &score).unwrap();

        let content = fs::read_to_string(&readme).unwrap();

        // RED TEST ASSERTIONS
        assert!(
            content.contains("repo%20health-99"),
            "Badge not updated with new score"
        );
        assert!(
            content.contains("brightgreen"),
            "Badge color not updated for A grade"
        );
        assert!(
            !content.contains("repo%20health-50"),
            "Old badge score still present (replacement failed)"
        );
        assert!(
            content.matches("<!-- PMAT-REPO-SCORE:START -->").count() == 1,
            "Badge markers duplicated (should only appear once)"
        );
        assert!(
            content.contains("Description text."),
            "Badge update removed other content"
        );
    }

    // ========================================================================
    // RED TEST 3: Badge URL Generation with Different Grades
    // ========================================================================

    #[test]
    fn test_badge_url_generation_with_grades() {
        // Test A+ grade
        let score_a_plus = create_test_score(110.0, Grade::APlus);
        let url_a_plus = generate_badge_url(&score_a_plus);
        assert!(
            url_a_plus.contains("brightgreen"),
            "A+ should be brightgreen"
        );
        assert!(url_a_plus.contains("A%2B"), "A+ should be URL-encoded");

        // Test B grade
        let score_b = create_test_score(85.0, Grade::B);
        let url_b = generate_badge_url(&score_b);
        assert!(url_b.contains("yellow"), "B should be yellow");
        assert!(url_b.contains("repo%20health-85"), "Score should be 85");

        // Test C grade
        let score_c = create_test_score(75.0, Grade::C);
        let url_c = generate_badge_url(&score_c);
        assert!(url_c.contains("orange"), "C should be orange");

        // Test F grade
        let score_f = create_test_score(50.0, Grade::F);
        let url_f = generate_badge_url(&score_f);
        assert!(url_f.contains("red"), "F should be red");

        // Test URL format
        assert!(url_a_plus.starts_with("https://img.shields.io/badge/"));
        assert!(url_a_plus.contains("style=flat-square"));
    }

    // ========================================================================
    // RED TEST 4: Badge Insertion After Heading
    // ========================================================================

    #[test]
    fn test_badge_insertion_after_heading_skips_blank_lines() {
        let temp_dir = TempDir::new().unwrap();
        let readme = temp_dir.path().join("README.md");
        let initial = "# My Project\n\n\n\nFirst paragraph here.";
        fs::write(&readme, initial).unwrap();

        let score = create_test_score(95.0, Grade::A);
        update_readme_badge(temp_dir.path(), &score).unwrap();

        let content = fs::read_to_string(&readme).unwrap();

        // Badge should be inserted after blank lines following heading
        assert!(content.contains("# My Project"), "Heading should remain");
        assert!(
            content.contains("<!-- PMAT-REPO-SCORE:START -->"),
            "Badge should be inserted"
        );
        assert!(
            content.contains("First paragraph here."),
            "Original content should remain"
        );

        // Check ordering: heading → badge → content
        let heading_pos = content.find("# My Project").unwrap();
        let badge_pos = content.find("<!-- PMAT-REPO-SCORE:START -->").unwrap();
        let content_pos = content.find("First paragraph here.").unwrap();
        assert!(heading_pos < badge_pos, "Badge should come after heading");
        assert!(badge_pos < content_pos, "Badge should come before content");
    }

    // ========================================================================
    // RED TEST 5: Graceful Handling of Missing README
    // ========================================================================

    #[test]
    fn test_missing_readme_handled_gracefully() {
        let temp_dir = TempDir::new().unwrap();
        // No README.md created

        let score = create_test_score(100.0, Grade::APlus);
        let result = update_readme_badge(temp_dir.path(), &score);

        // Should not error, just skip
        assert!(result.is_ok(), "Missing README should not cause error");
    }
}
