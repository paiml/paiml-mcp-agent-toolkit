use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::cli::enums::TdgOutputFormat;
use crate::tdg::formatters::{
    format_comparison, format_human, format_json, format_markdown, format_project,
};
use crate::tdg::TdgAnalyzer;

/// Configuration for TDG analysis (SPRINT-22)
#[derive(Debug, Clone)]
pub struct TdgAnalysisConfig {
    pub path: PathBuf,
    pub threshold: Option<f64>,
    pub top_files: Option<usize>,
    pub format: TdgOutputFormat,
    pub include_components: bool,
    pub output: Option<PathBuf>,
    pub critical_only: bool,
    pub verbose: bool,
}

/// True when `path` is test/bench/example/fuzz source rather than production code.
///
/// The detector has its own exclusion list but it only recognises `_test.rs`,
/// `_tests.rs` and `test_*` file names. This repo splits its test modules into
/// files the detector therefore scans as production code — `hygiene_scorer/tests.rs`
/// (a bare `tests.rs`) and `lang_analyzer_tests_part4.rs` (a `_tests_` infix) —
/// and every `.unwrap()` in a test helper that is not itself inside a `#[test]`
/// item was reported as a CRITICAL production defect. That is what made
/// `analyze tdg -p src` exit 1 with 80 "critical defects" while printing
/// `Average Score: 95.5/100 (B)` (issue #705).
fn is_test_source(path: &Path) -> bool {
    if path.parent().is_some_and(|parent| {
        parent.components().any(|c| {
            matches!(
                c.as_os_str().to_str(),
                Some("tests" | "benches" | "examples" | "fuzz")
            )
        })
    }) {
        return true;
    }

    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        return false;
    };
    let stem = name.strip_suffix(".rs").unwrap_or(name);
    stem == "tests"
        || stem == "test"
        || stem.starts_with("test_")
        || stem.starts_with("tests_")
        || stem.ends_with("_test")
        || stem.ends_with("_tests")
        || stem.contains("_test_")
        || stem.contains("_tests_")
}

/// Check for critical defects in the project (Known Defects v2.1).
///
/// Reports what it found; only the `analyze build-tdg` gate turns a finding into
/// a non-zero exit (see `run_tdg_analysis`).
async fn check_for_critical_defects(path: &Path, fail_on_critical: bool) -> Result<()> {
    use crate::services::defect_detector::{RustDefectDetector, Severity};
    use ignore::WalkBuilder;

    let detector = RustDefectDetector::new();
    let mut critical_defects_found = false;
    let mut critical_count = 0;

    eprintln!("🔍 Checking for critical defects...");

    // Scan Rust files in the project. Gitignore-aware: the bare `WalkDir`
    // this replaces also descended into gitignored trees — on this repo the
    // ephemeral `.claude/worktrees/` checkouts, which are copies of the very
    // files being scanned, so the same defect was counted once per copy.
    // `follow_links` stays off for the same reason.
    for entry in WalkBuilder::new(path)
        .follow_links(false)
        .hidden(false)
        .parents(true)
        .ignore(true)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .build()
        .filter_map(std::result::Result::ok)
    {
        let file_path = entry.path();

        // Only process Rust files
        if !file_path.is_file() || file_path.extension() != Some(std::ffi::OsStr::new("rs")) {
            continue;
        }

        // Test code is not a production defect — see `is_test_source`.
        if is_test_source(file_path) {
            continue;
        }

        // Read file content
        let content = match tokio::fs::read_to_string(file_path).await {
            Ok(c) => c,
            Err(_) => continue, // Skip files we can't read
        };

        // Detect defects
        let defects = detector.detect(&content, file_path);

        // Count critical defects
        for defect in &defects {
            if defect.severity == Severity::Critical {
                critical_defects_found = true;
                critical_count += defect.instances.len();

                // Print first occurrence for visibility
                if let Some(instance) = defect.instances.first() {
                    eprintln!(
                        "❌ CRITICAL DEFECT: {} in {}:{}:{}",
                        defect.name, instance.file, instance.line, instance.column
                    );
                    eprintln!("   Code: {}", instance.code_snippet);
                }
            }
        }
    }

    if critical_defects_found {
        if !fail_on_critical {
            // `analyze tdg` is a report. It used to bail here unconditionally, so
            // the command printed `Average Score: 95.5/100 (B)` on stdout and
            // then exited 1 on stderr — an exit code contradicting the analysis
            // it had just produced (issue #705). The gate lives in
            // `analyze build-tdg`, exactly as `--threshold` does.
            eprintln!(
                "\n⚠️  {critical_count} critical defect(s) found (advisory). `analyze tdg` \
                 reports; `pmat analyze build-tdg` gates on them."
            );
            eprintln!(
                "   Run: pmat analyze defects --path {} --format text",
                path.display()
            );
            return Ok(());
        }
        eprintln!(
            "\n⛔ TDG ANALYSIS FAILED: Found {} critical defect(s)",
            critical_count
        );
        eprintln!("   Critical defects must be fixed before deployment.");
        eprintln!(
            "   Run: pmat analyze defects --path {} --format text",
            path.display()
        );
        anyhow::bail!("TDG auto-fail: Critical defects detected")
    }

    eprintln!("✅ No critical defects found");
    Ok(())
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn handle_analyze_tdg(config: TdgAnalysisConfig) -> Result<()> {
    run_tdg_analysis(config, false).await
}

/// `analyze build-tdg`: the same analysis, plus the documented quality gate.
///
/// Kept separate from `handle_analyze_tdg` because `analyze tdg`'s `--threshold`
/// is documented as a result filter, not a gate; only `build-tdg` promises to
/// "fail fast" on it.
pub async fn handle_analyze_tdg_gated(config: TdgAnalysisConfig) -> Result<()> {
    run_tdg_analysis(config, true).await
}

async fn run_tdg_analysis(config: TdgAnalysisConfig, enforce_threshold: bool) -> Result<()> {
    eprintln!("🔍 Starting TDG (Technical Debt Grading) analysis...");

    let analyzer = TdgAnalyzer::new()?;
    let threshold = config.threshold;
    // `-n/--top-files` was read into a discarded binding, so it truncated
    // nothing: -n 5 / -n 10 / -n 100000 all emitted the same file list.
    let top_files = config.top_files.unwrap_or(10);

    // `--threshold` is a documented result FILTER for `analyze tdg`, but its
    // default (1.5) and its help text are phrased for the retired 0-5 debt
    // gradient; scores are now 0-100 where higher is better, so there is no
    // filter direction that both honours the flag and leaves the default
    // invocation showing anything at all. It is enforced as a gate by
    // `analyze build-tdg` and by nothing else — say so instead of accepting the
    // value and silently discarding it, which is what `let _threshold = ...`
    // used to do.
    if !enforce_threshold {
        if let Some(t) = threshold {
            if (t - 1.5).abs() > f64::EPSILON {
                eprintln!(
                    "⚠️  --threshold {t} was not applied: `analyze tdg` reports every analysed \
                     file (see -n/--top-files and --critical-only). --threshold gates \
                     `analyze build-tdg` only."
                );
            }
        }
    }

    // `--include-components` adds sections to the human renderers; the machine
    // ones serialise every component of every reported file unconditionally, so
    // say that rather than let the flag look inert.
    if config.include_components
        && matches!(
            config.format,
            TdgOutputFormat::Json | TdgOutputFormat::Sarif
        )
    {
        eprintln!(
            "ℹ️  --include-components: -f json/sarif already carry every per-file component; \
             the flag adds breakdown sections to -f table and -f markdown."
        );
    }

    let (result, measured_score) = if config.path.is_dir() {
        analyze_project_path(
            &analyzer,
            &config.path,
            &config.format,
            top_files,
            config.critical_only,
            config.include_components,
        )
        .await?
    } else {
        analyze_single_file(&analyzer, &config.path, &config.format).await?
    };

    write_or_print_result(&result, config.output).await?;

    // KNOWN DEFECTS v2.1: Check for critical defects. Auto-fail is the gated
    // (`analyze build-tdg`) command's job only — see #705.
    check_for_critical_defects(&config.path, enforce_threshold).await?;

    eprintln!("✅ TDG analysis complete");

    if enforce_threshold {
        enforce_tdg_threshold(measured_score, threshold.unwrap_or(2.0))?;
    }
    Ok(())
}

/// The Jidoka gate `analyze build-tdg --help` promises.
///
/// The threshold used to be bound to `_threshold` and never compared, so on a
/// deliberately pathological crate every value from 0.0 to 1000 exited 0 — a
/// gate that cannot fail is not a gate. It is now enforced against the ONE
/// number the command just printed.
///
/// The flag's help text ("fail if exceeded", defaults 1.5/2.0) is phrased for
/// the retired 0–5 debt-gradient scale. TDG now scores quality on 0–100 where
/// HIGHER IS BETTER, so comparing "exceeds" against it would fail every healthy
/// project; the threshold is enforced as a minimum score instead, and the
/// comparison is printed in full so the gate is never silent about what it
/// measured or which way round it read the number.
///
/// GH #704: `measured` is an `Option` because a run that analysed no file has
/// no score. A gate that could not measure must not report a pass, so `None`
/// fails the gate instead of being compared as 0.0.
fn enforce_tdg_threshold(measured: Option<f32>, threshold: f64) -> Result<()> {
    let Some(measured) = measured else {
        anyhow::bail!(
            "TDG gate failed: no file under this path could be graded, so there is no score to \
             compare against the required minimum of {threshold:.1} (--threshold)"
        );
    };
    eprintln!(
        "🚦 TDG gate: measured {measured:.1}/100 against required minimum {threshold:.1} \
         (--threshold, on the 0-100 scale where higher is better)"
    );
    if f64::from(measured) < threshold {
        anyhow::bail!(
            "TDG gate failed: score {measured:.1}/100 is below the required minimum of {threshold:.1} (--threshold)"
        );
    }
    Ok(())
}

async fn analyze_project_path(
    analyzer: &TdgAnalyzer,
    path: &Path,
    format: &TdgOutputFormat,
    top_files: usize,
    critical_only: bool,
    include_components: bool,
) -> Result<(String, Option<f32>)> {
    let mut project_score = analyzer.analyze_project(path).await?;
    // Honour --top-files for every renderer; aggregates stay whole-project and
    // the truncation is disclosed (files_reported / files_truncated).
    project_score.limit_to_worst_files(top_files);
    // `--critical-only` had no reader at all: on a tree whose worst file graded
    // A it still listed all 9 files and called them critical. A "critical" file
    // is an F-grade file — the same definition `f_grade_count` documents — so
    // filter to those and let the count fall to zero when there are none.
    // Applied after the --top-files cap so the reported/truncated bookkeeping
    // reflects the list actually printed.
    if critical_only {
        retain_critical_files(&mut project_score);
    }
    let average_score = project_score.average_score;
    // `root` is the analysed path, not the process CWD -- that distinction is
    // the #680-round-3 fix for grades depending on the caller's directory.
    Ok((
        format_project_result(&project_score, path, format, include_components)?,
        average_score,
    ))
}

/// Keep only the critical (F-grade) files in the reported list.
///
/// Whole-project aggregates (`total_files`, `average_score`, `grade_distribution`,
/// `f_grade_count`) are deliberately left untouched, exactly as `--top-files`
/// truncation leaves them, so the reported subset can never be mistaken for the
/// analysed population.
fn retain_critical_files(project_score: &mut crate::tdg::ProjectScore) {
    project_score
        .files
        .retain(|file| file.grade == crate::tdg::Grade::F);
    project_score.files_reported = project_score.files.len();
    project_score.files_truncated = project_score.files_reported < project_score.total_files;
}

async fn analyze_single_file(
    analyzer: &TdgAnalyzer,
    path: &Path,
    format: &TdgOutputFormat,
) -> Result<(String, Option<f32>)> {
    let score = analyzer.analyze_file(path).await?;
    let total = Some(score.total);
    Ok((format_file_result(&score, format)?, total))
}

fn format_project_result(
    project_score: &crate::tdg::ProjectScore,
    root: &Path,
    format: &TdgOutputFormat,
    include_components: bool,
) -> Result<String> {
    let result = match format {
        TdgOutputFormat::Table => {
            let mut out = format_project(project_score);
            if include_components {
                out.push_str(&components_text(project_score));
            }
            out
        }
        TdgOutputFormat::Json => serde_json::to_string_pretty(project_score)?,
        // `-f markdown` used to dispatch to `format_project`, i.e. the SAME
        // box-drawing table `-f table` prints (`md5sum` of the two outputs
        // matched byte for byte). Markdown now gets markdown.
        TdgOutputFormat::Markdown => project_markdown(project_score, include_components),
        TdgOutputFormat::Sarif => {
            let sarif = create_sarif_output(project_score, root);
            serde_json::to_string_pretty(&sarif)?
        }
    };
    Ok(result)
}

/// Percentage of `total`, or 0.0 when nothing was analysed (never NaN).
fn percent_of(count: usize, total: usize) -> f32 {
    if total == 0 {
        0.0
    } else {
        (count as f32 / total as f32) * 100.0
    }
}

/// Render a whole-project TDG score as Markdown.
///
/// `-f markdown` shared `format_project` with `-f table`, so it emitted
/// U+2500/U+2502 box drawing that no Markdown renderer turns into a table.
fn project_markdown(project: &crate::tdg::ProjectScore, include_components: bool) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    let w = &mut out;

    let _ = writeln!(w, "# Project TDG Score Report\n");
    // GH #704: same rule as the table renderer — 0 analysed files has no score
    // and no grade, so it must not print the 0.0/100 (F) default.
    let _ = match (project.average_score, project.average_grade) {
        (Some(score), Some(grade)) => {
            writeln!(w, "**Average Score:** {score:.1}/100 ({grade})")
        }
        _ => writeln!(w, "**Average Score:** not measured (no files analysed)"),
    };
    let _ = writeln!(w, "**Total Files:** {}", project.total_files);
    // A truncated list says so, exactly as the box-drawing renderer does.
    if project.files_truncated {
        let _ = writeln!(
            w,
            "**Files Listed:** {} of {} (--top-files)",
            project.files_reported, project.total_files
        );
    }
    if project.grade_capped {
        let _ = writeln!(
            w,
            "**Grade Capped:** yes ({} F-grade file(s))",
            project.f_grade_count
        );
    }

    let _ = writeln!(w, "\n## Language Distribution\n");
    let _ = writeln!(w, "| Language | Files | Share |");
    let _ = writeln!(w, "|---|---:|---:|");
    for (language, count) in &project.language_distribution {
        let _ = writeln!(
            w,
            "| {} | {} | {:.1}% |",
            language,
            count,
            percent_of(*count, project.total_files)
        );
    }

    let _ = writeln!(w, "\n## Grade Distribution\n");
    let _ = writeln!(w, "| Grade | Files | Share |");
    let _ = writeln!(w, "|---|---:|---:|");
    for (grade, count) in &project.grade_distribution {
        let _ = writeln!(
            w,
            "| {} | {} | {:.1}% |",
            grade,
            count,
            percent_of(*count, project.total_files)
        );
    }

    let _ = writeln!(w, "\n## Files\n");
    let _ = writeln!(w, "| File | Score | Grade |");
    let _ = writeln!(w, "|---|---:|---|");
    for file in &project.files {
        let _ = writeln!(
            w,
            "| `{}` | {:.1} | {} |",
            file_label(file),
            file.total,
            file.grade
        );
    }

    if include_components {
        let _ = writeln!(w, "\n## Component Breakdown\n");
        let _ = writeln!(
            w,
            "| File | Structural | Semantic | Duplication | Coupling | Documentation | Consistency |"
        );
        let _ = writeln!(w, "|---|---:|---:|---:|---:|---:|---:|");
        for file in &project.files {
            let _ = writeln!(
                w,
                "| `{}` | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} |",
                file_label(file),
                file.structural_complexity,
                file.semantic_complexity,
                file.duplication_ratio,
                file.coupling_score,
                file.doc_coverage,
                file.consistency_score
            );
        }
    }

    out
}

/// The per-file component breakdown for the box-drawing (`-f table`) renderer.
///
/// `--include-components` had no reader at all on `analyze tdg`
/// (`NewTdgConfig.include_components` was dead), so passing it produced
/// byte-identical output while the top-level `pmat tdg` honoured the same flag.
fn components_text(project: &crate::tdg::ProjectScore) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    let _ = writeln!(
        out,
        "\nComponent Breakdown (--include-components; points earned per metric):"
    );
    if project.files.is_empty() {
        let _ = writeln!(out, "  (no files reported)");
        return out;
    }
    for file in &project.files {
        let _ = writeln!(
            out,
            "  {:<48} structural {:>5.1}  semantic {:>5.1}  duplication {:>5.1}  \
             coupling {:>5.1}  documentation {:>5.1}  consistency {:>5.1}",
            file_label(file),
            file.structural_complexity,
            file.semantic_complexity,
            file.duplication_ratio,
            file.coupling_score,
            file.doc_coverage,
            file.consistency_score
        );
    }
    out
}

fn file_label(file: &crate::tdg::TdgScore) -> String {
    file.file_path
        .as_ref()
        .map_or_else(|| "unknown".to_string(), |p| p.display().to_string())
}

fn format_file_result(score: &crate::tdg::TdgScore, format: &TdgOutputFormat) -> Result<String> {
    let result = match format {
        TdgOutputFormat::Table => format_human(score),
        TdgOutputFormat::Json => format_json(score),
        TdgOutputFormat::Markdown => format_markdown(score),
        TdgOutputFormat::Sarif => {
            let sarif = create_file_sarif_output(score);
            serde_json::to_string_pretty(&sarif)?
        }
    };
    Ok(result)
}

async fn write_or_print_result(result: &str, output_path: Option<PathBuf>) -> Result<()> {
    if let Some(output_path) = output_path {
        tokio::fs::write(&output_path, result).await?;
        eprintln!("📝 Results written to {}", output_path.display());
    } else {
        println!("{result}");
    }
    Ok(())
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_tdg_compare(
    path1: PathBuf,
    path2: PathBuf,
    format: TdgOutputFormat,
    output: Option<PathBuf>,
) -> Result<()> {
    eprintln!("🔍 Starting TDG comparison...");

    let analyzer = TdgAnalyzer::new()?;
    let comparison = analyzer.compare(&path1, &path2).await?;
    let result = format_comparison_result(&comparison, &format)?;

    write_or_print_result(&result, output).await?;
    eprintln!("✅ TDG comparison complete");
    Ok(())
}

fn format_comparison_result(
    comparison: &crate::tdg::Comparison,
    format: &TdgOutputFormat,
) -> Result<String> {
    let result = match format {
        TdgOutputFormat::Table => format_comparison(comparison),
        TdgOutputFormat::Json => serde_json::to_string_pretty(comparison)?,
        TdgOutputFormat::Markdown => {
            let mut md = format_comparison(comparison);
            md.insert_str(0, "# TDG Comparison Report\n\n");
            md
        }
        TdgOutputFormat::Sarif => {
            anyhow::bail!("SARIF format is not supported for comparisons")
        }
    };
    Ok(result)
}

/// SARIF severity band for a TDG score.
///
/// One band table for the project summary and for every file, so a single
/// document can never grade the same number two different ways. `"none"` is a
/// legal SARIF level and is what a passing measurement should carry.
fn sarif_level(total: f32) -> &'static str {
    if total < 50.0 {
        "error"
    } else if total < 65.0 {
        "warning"
    } else if total < 75.0 {
        "note"
    } else {
        "none"
    }
}

fn sarif_location(uri: String) -> serde_json::Value {
    serde_json::json!([{
        "physicalLocation": {
            "artifactLocation": { "uri": uri }
        }
    }])
}

fn sarif_score_properties(score: &crate::tdg::TdgScore) -> serde_json::Value {
    serde_json::json!({
        "tdg_score": score.total,
        "grade": score.grade.to_string(),
        "language": score.language.to_string(),
        "confidence": score.confidence,
        "structural_complexity": score.structural_complexity,
        "semantic_complexity": score.semantic_complexity,
        "duplication_ratio": score.duplication_ratio,
        "coupling_score": score.coupling_score,
        "doc_coverage": score.doc_coverage,
        "consistency_score": score.consistency_score,
        "has_contract_coverage": score.has_contract_coverage,
    })
}

/// One SARIF result per analyzed file — emitted for every file, not only for
/// files under a threshold.
///
/// Issue #669, second round: the filter used to be `total < 75.0`, so an empty
/// directory, a 3-function fixture (100.0/A+) and a 40-branch "awful" fixture
/// (85.0/A-) all produced the SAME 1065-byte document with `results: []`. A
/// document that does not change when the input changes is not a measurement.
fn sarif_file_result(score: &crate::tdg::TdgScore) -> serde_json::Value {
    let issues = score
        .penalties_applied
        .iter()
        .map(|p| p.issue.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let issues = if issues.is_empty() {
        "none recorded".to_string()
    } else {
        issues
    };

    serde_json::json!({
        "ruleId": "TDG001",
        "level": sarif_level(score.total),
        "message": {
            "text": format!(
                "File TDG score {:.1}/100 ({}). Issues: {}",
                score.total, score.grade, issues
            )
        },
        "locations": sarif_location(
            score
                .file_path
                .as_ref()
                .map_or_else(|| "unknown".to_string(), |p| p.display().to_string()),
        ),
        "properties": sarif_score_properties(score),
    })
}

/// The project-level result: the ONE number a SARIF consumer should read as
/// "the" score, and the same number `--format json/markdown/table` print.
///
/// Issue #669, second round: SARIF used to state no project score at all. Its
/// only "TDG Score:" line came from a per-file finding, so on a tree whose
/// project score was 94.15/A- the SARIF document announced 72.5/100 (B-) — the
/// worst file — and disagreed with every other renderer of the same command.
fn sarif_project_result(project: &crate::tdg::ProjectScore, root: &Path) -> serde_json::Value {
    // GH #704: with nothing analysed there is no score to quote at all — the
    // message used to quote the 0.0/F default while explaining that it was not
    // based on any measured file.
    let text = match (project.average_score, project.average_grade) {
        (Some(score), Some(grade)) => format!(
            "Project TDG score {score:.1}/100 ({grade}) over {} file(s).",
            project.total_files
        ),
        _ => format!(
            "No analyzable files were found under {}; there is no project TDG score.",
            root.display()
        ),
    };

    serde_json::json!({
        "ruleId": "TDG000",
        "level": match project.average_score {
            Some(score) if project.total_files > 0 => sarif_level(score),
            _ => "error",
        },
        "message": { "text": text },
        "locations": sarif_location(root.display().to_string()),
        "properties": {
            "tdg_score": project.average_score,
            "grade": project.average_grade.map(|g| g.to_string()),
            "not_measured": project.not_measured,
            "total_files": project.total_files,
            "f_grade_count": project.f_grade_count,
            "grade_capped": project.grade_capped,
        },
    })
}

fn sarif_rules() -> serde_json::Value {
    serde_json::json!([
        {
            "id": "TDG000",
            "name": "ProjectTechnicalDebtGrading",
            "shortDescription": { "text": "Project-level Technical Debt Grading (TDG) score" },
            "fullDescription": { "text": "The aggregate TDG score for the analyzed path. This is the same number reported by --format json, markdown and table." },
            "help": { "text": "Raise the project score by improving the lowest-scoring files reported under TDG001." }
        },
        {
            "id": "TDG001",
            "name": "TechnicalDebtGrading",
            "shortDescription": { "text": "Technical Debt Grading (TDG) quality assessment" },
            "fullDescription": { "text": "Comprehensive code quality assessment using orthogonal metrics: structural complexity, semantic complexity, code duplication, coupling, documentation, and consistency." },
            "help": { "text": "Review the specific issues identified in the TDG analysis and consider refactoring to improve code quality." }
        }
    ])
}

fn sarif_document(
    results: Vec<serde_json::Value>,
    properties: serde_json::Value,
) -> serde_json::Value {
    serde_json::json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "pmat-tdg",
                    "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit",
                    "version": env!("CARGO_PKG_VERSION"),
                    "rules": sarif_rules(),
                }
            },
            // Supplied by the caller: SARIF has nowhere else to say that the
            // result set covers only the worst --top-files entries, and a
            // capped list must never read as the whole project.
            "properties": properties,
            "results": results
        }]
    })
}

/// Build a SARIF 2.1.0 document from a whole-project TDG score.
///
/// `pub(crate)` since issue #669: the top-level `pmat tdg --format sarif`
/// command had no SARIF emitter of its own and printed a bare score, so it
/// now reuses this one instead of growing a second implementation.
pub(crate) fn create_sarif_output(
    project: &crate::tdg::ProjectScore,
    root: &Path,
) -> serde_json::Value {
    // Ordered by path, not by whatever order the directory walk produced:
    // identical input must produce a byte-identical document.
    let mut files: Vec<&crate::tdg::TdgScore> = project.files.iter().collect();
    files.sort_by(|a, b| a.file_path.cmp(&b.file_path));

    let mut results = vec![sarif_project_result(project, root)];
    results.extend(files.into_iter().map(sarif_file_result));

    sarif_document(
        results,
        serde_json::json!({
            "total_files": project.total_files,
            // GH #704: null (and named in `not_measured`) when nothing was
            // analysed, never the 0.0/"F" default.
            "average_score": project.average_score,
            "average_grade": project.average_grade.map(|g| g.to_string()),
            "not_measured": project.not_measured,
            "f_grade_count": project.f_grade_count,
            "grade_capped": project.grade_capped,
        }),
    )
}

/// Build a SARIF 2.1.0 document from a single-file TDG score.
///
/// `pub(crate)` since issue #669 — see `create_sarif_output`.
pub(crate) fn create_file_sarif_output(score: &crate::tdg::TdgScore) -> serde_json::Value {
    // Issue #669, second round: this used to emit `results: []` for any file
    // scoring >= 75, so a clean file and an unreadable one produced the same
    // document. Every analyzed file now produces exactly one result whose
    // `level` carries the verdict.
    sarif_document(
        vec![sarif_file_result(score)],
        serde_json::json!({
            "total_files": 1,
            "average_score": score.total,
            "average_grade": score.grade.to_string(),
        }),
    )
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_handle_analyze_tdg_file() -> Result<()> {
        let mut temp_file = NamedTempFile::with_suffix(".rs")?;
        writeln!(
            temp_file,
            r#"
            /// A well-documented function
            pub fn simple_function() -> i32 {{
                42
            }}
            "#
        )?;

        let config = TdgAnalysisConfig {
            path: temp_file.path().to_path_buf(),
            threshold: Some(0.0),
            top_files: Some(10),
            format: TdgOutputFormat::Json,
            include_components: false,
            output: None,
            critical_only: false,
            verbose: false,
        };
        let result = handle_analyze_tdg(config).await;

        assert!(result.is_ok());
        Ok(())
    }

    // ── --threshold is a gate, not a discarded binding ──────────────────────

    #[test]
    fn threshold_gate_fails_when_the_measured_score_is_below_it() {
        let err = enforce_tdg_threshold(Some(85.0), 90.0)
            .expect_err("85.0/100 must not satisfy a required minimum of 90.0");
        let msg = err.to_string();
        assert!(
            msg.contains("85.0"),
            "gate must state what it measured: {msg}"
        );
        assert!(
            msg.contains("90.0"),
            "gate must state what it required: {msg}"
        );
    }

    #[test]
    fn threshold_gate_passes_when_the_measured_score_meets_it() {
        assert!(enforce_tdg_threshold(Some(85.0), 2.0).is_ok());
        assert!(enforce_tdg_threshold(Some(85.0), 85.0).is_ok());
    }

    /// GH #704: a gate with nothing to measure must FAIL, not pass on the 0.0
    /// an unmeasured aggregate used to supply.
    #[test]
    fn threshold_gate_fails_when_nothing_was_measured() {
        let err = enforce_tdg_threshold(None, 2.0)
            .expect_err("a gate that measured nothing must not report a pass");
        let msg = err.to_string();
        assert!(
            msg.contains("no file") && msg.contains("no score"),
            "gate must say it measured nothing: {msg}"
        );
    }

    #[tokio::test]
    async fn build_tdg_gate_rejects_a_project_below_the_threshold() -> Result<()> {
        // The whole point of the defect: every threshold used to exit 0.
        let dir = tempfile::tempdir()?;
        std::fs::write(
            dir.path().join("lib.rs"),
            "pub fn a() -> i32 { 1 }\npub fn b() -> i32 { 2 }\n",
        )?;

        let cfg = |threshold: f64| TdgAnalysisConfig {
            path: dir.path().to_path_buf(),
            threshold: Some(threshold),
            top_files: Some(10),
            format: TdgOutputFormat::Json,
            include_components: false,
            output: Some(dir.path().join("out.json")),
            critical_only: false,
            verbose: false,
        };

        // A minimum nothing can reach must fail the build...
        assert!(
            handle_analyze_tdg_gated(cfg(1000.0)).await.is_err(),
            "--threshold 1000 must fail: no project scores above 100/100"
        );
        // ...while the ungated `analyze tdg` path stays a report.
        assert!(handle_analyze_tdg(cfg(1000.0)).await.is_ok());
        Ok(())
    }

    // ── critical-defect auto-fail no longer contradicts the report (#705) ───

    /// Build a project whose ONLY `.unwrap()` calls live in test-module files
    /// that the detector's own exclusion list does not recognise.
    fn project_with_unwraps_only_in_test_files() -> Result<tempfile::TempDir> {
        let dir = tempfile::tempdir()?;
        let src = dir.path().join("src");
        std::fs::create_dir_all(&src)?;
        std::fs::write(
            src.join("lib.rs"),
            "/// Documented.\npub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n",
        )?;
        // Bare `tests.rs` — `should_exclude_file` only knows `*_test.rs` /
        // `*_tests.rs` / `test_*`, so this scanned as production code.
        std::fs::write(
            src.join("tests.rs"),
            "use super::*;\nfn helper() -> String {\n    std::env::var(\"HOME\").unwrap()\n}\n",
        )?;
        // `_tests_` infix — same blind spot, and this is the exact shape of
        // `src/services/lang_analyzer_tests_part4.rs` from the issue.
        std::fs::write(
            src.join("lang_analyzer_tests_part4.rs"),
            "fn helper() {\n    let v: Option<i32> = Some(1);\n    let _ = v.unwrap();\n}\n",
        )?;
        Ok(dir)
    }

    fn tdg_config(path: &Path, out: PathBuf) -> TdgAnalysisConfig {
        TdgAnalysisConfig {
            path: path.to_path_buf(),
            threshold: None,
            top_files: Some(10),
            format: TdgOutputFormat::Json,
            include_components: false,
            output: Some(out),
            critical_only: false,
            verbose: false,
        }
    }

    #[test]
    fn test_module_files_are_not_production_defect_sources() {
        // The two shapes the detector's own exclusion list misses.
        assert!(is_test_source(Path::new(
            "src/services/repo_score/scorers/hygiene_scorer/tests.rs"
        )));
        assert!(is_test_source(Path::new(
            "src/services/lang_analyzer_tests_part4.rs"
        )));
        assert!(is_test_source(Path::new("tests/integration.rs")));
        assert!(is_test_source(Path::new("src/foo_tests.rs")));
        // ...and production code whose name merely contains the substring is
        // still scanned: an over-broad skip would hide real defects.
        assert!(!is_test_source(Path::new("src/cli/handlers/latest.rs")));
        assert!(!is_test_source(Path::new("src/services/contest.rs")));
        assert!(!is_test_source(Path::new("src/tdg/analyzer.rs")));
    }

    #[tokio::test]
    async fn analyze_tdg_does_not_exit_nonzero_for_unwraps_in_test_files() -> Result<()> {
        let dir = project_with_unwraps_only_in_test_files()?;
        let out = dir.path().join("out.json");

        // The defect: this exited 1 ("TDG auto-fail: Critical defects detected")
        // while its own stdout reported a healthy A/B-grade project.
        handle_analyze_tdg(tdg_config(dir.path(), out.clone())).await?;
        // The gate command must agree: test-file unwraps are not defects at all.
        handle_analyze_tdg_gated(tdg_config(dir.path(), out)).await?;
        Ok(())
    }

    #[tokio::test]
    async fn a_production_unwrap_fails_the_gate_but_only_the_gate() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let src = dir.path().join("src");
        std::fs::create_dir_all(&src)?;
        std::fs::write(
            src.join("lib.rs"),
            "pub fn first(v: &[i32]) -> i32 {\n    *v.first().unwrap()\n}\n",
        )?;

        // `analyze tdg` is a report: it says so and exits 0.
        handle_analyze_tdg(tdg_config(dir.path(), dir.path().join("a.json"))).await?;
        // `analyze build-tdg` is the gate: it still auto-fails.
        let err = handle_analyze_tdg_gated(tdg_config(dir.path(), dir.path().join("b.json")))
            .await
            .expect_err("build-tdg must still auto-fail on a production .unwrap()");
        assert!(
            err.to_string().contains("Critical defects"),
            "unexpected error: {err}"
        );
        Ok(())
    }

    // ── --critical-only actually filters ────────────────────────────────────

    fn graded(path: &str, total: f32, grade: crate::tdg::Grade) -> crate::tdg::TdgScore {
        crate::tdg::TdgScore {
            total,
            grade,
            file_path: Some(PathBuf::from(path)),
            ..Default::default()
        }
    }

    #[test]
    fn critical_only_keeps_nothing_when_no_file_is_critical() {
        // The reported defect: `analyze tdg --critical-only` on a tree whose
        // worst file graded A still listed all 9 files as critical.
        let mut project = crate::tdg::ProjectScore::aggregate(vec![
            graded("src/a.rs", 98.0, crate::tdg::Grade::APlus),
            graded("src/b.rs", 91.0, crate::tdg::Grade::A),
        ]);

        retain_critical_files(&mut project);

        assert!(
            project.files.is_empty(),
            "no F-grade file exists, so --critical-only must report none"
        );
        assert_eq!(project.files_reported, 0);
        assert!(project.files_truncated, "the subset must be disclosed");
        // Whole-project aggregates are never rewritten by a display filter.
        assert_eq!(project.total_files, 2);
    }

    #[test]
    fn critical_only_keeps_exactly_the_f_grade_files() {
        let mut project = crate::tdg::ProjectScore::aggregate(vec![
            graded("src/a.rs", 98.0, crate::tdg::Grade::APlus),
            graded("src/bad.rs", 12.0, crate::tdg::Grade::F),
            graded("src/b.rs", 91.0, crate::tdg::Grade::A),
        ]);

        retain_critical_files(&mut project);

        assert_eq!(project.files.len(), 1);
        assert_eq!(
            project.files[0].file_path,
            Some(PathBuf::from("src/bad.rs"))
        );
        assert_eq!(project.files_reported, 1);
        assert_eq!(project.total_files, 3);
    }

    // ── -f markdown is markdown, and --include-components is read ───────────

    fn two_file_project() -> crate::tdg::ProjectScore {
        let mut a = graded("src/a.rs", 98.0, crate::tdg::Grade::APlus);
        a.structural_complexity = 24.0;
        a.semantic_complexity = 19.5;
        a.duplication_ratio = 20.0;
        a.coupling_score = 14.5;
        a.doc_coverage = 9.0;
        a.consistency_score = 10.0;
        crate::tdg::ProjectScore::aggregate(vec![a, graded("src/b.rs", 91.0, crate::tdg::Grade::A)])
    }

    /// The reported defect: `-f markdown` dispatched to `format_project`, the
    /// box-drawing renderer, so `md5sum` of the `-f table` and `-f markdown`
    /// outputs matched byte for byte.
    #[test]
    fn markdown_is_not_the_box_drawing_table() {
        let project = two_file_project();
        let root = Path::new("/tmp/x");

        let table = format_project_result(&project, root, &TdgOutputFormat::Table, false).unwrap();
        let markdown =
            format_project_result(&project, root, &TdgOutputFormat::Markdown, false).unwrap();

        assert_ne!(
            table, markdown,
            "-f markdown must not re-emit the -f table rendering"
        );
        assert!(
            !markdown.contains('\u{2500}') && !markdown.contains('\u{2502}'),
            "markdown must not contain box-drawing characters: {markdown}"
        );
        assert!(markdown.starts_with("# Project TDG Score Report"));
        assert!(
            markdown.contains("| File | Score | Grade |"),
            "markdown must carry a real pipe table: {markdown}"
        );
        assert!(markdown.contains("`src/a.rs`"));
    }

    /// The reported defect: `NewTdgConfig.include_components` had no reader, so
    /// `--include-components` produced byte-identical output.
    #[test]
    fn include_components_changes_the_human_renderings() {
        let project = two_file_project();
        let root = Path::new("/tmp/x");

        for format in [TdgOutputFormat::Table, TdgOutputFormat::Markdown] {
            let without = format_project_result(&project, root, &format, false).unwrap();
            let with = format_project_result(&project, root, &format, true).unwrap();
            assert_ne!(
                without, with,
                "--include-components must change -f {format:?} output"
            );
            assert!(
                !without.contains("24.0"),
                "components must be absent without the flag: {without}"
            );
            assert!(
                with.contains("24.0") && with.contains("19.5"),
                "components must be present with the flag: {with}"
            );
        }
    }
}
