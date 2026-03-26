#![cfg_attr(coverage_nightly, coverage(off))]
//! CLI handler for `pmat infra-score` command
//!
//! Calculates Infrastructure Score (0-100 + 10 bonus) for CI/CD,
//! build reliability, quality pipeline, deployment, supply chain,
//! and provable contracts.

use crate::cli::RepoScoreOutputFormat;
use crate::services::infra_score::aggregator::InfraScoreAggregator;
use anyhow::Result;
use std::path::Path;

/// Handle the infra-score command
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

fn format_text_output(
    result: &crate::services::infra_score::models::InfraScore,
    verbose: bool,
    failures_only: bool,
) -> String {
    use std::fmt::Write;
    let mut out = String::new();

    let _ = writeln!(out, "\x1b[2m{}\x1b[0m", "━".repeat(48));
    let _ = writeln!(out, "\x1b[1m\x1b[4mInfra Score v1.0\x1b[0m");
    let _ = writeln!(out, "\x1b[2m{}\x1b[0m", "━".repeat(48));

    // Summary
    let _ = writeln!(out, "\n\x1b[1mSummary\x1b[0m");
    let score_color = if result.total_score >= 90.0 {
        "\x1b[32m"
    } else if result.total_score >= 80.0 {
        "\x1b[33m"
    } else {
        "\x1b[31m"
    };
    let _ = writeln!(
        out,
        "  Score: {}{:.1}\x1b[0m/\x1b[2m100.0\x1b[0m",
        score_color, result.total_score
    );
    let _ = writeln!(out, "  Grade: {}{}\x1b[0m", score_color, result.grade.as_str());
    if result.auto_fail {
        let _ = writeln!(out, "  Status: \x1b[31mAUTO-FAIL\x1b[0m (< 90 required)");
    } else {
        let _ = writeln!(out, "  Status: \x1b[32mPASS\x1b[0m");
    }

    // Bonus
    let bonus = result.categories.provable_contracts.score;
    if bonus > 0.0 {
        let _ = writeln!(
            out,
            "  Bonus: \x1b[36m+{:.1}\x1b[0m (provable contracts)",
            bonus
        );
        let _ = writeln!(
            out,
            "  Total with bonus: {}{:.1}\x1b[0m/110.0",
            score_color,
            result.categories.total_with_bonus()
        );
    }

    // Categories
    let _ = writeln!(out, "\n\x1b[1mCategories\x1b[0m");
    let categories = [
        ("Workflow Architecture", &result.categories.workflow_architecture),
        ("Build Reliability", &result.categories.build_reliability),
        ("Quality Pipeline", &result.categories.quality_pipeline),
        ("Deployment & Release", &result.categories.deployment_release),
        ("Supply Chain Security", &result.categories.supply_chain),
    ];

    for (name, cat) in &categories {
        let icon = if cat.percentage >= 90.0 {
            "\x1b[32m✓\x1b[0m"
        } else if cat.percentage >= 70.0 {
            "\x1b[33m⚠\x1b[0m"
        } else {
            "\x1b[31m✗\x1b[0m"
        };
        let pct_color = if cat.percentage >= 90.0 {
            "\x1b[32m"
        } else if cat.percentage >= 70.0 {
            "\x1b[33m"
        } else {
            "\x1b[31m"
        };
        let _ = writeln!(
            out,
            "  {} {}: {}{:.1}\x1b[0m/\x1b[2m{:.1}\x1b[0m ({}{:.1}%\x1b[0m)",
            icon, name, pct_color, cat.score, cat.max_score, pct_color, cat.percentage
        );

        if verbose && !cat.checks.is_empty() {
            for check in &cat.checks {
                if failures_only && check.passed {
                    continue;
                }
                let check_icon = if check.passed { "  ✓" } else { "  ✗" };
                let _ = writeln!(
                    out,
                    "    {} {} ({}): {:.0}/{:.0}",
                    check_icon, check.id, check.name, check.score, check.max_score
                );
                if !check.passed || verbose {
                    for ev in &check.evidence {
                        let _ = writeln!(out, "      {}", ev);
                    }
                }
            }
        }
    }

    // Provable Contracts bonus
    let pv = &result.categories.provable_contracts;
    if pv.score > 0.0 || verbose {
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
                let check_icon = if check.passed { "  ✓" } else { "  ✗" };
                let _ = writeln!(
                    out,
                    "    {} {} ({}): {:.0}/{:.0}",
                    check_icon, check.id, check.name, check.score, check.max_score
                );
            }
        }
    }

    // Findings
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

    if !all_findings.is_empty() && (verbose || !failures_only) {
        let _ = writeln!(out, "\n\x1b[1mFindings\x1b[0m");
        for finding in &all_findings {
            let icon = match finding.severity {
                crate::services::infra_score::models::InfraSeverity::Fail => "\x1b[31m✗\x1b[0m",
                crate::services::infra_score::models::InfraSeverity::Warning => "\x1b[33m⚠\x1b[0m",
                crate::services::infra_score::models::InfraSeverity::Info => "\x1b[36mℹ\x1b[0m",
                crate::services::infra_score::models::InfraSeverity::Pass => "\x1b[32m✓\x1b[0m",
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

    // Recommendations
    if !result.recommendations.is_empty() {
        let _ = writeln!(out, "\n\x1b[1mRecommendations\x1b[0m");
        for rec in &result.recommendations {
            let _ = writeln!(
                out,
                "  \x1b[2;37m{}: {} (+{:.0} pts, ~{})\x1b[0m",
                rec.check_id, rec.description, rec.impact_points, rec.estimated_effort
            );
        }
    }

    // Metadata
    let _ = writeln!(out, "\n\x1b[2m{}\x1b[0m", "━".repeat(48));
    let _ = writeln!(
        out,
        "\x1b[2mExecuted in {}ms | pmat v{}\x1b[0m",
        result.metadata.execution_time_ms, result.metadata.pmat_version
    );

    out
}
