//! Advanced analysis command handlers
//!
//! This module contains handlers for advanced analysis features like
//! deep context, TDG, provability, and comprehensive analysis.

#![cfg_attr(coverage_nightly, coverage(off))]
use crate::cli::{
    ComprehensiveOutputFormat, DagType, DeepContextOutputFormat, DefectPredictionOutputFormat,
    GraphMetricType, GraphMetricsOutputFormat, MakefileOutputFormat, SymbolTableOutputFormat,
    SymbolTypeFilter, TdgOutputFormat,
};
use crate::services::simple_deep_context::{SimpleAnalysisConfig, SimpleDeepContext};
use anyhow::Result;
use std::path::PathBuf;
use tracing::{debug, info};

/// Handle deep context analysis command
///
/// Performs comprehensive analysis of project context, including code relationships,
/// dependencies, and architectural patterns. This addresses issue #33 where the
/// command wasn't finding anything by implementing proper file discovery and analysis.
///
/// # Examples
///
/// ```no_run
/// use pmat::cli::{DeepContextOutputFormat, DagType};
/// use pmat::cli::handlers::advanced_analysis_handlers::handle_analyze_deep_context;
/// use std::path::PathBuf;
///
/// # async fn example() -> anyhow::Result<()> {
/// // Basic deep context analysis
/// handle_analyze_deep_context(
///     PathBuf::from("."),
///     None,                              // output
///     DeepContextOutputFormat::Json,     // format
///     false,                             // full
///     vec![],                            // include
///     vec![],                            // exclude
///     30,                                // period_days
///     None,                              // dag_type
///     None,                              // max_depth
///     vec![],                            // include_patterns
///     vec![],                            // exclude_patterns
///     None,                              // cache_strategy
///     false,                             // parallel
///     false,                             // verbose
///     10,                                // top_files
/// ).await?;
/// # Ok(())
/// # }
/// ```
///
/// ```no_run
/// # use pmat::cli::{DeepContextOutputFormat, DagType};
/// # use pmat::cli::handlers::advanced_analysis_handlers::handle_analyze_deep_context;
/// # use std::path::PathBuf;
/// # async fn example() -> anyhow::Result<()> {
/// // Pattern-selected analysis written to a file
/// handle_analyze_deep_context(
///     PathBuf::from("./src"),
///     Some(PathBuf::from("context.json")),
///     DeepContextOutputFormat::Json,
///     false,                             // --full is not implemented (rejected)
///     vec![],                            // --include is not implemented (rejected)
///     vec![],                            // --exclude is not implemented (rejected)
///     90,                                // 90 day history
///     Some(DagType::CallGraph),
///     None,                              // --max-depth is not implemented (rejected)
///     vec!["**/*.rs".to_string()],       // only Rust files
///     vec!["**/tests/**".to_string()],   // exclude tests
///     Some("persistent".to_string()),
///     false,                             // --parallel is not implemented (rejected)
///     true,                              // verbose output
///     20,                                // top 20 files
/// ).await?;
/// # Ok(())
/// # }
/// ```
///
/// # Returns
///
/// Returns `Ok(())` if analysis completes successfully, or an error if:
/// - Project path doesn't exist
/// - An unimplemented flag was supplied (see
///   `reject_unimplemented_deep_context_flags`)
/// - No files found to analyze
/// - Output file cannot be written
/// - Analysis encounters errors
#[allow(clippy::too_many_arguments)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_analyze_deep_context(
    project_path: PathBuf,
    output: Option<PathBuf>,
    format: DeepContextOutputFormat,
    full: bool,
    include: Vec<String>,
    exclude: Vec<String>,
    period_days: u32,
    dag_type: Option<DagType>,
    max_depth: Option<usize>,
    include_patterns: Vec<String>,
    exclude_patterns: Vec<String>,
    cache_strategy: Option<String>,
    parallel: bool,
    verbose: bool,
    top_files: usize,
) -> Result<()> {
    // The doc comment above already promised "Returns an error if: Project path
    // doesn't exist" and the `path_exists` contract annotation asserted it, but
    // nothing checked: a nonexistent path produced a full report ending in
    // "Average Complexity: 0.0 / 1. No functions detected - verify file
    // discovery patterns" with exit 0. Found alongside GH-663/GH-666.
    crate::cli::ensure_analysis_path_exists(&project_path)?;

    reject_unimplemented_deep_context_flags(
        full,
        &include,
        &exclude,
        max_depth,
        parallel,
        dag_type.is_some(),
        cache_strategy.is_some(),
    )?;

    info!("🔍 Starting deep context analysis");
    info!("📂 Project path: {}", project_path.display());
    info!("📊 Analysis period: {} days", period_days);

    // Create simple deep context analyzer
    let analyzer = SimpleDeepContext::new();

    // Build configuration
    let mut include_features = include;
    if full {
        include_features.push("all".to_string());
    }

    let combined_exclude = with_common_exclusions(exclude_patterns);

    // Issue #659: `--format sarif` used to fall through to
    // `analyzer.format_as_json(&report)`, so its output was byte-identical to
    // `--format json` (top-level keys summary/files/recommendations; no
    // $schema, no version, no runs) and no SARIF consumer could ingest it.
    // `DeepContextAnalyzer::format_as_sarif` — a real SARIF 2.1.0 emitter —
    // was already compiled into the binary with no caller at all.
    if matches!(format, DeepContextOutputFormat::Sarif) {
        let sarif = deep_context_sarif(
            &project_path,
            period_days,
            &include_patterns,
            &combined_exclude,
        )
        .await?;
        return write_deep_context_output(&sarif, output.as_ref()).await;
    }

    let config = SimpleAnalysisConfig {
        project_path: project_path.clone(),
        include_features,
        include_patterns,
        exclude_patterns: combined_exclude,
        enable_verbose: verbose,
    };

    if verbose {
        debug!("Analysis configuration: {:?}", config);
    }

    // Perform analysis
    let report = analyzer.analyze(config).await?;

    // #1015: an empty directory produced the whole report — "Files Analyzed: 0
    // / Total Functions: 0 / Average Complexity: 0.0" and the recommendation
    // "No functions detected - verify file discovery patterns" — and exited 0.
    // That recommendation is the analyzer telling the user its own output is
    // untrustworthy while the exit code says the opposite; a caller that reads
    // the exit code (every CI gate) cannot see it.
    crate::cli::ensure_source_files_were_analyzed(
        "deep-context",
        &project_path,
        report.file_count,
    )?;

    // Format and output results
    let output_content = match (&format, &output) {
        (DeepContextOutputFormat::Json, _) => analyzer.format_as_json(&report)?,
        (DeepContextOutputFormat::Markdown, Some(_)) => {
            // File output: real markdown
            analyzer.format_as_markdown(&report, top_files)
        }
        (DeepContextOutputFormat::Markdown, None) => {
            // Terminal output: colorized text
            format_deep_context_text(&report, top_files)
        }
        // Sarif returned earlier via the real DeepContextAnalyzer (issue #659).
        (DeepContextOutputFormat::Sarif, _) => unreachable!("SARIF handled above"),
    };

    write_deep_context_output(&output_content, output.as_ref()).await?;

    // Print summary
    info!("✅ Deep context analysis completed successfully");
    info!(
        "📊 Analyzed {} files in {:?}",
        report.file_count, report.analysis_duration
    );
    info!(
        "💡 Generated {} recommendations",
        report.recommendations.len()
    );

    Ok(())
}

/// Refuse the flags `analyze deep-context` accepts but does not implement.
///
/// `--full`, `--include`, `--exclude`, `--max-depth` and `--parallel` were
/// bound to underscore-prefixed parameters and never read, so nine variants of
/// the command over the same corpus produced one identical report (same md5
/// after stripping the duration line) — including `--exclude-pattern '*.py'`
/// runs that still listed `main.py`. `--exclude-pattern`/`--include-pattern`
/// now really filter; the flags below still do nothing, and a flag that --help
/// documents as changing the analysis must fail loudly rather than be dropped
/// on the floor.
///
/// `--dag-type` and `--cache-strategy` are checked here too, since #920. They
/// used to be exempt because clap supplied a default for each, so "user asked
/// for it" and "clap filled it in" were indistinguishable — they are `Option`
/// now, and `Some` means the user typed it. deep-context builds no DAG
/// (`SimpleDeepContext` walks files for complexity and SATD) and consults no
/// cache, so all four `--dag-type` values and all three `--cache-strategy`
/// values produced one identical report.
///
/// `--period-days` is still not checked: it carries a clap default and does
/// reach the SARIF path.
#[allow(clippy::fn_params_excessive_bools)]
fn reject_unimplemented_deep_context_flags(
    full: bool,
    include: &[String],
    exclude: &[String],
    max_depth: Option<usize>,
    parallel: bool,
    dag_type: bool,
    cache_strategy: bool,
) -> Result<()> {
    let mut unsupported = Vec::new();
    if full {
        unsupported.push("--full");
    }
    if !include.is_empty() {
        unsupported.push("--include");
    }
    if !exclude.is_empty() {
        unsupported.push("--exclude");
    }
    if max_depth.is_some() {
        unsupported.push("--max-depth");
    }
    if parallel {
        unsupported.push("--parallel");
    }
    if dag_type {
        unsupported.push("--dag-type");
    }
    if cache_strategy {
        unsupported.push("--cache-strategy");
    }

    if unsupported.is_empty() {
        return Ok(());
    }

    anyhow::bail!(
        "analyze deep-context does not implement {}; the flag(s) would be accepted and ignored. \
         Use --include-pattern / --exclude-pattern to select files; --top-files sizes \
         the markdown and text reports (the JSON report is never truncated).",
        unsupported.join(", ")
    )
}

/// Append the exclusions every deep-context run applies.
fn with_common_exclusions(mut exclude_patterns: Vec<String>) -> Vec<String> {
    exclude_patterns.extend([
        "**/target/**".to_string(),
        "**/node_modules/**".to_string(),
        "**/.git/**".to_string(),
        "**/build/**".to_string(),
        "**/dist/**".to_string(),
        "**/__pycache__/**".to_string(),
    ]);
    exclude_patterns
}

/// Produce SARIF 2.1.0 for `analyze deep-context` (issue #659).
///
/// `SimpleDeepContext` carries no line numbers, so SARIF is produced by the
/// full `DeepContextAnalyzer`, whose findings carry real source locations.
async fn deep_context_sarif(
    project_path: &PathBuf,
    period_days: u32,
    include_patterns: &[String],
    exclude_patterns: &[String],
) -> Result<String> {
    use crate::services::deep_context::{DeepContextAnalyzer, DeepContextConfig};

    let config = DeepContextConfig {
        period_days,
        include_patterns: include_patterns.to_vec(),
        exclude_patterns: exclude_patterns.to_vec(),
        ..DeepContextConfig::default()
    };
    let analyzer = DeepContextAnalyzer::new(config);
    let context = analyzer.analyze_project(project_path).await?;
    // `--format sarif` goes through a different analyzer than the other three
    // formats, so it needs the same refusal stated over its own denominator:
    // `ast_contexts` is one entry per file this run built a context for, and an
    // empty SARIF run with zero results is otherwise a clean bill of health.
    crate::cli::ensure_source_files_were_analyzed(
        "deep-context",
        project_path,
        context.analyses.ast_contexts.len(),
    )?;
    analyzer.format_as_sarif(&context)
}

/// Write deep-context output to a file or stdout.
async fn write_deep_context_output(content: &str, output: Option<&PathBuf>) -> Result<()> {
    if let Some(output_path) = output {
        tokio::fs::write(output_path, content).await?;
        info!(
            "📄 Deep context analysis saved to: {}",
            output_path.display()
        );
    } else {
        println!("{content}");
    }
    Ok(())
}

/// Handle TDG (Technical Debt Gradient) analysis command
#[allow(clippy::too_many_arguments)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_analyze_tdg(
    path: PathBuf,
    threshold: Option<f64>,
    top: Option<usize>,
    format: TdgOutputFormat,
    include_components: bool,
    output: Option<PathBuf>,
    critical_only: bool,
    verbose: bool,
) -> Result<()> {
    // Use the enhanced implementation from stubs that supports all modes
    use super::new_tdg_handler::TdgAnalysisConfig;

    let config = TdgAnalysisConfig {
        path,
        threshold,
        top_files: top,
        format,
        include_components,
        output,
        critical_only,
        verbose,
    };

    super::new_tdg_handler::handle_analyze_tdg(config).await
}

/// Handle makefile analysis command
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_analyze_makefile(
    path: PathBuf,
    rules: Vec<String>,
    format: MakefileOutputFormat,
    fix: bool,
    gnu_version: Option<String>,
    top_files: usize,
) -> Result<()> {
    // Delegate to stub implementation for now - will be fully extracted later
    super::super::analysis_utilities::handle_analyze_makefile(
        path,
        rules,
        format,
        fix,
        gnu_version,
        top_files,
    )
    .await
}

// handle_analyze_provability has been moved to provability_handler.rs

/// Handle defect prediction analysis command
#[allow(clippy::too_many_arguments)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_analyze_defect_prediction(
    project_path: PathBuf,
    confidence_threshold: Option<f64>,
    min_lines: Option<usize>,
    include_low_confidence: bool,
    format: DefectPredictionOutputFormat,
    high_risk_only: bool,
    include_recommendations: bool,
    include: Vec<String>,
    exclude: Vec<String>,
    output: Option<PathBuf>,
    perf: bool,
    top_files: usize,
) -> Result<()> {
    // Delegate to the real implementation
    crate::cli::analysis::defect_prediction::handle_analyze_defect_prediction(
        project_path,
        confidence_threshold.unwrap_or(0.5) as f32,
        min_lines.unwrap_or(100),
        include_low_confidence,
        format,
        high_risk_only,
        include_recommendations,
        Some(include.join(",")),
        Some(exclude.join(",")),
        output,
        perf,
        top_files,
    )
    .await
}

/// Handle comprehensive analysis command
#[allow(clippy::too_many_arguments)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_analyze_comprehensive(
    project_path: PathBuf,
    file: Option<PathBuf>,
    files: Vec<PathBuf>,
    format: ComprehensiveOutputFormat,
    include_duplicates: bool,
    include_dead_code: bool,
    include_defects: bool,
    include_complexity: bool,
    include_tdg: bool,
    confidence_threshold: f32,
    min_lines: usize,
    include: Option<String>,
    exclude: Option<String>,
    output: Option<PathBuf>,
    perf: bool,
    executive_summary: bool,
    top_files: usize,
) -> Result<()> {
    use super::comprehensive_analysis_handler::ComprehensiveAnalysisConfig;

    // Create config struct
    let config = ComprehensiveAnalysisConfig {
        project_path,
        file,
        files,
        format,
        include_duplicates,
        include_dead_code,
        include_defects,
        include_complexity,
        include_tdg,
        confidence_threshold,
        min_lines,
        include,
        exclude,
        output,
        perf,
        executive_summary,
        // Was `top_files: 20, // default value`: a literal standing in for the
        // parsed flag, so no `--top-files` value could reach the report.
        top_files,
    };

    // Use the new orchestrator-based comprehensive handler implementation
    super::comprehensive_analysis_handler::handle_analyze_comprehensive(config).await
}

/// Handle graph metrics analysis command
#[allow(clippy::too_many_arguments)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_analyze_graph_metrics(
    project_path: PathBuf,
    metrics: Vec<GraphMetricType>,
    pagerank_seeds: Vec<String>,
    damping_factor: f32,
    max_iterations: usize,
    convergence_threshold: f64,
    export_graphml: bool,
    format: GraphMetricsOutputFormat,
    include: Option<String>,
    exclude: Option<String>,
    output: Option<PathBuf>,
    perf: bool,
    top_k: usize,
    min_centrality: f64,
) -> Result<()> {
    // Delegate to the actual implementation
    crate::cli::analysis::graph_metrics::handle_analyze_graph_metrics(
        project_path,
        metrics,
        pagerank_seeds,
        damping_factor,
        max_iterations,
        convergence_threshold,
        export_graphml,
        format,
        include,
        exclude,
        output,
        perf,
        top_k,
        min_centrality,
    )
    .await
}

/// Handle symbol table analysis command
#[allow(clippy::too_many_arguments)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_analyze_symbol_table(
    project_path: PathBuf,
    format: SymbolTableOutputFormat,
    filter: Option<SymbolTypeFilter>,
    query: Option<String>,
    include: Vec<String>,
    exclude: Vec<String>,
    show_unreferenced: bool,
    show_references: bool,
    output: Option<PathBuf>,
    perf: bool,
    top_files: usize,
) -> Result<()> {
    // Defect #654: this used to pass `Some(include.join(","))` / `Some(exclude.join(","))`.
    // With no --exclude given that is `Some("")`, and the collector skipped every path
    // containing "" — i.e. all of them — so total_symbols was always 0. Pattern lists are
    // now passed through untouched; empty means "no filter".
    crate::cli::analysis::symbol_table::handle_analyze_symbol_table(
        project_path,
        format,
        filter,
        query,
        &include,
        &exclude,
        show_unreferenced,
        show_references,
        output,
        perf,
        top_files,
    )
    .await
}

/// Format deep-context analysis as colorized terminal text
fn format_deep_context_text(
    report: &crate::services::simple_deep_context::SimpleAnalysisReport,
    top_files: usize,
) -> String {
    use crate::cli::colors as c;
    use std::fmt::Write;

    let mut out = String::new();
    let _ = writeln!(&mut out, "{}", c::header("Deep Context Analysis Report"));
    let _ = writeln!(&mut out);

    let _ = writeln!(&mut out, "{}\n", c::subheader("Summary"));
    let _ = writeln!(
        &mut out,
        "  Files Analyzed:         {}",
        c::number(&report.file_count.to_string())
    );
    let _ = writeln!(
        &mut out,
        "  Analysis Duration:      {}",
        c::number(&format!("{:?}", report.analysis_duration))
    );
    let _ = writeln!(
        &mut out,
        "  Total Functions:        {}",
        c::number(&report.complexity_metrics.total_functions.to_string())
    );
    let _ = writeln!(
        &mut out,
        "  High Complexity Funcs:  {}",
        if report.complexity_metrics.high_complexity_count > 0 {
            format!(
                "{}{}{}",
                c::YELLOW,
                report.complexity_metrics.high_complexity_count,
                c::RESET
            )
        } else {
            c::number(&report.complexity_metrics.high_complexity_count.to_string())
        }
    );
    let _ = writeln!(
        &mut out,
        "  Average Complexity:     {}\n",
        c::number(&format!("{:.1}", report.complexity_metrics.avg_complexity))
    );

    if !report.file_complexity_details.is_empty() {
        let _ = writeln!(&mut out, "{}\n", c::subheader("Top Files by Complexity"));
        let mut sorted_files = report.file_complexity_details.clone();
        sorted_files.sort_by(|a, b| {
            b.complexity_score
                .partial_cmp(&a.complexity_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        // `if top_files == 0 { 10 }` read the documented "0 = all" as "0 = ten",
        // so `--top-files 0` silently listed FEWER files than `--top-files 50`.
        // One authority decides what a limit permits: crate::cli::top_files_count.
        let files_to_show = crate::cli::top_files_count(sorted_files.len(), top_files);
        for (i, file_detail) in sorted_files.iter().take(files_to_show).enumerate() {
            // Was `file_name()`: a "Top Files" list of bare basenames cannot be
            // resolved back to a file in a tree that holds many `mod.rs`.
            let path_str = file_detail.file_path.to_string_lossy();
            let filename = crate::cli::report_paths::report_path(&path_str).to_string();
            let _ = writeln!(
                &mut out,
                "  {}. {} - {} avg complexity ({} functions, {} high complexity)",
                c::number(&(i + 1).to_string()),
                c::path(&filename),
                c::number(&format!("{:.1}", file_detail.avg_complexity)),
                c::number(&file_detail.function_count.to_string()),
                if file_detail.high_complexity_functions > 0 {
                    format!(
                        "{}{}{}",
                        c::YELLOW,
                        file_detail.high_complexity_functions,
                        c::RESET
                    )
                } else {
                    c::number(&file_detail.high_complexity_functions.to_string())
                },
            );
        }
        let _ = writeln!(&mut out);
    }

    let _ = writeln!(&mut out, "{}\n", c::subheader("Recommendations"));
    for (i, rec) in report.recommendations.iter().enumerate() {
        let _ = writeln!(&mut out, "  {}. {rec}", c::number(&(i + 1).to_string()));
    }

    out
}

// Tests extracted to advanced_analysis_handlers_tests.rs for file health compliance (CB-040)
#[cfg(test)]
#[path = "advanced_analysis_handlers_tests.rs"]
mod tests;

#[cfg(test)]
mod unimplemented_flag_tests {
    //! A deep-context flag either changes the analysis or is refused.
    use super::*;

    #[test]
    fn supported_invocation_is_accepted() {
        assert!(reject_unimplemented_deep_context_flags(
            false,
            &[],
            &[],
            None,
            false,
            false,
            false
        )
        .is_ok());
    }

    #[test]
    fn full_is_refused_rather_than_ignored() {
        let err =
            reject_unimplemented_deep_context_flags(true, &[], &[], None, false, false, false)
                .expect_err("--full changes nothing, so it must not be accepted");
        assert!(err.to_string().contains("--full"), "{err}");
    }

    /// `--dag-type` was exempt from the refusal because clap gave it a default,
    /// so "user asked" and "clap filled it in" were the same value. It is an
    /// `Option` now: all four values produced one identical report (same sha256
    /// after stripping the duration) because deep-context builds no DAG at all.
    #[test]
    fn dag_type_is_refused_rather_than_ignored() {
        let err =
            reject_unimplemented_deep_context_flags(false, &[], &[], None, false, true, false)
                .expect_err("--dag-type changes nothing, so it must not be accepted");
        assert!(err.to_string().contains("--dag-type"), "{err}");
    }

    /// Same story for `--cache-strategy`: this path consults and writes no
    /// cache, so `normal`, `force-refresh` and `offline` were one report.
    #[test]
    fn cache_strategy_is_refused_rather_than_ignored() {
        let err =
            reject_unimplemented_deep_context_flags(false, &[], &[], None, false, false, true)
                .expect_err("--cache-strategy changes nothing, so it must not be accepted");
        assert!(err.to_string().contains("--cache-strategy"), "{err}");
    }

    #[tokio::test]
    async fn deep_context_refuses_full_instead_of_producing_the_same_report() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("main.rs"), "fn main() {}\n").unwrap();

        let err = handle_analyze_deep_context(
            dir.path().to_path_buf(),
            None,
            DeepContextOutputFormat::Json,
            true, // --full
            vec![],
            vec![],
            30,
            None,
            None,
            vec![],
            vec![],
            None,
            false,
            false,
            10,
        )
        .await
        .expect_err("--full used to produce a report byte-identical to the default one");
        assert!(err.to_string().contains("--full"), "{err}");
    }

    #[test]
    fn every_unimplemented_flag_is_named() {
        let err = reject_unimplemented_deep_context_flags(
            true,
            &["complexity".to_string()],
            &["churn".to_string()],
            Some(0),
            true,
            true,
            true,
        )
        .unwrap_err()
        .to_string();
        for flag in [
            "--full",
            "--include",
            "--exclude",
            "--max-depth",
            "--parallel",
            "--dag-type",
            "--cache-strategy",
        ] {
            assert!(err.contains(flag), "{flag} missing from: {err}");
        }
    }
}
