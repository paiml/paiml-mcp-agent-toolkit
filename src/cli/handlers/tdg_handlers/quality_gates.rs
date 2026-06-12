#![cfg_attr(coverage_nightly, coverage(off))]
//! TDG quality gate commands: check-regression, check-quality, grade validation
//!
//! Sprint 66 Phase 2: Quality gate enforcement for CI/CD pipelines.

use super::baseline::{create_baseline, decor, ephemeral_temp_json, json_exclusive_stdout};
use super::display::{display_gate_result, display_gate_result_table};
use super::parse_grade;
use super::TdgCommandConfig;
use crate::tdg::TdgAnalyzer;
use anyhow::Result;
use std::path::{Path, PathBuf};

/// Execute TDG analysis on file or directory (cognitive complexity ≤3)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn execute_tdg_analysis<'a>(
    analyzer: &'a TdgAnalyzer,
    config: &'a TdgCommandConfig,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<crate::tdg::TdgScore>> + 'a>> {
    Box::pin(async move {
        if config.path.is_dir() {
            Ok(analyzer.analyze_project(&config.path).await?.average())
        } else {
            analyzer.analyze_file(&config.path).await
        }
    })
}

/// Validate minimum grade requirement (cognitive complexity ≤4)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn validate_minimum_grade(
    score: &crate::tdg::TdgScore,
    config: &TdgCommandConfig,
) -> Result<()> {
    if let Some(min_grade_str) = &config.min_grade {
        let min_grade = parse_grade(min_grade_str)?;
        // Grade ordering is APLus < A < ... < F (smaller = better), so the
        // old `score.grade < min_grade` rejected grades BETTER than the
        // minimum and accepted worse ones
        if !score.grade.meets_threshold(min_grade) {
            return Err(anyhow::anyhow!(
                "Grade {} is below minimum required grade {}",
                super::format_grade(score.grade),
                super::format_grade(min_grade)
            ));
        }
    }
    Ok(())
}

/// Handle check-regression command (Sprint 66 Phase 2)
pub(super) async fn handle_check_regression(
    analyzer: &TdgAnalyzer,
    baseline_path: &Path,
    current_path: &Path,
    format: crate::cli::TdgOutputFormat,
    fail_on_regression: bool,
    max_score_drop: Option<f32>,
    allow_grade_drop: bool,
) -> Result<()> {
    use crate::tdg::{GateConfig, QualityGate, RegressionGate, TdgBaseline};

    let quiet = json_exclusive_stdout(&format);

    decor!(quiet, "🔍 Checking for quality regressions...");

    // Load baseline
    let baseline = TdgBaseline::load(baseline_path)?;
    decor!(
        quiet,
        "   ✅ Loaded baseline: {} files",
        baseline.summary.total_files
    );

    // Create current baseline
    let temp_output = ephemeral_temp_json("pmat-regression-check");
    create_baseline(analyzer, current_path, &temp_output, false, None, quiet).await?;
    let current = TdgBaseline::load(&temp_output)?;
    std::fs::remove_file(&temp_output).ok();

    // Configure gate
    let mut config = GateConfig::default();
    if let Some(drop) = max_score_drop {
        config.max_score_drop = drop;
    }
    config.allow_grade_drop = allow_grade_drop;

    // Run regression gate
    let gate = RegressionGate::new(config);
    let result = gate.check(&baseline, &current)?;

    // Display results
    match &format {
        crate::cli::TdgOutputFormat::Table => display_gate_result_table(&result),
        crate::cli::TdgOutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        crate::cli::TdgOutputFormat::Sarif => {
            println!("SARIF format not yet implemented for quality gates");
        }
        crate::cli::TdgOutputFormat::Markdown => {
            println!("Markdown format not yet implemented for quality gates");
        }
    }

    // Exit with error if requested and gate failed
    if fail_on_regression && !result.passed {
        return Err(anyhow::anyhow!("Quality regression detected"));
    }

    Ok(())
}

/// Run the primary quality gate based on mode
fn run_primary_gate(
    new_files_only: bool,
    min_grade_str: Option<&str>,
    baseline_path: Option<&PathBuf>,
    current: &crate::tdg::TdgBaseline,
) -> Result<crate::tdg::GateResult> {
    use crate::tdg::{GateConfig, MinimumGradeGate, NewFileGate, QualityGate, TdgBaseline};

    if new_files_only {
        let baseline_path = baseline_path
            .ok_or_else(|| anyhow::anyhow!("Baseline required for --new-files-only mode"))?;
        let baseline = TdgBaseline::load(baseline_path)?;
        let mut config = GateConfig::default();
        if let Some(grade_str) = min_grade_str {
            config.new_file_min_grade = parse_grade(grade_str)?;
        }
        NewFileGate::new(config).check(&baseline, current)
    } else {
        let baseline = TdgBaseline::new(None);
        let mut config = GateConfig::default();
        if let Some(grade_str) = min_grade_str {
            config.default_min_grade = parse_grade(grade_str)?;
        }
        MinimumGradeGate::new(config).check(&baseline, current)
    }
}

/// Handle check-quality command (Sprint 66 Phase 2)
pub(super) async fn handle_check_quality(
    analyzer: &TdgAnalyzer,
    path: &Path,
    min_grade_str: Option<&str>,
    format: crate::cli::TdgOutputFormat,
    fail_on_violation: bool,
    new_files_only: bool,
    baseline_path: Option<&PathBuf>,
) -> Result<()> {
    use crate::tdg::{FGradeGate, QualityGate, TdgBaseline};

    let quiet = json_exclusive_stdout(&format);

    decor!(quiet, "🔍 Checking quality thresholds...");

    let temp_output = ephemeral_temp_json("pmat-quality-check");
    create_baseline(analyzer, path, &temp_output, false, None, quiet).await?;
    let current = TdgBaseline::load(&temp_output)?;
    std::fs::remove_file(&temp_output).ok();

    let f_grade_result = FGradeGate::with_defaults().check(&TdgBaseline::new(None), &current)?;
    let result = run_primary_gate(new_files_only, min_grade_str, baseline_path, &current)?;

    if quiet {
        // JSON mode: one combined document — two display_gate_result calls
        // would concatenate two JSON docs on stdout when F-grades exist
        println!("{}", check_quality_json(&result, &f_grade_result)?);
    } else {
        if !f_grade_result.violations.is_empty() {
            decor!(quiet, "\n⚠️  F-Grade Warning: {}", f_grade_result.message);
            decor!(
                quiet,
                "   F-grades cap project score at B regardless of average."
            );
            display_gate_result(&f_grade_result, &format)?;
            decor!(quiet);
        }

        display_gate_result(&result, &format)?;
    }

    if fail_on_violation && (!result.passed || !f_grade_result.passed) {
        return Err(anyhow::anyhow!("Quality violations detected"));
    }

    Ok(())
}

/// Single JSON document for check-quality: primary gate plus the F-grade
/// gate, so JSON consumers see both verdicts in one parseable payload
pub(super) fn check_quality_json(
    primary: &crate::tdg::GateResult,
    f_grade: &crate::tdg::GateResult,
) -> Result<String> {
    Ok(serde_json::to_string_pretty(&serde_json::json!({
        "gate": primary,
        "f_grade_gate": f_grade,
        "passed": primary.passed && f_grade.passed,
    }))?)
}
