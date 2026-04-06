//! Core analysis route handlers
//!
//! Handles: Bottleneck, Complexity, Churn, DeadCode, Defects, Dag, Satd

use crate::cli::{self, AnalyzeCommands};
use anyhow::Result;
use std::path::PathBuf;

/// Route bottleneck analysis command
pub(super) async fn route_bottleneck_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::Bottleneck {
        path,
        period,
        threshold,
        format,
        output,
    } = cmd
    {
        crate::cli::handlers::bottleneck_handler::handle_bottleneck(
            &path,
            &format,
            period,
            threshold,
            output.as_deref(),
        )
        .await
    } else {
        unreachable!("Expected Bottleneck command")
    }
}

/// Route complexity analysis command
pub(super) async fn route_complexity_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::Complexity {
        path,
        project_path,
        file,
        files,
        toolchain,
        format,
        output,
        max_cyclomatic,
        max_cognitive,
        include,
        watch,
        top_files,
        fail_on_violation,
        timeout,
        ml: _, // GH-97: ML flag (not yet implemented in handler)
    } = cmd
    {
        route_complexity_command(
            path,
            project_path,
            file,
            files,
            toolchain,
            format,
            output,
            max_cyclomatic,
            max_cognitive,
            include,
            watch,
            top_files,
            fail_on_violation,
            timeout,
        )
        .await
    } else {
        unreachable!("Expected Complexity command")
    }
}

/// Route churn analysis command
pub(super) async fn route_churn_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::Churn {
        path,
        project_path,
        days,
        format,
        output,
        top_files,
        include,
        exclude,
    } = cmd
    {
        let path = project_path.unwrap_or(path);
        crate::cli::handlers::complexity_handlers::handle_analyze_churn(
            path, days, format, output, top_files, include, exclude,
        )
        .await
    } else {
        unreachable!("Expected Churn command")
    }
}

/// Route dead code analysis command
pub(super) async fn route_dead_code_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::DeadCode {
        path,
        format,
        top_files,
        include_unreachable,
        min_dead_lines,
        include_tests,
        output,
        fail_on_violation,
        max_percentage,
        timeout,
        include,
        exclude,
        max_depth,
    } = cmd
    {
        crate::cli::handlers::dead_code_handlers::handle_analyze_dead_code(
            path,
            format,
            top_files,
            include_unreachable,
            min_dead_lines,
            include_tests,
            output,
            fail_on_violation,
            max_percentage,
            timeout,
            include,
            exclude,
            max_depth,
        )
        .await
    } else {
        unreachable!("Expected DeadCode command")
    }
}

/// Route defects analysis command
pub(super) async fn route_defects_analysis(cmd: AnalyzeCommands) -> Result<()> {
    use crate::cli::handlers::analyze_defects_handler::{handle_analyze_defects, OutputFormat};
    use crate::services::defect_detector::Severity;

    if let AnalyzeCommands::Defects {
        path,
        file,
        severity,
        format,
        output: _,
    } = cmd
    {
        // Convert format enum to handler's OutputFormat
        let output_format = match format {
            cli::DefectsOutputFormat::Text => OutputFormat::Text,
            cli::DefectsOutputFormat::Json => OutputFormat::Json,
            cli::DefectsOutputFormat::Junit => OutputFormat::Junit,
        };

        // Parse severity filter if provided
        let severity_filter = severity
            .as_deref()
            .and_then(|s| match s.to_lowercase().as_str() {
                "critical" => Some(Severity::Critical),
                "high" => Some(Severity::High),
                "medium" => Some(Severity::Medium),
                "low" => Some(Severity::Low),
                _ => None,
            });

        let exit_code = handle_analyze_defects(
            path.as_deref(),
            file.as_deref(),
            severity_filter,
            output_format,
        )
        .await?;

        // Exit with the handler's exit code (1 if critical defects found)
        if exit_code != 0 {
            std::process::exit(exit_code);
        }

        Ok(())
    } else {
        unreachable!("Expected Defects command")
    }
}

/// Route DAG analysis command
pub(super) async fn route_dag_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::Dag {
        dag_type,
        path,
        project_path,
        output,
        max_depth,
        target_nodes,
        filter_external,
        show_complexity,
        include_duplicates,
        include_dead_code,
        enhanced,
    } = cmd
    {
        let path = project_path.unwrap_or(path);
        crate::cli::handlers::complexity_handlers::handle_analyze_dag(
            dag_type,
            path,
            output,
            max_depth,
            target_nodes,
            filter_external,
            show_complexity,
            include_duplicates,
            include_dead_code,
            enhanced,
        )
        .await
    } else {
        unreachable!("Expected Dag command")
    }
}
/// Route SATD analysis command
pub(super) async fn route_satd_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::Satd {
        path,
        format,
        severity,
        critical_only,
        include_tests,
        strict,
        evolution,
        days,
        metrics,
        output,
        top_files,
        fail_on_violation,
        timeout,
        include,
        exclude,
        extended,
    } = cmd
    {
        use crate::cli::handlers::satd_handler::SatdAnalysisConfig;

        let config = SatdAnalysisConfig {
            path,
            format,
            severity,
            critical_only,
            include_tests,
            strict,
            evolution,
            days,
            metrics,
            output,
            top_files,
            fail_on_violation,
            timeout,
            include,
            exclude,
            extended,
        };

        crate::cli::handlers::satd_handler::handle_analyze_satd(config).await
    } else {
        unreachable!("Expected Satd command")
    }
}

/// Route complexity command (cognitive complexity ≤5)
#[allow(clippy::too_many_arguments)]
async fn route_complexity_command(
    path: PathBuf,
    project_path: Option<PathBuf>,
    file: Option<PathBuf>,
    files: Vec<PathBuf>,
    toolchain: Option<String>,
    format: crate::cli::ComplexityOutputFormat,
    output: Option<PathBuf>,
    max_cyclomatic: Option<u16>,
    max_cognitive: Option<u16>,
    include: Vec<String>,
    watch: bool,
    top_files: usize,
    fail_on_violation: bool,
    timeout: u64,
) -> Result<()> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    // Handle parameter migration: use new 'path' or deprecated 'project_path'
    // Silently accept both for backwards compatibility
    let analysis_path = project_path.unwrap_or(path);

    crate::cli::handlers::complexity_handlers::handle_analyze_complexity(
        analysis_path,
        file,
        files,
        toolchain,
        format,
        output,
        max_cyclomatic,
        max_cognitive,
        include,
        watch,
        top_files,
        fail_on_violation,
        timeout,
    )
    .await
}
