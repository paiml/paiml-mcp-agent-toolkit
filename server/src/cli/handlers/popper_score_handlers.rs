//! CLI handler for `pmat popper-score` command
//!
//! Calculates Popper Falsifiability Score (0-100 scale) evaluating
//! scientific rigor and falsifiability of software repositories.

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
pub async fn handle_popper_score(
    path: &Path,
    format: &RepoScoreOutputFormat,
    verbose: bool,
    failures_only: bool,
    output: Option<&Path>,
) -> Result<()> {
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

/// Format score as human-readable text
fn format_text(score: &PopperScore, verbose: bool, failures_only: bool) -> String {
    let mut output = String::new();

    // Header
    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    output.push_str(&format!(
        "🔬  Popper Falsifiability Score v{}\n",
        score.metadata.version
    ));
    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    output.push('\n');

    // Gateway status
    if score.gateway_passed {
        output.push_str("✅  Gateway: PASSED (Falsifiability >= 60%)\n");
    } else {
        output.push_str("❌  Gateway: FAILED (Falsifiability < 60%)\n");
        output.push_str("    Without falsifiable claims, score is 0.\n");
    }
    output.push('\n');

    // Summary
    output.push_str("📌  Summary\n");
    output.push_str(&format!(
        "  Score: {:.1}/{:.0}\n",
        score.raw_score, score.max_available
    ));
    output.push_str(&format!("  Normalized: {:.1}%\n", score.normalized_score));
    output.push_str(&format!("  Grade: {}\n", score.grade));
    output.push('\n');

    // Categories
    output.push_str("📂  Categories\n");

    let categories = [
        (
            "A. Falsifiability & Testability",
            &score.categories.falsifiability,
            true,
        ),
        (
            "B. Reproducibility Infrastructure",
            &score.categories.reproducibility,
            false,
        ),
        (
            "C. Transparency & Openness",
            &score.categories.transparency,
            false,
        ),
        (
            "D. Statistical Rigor",
            &score.categories.statistical_rigor,
            false,
        ),
        (
            "E. Historical Integrity",
            &score.categories.historical_integrity,
            false,
        ),
        (
            "F. ML/AI Reproducibility",
            &score.categories.ml_reproducibility,
            false,
        ),
    ];

    for (name, category, is_gateway) in categories {
        if category.is_not_applicable {
            output.push_str(&format!("  ⚪ {}: N/A\n", name));
            continue;
        }

        let percentage = category.percentage();
        let icon = if percentage >= 80.0 {
            "✅"
        } else if percentage >= 60.0 {
            "⚠️"
        } else {
            "❌"
        };

        let gateway_marker = if is_gateway { " [GATEWAY]" } else { "" };

        output.push_str(&format!(
            "  {} {}: {:.1}/{:.0} ({:.1}%){}\n",
            icon, name, category.earned, category.max, percentage, gateway_marker
        ));

        // Show sub-scores in verbose mode
        if verbose && !failures_only {
            for sub in &category.sub_scores {
                let sub_icon = if sub.earned >= sub.max * 0.8 {
                    "  ✓"
                } else if sub.earned >= sub.max * 0.5 {
                    "  ~"
                } else {
                    "  ✗"
                };
                output.push_str(&format!(
                    "    {} {}: {:.1}/{:.0} - {}\n",
                    sub_icon, sub.id, sub.earned, sub.max, sub.description
                ));
            }
        }
    }
    output.push('\n');

    // Verdict
    output.push_str("📋  Verdict\n");
    output.push_str(&format!("  {}\n", score.analysis.verdict));
    output.push('\n');

    // Recommendations
    if !score.recommendations.is_empty() && (!failures_only || !score.gateway_passed) {
        output.push_str("💡  Recommendations\n");
        for rec in &score.recommendations {
            let priority_icon = match rec.priority {
                crate::services::popper_score::RecommendationPriority::Critical => "🔴",
                crate::services::popper_score::RecommendationPriority::High => "🟠",
                crate::services::popper_score::RecommendationPriority::Medium => "🟡",
                crate::services::popper_score::RecommendationPriority::Low => "🟢",
            };
            output.push_str(&format!(
                "  {} [{}] {}\n",
                priority_icon, rec.category, rec.description
            ));
            if let Some(cmd) = &rec.command {
                output.push_str(&format!("     $ {}\n", cmd));
            }
        }
        output.push('\n');
    }

    // Footer
    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    output
}

/// Format score as JSON
fn format_json(score: &PopperScore) -> Result<String> {
    serde_json::to_string_pretty(score).context("Failed to serialize to JSON")
}

/// Format score as Markdown
fn format_markdown(score: &PopperScore, verbose: bool, _failures_only: bool) -> String {
    let mut output = String::new();

    // Header
    output.push_str(&format!(
        "# 🔬 Popper Falsifiability Score v{}\n\n",
        score.metadata.version
    ));

    // Gateway status
    if score.gateway_passed {
        output.push_str("> ✅ **Gateway PASSED**: Falsifiability >= 60%\n\n");
    } else {
        output.push_str("> ❌ **Gateway FAILED**: Falsifiability < 60%\n");
        output.push_str("> Without falsifiable claims, the total score is 0.\n\n");
    }

    // Summary
    output.push_str("## 📌 Summary\n\n");
    output.push_str(&format!(
        "- **Score**: {:.1}/{:.0}\n",
        score.raw_score, score.max_available
    ));
    output.push_str(&format!(
        "- **Normalized**: {:.1}%\n",
        score.normalized_score
    ));
    output.push_str(&format!("- **Grade**: {}\n\n", score.grade));

    // Categories table
    output.push_str("## 📂 Categories\n\n");
    output.push_str("| Category | Score | Percentage | Status |\n");
    output.push_str("|----------|-------|------------|--------|\n");

    let categories = [
        (
            "A. Falsifiability & Testability",
            &score.categories.falsifiability,
            true,
        ),
        (
            "B. Reproducibility Infrastructure",
            &score.categories.reproducibility,
            false,
        ),
        (
            "C. Transparency & Openness",
            &score.categories.transparency,
            false,
        ),
        (
            "D. Statistical Rigor",
            &score.categories.statistical_rigor,
            false,
        ),
        (
            "E. Historical Integrity",
            &score.categories.historical_integrity,
            false,
        ),
        (
            "F. ML/AI Reproducibility",
            &score.categories.ml_reproducibility,
            false,
        ),
    ];

    for (name, category, is_gateway) in categories {
        if category.is_not_applicable {
            output.push_str(&format!("| {} | N/A | N/A | ⚪ N/A |\n", name));
            continue;
        }

        let percentage = category.percentage();
        let icon = if percentage >= 80.0 {
            "✅"
        } else if percentage >= 60.0 {
            "⚠️"
        } else {
            "❌"
        };

        let status = if is_gateway {
            format!("{} GATEWAY", icon)
        } else {
            icon.to_string()
        };

        output.push_str(&format!(
            "| {} | {:.1}/{:.0} | {:.1}% | {} |\n",
            name, category.earned, category.max, percentage, status
        ));
    }
    output.push('\n');

    // Detailed sub-scores in verbose mode
    if verbose {
        output.push_str("## 📊 Detailed Breakdown\n\n");
        for (name, category, _) in categories {
            if category.is_not_applicable {
                continue;
            }
            output.push_str(&format!("### {}\n\n", name));
            for sub in &category.sub_scores {
                output.push_str(&format!(
                    "- **{}**: {:.1}/{:.0} - {}\n",
                    sub.id, sub.earned, sub.max, sub.description
                ));
            }
            output.push('\n');
        }
    }

    // Verdict
    output.push_str("## 📋 Verdict\n\n");
    output.push_str(&format!("{}\n\n", score.analysis.verdict));

    // Recommendations
    if !score.recommendations.is_empty() {
        output.push_str("## 💡 Recommendations\n\n");
        for rec in &score.recommendations {
            let priority = match rec.priority {
                crate::services::popper_score::RecommendationPriority::Critical => "🔴 Critical",
                crate::services::popper_score::RecommendationPriority::High => "🟠 High",
                crate::services::popper_score::RecommendationPriority::Medium => "🟡 Medium",
                crate::services::popper_score::RecommendationPriority::Low => "🟢 Low",
            };
            output.push_str(&format!(
                "- **[{}]** {}: {}\n",
                priority, rec.category, rec.description
            ));
            if let Some(cmd) = &rec.command {
                output.push_str(&format!("  ```bash\n  {}\n  ```\n", cmd));
            }
        }
        output.push('\n');
    }

    output
}

/// Format score as YAML
fn format_yaml(score: &PopperScore) -> Result<String> {
    serde_yaml::to_string(score).context("Failed to serialize to YAML")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_handler_invalid_path() {
        let result = handle_popper_score(
            Path::new("/nonexistent/path"),
            &RepoScoreOutputFormat::Text,
            false,
            false,
            None,
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

        let result =
            handle_popper_score(&file_path, &RepoScoreOutputFormat::Text, false, false, None).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a directory"));
    }

    #[tokio::test]
    async fn test_handler_empty_project() {
        let temp = TempDir::new().unwrap();

        let result = handle_popper_score(
            temp.path(),
            &RepoScoreOutputFormat::Text,
            false,
            false,
            None,
        )
        .await;

        // Should succeed but show gateway failure
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handler_json_output() {
        let temp = TempDir::new().unwrap();
        std::fs::write(
            temp.path().join("README.md"),
            "# Test\n\nSuccess criteria: Tests pass.",
        )
        .unwrap();

        let result = handle_popper_score(
            temp.path(),
            &RepoScoreOutputFormat::Json,
            false,
            false,
            None,
        )
        .await;

        assert!(result.is_ok());
    }
}
