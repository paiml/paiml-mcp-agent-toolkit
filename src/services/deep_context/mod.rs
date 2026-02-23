//! Deep context analysis for comprehensive code understanding.
//!
//! This module orchestrates multiple analysis techniques to generate rich,
//! multi-dimensional context about a codebase. It combines static analysis,
//! historical metrics, and quality indicators to provide AI/LLM systems with
//! deep understanding of code structure, quality, and evolution.
//!
//! # Analysis Dimensions
//!
//! - **Structure**: AST analysis, dependency graphs, call hierarchies
//! - **Quality**: Complexity metrics, technical debt, code smells
//! - **Evolution**: Code churn, hotspots, stability analysis
//! - **Semantics**: Dead code detection, SATD comments, provability
//! - **Performance**: Big-O complexity, resource usage patterns
//!
//! # Parallelization
//!
//! The analyzer uses Rayon for parallel processing of independent analyses,
//! significantly reducing analysis time for large codebases while maintaining
//! deterministic results through careful aggregation.
//!
//! # Example
//!
//! ```ignore
//! use pmat::services::deep_context::{DeepContext, DeepContextConfig, AnalysisType};
//! use std::path::Path;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = DeepContextConfig {
//!     include_analyses: vec![
//!         AnalysisType::Ast,
//!         AnalysisType::Complexity,
//!         AnalysisType::Churn,
//!         AnalysisType::TechnicalDebtGradient,
//!     ],
//!     period_days: 30,
//!     dag_type: DagType::Full,
//!     complexity_thresholds: None,
//!     max_depth: Some(3),
//!     include_patterns: vec!["**/*.rs".to_string()],
//!     exclude_patterns: vec!["**/tests/**".to_string()],
//!     cache_strategy: CacheStrategy::Persistent,
//!     parallel: 4,
//!     file_classifier_config: None,
//! };
//!
//! let analyzer = DeepContext::new(config);
//! let context = analyzer.analyze(Path::new("src/")).await?;
//!
//! // Access multi-dimensional analysis results
//! println!("Total complexity: {}", context.complexity_report.expect("internal error").total_complexity);
//! println!("Code churn hotspots: {}", context.churn_analysis.expect("internal error").summary.hotspot_files.len());
//! println!("Technical debt score: {:.2}", context.tdg_analysis.expect("internal error").summary.overall_tdg_score);
//! # Ok(())
//! # }
//! ```ignore

#![cfg_attr(coverage_nightly, coverage(off))]

use crate::models::{
    churn::CodeChurnAnalysis,
    dag::DependencyGraph,
    tdg::TDGScore,
};
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeepContext {
    pub metadata: ContextMetadata,
    pub file_tree: AnnotatedFileTree,
    pub analyses: AnalysisResults,
    pub quality_scorecard: QualityScorecard,
    pub template_provenance: Option<TemplateProvenance>,
    pub defect_summary: DefectSummary,
    pub hotspots: Vec<DefectHotspot>,
    pub recommendations: Vec<PrioritizedRecommendation>,
    pub qa_verification: Option<QAVerificationResult>,
    pub build_info: Option<crate::models::project_meta::BuildInfo>,
    pub project_overview: Option<crate::models::project_meta::ProjectOverview>,
}

/// Extended structure for QA verification that includes additional fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepContextResult {
    // Core fields from DeepContext
    pub metadata: ContextMetadata,
    pub file_tree: Vec<String>, // List of file paths for quality_gates
    pub analyses: AnalysisResults,
    pub quality_scorecard: QualityScorecard,
    pub template_provenance: Option<TemplateProvenance>,
    pub defect_summary: DefectSummary,
    pub hotspots: Vec<DefectHotspot>,
    pub recommendations: Vec<PrioritizedRecommendation>,
    pub qa_verification: Option<QAVerificationResult>,

    // Additional fields expected by quality_gates
    pub complexity_metrics: Option<ComplexityMetricsForQA>,
    pub dead_code_analysis: Option<DeadCodeAnalysis>,
    pub ast_summaries: Option<Vec<AstSummary>>,
    pub churn_analysis: Option<CodeChurnAnalysis>,
    pub language_stats: Option<FxHashMap<String, usize>>,

    // Project metadata fields
    pub build_info: Option<crate::models::project_meta::BuildInfo>,
    pub project_overview: Option<crate::models::project_meta::ProjectOverview>,
}

/// Summary of AST analysis for a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstSummary {
    pub path: String,
    pub language: String,
    pub total_items: usize,
    pub functions: usize,
    pub classes: usize,
    pub imports: usize,
}

/// Dead code analysis structure expected by `quality_gates`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadCodeAnalysis {
    pub summary: DeadCodeSummary,
    pub dead_functions: Vec<String>,
    pub warnings: Vec<String>,
}

/// Dead code summary structure expected by `quality_gates`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadCodeSummary {
    pub total_functions: usize,
    pub dead_functions: usize,
    pub total_lines: usize,
    pub total_dead_lines: usize,
    pub dead_percentage: f64,
}

/// Complexity metrics structure expected by `quality_gates`
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComplexityMetricsForQA {
    pub files: Vec<FileComplexityMetricsForQA>,
    pub summary: ComplexitySummaryForQA,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileComplexityMetricsForQA {
    pub path: std::path::PathBuf,
    pub functions: Vec<FunctionComplexityForQA>,
    pub total_cyclomatic: u32,
    pub total_cognitive: u32,
    pub total_lines: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionComplexityForQA {
    pub name: String,
    pub cyclomatic: u32,
    pub cognitive: u32,
    pub nesting_depth: u32,
    pub start_line: usize,
    pub end_line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComplexitySummaryForQA {
    pub total_files: usize,
    pub total_functions: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContextMetadata {
    pub generated_at: DateTime<Utc>,
    pub tool_version: String,
    pub project_root: PathBuf,
    pub cache_stats: CacheStats,
    pub analysis_duration: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheStats {
    pub hit_rate: f64,
    pub memory_efficiency: f64,
    pub time_saved_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnnotatedFileTree {
    pub root: AnnotatedNode,
    pub total_files: usize,
    pub total_size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnnotatedNode {
    pub name: String,
    pub path: PathBuf,
    pub node_type: NodeType,
    pub children: Vec<AnnotatedNode>,
    pub annotations: NodeAnnotations,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum NodeType {
    Directory,
    #[default]
    File,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NodeAnnotations {
    pub defect_score: Option<f32>,
    pub complexity_score: Option<f32>,
    pub cognitive_complexity: Option<u16>,
    pub churn_score: Option<f32>,
    pub dead_code_items: usize,
    pub satd_items: usize,
    pub centrality: Option<f32>,
    pub test_coverage: Option<f32>,
    pub big_o_complexity: Option<String>,
    pub memory_complexity: Option<String>,
    pub duplication_score: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnalysisResults {
    pub ast_contexts: Vec<EnhancedFileContext>,
    pub complexity_report: Option<ComplexityReport>,
    pub churn_analysis: Option<CodeChurnAnalysis>,
    pub dependency_graph: Option<DependencyGraph>,
    pub dead_code_results: Option<crate::models::dead_code::DeadCodeRankingResult>,
    pub duplicate_code_results: Option<crate::services::duplicate_detector::CloneReport>,
    pub satd_results: Option<SATDAnalysisResult>,
    pub provability_results:
        Option<Vec<crate::services::lightweight_provability_analyzer::ProofSummary>>,
    pub cross_language_refs: Vec<CrossLangReference>,
    pub big_o_analysis: Option<crate::services::big_o_analyzer::BigOAnalysisReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedFileContext {
    pub base: FileContext,
    pub complexity_metrics: Option<FileComplexityMetrics>,
    pub churn_metrics: Option<FileChurnMetrics>,
    pub defects: DefectAnnotations,
    pub symbol_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChurnMetrics {
    pub commits: u32,
    pub authors: u32,
    pub lines_added: u32,
    pub lines_deleted: u32,
    pub last_modified: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefectAnnotations {
    pub dead_code: Option<DeadCodeAnnotation>,
    pub technical_debt: Vec<TechnicalDebtItem>,
    pub complexity_violations: Vec<ComplexityViolation>,
    pub tdg_score: Option<TDGScore>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadCodeAnnotation {
    pub confidence: ConfidenceLevel,
    pub reason: String,
    pub items: Vec<DeadCodeItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfidenceLevel {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadCodeItem {
    pub name: String,
    pub item_type: DeadCodeItemType,
    pub line: u32,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeadCodeItemType {
    Function,
    Class,
    Module,
    Variable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalDebtItem {
    pub category: TechnicalDebtCategory,
    pub severity: TechnicalDebtSeverity,
    pub content: String,
    pub line: u32,
    pub age_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TechnicalDebtCategory {
    Design,
    Requirements,
    Implementation,
    Testing,
    Documentation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TechnicalDebtSeverity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityViolation {
    pub metric_type: ComplexityMetricType,
    pub actual_value: u32,
    pub threshold: u32,
    pub function_name: String,
    pub line: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplexityMetricType {
    Cyclomatic,
    Cognitive,
    Halstead,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossLangReference {
    pub source_file: PathBuf,
    pub target_file: PathBuf,
    pub reference_type: CrossLangReferenceType,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrossLangReferenceType {
    WasmBinding,
    FfiCall,
    PythonBinding,
    TypeDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QualityScorecard {
    pub overall_health: f64,
    pub complexity_score: f64,
    pub maintainability_index: f64,
    pub modularity_score: f64,
    pub test_coverage: Option<f64>,
    pub technical_debt_hours: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateProvenance {
    pub scaffold_timestamp: DateTime<Utc>,
    pub templates_used: Vec<String>,
    pub parameters: FxHashMap<String, serde_json::Value>,
    pub drift_analysis: DriftAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftAnalysis {
    pub added_files: Vec<PathBuf>,
    pub modified_files: Vec<PathBuf>,
    pub deleted_files: Vec<PathBuf>,
    pub drift_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DefectSummary {
    pub total_defects: usize,
    pub by_severity: FxHashMap<String, usize>,
    pub by_type: FxHashMap<String, usize>,
    pub defect_density: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefectHotspot {
    pub location: FileLocation,
    pub composite_score: f32,
    pub contributing_factors: Vec<DefectFactor>,
    pub refactoring_effort: RefactoringEstimate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileLocation {
    pub file: PathBuf,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DefectFactor {
    DeadCode {
        confidence: ConfidenceLevel,
        reason: String,
    },
    TechnicalDebt {
        category: TechnicalDebtCategory,
        severity: TechnicalDebtSeverity,
        age_days: u32,
    },
    Complexity {
        _cyclomatic: u32,
        _cognitive: u32,
        violations: Vec<String>,
    },
    ChurnRisk {
        commits: u32,
        authors: u32,
        defect_correlation: f32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringEstimate {
    pub estimated_hours: f32,
    pub priority: Priority,
    pub impact: Impact,
    pub suggested_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Impact {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrioritizedRecommendation {
    pub title: String,
    pub description: String,
    pub priority: Priority,
    pub estimated_effort: Duration,
    pub impact: Impact,
    pub prerequisites: Vec<String>,
}

/// The main analyzer for generating deep context reports.
///
/// Provides formatting methods (markdown, JSON, SARIF) and analysis orchestration.
pub struct DeepContextAnalyzer {
    config: DeepContextConfig,
}

impl DeepContextAnalyzer {
    /// Creates a new `DeepContextAnalyzer` with the given configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::services::deep_context::{DeepContextAnalyzer, DeepContextConfig};
    ///
    /// let config = DeepContextConfig::default();
    /// let analyzer = DeepContextAnalyzer::new(config);
    /// // Analyzer is ready to perform deep context analysis
    /// ```
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
    pub fn with_auto_scaling() -> Self {
        let mut config = Self::default();

        // Auto-scale concurrency based on system capabilities
        let logical_cores = num_cpus::get();
        let physical_cores = num_cpus::get_physical();

        // Optimal parallelism: Use physical cores + 1 for I/O bound tasks
        // Cap at logical cores to avoid over-subscription
        config.parallel = std::cmp::min(physical_cores + 1, logical_cores);

        // Ensure minimum of 2 for parallel processing
        config.parallel = std::cmp::max(2, config.parallel);

        config
    }
}

// DeepContextBuildParams moved to analyzer_core::params

// DeepContextAnalyzer formatting methods - split into submodules for file health (CB-040)
mod analyzer_formatting;

// DeepContextAnalyzer core analysis methods - split into submodules for file health (CB-040)
mod analyzer_core;

// Analysis helper functions - extracted for file health (CB-040)
include!("analysis_helpers.rs");

// Standalone analysis functions - refactored into submodules for file health (CB-040)
pub mod analysis_functions;
pub use analysis_functions::{
    analyze_churn, analyze_csharp_file, analyze_file_by_language, analyze_java_file,
    analyze_provability, analyze_python_language, analyze_rust_language, analyze_single_file,
    analyze_swift_file, analyze_typescript_language,
};
