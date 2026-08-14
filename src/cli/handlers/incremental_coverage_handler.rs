#![cfg_attr(coverage_nightly, coverage(off))]
//! Incremental Coverage Analysis Handler
//!
//! Refactored handler using the service facade pattern to reduce complexity.
//! Split into submodules via include!():
//! - incremental_coverage_handler_formatters.rs: format_summary, format_detailed, etc.
//! - incremental_coverage_handler_tests.rs: unit tests and property tests

use crate::cli::IncrementalCoverageOutputFormat;
use crate::services::facades::incremental_coverage_facade::{
    IncrementalCoverageFacade, IncrementalCoverageRequest, IncrementalCoverageResult,
};
use crate::services::service_registry::ServiceRegistry;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Configuration for incremental coverage analysis
#[derive(Debug, Clone)]
pub struct IncrementalCoverageConfig {
    pub project_path: PathBuf,
    pub base_branch: String,
    pub target_branch: Option<String>,
    pub format: IncrementalCoverageOutputFormat,
    pub coverage_threshold: f64,
    pub changed_files_only: bool,
    pub detailed: bool,
    pub output: Option<PathBuf>,
    pub perf: bool,
    pub cache_dir: Option<PathBuf>,
    pub force_refresh: bool,
    pub top_files: usize,
}

/// Refactored handler for incremental coverage analysis using the facade pattern.
///
/// This reduces complexity from 26 to ~8 by delegating to the facade service.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn handle_analyze_incremental_coverage(config: IncrementalCoverageConfig) -> Result<()> {
    // Print analysis header
    print_analysis_header(
        &config.project_path,
        &config.base_branch,
        &config.target_branch,
        config.coverage_threshold,
    );

    // Create service registry and facade
    let registry = Arc::new(ServiceRegistry::new());
    let facade = IncrementalCoverageFacade::new(registry);

    // Build analysis request
    let request = IncrementalCoverageRequest {
        project_path: config.project_path.clone(),
        base_branch: config.base_branch.clone(),
        target_branch: config.target_branch.clone(),
        coverage_threshold: config.coverage_threshold,
        changed_files_only: config.changed_files_only,
        detailed: config.detailed,
        cache_dir: config.cache_dir.clone(),
        force_refresh: config.force_refresh,
        top_files: config.top_files,
    };

    // Perform analysis using facade
    let result = facade.analyze_project(request).await?;

    // Format and output results.
    //
    // `--detailed` reached `IncrementalCoverageRequest.detailed` and stopped
    // there: no analyzer and no renderer read it, so the flag was
    // byte-identical to no flag in summary/json/detailed/markdown alike. It is
    // a shorthand for the report it names — the `detailed` renderer, which
    // already existed and was reachable only through `--format detailed`.
    output_results(
        result,
        effective_format(config.format, config.detailed),
        config.output,
        config.top_files,
    )
    .await?;

    crate::status_eprintln!("✅ Incremental coverage analysis complete");
    Ok(())
}

/// The format actually rendered, once `--detailed` has had its say.
///
/// `--detailed` upgrades only the DEFAULT report: an explicit `--format json`
/// (or markdown, lcov, delta, sarif) is a request for that document and must
/// not be silently turned into a different one. `--format summary --detailed`
/// upgrades, which is the only reading of the two together that leaves
/// `--detailed` meaning anything.
fn effective_format(
    format: IncrementalCoverageOutputFormat,
    detailed: bool,
) -> IncrementalCoverageOutputFormat {
    if detailed && matches!(format, IncrementalCoverageOutputFormat::Summary) {
        IncrementalCoverageOutputFormat::Detailed
    } else {
        format
    }
}

/// Print analysis header information
fn print_analysis_header(
    project_path: &Path,
    base_branch: &str,
    target_branch: &Option<String>,
    coverage_threshold: f64,
) {
    crate::status_eprintln!("📊 Analyzing incremental coverage...");
    crate::status_eprintln!("📁 Project path: {}", project_path.display());
    crate::status_eprintln!("🌿 Base branch: {base_branch}");
    crate::status_eprintln!(
        "🎯 Target branch: {}",
        target_branch.as_deref().unwrap_or("HEAD")
    );
    // `coverage_threshold` is already a percentage (`--help`: default 80.0).
    // Multiplying by 100 here announced "8000.0%" (GH #658).
    crate::status_eprintln!("📈 Coverage threshold: {coverage_threshold:.1}%");
}

/// Output results in the requested format
async fn output_results(
    result: IncrementalCoverageResult,
    format: IncrementalCoverageOutputFormat,
    output: Option<PathBuf>,
    top_files: usize,
) -> Result<()> {
    let content = format_result(result, format, top_files)?;

    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &content).await?;
        crate::status_eprintln!("📝 Written to {}", output_path.display());
    } else {
        println!("{content}");
    }

    Ok(())
}

/// Format the analysis result based on the requested format
fn format_result(
    result: IncrementalCoverageResult,
    format: IncrementalCoverageOutputFormat,
    top_files: usize,
) -> Result<String> {
    match format {
        IncrementalCoverageOutputFormat::Summary => Ok(format_summary(&result, top_files)),
        IncrementalCoverageOutputFormat::Detailed => Ok(format_detailed(&result, top_files)),
        IncrementalCoverageOutputFormat::Json => {
            serde_json::to_string_pretty(&result).map_err(Into::into)
        }
        IncrementalCoverageOutputFormat::Markdown => Ok(format_markdown(&result, top_files)),
        IncrementalCoverageOutputFormat::Lcov => Ok(format_lcov(&result)),
        IncrementalCoverageOutputFormat::Delta => Ok(format_delta(&result, top_files)),
        IncrementalCoverageOutputFormat::Sarif => Ok(format_sarif(&result)),
    }
}

// --- Include submodules ---
include!("incremental_coverage_handler_formatters.rs");
include!("incremental_coverage_handler_tests.rs");

#[cfg(test)]
mod detailed_flag_tests {
    //! `--detailed` was copied into `IncrementalCoverageRequest.detailed` and
    //! read by nothing — no analyzer, no formatter — so
    //! `analyze incremental-coverage --detailed` was `diff`-identical to the
    //! plain run on a fixture with 12 changed files, in every format.
    use super::*;

    #[test]
    fn detailed_upgrades_the_default_summary_report() {
        assert!(matches!(
            effective_format(IncrementalCoverageOutputFormat::Summary, true),
            IncrementalCoverageOutputFormat::Detailed
        ));
    }

    #[test]
    fn without_the_flag_the_default_report_is_unchanged() {
        assert!(matches!(
            effective_format(IncrementalCoverageOutputFormat::Summary, false),
            IncrementalCoverageOutputFormat::Summary
        ));
    }

    /// An explicit `--format` is a request for THAT document; `--detailed` must
    /// not silently swap a machine format for a human one.
    #[test]
    fn an_explicit_format_wins_over_detailed() {
        for format in [
            IncrementalCoverageOutputFormat::Json,
            IncrementalCoverageOutputFormat::Markdown,
            IncrementalCoverageOutputFormat::Lcov,
            IncrementalCoverageOutputFormat::Sarif,
            IncrementalCoverageOutputFormat::Delta,
        ] {
            assert_eq!(
                format!("{:?}", effective_format(format.clone(), true)),
                format!("{format:?}"),
                "--detailed must not override an explicit --format"
            );
        }
    }

    /// The two renderers really do differ, so the upgrade above is observable.
    #[test]
    fn the_detailed_renderer_is_not_the_summary_renderer() {
        let result = IncrementalCoverageResult {
            total_files: 3,
            covered_files: 1,
            coverage_percentage: Some(50.0),
            files_above_threshold: 0,
            files_below_threshold: 1,
            files_not_measured: 2,
            changed_files: vec![],
            summary: "3 changed files".to_string(),
        };
        assert_ne!(
            format_summary(&result, 10),
            format_detailed(&result, 10),
            "if these agreed, --detailed would still change nothing"
        );
    }
}
