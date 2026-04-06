#![cfg_attr(coverage_nightly, coverage(off))]
//! Main handler functions for Big-O complexity analysis commands

use crate::cli::{BigOOutputFormat, Path};
use crate::services::big_o_analyzer::{BigOAnalysisConfig, BigOAnalyzer};
use anyhow::Result;
use std::path::PathBuf;
use tracing::{debug, info};

use super::filters::apply_report_filters;
use super::output::{format_analysis_output, write_analysis_output};

/// Handle Big-O complexity analysis command
#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_big_o(
    project_path: PathBuf,
    format: BigOOutputFormat,
    confidence_threshold: u8,
    analyze_space: bool,
    include: Vec<String>,
    exclude: Vec<String>,
    high_complexity_only: bool,
    output: Option<PathBuf>,
    perf: bool,
    top_files: usize,
) -> Result<()> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let start_time = std::time::Instant::now();

    print_analysis_header(&project_path, confidence_threshold);

    let config = build_analysis_config(
        project_path,
        include,
        exclude,
        confidence_threshold,
        analyze_space,
    );

    if perf {
        debug!("Analysis configuration: {:?}", config);
    }

    let analyzer = BigOAnalyzer::new();
    let mut report = analyzer.analyze(config).await?;

    apply_report_filters(&mut report, high_complexity_only, top_files, perf);

    let output_content = format_analysis_output(&analyzer, &report, format)?;
    write_analysis_output(&output_content, output).await?;

    print_analysis_summary(&report, start_time.elapsed(), perf);

    Ok(())
}

/// Print analysis header information
pub(super) fn print_analysis_header(project_path: &Path, confidence_threshold: u8) {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    info!("🔍 Starting Big-O complexity analysis");
    info!("📂 Project path: {}", project_path.display());
    info!("🎯 Confidence threshold: {}%", confidence_threshold);
}

/// Build analysis configuration
pub(super) fn build_analysis_config(
    project_path: PathBuf,
    include: Vec<String>,
    exclude: Vec<String>,
    confidence_threshold: u8,
    analyze_space: bool,
) -> BigOAnalysisConfig {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    BigOAnalysisConfig {
        project_path,
        include_patterns: include,
        exclude_patterns: exclude,
        confidence_threshold,
        analyze_space_complexity: analyze_space,
    }
}

/// Print analysis summary
pub(super) fn print_analysis_summary(
    report: &crate::services::big_o_analyzer::BigOAnalysisReport,
    elapsed: std::time::Duration,
    perf: bool,
) {
    info!("✅ Big-O analysis completed in {:?}", elapsed);
    info!("📊 Analyzed {} functions", report.analyzed_functions);

    if !report.high_complexity_functions.is_empty() {
        info!(
            "⚠️ Found {} functions with high complexity",
            report.high_complexity_functions.len()
        );
    }

    if perf {
        let functions_per_sec = report.analyzed_functions as f64 / elapsed.as_secs_f64();
        info!("⚡ Performance: {:.0} functions/second", functions_per_sec);
    }
}
