//! Entropy analysis and semantic analysis route handlers
//!
//! Handles: Entropy analysis, Cluster, Topics (semantic analysis)

use crate::cli::AnalyzeCommands;
use anyhow::Result;
use std::path::Path;

/// Route entropy analysis command
///
/// Refactored to reduce complexity from 25 to <20 by extracting helper functions
pub(super) async fn route_entropy_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::Entropy {
        path,
        project_path,
        format,
        output,
        min_severity,
        top_violations,
        file,
        include_tests,
    } = cmd
    {
        use crate::entropy::EntropyAnalyzer;

        let path = project_path.unwrap_or(path);
        let config = create_entropy_config(min_severity, include_tests);
        let analyzer = EntropyAnalyzer::with_config(config);

        let analysis_path = file.unwrap_or(path);
        let report = analyzer.analyze(&analysis_path).await?;

        let output_content = format_entropy_report(&report, format, top_violations)?;

        output_entropy_results(output, &output_content)?;

        Ok(())
    } else {
        unreachable!("Expected Entropy command")
    }
}

/// Create entropy configuration from CLI parameters
pub(crate) fn create_entropy_config(
    min_severity: crate::cli::EntropySeverity,
    include_tests: bool,
) -> crate::entropy::EntropyConfig {
    use crate::cli::EntropySeverity;
    use crate::entropy::violation_detector::Severity;
    use crate::entropy::EntropyConfig;

    let min_sev = match min_severity {
        EntropySeverity::Low => Severity::Low,
        EntropySeverity::Medium => Severity::Medium,
        EntropySeverity::High => Severity::High,
    };

    let mut config = EntropyConfig {
        min_severity: min_sev,
        ..Default::default()
    };

    if !include_tests {
        config.exclude_paths.push("**/*test*.rs".to_string());
        config.exclude_paths.push("tests/**".to_string());
    }

    config
}

/// Format entropy report based on output format
pub(crate) fn format_entropy_report(
    report: &crate::entropy::EntropyReport,
    format: crate::cli::EntropyOutputFormat,
    top_violations: usize,
) -> Result<String> {
    use crate::cli::EntropyOutputFormat;

    match format {
        EntropyOutputFormat::Summary => Ok(format_summary_report(report, top_violations)),
        EntropyOutputFormat::Detailed => Ok(report.format_report()),
        EntropyOutputFormat::Json => Ok(serde_json::to_string_pretty(&report)?),
        EntropyOutputFormat::Markdown => Ok(format_markdown_report(report, top_violations)),
    }
}

/// Format summary report
fn format_summary_report(report: &crate::entropy::EntropyReport, top_violations: usize) -> String {
    use crate::cli::colors as c;

    let violations = get_top_violations(&report.actionable_violations, top_violations);

    format!(
        "{}{}Entropy Analysis Summary{}\n\n\
         {}Files Analyzed:{} {}{}{}\n\
         {}Total Violations:{} {}{}{}\n\
         {}Potential LOC Reduction:{} {}{}{} lines ({}{:.1}%{})\n\n\
         {}Top Violations:{}\n{}\n",
        c::BOLD,
        c::UNDERLINE,
        c::RESET,
        c::BOLD,
        c::RESET,
        c::BOLD_WHITE,
        report.total_files_analyzed,
        c::RESET,
        c::BOLD,
        c::RESET,
        c::BOLD_WHITE,
        report.actionable_violations.len(),
        c::RESET,
        c::BOLD,
        c::RESET,
        c::BOLD_WHITE,
        report.total_loc_reduction(),
        c::RESET,
        c::BOLD_WHITE,
        report.reduction_percentage(),
        c::RESET,
        c::BOLD,
        c::RESET,
        format_violation_list(&violations)
    )
}

/// Format markdown report
fn format_markdown_report(report: &crate::entropy::EntropyReport, top_violations: usize) -> String {
    let max_violations = if top_violations == 0 {
        usize::MAX
    } else {
        top_violations
    };

    format!(
        "# Entropy Analysis Report\n\n\
         ## Summary\n\n\
         - **Files Analyzed**: {}\n\
         - **Total Violations**: {}\n\
         - **Potential LOC Reduction**: {} lines ({:.1}%)\n\n\
         ## Violations\n\n{}\n",
        report.total_files_analyzed,
        report.actionable_violations.len(),
        report.total_loc_reduction(),
        report.reduction_percentage(),
        format_markdown_violations(&report.actionable_violations, max_violations)
    )
}

/// Get top N violations from list
pub(crate) fn get_top_violations(
    violations: &[crate::entropy::violation_detector::ActionableViolation],
    top_n: usize,
) -> Vec<crate::entropy::violation_detector::ActionableViolation> {
    if top_n > 0 && violations.len() > top_n {
        violations.iter().take(top_n).cloned().collect()
    } else {
        violations.to_vec()
    }
}

/// Format violation list for summary
pub(crate) fn format_violation_list(
    violations: &[crate::entropy::violation_detector::ActionableViolation],
) -> String {
    use crate::cli::colors as c;
    violations
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let sev_color = match v.severity {
                crate::entropy::violation_detector::Severity::High => c::RED,
                crate::entropy::violation_detector::Severity::Medium => c::YELLOW,
                crate::entropy::violation_detector::Severity::Low => c::GREEN,
            };
            format!(
                "  {}. {}{:?}{} {} (saves {} lines)\n     {}Fix:{} {}",
                c::number(&(i + 1).to_string()),
                sev_color,
                v.severity,
                c::RESET,
                v.message,
                c::number(&v.estimated_loc_reduction.to_string()),
                c::BOLD,
                c::RESET,
                v.fix_suggestion
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Format violations for markdown output
pub(crate) fn format_markdown_violations(
    violations: &[crate::entropy::violation_detector::ActionableViolation],
    max_count: usize,
) -> String {
    violations
        .iter()
        .take(max_count)
        .map(|v| {
            format!(
                "### {} ({:?})\n\n\
                 **Pattern**: {:?} (repeated {} times)\n\
                 **Fix**: {}\n\
                 **LOC Reduction**: {} lines\n\
                 **Affected Files**: {}\n",
                v.message,
                v.severity,
                v.pattern.pattern_type,
                v.pattern.repetitions,
                v.fix_suggestion,
                v.estimated_loc_reduction,
                v.affected_files.len()
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Output entropy results to file or stdout
pub(crate) fn output_entropy_results(
    output: Option<std::path::PathBuf>,
    content: &str,
) -> Result<()> {
    use std::fs;

    if let Some(output_path) = output {
        fs::write(output_path, content)?;
    } else {
        println!("{content}");
    }

    Ok(())
}

/// Index workspace and validate document count
fn index_workspace(
    engine: &mut crate::services::local_semantic::LocalSemanticEngine,
    workspace: &Path,
    language: Option<&str>,
) -> Result<usize> {
    println!("\u{1f50d} Indexing source files...");
    let num_docs = engine
        .index_directory(workspace, language)
        .map_err(|e| anyhow::anyhow!("Failed to index directory: {}", e))?;
    if num_docs == 0 {
        anyhow::bail!("No source files found to analyze");
    }
    println!("\u{1f4c1} Indexed {} source files", num_docs);
    Ok(num_docs)
}

/// Output clustering results in the requested format
fn output_cluster_results(
    result: &crate::services::local_semantic::LocalClusterResult,
    format: &crate::cli::enums::OutputFormat,
) -> Result<()> {
    match format {
        crate::cli::enums::OutputFormat::Json => {
            let json_output = serde_json::json!({
                "method": result.method,
                "num_documents": result.num_documents,
                "num_clusters": result.clusters.len(),
                "clusters": result.clusters.iter().map(|c| serde_json::json!({
                    "id": c.id, "size": c.size,
                    "files": c.files.iter().map(|f| f.display().to_string()).collect::<Vec<_>>()
                })).collect::<Vec<_>>()
            });
            println!("{}", serde_json::to_string_pretty(&json_output)?);
        }
        _ => {
            println!("\n\u{1f4ca} Clustering Results ({}):", result.method);
            println!("   Documents: {}", result.num_documents);
            println!("   Clusters: {}\n", result.clusters.len());
            for cluster in &result.clusters {
                println!("   Cluster {} ({} files):", cluster.id, cluster.size);
                for file in cluster.files.iter().take(5) {
                    println!("     - {}", file.display());
                }
                if cluster.files.len() > 5 {
                    println!("     ... and {} more", cluster.files.len() - 5);
                }
                println!();
            }
        }
    }
    Ok(())
}

/// Output topic extraction results in the requested format
fn output_topic_results(
    result: &crate::services::local_semantic::LocalTopicResult,
    format: &crate::cli::enums::OutputFormat,
) -> Result<()> {
    match format {
        crate::cli::enums::OutputFormat::Json => {
            let json_output = serde_json::json!({
                "num_documents": result.num_documents,
                "num_topics": result.topics.len(),
                "topics": result.topics.iter().map(|t| serde_json::json!({
                    "id": t.id, "document_count": t.document_count,
                    "top_terms": t.top_terms.iter().map(|(term, weight)| {
                        serde_json::json!({"term": term, "weight": weight})
                    }).collect::<Vec<_>>()
                })).collect::<Vec<_>>()
            });
            println!("{}", serde_json::to_string_pretty(&json_output)?);
        }
        _ => {
            println!("\n\u{1f4ca} Topic Extraction Results:");
            println!("   Documents: {}", result.num_documents);
            println!("   Topics: {}\n", result.topics.len());
            for topic in &result.topics {
                println!(
                    "   Topic {} ({} documents):",
                    topic.id, topic.document_count
                );
                println!("     Top terms:");
                for (term, weight) in topic.top_terms.iter().take(10) {
                    println!("       - {} ({:.3})", term, weight);
                }
                println!();
            }
        }
    }
    Ok(())
}

/// Route semantic analysis commands (PMAT-SEARCH-011)
/// Uses local aprender-based analysis - NO external API required
pub(super) async fn route_semantic_analysis(cmd: AnalyzeCommands) -> Result<()> {
    use crate::services::local_semantic::LocalSemanticEngine;

    let workspace = std::env::current_dir().unwrap_or_default();
    let mut engine = LocalSemanticEngine::new();

    match cmd {
        AnalyzeCommands::Cluster {
            method,
            k,
            language,
            format,
        } => {
            let method_str = match method {
                crate::cli::commands::ClusterMethod::Kmeans => "kmeans",
                crate::cli::commands::ClusterMethod::Hierarchical => "hierarchical",
                crate::cli::commands::ClusterMethod::Dbscan => "dbscan",
            };
            index_workspace(&mut engine, &workspace, language.as_deref())?;
            println!("\u{1f9ee} Running {} clustering...", method_str);
            let result = engine
                .cluster(method_str, k)
                .map_err(|e| anyhow::anyhow!("Clustering failed: {}", e))?;
            output_cluster_results(&result, &format)
        }
        AnalyzeCommands::Topics {
            num_topics,
            language,
            format,
        } => {
            index_workspace(&mut engine, &workspace, language.as_deref())?;
            println!("\u{1f52c} Extracting {} topics using LDA...", num_topics);
            let result = engine
                .extract_topics(num_topics, language)
                .map_err(|e| anyhow::anyhow!("Topic extraction failed: {}", e))?;
            output_topic_results(&result, &format)
        }
        _ => unreachable!("Expected semantic analysis command"),
    }
}
