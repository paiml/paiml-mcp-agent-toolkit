#![cfg_attr(coverage_nightly, coverage(off))]
//! Types for simplified deep context analysis.
use std::path::PathBuf;

/// Simplified deep context analysis service
pub struct SimpleDeepContext;

/// Analysis configuration
#[derive(Debug, Clone)]
pub struct SimpleAnalysisConfig {
    pub project_path: PathBuf,
    pub include_features: Vec<String>,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub enable_verbose: bool,
}

/// Analysis report
#[derive(Debug)]
pub struct SimpleAnalysisReport {
    pub file_count: usize,
    pub analysis_duration: std::time::Duration,
    pub complexity_metrics: ComplexityMetrics,
    pub recommendations: Vec<String>,
    pub file_complexity_details: Vec<FileComplexityDetail>,
}

#[derive(Debug)]
/// Complexity metrics.
pub struct ComplexityMetrics {
    pub total_functions: usize,
    pub high_complexity_count: usize,
    pub avg_complexity: f64,
}

#[derive(Debug, Clone)]
/// File complexity detail.
pub struct FileComplexityDetail {
    pub file_path: PathBuf,
    pub function_count: usize,
    pub high_complexity_functions: usize,
    pub avg_complexity: f64,
    pub complexity_score: f64,       // Weighted score for ranking
    pub function_names: Vec<String>, // Individual function names extracted from AST
}

/// Internal per-file complexity metrics (not pub)
#[derive(Debug)]
pub(super) struct FileComplexityMetrics {
    pub function_count: usize,
    pub high_complexity_functions: usize,
    pub avg_complexity: f64,
    pub function_names: Vec<String>,
}

impl Default for SimpleDeepContext {
    fn default() -> Self {
        Self::new()
    }
}
