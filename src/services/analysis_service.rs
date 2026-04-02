#![cfg_attr(coverage_nightly, coverage(off))]
//! Unified analysis service implementing the Service trait
//!
//! Provides various code analysis capabilities through a unified interface

use super::service_base::{Service, ServiceMetrics, ValidationError};
use crate::services::dead_code_analyzer::DeadCodeAnalyzer;
use crate::services::satd_detector::SATDDetector;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Input for analysis operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisInput {
    pub operation: AnalysisOperation,
    pub path: PathBuf,
    pub options: AnalysisOptions,
}

/// Available analysis operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisOperation {
    Complexity,
    Satd,
    DeadCode,
    All,
}

/// Options for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisOptions {
    pub max_complexity: Option<u32>,
    pub include_tests: bool,
    pub parallel: bool,
    pub format: OutputFormat,
}

impl Default for AnalysisOptions {
    fn default() -> Self {
        Self {
            max_complexity: Some(20),
            include_tests: false,
            parallel: true,
            format: OutputFormat::Json,
        }
    }
}

pub use crate::contracts::OutputFormat;

/// Output from analysis operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisOutput {
    pub operation: AnalysisOperation,
    pub results: AnalysisResults,
    pub summary: AnalysisSummary,
}

/// Analysis results container
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AnalysisResults {
    Complexity(ComplexityResults),
    Satd(SatdResults),
    DeadCode(DeadCodeResults),
    Combined(CombinedResults),
}

/// Complexity analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityResults {
    pub total_files: usize,
    pub average_complexity: f64,
    pub max_complexity: u32,
    pub violations: Vec<ComplexityViolation>,
}

/// SATD analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SatdResults {
    pub total_files: usize,
    pub total_satd: usize,
    pub violations: Vec<SatdViolation>,
}

/// Dead code analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadCodeResults {
    pub total_files: usize,
    pub dead_code_count: usize,
    pub dead_code_percentage: f64,
    pub unused_items: Vec<UnusedItem>,
}

/// Combined results from all analyses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombinedResults {
    pub complexity: ComplexityResults,
    pub satd: SatdResults,
    pub dead_code: DeadCodeResults,
}

/// Summary of analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisSummary {
    pub files_analyzed: usize,
    pub total_issues: usize,
    pub critical_issues: usize,
    pub duration_ms: u64,
}

/// Individual complexity violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityViolation {
    pub file: String,
    pub function: String,
    pub complexity: u32,
    pub line: usize,
}

/// Individual SATD violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SatdViolation {
    pub file: String,
    pub line: usize,
    pub comment: String,
    pub category: String,
}

/// Individual unused item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnusedItem {
    pub file: String,
    pub item: String,
    pub item_type: String,
    pub line: usize,
}

/// Unified analysis service
pub struct AnalysisService {
    metrics: Arc<RwLock<ServiceMetrics>>,
    satd_detector: SATDDetector,
}

impl AnalysisService {
    #[must_use]
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(ServiceMetrics::default())),
            satd_detector: SATDDetector::new(),
        }
    }
}

impl Default for AnalysisService {
    fn default() -> Self {
        Self::new()
    }
}

// Analysis methods: complexity, SATD, dead code
include!("analysis_service_analyzers.rs");
// Service trait impl: process dispatch, validation, metrics
include!("analysis_service_dispatch.rs");
// Unit tests and property tests
include!("analysis_service_tests.rs");
