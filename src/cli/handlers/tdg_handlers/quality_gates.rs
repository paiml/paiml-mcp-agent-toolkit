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

/// The single TDG analysis every `pmat tdg` renderer formats.
///
/// Issue #669, second round: `--format sarif` used to take its own branch out
/// of `handle_tdg_command`, run its own `analyze_project`, and report a number
/// no other format agreed with (SARIF said 72.5/100 (B-) where `--format json`
/// said 94.15/A-). One analysis, many renderers — so there is nothing left for
/// two formats to disagree about.
pub(crate) struct TdgAnalysis {
    /// Per-file scores; `None` when a single file was analyzed.
    pub(crate) project: Option<crate::tdg::ProjectScore>,
    /// The score EVERY format prints as "the" score.
    pub(crate) score: crate::tdg::TdgScore,
    /// The path that was analyzed, for SARIF locations.
    pub(crate) root: PathBuf,
}

/// Run the one TDG analysis backing every output format.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn run_tdg_analysis<'a>(
    analyzer: &'a TdgAnalyzer,
    config: &'a TdgCommandConfig,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<TdgAnalysis>> + 'a>> {
    Box::pin(async move {
        let root = config.path.clone();
        if config.path.is_dir() {
            let project = analyzer.analyze_project(&config.path).await?;
            let score = project.average();
            Ok(TdgAnalysis {
                project: Some(project),
                score,
                root,
            })
        } else {
            let score = analyzer.analyze_file(&config.path).await?;
            Ok(TdgAnalysis {
                project: None,
                score,
                root,
            })
        }
    })
}

/// Execute TDG analysis on file or directory (cognitive complexity ≤3)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn execute_tdg_analysis<'a>(
    analyzer: &'a TdgAnalyzer,
    config: &'a TdgCommandConfig,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<crate::tdg::TdgScore>> + 'a>> {
    Box::pin(async move { Ok(run_tdg_analysis(analyzer, config).await?.score) })
}

/// Validate minimum grade requirement (cognitive complexity ≤4)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn validate_minimum_grade(
    score: &crate::tdg::TdgScore,
    config: &TdgCommandConfig,
) -> Result<()> {
    if let Some(min_grade_str) = &config.min_grade {
        let min_grade = parse_grade(min_grade_str)?;
        // Grade ordering is APlus < A < ... < F (smaller = better), so the
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
            // `--min-grade` used to write only `default_min_grade`, which
            // `MinimumGradeGate::get_min_grade_for_file` consults ONLY for
            // extensions it does not recognise. Every .rs/.ts/.py/.js file was
            // still judged against the built-in per-language table
            // (rust=B+, typescript=B+, python=B, javascript=B), so
            // `--min-grade F` — the loosest threshold that exists — still
            // reported B/B-/C+/C files as "Below minimum grade" and exited 3.
            // An explicitly requested threshold replaces that table outright.
            config.default_min_grade = parse_grade(grade_str)?;
            config.min_grades.clear();
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
    use crate::tdg::{CriticalDefectGate, FGradeGate, QualityGate, TdgBaseline};

    let quiet = json_exclusive_stdout(&format);

    decor!(quiet, "🔍 Checking quality thresholds...");

    let temp_output = ephemeral_temp_json("pmat-quality-check");
    create_baseline(analyzer, path, &temp_output, false, None, quiet).await?;
    let current = TdgBaseline::load(&temp_output)?;
    std::fs::remove_file(&temp_output).ok();

    let f_grade_result = FGradeGate::with_defaults().check(&TdgBaseline::new(None), &current)?;
    // Critical defects are gated on their own flag, not inferred from an F
    // grade. The score they produce is now a graduated penalty, so a defective
    // file need not land in the F band — `FGradeGate` alone would miss it.
    let critical_result =
        CriticalDefectGate::with_defaults().check(&TdgBaseline::new(None), &current)?;
    let result = run_primary_gate(new_files_only, min_grade_str, baseline_path, &current)?;

    if quiet {
        // JSON mode: one combined document — two display_gate_result calls
        // would concatenate two JSON docs on stdout when F-grades exist
        println!(
            "{}",
            check_quality_json(&result, &f_grade_result, &critical_result)?
        );
    } else {
        if !critical_result.violations.is_empty() {
            decor!(quiet, "\n⛔ Critical Defects: {}", critical_result.message);
            display_gate_result(&critical_result, &format)?;
            decor!(quiet);
        }

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

    if fail_on_violation && (!result.passed || !f_grade_result.passed || !critical_result.passed) {
        return Err(anyhow::anyhow!("Quality violations detected"));
    }

    Ok(())
}

/// Single JSON document for check-quality: primary gate plus the F-grade
/// gate, so JSON consumers see both verdicts in one parseable payload
pub(super) fn check_quality_json(
    primary: &crate::tdg::GateResult,
    f_grade: &crate::tdg::GateResult,
    critical: &crate::tdg::GateResult,
) -> Result<String> {
    Ok(serde_json::to_string_pretty(&serde_json::json!({
        "gate": primary,
        "f_grade_gate": f_grade,
        "critical_defect_gate": critical,
        "passed": primary.passed && f_grade.passed && critical.passed,
    }))?)
}

#[cfg(test)]
mod min_grade_flag_tests {
    //! `tdg check-quality --min-grade` must be the threshold every file is
    //! judged against, including files whose language has a built-in default.
    use super::*;
    use crate::tdg::{BaselineEntry, ComponentScores, Grade, Language, TdgBaseline, TdgScore};

    fn baseline_with(path: &str, total: f32, grade: Grade) -> TdgBaseline {
        let mut baseline = TdgBaseline::new(None);
        let path = PathBuf::from(path);
        let entry = BaselineEntry {
            content_hash: blake3::hash(b"min-grade-flag-test"),
            score: TdgScore {
                total,
                grade,
                structural_complexity: total,
                semantic_complexity: total,
                duplication_ratio: 0.0,
                coupling_score: total,
                doc_coverage: total,
                consistency_score: total,
                entropy_score: total,
                confidence: 1.0,
                language: Language::Rust,
                file_path: Some(path.clone()),
                penalties_applied: Vec::new(),
                critical_defects_count: 0,
                has_critical_defects: false,
                critical_defects_suppressed: None,
                has_contract_coverage: false,
            },
            components: ComponentScores::default(),
            git_context: None,
        };
        baseline.add_entry(path, entry);
        baseline
    }

    #[test]
    fn min_grade_f_accepts_a_c_grade_rust_file() {
        let current = baseline_with("src/bad.rs", 73.0, Grade::C);
        let result = run_primary_gate(false, Some("F"), None, &current).unwrap();
        assert!(
            result.passed,
            "--min-grade F is the loosest threshold there is; a C-grade .rs file must pass it, \
             got violations: {:?}",
            result.violations
        );
    }

    #[test]
    fn min_grade_a_still_rejects_a_c_grade_rust_file() {
        let current = baseline_with("src/bad.rs", 73.0, Grade::C);
        let result = run_primary_gate(false, Some("A"), None, &current).unwrap();
        assert!(!result.passed, "--min-grade A must reject a C-grade file");
    }

    #[test]
    fn min_grade_a_plus_counts_every_file_the_distribution_puts_below_a_plus() {
        // The dogfood symptom ran the other way from `--min-grade F`: with
        // `--min-grade A+` the printed grade distribution said 903 files were
        // below A+ while the gate reported only 42 violations, because the
        // built-in rust=B+ entry kept every A/A-/B+ .rs file out of the count.
        let current = baseline_with("src/good.rs", 93.0, Grade::A);
        let result = run_primary_gate(false, Some("A+"), None, &current).unwrap();
        assert!(
            !result.passed && result.violations.len() == 1,
            "an A-grade .rs file is below A+ and must be counted, got: {:?}",
            result.violations
        );
    }

    #[test]
    fn without_min_grade_the_per_language_defaults_still_apply() {
        // Not passing --min-grade leaves the built-in rust=B+ table in place.
        let current = baseline_with("src/bad.rs", 83.0, Grade::B);
        let result = run_primary_gate(false, None, None, &current).unwrap();
        assert!(!result.passed, "rust defaults to B+, so a B file fails");
    }
}
