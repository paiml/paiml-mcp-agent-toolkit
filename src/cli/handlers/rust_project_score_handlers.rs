//! CLI handler for `pmat rust-project-score` command
//!
//! Calculates Rust project quality score (0-106 scale) across 6 categories.

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

    // Validate it has Cargo.toml
    if !path.join("Cargo.toml").exists() {
        anyhow::bail!(
            "Not a valid Rust project (no Cargo.toml found): {}",
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

/// Format score as human-readable text
fn format_text(
    score: &crate::services::rust_project_score::orchestrator::ProjectScore,
    recommendations: &[String],
    _verbose: bool,
) -> String {
    use crate::services::rust_project_score::orchestrator::SPEC_VERSION;

    let mut output = String::new();

    // Header
    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    output.push_str(&format!("🦀  Rust Project Score v{}\n", SPEC_VERSION));
    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    output.push('\n');

    // Summary
    output.push_str("📌  Summary\n");
    output.push_str(&format!(
        "  Score: {:.1}/{:.0}\n",
        score.total_earned, score.total_possible
    ));
    output.push_str(&format!("  Percentage: {:.1}%\n", score.percentage));
    output.push_str(&format!("  Grade: {}\n", score.grade));
    output.push('\n');

    // Categories
    output.push_str("📂  Categories\n");

    // Sort categories by name for consistent output
    let mut categories: Vec<_> = score.categories.iter().collect();
    categories.sort_by_key(|(name, _)| *name);

    for (name, category) in categories {
        let percentage = category.percentage();

        let icon = if percentage >= 90.0 {
            "✅"
        } else if percentage >= 70.0 {
            "⚠️"
        } else {
            "❌"
        };

        output.push_str(&format!(
            "  {} {}: {:.1}/{:.0} ({:.1}%)\n",
            icon, name, category.earned, category.max, percentage
        ));
    }
    output.push('\n');

    // Recommendations
    if !recommendations.is_empty() {
        output.push_str("💡  Recommendations\n");
        for rec in recommendations {
            output.push_str(&format!("  • {}\n", rec));
        }
        output.push('\n');
    }

    // Footer
    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    output
}

/// Format score as JSON
fn format_json(
    score: &crate::services::rust_project_score::orchestrator::ProjectScore,
    recommendations: &[String],
) -> Result<String> {
    let json = serde_json::json!({
        "version": "1.1",
        "total_earned": score.total_earned,
        "total_possible": score.total_possible,
        "percentage": score.percentage,
        "grade": score.grade.to_string(),
        "categories": score.categories.iter().map(|(name, cat)| {
            serde_json::json!({
                "name": name,
                "earned": cat.earned,
                "max": cat.max,
                "percentage": cat.percentage(),
            })
        }).collect::<Vec<_>>(),
        "recommendations": recommendations,
    });

    serde_json::to_string_pretty(&json).context("Failed to serialize to JSON")
}

/// Format score as Markdown
fn format_markdown(
    score: &crate::services::rust_project_score::orchestrator::ProjectScore,
    recommendations: &[String],
    _verbose: bool,
) -> String {
    use crate::services::rust_project_score::orchestrator::SPEC_VERSION;

    let mut output = String::new();

    // Header
    output.push_str(&format!("# 🦀 Rust Project Score v{}\n\n", SPEC_VERSION));

    // Summary
    output.push_str("## 📌 Summary\n\n");
    output.push_str(&format!(
        "- **Score**: {:.1}/{:.0}\n",
        score.total_earned, score.total_possible
    ));
    output.push_str(&format!("- **Percentage**: {:.1}%\n", score.percentage));
    output.push_str(&format!("- **Grade**: {}\n\n", score.grade));

    // Categories
    output.push_str("## 📂 Categories\n\n");
    output.push_str("| Category | Score | Percentage |\n");
    output.push_str("|----------|-------|------------|\n");

    let mut categories: Vec<_> = score.categories.iter().collect();
    categories.sort_by_key(|(name, _)| *name);

    for (name, category) in categories {
        let percentage = category.percentage();

        let icon = if percentage >= 90.0 {
            "✅"
        } else if percentage >= 70.0 {
            "⚠️"
        } else {
            "❌"
        };

        output.push_str(&format!(
            "| {} {} | {:.1}/{:.0} | {:.1}% |\n",
            icon, name, category.earned, category.max, percentage
        ));
    }
    output.push('\n');

    // Recommendations
    if !recommendations.is_empty() {
        output.push_str("## 💡 Recommendations\n\n");
        for rec in recommendations {
            output.push_str(&format!("- {}\n", rec));
        }
        output.push('\n');
    }

    output
}

/// Format score as YAML
fn format_yaml(
    score: &crate::services::rust_project_score::orchestrator::ProjectScore,
    recommendations: &[String],
) -> Result<String> {
    let yaml = serde_yaml::to_string(&serde_json::json!({
        "version": "1.1",
        "total_earned": score.total_earned,
        "total_possible": score.total_possible,
        "percentage": score.percentage,
        "grade": score.grade.to_string(),
        "categories": score.categories.iter().map(|(name, cat)| {
            serde_json::json!({
                "name": name,
                "earned": cat.earned,
                "max": cat.max,
                "percentage": cat.percentage(),
            })
        }).collect::<Vec<_>>(),
        "recommendations": recommendations,
    }))
    .context("Failed to serialize to YAML")?;

    Ok(yaml)
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::rust_project_score::models::CategoryScore;
    use crate::services::rust_project_score::orchestrator::{ProjectScore, SPEC_VERSION};
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn create_test_score() -> ProjectScore {
        let mut categories = HashMap::new();
        categories.insert(
            "Rust Tooling".to_string(),
            CategoryScore {
                earned: 20.0,
                max: 25.0,
            },
        );
        categories.insert(
            "Code Quality".to_string(),
            CategoryScore {
                earned: 15.0,
                max: 26.0,
            },
        );
        categories.insert(
            "Testing".to_string(),
            CategoryScore {
                earned: 18.0,
                max: 20.0,
            },
        );

        ProjectScore {
            total_earned: 53.0,
            total_possible: 71.0,
            percentage: 74.6,
            grade: crate::services::rust_project_score::models::Grade::B,
            categories,
            recommendations: vec![
                "Add more tests".to_string(),
                "Improve documentation".to_string(),
            ],
        }
    }

    #[tokio::test]
    async fn test_handler_invalid_path() {
        let result = handle_rust_project_score(
            Path::new("/nonexistent/path"),
            &RepoScoreOutputFormat::Text,
            false,
            false,
            None,
            false, // full mode
        )
        .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_handler_not_a_directory() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("file.txt");
        std::fs::write(&file_path, "not a directory").unwrap();

        let result = handle_rust_project_score(
            &file_path,
            &RepoScoreOutputFormat::Text,
            false,
            false,
            None,
            false, // full mode
        )
        .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a directory"));
    }

    #[tokio::test]
    async fn test_handler_no_cargo_toml() {
        let temp = TempDir::new().unwrap();

        let result = handle_rust_project_score(
            temp.path(),
            &RepoScoreOutputFormat::Text,
            false,
            false,
            None,
            false, // full mode
        )
        .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Cargo.toml"));
    }

    // =========================================================================
    // Format function tests
    // =========================================================================

    #[test]
    fn test_format_text_contains_header() {
        let score = create_test_score();
        let recommendations = score.recommendations.clone();
        let output = format_text(&score, &recommendations, false);

        assert!(output.contains("Rust Project Score"));
        assert!(output.contains(SPEC_VERSION));
    }

    #[test]
    fn test_format_text_contains_summary() {
        let score = create_test_score();
        let recommendations = score.recommendations.clone();
        let output = format_text(&score, &recommendations, false);

        assert!(output.contains("Summary"));
        assert!(output.contains("Score:"));
        assert!(output.contains("Percentage:"));
        assert!(output.contains("Grade:"));
    }

    #[test]
    fn test_format_text_contains_categories() {
        let score = create_test_score();
        let recommendations = score.recommendations.clone();
        let output = format_text(&score, &recommendations, false);

        assert!(output.contains("Categories"));
        assert!(output.contains("Rust Tooling"));
        assert!(output.contains("Code Quality"));
        assert!(output.contains("Testing"));
    }

    #[test]
    fn test_format_text_contains_recommendations() {
        let score = create_test_score();
        let recommendations = score.recommendations.clone();
        let output = format_text(&score, &recommendations, false);

        assert!(output.contains("Recommendations"));
        assert!(output.contains("Add more tests"));
        assert!(output.contains("Improve documentation"));
    }

    #[test]
    fn test_format_text_no_recommendations() {
        let mut score = create_test_score();
        score.recommendations = vec![];
        let output = format_text(&score, &[], false);

        // Should not contain Recommendations section when empty
        assert!(!output.contains("Recommendations"));
    }

    #[test]
    fn test_format_text_icons_passing() {
        let mut score = create_test_score();
        score.categories.clear();
        score.categories.insert(
            "Perfect".to_string(),
            CategoryScore {
                earned: 95.0,
                max: 100.0,
            },
        );
        let output = format_text(&score, &[], false);

        assert!(output.contains("✅"));
    }

    #[test]
    fn test_format_text_icons_warning() {
        let mut score = create_test_score();
        score.categories.clear();
        score.categories.insert(
            "Warning".to_string(),
            CategoryScore {
                earned: 75.0,
                max: 100.0,
            },
        );
        let output = format_text(&score, &[], false);

        assert!(output.contains("⚠️"));
    }

    #[test]
    fn test_format_text_icons_failing() {
        let mut score = create_test_score();
        score.categories.clear();
        score.categories.insert(
            "Failing".to_string(),
            CategoryScore {
                earned: 50.0,
                max: 100.0,
            },
        );
        let output = format_text(&score, &[], false);

        assert!(output.contains("❌"));
    }

    #[test]
    fn test_format_json_valid_json() {
        let score = create_test_score();
        let recommendations = score.recommendations.clone();
        let output = format_json(&score, &recommendations).unwrap();

        // Should be valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(parsed.is_object());
    }

    #[test]
    fn test_format_json_contains_fields() {
        let score = create_test_score();
        let recommendations = score.recommendations.clone();
        let output = format_json(&score, &recommendations).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(parsed["version"].is_string());
        assert!(parsed["total_earned"].is_f64());
        assert!(parsed["total_possible"].is_f64());
        assert!(parsed["percentage"].is_f64());
        assert!(parsed["grade"].is_string());
        assert!(parsed["categories"].is_array());
        assert!(parsed["recommendations"].is_array());
    }

    #[test]
    fn test_format_json_correct_values() {
        let score = create_test_score();
        let recommendations = score.recommendations.clone();
        let output = format_json(&score, &recommendations).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["total_earned"].as_f64().unwrap(), 53.0);
        assert_eq!(parsed["percentage"].as_f64().unwrap(), 74.6);
    }

    #[test]
    fn test_format_markdown_contains_header() {
        let score = create_test_score();
        let recommendations = score.recommendations.clone();
        let output = format_markdown(&score, &recommendations, false);

        assert!(output.contains("# 🦀 Rust Project Score"));
        assert!(output.contains(SPEC_VERSION));
    }

    #[test]
    fn test_format_markdown_contains_table() {
        let score = create_test_score();
        let recommendations = score.recommendations.clone();
        let output = format_markdown(&score, &recommendations, false);

        // Should contain markdown table syntax
        assert!(output.contains("| Category | Score | Percentage |"));
        assert!(output.contains("|----------|-------|------------|"));
    }

    #[test]
    fn test_format_markdown_contains_categories() {
        let score = create_test_score();
        let recommendations = score.recommendations.clone();
        let output = format_markdown(&score, &recommendations, false);

        assert!(output.contains("## 📂 Categories"));
        assert!(output.contains("Rust Tooling"));
        assert!(output.contains("Code Quality"));
    }

    #[test]
    fn test_format_markdown_recommendations_as_list() {
        let score = create_test_score();
        let recommendations = score.recommendations.clone();
        let output = format_markdown(&score, &recommendations, false);

        assert!(output.contains("## 💡 Recommendations"));
        assert!(output.contains("- Add more tests"));
        assert!(output.contains("- Improve documentation"));
    }

    #[test]
    fn test_format_yaml_valid_yaml() {
        let score = create_test_score();
        let recommendations = score.recommendations.clone();
        let output = format_yaml(&score, &recommendations).unwrap();

        // Should be valid YAML
        let parsed: serde_yaml::Value = serde_yaml::from_str(&output).unwrap();
        assert!(parsed.is_mapping());
    }

    #[test]
    fn test_format_yaml_contains_fields() {
        let score = create_test_score();
        let recommendations = score.recommendations.clone();
        let output = format_yaml(&score, &recommendations).unwrap();

        assert!(output.contains("version:"));
        assert!(output.contains("total_earned:"));
        assert!(output.contains("total_possible:"));
        assert!(output.contains("percentage:"));
        assert!(output.contains("grade:"));
        assert!(output.contains("categories:"));
        assert!(output.contains("recommendations:"));
    }

    // =========================================================================
    // Edge cases
    // =========================================================================

    #[test]
    fn test_format_text_empty_categories() {
        let mut score = create_test_score();
        score.categories.clear();
        let output = format_text(&score, &[], false);

        // Should still have Categories section but no entries
        assert!(output.contains("Categories"));
    }

    #[test]
    fn test_format_json_empty_recommendations() {
        let mut score = create_test_score();
        score.recommendations.clear();
        let output = format_json(&score, &[]).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let recommendations = parsed["recommendations"].as_array().unwrap();
        assert!(recommendations.is_empty());
    }

    #[test]
    fn test_format_markdown_empty_recommendations() {
        let mut score = create_test_score();
        score.recommendations.clear();
        let output = format_markdown(&score, &[], false);

        // Should not contain recommendations section when empty
        assert!(!output.contains("## 💡 Recommendations"));
    }
}
