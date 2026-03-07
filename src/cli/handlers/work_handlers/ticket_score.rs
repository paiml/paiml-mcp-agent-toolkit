// DBC Contract Scoring handler (DBC spec §13.4, §13.7, §14.6)
// `pmat work score <id>` — shows 5-dimension quality score + lint report
// `pmat work codebase-score` — shows aggregate portfolio score

use crate::cli::handlers::work_contract as wc;

pub async fn handle_work_score(
    id: String,
    min_score: f64,
    path: Option<PathBuf>,
    format: String,
) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));

    let contract = wc::WorkContract::load(&project_path, &id).with_context(|| {
        format!(
            "No contract found for '{}'. Run 'pmat work start {}' first.",
            id, id
        )
    })?;

    let score = wc::score_contract(&contract, &project_path);
    let drift = wc::compute_drift_metrics(&contract, &project_path);
    let lint_config = wc::LintConfig::load(&project_path);
    let effective_min = lint_config.min_score.max(min_score);
    let raw_report = wc::lint_contract(&contract, &project_path, effective_min);
    let lint_report = wc::apply_lint_config(&raw_report, &lint_config);
    let trend = wc::load_quality_trend(&project_path, &id);

    if format == "sarif" {
        let contract_path = format!(".pmat-work/{}/contract.json", id);
        let sarif = wc::lint_report_to_sarif(&lint_report, &contract_path);
        println!("{}", serde_json::to_string_pretty(&sarif)?);
        return Ok(());
    }

    if format == "json" {
        return print_score_json(&id, &score, &drift, &lint_report, &trend, &lint_config);
    }

    print_score_text(&id, &contract, &score, &drift, &lint_report, &trend, &lint_config);
    Ok(())
}

fn print_score_json(
    id: &str,
    score: &wc::ContractScore,
    drift: &wc::DriftMetrics,
    lint_report: &wc::LintReport,
    trend: &wc::QualityTrend,
    lint_config: &wc::LintConfig,
) -> Result<()> {
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
        "config": {
            "strict": lint_config.strict,
            "suppressed_rules": lint_config.suppress,
            "overrides": lint_config.rules,
        },
    });
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn print_score_text(
    id: &str,
    contract: &wc::WorkContract,
    score: &wc::ContractScore,
    drift: &wc::DriftMetrics,
    lint_report: &wc::LintReport,
    trend: &wc::QualityTrend,
    lint_config: &wc::LintConfig,
) {
    println!("Contract Score: {} ({})", id, contract.version);
    println!("==================================");
    println!();
    println!("  5-Dimension Quality Score");
    println!("  -------------------------");
    println!("  spec_depth:      {:.2}  (weight: 0.20)", score.spec_depth);
    println!("  falsification:   {:.2}  (weight: 0.25)", score.falsification_coverage);
    println!("  invariant_health:{:.2}  (weight: 0.25)", score.invariant_health);
    println!("  subcontracting:  {:.2}  (weight: 0.10)", score.subcontracting);
    println!("  traceability:    {:.2}  (weight: 0.20)", score.traceability);
    println!("  -------------------------");
    println!("  TOTAL:           {:.2}  Grade: {}", score.total, score.grade);
    println!();

    print_drift_text(drift);
    print_lint_text(lint_report, lint_config);
    print_trend_text(trend);

    if lint_report.passed {
        println!("Result: PASS");
    } else {
        println!(
            "Result: FAIL ({} error(s), {} warning(s))",
            lint_report.error_count, lint_report.warning_count
        );
    }
}

fn print_drift_text(drift: &wc::DriftMetrics) {
    println!("  Drift Metrics (ABC Theorem)");
    println!("  ---------------------------");
    println!("  Hours since checkpoint: {:.1}", drift.hours_since_checkpoint);
    println!("  Drift rate (alpha):    {:.3}", drift.drift_rate);
    println!("  Recovery rate (gamma): {:.3}", drift.recovery_rate);
    println!("  Bounded drift (D*):    {:.3}", drift.bounded_drift);
    let status = if drift.is_stale { "STALE (>24h without checkpoint)" } else { "Fresh" };
    println!("  STATUS: {}", status);
    println!();
}

fn print_lint_text(lint_report: &wc::LintReport, lint_config: &wc::LintConfig) {
    if lint_report.findings.is_empty() {
        return;
    }
    println!("  Lint Findings ({} total)", lint_report.findings.len());
    if lint_config.strict {
        println!("  (strict mode: warnings promoted to errors)");
    }
    if !lint_config.suppress.is_empty() {
        println!("  ({} rule(s) suppressed)", lint_config.suppress.len());
    }
    println!("  ---------------------------");
    for finding in &lint_report.findings {
        let icon = match finding.severity {
            wc::LintSeverity::Error => "E",
            wc::LintSeverity::Warning => "W",
            wc::LintSeverity::Info => "I",
        };
        println!("  [{}] {}: {}", icon, finding.rule_id, finding.message);
    }
    println!();
}

fn print_trend_text(trend: &wc::QualityTrend) {
    if trend.snapshots.is_empty() {
        return;
    }
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

/// Handle `pmat work codebase-score` — aggregate scoring across all contracts (§14.6)
pub async fn handle_work_codebase_score(path: Option<PathBuf>, format: String) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));
    let score = wc::compute_codebase_score(&project_path);

    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&score)?);
        return Ok(());
    }

    println!("Codebase Quality Score");
    println!("======================");
    println!();
    println!("  Contracts:         {}", score.contract_count);
    println!("  Coverage (>=C):    {:.0}%", score.contract_coverage * 100.0);
    println!("  Mean score:        {:.2}", score.mean_score);
    println!("  Min score:         {:.2}", score.min_score);
    println!("  Max score:         {:.2}", score.max_score);
    println!("  Mean drift:        {:.3}", score.mean_drift);
    println!("  Lint pass rate:    {:.0}%", score.lint_pass_rate * 100.0);
    println!("  ----------------------");
    println!("  COMPOSITE:         {:.2}  Grade: {}", score.composite, score.grade);

    if score.contract_count == 0 {
        println!();
        println!("  No active work contracts found.");
        println!("  Start a work item: pmat work start <ID>");
    }

    Ok(())
}
