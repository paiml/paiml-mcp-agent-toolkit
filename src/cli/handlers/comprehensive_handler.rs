#![cfg_attr(coverage_nightly, coverage(off))]
//! Comprehensive analysis handler implementation
//!
//! This module implements the comprehensive analysis command that aggregates
//! results from multiple analyzers into a unified report.

use crate::cli::ComprehensiveOutputFormat;
use crate::services::defect_report_service::{DefectReportService, ReportFormat};
use anyhow::{Context, Result};
use serde_json;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tracing::info;

/// Configuration for comprehensive analysis
pub struct ComprehensiveConfig {
    pub project_path: PathBuf,
    pub file: Option<PathBuf>,
    pub files: Vec<PathBuf>,
    pub format: ComprehensiveOutputFormat,
    pub include_duplicates: bool,
    pub include_dead_code: bool,
    pub include_defects: bool,
    pub include_complexity: bool,
    pub include_tdg: bool,
    pub confidence_threshold: f32,
    pub min_lines: usize,
    pub include: Option<String>,
    pub exclude: Option<String>,
    pub output: Option<PathBuf>,
    pub perf: bool,
    pub executive_summary: bool,
}

/// Handle comprehensive analysis command
///
/// This function performs a comprehensive multi-dimensional analysis of a project or single file,
/// combining results from multiple analyzers including complexity, technical debt, defects,
/// dead code, and duplicates.
///
/// # Arguments
///
/// * `project_path` - The project directory to analyze
/// * `file` - Optional single file to analyze (overrides project path)
/// * `format` - Output format for the report
/// * `include_duplicates` - Whether to include duplicate detection
/// * `include_dead_code` - Whether to include dead code analysis
/// * `include_defects` - Whether to include defect prediction
/// * `include_complexity` - Whether to include complexity metrics
/// * `include_tdg` - Whether to include Technical Debt Gradient
/// * `confidence_threshold` - Minimum confidence threshold for predictions (0.0-1.0)
/// * `min_lines` - Minimum lines of code for analysis
/// * `include` - Optional file pattern to include
/// * `exclude` - Optional file pattern to exclude
/// * `output` - Optional output file path
/// * `perf` - Whether to show performance metrics
/// * `executive_summary` - Whether to include executive summary
///
/// # Examples
///
/// ```no_run
/// # use std::path::PathBuf;
/// # use anyhow::Result;
/// # use pmat::cli::ComprehensiveOutputFormat;
/// # async fn example() -> Result<()> {
/// use pmat::cli::handlers::comprehensive_handler::handle_analyze_comprehensive;
///
/// // Analyze entire project
/// handle_analyze_comprehensive(
///     PathBuf::from("."),
///     None,
///     vec![],  // files
///     ComprehensiveOutputFormat::Summary,
///     true,  // include_duplicates
///     true,  // include_dead_code
///     true,  // include_defects
///     true,  // include_complexity
///     true,  // include_tdg
///     0.5,   // confidence_threshold
///     10,    // min_lines
///     None,  // include pattern
///     None,  // exclude pattern
///     None,  // output file
///     false, // perf
///     false, // executive_summary
/// ).await?;
///
/// // Analyze single file
/// handle_analyze_comprehensive(
///     PathBuf::from("."),
///     Some(PathBuf::from("src/main.rs")),
///     vec![],  // files
///     ComprehensiveOutputFormat::Detailed,
///     true,  // include_duplicates
///     true,  // include_dead_code
///     true,  // include_defects
///     true,  // include_complexity
///     true,  // include_tdg
///     0.7,   // confidence_threshold
///     10,    // min_lines
///     None,  // include pattern
///     None,  // exclude pattern
///     Some(PathBuf::from("report.md")), // output file
///     true,  // perf
///     true,  // executive_summary
/// ).await?;
/// # Ok(())
/// # }
/// ```
#[allow(clippy::too_many_arguments)]
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
) -> Result<()> {
    let config = ComprehensiveConfig {
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
    };

    handle_analyze_comprehensive_with_config(config).await
}

include!("comprehensive_handler_analysis.rs");
include!("comprehensive_handler_tests.rs");
