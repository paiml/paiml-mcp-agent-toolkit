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
    // `-n/--top-files` was read into a discarded binding, so it truncated
    // nothing: -n 5 / -n 10 / -n 100000 all emitted the same file list.
    let top_files = config.top_files.unwrap_or(10);

    let result = if config.path.is_dir() {
        analyze_project_path(&analyzer, &config.path, &config.format, top_files).await?
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
    top_files: usize,
) -> Result<String> {
    let mut project_score = analyzer.analyze_project(path).await?;
    // Honour --top-files for every renderer; aggregates stay whole-project and
    // the truncation is disclosed (files_reported / files_truncated).
    project_score.limit_to_worst_files(top_files);
    format_project_result(&project_score, format)
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
    format: &TdgOutputFormat,
) -> Result<String> {
    let result = match format {
        TdgOutputFormat::Table => format_project(project_score),
        TdgOutputFormat::Json => serde_json::to_string_pretty(project_score)?,
        TdgOutputFormat::Markdown => format_project(project_score),
        TdgOutputFormat::Sarif => {
            let sarif = create_sarif_output(project_score);
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

fn create_sarif_output(project: &crate::tdg::ProjectScore) -> serde_json::Value {
    let results = project
        .files
        .iter()
        .filter(|score| score.total < 75.0)
        .map(|score| {
            let level = if score.total < 50.0 {
                "error"
            } else if score.total < 65.0 {
                "warning"
            } else {
                "note"
            };

            let message = format!(
                "TDG Score: {:.1}/100 ({}). Issues: {}",
                score.total,
                score.grade,
                score
                    .penalties_applied
                    .iter()
                    .map(|p| p.issue.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );

            serde_json::json!({
                "ruleId": "TDG001",
                "level": level,
                "message": {
                    "text": message
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": score.file_path.as_ref().map_or_else(|| "unknown".to_string(), |p| p.display().to_string())
                        }
                    }
                }],
                "properties": {
                    "tdg_score": score.total,
                    "grade": score.grade.to_string(),
                    "language": score.language.to_string(),
                    "confidence": score.confidence,
                    "structural_complexity": score.structural_complexity,
                    "semantic_complexity": score.semantic_complexity,
                    "duplication_ratio": score.duplication_ratio,
                    "coupling_score": score.coupling_score,
                    "doc_coverage": score.doc_coverage,
                    "consistency_score": score.consistency_score
                }
            })
        })
        .collect::<Vec<_>>();

    serde_json::json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "pmat-tdg",
                    "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit",
                    "version": env!("CARGO_PKG_VERSION"),
                    "rules": [{
                        "id": "TDG001",
                        "name": "TechnicalDebtGrading",
                        "shortDescription": {
                            "text": "Technical Debt Grading (TDG) quality assessment"
                        },
                        "fullDescription": {
                            "text": "Comprehensive code quality assessment using orthogonal metrics: structural complexity, semantic complexity, code duplication, coupling, documentation, and consistency."
                        },
                        "help": {
                            "text": "Review the specific issues identified in the TDG analysis and consider refactoring to improve code quality."
                        }
                    }]
                }
            },
            // Name both numbers: SARIF has nowhere else to say that the result
            // set covers only the worst --top-files entries, and a capped list
            // must never read as the whole project.
            "properties": {
                "filesAnalyzed": project.total_files,
                "filesReported": project.files_reported,
                "filesTruncated": project.files_truncated
            },
            "results": results
        }]
    })
}

fn create_file_sarif_output(score: &crate::tdg::TdgScore) -> serde_json::Value {
    let level = if score.total < 50.0 {
        "error"
    } else if score.total < 65.0 {
        "warning"
    } else {
        "note"
    };

    let message = format!(
        "TDG Score: {:.1}/100 ({}). Issues: {}",
        score.total,
        score.grade,
        score
            .penalties_applied
            .iter()
            .map(|p| p.issue.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );

    let results = if score.total < 75.0 {
        vec![serde_json::json!({
            "ruleId": "TDG001",
            "level": level,
            "message": {
                "text": message
            },
            "locations": [{
                "physicalLocation": {
                    "artifactLocation": {
                        "uri": score.file_path.as_ref().map_or_else(|| "unknown".to_string(), |p| p.display().to_string())
                    }
                }
            }],
            "properties": {
                "tdg_score": score.total,
                "grade": score.grade.to_string(),
                "language": score.language.to_string(),
                "confidence": score.confidence,
                "structural_complexity": score.structural_complexity,
                "semantic_complexity": score.semantic_complexity,
                "duplication_ratio": score.duplication_ratio,
                "coupling_score": score.coupling_score,
                "doc_coverage": score.doc_coverage,
                "consistency_score": score.consistency_score
            }
        })]
    } else {
        vec![]
    };

    serde_json::json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "pmat-tdg",
                    "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit",
                    "version": env!("CARGO_PKG_VERSION"),
                    "rules": [{
                        "id": "TDG001",
                        "name": "TechnicalDebtGrading",
                        "shortDescription": {
                            "text": "Technical Debt Grading (TDG) quality assessment"
                        },
                        "fullDescription": {
                            "text": "Comprehensive code quality assessment using orthogonal metrics: structural complexity, semantic complexity, code duplication, coupling, documentation, and consistency."
                        },
                        "help": {
                            "text": "Review the specific issues identified in the TDG analysis and consider refactoring to improve code quality."
                        }
                    }]
                }
            },
            "results": results
        }]
    })
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
