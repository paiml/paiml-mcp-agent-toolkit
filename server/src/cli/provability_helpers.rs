//! Helper functions for provability analysis to reduce complexity

use crate::services::lightweight_provability_analyzer::{FunctionId, ProofSummary};
use anyhow::Result;
use std::collections::HashMap;
use std::fmt::Write;
use std::path::Path;

/// Parse function specification string into FunctionId
pub fn parse_function_spec(spec: &str, project_path: &Path) -> Result<FunctionId> {
    // Parse function specification in format: path/to/file.rs:function_name
    // or just function_name (search all files)
    if let Some((file_part, func_part)) = spec.split_once(':') {
        Ok(FunctionId {
            file_path: project_path.join(file_part).to_string_lossy().to_string(),
            function_name: func_part.to_string(),
            line_number: 0, // Will be populated by analyzer
        })
    } else {
        // Just function name - will search all files
        Ok(FunctionId {
            file_path: String::new(),
            function_name: spec.to_string(),
            line_number: 0,
        })
    }
}

/// Discover all functions in project
pub async fn discover_project_functions(project_path: &Path) -> Result<Vec<FunctionId>> {
    eprintln!("📂 Discovering functions in project...");

    let mut function_ids = Vec::new();
    let mut file_count = 0;

    // Use file discovery service for better performance
    use crate::services::file_discovery::{FileDiscoveryConfig, ProjectFileDiscovery};

    let discovery = ProjectFileDiscovery::new(project_path.to_path_buf())
        .with_config(FileDiscoveryConfig::default());

    let files = discovery.discover_files()?;

    for path in files {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        // Only process source files
        if matches!(ext, "rs" | "ts" | "js" | "py" | "c" | "cpp" | "h" | "hpp") {
            file_count += 1;
            let relative_path = path
                .strip_prefix(project_path)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();

            // For stub implementation, add common function names
            // In real implementation, would parse AST
            function_ids.push(FunctionId {
                file_path: relative_path.clone(),
                function_name: "main".to_string(),
                line_number: 1,
            });

            function_ids.push(FunctionId {
                file_path: relative_path,
                function_name: "test".to_string(),
                line_number: 10,
            });
        }
    }

    eprintln!("📊 Found {} source files", file_count);
    Ok(function_ids)
}

/// Extract function name from a line
#[allow(dead_code)]
fn extract_function_name(line: &str) -> Option<String> {
    let line = line.trim();
    let start = line.find("fn ")? + 3;
    let end = line[start..].find(['(', '<'])?;
    Some(line[start..start + end].trim().to_string())
}

/// Filter function summaries based on confidence
pub fn filter_summaries(
    summaries: &[ProofSummary],
    high_confidence_only: bool,
) -> Vec<&ProofSummary> {
    summaries
        .iter()
        .filter(|s| !high_confidence_only || s.provability_score >= 0.8)
        .collect()
}

/// Format provability results as JSON
pub fn format_provability_json(
    function_ids: &[FunctionId],
    summaries: &[ProofSummary],
    include_evidence: bool,
) -> Result<String> {
    let results: Vec<_> = function_ids
        .iter()
        .zip(summaries.iter())
        .map(|(func_id, summary)| {
            let mut result = serde_json::json!({
                "function": {
                    "name": func_id.function_name,
                    "file": func_id.file_path,
                    "line": func_id.line_number,
                },
                "provability_score": summary.provability_score,
                "analysis_time_us": summary.analysis_time_us,
                "verified_properties": summary.verified_properties.len(),
            });

            if include_evidence {
                result["properties"] = serde_json::json!(summary
                    .verified_properties
                    .iter()
                    .map(|prop| {
                        serde_json::json!({
                            "type": format!("{:?}", prop.property_type),
                            "confidence": prop.confidence,
                            "evidence": prop.evidence,
                        })
                    })
                    .collect::<Vec<_>>());
            }

            result
        })
        .collect();

    let output = serde_json::json!({
        "provability_analysis": {
            "total_functions": function_ids.len(),
            "results": results,
        }
    });

    serde_json::to_string_pretty(&output).map_err(Into::into)
}

/// Format provability results as summary
/// 
/// # Example
/// 
/// ```
/// use pmat::cli::provability_helpers::format_provability_summary;
/// use pmat::services::lightweight_provability_analyzer::{FunctionId, ProofSummary};
/// use std::path::PathBuf;
/// 
/// let function_ids = vec![
///     FunctionId {
///         file_path: "src/main.rs".to_string(),
///         function_name: "high_score_func".to_string(),
///         line_number: 10,
///     },
///     FunctionId {
///         file_path: "src/lib.rs".to_string(),
///         function_name: "low_score_func".to_string(),
///         line_number: 20,
///     },
/// ];
/// 
/// let summaries = vec![
///     ProofSummary {
///         provability_score: 0.9,
///         analysis_time_us: 1000,
///         verified_properties: vec![],
///         version: 1,
///     },
///     ProofSummary {
///         provability_score: 0.3,
///         analysis_time_us: 500,
///         verified_properties: vec![],
///         version: 1,
///     },
/// ];
/// 
/// let output = format_provability_summary(&function_ids, &summaries, 5).unwrap();
/// 
/// assert!(output.contains("# Provability Analysis Summary"));
/// assert!(output.contains("Total functions analyzed: 2"));
/// assert!(output.contains("## Top Files by Provability"));
/// assert!(output.contains("1. `main.rs` - 90.0% avg score"));
/// ```
pub fn format_provability_summary(
    function_ids: &[FunctionId],
    summaries: &[ProofSummary],
    top_files: usize,
) -> Result<String> {
    let mut output = String::new();

    writeln!(&mut output, "# Provability Analysis Summary\n")?;
    writeln!(
        &mut output,
        "Total functions analyzed: {}",
        function_ids.len()
    )?;

    let high_provability = summaries
        .iter()
        .filter(|s| s.provability_score >= 0.8)
        .count();
    let medium_provability = summaries
        .iter()
        .filter(|s| s.provability_score >= 0.5 && s.provability_score < 0.8)
        .count();
    let low_provability = summaries
        .iter()
        .filter(|s| s.provability_score < 0.5)
        .count();

    writeln!(&mut output, "\n## Score Distribution:")?;
    writeln!(&mut output, "- High (≥80%): {} functions", high_provability)?;
    writeln!(
        &mut output,
        "- Medium (50-79%): {} functions",
        medium_provability
    )?;
    writeln!(&mut output, "- Low (<50%): {} functions", low_provability)?;

    let avg_score = if !summaries.is_empty() {
        summaries.iter().map(|s| s.provability_score).sum::<f64>() / summaries.len() as f64
    } else {
        0.0
    };

    writeln!(
        &mut output,
        "\nAverage provability score: {:.1}%",
        avg_score * 100.0
    )?;

    // Show top files by provability
    if !function_ids.is_empty() {
        writeln!(&mut output, "\n## Top Files by Provability\n")?;
        
        // Group by file and calculate average score per file
        let mut file_scores: HashMap<&str, Vec<f64>> = HashMap::new();
        for (func_id, summary) in function_ids.iter().zip(summaries.iter()) {
            file_scores.entry(&func_id.file_path).or_default().push(summary.provability_score);
        }
        
        // Calculate average scores and sort
        let mut file_avg_scores: Vec<_> = file_scores.iter()
            .map(|(file_path, scores)| {
                let avg_score = scores.iter().sum::<f64>() / scores.len() as f64;
                (file_path, avg_score, scores.len())
            })
            .collect();
        file_avg_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        let files_to_show = if top_files == 0 { 10 } else { top_files };
        for (i, (file_path, avg_score, function_count)) in file_avg_scores.iter().take(files_to_show).enumerate() {
            let filename = std::path::Path::new(file_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(file_path);
            writeln!(
                &mut output,
                "{}. `{}` - {:.1}% avg score ({} functions)",
                i + 1,
                filename,
                avg_score * 100.0,
                function_count
            )?;
        }
    }

    Ok(output)
}

/// Format provability results as detailed markdown
pub fn format_provability_detailed(
    function_ids: &[FunctionId],
    summaries: &[ProofSummary],
    include_evidence: bool,
) -> Result<String> {
    let mut output = String::new();

    writeln!(&mut output, "# Detailed Provability Analysis\n")?;

    // Group by file
    let mut by_file: HashMap<&str, Vec<(&FunctionId, &ProofSummary)>> = HashMap::new();
    for (func_id, summary) in function_ids.iter().zip(summaries.iter()) {
        by_file
            .entry(&func_id.file_path)
            .or_default()
            .push((func_id, summary));
    }

    for (file_path, functions) in by_file {
        writeln!(&mut output, "## {}\n", file_path)?;

        for (func_id, summary) in functions {
            writeln!(&mut output, "### Function: `{}`", func_id.function_name)?;
            writeln!(&mut output, "- **Line**: {}", func_id.line_number)?;
            writeln!(
                &mut output,
                "- **Provability Score**: {:.1}%",
                summary.provability_score * 100.0
            )?;
            writeln!(
                &mut output,
                "- **Analysis Time**: {}μs",
                summary.analysis_time_us
            )?;
            writeln!(
                &mut output,
                "- **Verified Properties**: {}",
                summary.verified_properties.len()
            )?;

            if include_evidence && !summary.verified_properties.is_empty() {
                writeln!(&mut output, "\n#### Verified Properties:")?;
                for prop in &summary.verified_properties {
                    writeln!(
                        &mut output,
                        "- **{:?}** (confidence: {:.0}%)",
                        prop.property_type,
                        prop.confidence * 100.0
                    )?;
                    writeln!(&mut output, "  - Evidence: {}", prop.evidence)?;
                }
            }

            writeln!(&mut output)?;
        }
    }

    Ok(output)
}

/// Format provability results as SARIF
pub fn format_provability_sarif(
    function_ids: &[FunctionId],
    summaries: &[ProofSummary],
) -> Result<String> {
    let mut results = Vec::new();

    for (func_id, summary) in function_ids.iter().zip(summaries.iter()) {
        let rule_id = if summary.provability_score < 0.5 {
            "low-provability"
        } else if summary.provability_score < 0.8 {
            "medium-provability"
        } else {
            "high-provability"
        };

        let level = if summary.provability_score < 0.5 {
            "warning"
        } else if summary.provability_score < 0.8 {
            "note"
        } else {
            "none"
        };

        results.push(serde_json::json!({
            "ruleId": rule_id,
            "level": level,
            "message": {
                "text": format!(
                    "Function '{}' has {:.0}% provability score",
                    func_id.function_name,
                    summary.provability_score * 100.0
                )
            },
            "locations": [{
                "physicalLocation": {
                    "artifactLocation": {
                        "uri": func_id.file_path
                    },
                    "region": {
                        "startLine": func_id.line_number
                    }
                }
            }]
        }));
    }

    let sarif = serde_json::json!({
        "version": "2.1.0",
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "paiml-provability-analyzer",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit",
                    "rules": generate_provability_rules(),
                }
            },
            "results": results
        }]
    });

    serde_json::to_string_pretty(&sarif).map_err(Into::into)
}

/// Generate SARIF rules for provability analysis
fn generate_provability_rules() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({
            "id": "low-provability",
            "name": "Low Provability Score",
            "shortDescription": {
                "text": "Function has low formal verification confidence"
            },
            "fullDescription": {
                "text": "Functions with low provability scores may contain complex control flow or lack sufficient type information for verification"
            },
            "defaultConfiguration": {
                "level": "warning"
            }
        }),
        serde_json::json!({
            "id": "medium-provability",
            "name": "Medium Provability Score",
            "shortDescription": {
                "text": "Function has medium formal verification confidence"
            },
            "fullDescription": {
                "text": "Functions with medium provability scores can be partially verified but may have some unverifiable properties"
            },
            "defaultConfiguration": {
                "level": "note"
            }
        }),
        serde_json::json!({
            "id": "high-provability",
            "name": "High Provability Score",
            "shortDescription": {
                "text": "Function has high formal verification confidence"
            },
            "fullDescription": {
                "text": "Functions with high provability scores can be formally verified with high confidence"
            },
            "defaultConfiguration": {
                "level": "none"
            }
        }),
    ]
}
