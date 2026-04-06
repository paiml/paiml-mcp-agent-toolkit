//! Detailed and JSON format output for defect predictions

use crate::services::defect_probability::DefectScore;
use anyhow::Result;

/// Format predictions as JSON
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn format_defect_json(
    predictions: &[(String, DefectScore)],
    elapsed: std::time::Duration,
) -> Result<String> {
    let report = serde_json::json!({
        "analysis_type": "defect_prediction",
        "summary": {
            "total_files_analyzed": predictions.len(),
            "high_risk_files": predictions.iter().filter(|(_, s)| s.probability > 0.7).count(),
            "medium_risk_files": predictions.iter().filter(|(_, s)| s.probability > 0.3 && s.probability <= 0.7).count(),
            "low_risk_files": predictions.iter().filter(|(_, s)| s.probability <= 0.3).count(),
            "analysis_time_ms": elapsed.as_millis(),
        },
        "predictions": predictions.iter().map(|(file, score)| {
            serde_json::json!({
                "file": file,
                "probability": score.probability,
                "confidence": score.confidence,
                "risk_level": format!("{:?}", score.risk_level),
                "contributing_factors": score.contributing_factors,
                "recommendations": score.recommendations,
            })
        }).collect::<Vec<_>>(),
    });

    Ok(serde_json::to_string_pretty(&report)?)
}

/// Format predictions as detailed report
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn format_defect_detailed(
    predictions: &[(String, DefectScore)],
    elapsed: std::time::Duration,
    include_recommendations: bool,
) -> Result<String> {
    let mut output = String::new();

    write_detailed_header(&mut output)?;

    for (file, score) in predictions {
        write_file_details(&mut output, file, score, include_recommendations)?;
    }

    write_analysis_footer(&mut output, elapsed)?;
    Ok(output)
}

/// Write detailed report header
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn write_detailed_header(output: &mut String) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "🔮 Defect Prediction Detailed Report")?;
    writeln!(output, "===================================")?;
    writeln!(output)?;
    Ok(())
}

/// Write details for a single file
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn write_file_details(
    output: &mut String,
    file: &str,
    score: &DefectScore,
    include_recommendations: bool,
) -> Result<()> {
    use std::fmt::Write;

    writeln!(output, "📄 File: {file}")?;
    write_risk_level(output, score)?;
    write_confidence_level(output, score)?;
    write_contributing_factors(output, score)?;

    if include_recommendations {
        write_recommendations(output, score)?;
    }

    writeln!(output)?;
    Ok(())
}

/// Write risk level information
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn write_risk_level(output: &mut String, score: &DefectScore) -> Result<()> {
    use std::fmt::Write;
    let risk_display = format_risk_level_display(&score.risk_level);
    writeln!(
        output,
        "   Risk Level: {} ({:.1}%)",
        risk_display,
        score.probability * 100.0
    )?;
    Ok(())
}

/// Format risk level for display
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn format_risk_level_display(
    risk_level: &crate::services::defect_probability::RiskLevel,
) -> &'static str {
    match risk_level {
        crate::services::defect_probability::RiskLevel::High => "🔴 HIGH",
        crate::services::defect_probability::RiskLevel::Medium => "🟡 MEDIUM",
        crate::services::defect_probability::RiskLevel::Low => "🟢 LOW",
    }
}

/// Write confidence level information
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn write_confidence_level(output: &mut String, score: &DefectScore) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "   Confidence: {:.1}%", score.confidence * 100.0)?;
    Ok(())
}

/// Write contributing factors section
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn write_contributing_factors(output: &mut String, score: &DefectScore) -> Result<()> {
    use std::fmt::Write;

    if score.contributing_factors.is_empty() {
        return Ok(());
    }

    writeln!(output, "   Contributing Factors:")?;
    for (factor, weight) in &score.contributing_factors {
        writeln!(output, "     - {}: {:.1}%", factor, weight * 100.0)?;
    }
    Ok(())
}

/// Write recommendations section
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn write_recommendations(output: &mut String, score: &DefectScore) -> Result<()> {
    use std::fmt::Write;

    if score.recommendations.is_empty() {
        return Ok(());
    }

    writeln!(output, "   Recommendations:")?;
    for rec in &score.recommendations {
        writeln!(output, "     • {rec}")?;
    }
    Ok(())
}

/// Write analysis footer with timing
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn write_analysis_footer(
    output: &mut String,
    elapsed: std::time::Duration,
) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "⏱️  Analysis time: {elapsed:.2?}")?;
    Ok(())
}
