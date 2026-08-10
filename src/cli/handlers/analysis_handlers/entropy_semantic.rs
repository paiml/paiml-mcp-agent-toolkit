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
        // GH-681: a nonexistent path (via `-p` or `--file`) exited 0 reporting
        // "Files Analyzed: 0 / Total Violations: 1" — a Medium-severity quality
        // finding about code on a path that does not exist.
        crate::cli::ensure_analysis_path_exists(&analysis_path)?;
        let report = analyzer.analyze(&analysis_path).await?;

        let output_content = format_entropy_report(&report, format, top_violations)?;

        output_entropy_results(output, &output_content)?;

        Ok(())
    } else {
        unreachable!("Expected Entropy command")
    }
}

/// Create entropy configuration from CLI parameters
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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

    // The exclusion list is shared with `quality-gate --checks entropy` and the
    // MCP entropy tool. It used to be assembled here from a different set of
    // globs than the gate used, so on the same `-p` the two commands analyzed
    // 939 and 1328 files and reported 62.6% / 14 violations against
    // 63.9% / 16 for the same metric on the same tree.
    EntropyConfig {
        min_severity: min_sev,
        exclude_paths: EntropyConfig::analysis_excludes(include_tests),
        ..Default::default()
    }
}

/// Format entropy report based on output format
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
///
/// Reports the same measured values the JSON renderer emits — including
/// "not measured" where the JSON carries `null` — so the two renderers of this
/// command cannot disagree about a number.
fn format_summary_report(report: &crate::entropy::EntropyReport, top_violations: usize) -> String {
    use crate::cli::colors as c;

    let violations = get_top_violations(&report.actionable_violations, top_violations);
    let note = report
        .measurement_note
        .as_ref()
        .map_or_else(String::new, |n| format!("Note: {n}\n"));

    format!(
        "{}{}Entropy Analysis Summary{}\n\n\
         {}Files Analyzed:{} {}{}{}\n\
         {}Source Lines Analyzed:{} {}{}{}\n\
         {}Pattern Diversity:{} {}{}{}\n\
         {}Total Violations:{} {}{}{}\n\
         {}Potential LOC Reduction:{} {}{}{} lines ({}{:.1}%{})\n\
         {}\n\
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
        report.entropy_metrics.total_loc,
        c::RESET,
        c::BOLD,
        c::RESET,
        c::BOLD_WHITE,
        crate::entropy::EntropyReport::render_measurement(report.entropy_metrics.pattern_diversity),
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
        note,
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

    let note = report
        .measurement_note
        .as_ref()
        .map_or_else(String::new, |n| format!("\n> {n}\n"));

    format!(
        "# Entropy Analysis Report\n\n\
         ## Summary\n\n\
         - **Files Analyzed**: {}\n\
         - **Source Lines Analyzed**: {}\n\
         - **Pattern Diversity**: {}\n\
         - **Total Violations**: {}\n\
         - **Potential LOC Reduction**: {} lines ({:.1}%)\n{}\n\
         ## Violations\n\n{}\n",
        report.total_files_analyzed,
        report.entropy_metrics.total_loc,
        crate::entropy::EntropyReport::render_measurement(report.entropy_metrics.pattern_diversity),
        report.actionable_violations.len(),
        report.total_loc_reduction(),
        report.reduction_percentage(),
        note,
        format_markdown_violations(&report.actionable_violations, max_violations)
    )
}

/// Get top N violations from list
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
            // "saves" renders "not estimated" where no per-pattern size was
            // measured (the low-diversity finding), instead of the old fixed
            // 15%-of-total_loc figure printed as if it were derived.
            format!(
                "  {}. {}{:?}{} {} (saves {})\n     {}Fix:{} {}",
                c::number(&(i + 1).to_string()),
                sev_color,
                v.severity,
                c::RESET,
                v.message,
                c::number(&v.render_loc_reduction()),
                c::BOLD,
                c::RESET,
                v.fix_suggestion
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Format violations for markdown output
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn format_markdown_violations(
    violations: &[crate::entropy::violation_detector::ActionableViolation],
    max_count: usize,
) -> String {
    violations
        .iter()
        .take(max_count)
        .map(|v| {
            // A project-level finding (low diversity) has no pattern; it used to
            // be rendered with a placeholder "ControlFlow (repeated 0 times)".
            let pattern_line = v.pattern.as_ref().map_or_else(
                || "**Pattern**: project-wide (not a single construct)\n".to_string(),
                |p| {
                    format!(
                        "**Pattern**: {:?} (repeated {} times)\n",
                        p.pattern_type, p.repetitions
                    )
                },
            );
            format!(
                "### {} ({:?})\n\n\
                 {}\
                 **Fix**: {}\n\
                 **LOC Reduction**: {}\n\
                 **Affected Files**: {}\n",
                v.message,
                v.severity,
                pattern_line,
                v.fix_suggestion,
                v.render_loc_reduction(),
                v.affected_files.len()
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Output entropy results to file or stdout
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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
    // Progress chatter goes to stderr: with `--format json` these banners used
    // to precede the JSON document on stdout, so piping the output into a JSON
    // parser failed on the very first character.
    eprintln!("\u{1f50d} Indexing source files...");
    let num_docs = engine
        .index_directory(workspace, language)
        .map_err(|e| anyhow::anyhow!("Failed to index directory: {}", e))?;
    if num_docs == 0 {
        anyhow::bail!("No source files found to analyze");
    }
    eprintln!("\u{1f4c1} Indexed {} source files", num_docs);
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
            // stderr, so `--format json` stdout stays a single JSON document.
            eprintln!("\u{1f9ee} Running {} clustering...", method_str);
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
            // stderr, so `--format json` stdout stays a single JSON document.
            eprintln!("\u{1f52c} Extracting {} topics using LDA...", num_topics);
            let result = engine
                .extract_topics(num_topics, language)
                .map_err(|e| anyhow::anyhow!("Topic extraction failed: {}", e))?;
            output_topic_results(&result, &format)
        }
        _ => unreachable!("Expected semantic analysis command"),
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod stdout_purity_tests {
    /// `analyze cluster --format json` used to emit its progress banners with
    /// println!, so stdout began with "🔍 Indexing source files..." and a JSON
    /// consumer failed at character 0. Pin every banner in this module to
    /// stderr — there is no way to observe the real process stdout from a unit
    /// test, so the check is made against the module source itself.
    #[test]
    fn test_progress_banners_never_go_to_stdout() {
        let src = include_str!("entropy_semantic.rs");
        let banners = [
            "Indexing source files",
            "Indexed {} source files",
            "clustering...",
            "topics using LDA",
        ];
        for line in src.lines() {
            let trimmed = line.trim_start();
            if !trimmed.starts_with("println!") {
                continue;
            }
            for banner in banners {
                assert!(
                    !trimmed.contains(banner),
                    "progress banner must be written to stderr: {trimmed}"
                );
            }
        }
    }
}
