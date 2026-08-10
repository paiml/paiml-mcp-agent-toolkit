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
        ml,
    } = cmd
    {
        // GH-97: the ML scorer is not wired into this handler. The flag used to
        // be destructured and thrown away (`ml: _`), so `analyze complexity
        // --ml` returned byte-identical JSON to a plain run — the same
        // heuristic numbers, presented under a banner promising "trained ML
        // models instead of heuristic formulas". Refuse rather than relabel.
        if ml {
            anyhow::bail!(
                "--ml is not implemented: complexity scores are still computed by the \
                 heuristic formulas, so this flag would relabel them without changing them. \
                 Re-run without --ml (see GH-97)."
            );
        }

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
        let severity_filter = parse_defect_severity_filter(severity.as_deref())?;

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

/// Parse `analyze defects --severity` into a filter.
///
/// `--severity` is an `Option<String>` on the clap side, so clap cannot reject a
/// bad value for us. The match used to end in `_ => None`, which is the SAME
/// value as "the flag was not given": `--severity bogus` printed the complete
/// unfiltered report and exited as if the filter had been honoured. A filter
/// nobody applied must be an error, not a silent no-op.
fn parse_defect_severity_filter(
    severity: Option<&str>,
) -> Result<Option<crate::services::defect_detector::Severity>> {
    use crate::services::defect_detector::Severity;

    let Some(raw) = severity else {
        return Ok(None);
    };
    match raw.to_lowercase().as_str() {
        "critical" => Ok(Some(Severity::Critical)),
        "high" => Ok(Some(Severity::High)),
        "medium" => Ok(Some(Severity::Medium)),
        "low" => Ok(Some(Severity::Low)),
        _ => {
            anyhow::bail!("invalid --severity '{raw}': expected one of critical, high, medium, low")
        }
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

#[cfg(test)]
mod ml_flag_tests {
    use super::*;

    fn complexity_command(ml: bool) -> AnalyzeCommands {
        AnalyzeCommands::Complexity {
            path: PathBuf::from("."),
            project_path: None,
            file: None,
            files: Vec::new(),
            toolchain: None,
            format: cli::ComplexityOutputFormat::Summary,
            output: None,
            max_cyclomatic: None,
            max_cognitive: None,
            include: Vec::new(),
            watch: false,
            top_files: 10,
            fail_on_violation: false,
            timeout: 60,
            ml,
        }
    }

    /// `--ml` was destructured and discarded, so `analyze complexity --ml`
    /// produced byte-identical JSON to a plain run: the heuristic scores under
    /// a banner promising trained ML models. Refusing is honest; silently
    /// relabelling is not.
    #[tokio::test]
    async fn ml_flag_is_refused_rather_than_ignored() {
        let err = route_complexity_analysis(complexity_command(true))
            .await
            .expect_err("--ml must not silently return heuristic scores");
        let msg = err.to_string();
        assert!(
            msg.contains("--ml is not implemented"),
            "unexpected error: {msg}"
        );
    }

    // ── --severity is validated, not silently dropped ───────────────────────

    /// The reported defect: `analyze defects --severity bogus` mapped to
    /// `None`, i.e. exactly what "no --severity at all" means, so the caller
    /// got the full unfiltered report and exit 0.
    #[test]
    fn unrecognised_severity_is_rejected() {
        let err = parse_defect_severity_filter(Some("bogus"))
            .expect_err("an unknown --severity value must not mean 'no filter'");
        let msg = err.to_string();
        assert!(msg.contains("bogus"), "error must quote the value: {msg}");
        assert!(
            msg.contains("critical") && msg.contains("low"),
            "error must list the accepted values: {msg}"
        );
    }

    #[test]
    fn recognised_severities_parse_case_insensitively() {
        use crate::services::defect_detector::Severity;
        assert_eq!(
            parse_defect_severity_filter(Some("Critical")).unwrap(),
            Some(Severity::Critical)
        );
        assert_eq!(
            parse_defect_severity_filter(Some("high")).unwrap(),
            Some(Severity::High)
        );
        assert_eq!(
            parse_defect_severity_filter(Some("MEDIUM")).unwrap(),
            Some(Severity::Medium)
        );
        assert_eq!(
            parse_defect_severity_filter(Some("low")).unwrap(),
            Some(Severity::Low)
        );
        assert_eq!(parse_defect_severity_filter(None).unwrap(), None);
    }
}
