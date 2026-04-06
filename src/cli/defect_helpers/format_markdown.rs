#![cfg_attr(coverage_nightly, coverage(off))]
//! Summary and Markdown output formatting for defect predictions

use crate::services::defect_probability::DefectScore;
use anyhow::Result;
use std::fmt::Write;

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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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

/// Write summary section (cognitive complexity <=3)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn write_summary_section(
    output: &mut String,
    predictions: &[(String, DefectScore)],
) -> Result<()> {
    writeln!(output, "## Summary\n")?;
    writeln!(output, "**Total files analyzed**: {}", predictions.len())?;
    Ok(())
}

/// Write risk distribution table (cognitive complexity <=8)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn write_risk_distribution_table(
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

/// Calculate risk counts (cognitive complexity <=6)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
pub(crate) fn calculate_risk_counts(
    predictions: &[(String, DefectScore)],
) -> (usize, usize, usize) {
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

/// Write a single risk row (cognitive complexity <=3)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn write_risk_row(
    output: &mut String,
    label: &str,
    count: usize,
    total: f64,
) -> Result<()> {
    writeln!(
        output,
        "| {} | {} | {:.1}% |",
        label,
        count,
        (count as f64 / total) * 100.0
    )?;
    Ok(())
}

/// Write detailed predictions section (cognitive complexity <=7)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn write_detailed_predictions(
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

/// Write a single prediction (cognitive complexity <=8)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn write_single_prediction(
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

/// Write prediction metrics (cognitive complexity <=4)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn write_prediction_metrics(output: &mut String, score: &DefectScore) -> Result<()> {
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

/// Write recommendations based on probability (cognitive complexity <=7)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn write_recommendations(output: &mut String, probability: f64) -> Result<()> {
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
