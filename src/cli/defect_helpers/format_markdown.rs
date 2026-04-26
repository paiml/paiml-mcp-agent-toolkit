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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::defect_probability::RiskLevel;

    fn score(probability: f32, confidence: f32, risk: RiskLevel) -> DefectScore {
        DefectScore {
            probability,
            confidence,
            contributing_factors: vec![("complexity".to_string(), 0.5)],
            risk_level: risk,
            recommendations: vec!["refactor".to_string()],
        }
    }

    fn pred(file: &str, probability: f32) -> (String, DefectScore) {
        (file.to_string(), score(probability, 0.9, RiskLevel::High))
    }

    // ── calculate_risk_counts ───────────────────────────────────────────────

    #[test]
    fn test_calculate_risk_counts_empty() {
        assert_eq!(calculate_risk_counts(&[]), (0, 0, 0));
    }

    #[test]
    fn test_calculate_risk_counts_single_high() {
        let p = vec![pred("a.rs", 0.9)];
        assert_eq!(calculate_risk_counts(&p), (1, 0, 0));
    }

    #[test]
    fn test_calculate_risk_counts_single_medium() {
        let p = vec![pred("a.rs", 0.5)];
        assert_eq!(calculate_risk_counts(&p), (0, 1, 0));
    }

    #[test]
    fn test_calculate_risk_counts_single_low() {
        let p = vec![pred("a.rs", 0.1)];
        assert_eq!(calculate_risk_counts(&p), (0, 0, 1));
    }

    #[test]
    fn test_calculate_risk_counts_boundary_at_07_is_medium() {
        // 0.7 falls into medium (>0.4 && <=0.7), not high (>0.7)
        let p = vec![pred("a.rs", 0.7)];
        assert_eq!(calculate_risk_counts(&p), (0, 1, 0));
    }

    #[test]
    fn test_calculate_risk_counts_boundary_at_04_is_low() {
        // 0.4 falls into low (<=0.4), not medium (>0.4)
        let p = vec![pred("a.rs", 0.4)];
        assert_eq!(calculate_risk_counts(&p), (0, 0, 1));
    }

    #[test]
    fn test_calculate_risk_counts_mixed() {
        let p = vec![
            pred("hi.rs", 0.95),
            pred("med.rs", 0.55),
            pred("med2.rs", 0.50),
            pred("lo.rs", 0.20),
            pred("lo2.rs", 0.0),
        ];
        assert_eq!(calculate_risk_counts(&p), (1, 2, 2));
    }

    // ── write_summary_section ───────────────────────────────────────────────

    #[test]
    fn test_write_summary_section_includes_count() {
        let mut out = String::new();
        let p = vec![pred("a.rs", 0.5), pred("b.rs", 0.5), pred("c.rs", 0.5)];
        write_summary_section(&mut out, &p).unwrap();
        assert!(out.contains("## Summary"));
        assert!(out.contains("**Total files analyzed**: 3"));
    }

    #[test]
    fn test_write_summary_section_zero() {
        let mut out = String::new();
        write_summary_section(&mut out, &[]).unwrap();
        assert!(out.contains("**Total files analyzed**: 0"));
    }

    // ── write_risk_row ──────────────────────────────────────────────────────

    #[test]
    fn test_write_risk_row_emits_pipe_table_row() {
        let mut out = String::new();
        write_risk_row(&mut out, "High", 3, 10.0).unwrap();
        assert!(out.contains("| High | 3 | 30.0% |"));
    }

    #[test]
    fn test_write_risk_row_zero_count_zero_pct() {
        let mut out = String::new();
        write_risk_row(&mut out, "Low", 0, 5.0).unwrap();
        assert!(out.contains("| Low | 0 | 0.0% |"));
    }

    // ── write_risk_distribution_table ───────────────────────────────────────

    #[test]
    fn test_write_risk_distribution_table_full_table_emitted() {
        let mut out = String::new();
        let p = vec![pred("a.rs", 0.9), pred("b.rs", 0.5), pred("c.rs", 0.1)];
        write_risk_distribution_table(&mut out, &p).unwrap();
        assert!(out.contains("### Risk Distribution"));
        assert!(out.contains("| Risk Level | Count | Percentage |"));
        assert!(out.contains("|------------|-------|------------|"));
        assert!(out.contains("High (>70%)"));
        assert!(out.contains("Medium (40-70%)"));
        assert!(out.contains("Low (<40%)"));
    }

    // ── write_prediction_metrics ────────────────────────────────────────────

    #[test]
    fn test_write_prediction_metrics_emits_pct_lines() {
        let mut out = String::new();
        let s = score(0.85, 0.9, RiskLevel::High);
        write_prediction_metrics(&mut out, &s).unwrap();
        assert!(out.contains("**Probability**: 85.0%"));
        assert!(out.contains("**Confidence**: 90.0%"));
        assert!(out.contains("**Risk Factors**:"));
        assert!(out.contains("complexity"));
    }

    // ── write_recommendations ───────────────────────────────────────────────

    #[test]
    fn test_write_recommendations_high_branch() {
        let mut out = String::new();
        write_recommendations(&mut out, 0.8).unwrap();
        assert!(out.contains("🔴 High priority"));
        assert!(out.contains("comprehensive test coverage"));
        assert!(out.contains("refactoring"));
    }

    #[test]
    fn test_write_recommendations_medium_branch() {
        let mut out = String::new();
        write_recommendations(&mut out, 0.5).unwrap();
        assert!(out.contains("🟡 Schedule for regular review"));
        assert!(out.contains("Improve test coverage"));
    }

    #[test]
    fn test_write_recommendations_low_branch() {
        let mut out = String::new();
        write_recommendations(&mut out, 0.2).unwrap();
        assert!(out.contains("🟢 Monitor during regular maintenance"));
    }

    #[test]
    fn test_write_recommendations_at_07_falls_into_medium() {
        let mut out = String::new();
        write_recommendations(&mut out, 0.7).unwrap();
        // boundary: > 0.7 is high, == 0.7 falls through to else-if (medium)
        assert!(out.contains("🟡"));
    }

    #[test]
    fn test_write_recommendations_at_04_falls_into_low() {
        let mut out = String::new();
        write_recommendations(&mut out, 0.4).unwrap();
        assert!(out.contains("🟢"));
    }

    // ── write_single_prediction ─────────────────────────────────────────────

    #[test]
    fn test_write_single_prediction_includes_recommendations_when_flag_set() {
        let mut out = String::new();
        let s = score(0.9, 0.85, RiskLevel::High);
        write_single_prediction(&mut out, "src/foo.rs", &s, true).unwrap();
        assert!(out.contains("### src/foo.rs"));
        assert!(out.contains("**Probability**:"));
        assert!(out.contains("Recommendations:"));
        assert!(out.contains("🔴"));
    }

    #[test]
    fn test_write_single_prediction_skips_recommendations_when_flag_false() {
        let mut out = String::new();
        let s = score(0.9, 0.85, RiskLevel::High);
        write_single_prediction(&mut out, "src/foo.rs", &s, false).unwrap();
        assert!(out.contains("### src/foo.rs"));
        assert!(out.contains("**Probability**:"));
        assert!(!out.contains("Recommendations:"));
    }

    // ── write_detailed_predictions ──────────────────────────────────────────

    #[test]
    fn test_write_detailed_predictions_caps_at_20() {
        let mut out = String::new();
        let preds: Vec<_> = (0..30).map(|i| pred(&format!("f{i}.rs"), 0.5)).collect();
        write_detailed_predictions(&mut out, &preds, false).unwrap();
        // Verify the 21st file is not present
        assert!(out.contains("### f0.rs"));
        assert!(out.contains("### f19.rs"));
        assert!(!out.contains("### f20.rs"));
        assert!(!out.contains("### f29.rs"));
    }

    #[test]
    fn test_write_detailed_predictions_empty_emits_header_only() {
        let mut out = String::new();
        write_detailed_predictions(&mut out, &[], false).unwrap();
        assert!(out.contains("## Detailed Predictions"));
        assert!(!out.contains("###"));
    }

    // ── format_defect_markdown (top-level) ──────────────────────────────────

    #[test]
    fn test_format_defect_markdown_has_all_sections() {
        let p = vec![pred("a.rs", 0.9), pred("b.rs", 0.3)];
        let out = format_defect_markdown(&p, true).unwrap();
        assert!(out.starts_with("# Defect Prediction Report"));
        assert!(out.contains("## Summary"));
        assert!(out.contains("### Risk Distribution"));
        assert!(out.contains("## Detailed Predictions"));
        assert!(out.contains("Recommendations:"));
    }

    #[test]
    fn test_format_defect_markdown_no_recommendations_when_flag_false() {
        let p = vec![pred("a.rs", 0.9)];
        let out = format_defect_markdown(&p, false).unwrap();
        assert!(!out.contains("Recommendations:"));
    }

    #[test]
    fn test_format_defect_markdown_empty_predictions() {
        let out = format_defect_markdown(&[], false).unwrap();
        assert!(out.contains("# Defect Prediction Report"));
        assert!(out.contains("**Total files analyzed**: 0"));
    }

    // ── format_defect_summary (top-level) ───────────────────────────────────

    #[test]
    fn test_format_defect_summary_basic() {
        let p = vec![pred("a.rs", 0.9), pred("b.rs", 0.5), pred("c.rs", 0.1)];
        let out = format_defect_summary(&p).unwrap();
        assert!(out.contains("# Defect Prediction Summary"));
        assert!(out.contains("**Total files analyzed**: 3"));
        assert!(out.contains("High Risk (>70%): 1 files"));
        assert!(out.contains("Medium Risk (40-70%): 1 files"));
        assert!(out.contains("Low Risk (<40%): 1 files"));
        assert!(out.contains("Top 10 High-Risk Files:"));
    }

    #[test]
    fn test_format_defect_summary_empty_skips_top_files_section() {
        let out = format_defect_summary(&[]).unwrap();
        assert!(out.contains("# Defect Prediction Summary"));
        assert!(out.contains("**Total files analyzed**: 0"));
        assert!(!out.contains("Top 10 High-Risk Files"));
    }

    #[test]
    fn test_format_defect_summary_caps_top_at_10() {
        let preds: Vec<_> = (0..15).map(|i| pred(&format!("f{i}.rs"), 0.9)).collect();
        let out = format_defect_summary(&preds).unwrap();
        // first 10 files appear; #11-15 do not
        assert!(out.contains("1. f0.rs"));
        assert!(out.contains("10. f9.rs"));
        assert!(!out.contains("11. f10.rs"));
    }

    #[test]
    fn test_format_defect_summary_probability_formatted_as_percent() {
        let p = vec![pred("a.rs", 0.851)];
        let out = format_defect_summary(&p).unwrap();
        assert!(out.contains("85.1% probability"));
    }
}
