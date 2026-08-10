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

/// Reject a `--format` value this subcommand cannot actually produce.
///
/// `--format` accepts nine values but only `json` was ever implemented: yaml,
/// csv, markdown, junit, summary, text, plain and table all fell through one
/// `_ =>` arm, so eight of the nine advertised formats emitted byte-identical
/// human text. The formats below are now produced for real; `junit` describes
/// test cases, of which a clustering run has none, so it is refused rather than
/// silently answered with prose.
fn reject_unsupported_format(
    subcommand: &str,
    format: &crate::cli::enums::OutputFormat,
) -> Result<()> {
    if matches!(format, crate::cli::enums::OutputFormat::Junit) {
        anyhow::bail!(
            "`analyze {subcommand} --format junit` is not supported: there are no test cases to \
             report. Use json, yaml, csv, markdown or table."
        );
    }
    Ok(())
}

/// Machine-readable view of a clustering run, shared by json/yaml/csv/markdown.
fn cluster_results_json(
    result: &crate::services::local_semantic::LocalClusterResult,
) -> serde_json::Value {
    serde_json::json!({
        "method": result.method,
        "num_documents": result.num_documents,
        "num_clusters": result.clusters.len(),
        "clusters": result.clusters.iter().map(|c| serde_json::json!({
            "id": c.id, "size": c.size,
            "files": c.files.iter().map(|f| f.display().to_string()).collect::<Vec<_>>()
        })).collect::<Vec<_>>()
    })
}

/// Render clustering results in the requested format.
fn format_cluster_results(
    result: &crate::services::local_semantic::LocalClusterResult,
    format: &crate::cli::enums::OutputFormat,
) -> Result<String> {
    use crate::cli::enums::OutputFormat;
    use std::fmt::Write as _;

    reject_unsupported_format("cluster", format)?;

    let mut out = String::new();
    match format {
        OutputFormat::Json => {
            out.push_str(&serde_json::to_string_pretty(&cluster_results_json(
                result,
            ))?);
        }
        OutputFormat::Yaml => {
            out.push_str(&serde_yaml_ng::to_string(&cluster_results_json(result))?);
        }
        OutputFormat::Csv => {
            out.push_str("cluster_id,size,file\n");
            for cluster in &result.clusters {
                for file in &cluster.files {
                    let _ = writeln!(out, "{},{},{}", cluster.id, cluster.size, file.display());
                }
            }
        }
        OutputFormat::Markdown => {
            let _ = writeln!(out, "# Clustering Results ({})\n", result.method);
            let _ = writeln!(out, "- **Documents**: {}", result.num_documents);
            let _ = writeln!(out, "- **Clusters**: {}\n", result.clusters.len());
            out.push_str("| Cluster | Size | Files |\n| --- | --- | --- |\n");
            for cluster in &result.clusters {
                let files: Vec<String> = cluster
                    .files
                    .iter()
                    .map(|f| f.display().to_string())
                    .collect();
                let _ = writeln!(
                    out,
                    "| {} | {} | {} |",
                    cluster.id,
                    cluster.size,
                    files.join("<br>")
                );
            }
        }
        _ => {
            let _ = writeln!(out, "\n\u{1f4ca} Clustering Results ({}):", result.method);
            let _ = writeln!(out, "   Documents: {}", result.num_documents);
            let _ = writeln!(out, "   Clusters: {}\n", result.clusters.len());
            for cluster in &result.clusters {
                let _ = writeln!(out, "   Cluster {} ({} files):", cluster.id, cluster.size);
                for file in cluster.files.iter().take(5) {
                    let _ = writeln!(out, "     - {}", file.display());
                }
                if cluster.files.len() > 5 {
                    let _ = writeln!(out, "     ... and {} more", cluster.files.len() - 5);
                }
                out.push('\n');
            }
        }
    }
    Ok(out)
}

/// Output clustering results in the requested format
fn output_cluster_results(
    result: &crate::services::local_semantic::LocalClusterResult,
    format: &crate::cli::enums::OutputFormat,
) -> Result<()> {
    println!("{}", format_cluster_results(result, format)?);
    Ok(())
}

/// Machine-readable view of a topic run, shared by json/yaml/csv/markdown.
fn topic_results_json(
    result: &crate::services::local_semantic::LocalTopicResult,
) -> serde_json::Value {
    serde_json::json!({
        "num_documents": result.num_documents,
        "num_topics": result.topics.len(),
        "topics": result.topics.iter().map(|t| serde_json::json!({
            "id": t.id, "document_count": t.document_count,
            "top_terms": t.top_terms.iter().map(|(term, weight)| {
                serde_json::json!({"term": term, "weight": weight})
            }).collect::<Vec<_>>()
        })).collect::<Vec<_>>()
    })
}

/// Render topic extraction results in the requested format.
fn format_topic_results(
    result: &crate::services::local_semantic::LocalTopicResult,
    format: &crate::cli::enums::OutputFormat,
) -> Result<String> {
    use crate::cli::enums::OutputFormat;
    use std::fmt::Write as _;

    reject_unsupported_format("topics", format)?;

    let mut out = String::new();
    match format {
        OutputFormat::Json => {
            out.push_str(&serde_json::to_string_pretty(&topic_results_json(result))?);
        }
        OutputFormat::Yaml => {
            out.push_str(&serde_yaml_ng::to_string(&topic_results_json(result))?);
        }
        OutputFormat::Csv => {
            out.push_str("topic_id,document_count,term,weight\n");
            for topic in &result.topics {
                for (term, weight) in &topic.top_terms {
                    let _ = writeln!(
                        out,
                        "{},{},{},{:.6}",
                        topic.id, topic.document_count, term, weight
                    );
                }
            }
        }
        OutputFormat::Markdown => {
            out.push_str("# Topic Extraction Results\n\n");
            let _ = writeln!(out, "- **Documents**: {}", result.num_documents);
            let _ = writeln!(out, "- **Topics**: {}\n", result.topics.len());
            for topic in &result.topics {
                let _ = writeln!(
                    out,
                    "## Topic {} ({} documents)\n",
                    topic.id, topic.document_count
                );
                for (term, weight) in topic.top_terms.iter().take(10) {
                    let _ = writeln!(out, "- {term} ({weight:.3})");
                }
                out.push('\n');
            }
        }
        _ => {
            out.push_str("\n\u{1f4ca} Topic Extraction Results:\n");
            let _ = writeln!(out, "   Documents: {}", result.num_documents);
            let _ = writeln!(out, "   Topics: {}\n", result.topics.len());
            for topic in &result.topics {
                let _ = writeln!(
                    out,
                    "   Topic {} ({} documents):",
                    topic.id, topic.document_count
                );
                out.push_str("     Top terms:\n");
                for (term, weight) in topic.top_terms.iter().take(10) {
                    let _ = writeln!(out, "       - {} ({:.3})", term, weight);
                }
                out.push('\n');
            }
        }
    }
    Ok(out)
}

/// Output topic extraction results in the requested format
fn output_topic_results(
    result: &crate::services::local_semantic::LocalTopicResult,
    format: &crate::cli::enums::OutputFormat,
) -> Result<()> {
    println!("{}", format_topic_results(result, format)?);
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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod format_coverage_tests {
    //! Regression tests for `analyze cluster` / `analyze topics --format`:
    //! the enum advertises nine values but only `json` was implemented, so
    //! yaml, csv, markdown, junit, summary, text, plain and table all produced
    //! byte-identical human text.
    use super::{format_cluster_results, format_topic_results};
    use crate::cli::enums::OutputFormat;
    use crate::services::local_semantic::{
        LocalCluster, LocalClusterResult, LocalTopic, LocalTopicResult,
    };

    fn cluster_result() -> LocalClusterResult {
        LocalClusterResult {
            clusters: vec![LocalCluster {
                id: 0,
                files: vec![std::path::PathBuf::from("src/lib.rs")],
                size: 1,
            }],
            method: "kmeans".to_string(),
            num_documents: 1,
        }
    }

    fn topic_result() -> LocalTopicResult {
        LocalTopicResult {
            topics: vec![LocalTopic {
                id: 0,
                top_terms: vec![("network".to_string(), 0.5)],
                document_count: 1,
            }],
            num_documents: 1,
        }
    }

    #[test]
    fn test_cluster_formats_are_distinguishable() {
        let result = cluster_result();
        let json = format_cluster_results(&result, &OutputFormat::Json).unwrap();
        let yaml = format_cluster_results(&result, &OutputFormat::Yaml).unwrap();
        let csv = format_cluster_results(&result, &OutputFormat::Csv).unwrap();
        let markdown = format_cluster_results(&result, &OutputFormat::Markdown).unwrap();
        let table = format_cluster_results(&result, &OutputFormat::Table).unwrap();

        for (name, rendered) in [
            ("json", &json),
            ("yaml", &yaml),
            ("csv", &csv),
            ("markdown", &markdown),
        ] {
            assert_ne!(
                rendered, &table,
                "--format {name} must not be the human-text rendering"
            );
        }
        assert!(csv.starts_with("cluster_id,size,file"));
        assert!(markdown.starts_with("# Clustering Results"));
        assert!(yaml.contains("method: kmeans"));
    }

    #[test]
    fn test_topic_formats_are_distinguishable() {
        let result = topic_result();
        let csv = format_topic_results(&result, &OutputFormat::Csv).unwrap();
        let markdown = format_topic_results(&result, &OutputFormat::Markdown).unwrap();
        let yaml = format_topic_results(&result, &OutputFormat::Yaml).unwrap();
        let table = format_topic_results(&result, &OutputFormat::Table).unwrap();

        assert!(csv.starts_with("topic_id,document_count,term,weight"));
        assert_ne!(markdown, table);
        assert_ne!(yaml, table);
        assert!(yaml.contains("num_topics"));
    }

    #[test]
    fn test_junit_is_refused_rather_than_answered_with_prose() {
        let err = format_cluster_results(&cluster_result(), &OutputFormat::Junit)
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("junit"),
            "an unsupported format must be an error naming it, got {err}"
        );
        assert!(format_topic_results(&topic_result(), &OutputFormat::Junit).is_err());
    }
}
