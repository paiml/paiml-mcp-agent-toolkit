#![cfg_attr(coverage_nightly, coverage(off))]
//! TDG quality gate commands: check-regression, check-quality, grade validation
//!
//! Sprint 66 Phase 2: Quality gate enforcement for CI/CD pipelines.

use super::baseline::{create_baseline, decor, ephemeral_temp_json, json_exclusive_stdout};
use super::display::display_gate_result_table;
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

/// Whether `format` owns stdout exclusively, so progress prose must go to
/// stderr.
///
/// GH: `--format sarif` used to print "SARIF format not yet implemented for
/// quality gates" — once per gate — into the SARIF stream, on top of the
/// progress text, and still exit 3. A CI job consuming that artifact saw a
/// failure AND a file `json.tool` rejects at line 1 column 1. SARIF is a
/// machine format like JSON and is held to the same rule.
fn machine_exclusive_stdout(format: &crate::cli::TdgOutputFormat) -> bool {
    json_exclusive_stdout(format) || matches!(format, crate::cli::TdgOutputFormat::Sarif)
}

/// SARIF `level` for a gate violation severity.
fn sarif_level(severity: crate::tdg::Severity) -> &'static str {
    use crate::tdg::Severity;
    match severity {
        Severity::Info => "note",
        Severity::Warning => "warning",
        Severity::Error | Severity::Critical => "error",
    }
}

/// SARIF 2.1.0 document for one or more gate results.
///
/// One run, one rule per gate, one result per violation. `properties` carries
/// the pass/fail verdict of every gate so a consumer can tell "gate ran and
/// passed" from "gate produced no findings".
fn gate_results_sarif(results: &[&crate::tdg::GateResult]) -> serde_json::Value {
    let rules: Vec<serde_json::Value> = results
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.gate_name,
                "name": r.gate_name,
                "shortDescription": { "text": format!("pmat TDG quality gate: {}", r.gate_name) },
            })
        })
        .collect();

    let mut sarif_results = Vec::new();
    for gate in results {
        for v in &gate.violations {
            sarif_results.push(serde_json::json!({
                "ruleId": gate.gate_name,
                "level": sarif_level(v.severity),
                "message": { "text": v.message },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": { "uri": v.path.display().to_string() },
                    }
                }],
                "properties": {
                    "violationType": format!("{:?}", v.violation_type),
                    "severity": format!("{:?}", v.severity),
                    "oldScore": v.old_score,
                    "newScore": v.new_score,
                    "oldGrade": v.old_grade.map(|g| g.to_string()),
                    "newGrade": v.new_grade.to_string(),
                },
            }));
        }
    }

    serde_json::json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": { "driver": {
                "name": "pmat-tdg-quality-gate",
                "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit",
                "version": env!("CARGO_PKG_VERSION"),
                "rules": rules,
            }},
            "properties": {
                "passed": results.iter().all(|r| r.passed),
                "gates": results.iter().map(|r| serde_json::json!({
                    "name": r.gate_name,
                    "passed": r.passed,
                    "message": r.message,
                })).collect::<Vec<_>>(),
            },
            "results": sarif_results,
        }]
    })
}

/// Markdown report for one or more gate results.
fn gate_results_markdown(results: &[&crate::tdg::GateResult]) -> String {
    let mut out = String::from("# TDG Quality Gates\n\n");
    let passed = results.iter().all(|r| r.passed);
    out.push_str(&format!(
        "**Result**: {}\n\n",
        if passed { "PASS" } else { "FAIL" }
    ));

    for gate in results {
        out.push_str(&format!(
            "## {} — {}\n\n{}\n\n",
            gate.gate_name,
            if gate.passed { "pass" } else { "fail" },
            gate.message
        ));
        if gate.violations.is_empty() {
            continue;
        }
        out.push_str("| File | Type | Severity | Grade | Score | Message |\n");
        out.push_str("|------|------|----------|-------|-------|---------|\n");
        for v in &gate.violations {
            out.push_str(&format!(
                "| `{}` | {:?} | {:?} | {} | {:.1} | {} |\n",
                v.path.display(),
                v.violation_type,
                v.severity,
                v.new_grade,
                v.new_score,
                v.message.replace('|', "\\|")
            ));
        }
        out.push('\n');
    }
    out
}

/// Render gate results in `format`, as ONE document per invocation.
///
/// `json_doc` is supplied by the caller because each command has its own
/// established JSON shape (`check-quality` keys its three gates by name).
fn print_gate_results(
    results: &[&crate::tdg::GateResult],
    format: &crate::cli::TdgOutputFormat,
    json_doc: impl FnOnce() -> Result<String>,
) -> Result<()> {
    match format {
        crate::cli::TdgOutputFormat::Table => {
            for r in results {
                display_gate_result_table(r);
            }
        }
        crate::cli::TdgOutputFormat::Json => {
            println!("{}", json_doc()?);
        }
        crate::cli::TdgOutputFormat::Sarif => {
            println!(
                "{}",
                serde_json::to_string_pretty(&gate_results_sarif(results))?
            );
        }
        crate::cli::TdgOutputFormat::Markdown => {
            print!("{}", gate_results_markdown(results));
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

    let quiet = machine_exclusive_stdout(&format);

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
    print_gate_results(&[&result], &format, || {
        Ok(serde_json::to_string_pretty(&result)?)
    })?;

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

/// The file set every `check-quality` gate judges.
///
/// `--new-files-only` is documented as "Check only new files (requires
/// baseline)", but it used to scope only the primary grade gate: `FGradeGate`
/// and `CriticalDefectGate` still ran over the whole project. A repo with a
/// single pre-existing F-grade file therefore exited 3 while printing
/// "✅ No new files added" beside two violation tables for a file that is in
/// the baseline — the mode could never pass on any real repo. The flag names a
/// file set, so it must name it for every gate, not just one.
fn scope_to_new_files(
    new_files_only: bool,
    baseline_path: Option<&PathBuf>,
    current: &crate::tdg::TdgBaseline,
) -> Result<crate::tdg::TdgBaseline> {
    use crate::tdg::TdgBaseline;

    if !new_files_only {
        return Ok(current.clone());
    }
    let baseline_path = baseline_path
        .ok_or_else(|| anyhow::anyhow!("Baseline required for --new-files-only mode"))?;
    let baseline = TdgBaseline::load(baseline_path)?;
    let mut scoped = TdgBaseline::new(current.git_context.clone());
    for (path, entry) in &current.files {
        if !baseline.files.contains_key(path) {
            scoped.add_entry(path.clone(), entry.clone());
        }
    }
    Ok(scoped)
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

    let quiet = machine_exclusive_stdout(&format)
        || matches!(format, crate::cli::TdgOutputFormat::Markdown);

    decor!(quiet, "🔍 Checking quality thresholds...");

    let temp_output = ephemeral_temp_json("pmat-quality-check");
    create_baseline(analyzer, path, &temp_output, false, None, quiet).await?;
    let current = TdgBaseline::load(&temp_output)?;
    std::fs::remove_file(&temp_output).ok();

    let scoped = scope_to_new_files(new_files_only, baseline_path, &current)?;

    let f_grade_result = FGradeGate::with_defaults().check(&TdgBaseline::new(None), &scoped)?;
    // Critical defects are gated on their own flag, not inferred from an F
    // grade. The score they produce is now a graduated penalty, so a defective
    // file need not land in the F band — `FGradeGate` alone would miss it.
    let critical_result =
        CriticalDefectGate::with_defaults().check(&TdgBaseline::new(None), &scoped)?;
    let result = run_primary_gate(new_files_only, min_grade_str, baseline_path, &current)?;

    if matches!(format, crate::cli::TdgOutputFormat::Table) {
        // Human framing around the three tables. Every machine format below
        // gets ONE document instead: three `display_gate_result` calls used to
        // concatenate three payloads on stdout.
        if !critical_result.violations.is_empty() {
            println!("\n⛔ Critical Defects: {}", critical_result.message);
            display_gate_result_table(&critical_result);
            println!();
        }

        if !f_grade_result.violations.is_empty() {
            println!("\n⚠️  F-Grade Warning: {}", f_grade_result.message);
            println!("   F-grades cap project score at B regardless of average.");
            display_gate_result_table(&f_grade_result);
            println!();
        }

        display_gate_result_table(&result);
    } else {
        print_gate_results(
            &[&result, &f_grade_result, &critical_result],
            &format,
            || check_quality_json(&result, &f_grade_result, &critical_result),
        )?;
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

#[cfg(test)]
mod new_files_only_and_machine_format_tests {
    //! Two things `tdg check-quality` used to get wrong on the same run:
    //! `--new-files-only` scoped only the primary gate, and `--format
    //! sarif`/`markdown` wrote English prose into a machine stream.
    use super::*;
    use crate::tdg::{
        BaselineEntry, ComponentScores, CriticalDefectGate, FGradeGate, Grade, Language,
        QualityGate, TdgBaseline, TdgScore,
    };

    fn entry(total: f32, grade: Grade, critical: usize) -> BaselineEntry {
        BaselineEntry {
            content_hash: blake3::hash(b"new-files-only-test"),
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
                file_path: None,
                penalties_applied: Vec::new(),
                critical_defects_count: critical,
                has_critical_defects: critical > 0,
                critical_defects_suppressed: None,
                has_contract_coverage: false,
            },
            components: ComponentScores::default(),
            git_context: None,
        }
    }

    fn write_baseline(dir: &std::path::Path, files: &[(&str, f32, Grade, usize)]) -> PathBuf {
        let mut baseline = TdgBaseline::new(None);
        for (path, total, grade, critical) in files {
            baseline.add_entry(PathBuf::from(path), entry(*total, *grade, *critical));
        }
        let out = dir.join("baseline.json");
        baseline.save(&out).unwrap();
        out
    }

    /// The dogfood symptom: a repo whose ONLY F-grade / critical-defect file is
    /// already in the baseline exited 3 with "✅ No new files added" printed
    /// beside two violation tables for that same baselined file. With no new
    /// files, `--new-files-only` has nothing to judge and every gate must see
    /// an empty set.
    #[test]
    fn new_files_only_hides_pre_existing_f_grade_and_critical_defect_files() {
        let dir = tempfile::tempdir().unwrap();
        let baseline_path = write_baseline(dir.path(), &[("./src/lib.rs", 25.2, Grade::F, 3)]);

        let mut current = TdgBaseline::new(None);
        current.add_entry(PathBuf::from("./src/lib.rs"), entry(25.2, Grade::F, 3));

        let scoped = scope_to_new_files(true, Some(&baseline_path), &current).unwrap();
        assert!(
            scoped.files.is_empty(),
            "a file present in the baseline is not new: {:?}",
            scoped.files.keys().collect::<Vec<_>>()
        );

        let empty = TdgBaseline::new(None);
        let f_grade = FGradeGate::with_defaults().check(&empty, &scoped).unwrap();
        let critical = CriticalDefectGate::with_defaults()
            .check(&empty, &scoped)
            .unwrap();
        assert!(
            f_grade.passed && f_grade.violations.is_empty(),
            "FGradeGate must honour --new-files-only, got {:?}",
            f_grade.violations
        );
        assert!(
            critical.passed && critical.violations.is_empty(),
            "CriticalDefectGate must honour --new-files-only, got {:?}",
            critical.violations
        );
    }

    /// …and a genuinely new bad file is still caught, so the scoping is not a
    /// blanket "pass".
    #[test]
    fn new_files_only_still_reports_a_newly_added_bad_file() {
        let dir = tempfile::tempdir().unwrap();
        let baseline_path = write_baseline(dir.path(), &[("./src/old.rs", 25.2, Grade::F, 3)]);

        let mut current = TdgBaseline::new(None);
        current.add_entry(PathBuf::from("./src/old.rs"), entry(25.2, Grade::F, 3));
        current.add_entry(PathBuf::from("./src/new.rs"), entry(20.0, Grade::F, 2));

        let scoped = scope_to_new_files(true, Some(&baseline_path), &current).unwrap();
        assert_eq!(scoped.files.len(), 1);
        assert!(scoped.files.contains_key(&PathBuf::from("./src/new.rs")));

        let empty = TdgBaseline::new(None);
        let f_grade = FGradeGate::with_defaults().check(&empty, &scoped).unwrap();
        assert!(!f_grade.passed, "the new F-grade file must still fail");
        assert_eq!(f_grade.violations.len(), 1);
    }

    /// Without the flag, the gates keep seeing the whole project.
    #[test]
    fn without_the_flag_every_file_is_still_judged() {
        let mut current = TdgBaseline::new(None);
        current.add_entry(PathBuf::from("./src/lib.rs"), entry(25.2, Grade::F, 3));
        let scoped = scope_to_new_files(false, None, &current).unwrap();
        assert_eq!(scoped.files.len(), 1);
    }

    fn failing_result(name: &str) -> crate::tdg::GateResult {
        crate::tdg::GateResult {
            passed: false,
            gate_name: name.to_string(),
            violations: vec![crate::tdg::Violation {
                path: PathBuf::from("./src/lib.rs"),
                violation_type: crate::tdg::ViolationType::BelowMinimum,
                severity: crate::tdg::Severity::Critical,
                message: "F-grade file: F (25.2 points)".to_string(),
                old_score: None,
                new_score: 25.2,
                old_grade: None,
                new_grade: Grade::F,
            }],
            message: "❌ 1 F-grade file(s) detected".to_string(),
        }
    }

    /// `--format sarif` wrote "SARIF format not yet implemented for quality
    /// gates" — three times, one per gate — onto stdout and still exited 3, so
    /// a CI job got a failure AND an artifact `json.tool` rejects at line 1.
    #[test]
    fn sarif_output_is_a_parseable_sarif_document() {
        let gate = failing_result("FGradeGate");
        let doc = gate_results_sarif(&[&gate]);
        let text = serde_json::to_string(&doc).unwrap();
        assert!(
            !text.contains("not yet implemented"),
            "prose must never reach the SARIF stream: {text}"
        );

        let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(parsed["version"], "2.1.0");
        let run = &parsed["runs"][0];
        assert_eq!(run["tool"]["driver"]["rules"][0]["id"], "FGradeGate");
        assert_eq!(run["properties"]["passed"], false);
        let results = run["results"].as_array().unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0]["ruleId"], "FGradeGate");
        assert_eq!(results[0]["level"], "error");
        assert_eq!(
            results[0]["locations"][0]["physicalLocation"]["artifactLocation"]["uri"],
            "./src/lib.rs"
        );
        // The grade goes out in the symbolic spelling, like every other surface.
        assert_eq!(results[0]["properties"]["newGrade"], "F");
    }

    #[test]
    fn sarif_severity_maps_onto_sarif_levels() {
        use crate::tdg::Severity;
        assert_eq!(sarif_level(Severity::Info), "note");
        assert_eq!(sarif_level(Severity::Warning), "warning");
        assert_eq!(sarif_level(Severity::Error), "error");
        assert_eq!(sarif_level(Severity::Critical), "error");
    }

    /// Same for markdown: a table, not "Markdown format not yet implemented".
    #[test]
    fn markdown_output_is_a_markdown_report() {
        let gate = failing_result("FGradeGate");
        let md = gate_results_markdown(&[&gate]);
        assert!(
            !md.contains("not yet implemented"),
            "prose placeholder still present: {md}"
        );
        assert!(md.starts_with("# TDG Quality Gates"));
        assert!(md.contains("**Result**: FAIL"));
        assert!(md.contains("## FGradeGate — fail"));
        assert!(md.contains("| `./src/lib.rs` |"));
        assert!(md.contains("| F |"));
    }

    /// SARIF is a machine format, so the progress prose must leave stdout to it
    /// exactly as it does for JSON.
    #[test]
    fn sarif_owns_stdout_like_json() {
        use crate::cli::TdgOutputFormat;
        assert!(machine_exclusive_stdout(&TdgOutputFormat::Json));
        assert!(machine_exclusive_stdout(&TdgOutputFormat::Sarif));
        assert!(!machine_exclusive_stdout(&TdgOutputFormat::Table));
    }
}
