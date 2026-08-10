#![cfg_attr(coverage_nightly, coverage(off))]
//! CLI handler for `pmat infra-score` command
//!
//! Calculates Infrastructure Score (0-100 + 12 bonus) for CI/CD,
//! build reliability, quality pipeline, deployment, supply chain,
//! and provable contracts.

use crate::cli::RepoScoreOutputFormat;
use crate::services::infra_score::aggregator::InfraScoreAggregator;
use crate::services::infra_score::models::{InfraCheck, InfraScore, InfraSeverity};
use anyhow::Result;
use std::path::Path;

/// Handle the infra-score command
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_infra_score(
    path: &Path,
    format: &RepoScoreOutputFormat,
    verbose: bool,
    failures_only: bool,
    output: Option<&Path>,
) -> Result<()> {
    if !path.exists() {
        anyhow::bail!("Path not found: {}", path.display());
    }

    let aggregator = InfraScoreAggregator::new();
    let result = aggregator.aggregate(path).await?;

    let output_str = match format {
        RepoScoreOutputFormat::Json => serde_json::to_string_pretty(&result)?,
        _ => format_text_output(&result, verbose, failures_only),
    };

    if let Some(out_path) = output {
        std::fs::write(out_path, &output_str)?;
        eprintln!("Output written to {}", out_path.display());
    } else {
        println!("{output_str}");
    }

    if result.auto_fail {
        std::process::exit(1);
    }

    Ok(())
}

/// Score color: green ≥90, yellow ≥80, red otherwise.
fn infra_score_color(score: f64) -> &'static str {
    if score >= 90.0 {
        "\x1b[32m"
    } else if score >= 80.0 {
        "\x1b[33m"
    } else {
        "\x1b[31m"
    }
}

/// Category percentage color: green ≥90, yellow ≥70, red otherwise.
fn infra_pct_color(pct: f64) -> &'static str {
    if pct >= 90.0 {
        "\x1b[32m"
    } else if pct >= 70.0 {
        "\x1b[33m"
    } else {
        "\x1b[31m"
    }
}

/// Category status icon: ✓ ≥90, ⚠ ≥70, ✗ otherwise.
fn infra_pct_icon(pct: f64) -> &'static str {
    if pct >= 90.0 {
        "\x1b[32m✓\x1b[0m"
    } else if pct >= 70.0 {
        "\x1b[33m⚠\x1b[0m"
    } else {
        "\x1b[31m✗\x1b[0m"
    }
}

/// Write a single check line, optionally followed by its evidence.
fn write_infra_check(out: &mut String, check: &InfraCheck, verbose: bool, show_evidence: bool) {
    use std::fmt::Write;
    let check_icon = if check.passed { "  ✓" } else { "  ✗" };
    let _ = writeln!(
        out,
        "    {} {} ({}): {:.0}/{:.0}",
        check_icon, check.id, check.name, check.score, check.max_score
    );
    if show_evidence && (!check.passed || verbose) {
        for ev in &check.evidence {
            let _ = writeln!(out, "      {}", ev);
        }
    }
}

fn write_infra_summary(out: &mut String, result: &InfraScore) {
    use std::fmt::Write;
    let _ = writeln!(out, "\n\x1b[1mSummary\x1b[0m");
    let score_color = infra_score_color(result.total_score);
    let _ = writeln!(
        out,
        "  Score: {}{:.1}\x1b[0m/\x1b[2m100.0\x1b[0m",
        score_color, result.total_score
    );
    let _ = writeln!(
        out,
        "  Grade: {}{}\x1b[0m",
        score_color,
        result.grade.as_str()
    );
    if result.auto_fail {
        let _ = writeln!(out, "  Status: \x1b[31mAUTO-FAIL\x1b[0m (< 90 required)");
    } else {
        let _ = writeln!(out, "  Status: \x1b[32mPASS\x1b[0m");
    }

    let bonus = result.categories.provable_contracts.score;
    if bonus > 0.0 {
        let _ = writeln!(
            out,
            "  Bonus: \x1b[36m+{:.1}\x1b[0m (provable contracts)",
            bonus
        );
        // Denominator is derived, not hardcoded: the bonus category is worth 12
        // (PV-01..PV-05), so a hardcoded "/110.0" could print a total above its
        // own maximum.
        let _ = writeln!(
            out,
            "  Total with bonus: {}{:.1}\x1b[0m/{:.1}",
            score_color,
            result.categories.total_with_bonus(),
            crate::services::infra_score::models::INFRA_SCORE_MAX_POINTS
                + result.categories.provable_contracts.max_score
        );
    }
}

fn write_infra_categories(
    out: &mut String,
    result: &InfraScore,
    verbose: bool,
    failures_only: bool,
) {
    use std::fmt::Write;
    let _ = writeln!(out, "\n\x1b[1mCategories\x1b[0m");
    let categories = [
        (
            "Workflow Architecture",
            &result.categories.workflow_architecture,
        ),
        ("Build Reliability", &result.categories.build_reliability),
        ("Quality Pipeline", &result.categories.quality_pipeline),
        (
            "Deployment & Release",
            &result.categories.deployment_release,
        ),
        ("Supply Chain Security", &result.categories.supply_chain),
    ];

    for (name, cat) in &categories {
        let pct_color = infra_pct_color(cat.percentage);
        let _ = writeln!(
            out,
            "  {} {}: {}{:.1}\x1b[0m/\x1b[2m{:.1}\x1b[0m ({}{:.1}%\x1b[0m)",
            infra_pct_icon(cat.percentage),
            name,
            pct_color,
            cat.score,
            cat.max_score,
            pct_color,
            cat.percentage
        );

        if verbose && !cat.checks.is_empty() {
            for check in &cat.checks {
                if failures_only && check.passed {
                    continue;
                }
                write_infra_check(out, check, verbose, true);
            }
        }
    }
}

fn write_infra_provable_contracts(
    out: &mut String,
    result: &InfraScore,
    verbose: bool,
    failures_only: bool,
) {
    use std::fmt::Write;
    let pv = &result.categories.provable_contracts;
    if !(pv.score > 0.0 || verbose) {
        return;
    }
    let icon = if pv.percentage >= 80.0 {
        "\x1b[36m★\x1b[0m"
    } else if pv.score > 0.0 {
        "\x1b[36m◆\x1b[0m"
    } else {
        "\x1b[2m-\x1b[0m"
    };
    let _ = writeln!(
        out,
        "  {} Provable Contracts (bonus): \x1b[36m{:.1}\x1b[0m/\x1b[2m{:.1}\x1b[0m ({:.1}%)",
        icon, pv.score, pv.max_score, pv.percentage
    );

    if verbose {
        for check in &pv.checks {
            if failures_only && check.passed {
                continue;
            }
            write_infra_check(out, check, verbose, false);
        }
    }
}

fn write_infra_findings(out: &mut String, result: &InfraScore, verbose: bool, failures_only: bool) {
    use std::fmt::Write;
    let all_findings: Vec<_> = [
        &result.categories.workflow_architecture.findings,
        &result.categories.build_reliability.findings,
        &result.categories.quality_pipeline.findings,
        &result.categories.deployment_release.findings,
        &result.categories.supply_chain.findings,
        &result.categories.provable_contracts.findings,
    ]
    .iter()
    .flat_map(|f| f.iter())
    .collect();

    if all_findings.is_empty() || (!verbose && failures_only) {
        return;
    }
    let _ = writeln!(out, "\n\x1b[1mFindings\x1b[0m");
    for finding in &all_findings {
        let icon = match finding.severity {
            InfraSeverity::Fail => "\x1b[31m✗\x1b[0m",
            InfraSeverity::Warning => "\x1b[33m⚠\x1b[0m",
            InfraSeverity::Info => "\x1b[36mℹ\x1b[0m",
            InfraSeverity::Pass => "\x1b[32m✓\x1b[0m",
        };
        let loc = finding
            .location
            .as_deref()
            .map(|l| format!(" ({})", l))
            .unwrap_or_default();
        let _ = writeln!(
            out,
            "  {} [{}]{}: {}",
            icon, finding.check_id, loc, finding.message
        );
    }
}

fn write_infra_recommendations(out: &mut String, result: &InfraScore) {
    use std::fmt::Write;
    if result.recommendations.is_empty() {
        return;
    }
    let _ = writeln!(out, "\n\x1b[1mRecommendations\x1b[0m");
    for rec in &result.recommendations {
        let _ = writeln!(
            out,
            "  \x1b[2;37m{}: {} (+{:.0} pts, ~{})\x1b[0m",
            rec.check_id, rec.description, rec.impact_points, rec.estimated_effort
        );
    }
}

fn format_text_output(result: &InfraScore, verbose: bool, failures_only: bool) -> String {
    use std::fmt::Write;
    let mut out = String::new();

    let _ = writeln!(out, "\x1b[2m{}\x1b[0m", "━".repeat(48));
    let _ = writeln!(out, "\x1b[1m\x1b[4mInfra Score v1.0\x1b[0m");
    let _ = writeln!(out, "\x1b[2m{}\x1b[0m", "━".repeat(48));

    write_infra_summary(&mut out, result);
    write_infra_categories(&mut out, result, verbose, failures_only);
    write_infra_provable_contracts(&mut out, result, verbose, failures_only);
    write_infra_findings(&mut out, result, verbose, failures_only);
    write_infra_recommendations(&mut out, result);

    // Metadata
    let _ = writeln!(out, "\n\x1b[2m{}\x1b[0m", "━".repeat(48));
    let _ = writeln!(
        out,
        "\x1b[2mExecuted in {}ms | pmat v{}\x1b[0m",
        result.metadata.execution_time_ms, result.metadata.pmat_version
    );

    out
}

#[cfg(test)]
mod format_text_tests {
    //! Covers format_text_output in infra_score_handlers.rs (246 lines,
    //! 0 prior tests). Skips async handle_infra_score (filesystem +
    //! process exit + InfraScoreAggregator subprocess work).
    use super::*;
    use crate::services::infra_score::models::{
        InfraCategoryScores, InfraCheck, InfraFinding, InfraGrade, InfraRecommendation, InfraScore,
        InfraScoreMetadata, InfraSeverity,
    };
    use std::path::PathBuf;

    fn empty_metadata() -> InfraScoreMetadata {
        InfraScoreMetadata::new(PathBuf::from("/tmp"))
    }

    fn make_score(total: f64, grade: InfraGrade, auto_fail: bool) -> InfraScore {
        InfraScore {
            total_score: total,
            grade,
            auto_fail,
            categories: InfraCategoryScores::default(),
            recommendations: vec![],
            metadata: empty_metadata(),
        }
    }

    fn check(id: &str, name: &str, passed: bool, evidence: Vec<String>) -> InfraCheck {
        InfraCheck {
            id: id.to_string(),
            name: name.to_string(),
            score: if passed { 5.0 } else { 0.0 },
            max_score: 5.0,
            passed,
            evidence,
        }
    }

    fn finding(severity: InfraSeverity, check_id: &str, msg: &str) -> InfraFinding {
        InfraFinding {
            severity,
            check_id: check_id.to_string(),
            message: msg.to_string(),
            location: Some("file:line".to_string()),
            impact_points: 1.0,
        }
    }

    // ── Score-color tier arms ──

    #[test]
    fn test_format_text_output_high_score_uses_green() {
        let r = make_score(95.0, InfraGrade::APlus, false);
        let out = format_text_output(&r, false, false);
        // Score >= 90 → green ANSI \x1b[32m.
        assert!(out.contains("\x1b[32m"));
        assert!(out.contains("PASS"));
    }

    #[test]
    fn test_format_text_output_mid_score_uses_yellow() {
        let r = make_score(85.0, InfraGrade::B, true);
        let out = format_text_output(&r, false, false);
        // Score 80-89 → yellow.
        assert!(out.contains("\x1b[33m"));
        assert!(out.contains("AUTO-FAIL"));
    }

    #[test]
    fn test_format_text_output_low_score_uses_red() {
        let r = make_score(50.0, InfraGrade::D, true);
        let out = format_text_output(&r, false, false);
        // Score < 80 → red.
        assert!(out.contains("\x1b[31m"));
        assert!(out.contains("AUTO-FAIL"));
    }

    // ── Bonus block ──

    #[test]
    fn test_format_text_output_with_provable_bonus_emits_total_with_bonus() {
        let mut r = make_score(95.0, InfraGrade::APlus, false);
        r.categories.provable_contracts.score = 5.0;
        r.categories.provable_contracts.percentage = 50.0;
        let out = format_text_output(&r, false, false);
        assert!(out.contains("Bonus:"));
        assert!(out.contains("Total with bonus:"));
    }

    #[test]
    fn test_format_text_output_no_bonus_skips_bonus_block() {
        let r = make_score(95.0, InfraGrade::APlus, false);
        // bonus = 0 by default.
        let out = format_text_output(&r, false, false);
        assert!(!out.contains("Total with bonus:"));
    }

    // ── Categories table icon arms ──

    #[test]
    fn test_format_text_output_category_icons_for_each_threshold() {
        let mut r = make_score(80.0, InfraGrade::B, true);
        // High (>= 90%) → ✓
        r.categories.workflow_architecture.percentage = 95.0;
        // Mid (70-90) → ⚠
        r.categories.build_reliability.percentage = 80.0;
        // Low (< 70) → ✗
        r.categories.quality_pipeline.percentage = 50.0;
        let out = format_text_output(&r, false, false);
        assert!(out.contains("✓"));
        assert!(out.contains("⚠"));
        assert!(out.contains("✗"));
    }

    // ── Verbose mode ──

    #[test]
    fn test_format_text_output_verbose_includes_check_evidence() {
        let mut r = make_score(95.0, InfraGrade::APlus, false);
        r.categories.workflow_architecture.checks = vec![check(
            "WA-01",
            "Workflow",
            true,
            vec!["evidence line 1".to_string()],
        )];
        let out = format_text_output(&r, true, false);
        assert!(out.contains("WA-01"));
        assert!(out.contains("Workflow"));
    }

    #[test]
    fn test_format_text_output_failures_only_skips_passed_checks() {
        let mut r = make_score(95.0, InfraGrade::APlus, false);
        r.categories.workflow_architecture.checks = vec![
            check("PASS-01", "Pass", true, vec![]),
            check("FAIL-01", "Fail", false, vec![]),
        ];
        let out = format_text_output(&r, true, true);
        // failures_only=true with verbose=true → only failed check shown.
        assert!(out.contains("FAIL-01"));
        assert!(!out.contains("PASS-01"));
    }

    // ── Findings ──

    #[test]
    fn test_format_text_output_emits_findings_with_severity_icons() {
        let mut r = make_score(80.0, InfraGrade::B, true);
        r.categories.workflow_architecture.findings = vec![
            finding(InfraSeverity::Fail, "F-01", "Failed"),
            finding(InfraSeverity::Warning, "W-01", "Warning"),
            finding(InfraSeverity::Info, "I-01", "Info"),
            finding(InfraSeverity::Pass, "P-01", "Pass"),
        ];
        let out = format_text_output(&r, false, false);
        assert!(out.contains("Findings"));
        // All 4 finding severities reachable.
        assert!(out.contains("F-01"));
        assert!(out.contains("W-01"));
        assert!(out.contains("I-01"));
        assert!(out.contains("P-01"));
    }

    #[test]
    fn test_format_text_output_no_findings_skips_findings_section() {
        let r = make_score(95.0, InfraGrade::APlus, false);
        // No findings populated.
        let out = format_text_output(&r, false, false);
        // Section header not emitted when findings empty.
        assert!(!out.contains("\x1b[1mFindings\x1b[0m"));
    }

    // ── Recommendations ──

    #[test]
    fn test_format_text_output_with_recommendations_emits_section() {
        let mut r = make_score(80.0, InfraGrade::B, true);
        r.recommendations.push(InfraRecommendation {
            priority: crate::services::infra_score::models::InfraPriority::High,
            check_id: "WA-01".to_string(),
            title: "Add OIDC".to_string(),
            description: "Switch to OIDC for AWS".to_string(),
            impact_points: 5.0,
            estimated_effort: "1h".to_string(),
        });
        let out = format_text_output(&r, false, false);
        assert!(out.contains("Recommendations"));
        assert!(out.contains("WA-01"));
        assert!(out.contains("Switch to OIDC"));
    }

    #[test]
    fn test_format_text_output_no_recommendations_skips_section() {
        let r = make_score(95.0, InfraGrade::APlus, false);
        let out = format_text_output(&r, false, false);
        assert!(!out.contains("\x1b[1mRecommendations\x1b[0m"));
    }

    // ── Bonus denominator ──

    #[test]
    fn test_bonus_denominator_matches_bonus_category_max() {
        use crate::services::infra_score::models::{
            INFRA_SCORE_BONUS_MAX_POINTS, INFRA_SCORE_MAX_POINTS,
        };
        let mut r = make_score(99.0, InfraGrade::APlus, false);
        // A perfect bonus run: the denominator printed must not be smaller than
        // the bonus the same output claims to have awarded.
        r.categories.provable_contracts.score = INFRA_SCORE_BONUS_MAX_POINTS;
        let out = format_text_output(&r, false, false);
        let expected = format!(
            "/{:.1}",
            INFRA_SCORE_MAX_POINTS + INFRA_SCORE_BONUS_MAX_POINTS
        );
        assert!(
            out.contains(&expected),
            "expected bonus denominator {expected} in:\n{out}"
        );
        assert!(!out.contains("/110.0"), "hardcoded /110.0 denominator");
    }

    #[test]
    fn test_bonus_category_default_max_matches_scorer_max() {
        use crate::services::infra_score::scorers::InfraScorer;
        let scorer =
            crate::services::infra_score::scorers::provable_contracts::ProvableContractsScorer::new(
            );
        let defaults = InfraCategoryScores::default();
        assert_eq!(
            defaults.provable_contracts.max_score,
            scorer.max_score(),
            "model default and scorer disagree on the bonus maximum"
        );
    }

    // ── Header always present ──

    #[test]
    fn test_format_text_output_always_emits_header_and_footer() {
        let r = make_score(95.0, InfraGrade::APlus, false);
        let out = format_text_output(&r, false, false);
        assert!(out.contains("Infra Score v1.0"));
        assert!(out.contains("Executed in"));
        assert!(out.contains("pmat v"));
    }
}
