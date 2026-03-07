// DBC Contract Scoring handler (DBC spec §13.4)
// `pmat work score <id>` — shows 5-dimension quality score + lint report

pub async fn handle_work_score(
    id: String,
    min_score: f64,
    path: Option<PathBuf>,
    format: String,
) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));

    let contract = crate::cli::handlers::work_contract::WorkContract::load(&project_path, &id)
        .with_context(|| {
            format!(
                "No contract found for '{}'. Run 'pmat work start {}' first.",
                id, id
            )
        })?;

    let score =
        crate::cli::handlers::work_contract::score_contract(&contract, &project_path);
    let drift =
        crate::cli::handlers::work_contract::compute_drift_metrics(&contract, &project_path);
    let lint_report =
        crate::cli::handlers::work_contract::lint_contract(&contract, &project_path, min_score);
    let trend =
        crate::cli::handlers::work_contract::load_quality_trend(&project_path, &id);

    // SARIF output (§13.4)
    if format == "sarif" {
        let contract_path = format!(".pmat-work/{}/contract.json", id);
        let sarif = crate::cli::handlers::work_contract::lint_report_to_sarif(
            &lint_report,
            &contract_path,
        );
        println!("{}", serde_json::to_string_pretty(&sarif)?);
        return Ok(());
    }

    if format == "json" {
        let output = serde_json::json!({
            "work_item_id": id,
            "score": {
                "spec_depth": score.spec_depth,
                "falsification_coverage": score.falsification_coverage,
                "invariant_health": score.invariant_health,
                "subcontracting": score.subcontracting,
                "traceability": score.traceability,
                "total": score.total,
                "grade": score.grade.to_string(),
            },
            "drift": {
                "hours_since_checkpoint": drift.hours_since_checkpoint,
                "drift_rate": drift.drift_rate,
                "recovery_rate": drift.recovery_rate,
                "bounded_drift": drift.bounded_drift,
                "is_stale": drift.is_stale,
            },
            "lint": {
                "passed": lint_report.passed,
                "error_count": lint_report.error_count,
                "warning_count": lint_report.warning_count,
                "info_count": lint_report.info_count,
                "findings": lint_report.findings,
            },
            "trend": {
                "snapshots": trend.snapshots.len(),
                "rolling_average": trend.rolling_average,
                "drift_detected": trend.drift_detected,
                "direction": trend.direction.to_string(),
            },
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    // Text format
    println!("Contract Score: {} ({})", id, contract.version);
    println!("==================================");
    println!();
    println!("  5-Dimension Quality Score");
    println!("  -------------------------");
    println!("  spec_depth:      {:.2}  (weight: 0.20)", score.spec_depth);
    println!(
        "  falsification:   {:.2}  (weight: 0.25)",
        score.falsification_coverage
    );
    println!(
        "  invariant_health:{:.2}  (weight: 0.25)",
        score.invariant_health
    );
    println!(
        "  subcontracting:  {:.2}  (weight: 0.10)",
        score.subcontracting
    );
    println!("  traceability:    {:.2}  (weight: 0.20)", score.traceability);
    println!("  -------------------------");
    println!("  TOTAL:           {:.2}  Grade: {}", score.total, score.grade);
    println!();

    println!("  Drift Metrics (ABC Theorem)");
    println!("  ---------------------------");
    println!(
        "  Hours since checkpoint: {:.1}",
        drift.hours_since_checkpoint
    );
    println!("  Drift rate (alpha):    {:.3}", drift.drift_rate);
    println!("  Recovery rate (gamma): {:.3}", drift.recovery_rate);
    println!("  Bounded drift (D*):    {:.3}", drift.bounded_drift);
    if drift.is_stale {
        println!("  STATUS: STALE (>24h without checkpoint)");
    } else {
        println!("  STATUS: Fresh");
    }
    println!();

    if !lint_report.findings.is_empty() {
        println!("  Lint Findings ({} total)", lint_report.findings.len());
        println!("  ---------------------------");
        for finding in &lint_report.findings {
            let icon = match finding.severity {
                crate::cli::handlers::work_contract::LintSeverity::Error => "E",
                crate::cli::handlers::work_contract::LintSeverity::Warning => "W",
                crate::cli::handlers::work_contract::LintSeverity::Info => "I",
            };
            println!("  [{}] {}: {}", icon, finding.rule_id, finding.message);
        }
        println!();
    }

    if !trend.snapshots.is_empty() {
        println!(
            "  Trend: {} snapshots, rolling avg {:.2}, {}",
            trend.snapshots.len(),
            trend.rolling_average,
            trend.direction
        );
        if trend.drift_detected {
            println!("  WARNING: Quality drift detected (>5% drop from rolling average)");
        }
        println!();
    }

    if lint_report.passed {
        println!("Result: PASS");
    } else {
        println!(
            "Result: FAIL ({} error(s), {} warning(s))",
            lint_report.error_count, lint_report.warning_count
        );
    }

    Ok(())
}
