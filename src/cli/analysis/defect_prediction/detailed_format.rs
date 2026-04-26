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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::defect_probability::RiskLevel;
    use std::time::Duration;

    fn score(probability: f32, confidence: f32, risk: RiskLevel) -> DefectScore {
        DefectScore {
            probability,
            confidence,
            contributing_factors: vec![("complexity".to_string(), 0.4)],
            risk_level: risk,
            recommendations: vec!["refactor".to_string()],
        }
    }

    fn pred(file: &str, p: f32, risk: RiskLevel) -> (String, DefectScore) {
        (file.to_string(), score(p, 0.85, risk))
    }

    // ── format_risk_level_display ───────────────────────────────────────────

    #[test]
    fn test_format_risk_level_display_high() {
        assert_eq!(format_risk_level_display(&RiskLevel::High), "🔴 HIGH");
    }

    #[test]
    fn test_format_risk_level_display_medium() {
        assert_eq!(format_risk_level_display(&RiskLevel::Medium), "🟡 MEDIUM");
    }

    #[test]
    fn test_format_risk_level_display_low() {
        assert_eq!(format_risk_level_display(&RiskLevel::Low), "🟢 LOW");
    }

    // ── write_detailed_header ───────────────────────────────────────────────

    #[test]
    fn test_write_detailed_header_emits_title() {
        let mut out = String::new();
        write_detailed_header(&mut out).unwrap();
        assert!(out.contains("🔮 Defect Prediction Detailed Report"));
        assert!(out.contains("==="));
    }

    // ── write_risk_level ────────────────────────────────────────────────────

    #[test]
    fn test_write_risk_level_includes_pct_and_emoji() {
        let mut out = String::new();
        let s = score(0.85, 0.9, RiskLevel::High);
        write_risk_level(&mut out, &s).unwrap();
        assert!(out.contains("Risk Level"));
        assert!(out.contains("HIGH"));
        assert!(out.contains("85.0%"));
    }

    // ── write_confidence_level ──────────────────────────────────────────────

    #[test]
    fn test_write_confidence_level_includes_pct() {
        let mut out = String::new();
        let s = score(0.5, 0.92, RiskLevel::Medium);
        write_confidence_level(&mut out, &s).unwrap();
        assert!(out.contains("Confidence"));
        assert!(out.contains("92.0%"));
    }

    // ── write_contributing_factors ──────────────────────────────────────────

    #[test]
    fn test_write_contributing_factors_with_factors() {
        let mut out = String::new();
        let s = score(0.5, 0.9, RiskLevel::Medium);
        write_contributing_factors(&mut out, &s).unwrap();
        assert!(out.contains("Contributing Factors"));
        assert!(out.contains("complexity"));
        assert!(out.contains("40.0%"));
    }

    #[test]
    fn test_write_contributing_factors_empty_skipped() {
        let mut out = String::new();
        let mut s = score(0.5, 0.9, RiskLevel::Medium);
        s.contributing_factors.clear();
        write_contributing_factors(&mut out, &s).unwrap();
        // Empty contributing_factors → section omitted
        assert!(!out.contains("Contributing Factors"));
    }

    // ── write_recommendations ───────────────────────────────────────────────

    #[test]
    fn test_write_recommendations_with_items() {
        let mut out = String::new();
        let s = score(0.5, 0.9, RiskLevel::Medium);
        write_recommendations(&mut out, &s).unwrap();
        assert!(out.contains("Recommendations"));
        assert!(out.contains("refactor"));
    }

    #[test]
    fn test_write_recommendations_empty_skipped() {
        let mut out = String::new();
        let mut s = score(0.5, 0.9, RiskLevel::Medium);
        s.recommendations.clear();
        write_recommendations(&mut out, &s).unwrap();
        // Empty recommendations → section omitted
        assert!(!out.contains("Recommendations"));
    }

    // ── write_analysis_footer ───────────────────────────────────────────────

    #[test]
    fn test_write_analysis_footer_includes_timing() {
        let mut out = String::new();
        write_analysis_footer(&mut out, Duration::from_millis(1234)).unwrap();
        assert!(out.contains("Analysis time"));
    }

    // ── write_file_details ──────────────────────────────────────────────────

    #[test]
    fn test_write_file_details_with_recommendations() {
        let mut out = String::new();
        let s = score(0.85, 0.9, RiskLevel::High);
        write_file_details(&mut out, "src/foo.rs", &s, true).unwrap();
        assert!(out.contains("📄 File: src/foo.rs"));
        assert!(out.contains("Risk Level"));
        assert!(out.contains("Confidence"));
        assert!(out.contains("Recommendations"));
    }

    #[test]
    fn test_write_file_details_no_recommendations_flag() {
        let mut out = String::new();
        let s = score(0.85, 0.9, RiskLevel::High);
        write_file_details(&mut out, "src/foo.rs", &s, false).unwrap();
        assert!(out.contains("📄 File: src/foo.rs"));
        assert!(!out.contains("Recommendations"));
    }

    // ── format_defect_detailed (orchestrator) ───────────────────────────────

    #[test]
    fn test_format_defect_detailed_full_pipeline() {
        let preds = vec![
            pred("a.rs", 0.9, RiskLevel::High),
            pred("b.rs", 0.4, RiskLevel::Medium),
        ];
        let out = format_defect_detailed(&preds, Duration::from_millis(50), true).unwrap();
        assert!(out.contains("🔮 Defect Prediction Detailed Report"));
        assert!(out.contains("a.rs"));
        assert!(out.contains("b.rs"));
        assert!(out.contains("Recommendations"));
        assert!(out.contains("Analysis time"));
    }

    #[test]
    fn test_format_defect_detailed_empty_predictions() {
        let out = format_defect_detailed(&[], Duration::from_millis(10), false).unwrap();
        assert!(out.contains("🔮 Defect Prediction Detailed Report"));
        assert!(out.contains("Analysis time"));
    }

    // ── format_defect_json ──────────────────────────────────────────────────

    #[test]
    fn test_format_defect_json_basic() {
        let preds = vec![
            pred("a.rs", 0.9, RiskLevel::High),
            pred("b.rs", 0.5, RiskLevel::Medium),
            pred("c.rs", 0.1, RiskLevel::Low),
        ];
        let json = format_defect_json(&preds, Duration::from_millis(42)).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["analysis_type"], "defect_prediction");
        let summary = &parsed["summary"];
        assert_eq!(summary["total_files_analyzed"], 3);
        assert_eq!(summary["high_risk_files"], 1);
        assert_eq!(summary["medium_risk_files"], 1);
        assert_eq!(summary["low_risk_files"], 1);
        assert_eq!(summary["analysis_time_ms"], 42);
        assert_eq!(parsed["predictions"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_format_defect_json_threshold_boundaries() {
        // Probability 0.7 → medium (boundary check: > 0.7 is high, == 0.7 is medium)
        let preds = vec![pred("a.rs", 0.7, RiskLevel::Medium)];
        let json = format_defect_json(&preds, Duration::from_millis(10)).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["summary"]["high_risk_files"], 0);
        assert_eq!(parsed["summary"]["medium_risk_files"], 1);
    }

    #[test]
    fn test_format_defect_json_empty() {
        let json = format_defect_json(&[], Duration::from_millis(0)).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["summary"]["total_files_analyzed"], 0);
        assert_eq!(parsed["predictions"].as_array().unwrap().len(), 0);
    }
}
