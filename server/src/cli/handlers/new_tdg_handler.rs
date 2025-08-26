use anyhow::Result;
use std::path::PathBuf;

use crate::cli::enums::TdgOutputFormat;
use crate::tdg::formatters::{
    format_comparison, format_human, format_json, format_markdown, format_project,
};
use crate::tdg::TdgAnalyzer;

pub async fn handle_analyze_tdg(
    path: PathBuf,
    threshold: Option<f64>,
    top_files: Option<usize>,
    format: TdgOutputFormat,
    _include_components: bool,
    output: Option<PathBuf>,
    _critical_only: bool,
    _verbose: bool,
) -> Result<()> {
    eprintln!("🔍 Starting TDG (Technical Debt Grading) analysis...");

    let analyzer = TdgAnalyzer::new()?;
    let _threshold = threshold.unwrap_or(1.5);
    let _top_files = top_files.unwrap_or(10);

    let result = if path.is_dir() {
        let project_score = analyzer.analyze_project(&path)?;

        match format {
            TdgOutputFormat::Table => format_project(&project_score),
            TdgOutputFormat::Json => serde_json::to_string_pretty(&project_score)?,
            TdgOutputFormat::Markdown => format_project(&project_score),
            TdgOutputFormat::Sarif => {
                let sarif = create_sarif_output(&project_score);
                serde_json::to_string_pretty(&sarif)?
            }
        }
    } else {
        let score = analyzer.analyze_file(&path)?;

        match format {
            TdgOutputFormat::Table => format_human(&score),
            TdgOutputFormat::Json => format_json(&score),
            TdgOutputFormat::Markdown => format_markdown(&score),
            TdgOutputFormat::Sarif => {
                let sarif = create_file_sarif_output(&score);
                serde_json::to_string_pretty(&sarif)?
            }
        }
    };

    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &result).await?;
        eprintln!("📝 Results written to {}", output_path.display());
    } else {
        println!("{}", result);
    }

    eprintln!("✅ TDG analysis complete");
    Ok(())
}

pub async fn handle_tdg_compare(
    path1: PathBuf,
    path2: PathBuf,
    format: TdgOutputFormat,
    output: Option<PathBuf>,
) -> Result<()> {
    eprintln!("🔍 Starting TDG comparison...");

    let analyzer = TdgAnalyzer::new()?;
    let comparison = analyzer.compare(&path1, &path2)?;

    let result = match format {
        TdgOutputFormat::Table => format_comparison(&comparison),
        TdgOutputFormat::Json => serde_json::to_string_pretty(&comparison)?,
        TdgOutputFormat::Markdown => {
            let mut md = format_comparison(&comparison);
            md.insert_str(0, "# TDG Comparison Report\n\n");
            md
        }
        TdgOutputFormat::Sarif => {
            anyhow::bail!("SARIF format is not supported for comparisons")
        }
    };

    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &result).await?;
        eprintln!("📝 Results written to {}", output_path.display());
    } else {
        println!("{}", result);
    }

    eprintln!("✅ TDG comparison complete");
    Ok(())
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
                            "uri": score.file_path.as_ref()
                                .map(|p| p.display().to_string())
                                .unwrap_or_else(|| "unknown".to_string())
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
                        "uri": score.file_path.as_ref()
                            .map(|p| p.display().to_string())
                            .unwrap_or_else(|| "unknown".to_string())
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

        let result = handle_analyze_tdg(
            temp_file.path().to_path_buf(),
            Some(0.0),
            Some(10),
            TdgOutputFormat::Json,
            false,
            None,
            false,
            false,
        )
        .await;

        assert!(result.is_ok());
        Ok(())
    }
}
