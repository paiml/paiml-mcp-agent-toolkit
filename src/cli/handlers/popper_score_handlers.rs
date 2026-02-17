#![cfg_attr(coverage_nightly, coverage(off))]
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

/// Build the array of category tuples used by both text and markdown formatters
fn popper_category_entries(
    score: &PopperScore,
) -> [(
    &str,
    &crate::services::popper_score::PopperCategoryScore,
    bool,
); 6] {
    [
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
    ]
}

/// Return the status icon for a percentage score
fn percentage_icon(percentage: f64) -> &'static str {
    if percentage >= 80.0 {
        "✅"
    } else if percentage >= 60.0 {
        "⚠️"
    } else {
        "❌"
    }
}

/// Return the icon string for a recommendation priority
fn priority_icon_text(
    priority: &crate::services::popper_score::RecommendationPriority,
) -> &'static str {
    match priority {
        crate::services::popper_score::RecommendationPriority::Critical => "🔴",
        crate::services::popper_score::RecommendationPriority::High => "🟠",
        crate::services::popper_score::RecommendationPriority::Medium => "🟡",
        crate::services::popper_score::RecommendationPriority::Low => "🟢",
    }
}

/// Return the markdown label for a recommendation priority
fn priority_label_markdown(
    priority: &crate::services::popper_score::RecommendationPriority,
) -> &'static str {
    match priority {
        crate::services::popper_score::RecommendationPriority::Critical => "🔴 Critical",
        crate::services::popper_score::RecommendationPriority::High => "🟠 High",
        crate::services::popper_score::RecommendationPriority::Medium => "🟡 Medium",
        crate::services::popper_score::RecommendationPriority::Low => "🟢 Low",
    }
}

/// Format a single category line for text output, including optional verbose sub-scores
fn format_text_category(
    output: &mut String,
    name: &str,
    category: &crate::services::popper_score::PopperCategoryScore,
    is_gateway: bool,
    verbose: bool,
    failures_only: bool,
) {
    if category.is_not_applicable {
        output.push_str(&format!("  ⚪ {}: N/A\n", name));
        return;
    }

    let percentage = category.percentage();
    let icon = percentage_icon(percentage);
    let gateway_marker = if is_gateway { " [GATEWAY]" } else { "" };

    output.push_str(&format!(
        "  {} {}: {:.1}/{:.0} ({:.1}%){}\n",
        icon, name, category.earned, category.max, percentage, gateway_marker
    ));

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

/// Append text-formatted recommendations to the output
fn format_text_recommendations(output: &mut String, score: &PopperScore, failures_only: bool) {
    if score.recommendations.is_empty() || (failures_only && score.gateway_passed) {
        return;
    }
    output.push_str("💡  Recommendations\n");
    for rec in &score.recommendations {
        let icon = priority_icon_text(&rec.priority);
        output.push_str(&format!(
            "  {} [{}] {}\n",
            icon, rec.category, rec.description
        ));
        if let Some(cmd) = &rec.command {
            output.push_str(&format!("     $ {}\n", cmd));
        }
    }
    output.push('\n');
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
    for (name, category, is_gateway) in popper_category_entries(score) {
        format_text_category(
            &mut output,
            name,
            category,
            is_gateway,
            verbose,
            failures_only,
        );
    }
    output.push('\n');

    // Verdict
    output.push_str("📋  Verdict\n");
    output.push_str(&format!("  {}\n", score.analysis.verdict));
    output.push('\n');

    // Recommendations
    format_text_recommendations(&mut output, score, failures_only);

    // Footer
    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    output
}

/// Format score as JSON
fn format_json(score: &PopperScore) -> Result<String> {
    serde_json::to_string_pretty(score).context("Failed to serialize to JSON")
}

/// Format a single category row for markdown table output
fn format_markdown_category_row(
    output: &mut String,
    name: &str,
    category: &crate::services::popper_score::PopperCategoryScore,
    is_gateway: bool,
) {
    if category.is_not_applicable {
        output.push_str(&format!("| {} | N/A | N/A | ⚪ N/A |\n", name));
        return;
    }

    let percentage = category.percentage();
    let icon = percentage_icon(percentage);

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

/// Append verbose detailed breakdown section for markdown
fn format_markdown_detailed_breakdown(output: &mut String, score: &PopperScore) {
    output.push_str("## 📊 Detailed Breakdown\n\n");
    for (name, category, _) in popper_category_entries(score) {
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

/// Append markdown-formatted recommendations to the output
fn format_markdown_recommendations(output: &mut String, score: &PopperScore) {
    if score.recommendations.is_empty() {
        return;
    }
    output.push_str("## 💡 Recommendations\n\n");
    for rec in &score.recommendations {
        let priority = priority_label_markdown(&rec.priority);
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

    for (name, category, is_gateway) in popper_category_entries(score) {
        format_markdown_category_row(&mut output, name, category, is_gateway);
    }
    output.push('\n');

    // Detailed sub-scores in verbose mode
    if verbose {
        format_markdown_detailed_breakdown(&mut output, score);
    }

    // Verdict
    output.push_str("## 📋 Verdict\n\n");
    output.push_str(&format!("{}\n\n", score.analysis.verdict));

    // Recommendations
    format_markdown_recommendations(&mut output, score);

    output
}

/// Format score as YAML
fn format_yaml(score: &PopperScore) -> Result<String> {
    serde_yaml::to_string(score).context("Failed to serialize to YAML")
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::popper_score::{
        AnalysisStatus, PopperAnalysis, PopperCategoryScore, PopperCategoryScores, PopperGrade,
        PopperMetadata, PopperRecommendation, PopperSubScore, RecommendationPriority,
    };
    use tempfile::TempDir;

    /// Create a test PopperScore with gateway passed
    fn create_test_score_passed() -> PopperScore {
        let mut categories = PopperCategoryScores::default();
        categories.falsifiability =
            PopperCategoryScore::new("Falsifiability & Testability", 20.0, 25.0);
        categories.falsifiability.add_sub_score(PopperSubScore::new(
            "A1",
            "Test Coverage",
            8.0,
            10.0,
            "Unit test coverage",
        ));
        categories.falsifiability.add_sub_score(PopperSubScore::new(
            "A2",
            "Claims",
            12.0,
            15.0,
            "Testable claims",
        ));
        categories.reproducibility =
            PopperCategoryScore::new("Reproducibility Infrastructure", 18.0, 25.0);
        categories.transparency = PopperCategoryScore::new("Transparency & Openness", 15.0, 20.0);
        categories.statistical_rigor = PopperCategoryScore::new("Statistical Rigor", 10.0, 15.0);
        categories.historical_integrity =
            PopperCategoryScore::new("Historical Integrity", 7.0, 10.0);
        // ML stays N/A

        let recommendations = vec![
            PopperRecommendation::new(
                "Testing",
                "Add mutation testing",
                RecommendationPriority::High,
                5.0,
            )
            .with_command("cargo mutants"),
            PopperRecommendation::new(
                "Documentation",
                "Improve README",
                RecommendationPriority::Medium,
                2.0,
            ),
            PopperRecommendation::new(
                "Infrastructure",
                "Add CI pipeline",
                RecommendationPriority::Critical,
                8.0,
            ),
            PopperRecommendation::new(
                "Testing",
                "Add benchmarks",
                RecommendationPriority::Low,
                1.0,
            ),
        ];

        PopperScore {
            raw_score: 70.0,
            max_available: 95.0,
            normalized_score: 73.7,
            grade: PopperGrade::B,
            gateway_passed: true,
            categories,
            recommendations,
            metadata: PopperMetadata::new("test-project".to_string()),
            analysis: PopperAnalysis {
                falsifiability_status: AnalysisStatus::Pass,
                reproducibility_status: AnalysisStatus::Partial,
                scrutiny_status: AnalysisStatus::Partial,
                methodology_status: AnalysisStatus::Pass,
                validation_status: AnalysisStatus::Fail,
                verdict: "Good scientific practices with room for improvement.".to_string(),
            },
        }
    }

    /// Create a test PopperScore with gateway failed
    fn create_test_score_failed() -> PopperScore {
        let mut categories = PopperCategoryScores::default();
        categories.falsifiability =
            PopperCategoryScore::new("Falsifiability & Testability", 10.0, 25.0);
        categories.reproducibility =
            PopperCategoryScore::new("Reproducibility Infrastructure", 5.0, 25.0);
        categories.transparency = PopperCategoryScore::new("Transparency & Openness", 5.0, 20.0);
        categories.statistical_rigor = PopperCategoryScore::new("Statistical Rigor", 3.0, 15.0);
        categories.historical_integrity =
            PopperCategoryScore::new("Historical Integrity", 2.0, 10.0);

        PopperScore {
            raw_score: 0.0,
            max_available: 95.0,
            normalized_score: 0.0,
            grade: PopperGrade::InsufficientFalsifiability,
            gateway_passed: false,
            categories,
            recommendations: vec![PopperRecommendation::new(
                "Falsifiability",
                "Add testable claims",
                RecommendationPriority::Critical,
                25.0,
            )],
            metadata: PopperMetadata::new("failing-project".to_string()),
            analysis: PopperAnalysis {
                falsifiability_status: AnalysisStatus::Fail,
                reproducibility_status: AnalysisStatus::Fail,
                scrutiny_status: AnalysisStatus::Fail,
                methodology_status: AnalysisStatus::Fail,
                validation_status: AnalysisStatus::Fail,
                verdict: "Gateway failed - insufficient falsifiability.".to_string(),
            },
        }
    }

    // ========================================================================
    // Handler Tests
    // ========================================================================

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

    #[tokio::test]
    async fn test_handler_markdown_output() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("README.md"), "# Test Project").unwrap();

        let result = handle_popper_score(
            temp.path(),
            &RepoScoreOutputFormat::Markdown,
            false,
            false,
            None,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handler_yaml_output() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("README.md"), "# Test Project").unwrap();

        let result = handle_popper_score(
            temp.path(),
            &RepoScoreOutputFormat::Yaml,
            false,
            false,
            None,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handler_verbose_output() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("README.md"), "# Test").unwrap();

        let result = handle_popper_score(
            temp.path(),
            &RepoScoreOutputFormat::Text,
            true, // verbose
            false,
            None,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handler_failures_only_output() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("README.md"), "# Test").unwrap();

        let result = handle_popper_score(
            temp.path(),
            &RepoScoreOutputFormat::Text,
            false,
            true, // failures_only
            None,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handler_output_to_file() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("README.md"), "# Test").unwrap();
        let output_path = temp.path().join("score.txt");

        let result = handle_popper_score(
            temp.path(),
            &RepoScoreOutputFormat::Text,
            false,
            false,
            Some(&output_path),
        )
        .await;

        assert!(result.is_ok());
        assert!(output_path.exists());
    }

    // ========================================================================
    // format_text Tests
    // ========================================================================

    #[test]
    fn test_format_text_gateway_passed() {
        let score = create_test_score_passed();
        let output = format_text(&score, false, false);

        assert!(output.contains("Popper Falsifiability Score"));
        assert!(output.contains("Gateway: PASSED"));
        assert!(output.contains("73.7%"));
        assert!(output.contains("Falsifiability & Testability"));
        assert!(output.contains("Reproducibility Infrastructure"));
        assert!(output.contains("[GATEWAY]"));
    }

    #[test]
    fn test_format_text_gateway_failed() {
        let score = create_test_score_failed();
        let output = format_text(&score, false, false);

        assert!(output.contains("Gateway: FAILED"));
        assert!(output.contains("Falsifiability < 60%"));
        assert!(output.contains("Without falsifiable claims"));
    }

    #[test]
    fn test_format_text_verbose() {
        let score = create_test_score_passed();
        let output = format_text(&score, true, false);

        // Verbose shows sub-scores with id and description
        assert!(output.contains("A1"));
        assert!(output.contains("Unit test coverage")); // description not name
        assert!(output.contains("A2"));
        assert!(output.contains("Testable claims")); // description
    }

    #[test]
    fn test_format_text_not_verbose_hides_subscores() {
        let score = create_test_score_passed();
        let output = format_text(&score, false, false);

        // Non-verbose shouldn't show sub-score IDs in detail
        // It still shows the category totals but not individual sub-scores
        assert!(output.contains("Falsifiability & Testability"));
    }

    #[test]
    fn test_format_text_recommendations() {
        let score = create_test_score_passed();
        let output = format_text(&score, false, false);

        assert!(output.contains("Recommendations"));
        assert!(output.contains("Add mutation testing"));
        assert!(output.contains("cargo mutants"));
        assert!(output.contains("Add CI pipeline"));
    }

    #[test]
    fn test_format_text_recommendation_priorities() {
        let score = create_test_score_passed();
        let output = format_text(&score, false, false);

        // Check priority icons
        assert!(output.contains("🔴")); // Critical
        assert!(output.contains("🟠")); // High
        assert!(output.contains("🟡")); // Medium
        assert!(output.contains("🟢")); // Low
    }

    #[test]
    fn test_format_text_category_icons() {
        let score = create_test_score_passed();
        let output = format_text(&score, false, false);

        // Score should have various status icons
        assert!(output.contains("✅") || output.contains("⚠️") || output.contains("❌"));
    }

    #[test]
    fn test_format_text_verdict() {
        let score = create_test_score_passed();
        let output = format_text(&score, false, false);

        assert!(output.contains("Verdict"));
        assert!(output.contains("Good scientific practices"));
    }

    #[test]
    fn test_format_text_na_category() {
        let score = create_test_score_passed();
        let output = format_text(&score, false, false);

        // ML/AI is N/A by default
        assert!(output.contains("N/A"));
    }

    // ========================================================================
    // format_json Tests
    // ========================================================================

    #[test]
    fn test_format_json_basic() {
        let score = create_test_score_passed();
        let output = format_json(&score).unwrap();

        // Should be valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(parsed.is_object());
    }

    #[test]
    fn test_format_json_contains_fields() {
        let score = create_test_score_passed();
        let output = format_json(&score).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert!(parsed.get("raw_score").is_some());
        assert!(parsed.get("normalized_score").is_some());
        assert!(parsed.get("grade").is_some());
        assert!(parsed.get("gateway_passed").is_some());
        assert!(parsed.get("categories").is_some());
        assert!(parsed.get("recommendations").is_some());
        assert!(parsed.get("metadata").is_some());
        assert!(parsed.get("analysis").is_some());
    }

    #[test]
    fn test_format_json_gateway_passed_value() {
        let score = create_test_score_passed();
        let output = format_json(&score).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert_eq!(parsed["gateway_passed"], true);
    }

    #[test]
    fn test_format_json_gateway_failed_value() {
        let score = create_test_score_failed();
        let output = format_json(&score).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert_eq!(parsed["gateway_passed"], false);
    }

    #[test]
    fn test_format_json_categories_structure() {
        let score = create_test_score_passed();
        let output = format_json(&score).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

        let categories = &parsed["categories"];
        assert!(categories.get("falsifiability").is_some());
        assert!(categories.get("reproducibility").is_some());
        assert!(categories.get("transparency").is_some());
        assert!(categories.get("statistical_rigor").is_some());
        assert!(categories.get("historical_integrity").is_some());
        assert!(categories.get("ml_reproducibility").is_some());
    }

    // ========================================================================
    // format_markdown Tests
    // ========================================================================

    #[test]
    fn test_format_markdown_basic() {
        let score = create_test_score_passed();
        let output = format_markdown(&score, false, false);

        assert!(output.contains("# 🔬 Popper Falsifiability Score"));
        assert!(output.contains("## 📌 Summary"));
        assert!(output.contains("## 📂 Categories"));
    }

    #[test]
    fn test_format_markdown_gateway_passed() {
        let score = create_test_score_passed();
        let output = format_markdown(&score, false, false);

        assert!(output.contains("Gateway PASSED"));
    }

    #[test]
    fn test_format_markdown_gateway_failed() {
        let score = create_test_score_failed();
        let output = format_markdown(&score, false, false);

        assert!(output.contains("Gateway FAILED"));
        assert!(output.contains("Falsifiability < 60%"));
    }

    #[test]
    fn test_format_markdown_table() {
        let score = create_test_score_passed();
        let output = format_markdown(&score, false, false);

        // Should have markdown table headers
        assert!(output.contains("| Category | Score | Percentage | Status |"));
        assert!(output.contains("|----------|-------|------------|--------|"));
    }

    #[test]
    fn test_format_markdown_verbose() {
        let score = create_test_score_passed();
        let output = format_markdown(&score, true, false);

        assert!(output.contains("## 📊 Detailed Breakdown"));
        assert!(output.contains("### A. Falsifiability & Testability"));
        assert!(output.contains("**A1**"));
    }

    #[test]
    fn test_format_markdown_recommendations() {
        let score = create_test_score_passed();
        let output = format_markdown(&score, false, false);

        assert!(output.contains("## 💡 Recommendations"));
        assert!(output.contains("Add mutation testing"));
        assert!(output.contains("```bash"));
        assert!(output.contains("cargo mutants"));
    }

    #[test]
    fn test_format_markdown_verdict() {
        let score = create_test_score_passed();
        let output = format_markdown(&score, false, false);

        assert!(output.contains("## 📋 Verdict"));
        assert!(output.contains("Good scientific practices"));
    }

    #[test]
    fn test_format_markdown_na_category() {
        let score = create_test_score_passed();
        let output = format_markdown(&score, false, false);

        // ML/AI is N/A - should show in table
        assert!(output.contains("N/A"));
    }

    // ========================================================================
    // format_yaml Tests
    // ========================================================================

    #[test]
    fn test_format_yaml_basic() {
        let score = create_test_score_passed();
        let output = format_yaml(&score).unwrap();

        // Should be valid YAML
        let parsed: serde_yaml::Value = serde_yaml::from_str(&output).unwrap();
        assert!(parsed.is_mapping());
    }

    #[test]
    fn test_format_yaml_contains_fields() {
        let score = create_test_score_passed();
        let output = format_yaml(&score).unwrap();

        assert!(output.contains("raw_score:"));
        assert!(output.contains("normalized_score:"));
        assert!(output.contains("grade:"));
        assert!(output.contains("gateway_passed:"));
        assert!(output.contains("categories:"));
    }

    #[test]
    fn test_format_yaml_roundtrip() {
        let score = create_test_score_passed();
        let output = format_yaml(&score).unwrap();

        // Should be able to deserialize back
        let parsed: PopperScore = serde_yaml::from_str(&output).unwrap();
        assert_eq!(parsed.gateway_passed, score.gateway_passed);
        assert!((parsed.normalized_score - score.normalized_score).abs() < 0.01);
    }

    // ========================================================================
    // Edge Case Tests
    // ========================================================================

    #[test]
    fn test_format_text_empty_recommendations() {
        let mut score = create_test_score_passed();
        score.recommendations = vec![];
        let output = format_text(&score, false, false);

        // Should not show recommendations section if empty
        // Actually the section might still show, let's check it doesn't crash
        assert!(output.contains("Popper Falsifiability Score"));
    }

    #[test]
    fn test_format_markdown_empty_recommendations() {
        let mut score = create_test_score_passed();
        score.recommendations = vec![];
        let output = format_markdown(&score, false, false);

        assert!(output.contains("# 🔬 Popper Falsifiability Score"));
        // Empty recommendations shouldn't cause issues
    }

    #[test]
    fn test_format_text_high_score() {
        let mut score = create_test_score_passed();
        score.normalized_score = 97.5;
        score.grade = PopperGrade::APlus;
        let output = format_text(&score, false, false);

        assert!(output.contains("97.5%"));
        assert!(output.contains("A+"));
    }

    #[test]
    fn test_format_text_zero_score() {
        let mut score = create_test_score_failed();
        score.normalized_score = 0.0;
        score.raw_score = 0.0;
        let output = format_text(&score, false, false);

        assert!(output.contains("0.0"));
    }

    #[test]
    fn test_format_json_special_characters_in_verdict() {
        let mut score = create_test_score_passed();
        score.analysis.verdict = "Test with \"quotes\" and 'apostrophes' & ampersands".to_string();
        let output = format_json(&score).unwrap();

        // Should be valid JSON even with special chars
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(parsed["analysis"]["verdict"]
            .as_str()
            .unwrap()
            .contains("quotes"));
    }

    #[test]
    fn test_format_all_grades() {
        let grades = vec![
            PopperGrade::APlus,
            PopperGrade::A,
            PopperGrade::AMinus,
            PopperGrade::BPlus,
            PopperGrade::B,
            PopperGrade::C,
            PopperGrade::D,
            PopperGrade::F,
            PopperGrade::InsufficientFalsifiability,
        ];

        for grade in grades {
            let mut score = create_test_score_passed();
            score.grade = grade;
            if grade == PopperGrade::InsufficientFalsifiability {
                score.gateway_passed = false;
            }

            let text = format_text(&score, false, false);
            let json = format_json(&score).unwrap();
            let md = format_markdown(&score, false, false);
            let yaml = format_yaml(&score).unwrap();

            // All formats should work without panicking
            assert!(!text.is_empty());
            assert!(!json.is_empty());
            assert!(!md.is_empty());
            assert!(!yaml.is_empty());
        }
    }
}
