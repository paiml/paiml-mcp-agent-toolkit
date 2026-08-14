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

    // GH-934: `summary` and `markdown` narrowed the listing to `--top-violations`
    // while `detailed` called `report.format_report()` and `json` serialised the
    // whole report, both DISCARDING the parameter — so `--top-violations 1` and
    // `--top-violations 100` produced byte-identical output on the two formats an
    // agent actually parses, and the documented default of 20 was silently
    // violated by the JSON surface. Every arm takes the limit now.
    match format {
        EntropyOutputFormat::Summary => Ok(format_summary_report(report, top_violations)),
        EntropyOutputFormat::Detailed => Ok(report.format_report_top(top_violations)),
        EntropyOutputFormat::Json => Ok(serde_json::to_string_pretty(&entropy_report_json(
            report,
            top_violations,
        )?)?),
        EntropyOutputFormat::Markdown => Ok(format_markdown_report(report, top_violations)),
    }
}

/// The JSON document `--format json` emits: the whole report, with
/// `actionable_violations` narrowed to `--top-violations`.
///
/// Narrowing a LISTING must never look like a smaller measurement, so the full
/// count travels beside the narrowed array as `total_actionable_violations`. A
/// consumer that reads `len(actionable_violations)` and one that reads the total
/// can then never disagree about whether 20 rows means "20 violations exist" or
/// "20 were shown".
fn entropy_report_json(
    report: &crate::entropy::EntropyReport,
    top_violations: usize,
) -> Result<serde_json::Value> {
    let mut value = serde_json::to_value(report)?;
    let shown = get_top_violations(&report.actionable_violations, top_violations);

    if let Some(obj) = value.as_object_mut() {
        obj.insert(
            "total_actionable_violations".to_string(),
            serde_json::json!(report.actionable_violations.len()),
        );
        obj.insert(
            "actionable_violations".to_string(),
            serde_json::to_value(&shown)?,
        );
    }

    Ok(value)
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

    // COLOUR: every field here interpolated the raw `c::BOLD`/`c::RESET`
    // consts, which are unconditional — `--color never`, `NO_COLOR=1` and a
    // redirected stdout all still produced `^[[1mFiles Analyzed:^[[0m`
    // (GH #684 class). The `c::header`/`c::label`/`c::number` helpers consult
    // `colors_enabled()` and emit the bare payload when colour is off.
    format!(
        "{}\n\n\
         {} {}\n\
         {} {}\n\
         {} {}\n\
         {} {}\n\
         {} {} lines ({})\n\
         {}\n\
         {}\n{}\n",
        c::header("Entropy Analysis Summary"),
        c::label("Files Analyzed:"),
        c::number(&report.total_files_analyzed.to_string()),
        c::label("Source Lines Analyzed:"),
        c::number(&report.entropy_metrics.total_loc.to_string()),
        c::label("Pattern Diversity:"),
        c::number(&crate::entropy::EntropyReport::render_measurement(
            report.entropy_metrics.pattern_diversity
        )),
        c::label("Total Violations:"),
        c::number(&report.actionable_violations.len().to_string()),
        c::label("Potential LOC Reduction:"),
        c::number(&report.total_loc_reduction().to_string()),
        c::number(&format!("{:.1}%", report.reduction_percentage())),
        note,
        c::label("Top Violations:"),
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
            // COLOUR: `sev_color`/`c::RESET`/`c::BOLD` were interpolated raw, so
            // `--color never` still emitted escapes around the severity and the
            // "Fix:" label. `c::colored`/`c::label` honour `colors_enabled()`.
            format!(
                "  {}. {} {} (saves {})\n     {} {}",
                c::number(&(i + 1).to_string()),
                c::colored(sev_color, &format!("{:?}", v.severity)),
                v.message,
                c::number(&v.render_loc_reduction()),
                c::label("Fix:"),
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
        OutputFormat::Summary => {
            // `summary` is documented as "summary statistics only"; it used to be
            // byte-identical to the default rendering, listing every file.
            let _ = writeln!(
                out,
                "{} clustering: {} document(s) in {} cluster(s)",
                result.method,
                result.num_documents,
                result.clusters.len()
            );
            for cluster in &result.clusters {
                let _ = writeln!(out, "  cluster {}: {} file(s)", cluster.id, cluster.size);
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
        OutputFormat::Summary => {
            // Counts only — see the note on the cluster renderer: `summary` used
            // to print the full per-topic term listing, identical to `text`.
            let _ = writeln!(
                out,
                "{} document(s), {} topic(s)",
                result.num_documents,
                result.topics.len()
            );
            for topic in &result.topics {
                let _ = writeln!(
                    out,
                    "  topic {}: {} document(s), {} term(s)",
                    topic.id,
                    topic.document_count,
                    topic.top_terms.len()
                );
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
        let summary = format_cluster_results(&result, &OutputFormat::Summary).unwrap();
        let table = format_cluster_results(&result, &OutputFormat::Table).unwrap();

        for (name, rendered) in [
            ("json", &json),
            ("yaml", &yaml),
            ("csv", &csv),
            ("markdown", &markdown),
            ("summary", &summary),
        ] {
            assert_ne!(
                rendered, &table,
                "--format {name} must not be the human-text rendering"
            );
        }
        assert!(csv.starts_with("cluster_id,size,file"));
        assert!(markdown.starts_with("# Clustering Results"));
        assert!(yaml.contains("method: kmeans"));
        // "Summary statistics only" means no per-file listing.
        assert!(
            !summary.contains("src/lib.rs"),
            "--format summary must not print the full per-file listing: {summary}"
        );
    }

    #[test]
    fn test_topic_formats_are_distinguishable() {
        let result = topic_result();
        let csv = format_topic_results(&result, &OutputFormat::Csv).unwrap();
        let markdown = format_topic_results(&result, &OutputFormat::Markdown).unwrap();
        let yaml = format_topic_results(&result, &OutputFormat::Yaml).unwrap();
        let summary = format_topic_results(&result, &OutputFormat::Summary).unwrap();
        let table = format_topic_results(&result, &OutputFormat::Table).unwrap();

        assert!(csv.starts_with("topic_id,document_count,term,weight"));
        assert_ne!(markdown, table);
        assert_ne!(yaml, table);
        assert_ne!(summary, table);
        assert!(yaml.contains("num_topics"));
        // "Summary statistics only": counts, not the per-topic term listing.
        assert!(
            !summary.contains("network"),
            "--format summary must not print the term listing: {summary}"
        );
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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod colour_gating_tests {
    //! `analyze entropy --color never` (and `NO_COLOR=1`, and a redirected
    //! stdout) still wrote `^[[1mFiles Analyzed:^[[0m ^[[1;37m1^[[0m`, because
    //! the summary renderer interpolated the raw `c::BOLD` / `c::RESET` consts
    //! rather than the helpers that consult `colors_enabled()`.
    use super::{format_summary_report, format_violation_list};
    use crate::entropy::entropy_calculator::{EntropyMetrics, EntropyReport};
    use crate::entropy::violation_detector::{ActionableViolation, Severity};
    use std::collections::BTreeMap;

    fn violation(severity: Severity) -> ActionableViolation {
        ActionableViolation {
            severity,
            pattern: None,
            message: "Result handling repeated 7 times".to_string(),
            fix_suggestion: "Extract a helper".to_string(),
            estimated_loc_reduction: Some(12),
            affected_files: vec![],
            priority_score: 0.5,
        }
    }

    fn report() -> EntropyReport {
        EntropyReport {
            total_files_analyzed: 3,
            actionable_violations: vec![
                violation(Severity::High),
                violation(Severity::Medium),
                violation(Severity::Low),
            ],
            pattern_summary: None,
            entropy_metrics: EntropyMetrics {
                file_level_entropy: Some(0.8),
                module_level_entropy: Some(0.7),
                project_level_entropy: Some(0.65),
                pattern_diversity: Some(0.72),
                total_patterns: 5,
                total_instances: 21,
                total_loc: 120,
                patterns_by_type: BTreeMap::new(),
            },
            measurement_note: None,
        }
    }

    #[test]
    fn summary_emits_no_ansi_when_colour_is_disabled() {
        assert!(
            !crate::cli::colors::colors_enabled(),
            "cargo test captures stdout, so colour must resolve to off here"
        );
        let out = format_summary_report(&report(), 10);
        assert!(
            !out.contains('\x1b'),
            "entropy summary must be plain with colour off, got {out:?}"
        );
    }

    #[test]
    fn violation_list_emits_no_ansi_when_colour_is_disabled() {
        let out = format_violation_list(&[
            violation(Severity::High),
            violation(Severity::Medium),
            violation(Severity::Low),
        ]);
        assert!(
            !out.contains('\x1b'),
            "entropy violation list must be plain with colour off, got {out:?}"
        );
    }

    #[test]
    fn summary_keeps_its_payload_text() {
        // The escapes must go, not the values they wrapped.
        let out = format_summary_report(&report(), 10);
        assert!(out.contains("Entropy Analysis Summary"), "{out}");
        assert!(out.contains("Files Analyzed: 3"), "{out}");
        assert!(out.contains("Source Lines Analyzed: 120"), "{out}");
        assert!(out.contains("Top Violations:"), "{out}");
        assert!(out.contains("Fix: Extract a helper"), "{out}");
        assert!(out.contains("High"), "{out}");
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod top_violations_reaches_every_format_tests {
    //! GH-934: `--top-violations` was applied by `--format summary` and
    //! `--format markdown` and DISCARDED by `--format detailed` (which called
    //! `report.format_report()`) and `--format json` (which serialised the whole
    //! report). `--top-violations 1` and `--top-violations 100` produced
    //! byte-identical output on the two formats an agent parses, so the
    //! documented default of 20 was silently violated by the JSON surface and
    //! two renderers of one command disagreed about how many violations the run
    //! reports.
    use super::format_entropy_report;
    use crate::cli::EntropyOutputFormat;
    use crate::entropy::entropy_calculator::{EntropyMetrics, EntropyReport};
    use crate::entropy::violation_detector::{ActionableViolation, Severity};
    use std::collections::BTreeMap;

    const TOTAL: usize = 6;

    fn report() -> EntropyReport {
        let severities = [
            Severity::High,
            Severity::High,
            Severity::Medium,
            Severity::Medium,
            Severity::Low,
            Severity::Low,
        ];
        EntropyReport {
            total_files_analyzed: 4,
            actionable_violations: severities
                .iter()
                .enumerate()
                .map(|(i, severity)| ActionableViolation {
                    severity: *severity,
                    pattern: None,
                    message: format!("violation number {i}"),
                    fix_suggestion: format!("extract helper {i}"),
                    estimated_loc_reduction: Some(3),
                    affected_files: vec![],
                    priority_score: 1.0 - (i as f64 / 10.0),
                })
                .collect(),
            pattern_summary: None,
            entropy_metrics: EntropyMetrics {
                file_level_entropy: Some(0.8),
                module_level_entropy: Some(0.7),
                project_level_entropy: Some(0.65),
                pattern_diversity: Some(0.72),
                total_patterns: 5,
                total_instances: 21,
                total_loc: 200,
                patterns_by_type: BTreeMap::new(),
            },
            measurement_note: None,
        }
    }

    fn rendered(format: EntropyOutputFormat, top: usize) -> String {
        format_entropy_report(&report(), format, top).unwrap()
    }

    /// `--format json --top-violations 1` returned all 26 violations on this
    /// repo, byte-identically to `--top-violations 100`.
    #[test]
    fn json_honours_the_limit() {
        for limit in [1usize, 2, 4] {
            let json: serde_json::Value =
                serde_json::from_str(&rendered(EntropyOutputFormat::Json, limit)).unwrap();
            assert_eq!(
                json["actionable_violations"].as_array().unwrap().len(),
                limit,
                "--top-violations {limit} must narrow the JSON listing"
            );
        }
        // "0 = all", per the flag's own help text.
        let all: serde_json::Value =
            serde_json::from_str(&rendered(EntropyOutputFormat::Json, 0)).unwrap();
        assert_eq!(
            all["actionable_violations"].as_array().unwrap().len(),
            TOTAL
        );
        // A limit larger than the finding cannot invent rows.
        let over: serde_json::Value =
            serde_json::from_str(&rendered(EntropyOutputFormat::Json, 99)).unwrap();
        assert_eq!(
            over["actionable_violations"].as_array().unwrap().len(),
            TOTAL
        );
    }

    /// Narrowing a LISTING must not look like a smaller measurement.
    #[test]
    fn json_keeps_the_total_beside_the_narrowed_listing() {
        let json: serde_json::Value =
            serde_json::from_str(&rendered(EntropyOutputFormat::Json, 1)).unwrap();
        assert_eq!(
            json["total_actionable_violations"].as_u64().unwrap() as usize,
            TOTAL,
            "the measured count must survive the display limit: {json:#}"
        );
    }

    /// `--format detailed --top-violations 1` printed all 26 `Fix:` lines.
    #[test]
    fn detailed_honours_the_limit() {
        let fix_lines = |s: &str| -> usize {
            s.lines()
                .filter(|l| l.contains("Fix: extract helper"))
                .count()
        };

        for limit in [1usize, 2, 4] {
            let out = rendered(EntropyOutputFormat::Detailed, limit);
            assert_eq!(
                fix_lines(&out),
                limit,
                "--top-violations {limit} must narrow the detailed listing:\n{out}"
            );
        }
        assert_eq!(
            fix_lines(&rendered(EntropyOutputFormat::Detailed, 0)),
            TOTAL
        );
    }

    /// Same rule as the JSON surface: the header count is the measurement.
    #[test]
    fn detailed_keeps_the_total_in_its_header() {
        let out = rendered(EntropyOutputFormat::Detailed, 1);
        assert!(
            out.contains(&format!("Actionable Violations: {TOTAL}")),
            "the full count must survive the display limit:\n{out}"
        );
        assert!(
            out.contains(&format!("Listing: top 1 of {TOTAL}")),
            "a narrowed listing must say so:\n{out}"
        );
    }

    /// The defect was that two formats disagreed with the other two about the
    /// same run; assert all four move together.
    #[test]
    fn every_format_responds_to_the_flag() {
        for format in [
            EntropyOutputFormat::Summary,
            EntropyOutputFormat::Detailed,
            EntropyOutputFormat::Json,
            EntropyOutputFormat::Markdown,
        ] {
            assert_ne!(
                rendered(format.clone(), 1),
                rendered(format.clone(), TOTAL),
                "--format {format:?} ignores --top-violations"
            );
        }
    }
}
