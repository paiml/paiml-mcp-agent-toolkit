//! Summary format output for defect predictions

use crate::services::defect_probability::DefectScore;
use anyhow::Result;

/// Toyota Way: Extract Method - Reduced complexity by separating concerns
pub(crate) fn format_defect_summary(
    predictions: &[(String, DefectScore)],
    elapsed: std::time::Duration,
) -> Result<String> {
    debug_assert!(!predictions.is_empty(), "predictions must not be empty");
    let mut output = String::new();

    write_summary_header(&mut output)?;
    write_risk_distribution(&mut output, predictions)?;
    write_top_risk_files(&mut output, predictions)?;
    write_summary_footer(&mut output, elapsed)?;

    Ok(output)
}

/// Toyota Way: Extract Method - Write summary header
pub(crate) fn write_summary_header(output: &mut String) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "🔮 Defect Prediction Summary")?;
    writeln!(output, "==========================")?;
    writeln!(output)?;
    Ok(())
}

/// Toyota Way: Extract Method - Calculate and write risk distribution
pub(crate) fn write_risk_distribution(
    output: &mut String,
    predictions: &[(String, DefectScore)],
) -> Result<()> {
    debug_assert!(!predictions.is_empty(), "predictions must not be empty");
    use std::fmt::Write;

    let risk_stats = calculate_risk_statistics(predictions);

    writeln!(output, "📊 Risk Distribution:")?;
    writeln!(output, "  🔴 High risk:   {} files", risk_stats.high_risk)?;
    writeln!(output, "  🟡 Medium risk: {} files", risk_stats.medium_risk)?;
    writeln!(output, "  🟢 Low risk:    {} files", risk_stats.low_risk)?;
    writeln!(output)?;

    Ok(())
}

/// Toyota Way: Extract Method - Risk statistics calculation
pub(crate) struct RiskStatistics {
    pub(crate) high_risk: usize,
    pub(crate) medium_risk: usize,
    pub(crate) low_risk: usize,
}

pub(crate) fn calculate_risk_statistics(predictions: &[(String, DefectScore)]) -> RiskStatistics {
    debug_assert!(!predictions.is_empty(), "predictions must not be empty");
    let high_risk = predictions
        .iter()
        .filter(|(_, s)| s.probability > 0.7)
        .count();
    let medium_risk = predictions
        .iter()
        .filter(|(_, s)| s.probability > 0.3 && s.probability <= 0.7)
        .count();
    let low_risk = predictions
        .iter()
        .filter(|(_, s)| s.probability <= 0.3)
        .count();

    RiskStatistics {
        high_risk,
        medium_risk,
        low_risk,
    }
}

/// Toyota Way: Extract Method - Write top risk files section
pub(crate) fn write_top_risk_files(
    output: &mut String,
    predictions: &[(String, DefectScore)],
) -> Result<()> {
    debug_assert!(!predictions.is_empty(), "predictions must not be empty");
    use std::fmt::Write;

    if !predictions.is_empty() {
        writeln!(output, "🎯 Top Risk Files:")?;
        for (file, score) in predictions.iter().take(10) {
            let risk_icon = get_risk_icon(&score.risk_level);
            writeln!(
                output,
                "  {} {:.1}% - {} (confidence: {:.1}%)",
                risk_icon,
                score.probability * 100.0,
                file,
                score.confidence * 100.0
            )?;
        }
    }

    Ok(())
}

/// Toyota Way: Extract Method - Get risk level icon
pub(crate) fn get_risk_icon(
    risk_level: &crate::services::defect_probability::RiskLevel,
) -> &'static str {
    match risk_level {
        crate::services::defect_probability::RiskLevel::High => "🔴",
        crate::services::defect_probability::RiskLevel::Medium => "🟡",
        crate::services::defect_probability::RiskLevel::Low => "🟢",
    }
}

/// Toyota Way: Extract Method - Write summary footer
pub(crate) fn write_summary_footer(
    output: &mut String,
    elapsed: std::time::Duration,
) -> Result<()> {
    use std::fmt::Write;
    writeln!(output)?;
    writeln!(output, "⏱️  Analysis time: {elapsed:.2?}")?;
    Ok(())
}
