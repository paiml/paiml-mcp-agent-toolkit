#![cfg_attr(coverage_nightly, coverage(off))]
//! CLI handler for `pmat infra-score` command
//!
//! Calculates Infrastructure Score (0-100 + 12 bonus) for CI/CD,
//! build reliability, quality pipeline, deployment, supply chain,
//! and provable contracts.

use crate::cli::RepoScoreOutputFormat;
use crate::services::infra_score::aggregator::InfraScoreAggregator;
use crate::services::infra_score::models::{
    InfraCategoryScore, InfraCheck, InfraFinding, InfraScore, InfraSeverity,
};
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

    // Every advertised format is rendered by a formatter of its own.
    //
    // This match used to be `Json => json, _ => text`, so `-f yaml` and
    // `-f markdown` both emitted the ANSI text report — byte-identical to
    // `-f text` — and `yaml.safe_load` failed on the box-drawing rule. The
    // wildcard arm is deliberately gone: adding a variant to
    // `RepoScoreOutputFormat` must now fail to compile here rather than
    // silently resolve to "print the text report and exit 0".
    let output_str = match format {
        RepoScoreOutputFormat::Text => format_text_output(&result, verbose, failures_only),
        RepoScoreOutputFormat::Json => serde_json::to_string_pretty(&result)?,
        RepoScoreOutputFormat::Yaml => serde_yaml_ng::to_string(&result)?,
        RepoScoreOutputFormat::Markdown => format_markdown_output(&result, verbose, failures_only),
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
///
/// This whole renderer used to interpolate raw `"\x1b[32m"` literals, so
/// `infra-score --color never` was BYTE-IDENTICAL to `--color always` (173
/// escape sequences either way) and a redirected `infra-score` report was not
/// diffable. Colour SELECTION is delegated to `colors::threshold_color`, which
/// is the same higher-is-better rule this file re-implemented; colour EMISSION
/// goes through `colors::seq`/`colors::colored`, which honour `--color never`,
/// `NO_COLOR` and a non-TTY stdout.
fn infra_score_color(score: f64) -> crate::cli::colors::Sgr {
    crate::cli::colors::threshold_color(score, 90.0, 80.0)
}

/// Category percentage color: green ≥90, yellow ≥70, red otherwise.
fn infra_pct_color(pct: f64) -> crate::cli::colors::Sgr {
    crate::cli::colors::threshold_color(pct, 90.0, 70.0)
}

/// Category status icon: ✓ ≥90, ⚠ ≥70, ✗ otherwise.
fn infra_pct_icon(pct: f64) -> String {
    use crate::cli::colors as c;
    if pct >= 90.0 {
        c::colored(c::GREEN, "✓")
    } else if pct >= 70.0 {
        c::colored(c::YELLOW, "⚠")
    } else {
        c::colored(c::RED, "✗")
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
    use crate::cli::colors as c;
    use std::fmt::Write;
    let _ = writeln!(out, "\n{}", c::subheader("Summary"));
    let score_color = infra_score_color(result.total_score);
    let _ = writeln!(
        out,
        "  Score: {}/{}",
        c::colored(score_color, &format!("{:.1}", result.total_score)),
        c::dim("100.0")
    );
    let _ = writeln!(
        out,
        "  Grade: {}",
        c::colored(score_color, result.grade.as_str())
    );
    if result.auto_fail {
        let _ = writeln!(
            out,
            "  Status: {} (< 90 required)",
            c::colored(c::RED, "AUTO-FAIL")
        );
    } else {
        let _ = writeln!(out, "  Status: {}", c::colored(c::GREEN, "PASS"));
    }

    let bonus = result.categories.provable_contracts.score;
    if bonus > 0.0 {
        let _ = writeln!(
            out,
            "  Bonus: {} (provable contracts)",
            c::colored(c::CYAN, &format!("+{bonus:.1}"))
        );
        // Denominator is derived, not hardcoded: the bonus category is worth 12
        // (PV-01..PV-05), so a hardcoded "/110.0" could print a total above its
        // own maximum.
        let _ = writeln!(
            out,
            "  Total with bonus: {}/{:.1}",
            c::colored(
                score_color,
                &format!("{:.1}", result.categories.total_with_bonus())
            ),
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
    use crate::cli::colors as c;
    use std::fmt::Write;
    let _ = writeln!(out, "\n{}", c::subheader("Categories"));
    for (name, cat) in &infra_categories(result) {
        let pct_color = infra_pct_color(cat.percentage);
        let _ = writeln!(
            out,
            "  {} {}: {}/{} ({})",
            infra_pct_icon(cat.percentage),
            name,
            c::colored(pct_color, &format!("{:.1}", cat.score)),
            c::dim(&format!("{:.1}", cat.max_score)),
            c::colored(pct_color, &format!("{:.1}%", cat.percentage))
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
    use crate::cli::colors as c;
    use std::fmt::Write;
    let pv = &result.categories.provable_contracts;
    if !(pv.score > 0.0 || verbose) {
        return;
    }
    let icon = if pv.percentage >= 80.0 {
        c::colored(c::CYAN, "★")
    } else if pv.score > 0.0 {
        c::colored(c::CYAN, "◆")
    } else {
        c::dim("-")
    };
    let _ = writeln!(
        out,
        "  {} Provable Contracts (bonus): {}/{} ({:.1}%)",
        icon,
        c::colored(c::CYAN, &format!("{:.1}", pv.score)),
        c::dim(&format!("{:.1}", pv.max_score)),
        pv.percentage
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
    use crate::cli::colors as c;
    use std::fmt::Write;
    let all_findings: Vec<_> = infra_categories(result)
        .iter()
        .map(|(_, cat)| &cat.findings)
        .chain(std::iter::once(
            &result.categories.provable_contracts.findings,
        ))
        .flat_map(|f| f.iter())
        .collect();

    if all_findings.is_empty() || (!verbose && failures_only) {
        return;
    }
    let _ = writeln!(out, "\n{}", c::subheader("Findings"));
    for finding in &all_findings {
        let icon = match finding.severity {
            InfraSeverity::Fail => c::colored(c::RED, "✗"),
            InfraSeverity::Warning => c::colored(c::YELLOW, "⚠"),
            InfraSeverity::Info => c::colored(c::CYAN, "ℹ"),
            InfraSeverity::Pass => c::colored(c::GREEN, "✓"),
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
    use crate::cli::colors as c;
    use std::fmt::Write;
    if result.recommendations.is_empty() {
        return;
    }
    let _ = writeln!(out, "\n{}", c::subheader("Recommendations"));
    for rec in &result.recommendations {
        let _ = writeln!(
            out,
            "  {}",
            c::colored(
                c::DIM_WHITE,
                &format!(
                    "{}: {} (+{:.0} pts, ~{})",
                    rec.check_id, rec.description, rec.impact_points, rec.estimated_effort
                )
            )
        );
    }
}

fn format_text_output(result: &InfraScore, verbose: bool, failures_only: bool) -> String {
    use crate::cli::colors as c;
    use std::fmt::Write;
    let mut out = String::new();

    let _ = writeln!(out, "{}", c::dim(&"━".repeat(48)));
    let _ = writeln!(out, "{}", c::header("Infra Score v1.0"));
    let _ = writeln!(out, "{}", c::dim(&"━".repeat(48)));

    write_infra_summary(&mut out, result);
    write_infra_categories(&mut out, result, verbose, failures_only);
    write_infra_provable_contracts(&mut out, result, verbose, failures_only);
    write_infra_findings(&mut out, result, verbose, failures_only);
    write_infra_recommendations(&mut out, result);

    // Metadata
    let _ = writeln!(out, "\n{}", c::dim(&"━".repeat(48)));
    let _ = writeln!(
        out,
        "{}",
        c::dim(&format!(
            "Executed in {}ms | pmat v{}",
            result.metadata.execution_time_ms, result.metadata.pmat_version
        ))
    );

    out
}

/// The five scored categories plus their display names, in report order.
///
/// One list, used by the text renderer and the markdown renderer alike — the
/// two must never disagree about which categories exist.
fn infra_categories(result: &InfraScore) -> [(&'static str, &InfraCategoryScore); 5] {
    [
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
    ]
}

/// Render the report as Markdown: no ANSI, no box drawing, tables a docs
/// pipeline can embed verbatim.
fn format_markdown_output(result: &InfraScore, verbose: bool, failures_only: bool) -> String {
    use std::fmt::Write;
    let mut out = String::new();

    let _ = writeln!(out, "# Infra Score v1.0\n");
    let _ = writeln!(out, "## Summary\n");
    let _ = writeln!(
        out,
        "- **Score**: {:.1}/{:.1}",
        result.total_score,
        crate::services::infra_score::models::INFRA_SCORE_MAX_POINTS
    );
    let _ = writeln!(out, "- **Grade**: {}", result.grade.as_str());
    let _ = writeln!(
        out,
        "- **Status**: {}",
        if result.auto_fail {
            "AUTO-FAIL (< 90 required)"
        } else {
            "PASS"
        }
    );
    let bonus = result.categories.provable_contracts.score;
    if bonus > 0.0 {
        let _ = writeln!(out, "- **Bonus**: +{bonus:.1} (provable contracts)");
        let _ = writeln!(
            out,
            "- **Total with bonus**: {:.1}/{:.1}",
            result.categories.total_with_bonus(),
            crate::services::infra_score::models::INFRA_SCORE_MAX_POINTS
                + result.categories.provable_contracts.max_score
        );
    }

    let _ = writeln!(out, "\n## Categories\n");
    let _ = writeln!(out, "| Category | Score | Max | Percentage |");
    let _ = writeln!(out, "|----------|-------|-----|------------|");
    for (name, cat) in infra_categories(result) {
        let _ = writeln!(
            out,
            "| {} | {:.1} | {:.1} | {:.1}% |",
            name, cat.score, cat.max_score, cat.percentage
        );
    }
    let pv = &result.categories.provable_contracts;
    if pv.score > 0.0 || verbose {
        let _ = writeln!(
            out,
            "| Provable Contracts (bonus) | {:.1} | {:.1} | {:.1}% |",
            pv.score, pv.max_score, pv.percentage
        );
    }

    if verbose {
        let _ = writeln!(out, "\n## Checks\n");
        let _ = writeln!(out, "| Check | Name | Score | Max | Result |");
        let _ = writeln!(out, "|-------|------|-------|-----|--------|");
        let mut shown = 0usize;
        let mut total = 0usize;
        for (_, cat) in infra_categories(result).iter().chain(std::iter::once(&(
            "Provable Contracts",
            &result.categories.provable_contracts,
        ))) {
            for check in &cat.checks {
                total += 1;
                if failures_only && check.passed {
                    continue;
                }
                shown += 1;
                let _ = writeln!(
                    out,
                    "| {} | {} | {:.0} | {:.0} | {} |",
                    check.id,
                    check.name,
                    check.score,
                    check.max_score,
                    if check.passed { "pass" } else { "FAIL" }
                );
            }
        }
        if shown < total {
            let _ = writeln!(
                out,
                "\n_Showing {shown} of {total} checks (--failures-only)._"
            );
        }
    }

    let findings: Vec<&InfraFinding> = infra_categories(result)
        .iter()
        .map(|(_, cat)| &cat.findings)
        .chain(std::iter::once(
            &result.categories.provable_contracts.findings,
        ))
        .flat_map(|f| f.iter())
        .filter(|f| !(failures_only && f.severity == InfraSeverity::Pass))
        .collect();
    if !findings.is_empty() {
        let _ = writeln!(out, "\n## Findings\n");
        let _ = writeln!(out, "| Severity | Check | Location | Message |");
        let _ = writeln!(out, "|----------|-------|----------|---------|");
        for f in findings {
            let severity = match f.severity {
                InfraSeverity::Fail => "fail",
                InfraSeverity::Warning => "warning",
                InfraSeverity::Info => "info",
                InfraSeverity::Pass => "pass",
            };
            let _ = writeln!(
                out,
                "| {} | {} | {} | {} |",
                severity,
                f.check_id,
                f.location.as_deref().unwrap_or("-"),
                f.message.replace('|', "\\|").replace('\n', " ")
            );
        }
    }

    if !result.recommendations.is_empty() {
        let _ = writeln!(out, "\n## Recommendations\n");
        for rec in &result.recommendations {
            let _ = writeln!(
                out,
                "- **{}**: {} (+{:.0} pts, ~{})",
                rec.check_id, rec.description, rec.impact_points, rec.estimated_effort
            );
        }
    }

    let _ = writeln!(
        out,
        "\n---\n\n_Executed in {}ms | pmat v{}_",
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
    //
    // These three used to assert `out.contains("\x1b[32m")` — i.e. they PINNED
    // the defect: the renderer emitted ANSI unconditionally, so `infra-score
    // --color never` wrote the same 173 escape sequences as `--color always`.
    // Colour SELECTION (which colour a score maps to) is now asserted on the
    // pure selector, and EMISSION (whether any escape is written) is asserted on
    // the rendered text, which must be plain here because a test binary's stdout
    // is not a terminal.

    #[test]
    fn test_format_text_output_high_score_uses_green() {
        let r = make_score(95.0, InfraGrade::APlus, false);
        let out = format_text_output(&r, false, false);
        // Score >= 90 → green.
        assert_eq!(infra_score_color(95.0), crate::cli::colors::GREEN);
        assert!(out.contains("PASS"));
    }

    #[test]
    fn test_format_text_output_mid_score_uses_yellow() {
        let r = make_score(85.0, InfraGrade::B, true);
        let out = format_text_output(&r, false, false);
        // Score 80-89 → yellow.
        assert_eq!(infra_score_color(85.0), crate::cli::colors::YELLOW);
        assert!(out.contains("AUTO-FAIL"));
    }

    #[test]
    fn test_format_text_output_low_score_uses_red() {
        let r = make_score(50.0, InfraGrade::D, true);
        let out = format_text_output(&r, false, false);
        // Score < 80 → red.
        assert_eq!(infra_score_color(50.0), crate::cli::colors::RED);
        assert!(out.contains("AUTO-FAIL"));
    }

    /// GH round-4: `infra-score --color never` was byte-identical to
    /// `--color always`. Nothing this renderer writes may carry an escape when
    /// colour is off.
    #[test]
    fn infra_text_output_is_plain_when_colour_is_disabled() {
        assert!(
            !crate::cli::colors::colors_enabled(),
            "cargo test captures stdout, so colour must resolve to off here"
        );
        let mut r = make_score(50.0, InfraGrade::D, true);
        r.categories.provable_contracts.score = 5.0;
        r.categories.provable_contracts.percentage = 50.0;
        r.recommendations.push(InfraRecommendation {
            priority: crate::services::infra_score::models::InfraPriority::High,
            check_id: "CI-01".to_string(),
            title: "Add a workflow".to_string(),
            description: "add a workflow".to_string(),
            impact_points: 5.0,
            estimated_effort: "1h".to_string(),
        });
        r.categories.workflow_architecture.findings.push(finding(
            InfraSeverity::Fail,
            "CI-01",
            "no workflow",
        ));
        let out = format_text_output(&r, true, false);
        assert!(
            !out.contains('\x1b'),
            "infra-score text output must be plain with colour off: {out:?}"
        );
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
        // Section header not emitted when findings empty. Asserted on the TEXT,
        // not on an ANSI-wrapped spelling of it: once the renderer honours
        // `--color never` the escape-bearing form never appears at all, so the
        // old assertion passed for a reason unrelated to what it claimed.
        assert!(!out.contains("Findings"));
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
        assert!(!out.contains("Recommendations"));
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

    // ── #942: markdown and yaml were the text report ──

    /// `-f markdown` used to fall through the `_ =>` arm and emit the ANSI text
    /// report, so a docs pipeline embedded box-drawing rules and escape
    /// sequences. Markdown must be markdown.
    #[test]
    fn markdown_output_is_markdown_not_the_text_report() {
        let mut r = make_score(85.0, InfraGrade::B, true);
        r.categories.workflow_architecture.checks = vec![check("WA-01", "Workflow", false, vec![])];
        r.categories.workflow_architecture.findings =
            vec![finding(InfraSeverity::Fail, "WA-01", "no workflow")];

        let md = format_markdown_output(&r, true, false);
        let text = format_text_output(&r, true, false);

        assert_ne!(md, text, "markdown must not be the text report");
        assert!(md.starts_with("# Infra Score v1.0"), "{md}");
        assert!(!md.contains('\u{2501}'), "no box drawing in markdown: {md}");
        assert!(!md.contains('\x1b'), "no ANSI escapes in markdown: {md}");
        assert!(
            md.contains("| Category | Score | Max | Percentage |"),
            "{md}"
        );
        assert!(md.contains("| WA-01 | Workflow |"), "{md}");
        assert!(md.contains("**Score**: 85.0/100.0"), "{md}");
        assert!(md.contains("AUTO-FAIL"), "{md}");
    }

    /// `-f yaml` used to emit the same text report; `yaml.safe_load` failed on
    /// `Score: 7.0/100.0`. The YAML must round-trip back to the same numbers.
    #[test]
    fn yaml_output_parses_back_to_the_scored_report() {
        let r = make_score(42.5, InfraGrade::D, true);
        let yaml = serde_yaml_ng::to_string(&r).expect("serialize infra score to YAML");
        assert!(!yaml.contains('\u{2501}'), "{yaml}");
        assert!(!yaml.contains('\x1b'), "{yaml}");

        let back: InfraScore = serde_yaml_ng::from_str(&yaml).expect("YAML must parse back");
        assert_eq!(back.total_score, 42.5);
        assert!(back.auto_fail);
        assert_ne!(yaml, format_text_output(&r, false, false));
    }

    /// `--failures-only` must reach the markdown check table, exactly as it
    /// reaches the text one.
    #[test]
    fn markdown_failures_only_drops_passing_checks() {
        let mut r = make_score(95.0, InfraGrade::APlus, false);
        r.categories.workflow_architecture.checks = vec![
            check("PASS-01", "Pass", true, vec![]),
            check("FAIL-01", "Fail", false, vec![]),
        ];
        let md = format_markdown_output(&r, true, true);
        assert!(md.contains("FAIL-01"), "{md}");
        assert!(!md.contains("PASS-01"), "{md}");
        assert!(md.contains("(--failures-only)"), "{md}");
    }

    #[test]
    fn test_format_text_output_always_emits_header_and_footer() {
        let r = make_score(95.0, InfraGrade::APlus, false);
        let out = format_text_output(&r, false, false);
        assert!(out.contains("Infra Score v1.0"));
        assert!(out.contains("Executed in"));
        assert!(out.contains("pmat v"));
    }
}
