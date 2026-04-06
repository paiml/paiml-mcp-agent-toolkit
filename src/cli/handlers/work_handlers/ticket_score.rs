// DBC Contract Scoring handler (DBC spec §13.4, §13.7, §14.6)
// `pmat work score <id>` — shows 5-dimension quality score + lint report
// `pmat work codebase-score` — shows aggregate portfolio score

use crate::cli::handlers::work_contract as wc;

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_work_score(
    id: String,
    min_score: f64,
    path: Option<PathBuf>,
    format: String,
) -> Result<()> {
    debug_assert!(min_score >= 0.0, "min_score must be non-negative");
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
    debug_assert!(!id.is_empty(), "id must not be empty");
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
    debug_assert!(!id.is_empty(), "id must not be empty");
    use crate::cli::colors as c;
    println!("{}", c::header(&format!("Contract Score: {} ({})", id, contract.version)));
    println!();
    println!("  {}", c::subheader("5-Dimension Quality Score"));
    println!("  {}", c::separator());
    println!("  spec_depth:      {}  {}(weight: 0.20){}", c::number(&format!("{:.2}", score.spec_depth)), c::DIM, c::RESET);
    println!("  falsification:   {}  {}(weight: 0.25){}", c::number(&format!("{:.2}", score.falsification_coverage)), c::DIM, c::RESET);
    println!("  invariant_health:{}  {}(weight: 0.25){}", c::number(&format!("{:.2}", score.invariant_health)), c::DIM, c::RESET);
    println!("  subcontracting:  {}  {}(weight: 0.10){}", c::number(&format!("{:.2}", score.subcontracting)), c::DIM, c::RESET);
    println!("  traceability:    {}  {}(weight: 0.20){}", c::number(&format!("{:.2}", score.traceability)), c::DIM, c::RESET);
    println!("  {}", c::separator());
    println!("  TOTAL:           {}  Grade: {}", c::number(&format!("{:.2}", score.total)), c::grade(&score.grade.to_string()));
    println!();

    print_drift_text(drift);
    print_lint_text(lint_report, lint_config);
    print_trend_text(trend);

    if lint_report.passed {
        println!("{}", c::pass("Result: PASS"));
    } else {
        println!(
            "{}", c::fail(&format!("Result: FAIL ({} error(s), {} warning(s))", lint_report.error_count, lint_report.warning_count))
        );
    }
}

fn print_drift_text(drift: &wc::DriftMetrics) {
    debug_assert!(true, "contract: print_drift_text");
    use crate::cli::colors as c;
    println!("  {}", c::subheader("Drift Metrics (ABC Theorem)"));
    println!("  {}", c::separator());
    println!("  Hours since checkpoint: {}", c::number(&format!("{:.1}", drift.hours_since_checkpoint)));
    println!("  Drift rate (alpha):    {}", c::number(&format!("{:.3}", drift.drift_rate)));
    println!("  Recovery rate (gamma): {}", c::number(&format!("{:.3}", drift.recovery_rate)));
    println!("  Bounded drift (D*):    {}", c::number(&format!("{:.3}", drift.bounded_drift)));
    if drift.is_stale {
        println!("  STATUS: {}", c::fail("STALE (>24h without checkpoint)"));
    } else {
        println!("  STATUS: {}", c::pass("Fresh"));
    }
    println!();
}

fn print_lint_text(lint_report: &wc::LintReport, lint_config: &wc::LintConfig) {
    debug_assert!(true, "contract: print_lint_text");
    use crate::cli::colors as c;
    if lint_report.findings.is_empty() {
        return;
    }
    println!("  {} {}", c::subheader("Lint Findings"), c::dim(&format!("({} total)", lint_report.findings.len())));
    if lint_config.strict {
        println!("  {}(strict mode: warnings promoted to errors){}", c::YELLOW, c::RESET);
    }
    if !lint_config.suppress.is_empty() {
        println!("  {}({} rule(s) suppressed){}", c::DIM, lint_config.suppress.len(), c::RESET);
    }
    println!("  {}", c::separator());
    for finding in &lint_report.findings {
        let (icon, color) = match finding.severity {
            wc::LintSeverity::Error => ("✗", c::RED),
            wc::LintSeverity::Warning => ("⚠", c::YELLOW),
            wc::LintSeverity::Info => ("ℹ", c::CYAN),
        };
        println!("  {color}{icon}{} {}{}{}: {}", c::RESET, c::DIM, finding.rule_id, c::RESET, finding.message);
    }
    println!();
}

fn print_trend_text(trend: &wc::QualityTrend) {
    debug_assert!(true, "contract: print_trend_text");
    use crate::cli::colors as c;
    if trend.snapshots.is_empty() {
        return;
    }
    println!(
        "  Trend: {}{}{} snapshots, rolling avg {}, {}",
        c::BOLD_WHITE, trend.snapshots.len(), c::RESET,
        c::number(&format!("{:.2}", trend.rolling_average)),
        trend.direction
    );
    if trend.drift_detected {
        println!("  {}", c::warn("Quality drift detected (>5% drop from rolling average)"));
    }
    println!();
}

/// Handle `pmat work codebase-score` — aggregate scoring across all contracts (§14.6)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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
