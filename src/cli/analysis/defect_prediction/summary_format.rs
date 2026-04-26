//! Summary format output for defect predictions

use crate::services::defect_probability::DefectScore;
use anyhow::Result;

/// Toyota Way: Extract Method - Reduced complexity by separating concerns
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn format_defect_summary(
    predictions: &[(String, DefectScore)],
    elapsed: std::time::Duration,
) -> Result<String> {
    let mut output = String::new();

    write_summary_header(&mut output)?;
    write_risk_distribution(&mut output, predictions)?;
    write_top_risk_files(&mut output, predictions)?;
    write_summary_footer(&mut output, elapsed)?;

    Ok(output)
}

/// Toyota Way: Extract Method - Write summary header
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn write_summary_header(output: &mut String) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "🔮 Defect Prediction Summary")?;
    writeln!(output, "==========================")?;
    writeln!(output)?;
    Ok(())
}

/// Toyota Way: Extract Method - Calculate and write risk distribution
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn write_risk_distribution(
    output: &mut String,
    predictions: &[(String, DefectScore)],
) -> Result<()> {
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

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
pub(crate) fn calculate_risk_statistics(predictions: &[(String, DefectScore)]) -> RiskStatistics {
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn write_top_risk_files(
    output: &mut String,
    predictions: &[(String, DefectScore)],
) -> Result<()> {
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn write_summary_footer(
    output: &mut String,
    elapsed: std::time::Duration,
) -> Result<()> {
    use std::fmt::Write;
    writeln!(output)?;
    writeln!(output, "⏱️  Analysis time: {elapsed:.2?}")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::defect_probability::RiskLevel;
    use std::time::Duration;

    fn score(p: f32, risk: RiskLevel) -> DefectScore {
        DefectScore {
            probability: p,
            confidence: 0.9,
            contributing_factors: vec![],
            risk_level: risk,
            recommendations: vec![],
        }
    }

    fn pred(file: &str, p: f32, r: RiskLevel) -> (String, DefectScore) {
        (file.to_string(), score(p, r))
    }

    // ── get_risk_icon ───────────────────────────────────────────────────────

    #[test]
    fn test_get_risk_icon_all_arms() {
        assert_eq!(get_risk_icon(&RiskLevel::High), "🔴");
        assert_eq!(get_risk_icon(&RiskLevel::Medium), "🟡");
        assert_eq!(get_risk_icon(&RiskLevel::Low), "🟢");
    }

    // ── calculate_risk_statistics ───────────────────────────────────────────

    #[test]
    fn test_calculate_risk_statistics_empty() {
        let s = calculate_risk_statistics(&[]);
        assert_eq!(s.high_risk, 0);
        assert_eq!(s.medium_risk, 0);
        assert_eq!(s.low_risk, 0);
    }

    #[test]
    fn test_calculate_risk_statistics_high_only() {
        let p = vec![pred("a.rs", 0.9, RiskLevel::High)];
        let s = calculate_risk_statistics(&p);
        assert_eq!(s.high_risk, 1);
        assert_eq!(s.medium_risk, 0);
        assert_eq!(s.low_risk, 0);
    }

    #[test]
    fn test_calculate_risk_statistics_medium_only() {
        let p = vec![pred("a.rs", 0.5, RiskLevel::Medium)];
        let s = calculate_risk_statistics(&p);
        assert_eq!(s.medium_risk, 1);
    }

    #[test]
    fn test_calculate_risk_statistics_low_only() {
        let p = vec![pred("a.rs", 0.1, RiskLevel::Low)];
        let s = calculate_risk_statistics(&p);
        assert_eq!(s.low_risk, 1);
    }

    #[test]
    fn test_calculate_risk_statistics_threshold_07_is_medium() {
        // > 0.7 = high; == 0.7 falls through to medium (filter is > 0.3 && <= 0.7)
        let p = vec![pred("a.rs", 0.7, RiskLevel::Medium)];
        let s = calculate_risk_statistics(&p);
        assert_eq!(s.high_risk, 0);
        assert_eq!(s.medium_risk, 1);
    }

    #[test]
    fn test_calculate_risk_statistics_threshold_03_is_low() {
        // > 0.3 = medium; == 0.3 falls through to low (filter is <= 0.3)
        let p = vec![pred("a.rs", 0.3, RiskLevel::Low)];
        let s = calculate_risk_statistics(&p);
        assert_eq!(s.medium_risk, 0);
        assert_eq!(s.low_risk, 1);
    }

    #[test]
    fn test_calculate_risk_statistics_mixed() {
        let p = vec![
            pred("h1.rs", 0.95, RiskLevel::High),
            pred("h2.rs", 0.72, RiskLevel::High),
            pred("m1.rs", 0.5, RiskLevel::Medium),
            pred("m2.rs", 0.4, RiskLevel::Medium),
            pred("m3.rs", 0.31, RiskLevel::Medium),
            pred("l1.rs", 0.2, RiskLevel::Low),
            pred("l2.rs", 0.0, RiskLevel::Low),
        ];
        let s = calculate_risk_statistics(&p);
        assert_eq!(s.high_risk, 2);
        assert_eq!(s.medium_risk, 3);
        assert_eq!(s.low_risk, 2);
    }

    // ── write_summary_header ────────────────────────────────────────────────

    #[test]
    fn test_write_summary_header_emits_title() {
        let mut out = String::new();
        write_summary_header(&mut out).unwrap();
        assert!(out.contains("🔮 Defect Prediction Summary"));
        assert!(out.contains("==="));
    }

    // ── write_risk_distribution ─────────────────────────────────────────────

    #[test]
    fn test_write_risk_distribution_includes_all_three_levels() {
        let mut out = String::new();
        let p = vec![
            pred("h.rs", 0.9, RiskLevel::High),
            pred("m.rs", 0.5, RiskLevel::Medium),
            pred("l.rs", 0.1, RiskLevel::Low),
        ];
        write_risk_distribution(&mut out, &p).unwrap();
        assert!(out.contains("📊 Risk Distribution"));
        assert!(out.contains("🔴 High risk:   1"));
        assert!(out.contains("🟡 Medium risk: 1"));
        assert!(out.contains("🟢 Low risk:    1"));
    }

    // ── write_top_risk_files ────────────────────────────────────────────────

    #[test]
    fn test_write_top_risk_files_lists_predictions() {
        let mut out = String::new();
        let p = vec![pred("foo.rs", 0.85, RiskLevel::High)];
        write_top_risk_files(&mut out, &p).unwrap();
        assert!(out.contains("🎯 Top Risk Files"));
        assert!(out.contains("foo.rs"));
        assert!(out.contains("85.0%"));
        assert!(out.contains("🔴"));
        assert!(out.contains("90.0%")); // confidence
    }

    #[test]
    fn test_write_top_risk_files_caps_at_10() {
        let mut out = String::new();
        let preds: Vec<_> = (0..15)
            .map(|i| pred(&format!("f{i}.rs"), 0.8, RiskLevel::High))
            .collect();
        write_top_risk_files(&mut out, &preds).unwrap();
        assert!(out.contains("f0.rs"));
        assert!(out.contains("f9.rs"));
        assert!(!out.contains("f10.rs"));
    }

    #[test]
    fn test_write_top_risk_files_empty_emits_nothing() {
        let mut out = String::new();
        write_top_risk_files(&mut out, &[]).unwrap();
        assert!(!out.contains("🎯 Top Risk Files"));
    }

    // ── write_summary_footer ────────────────────────────────────────────────

    #[test]
    fn test_write_summary_footer_includes_timing() {
        let mut out = String::new();
        write_summary_footer(&mut out, Duration::from_millis(100)).unwrap();
        assert!(out.contains("Analysis time"));
    }

    // ── format_defect_summary (orchestrator) ────────────────────────────────

    #[test]
    fn test_format_defect_summary_full_pipeline() {
        let p = vec![
            pred("a.rs", 0.9, RiskLevel::High),
            pred("b.rs", 0.5, RiskLevel::Medium),
            pred("c.rs", 0.1, RiskLevel::Low),
        ];
        let out = format_defect_summary(&p, Duration::from_millis(50)).unwrap();
        assert!(out.starts_with("🔮 Defect Prediction Summary"));
        assert!(out.contains("Risk Distribution"));
        assert!(out.contains("Top Risk Files"));
        assert!(out.contains("Analysis time"));
    }

    #[test]
    fn test_format_defect_summary_empty() {
        let out = format_defect_summary(&[], Duration::from_millis(0)).unwrap();
        assert!(out.contains("🔮 Defect Prediction Summary"));
        assert!(out.contains("0 files"));
        // No top-risk-files section when empty
        assert!(!out.contains("🎯"));
    }
}
