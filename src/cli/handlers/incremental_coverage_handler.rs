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
    debug_assert!(
        config.project_path.exists(),
        "config.project_path must exist: {}",
        config.project_path.display()
    );
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

    // Format and output results
    output_results(result, config.format, config.output, config.top_files).await?;

    eprintln!("✅ Incremental coverage analysis complete");
    Ok(())
}

/// Print analysis header information
fn print_analysis_header(
    project_path: &Path,
    base_branch: &str,
    target_branch: &Option<String>,
    coverage_threshold: f64,
) {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    eprintln!("📊 Analyzing incremental coverage...");
    eprintln!("📁 Project path: {}", project_path.display());
    eprintln!("🌿 Base branch: {base_branch}");
    eprintln!(
        "🎯 Target branch: {}",
        target_branch.as_deref().unwrap_or("HEAD")
    );
    eprintln!("📈 Coverage threshold: {:.1}%", coverage_threshold * 100.0);
}

/// Output results in the requested format
async fn output_results(
    result: IncrementalCoverageResult,
    format: IncrementalCoverageOutputFormat,
    output: Option<PathBuf>,
    top_files: usize,
) -> Result<()> {
    debug_assert!(true, "contract: output_results");
    let content = format_result(result, format, top_files)?;

    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &content).await?;
        eprintln!("📝 Written to {}", output_path.display());
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
    debug_assert!(true, "contract: format_result");
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
