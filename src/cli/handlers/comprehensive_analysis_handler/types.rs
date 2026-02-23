#![cfg_attr(coverage_nightly, coverage(off))]

use crate::cli::ComprehensiveOutputFormat;
use std::path::PathBuf;

/// Configuration for comprehensive analysis
#[derive(Debug, Clone)]
pub struct ComprehensiveAnalysisConfig {
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
    pub top_files: usize,
}
