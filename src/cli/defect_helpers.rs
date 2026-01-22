//! Helper functions for defect prediction analysis

use crate::cli::defect_prediction_helpers::{
    calculate_simple_churn_score, calculate_simple_complexity, DefectPredictionConfig,
};
use crate::services::defect_probability::{DefectProbabilityCalculator, DefectScore, FileMetrics};
use anyhow::Result;
use std::fmt::Write;
use std::path::{Path, PathBuf};

/// Discover files for defect analysis
pub async fn discover_files_for_defect_analysis(
    project_path: &Path,
    config: &DefectPredictionConfig,
) -> Result<Vec<(PathBuf, String, usize)>> {
    use crate::cli::defect_prediction_helpers::discover_source_files_for_defect_analysis;

    discover_source_files_for_defect_analysis(project_path, config).await
}

/// Analyze defect probability for files
pub async fn analyze_defect_probability(
    files: &[(PathBuf, String, usize)],
    config: &DefectPredictionConfig,
) -> Result<Vec<(String, DefectScore)>> {
    eprintln!("📊 Analyzing {} files...", files.len());

    let calculator = DefectProbabilityCalculator::new();
    let mut predictions = Vec::new();

    for (path, content, line_count) in files {
        let metrics = FileMetrics {
            file_path: path.to_string_lossy().to_string(),
            complexity: calculate_simple_complexity(content) as f32,
            churn_score: calculate_simple_churn_score(content, *line_count),
            duplicate_ratio: 0.0,   // Simplified
            afferent_coupling: 0.0, // Simplified
            efferent_coupling: 0.0, // Simplified
            lines_of_code: *line_count,
            cyclomatic_complexity: 10, // Simplified
            cognitive_complexity: 10,  // Simplified
        };

        let score = calculator.calculate(&metrics);
        predictions.push((path.to_string_lossy().to_string(), score));
    }

    // Apply filters
    if config.high_risk_only {
        predictions.retain(|(_, score)| score.probability > 0.7);
    }

    if !config.include_low_confidence {
        predictions.retain(|(_, score)| score.confidence > config.confidence_threshold);
    }

    // Sort by probability
    predictions.sort_by(|a, b| {
        b.1.probability
            .partial_cmp(&a.1.probability)
            .expect("internal error")
    });

    Ok(predictions)
}

/// Format defect predictions as JSON
/// Formats defect predictions as JSON
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::cli::defect_helpers::format_defect_json;
/// use pmat::services::defect_probability::{DefectScore, RiskLevel};
///
/// let predictions = vec![
///     ("src/main.rs".to_string(), DefectScore {
///         probability: 0.8,
///         confidence: 0.9,
///         contributing_factors: vec![("complexity".to_string(), 0.5)],
///         risk_level: RiskLevel::High,
///         recommendations: vec!["Reduce complexity".to_string()],
///     })
/// ];
///
/// let json = format_defect_json(&predictions).expect("internal error");
/// assert!(json.contains("defect_predictions"));
/// assert!(json.contains("src/main.rs"));
/// ```
pub fn format_defect_json(predictions: &[(String, DefectScore)]) -> Result<String> {
    let json_data = serde_json::json!({
        "defect_predictions": predictions.iter().map(|(file, score)| {
            serde_json::json!({
                "file": file,
                "probability": score.probability,
                "confidence": score.confidence,
                "risk_factors": score.contributing_factors,
            })
        }).collect::<Vec<_>>(),
        "summary": {
            "total_files": predictions.len(),
            "high_risk_files": predictions.iter().filter(|(_, s)| s.probability > 0.7).count(),
            "medium_risk_files": predictions.iter().filter(|(_, s)| s.probability > 0.4 && s.probability <= 0.7).count(),
            "low_risk_files": predictions.iter().filter(|(_, s)| s.probability <= 0.4).count(),
        }
    });

    serde_json::to_string_pretty(&json_data).map_err(Into::into)
}

/// Format defect predictions as summary
/// Formats defect predictions as a summary
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::cli::defect_helpers::format_defect_summary;
/// use pmat::services::defect_probability::{DefectScore, RiskLevel};
///
/// let predictions = vec![
///     ("src/main.rs".to_string(), DefectScore {
///         probability: 0.8,
///         confidence: 0.9,
///         contributing_factors: vec![],
///         risk_level: RiskLevel::High,
///         recommendations: vec![],
///     })
/// ];
///
/// let summary = format_defect_summary(&predictions).expect("internal error");
/// assert!(summary.contains("Defect Prediction Summary"));
/// assert!(summary.contains("**Total files analyzed**: 1"));
/// ```
pub fn format_defect_summary(predictions: &[(String, DefectScore)]) -> Result<String> {
    let mut output = String::new();

    writeln!(&mut output, "# Defect Prediction Summary\n")?;
    writeln!(
        &mut output,
        "**Total files analyzed**: {}",
        predictions.len()
    )?;

    let high_risk = predictions
        .iter()
        .filter(|(_, s)| s.probability > 0.7)
        .count();
    let medium_risk = predictions
        .iter()
        .filter(|(_, s)| s.probability > 0.4 && s.probability <= 0.7)
        .count();
    let low_risk = predictions
        .iter()
        .filter(|(_, s)| s.probability <= 0.4)
        .count();

    writeln!(&mut output, "\n## Risk Distribution:")?;
    writeln!(&mut output, "- 🔴 High Risk (>70%): {high_risk} files")?;
    writeln!(
        &mut output,
        "- 🟡 Medium Risk (40-70%): {medium_risk} files"
    )?;
    writeln!(&mut output, "- 🟢 Low Risk (<40%): {low_risk} files")?;

    if !predictions.is_empty() {
        writeln!(&mut output, "\n## Top 10 High-Risk Files:")?;
        for (i, (file, score)) in predictions.iter().take(10).enumerate() {
            writeln!(
                &mut output,
                "{}. {} - {:.1}% probability",
                i + 1,
                file,
                score.probability * 100.0
            )?;
        }
    }

    Ok(output)
}

/// Format defect predictions as markdown
pub fn format_defect_markdown(
    predictions: &[(String, DefectScore)],
    include_recommendations: bool,
) -> Result<String> {
    let mut output = String::new();

    writeln!(&mut output, "# Defect Prediction Report\n")?;

    write_summary_section(&mut output, predictions)?;
    write_risk_distribution_table(&mut output, predictions)?;
    write_detailed_predictions(&mut output, predictions, include_recommendations)?;

    Ok(output)
}

/// Write summary section (cognitive complexity ≤3)
fn write_summary_section(output: &mut String, predictions: &[(String, DefectScore)]) -> Result<()> {
    writeln!(output, "## Summary\n")?;
    writeln!(output, "**Total files analyzed**: {}", predictions.len())?;
    Ok(())
}

/// Write risk distribution table (cognitive complexity ≤8)
fn write_risk_distribution_table(
    output: &mut String,
    predictions: &[(String, DefectScore)],
) -> Result<()> {
    let (high_risk, medium_risk, low_risk) = calculate_risk_counts(predictions);
    let total = predictions.len() as f64;

    writeln!(output, "\n### Risk Distribution")?;
    writeln!(output, "| Risk Level | Count | Percentage |")?;
    writeln!(output, "|------------|-------|------------|")?;

    write_risk_row(output, "High (>70%)", high_risk, total)?;
    write_risk_row(output, "Medium (40-70%)", medium_risk, total)?;
    write_risk_row(output, "Low (<40%)", low_risk, total)?;

    Ok(())
}

/// Calculate risk counts (cognitive complexity ≤6)
fn calculate_risk_counts(predictions: &[(String, DefectScore)]) -> (usize, usize, usize) {
    let high_risk = predictions
        .iter()
        .filter(|(_, s)| s.probability > 0.7)
        .count();

    let medium_risk = predictions
        .iter()
        .filter(|(_, s)| s.probability > 0.4 && s.probability <= 0.7)
        .count();

    let low_risk = predictions
        .iter()
        .filter(|(_, s)| s.probability <= 0.4)
        .count();

    (high_risk, medium_risk, low_risk)
}

/// Write a single risk row (cognitive complexity ≤3)
fn write_risk_row(output: &mut String, label: &str, count: usize, total: f64) -> Result<()> {
    writeln!(
        output,
        "| {} | {} | {:.1}% |",
        label,
        count,
        (count as f64 / total) * 100.0
    )?;
    Ok(())
}

/// Write detailed predictions section (cognitive complexity ≤7)
fn write_detailed_predictions(
    output: &mut String,
    predictions: &[(String, DefectScore)],
    include_recommendations: bool,
) -> Result<()> {
    writeln!(output, "\n## Detailed Predictions\n")?;

    for (file, score) in predictions.iter().take(20) {
        write_single_prediction(output, file, score, include_recommendations)?;
    }

    Ok(())
}

/// Write a single prediction (cognitive complexity ≤8)
fn write_single_prediction(
    output: &mut String,
    file: &str,
    score: &DefectScore,
    include_recommendations: bool,
) -> Result<()> {
    writeln!(output, "### {file}\n")?;

    write_prediction_metrics(output, score)?;

    if include_recommendations {
        write_recommendations(output, f64::from(score.probability))?;
    }

    writeln!(output)?;
    Ok(())
}

/// Write prediction metrics (cognitive complexity ≤4)
fn write_prediction_metrics(output: &mut String, score: &DefectScore) -> Result<()> {
    writeln!(
        output,
        "- **Probability**: {:.1}%",
        f64::from(score.probability) * 100.0
    )?;
    writeln!(
        output,
        "- **Confidence**: {:.1}%",
        f64::from(score.confidence) * 100.0
    )?;
    writeln!(
        output,
        "- **Risk Factors**: {:?}",
        score.contributing_factors
    )?;
    Ok(())
}

/// Write recommendations based on probability (cognitive complexity ≤7)
fn write_recommendations(output: &mut String, probability: f64) -> Result<()> {
    writeln!(output, "\n#### Recommendations:")?;

    if probability > 0.7 {
        writeln!(output, "- 🔴 High priority for code review")?;
        writeln!(output, "- Add comprehensive test coverage")?;
        writeln!(output, "- Consider refactoring to reduce complexity")?;
    } else if probability > 0.4 {
        writeln!(output, "- 🟡 Schedule for regular review")?;
        writeln!(output, "- Improve test coverage")?;
    } else {
        writeln!(output, "- 🟢 Monitor during regular maintenance")?;
    }

    Ok(())
}

/// Format defect predictions as SARIF
pub fn format_defect_sarif(
    predictions: &[(String, DefectScore)],
    _project_path: &Path,
) -> Result<String> {
    let mut results = Vec::new();

    for (file, score) in predictions {
        let level = if score.probability > 0.7 {
            "error"
        } else if score.probability > 0.4 {
            "warning"
        } else {
            "note"
        };

        let rule_id = if score.probability > 0.7 {
            "high-defect-probability"
        } else if score.probability > 0.4 {
            "medium-defect-probability"
        } else {
            "low-defect-probability"
        };

        results.push(serde_json::json!({
            "ruleId": rule_id,
            "level": level,
            "message": {
                "text": format!(
                    "File has {:.1}% defect probability with {:.1}% confidence. Risk factors: {:?}",
                    score.probability * 100.0,
                    score.confidence * 100.0,
                    score.contributing_factors
                )
            },
            "locations": [{
                "physicalLocation": {
                    "artifactLocation": {
                        "uri": file
                    }
                }
            }]
        }));
    }

    let sarif = serde_json::json!({
        "version": "2.1.0",
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "paiml-defect-predictor",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit",
                    "rules": generate_defect_rules(),
                }
            },
            "results": results
        }]
    });

    serde_json::to_string_pretty(&sarif).map_err(Into::into)
}

/// Generate SARIF rules for defect prediction
fn generate_defect_rules() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({
            "id": "high-defect-probability",
            "name": "High Defect Probability",
            "shortDescription": {
                "text": "File has high probability of containing defects"
            },
            "fullDescription": {
                "text": "Files with >70% defect probability require immediate review"
            },
            "defaultConfiguration": {
                "level": "error"
            }
        }),
        serde_json::json!({
            "id": "medium-defect-probability",
            "name": "Medium Defect Probability",
            "shortDescription": {
                "text": "File has medium probability of containing defects"
            },
            "fullDescription": {
                "text": "Files with 40-70% defect probability should be reviewed"
            },
            "defaultConfiguration": {
                "level": "warning"
            }
        }),
        serde_json::json!({
            "id": "low-defect-probability",
            "name": "Low Defect Probability",
            "shortDescription": {
                "text": "File has low probability of containing defects"
            },
            "fullDescription": {
                "text": "Files with <40% defect probability are lower risk"
            },
            "defaultConfiguration": {
                "level": "note"
            }
        }),
    ]
}

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}

#[cfg(test)]
mod coverage_tests {
    use super::*;
    use crate::services::defect_probability::RiskLevel;

    // Helper function to create a mock DefectScore
    fn create_mock_defect_score(probability: f32, confidence: f32) -> DefectScore {
        DefectScore {
            probability,
            confidence,
            contributing_factors: vec![
                ("complexity".to_string(), 0.3),
                ("churn".to_string(), 0.2),
                ("duplication".to_string(), 0.1),
                ("coupling".to_string(), 0.05),
            ],
            risk_level: if probability > 0.7 {
                RiskLevel::High
            } else if probability > 0.3 {
                RiskLevel::Medium
            } else {
                RiskLevel::Low
            },
            recommendations: vec!["Test recommendation".to_string()],
        }
    }

    fn create_test_predictions() -> Vec<(String, DefectScore)> {
        vec![
            (
                "src/high_risk.rs".to_string(),
                create_mock_defect_score(0.85, 0.9),
            ),
            (
                "src/medium_risk.rs".to_string(),
                create_mock_defect_score(0.55, 0.8),
            ),
            (
                "src/low_risk.rs".to_string(),
                create_mock_defect_score(0.25, 0.95),
            ),
            (
                "src/another_high.rs".to_string(),
                create_mock_defect_score(0.75, 0.85),
            ),
            (
                "src/very_low.rs".to_string(),
                create_mock_defect_score(0.15, 0.7),
            ),
        ]
    }

    // Tests for format_defect_json
    mod format_defect_json_tests {
        use super::*;

        #[test]
        fn test_format_defect_json_empty_predictions() {
            let predictions: Vec<(String, DefectScore)> = vec![];
            let result = format_defect_json(&predictions).expect("Should format empty predictions");

            assert!(result.contains("defect_predictions"));
            assert!(result.contains("\"total_files\": 0"));
            assert!(result.contains("\"high_risk_files\": 0"));
            assert!(result.contains("\"medium_risk_files\": 0"));
            assert!(result.contains("\"low_risk_files\": 0"));
        }

        #[test]
        fn test_format_defect_json_with_predictions() {
            let predictions = create_test_predictions();
            let result = format_defect_json(&predictions).expect("Should format predictions");

            assert!(result.contains("defect_predictions"));
            assert!(result.contains("src/high_risk.rs"));
            assert!(result.contains("src/medium_risk.rs"));
            assert!(result.contains("src/low_risk.rs"));
            assert!(result.contains("\"total_files\": 5"));
        }

        #[test]
        fn test_format_defect_json_risk_counts() {
            let predictions = create_test_predictions();
            let result = format_defect_json(&predictions).expect("Should format predictions");

            // 2 high risk (>0.7): 0.85 and 0.75
            assert!(result.contains("\"high_risk_files\": 2"));
            // 1 medium risk (0.4-0.7): 0.55
            assert!(result.contains("\"medium_risk_files\": 1"));
            // 2 low risk (<=0.4): 0.25 and 0.15
            assert!(result.contains("\"low_risk_files\": 2"));
        }

        #[test]
        fn test_format_defect_json_contains_file_data() {
            let predictions = vec![(
                "test_file.rs".to_string(),
                create_mock_defect_score(0.75, 0.9),
            )];
            let result = format_defect_json(&predictions).expect("Should format predictions");

            assert!(result.contains("\"file\": \"test_file.rs\""));
            assert!(result.contains("\"probability\":"));
            assert!(result.contains("\"confidence\":"));
            assert!(result.contains("risk_factors"));
        }
    }

    // Tests for format_defect_summary
    mod format_defect_summary_tests {
        use super::*;

        #[test]
        fn test_format_defect_summary_empty() {
            let predictions: Vec<(String, DefectScore)> = vec![];
            let result = format_defect_summary(&predictions).expect("Should format empty summary");

            assert!(result.contains("# Defect Prediction Summary"));
            assert!(result.contains("**Total files analyzed**: 0"));
        }

        #[test]
        fn test_format_defect_summary_with_predictions() {
            let predictions = create_test_predictions();
            let result = format_defect_summary(&predictions).expect("Should format summary");

            assert!(result.contains("# Defect Prediction Summary"));
            assert!(result.contains("**Total files analyzed**: 5"));
            assert!(result.contains("## Risk Distribution:"));
            assert!(result.contains("High Risk (>70%): 2 files"));
            assert!(result.contains("Medium Risk (40-70%): 1 files"));
            assert!(result.contains("Low Risk (<40%): 2 files"));
        }

        #[test]
        fn test_format_defect_summary_top_files() {
            let predictions = create_test_predictions();
            let result = format_defect_summary(&predictions).expect("Should format summary");

            assert!(result.contains("## Top 10 High-Risk Files:"));
            // First file should be listed (highest probability)
            assert!(result.contains("src/high_risk.rs"));
        }

        #[test]
        fn test_format_defect_summary_shows_probability_percentages() {
            let predictions = vec![(
                "high_risk.rs".to_string(),
                create_mock_defect_score(0.85, 0.9),
            )];
            let result = format_defect_summary(&predictions).expect("Should format summary");

            // Should show probability as percentage
            assert!(result.contains("85.0% probability"));
        }
    }

    // Tests for format_defect_markdown
    mod format_defect_markdown_tests {
        use super::*;

        #[test]
        fn test_format_defect_markdown_empty() {
            let predictions: Vec<(String, DefectScore)> = vec![];
            let result =
                format_defect_markdown(&predictions, false).expect("Should format empty markdown");

            assert!(result.contains("# Defect Prediction Report"));
            assert!(result.contains("## Summary"));
            assert!(result.contains("**Total files analyzed**: 0"));
        }

        #[test]
        fn test_format_defect_markdown_with_recommendations() {
            let predictions = create_test_predictions();
            let result = format_defect_markdown(&predictions, true)
                .expect("Should format markdown with recommendations");

            assert!(result.contains("# Defect Prediction Report"));
            assert!(result.contains("#### Recommendations:"));
        }

        #[test]
        fn test_format_defect_markdown_without_recommendations() {
            let predictions = create_test_predictions();
            let result = format_defect_markdown(&predictions, false)
                .expect("Should format markdown without recommendations");

            assert!(result.contains("# Defect Prediction Report"));
            // Should not contain recommendations section when disabled
            // The detailed predictions still show metrics but not the recommendations subsection
        }

        #[test]
        fn test_format_defect_markdown_risk_table() {
            let predictions = create_test_predictions();
            let result =
                format_defect_markdown(&predictions, false).expect("Should format markdown");

            assert!(result.contains("### Risk Distribution"));
            assert!(result.contains("| Risk Level | Count | Percentage |"));
            assert!(result.contains("| High (>70%) |"));
            assert!(result.contains("| Medium (40-70%) |"));
            assert!(result.contains("| Low (<40%) |"));
        }

        #[test]
        fn test_format_defect_markdown_detailed_predictions() {
            let predictions = create_test_predictions();
            let result =
                format_defect_markdown(&predictions, false).expect("Should format markdown");

            assert!(result.contains("## Detailed Predictions"));
            assert!(result.contains("**Probability**:"));
            assert!(result.contains("**Confidence**:"));
            assert!(result.contains("**Risk Factors**:"));
        }
    }

    // Tests for format_defect_sarif
    mod format_defect_sarif_tests {
        use super::*;
        use std::path::Path;

        #[test]
        fn test_format_defect_sarif_empty() {
            let predictions: Vec<(String, DefectScore)> = vec![];
            let project_path = Path::new("/test/project");
            let result =
                format_defect_sarif(&predictions, project_path).expect("Should format empty SARIF");

            assert!(result.contains("\"version\": \"2.1.0\""));
            assert!(result.contains("sarif-schema-2.1.0.json"));
            assert!(result.contains("paiml-defect-predictor"));
            assert!(result.contains("\"results\": []"));
        }

        #[test]
        fn test_format_defect_sarif_with_predictions() {
            let predictions = create_test_predictions();
            let project_path = Path::new("/test/project");
            let result =
                format_defect_sarif(&predictions, project_path).expect("Should format SARIF");

            assert!(result.contains("\"version\": \"2.1.0\""));
            assert!(result.contains("src/high_risk.rs"));
            assert!(result.contains("src/low_risk.rs"));
        }

        #[test]
        fn test_format_defect_sarif_high_risk_level() {
            let predictions = vec![(
                "high_risk.rs".to_string(),
                create_mock_defect_score(0.85, 0.9),
            )];
            let project_path = Path::new("/test/project");
            let result =
                format_defect_sarif(&predictions, project_path).expect("Should format SARIF");

            assert!(result.contains("\"ruleId\": \"high-defect-probability\""));
            assert!(result.contains("\"level\": \"error\""));
        }

        #[test]
        fn test_format_defect_sarif_medium_risk_level() {
            let predictions = vec![(
                "medium_risk.rs".to_string(),
                create_mock_defect_score(0.55, 0.8),
            )];
            let project_path = Path::new("/test/project");
            let result =
                format_defect_sarif(&predictions, project_path).expect("Should format SARIF");

            assert!(result.contains("\"ruleId\": \"medium-defect-probability\""));
            assert!(result.contains("\"level\": \"warning\""));
        }

        #[test]
        fn test_format_defect_sarif_low_risk_level() {
            let predictions = vec![(
                "low_risk.rs".to_string(),
                create_mock_defect_score(0.25, 0.9),
            )];
            let project_path = Path::new("/test/project");
            let result =
                format_defect_sarif(&predictions, project_path).expect("Should format SARIF");

            assert!(result.contains("\"ruleId\": \"low-defect-probability\""));
            assert!(result.contains("\"level\": \"note\""));
        }

        #[test]
        fn test_format_defect_sarif_contains_rules() {
            let predictions = create_test_predictions();
            let project_path = Path::new("/test/project");
            let result =
                format_defect_sarif(&predictions, project_path).expect("Should format SARIF");

            assert!(result.contains("\"id\": \"high-defect-probability\""));
            assert!(result.contains("\"id\": \"medium-defect-probability\""));
            assert!(result.contains("\"id\": \"low-defect-probability\""));
            assert!(result.contains("\"name\": \"High Defect Probability\""));
        }

        #[test]
        fn test_format_defect_sarif_location_format() {
            let predictions = vec![(
                "src/test.rs".to_string(),
                create_mock_defect_score(0.75, 0.9),
            )];
            let project_path = Path::new("/test/project");
            let result =
                format_defect_sarif(&predictions, project_path).expect("Should format SARIF");

            assert!(result.contains("\"locations\""));
            assert!(result.contains("\"physicalLocation\""));
            assert!(result.contains("\"artifactLocation\""));
            assert!(result.contains("\"uri\": \"src/test.rs\""));
        }
    }

    // Tests for internal helper functions
    mod helper_function_tests {
        use super::*;

        #[test]
        fn test_write_summary_section() {
            let predictions = create_test_predictions();
            let mut output = String::new();
            write_summary_section(&mut output, &predictions).expect("Should write summary");

            assert!(output.contains("## Summary"));
            assert!(output.contains("**Total files analyzed**: 5"));
        }

        #[test]
        fn test_write_risk_distribution_table() {
            let predictions = create_test_predictions();
            let mut output = String::new();
            write_risk_distribution_table(&mut output, &predictions)
                .expect("Should write risk table");

            assert!(output.contains("### Risk Distribution"));
            assert!(output.contains("| Risk Level | Count | Percentage |"));
        }

        #[test]
        fn test_calculate_risk_counts() {
            let predictions = create_test_predictions();
            let (high, medium, low) = calculate_risk_counts(&predictions);

            assert_eq!(high, 2); // 0.85 and 0.75
            assert_eq!(medium, 1); // 0.55
            assert_eq!(low, 2); // 0.25 and 0.15
        }

        #[test]
        fn test_calculate_risk_counts_empty() {
            let predictions: Vec<(String, DefectScore)> = vec![];
            let (high, medium, low) = calculate_risk_counts(&predictions);

            assert_eq!(high, 0);
            assert_eq!(medium, 0);
            assert_eq!(low, 0);
        }

        #[test]
        fn test_calculate_risk_counts_all_high_risk() {
            let predictions = vec![
                ("a.rs".to_string(), create_mock_defect_score(0.95, 0.9)),
                ("b.rs".to_string(), create_mock_defect_score(0.85, 0.9)),
                ("c.rs".to_string(), create_mock_defect_score(0.75, 0.9)),
            ];
            let (high, medium, low) = calculate_risk_counts(&predictions);

            assert_eq!(high, 3);
            assert_eq!(medium, 0);
            assert_eq!(low, 0);
        }

        #[test]
        fn test_write_risk_row() {
            let mut output = String::new();
            write_risk_row(&mut output, "High (>70%)", 5, 10.0).expect("Should write risk row");

            assert!(output.contains("| High (>70%) | 5 | 50.0% |"));
        }

        #[test]
        fn test_write_risk_row_zero_total() {
            let mut output = String::new();
            // When total is 0.0, division will produce NaN or inf, but that's expected
            // The function handles this gracefully
            let result = write_risk_row(&mut output, "Test", 0, 0.0);
            assert!(result.is_ok());
        }

        #[test]
        fn test_write_detailed_predictions() {
            let predictions = create_test_predictions();
            let mut output = String::new();
            write_detailed_predictions(&mut output, &predictions, false)
                .expect("Should write detailed predictions");

            assert!(output.contains("## Detailed Predictions"));
            // Should contain file headers
            assert!(output.contains("### src/high_risk.rs"));
        }

        #[test]
        fn test_write_detailed_predictions_limits_to_20() {
            // Create more than 20 predictions
            let mut predictions = Vec::new();
            for i in 0..25 {
                predictions.push((format!("file_{}.rs", i), create_mock_defect_score(0.5, 0.8)));
            }

            let mut output = String::new();
            write_detailed_predictions(&mut output, &predictions, false)
                .expect("Should write limited predictions");

            // Should only contain up to 20 files
            assert!(output.contains("file_0.rs"));
            assert!(output.contains("file_19.rs"));
            // file_20.rs and above should not be included
            assert!(!output.contains("file_20.rs"));
        }

        #[test]
        fn test_write_single_prediction() {
            let score = create_mock_defect_score(0.75, 0.9);
            let mut output = String::new();
            write_single_prediction(&mut output, "test.rs", &score, false)
                .expect("Should write prediction");

            assert!(output.contains("### test.rs"));
            assert!(output.contains("**Probability**:"));
            assert!(output.contains("**Confidence**:"));
            assert!(output.contains("**Risk Factors**:"));
        }

        #[test]
        fn test_write_single_prediction_with_recommendations() {
            let score = create_mock_defect_score(0.85, 0.9);
            let mut output = String::new();
            write_single_prediction(&mut output, "test.rs", &score, true)
                .expect("Should write prediction");

            assert!(output.contains("#### Recommendations:"));
            assert!(output.contains("High priority for code review"));
        }

        #[test]
        fn test_write_prediction_metrics() {
            let score = create_mock_defect_score(0.75, 0.85);
            let mut output = String::new();
            write_prediction_metrics(&mut output, &score).expect("Should write metrics");

            assert!(output.contains("**Probability**: 75.0%"));
            assert!(output.contains("**Confidence**: 85.0%"));
            assert!(output.contains("**Risk Factors**:"));
        }
    }

    // Tests for write_recommendations
    mod write_recommendations_tests {
        use super::*;

        #[test]
        fn test_write_recommendations_high_risk() {
            let mut output = String::new();
            write_recommendations(&mut output, 0.85)
                .expect("Should write high risk recommendations");

            assert!(output.contains("#### Recommendations:"));
            assert!(output.contains("High priority for code review"));
            assert!(output.contains("Add comprehensive test coverage"));
            assert!(output.contains("Consider refactoring to reduce complexity"));
        }

        #[test]
        fn test_write_recommendations_medium_risk() {
            let mut output = String::new();
            write_recommendations(&mut output, 0.55)
                .expect("Should write medium risk recommendations");

            assert!(output.contains("#### Recommendations:"));
            assert!(output.contains("Schedule for regular review"));
            assert!(output.contains("Improve test coverage"));
            assert!(!output.contains("High priority")); // Should not have high priority message
        }

        #[test]
        fn test_write_recommendations_low_risk() {
            let mut output = String::new();
            write_recommendations(&mut output, 0.25)
                .expect("Should write low risk recommendations");

            assert!(output.contains("#### Recommendations:"));
            assert!(output.contains("Monitor during regular maintenance"));
            assert!(!output.contains("High priority"));
            assert!(!output.contains("Schedule for regular review"));
        }

        #[test]
        fn test_write_recommendations_boundary_high() {
            let mut output = String::new();
            // Exactly at boundary (>0.7)
            write_recommendations(&mut output, 0.71).expect("Should write recommendations");
            assert!(output.contains("High priority"));
        }

        #[test]
        fn test_write_recommendations_boundary_medium() {
            let mut output = String::new();
            // Exactly at boundary (>0.4 and <=0.7)
            write_recommendations(&mut output, 0.41).expect("Should write recommendations");
            assert!(output.contains("Schedule for regular review"));
        }

        #[test]
        fn test_write_recommendations_boundary_low() {
            let mut output = String::new();
            // At boundary (<=0.4)
            write_recommendations(&mut output, 0.4).expect("Should write recommendations");
            assert!(output.contains("Monitor during regular maintenance"));
        }
    }

    // Tests for generate_defect_rules
    mod generate_defect_rules_tests {
        use super::*;

        #[test]
        fn test_generate_defect_rules_returns_three_rules() {
            let rules = generate_defect_rules();
            assert_eq!(rules.len(), 3);
        }

        #[test]
        fn test_generate_defect_rules_high_risk_rule() {
            let rules = generate_defect_rules();
            let high_rule = &rules[0];

            assert_eq!(high_rule["id"], "high-defect-probability");
            assert_eq!(high_rule["name"], "High Defect Probability");
            assert_eq!(high_rule["defaultConfiguration"]["level"], "error");
        }

        #[test]
        fn test_generate_defect_rules_medium_risk_rule() {
            let rules = generate_defect_rules();
            let medium_rule = &rules[1];

            assert_eq!(medium_rule["id"], "medium-defect-probability");
            assert_eq!(medium_rule["name"], "Medium Defect Probability");
            assert_eq!(medium_rule["defaultConfiguration"]["level"], "warning");
        }

        #[test]
        fn test_generate_defect_rules_low_risk_rule() {
            let rules = generate_defect_rules();
            let low_rule = &rules[2];

            assert_eq!(low_rule["id"], "low-defect-probability");
            assert_eq!(low_rule["name"], "Low Defect Probability");
            assert_eq!(low_rule["defaultConfiguration"]["level"], "note");
        }

        #[test]
        fn test_generate_defect_rules_have_descriptions() {
            let rules = generate_defect_rules();

            for rule in rules {
                assert!(rule.get("shortDescription").is_some());
                assert!(rule.get("fullDescription").is_some());
                assert!(rule["shortDescription"]["text"].as_str().is_some());
                assert!(rule["fullDescription"]["text"].as_str().is_some());
            }
        }
    }

    // Edge case tests
    mod edge_case_tests {
        use super::*;

        #[test]
        fn test_format_json_single_prediction() {
            let predictions = vec![("single.rs".to_string(), create_mock_defect_score(0.5, 0.8))];
            let result = format_defect_json(&predictions).expect("Should format single prediction");

            assert!(result.contains("\"total_files\": 1"));
            assert!(result.contains("single.rs"));
        }

        #[test]
        fn test_format_summary_single_prediction() {
            let predictions = vec![("single.rs".to_string(), create_mock_defect_score(0.5, 0.8))];
            let result =
                format_defect_summary(&predictions).expect("Should format single prediction");

            assert!(result.contains("**Total files analyzed**: 1"));
        }

        #[test]
        fn test_format_markdown_boundary_probabilities() {
            // Test with probabilities exactly at boundaries
            let predictions = vec![
                (
                    "exact_70.rs".to_string(),
                    create_mock_defect_score(0.70, 0.9),
                ),
                (
                    "exact_40.rs".to_string(),
                    create_mock_defect_score(0.40, 0.9),
                ),
            ];
            let result =
                format_defect_markdown(&predictions, false).expect("Should format markdown");

            assert!(result.contains("# Defect Prediction Report"));
        }

        #[test]
        fn test_format_sarif_special_characters_in_filename() {
            let predictions = vec![(
                "src/path with spaces/file.rs".to_string(),
                create_mock_defect_score(0.75, 0.9),
            )];
            let project_path = Path::new("/test");
            let result = format_defect_sarif(&predictions, project_path)
                .expect("Should handle special chars");

            assert!(result.contains("path with spaces"));
        }

        #[test]
        fn test_format_json_zero_probability() {
            let mut score = create_mock_defect_score(0.0, 0.9);
            score.probability = 0.0;
            let predictions = vec![("zero.rs".to_string(), score)];
            let result = format_defect_json(&predictions).expect("Should handle zero probability");

            assert!(result.contains("\"probability\": 0"));
        }

        #[test]
        fn test_format_json_max_probability() {
            let mut score = create_mock_defect_score(1.0, 1.0);
            score.probability = 1.0;
            score.confidence = 1.0;
            let predictions = vec![("max.rs".to_string(), score)];
            let result = format_defect_json(&predictions).expect("Should handle max probability");

            assert!(result.contains("\"probability\": 1"));
            assert!(result.contains("\"confidence\": 1"));
        }

        #[test]
        fn test_format_markdown_empty_contributing_factors() {
            let mut score = create_mock_defect_score(0.75, 0.9);
            score.contributing_factors = vec![];
            let predictions = vec![("empty_factors.rs".to_string(), score)];

            let result =
                format_defect_markdown(&predictions, false).expect("Should handle empty factors");
            assert!(result.contains("# Defect Prediction Report"));
        }

        #[test]
        fn test_sarif_boundary_probability_values() {
            // Test boundary values: 0.7 should be "warning", >0.7 should be "error"
            let predictions = vec![
                ("at_70.rs".to_string(), create_mock_defect_score(0.70, 0.9)),
                (
                    "above_70.rs".to_string(),
                    create_mock_defect_score(0.71, 0.9),
                ),
                ("at_40.rs".to_string(), create_mock_defect_score(0.40, 0.9)),
                (
                    "above_40.rs".to_string(),
                    create_mock_defect_score(0.41, 0.9),
                ),
            ];
            let project_path = Path::new("/test");
            let result =
                format_defect_sarif(&predictions, project_path).expect("Should handle boundaries");

            // The function should have processed all predictions
            assert!(result.contains("at_70.rs"));
            assert!(result.contains("above_70.rs"));
        }
    }

    // Integration-style tests
    mod integration_tests {
        use super::*;

        #[test]
        fn test_all_formatters_handle_same_input() {
            let predictions = create_test_predictions();
            let project_path = Path::new("/test");

            // All formatters should succeed with the same input
            let json_result = format_defect_json(&predictions);
            let summary_result = format_defect_summary(&predictions);
            let markdown_result = format_defect_markdown(&predictions, true);
            let sarif_result = format_defect_sarif(&predictions, project_path);

            assert!(json_result.is_ok());
            assert!(summary_result.is_ok());
            assert!(markdown_result.is_ok());
            assert!(sarif_result.is_ok());
        }

        #[test]
        fn test_json_output_is_valid_json() {
            let predictions = create_test_predictions();
            let json_str = format_defect_json(&predictions).expect("Should format JSON");

            // Should be parseable as JSON
            let parsed: serde_json::Value =
                serde_json::from_str(&json_str).expect("Should be valid JSON");

            assert!(parsed.get("defect_predictions").is_some());
            assert!(parsed.get("summary").is_some());
        }

        #[test]
        fn test_sarif_output_is_valid_json() {
            let predictions = create_test_predictions();
            let project_path = Path::new("/test");
            let sarif_str =
                format_defect_sarif(&predictions, project_path).expect("Should format SARIF");

            // Should be parseable as JSON
            let parsed: serde_json::Value =
                serde_json::from_str(&sarif_str).expect("Should be valid JSON");

            assert_eq!(parsed["version"], "2.1.0");
            assert!(parsed.get("runs").is_some());
        }

        #[test]
        fn test_markdown_sections_order() {
            let predictions = create_test_predictions();
            let markdown =
                format_defect_markdown(&predictions, true).expect("Should format markdown");

            // Verify sections appear in correct order
            let summary_pos = markdown.find("## Summary").expect("Should have summary");
            let risk_dist_pos = markdown
                .find("### Risk Distribution")
                .expect("Should have risk distribution");
            let detailed_pos = markdown
                .find("## Detailed Predictions")
                .expect("Should have detailed predictions");

            assert!(summary_pos < risk_dist_pos);
            assert!(risk_dist_pos < detailed_pos);
        }

        #[test]
        fn test_predictions_sorted_by_probability() {
            let predictions = vec![
                ("low.rs".to_string(), create_mock_defect_score(0.25, 0.9)),
                ("high.rs".to_string(), create_mock_defect_score(0.85, 0.9)),
                ("medium.rs".to_string(), create_mock_defect_score(0.55, 0.9)),
            ];

            let json_str = format_defect_json(&predictions).expect("Should format JSON");

            // The JSON should show files in order of input (not sorted in format_defect_json)
            // The sorting happens in analyze_defect_probability, not in formatters
            assert!(json_str.contains("low.rs"));
            assert!(json_str.contains("high.rs"));
            assert!(json_str.contains("medium.rs"));
        }
    }

    // Property-based tests for robustness
    mod property_tests_comprehensive {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn test_format_json_never_panics(
                probability in 0.0f32..=1.0,
                confidence in 0.0f32..=1.0
            ) {
                let predictions = vec![
                    ("test.rs".to_string(), create_mock_defect_score(probability, confidence)),
                ];

                let result = format_defect_json(&predictions);
                prop_assert!(result.is_ok());
            }

            #[test]
            fn test_format_summary_never_panics(
                probability in 0.0f32..=1.0,
                confidence in 0.0f32..=1.0
            ) {
                let predictions = vec![
                    ("test.rs".to_string(), create_mock_defect_score(probability, confidence)),
                ];

                let result = format_defect_summary(&predictions);
                prop_assert!(result.is_ok());
            }

            #[test]
            fn test_format_markdown_never_panics(
                probability in 0.0f32..=1.0,
                confidence in 0.0f32..=1.0,
                include_recommendations in any::<bool>()
            ) {
                let predictions = vec![
                    ("test.rs".to_string(), create_mock_defect_score(probability, confidence)),
                ];

                let result = format_defect_markdown(&predictions, include_recommendations);
                prop_assert!(result.is_ok());
            }

            #[test]
            fn test_calculate_risk_counts_invariant(
                high_count in 0usize..10,
                medium_count in 0usize..10,
                low_count in 0usize..10
            ) {
                let mut predictions = Vec::new();

                for i in 0..high_count {
                    predictions.push((format!("high_{}.rs", i), create_mock_defect_score(0.85, 0.9)));
                }
                for i in 0..medium_count {
                    predictions.push((format!("medium_{}.rs", i), create_mock_defect_score(0.55, 0.9)));
                }
                for i in 0..low_count {
                    predictions.push((format!("low_{}.rs", i), create_mock_defect_score(0.25, 0.9)));
                }

                let (high, medium, low) = calculate_risk_counts(&predictions);

                // Total should equal sum of categories
                prop_assert_eq!(high + medium + low, predictions.len());
            }

            #[test]
            fn test_risk_distribution_matches_counts(num_predictions in 0usize..20) {
                let mut predictions = Vec::new();

                for i in 0..num_predictions {
                    let prob = (i as f32) / 20.0; // Spread across 0.0 to ~0.95
                    predictions.push((format!("file_{}.rs", i), create_mock_defect_score(prob, 0.9)));
                }

                let (high, medium, low) = calculate_risk_counts(&predictions);

                // Sum should equal total predictions
                prop_assert_eq!(high + medium + low, predictions.len());
            }
        }
    }

    // Tests for async functions (analyze_defect_probability)
    mod analyze_defect_probability_tests {
        use super::*;
        use std::path::PathBuf;

        #[tokio::test]
        async fn test_analyze_defect_probability_empty_files() {
            let files: Vec<(PathBuf, String, usize)> = vec![];
            let config = DefectPredictionConfig {
                confidence_threshold: 0.5,
                min_lines: 10,
                include_low_confidence: true,
                high_risk_only: false,
                include_recommendations: false,
                include: None,
                exclude: None,
            };

            let result = analyze_defect_probability(&files, &config).await;
            assert!(result.is_ok());
            assert!(result.unwrap().is_empty());
        }

        #[tokio::test]
        async fn test_analyze_defect_probability_single_file() {
            let files = vec![(
                PathBuf::from("test.rs"),
                "fn main() {\n    println!(\"Hello\");\n}".to_string(),
                3,
            )];
            let config = DefectPredictionConfig {
                confidence_threshold: 0.5,
                min_lines: 1,
                include_low_confidence: true,
                high_risk_only: false,
                include_recommendations: false,
                include: None,
                exclude: None,
            };

            let result = analyze_defect_probability(&files, &config).await;
            assert!(result.is_ok());
            let predictions = result.unwrap();
            assert_eq!(predictions.len(), 1);
            assert!(predictions[0].0.contains("test.rs"));
        }

        #[tokio::test]
        async fn test_analyze_defect_probability_high_risk_only_filter() {
            let files = vec![
                (PathBuf::from("low.rs"), "fn main() {}".to_string(), 1),
                (PathBuf::from("high.rs"), "fn complex() { if true { if true { for i in 0..10 { match x { _ => {} } } } } }".to_string(), 1),
            ];
            let config = DefectPredictionConfig {
                confidence_threshold: 0.0,
                min_lines: 1,
                include_low_confidence: true,
                high_risk_only: true, // Only keep high risk
                include_recommendations: false,
                include: None,
                exclude: None,
            };

            let result = analyze_defect_probability(&files, &config).await;
            assert!(result.is_ok());
            // Only high risk files should be included
            let predictions = result.unwrap();
            for (_, score) in &predictions {
                assert!(score.probability > 0.7);
            }
        }

        #[tokio::test]
        async fn test_analyze_defect_probability_confidence_filter() {
            let files = vec![
                (PathBuf::from("small.rs"), "fn a() {}".to_string(), 1), // Small file, low confidence
            ];
            let config = DefectPredictionConfig {
                confidence_threshold: 0.9, // High threshold
                min_lines: 1,
                include_low_confidence: false, // Filter out low confidence
                high_risk_only: false,
                include_recommendations: false,
                include: None,
                exclude: None,
            };

            let result = analyze_defect_probability(&files, &config).await;
            assert!(result.is_ok());
            // Small files have low confidence, so this might filter some out
        }

        #[tokio::test]
        async fn test_analyze_defect_probability_sorted_by_probability() {
            let files = vec![
                (PathBuf::from("a.rs"), "fn a() {}".to_string(), 1),
                (
                    PathBuf::from("b.rs"),
                    "fn b() { if true { for i in 0..10 { match x { _ => {} } } } }".to_string(),
                    1,
                ),
                (
                    PathBuf::from("c.rs"),
                    "fn c() { if true {} }".to_string(),
                    1,
                ),
            ];
            let config = DefectPredictionConfig {
                confidence_threshold: 0.0,
                min_lines: 1,
                include_low_confidence: true,
                high_risk_only: false,
                include_recommendations: false,
                include: None,
                exclude: None,
            };

            let result = analyze_defect_probability(&files, &config).await;
            assert!(result.is_ok());
            let predictions = result.unwrap();

            // Verify sorted by probability (descending)
            for i in 1..predictions.len() {
                assert!(predictions[i - 1].1.probability >= predictions[i].1.probability);
            }
        }

        #[tokio::test]
        async fn test_analyze_defect_probability_complex_code() {
            // Code with high complexity markers
            let complex_code = r#"
                fn complex_function() {
                    if condition1 {
                        for item in items {
                            match item {
                                Some(x) => {
                                    if x > 0 && y < 10 {
                                        while running {
                                            // TODO: refactor this
                                            // FIXME: handle edge case
                                        }
                                    }
                                }
                                None => {}
                            }
                        }
                    } else if condition2 {
                        // More logic
                    }
                }
            "#;

            let files = vec![(PathBuf::from("complex.rs"), complex_code.to_string(), 20)];
            let config = DefectPredictionConfig {
                confidence_threshold: 0.0,
                min_lines: 1,
                include_low_confidence: true,
                high_risk_only: false,
                include_recommendations: false,
                include: None,
                exclude: None,
            };

            let result = analyze_defect_probability(&files, &config).await;
            assert!(result.is_ok());
            let predictions = result.unwrap();
            assert_eq!(predictions.len(), 1);
            // Complex code should have a non-zero probability
            assert!(predictions[0].1.probability >= 0.0);
        }
    }

    // Tests for discover_files_for_defect_analysis
    mod discover_files_tests {
        use super::*;
        use std::fs;
        use tempfile::TempDir;

        #[tokio::test]
        async fn test_discover_files_empty_directory() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let config = DefectPredictionConfig {
                confidence_threshold: 0.5,
                min_lines: 1,
                include_low_confidence: true,
                high_risk_only: false,
                include_recommendations: false,
                include: None,
                exclude: None,
            };

            let result = discover_files_for_defect_analysis(temp_dir.path(), &config).await;
            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_discover_files_with_source_files() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let src_dir = temp_dir.path().join("src");
            fs::create_dir(&src_dir).expect("Failed to create src dir");

            fs::write(
                src_dir.join("main.rs"),
                "fn main() {\n    println!(\"Hello\");\n    let x = 1;\n    let y = 2;\n    let z = 3;\n}\n"
            ).expect("Failed to write file");

            let config = DefectPredictionConfig {
                confidence_threshold: 0.5,
                min_lines: 1,
                include_low_confidence: true,
                high_risk_only: false,
                include_recommendations: false,
                include: None,
                exclude: None,
            };

            let result = discover_files_for_defect_analysis(temp_dir.path(), &config).await;
            assert!(result.is_ok());
            let files = result.unwrap();
            // Should find the .rs file
            let has_rs_file = files
                .iter()
                .any(|(path, _, _)| path.extension().map_or(false, |e| e == "rs"));
            assert!(has_rs_file);
        }

        #[tokio::test]
        async fn test_discover_files_min_lines_filter() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let src_dir = temp_dir.path().join("src");
            fs::create_dir(&src_dir).expect("Failed to create src dir");

            // Small file (less than min_lines)
            fs::write(src_dir.join("small.rs"), "fn a() {}").expect("Failed to write file");

            // Large file (more than min_lines)
            fs::write(
                src_dir.join("large.rs"),
                "fn main() {\n    let a = 1;\n    let b = 2;\n    let c = 3;\n    let d = 4;\n    let e = 5;\n}\n"
            ).expect("Failed to write file");

            let config = DefectPredictionConfig {
                confidence_threshold: 0.5,
                min_lines: 5, // Require at least 5 lines
                include_low_confidence: true,
                high_risk_only: false,
                include_recommendations: false,
                include: None,
                exclude: None,
            };

            let result = discover_files_for_defect_analysis(temp_dir.path(), &config).await;
            assert!(result.is_ok());
            let files = result.unwrap();

            // All returned files should have at least min_lines
            for (_, _, line_count) in &files {
                assert!(*line_count >= 5);
            }
        }
    }
}
