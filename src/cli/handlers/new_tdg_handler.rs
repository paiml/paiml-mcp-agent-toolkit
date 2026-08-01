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

/// Check for critical defects in the project (Known Defects v2.1)
/// Auto-fails TDG analysis if critical defects are found
async fn check_for_critical_defects(path: &Path) -> Result<()> {
    use crate::services::defect_detector::{RustDefectDetector, Severity};
    use walkdir::WalkDir;

    let detector = RustDefectDetector::new();
    let mut critical_defects_found = false;
    let mut critical_count = 0;

    eprintln!("🔍 Checking for critical defects...");

    // Scan Rust files in the project
    for entry in WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let file_path = entry.path();

        // Only process Rust files
        if !file_path.is_file() || file_path.extension() != Some(std::ffi::OsStr::new("rs")) {
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
    eprintln!("🔍 Starting TDG (Technical Debt Grading) analysis...");

    let analyzer = TdgAnalyzer::new()?;
    let _threshold = config.threshold.unwrap_or(1.5);
    let _top_files = config.top_files.unwrap_or(10);

    let result = if config.path.is_dir() {
        analyze_project_path(&analyzer, &config.path, &config.format).await?
    } else {
        analyze_single_file(&analyzer, &config.path, &config.format).await?
    };

    write_or_print_result(&result, config.output).await?;

    // KNOWN DEFECTS v2.1: Check for critical defects and auto-fail
    check_for_critical_defects(&config.path).await?;

    eprintln!("✅ TDG analysis complete");
    Ok(())
}

async fn analyze_project_path(
    analyzer: &TdgAnalyzer,
    path: &Path,
    format: &TdgOutputFormat,
) -> Result<String> {
    let project_score = analyzer.analyze_project(path).await?;
    format_project_result(&project_score, path, format)
}

async fn analyze_single_file(
    analyzer: &TdgAnalyzer,
    path: &Path,
    format: &TdgOutputFormat,
) -> Result<String> {
    let score = analyzer.analyze_file(path).await?;
    format_file_result(&score, format)
}

fn format_project_result(
    project_score: &crate::tdg::ProjectScore,
    root: &Path,
    format: &TdgOutputFormat,
) -> Result<String> {
    let result = match format {
        TdgOutputFormat::Table => format_project(project_score),
        TdgOutputFormat::Json => serde_json::to_string_pretty(project_score)?,
        TdgOutputFormat::Markdown => format_project(project_score),
        TdgOutputFormat::Sarif => {
            let sarif = create_sarif_output(project_score, root);
            serde_json::to_string_pretty(&sarif)?
        }
    };
    Ok(result)
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
    let text = if project.total_files == 0 {
        format!(
            "No analyzable files were found under {}; project TDG score {:.1}/100 ({}) is not based on any measured file.",
            root.display(),
            project.average_score,
            project.average_grade
        )
    } else {
        format!(
            "Project TDG score {:.1}/100 ({}) over {} file(s).",
            project.average_score, project.average_grade, project.total_files
        )
    };

    serde_json::json!({
        "ruleId": "TDG000",
        "level": if project.total_files == 0 { "error" } else { sarif_level(project.average_score) },
        "message": { "text": text },
        "locations": sarif_location(root.display().to_string()),
        "properties": {
            "tdg_score": project.average_score,
            "grade": project.average_grade.to_string(),
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
            "results": results,
            "properties": properties,
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
            "average_score": project.average_score,
            "average_grade": project.average_grade.to_string(),
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
}
