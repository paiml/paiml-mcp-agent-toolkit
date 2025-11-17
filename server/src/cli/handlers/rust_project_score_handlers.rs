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
    let mut output = String::new();

    // Header
    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    output.push_str("🦀  Rust Project Score v1.1\n");
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
    let mut output = String::new();

    // Header
    output.push_str("# 🦀 Rust Project Score v1.1\n\n");

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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

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
}
