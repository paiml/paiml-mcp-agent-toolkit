//! Complexity analysis command handlers
//!
//! This module contains all complexity-related command implementations
//! extracted from the main CLI module to reduce cognitive complexity.

use crate::cli::*;
use anyhow::Result;
use std::path::PathBuf;

/// Handle complexity analysis command
#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_complexity(
    project_path: PathBuf,
    toolchain: Option<String>,
    format: ComplexityOutputFormat,
    output: Option<PathBuf>,
    max_cyclomatic: Option<u16>,
    max_cognitive: Option<u16>,
    include: Vec<String>,
    watch: bool,
    top_files: usize,
) -> Result<()> {
    use crate::services::complexity::{
        aggregate_results, format_as_sarif, format_complexity_report, format_complexity_summary,
    };

    if watch {
        eprintln!("❌ Watch mode not yet implemented");
        return Ok(());
    }

    // Detect toolchain if not specified
    let detected_toolchain = super::super::detect_toolchain(&project_path, toolchain)?;

    eprintln!("🔍 Analyzing {detected_toolchain} project complexity...");

    // Custom thresholds
    let _thresholds = super::super::build_complexity_thresholds(max_cyclomatic, max_cognitive);

    // Analyze files
    let file_metrics =
        super::super::analyze_project_files(&project_path, &detected_toolchain, &include).await?;

    eprintln!("📊 Analyzed {} files", file_metrics.len());

    // Aggregate results
    let report = aggregate_results(file_metrics.clone());

    // Handle top-files ranking if requested
    let mut content = match format {
        ComplexityOutputFormat::Summary => format_complexity_summary(&report),
        ComplexityOutputFormat::Full => format_complexity_report(&report),
        ComplexityOutputFormat::Json => serde_json::to_string_pretty(&report)?,
        ComplexityOutputFormat::Sarif => format_as_sarif(&report)?,
    };

    // Add top files ranking if requested
    if top_files > 0 {
        content = super::super::add_top_files_ranking(content, format, &file_metrics, top_files)?;
    }

    // Write output
    super::super::analysis_helpers::write_analysis_output(
        &content,
        output,
        "Complexity analysis written to:",
    )
    .await?;
    Ok(())
}

/// Handle churn analysis command
pub async fn handle_analyze_churn(
    project_path: PathBuf,
    days: u32,
    format: crate::models::churn::ChurnOutputFormat,
    output: Option<PathBuf>,
) -> Result<()> {
    // Delegate to main implementation for now - will be extracted in Phase 3 Day 8
    super::super::handle_analyze_churn(project_path, days, format, output).await
}

/// Handle dead code analysis command
pub async fn handle_analyze_dead_code(
    path: PathBuf,
    format: DeadCodeOutputFormat,
    top_files: Option<usize>,
    include_unreachable: bool,
    min_dead_lines: usize,
    include_tests: bool,
    output: Option<PathBuf>,
) -> Result<()> {
    use crate::models::dead_code::DeadCodeAnalysisConfig;
    use crate::services::dead_code_analyzer::DeadCodeAnalyzer;

    eprintln!("☠️ Analyzing dead code in project...");

    // Create analyzer with a reasonable capacity (we'll adjust this as needed)
    let mut analyzer = DeadCodeAnalyzer::new(10000);

    // Configure analysis
    let config = DeadCodeAnalysisConfig {
        include_unreachable,
        include_tests,
        min_dead_lines,
    };

    // Run analysis with ranking
    let mut result = analyzer.analyze_with_ranking(&path, config).await?;

    // Apply top_files limit if specified
    if let Some(limit) = top_files {
        result.ranked_files.truncate(limit);
    }

    eprintln!(
        "📊 Analysis complete: {} files analyzed, {} with dead code",
        result.summary.total_files_analyzed, result.summary.files_with_dead_code
    );

    // Format and output results
    let content = super::super::format_dead_code_output(&result, &format)?;

    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &content).await?;
        eprintln!(
            "✅ Dead code analysis written to: {}",
            output_path.display()
        );
    } else {
        println!("{content}");
    }

    Ok(())
}

/// Handle SATD (Self-Admitted Technical Debt) analysis command
#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_satd(
    path: PathBuf,
    format: SatdOutputFormat,
    severity: Option<SatdSeverity>,
    critical_only: bool,
    include_tests: bool,
    evolution: bool,
    days: u32,
    metrics: bool,
    output: Option<PathBuf>,
) -> Result<()> {
    // Delegate to main implementation for now - will be extracted in Phase 3 Day 8
    super::super::handle_analyze_satd(
        path,
        format,
        severity,
        critical_only,
        include_tests,
        evolution,
        days,
        metrics,
        output,
    )
    .await
}

/// Handle DAG (Dependency Analysis Graph) generation command
#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_dag(
    dag_type: DagType,
    project_path: PathBuf,
    output: Option<PathBuf>,
    max_depth: Option<usize>,
    filter_external: bool,
    show_complexity: bool,
    include_duplicates: bool,
    include_dead_code: bool,
    enhanced: bool,
) -> Result<()> {
    // Delegate to main implementation for now - will be extracted in Phase 3 Day 8
    super::super::handle_analyze_dag(
        dag_type,
        project_path,
        output,
        max_depth,
        filter_external,
        show_complexity,
        include_duplicates,
        include_dead_code,
        enhanced,
    )
    .await
}
