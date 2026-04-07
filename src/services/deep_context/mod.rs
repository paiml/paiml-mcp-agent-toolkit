//! Deep context analysis for comprehensive code understanding.
//!
//! Orchestrates multiple analysis techniques to generate rich, multi-dimensional
//! context about a codebase. Combines static analysis, historical metrics, and
//! quality indicators for AI/LLM systems.
//!
//! # Analysis Dimensions
//!
//! - **Structure**: AST analysis, dependency graphs, call hierarchies
//! - **Quality**: Complexity metrics, technical debt, code smells
//! - **Evolution**: Code churn, hotspots, stability analysis
//! - **Semantics**: Dead code detection, SATD comments, provability
//! - **Performance**: Big-O complexity, resource usage patterns
//!
//! Uses Rayon for parallel processing of independent analyses.

#![cfg_attr(coverage_nightly, coverage(off))]

use crate::models::{churn::CodeChurnAnalysis, dag::DependencyGraph, tdg::TDGScore};
use crate::services::context::FileContext;
use crate::services::{
    complexity::{ComplexityReport, FileComplexityMetrics},
    file_classifier::FileClassifierConfig,
    quality_gates::QAVerificationResult,
    satd_detector::SATDAnalysisResult,
};
use chrono::{DateTime, Utc};
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use tracing::{debug, info};

// --- Configuration types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepContextConfig {
    pub include_analyses: Vec<AnalysisType>,
    pub period_days: u32,
    pub dag_type: DagType,
    pub complexity_thresholds: Option<ComplexityThresholds>,
    pub max_depth: Option<usize>,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub cache_strategy: CacheStrategy,
    pub parallel: usize,
    /// Configuration for file classification (vendor detection, etc.)
    pub file_classifier_config: Option<FileClassifierConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnalysisType {
    Ast,
    Complexity,
    Churn,
    Dag,
    DeadCode,
    DuplicateCode,
    Satd,
    Provability,
    TechnicalDebtGradient,
    BigO,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DagType {
    CallGraph,
    ImportGraph,
    Inheritance,
    FullDependency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityThresholds {
    pub max_cyclomatic: u16,
    pub max_cognitive: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CacheStrategy {
    Normal,
    ForceRefresh,
    Offline,
}

// --- Analyzer struct ---

/// The main analyzer for generating deep context reports.
pub struct DeepContextAnalyzer {
    config: DeepContextConfig,
}

impl DeepContextAnalyzer {
    /// Creates a new `DeepContextAnalyzer` with the given configuration
    #[must_use]
    pub fn new(config: DeepContextConfig) -> Self {
        Self { config }
    }
}

impl Default for DeepContextConfig {
    fn default() -> Self {
        Self {
            include_analyses: vec![
                AnalysisType::Ast,
                AnalysisType::Complexity,
                AnalysisType::Churn,
                AnalysisType::Dag,
                AnalysisType::DeadCode,
                AnalysisType::Satd,
                AnalysisType::TechnicalDebtGradient,
            ],
            period_days: 30,
            dag_type: DagType::CallGraph,
            complexity_thresholds: None,
            max_depth: Some(10),
            include_patterns: vec![],
            exclude_patterns: vec![
                "**/node_modules/**".to_string(),
                "**/target/**".to_string(),
                "**/.git/**".to_string(),
                "**/vendor/**".to_string(),
            ],
            cache_strategy: CacheStrategy::Normal,
            parallel: num_cpus::get(),
            file_classifier_config: None,
        }
    }
}

impl DeepContextConfig {
    /// Create configuration with auto-scaling concurrency based on system capabilities
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_auto_scaling() -> Self {
        let mut config = Self::default();
        let logical_cores = num_cpus::get();
        let physical_cores = num_cpus::get_physical();
        config.parallel = std::cmp::min(physical_cores + 1, logical_cores);
        config.parallel = std::cmp::max(2, config.parallel);
        config
    }
}

// --- Type definitions split into include files for file health (CB-040) ---

// Core result types: DeepContext, DeepContextResult, AstSummary, DeadCodeAnalysis, ComplexityMetricsForQA
include!("deep_context_result_types.rs");

// Tree and metadata types: ContextMetadata, CacheStats, AnnotatedFileTree, AnalysisResults
include!("deep_context_tree_types.rs");

// Defect types: DefectAnnotations, DeadCodeAnnotation, TechnicalDebtItem, ComplexityViolation
include!("deep_context_defect_types.rs");

// Quality types: QualityScorecard, DefectHotspot, PrioritizedRecommendation
include!("deep_context_quality_types.rs");

// --- Submodules ---

mod analyzer_core;
mod analyzer_formatting;

// Analysis helper functions
include!("analysis_helpers.rs");

// Standalone analysis functions
pub mod analysis_functions;
pub use analysis_functions::{
    analyze_churn, analyze_csharp_file, analyze_file_by_language, analyze_java_file,
    analyze_provability, analyze_python_language, analyze_rust_language, analyze_single_file,
    analyze_swift_file, analyze_typescript_language,
};
