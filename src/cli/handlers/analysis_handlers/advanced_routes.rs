//! Advanced analysis route handlers
//!
//! Handles: DeepContext, Tdg, BuildTdg, LintHotspot, Comprehensive,
//! Duplicates, DefectPrediction, Provability, Clippy

use crate::cli::{self, AnalyzeCommands};
use anyhow::Result;
use std::path::Path;

/// Route deep context analysis command
pub(super) async fn route_deep_context_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::DeepContext {
        path,
        project_path,
        output,
        format,
        full,
        include,
        exclude,
        period_days,
        dag_type,
        max_depth,
        include_patterns,
        exclude_patterns,
        cache_strategy,
        parallel,
        verbose,
        top_files,
    } = cmd
    {
        let path = project_path.unwrap_or(path);
        let converted_dag_type = convert_deep_context_dag_type(dag_type);
        let converted_cache_strategy = convert_cache_strategy(cache_strategy);

        crate::cli::handlers::advanced_analysis_handlers::handle_analyze_deep_context(
            path,
            output,
            format,
            full,
            include,
            exclude,
            period_days,
            Some(converted_dag_type),
            max_depth,
            include_patterns,
            exclude_patterns,
            Some(converted_cache_strategy),
            parallel.is_some(),
            verbose,
            top_files,
        )
        .await
    } else {
        unreachable!("Expected DeepContext command")
    }
}

/// Convert deep context DAG type to standard DAG type
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn convert_deep_context_dag_type(dag_type: cli::DeepContextDagType) -> cli::DagType {
    match dag_type {
        cli::DeepContextDagType::CallGraph => cli::DagType::CallGraph,
        cli::DeepContextDagType::ImportGraph => cli::DagType::ImportGraph,
        cli::DeepContextDagType::Inheritance => cli::DagType::Inheritance,
        cli::DeepContextDagType::FullDependency => cli::DagType::FullDependency,
    }
}

/// Convert cache strategy to string
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn convert_cache_strategy(strategy: cli::DeepContextCacheStrategy) -> String {
    match strategy {
        cli::DeepContextCacheStrategy::Normal => "normal".to_string(),
        cli::DeepContextCacheStrategy::ForceRefresh => "force-refresh".to_string(),
        cli::DeepContextCacheStrategy::Offline => "offline".to_string(),
    }
}
/// Route TDG analysis command
pub(super) async fn route_tdg_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::Tdg {
        path,
        threshold,
        top_files,
        format,
        include_components,
        output,
        critical_only,
        verbose,
        ml: _, // GH-97: ML flag (not yet implemented in handler)
    } = cmd
    {
        use crate::cli::handlers::new_tdg_handler::TdgAnalysisConfig;

        let config = TdgAnalysisConfig {
            path,
            threshold: Some(threshold),
            top_files: Some(top_files),
            format,
            include_components,
            output,
            critical_only,
            verbose,
        };

        crate::cli::handlers::new_tdg_handler::handle_analyze_tdg(config).await
    } else {
        unreachable!("Expected Tdg command")
    }
}

/// Run cargo build step for build-tdg command
fn run_cargo_build(path: &Path, release: bool) -> Result<()> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    use std::process::Command;
    println!("\u{1f4e6} Building project...");
    let mut build_cmd = Command::new("cargo");
    build_cmd.arg("build");
    if release {
        build_cmd.arg("--release");
    }
    build_cmd.current_dir(path);
    let status = build_cmd.status()?;
    if !status.success() {
        anyhow::bail!("Build failed with exit code: {:?}", status.code());
    }
    println!("\u{2705} Build successful\n");
    Ok(())
}

/// Check for quality regressions against baseline
fn check_quality_regression(path: &Path) -> Result<()> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let baseline_path = path.join(".pmat/baseline.json");
    if !baseline_path.exists() {
        println!(
            "\u{26a0}\u{fe0f}  No baseline found at {}, skipping regression check",
            baseline_path.display()
        );
        println!("   Run 'pmat tdg baseline create' to create a baseline");
        return Ok(());
    }
    println!("\u{1f50d} Checking for quality regressions...");
    let status = std::process::Command::new("pmat")
        .args([
            "tdg",
            "check-regression",
            "--baseline",
            baseline_path.to_str().unwrap_or(".pmat/baseline.json"),
            "--path",
            path.to_str().unwrap_or("."),
            "--fail-on-regression",
        ])
        .status();
    match status {
        Ok(s) if s.success() => println!("\u{2705} No regressions detected"),
        Ok(_) => anyhow::bail!("Quality regression detected"),
        Err(e) => println!("\u{26a0}\u{fe0f}  Could not run regression check: {}", e),
    }
    Ok(())
}

/// Route build-tdg analysis command (build + TDG quality gate)
pub(super) async fn route_build_tdg_analysis(cmd: AnalyzeCommands) -> Result<()> {
    let AnalyzeCommands::BuildTdg {
        path,
        release,
        threshold,
        fail_on_regression,
        tdg_only,
        top_files,
        format,
        output,
    } = cmd
    else {
        unreachable!("Expected BuildTdg command")
    };

    use crate::cli::handlers::new_tdg_handler::TdgAnalysisConfig;

    if !tdg_only {
        run_cargo_build(&path, release)?;
    }

    println!("\u{1f4ca} Running TDG analysis...");
    let config = TdgAnalysisConfig {
        path: path.clone(),
        threshold: Some(threshold),
        top_files: Some(top_files),
        format,
        include_components: false,
        output,
        critical_only: false,
        verbose: false,
    };

    let result = crate::cli::handlers::new_tdg_handler::handle_analyze_tdg(config).await;

    if fail_on_regression {
        check_quality_regression(&path)?;
    }

    result
}

/// Route lint hotspot analysis command
pub(super) async fn route_lint_hotspot_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::LintHotspot {
        path,
        project_path,
        file,
        format,
        max_density,
        min_confidence,
        enforce,
        dry_run,
        enforcement_metadata,
        output,
        perf,
        clippy_flags,
        top_files,
        include,
        exclude,
    } = cmd
    {
        let path = project_path.unwrap_or(path);
        crate::cli::handlers::lint_hotspot_handlers::handle_analyze_lint_hotspot(
            path,
            file,
            format,
            max_density,
            min_confidence,
            enforce,
            dry_run,
            enforcement_metadata,
            output,
            perf,
            clippy_flags,
            top_files,
            include,
            exclude,
        )
        .await
    } else {
        unreachable!("Expected LintHotspot command")
    }
}

/// Route comprehensive analysis command
pub(super) async fn route_comprehensive_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::Comprehensive {
        path,
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
        top_files: _,
    } = cmd
    {
        let path = project_path.unwrap_or(path);
        crate::cli::handlers::advanced_analysis_handlers::handle_analyze_comprehensive(
            path,
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
        )
        .await
    } else {
        unreachable!("Expected Comprehensive command")
    }
}

/// Route duplicates analysis command
pub(super) async fn route_duplicates_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::Duplicates {
        path,
        project_path,
        detection_type,
        threshold,
        min_lines,
        max_tokens,
        format,
        perf,
        include,
        exclude,
        output,
        top_files,
    } = cmd
    {
        let path = project_path.unwrap_or(path);
        let config = crate::cli::handlers::duplication_analysis::DuplicateAnalysisConfig {
            project_path: path,
            detection_type,
            threshold: f64::from(threshold),
            min_lines,
            max_tokens,
            format,
            perf,
            include,
            exclude,
            output,
            top_files,
        };
        crate::cli::handlers::duplication_analysis::handle_analyze_duplicates(config).await
    } else {
        unreachable!("Expected Duplicates command")
    }
}

/// Route defect prediction analysis command
pub(super) async fn route_defect_prediction_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::DefectPrediction {
        path,
        project_path,
        confidence_threshold,
        min_lines,
        include_low_confidence,
        format,
        high_risk_only,
        include_recommendations,
        include,
        exclude,
        output,
        perf,
        top_files,
    } = cmd
    {
        use crate::cli::handlers::defect_prediction_handler::DefectPredictionConfig;

        let path = project_path.unwrap_or(path);
        let config = DefectPredictionConfig {
            project_path: path,
            confidence_threshold,
            min_lines,
            include_low_confidence,
            format,
            high_risk_only,
            include_recommendations,
            include,
            exclude,
            output,
            perf,
            top_files,
        };

        crate::cli::handlers::defect_prediction_handler::handle_analyze_defect_prediction(config)
            .await
    } else {
        unreachable!("Expected DefectPrediction command")
    }
}

/// Route provability analysis command
pub(super) async fn route_provability_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::Provability {
        path,
        project_path,
        functions,
        analysis_depth,
        format,
        high_confidence_only,
        include_evidence,
        output,
        top_files,
    } = cmd
    {
        use crate::cli::handlers::provability_handler::ProvabilityConfig;

        let path = project_path.unwrap_or(path);
        let config = ProvabilityConfig {
            project_path: path,
            functions,
            analysis_depth,
            format,
            high_confidence_only,
            include_evidence,
            output,
            top_files,
        };

        crate::cli::handlers::provability_handler::handle_analyze_provability(config).await
    } else {
        unreachable!("Expected Provability command")
    }
}

/// Route clippy analysis command (complexity: 4)
pub(super) async fn route_clippy_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::Clippy {
        path,
        project_path,
        confidence,
        dry_run,
        fix_codes,
        output,
        perf: _perf,
    } = cmd
    {
        let path = project_path.unwrap_or(path);
        // Call the auto_clippy_fix MCP tool function directly
        use crate::mcp_pmcp::tools::auto_clippy_fix::auto_clippy_fix;

        let confidence_level = Some(confidence.clone());
        let codes = if fix_codes.is_empty() {
            None
        } else {
            Some(fix_codes.clone())
        };

        let result = auto_clippy_fix(
            Some(path.to_string_lossy().to_string()),
            confidence_level,
            Some(dry_run),
            codes,
        )
        .await?;

        if let Some(output_path) = output {
            use std::fs;
            let content = serde_json::to_string_pretty(&result)?;
            fs::write(&output_path, content)?;
            eprintln!("\u{1f4c1} Results written to {}", output_path.display());
        } else {
            eprintln!("{result:?}");
        }

        Ok(())
    } else {
        unreachable!("Expected Clippy command")
    }
}
