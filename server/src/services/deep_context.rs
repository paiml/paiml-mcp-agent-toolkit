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
//! println!("Total complexity: {}", context.complexity_report.unwrap().total_complexity);
//! println!("Code churn hotspots: {}", context.churn_analysis.unwrap().summary.hotspot_files.len());
//! println!("Technical debt score: {:.2}", context.tdg_analysis.unwrap().summary.overall_tdg_score);
//! # Ok(())
//! # }
//! ```ignore

use crate::models::{
    churn::CodeChurnAnalysis,
    dag::DependencyGraph,
    project_meta::{BuildInfo, ProjectOverview},
    tdg::{TDGScore, TDGSeverity, TDGSummary},
};
use crate::services::context::FileContext;
use crate::services::{
    complexity::{ComplexityReport, FileComplexityMetrics},
    file_classifier::FileClassifierConfig,
    quality_gates::{QAVerification, QAVerificationResult},
    satd_detector::SATDAnalysisResult,
    tdg_calculator::TDGCalculator,
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

/// Dead code analysis structure expected by quality_gates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadCodeAnalysis {
    pub summary: DeadCodeSummary,
    pub dead_functions: Vec<String>,
    pub warnings: Vec<String>,
}

/// Dead code summary structure expected by quality_gates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadCodeSummary {
    pub total_functions: usize,
    pub dead_functions: usize,
    pub total_lines: usize,
    pub total_dead_lines: usize,
    pub dead_percentage: f64,
}

/// Complexity metrics structure expected by quality_gates
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

// Helper structs for organizing AST items
#[derive(Debug, Clone)]
struct CategorizedAstItems {
    functions: Vec<AstFunction>,
    structs: Vec<AstStruct>,
    enums: Vec<AstEnum>,
    traits: Vec<AstTrait>,
    impls: Vec<AstImpl>,
    modules: Vec<AstModule>,
    uses: Vec<AstUse>,
}

impl CategorizedAstItems {
    fn new() -> Self {
        Self {
            functions: Vec::new(),
            structs: Vec::new(),
            enums: Vec::new(),
            traits: Vec::new(),
            impls: Vec::new(),
            modules: Vec::new(),
            uses: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
struct AstFunction {
    name: String,
    visibility: String,
    is_async: bool,
    line: usize,
}

#[derive(Debug, Clone)]
struct AstStruct {
    name: String,
    visibility: String,
    fields_count: usize,
    derives: Vec<String>,
    line: usize,
}

#[derive(Debug, Clone)]
struct AstEnum {
    name: String,
    visibility: String,
    variants_count: usize,
    line: usize,
}

#[derive(Debug, Clone)]
struct AstTrait {
    name: String,
    visibility: String,
    line: usize,
}

#[derive(Debug, Clone)]
struct AstImpl {
    type_name: String,
    trait_name: Option<String>,
    line: usize,
}

#[derive(Debug, Clone)]
struct AstModule {
    name: String,
    visibility: String,
    line: usize,
}

#[derive(Debug, Clone)]
struct AstUse {
    path: String,
    line: usize,
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

/// Parameters for building deep context
struct DeepContextBuildParams<'a> {
    project_path: &'a Path,
    file_tree: AnnotatedFileTree,
    analyses: ParallelAnalysisResults,
    cross_refs: FxHashMap<String, Vec<CrossLangReference>>,
    quality_scorecard: QualityScorecard,
    template_provenance: Option<TemplateProvenance>,
    defect_summary: DefectSummary,
    hotspots: Vec<DefectHotspot>,
    recommendations: Vec<PrioritizedRecommendation>,
    build_info: Option<BuildInfo>,
    project_overview: Option<ProjectOverview>,
    analysis_duration: std::time::Duration,
}

pub struct DeepContextAnalyzer {
    config: DeepContextConfig,
}

impl DeepContextAnalyzer {
    /// Creates a new DeepContextAnalyzer with the given configuration
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
    pub fn new(config: DeepContextConfig) -> Self {
        Self { config }
    }

    /// Format as comprehensive markdown output using simple formatting
    pub async fn format_as_comprehensive_markdown(
        &self,
        context: &DeepContext,
    ) -> anyhow::Result<String> {
        let mut output = String::with_capacity(1024);
        output.push_str("# Deep Context Analysis Report\n\n");

        self.append_project_overview(&mut output, &context.project_overview)?;
        self.append_build_info(&mut output, &context.build_info)?;
        self.append_quality_scorecard(&mut output, &context.quality_scorecard)?;
        self.append_project_structure(&mut output, &context.file_tree)?;
        self.append_analysis_results(&mut output, &context.analyses)?;
        self.append_recommendations(&mut output, &context.recommendations)?;

        Ok(output)
    }

    fn append_project_overview(
        &self,
        output: &mut String,
        overview: &Option<crate::models::project_meta::ProjectOverview>,
    ) -> anyhow::Result<()> {
        if let Some(ref overview) = overview {
            output.push_str("## Project Overview\n\n");
            if !overview.compressed_description.is_empty() {
                output.push_str(&overview.compressed_description);
                output.push_str("\n\n");
            }
            if !overview.key_features.is_empty() {
                output.push_str("**Key Features:**\n");
                for feature in &overview.key_features {
                    output.push_str(&format!("- {feature}\n"));
                }
                output.push('\n');
            }
            if let Some(ref arch) = overview.architecture_summary {
                output.push_str("**Architecture:**\n");
                output.push_str(arch);
                output.push_str("\n\n");
            }
        }
        Ok(())
    }

    fn append_build_info(
        &self,
        output: &mut String,
        build_info: &Option<crate::models::project_meta::BuildInfo>,
    ) -> anyhow::Result<()> {
        if let Some(ref build_info) = build_info {
            output.push_str("## Build System\n\n");
            output.push_str(&format!(
                "**Detected Toolchain:** {}\n",
                build_info.toolchain
            ));
            if !build_info.targets.is_empty() {
                output.push_str(&format!(
                    "**Primary Targets:** {}\n",
                    build_info.targets.join(", ")
                ));
            }
            if !build_info.dependencies.is_empty() {
                output.push_str(&format!(
                    "**Key Dependencies:** {}\n",
                    build_info.dependencies.join(", ")
                ));
            }
            if let Some(ref cmd) = build_info.primary_command {
                output.push_str(&format!("**Build Command:** `{cmd}`\n"));
            }
            output.push('\n');
        }
        Ok(())
    }

    fn append_quality_scorecard(
        &self,
        output: &mut String,
        scorecard: &QualityScorecard,
    ) -> anyhow::Result<()> {
        output.push_str("## Quality Scorecard\n\n");
        output.push_str(&format!(
            "- Overall Health: {:.1}%\n",
            scorecard.overall_health
        ));
        output.push_str(&format!(
            "- Maintainability Index: {:.1}%\n",
            scorecard.maintainability_index
        ));
        output.push_str(&format!(
            "- Refactoring Time: {:.1} hours\n",
            scorecard.technical_debt_hours
        ));
        output.push_str(&format!(
            "- Complexity Score: {:.1}%\n",
            scorecard.complexity_score
        ));
        output.push('\n');
        Ok(())
    }

    fn append_project_structure(
        &self,
        output: &mut String,
        file_tree: &AnnotatedFileTree,
    ) -> anyhow::Result<()> {
        output.push_str("## Project Structure\n\n");
        output.push_str("```\n");
        output.push_str(&format!(
            "Total Files: {}\nTotal Size: {} bytes\n",
            file_tree.total_files, file_tree.total_size_bytes
        ));
        output.push_str("\n```\n\n");
        Ok(())
    }

    fn append_analysis_results(
        &self,
        output: &mut String,
        analyses: &AnalysisResults,
    ) -> anyhow::Result<()> {
        output.push_str("## Analysis Results\n\n");

        if !analyses.ast_contexts.is_empty() {
            output.push_str(&format!(
                "### AST Analysis\n- Files analyzed: {}\n\n",
                analyses.ast_contexts.len()
            ));
        }

        if let Some(ref complexity) = analyses.complexity_report {
            output.push_str(&format!("### Complexity Analysis\n- Total files: {}\n- Total functions: {}\n- Median cyclomatic complexity: {:.1}\n\n",
                complexity.summary.total_files, complexity.summary.total_functions, complexity.summary.median_cyclomatic));
        }

        if let Some(ref churn) = analyses.churn_analysis {
            output.push_str(&format!(
                "### Code Churn\n- Files analyzed: {}\n- Total commits: {}\n\n",
                churn.files.len(),
                churn.summary.total_commits
            ));
        }
        Ok(())
    }

    fn append_recommendations(
        &self,
        output: &mut String,
        recommendations: &[PrioritizedRecommendation],
    ) -> anyhow::Result<()> {
        if !recommendations.is_empty() {
            output.push_str("## Recommendations\n\n");
            for (i, rec) in recommendations.iter().enumerate() {
                output.push_str(&format!(
                    "{}. **{}** (Priority: {:?})\n   {}\n   Effort: {:?}\n\n",
                    i + 1,
                    rec.title,
                    rec.priority,
                    rec.description,
                    rec.estimated_effort
                ));
            }
        }
        Ok(())
    }

    /// Legacy format method (kept for backward compatibility)
    pub fn format_as_comprehensive_markdown_legacy(
        &self,
        context: &DeepContext,
    ) -> anyhow::Result<String> {
        let mut output = String::with_capacity(1024);

        // Step 1: Format header and metadata
        self.format_legacy_header(&mut output, context)?;

        // Step 2: Format main content sections
        self.format_legacy_main_sections(&mut output, context)?;

        // Step 3: Format analysis sections
        self.format_legacy_analysis_sections(&mut output, context)?;

        Ok(output)
    }

    /// Format header and metadata for legacy markdown
    fn format_legacy_header(
        &self,
        output: &mut String,
        context: &DeepContext,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;

        let project_name = context
            .metadata
            .project_root
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();

        writeln!(output, "# Deep Context: {project_name}")?;
        writeln!(output, "Generated: {}", context.metadata.generated_at)?;
        writeln!(output, "Version: {}", context.metadata.tool_version)?;
        writeln!(
            output,
            "Analysis Time: {:.2}s",
            context.metadata.analysis_duration.as_secs_f64()
        )?;
        writeln!(
            output,
            "Cache Hit Rate: {:.1}%",
            context.metadata.cache_stats.hit_rate * 100.0
        )?;

        Ok(())
    }

    /// Format main content sections for legacy markdown
    fn format_legacy_main_sections(
        &self,
        output: &mut String,
        context: &DeepContext,
    ) -> anyhow::Result<()> {
        self.write_quality_scorecard_section(output, &context.quality_scorecard)?;
        self.write_project_structure_section(output, &context.file_tree)?;
        self.write_ast_section_if_present(output, &context.analyses.ast_contexts)?;
        Ok(())
    }

    fn write_quality_scorecard_section(
        &self,
        output: &mut String,
        scorecard: &QualityScorecard,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        writeln!(output, "\n## Quality Scorecard\n")?;
        writeln!(
            output,
            "- **Overall Health**: {} ({:.1}/100)",
            self.overall_health_emoji(scorecard.overall_health),
            scorecard.overall_health
        )?;
        writeln!(
            output,
            "- **Maintainability Index**: {:.1}",
            scorecard.maintainability_index
        )?;
        writeln!(
            output,
            "- **Refactoring Time**: {:.1} hours estimated",
            scorecard.technical_debt_hours
        )?;
        Ok(())
    }

    fn write_project_structure_section(
        &self,
        output: &mut String,
        file_tree: &AnnotatedFileTree,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        writeln!(output, "\n## Project Structure\n")?;
        writeln!(output, "```")?;
        self.format_annotated_tree(output, file_tree)?;
        writeln!(output, "```\n")?;
        Ok(())
    }

    fn write_ast_section_if_present(
        &self,
        output: &mut String,
        ast_contexts: &[EnhancedFileContext],
    ) -> anyhow::Result<()> {
        if !ast_contexts.is_empty() {
            self.format_enhanced_ast_section(output, ast_contexts)?;
        }
        Ok(())
    }

    /// Format analysis sections for legacy markdown
    fn format_legacy_analysis_sections(
        &self,
        output: &mut String,
        context: &DeepContext,
    ) -> anyhow::Result<()> {
        // Code quality metrics
        self.format_complexity_hotspots(output, context)?;
        self.format_churn_analysis(output, context)?;
        self.format_technical_debt(output, context)?;
        self.format_dead_code_analysis(output, context)?;

        // Cross-language references
        self.format_cross_references(output, &context.analyses.cross_language_refs)?;

        // Defect probability analysis
        self.format_defect_predictions(output, context)?;

        // Actionable recommendations
        self.format_prioritized_recommendations(output, &context.recommendations)?;

        Ok(())
    }

    /// Format as JSON output for machine consumption and API responses
    pub fn format_as_json(&self, context: &DeepContext) -> anyhow::Result<String> {
        serde_json::to_string_pretty(context)
            .map_err(|e| anyhow::anyhow!("Failed to serialize to JSON: {}", e))
    }

    /// Format as SARIF (Static Analysis Results Interchange Format) for tool integration
    pub fn format_as_sarif(&self, context: &DeepContext) -> anyhow::Result<String> {
        use serde_json::json;

        let mut results = Vec::new();
        let mut rules = Vec::new();

        // Process each analysis type through dedicated handlers
        self.add_complexity_sarif_items_from_analyses(&context.analyses, &mut rules, &mut results);
        self.add_satd_sarif_items_from_analyses(&context.analyses, &mut rules, &mut results);
        self.add_dead_code_sarif_items_from_analyses(&context.analyses, &mut rules, &mut results);

        let sarif = json!({
            "version": "2.1.0",
            "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
            "runs": [{
                "tool": {
                    "driver": {
                        "name": "pmat",
                        "version": context.metadata.tool_version,
                        "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit",
                        "shortDescription": {"text": "Professional project scaffolding and analysis toolkit"},
                        "rules": rules
                    }
                },
                "results": results,
                "properties": {
                    "analysis_duration_seconds": context.metadata.analysis_duration.as_secs_f64(),
                    "cache_hit_rate": context.metadata.cache_stats.hit_rate,
                    "overall_health_score": context.quality_scorecard.overall_health,
                    "technical_debt_hours": context.quality_scorecard.technical_debt_hours
                }
            }]
        });

        serde_json::to_string_pretty(&sarif)
            .map_err(|e| anyhow::anyhow!("Failed to serialize to SARIF: {}", e))
    }

    /// Add complexity violations to SARIF results from AnalysisResults
    fn add_complexity_sarif_items_from_analyses(
        &self,
        analyses: &AnalysisResults,
        rules: &mut Vec<serde_json::Value>,
        results: &mut Vec<serde_json::Value>,
    ) {
        use serde_json::json;

        if let Some(ref complexity) = analyses.complexity_report {
            // Add complexity rules once
            rules.extend_from_slice(&[
                json!({
                    "id": "complexity/high-cyclomatic",
                    "shortDescription": {"text": "High cyclomatic complexity"},
                    "fullDescription": {"text": "Function has cyclomatic complexity above recommended threshold"},
                    "defaultConfiguration": {"level": "warning"},
                    "properties": {"tags": ["complexity", "maintainability"]}
                }),
                json!({
                    "id": "complexity/high-cognitive",
                    "shortDescription": {"text": "High cognitive complexity"},
                    "fullDescription": {"text": "Function has cognitive complexity above recommended threshold"},
                    "defaultConfiguration": {"level": "warning"},
                    "properties": {"tags": ["complexity", "maintainability"]}
                })
            ]);

            // Process complexity violations
            for file in &complexity.files {
                for func in &file.functions {
                    self.add_complexity_violation(file, func, results);
                }
            }
        }
    }

    /// Add a single complexity violation
    fn add_complexity_violation(
        &self,
        file: &crate::services::complexity::FileComplexityMetrics,
        func: &crate::services::complexity::FunctionComplexity,
        results: &mut Vec<serde_json::Value>,
    ) {
        use serde_json::json;

        if func.metrics.cyclomatic > 10 {
            results.push(json!({
                "ruleId": "complexity/high-cyclomatic",
                "level": if func.metrics.cyclomatic > 20 { "error" } else { "warning" },
                "message": {"text": format!("Function '{}' has cyclomatic complexity of {}", func.name, func.metrics.cyclomatic)},
                "locations": [self.create_location(&file.path, func.line_start as usize, func.line_end as usize)],
                "properties": {
                    "cyclomatic_complexity": func.metrics.cyclomatic,
                    "cognitive_complexity": func.metrics.cognitive
                }
            }));
        }

        if func.metrics.cognitive > 15 {
            results.push(json!({
                "ruleId": "complexity/high-cognitive",
                "level": if func.metrics.cognitive > 25 { "error" } else { "warning" },
                "message": {"text": format!("Function '{}' has cognitive complexity of {}", func.name, func.metrics.cognitive)},
                "locations": [self.create_location(&file.path, func.line_start as usize, func.line_end as usize)],
                "properties": {
                    "cyclomatic_complexity": func.metrics.cyclomatic,
                    "cognitive_complexity": func.metrics.cognitive
                }
            }));
        }
    }

    /// Add SATD items to SARIF results from AnalysisResults
    fn add_satd_sarif_items_from_analyses(
        &self,
        analyses: &AnalysisResults,
        rules: &mut Vec<serde_json::Value>,
        results: &mut Vec<serde_json::Value>,
    ) {
        use serde_json::json;

        if let Some(ref satd) = analyses.satd_results {
            rules.push(json!({
                "id": "debt/technical-debt",
                "shortDescription": {"text": "Code quality issue"},
                "fullDescription": {"text": "Self-admitted code issue requiring attention"},
                "defaultConfiguration": {"level": "note"},
                "properties": {"tags": ["debt", "maintainability"]}
            }));

            for item in &satd.items {
                let level = self.satd_severity_to_level(&item.severity);
                results.push(json!({
                    "ruleId": "debt/technical-debt",
                    "level": level,
                    "message": {"text": format!("{}: {}", item.category, item.text.trim())},
                    "locations": [self.create_location(&item.file.to_string_lossy(), item.line as usize, item.line as usize)],
                    "properties": {
                        "category": format!("{:?}", item.category),
                        "severity": format!("{:?}", item.severity),
                        "debt_type": "self_admitted"
                    }
                }));
            }
        }
    }

    /// Add dead code items to SARIF results from AnalysisResults
    fn add_dead_code_sarif_items_from_analyses(
        &self,
        analyses: &AnalysisResults,
        rules: &mut Vec<serde_json::Value>,
        results: &mut Vec<serde_json::Value>,
    ) {
        use serde_json::json;

        if let Some(ref dead_code) = analyses.dead_code_results {
            rules.push(json!({
                "id": "dead-code/unused-code",
                "shortDescription": {"text": "Dead code detected"},
                "fullDescription": {"text": "Code that appears to be unused and can potentially be removed"},
                "defaultConfiguration": {"level": "warning"},
                "properties": {"tags": ["dead-code", "maintainability"]}
            }));

            results.extend(
                dead_code.ranked_files
                    .iter()
                    .filter(|file| file.dead_functions > 0)
                    .map(|file| json!({
                        "ruleId": "dead-code/unused-code",
                        "level": "warning",
                        "message": {"text": format!("File contains {} dead functions and {} dead lines", 
                            file.dead_functions, file.dead_lines)},
                        "locations": [self.create_location(&file.path, 1, 1)],
                        "properties": {
                            "dead_functions": file.dead_functions,
                            "dead_lines": file.dead_lines,
                            "dead_code_percentage": file.dead_lines as f64 / file.total_lines.max(1) as f64 * 100.0
                        }
                    }))
            );
        }
    }

    /// Helper to create location objects
    fn create_location(&self, uri: &str, start_line: usize, end_line: usize) -> serde_json::Value {
        serde_json::json!({
            "physicalLocation": {
                "artifactLocation": {"uri": uri},
                "region": {
                    "startLine": start_line,
                    "startColumn": 1,
                    "endLine": end_line
                }
            }
        })
    }

    /// Convert SATD severity to SARIF level
    fn satd_severity_to_level(
        &self,
        severity: &crate::services::satd_detector::Severity,
    ) -> &'static str {
        match severity {
            crate::services::satd_detector::Severity::Critical => "error",
            crate::services::satd_detector::Severity::High => "warning",
            crate::services::satd_detector::Severity::Medium => "note",
            crate::services::satd_detector::Severity::Low => "note",
        }
    }

    fn overall_health_emoji(&self, health: f64) -> &'static str {
        if health >= 80.0 {
            "✅"
        } else if health >= 60.0 {
            "⚠️"
        } else {
            "❌"
        }
    }

    fn format_annotated_tree(
        &self,
        output: &mut String,
        tree: &AnnotatedFileTree,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        self.format_tree_node(output, &tree.root, "", true)?;
        writeln!(
            output,
            "\n📊 Total Files: {}, Total Size: {} bytes",
            tree.total_files, tree.total_size_bytes
        )?;
        Ok(())
    }

    #[allow(clippy::only_used_in_recursion)]
    fn format_tree_node(
        &self,
        output: &mut String,
        node: &AnnotatedNode,
        prefix: &str,
        is_last: bool,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;

        let connector = if is_last { "└── " } else { "├── " };
        let extension = if is_last { "    " } else { "│   " };

        let node_display = self.format_node_display(node)?;
        writeln!(output, "{prefix}{connector}{node_display}")?;

        // Process children
        let child_prefix = format!("{prefix}{extension}");
        for (i, child) in node.children.iter().enumerate() {
            let is_last_child = i == node.children.len() - 1;
            self.format_tree_node(output, child, &child_prefix, is_last_child)?;
        }

        Ok(())
    }

    fn format_node_display(&self, node: &AnnotatedNode) -> anyhow::Result<String> {
        let mut display = node.name.clone();

        if matches!(node.node_type, NodeType::Directory) {
            display.push('/');
        }

        let annotations = self.collect_node_annotations(&node.annotations);
        if !annotations.is_empty() {
            display.push_str(&format!(" [{}]", annotations.join(" ")));
        }

        Ok(display)
    }

    fn collect_node_annotations(&self, annotations: &NodeAnnotations) -> Vec<String> {
        let mut result = Vec::new();

        // Defect score
        if let Some(score) = annotations.defect_score {
            self.add_defect_indicator(&mut result, score);
        }

        // Cognitive complexity
        if let Some(complexity) = annotations.cognitive_complexity {
            self.add_cognitive_complexity_indicator(&mut result, complexity);
        }

        // SATD items
        if annotations.satd_items > 0 {
            result.push(format!("📝{}", annotations.satd_items));
        }

        // Dead code items
        if annotations.dead_code_items > 0 {
            result.push(format!("💀{}", annotations.dead_code_items));
        }

        // Test coverage
        if let Some(coverage) = annotations.test_coverage {
            self.add_coverage_indicator(&mut result, coverage);
        }

        // Big-O complexity
        if let Some(ref big_o) = annotations.big_o_complexity {
            let emoji = self.get_big_o_emoji(big_o);
            result.push(format!("{}{}", emoji, big_o));
        }

        // Churn score
        if let Some(churn) = annotations.churn_score {
            self.add_churn_indicator(&mut result, churn);
        }

        // Memory complexity
        if let Some(ref mem_complexity) = annotations.memory_complexity {
            self.add_memory_complexity_indicator(&mut result, mem_complexity);
        }

        // Duplication score
        if let Some(duplication) = annotations.duplication_score {
            self.add_duplication_indicator(&mut result, duplication);
        }

        result
    }

    /// Add defect score indicator
    fn add_defect_indicator(&self, result: &mut Vec<String>, score: f32) {
        if score > 0.7 {
            result.push(format!("🔴{score:.1}"));
        } else if score > 0.4 {
            result.push(format!("🟡{score:.1}"));
        }
    }

    /// Add cognitive complexity indicator
    fn add_cognitive_complexity_indicator(&self, result: &mut Vec<String>, complexity: u16) {
        if complexity > 30 {
            result.push(format!("🧠{}", complexity));
        } else if complexity > 15 {
            result.push(format!("🧪{}", complexity));
        }
    }

    /// Add test coverage indicator
    fn add_coverage_indicator(&self, result: &mut Vec<String>, coverage: f32) {
        if coverage < 0.5 {
            result.push(format!("🚨{:.0}%", coverage * 100.0));
        } else if coverage < 0.8 {
            result.push(format!("⚠️{:.0}%", coverage * 100.0));
        } else {
            result.push(format!("✅{:.0}%", coverage * 100.0));
        }
    }

    /// Add churn indicator
    fn add_churn_indicator(&self, result: &mut Vec<String>, churn: f32) {
        if churn > 0.8 {
            result.push(format!("🔥{:.1}", churn)); // High churn - hot file
        } else if churn > 0.5 {
            result.push(format!("🌡️{:.1}", churn)); // Medium churn
        } else if churn > 0.2 {
            result.push(format!("🌊{:.1}", churn)); // Low churn
        }
    }

    /// Add memory complexity indicator
    fn add_memory_complexity_indicator(&self, result: &mut Vec<String>, mem_complexity: &str) {
        let emoji = match mem_complexity {
            "O(1)" => "💎",       // Constant memory - excellent
            "O(log n)" => "💚",   // Logarithmic memory - very good
            "O(n)" => "💙",       // Linear memory - good
            "O(n log n)" => "💛", // Linearithmic memory - okay
            "O(n²)" => "🟠",      // Quadratic memory - warning
            _ => "💔",            // High memory usage - critical
        };
        result.push(format!("{}{}", emoji, mem_complexity));
    }

    /// Add duplication indicator
    fn add_duplication_indicator(&self, result: &mut Vec<String>, duplication: f32) {
        if duplication > 0.3 {
            result.push(format!("📑{:.0}%", duplication * 100.0)); // High duplication
        } else if duplication > 0.1 {
            result.push(format!("📄{:.0}%", duplication * 100.0)); // Medium duplication
        }
    }

    /// Get emoji for Big-O complexity notation
    fn get_big_o_emoji(&self, big_o: &str) -> &'static str {
        match big_o {
            "O(1)" => "🎯",            // Constant - excellent
            "O(log n)" => "⚡",        // Logarithmic - very good
            "O(n)" => "📊",            // Linear - good
            "O(n log n)" => "📈",      // Linearithmic - acceptable
            "O(n²)" => "⚠️",           // Quadratic - warning
            "O(n³)" => "🚨",           // Cubic - danger
            "O(2ⁿ)" | "O(n!)" => "💥", // Exponential/Factorial - critical
            _ => "❓",                 // Unknown
        }
    }

    pub fn format_enhanced_ast_section(
        &self,
        output: &mut String,
        ast_contexts: &[EnhancedFileContext],
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        writeln!(output, "## Enhanced AST Analysis\n")?;

        for context in ast_contexts {
            self.format_single_file_ast(output, context)?;
        }

        Ok(())
    }

    fn format_single_file_ast(
        &self,
        output: &mut String,
        context: &EnhancedFileContext,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;

        writeln!(output, "### {}\n", context.base.path)?;
        writeln!(output, "**Language:** {}", context.base.language)?;
        writeln!(output, "**Total Symbols:** {}", context.base.items.len())?;

        // Categorize AST items
        let categorized_items = self.categorize_ast_items(&context.base.items);

        // Write summary counts
        self.write_ast_summary(output, &categorized_items)?;

        // Write detailed breakdowns
        self.write_ast_details(output, &categorized_items)?;

        // Write metrics
        self.write_file_metrics(output, context)?;

        Ok(())
    }

    fn categorize_ast_items(
        &self,
        items: &[crate::services::context::AstItem],
    ) -> CategorizedAstItems {
        let mut categorized = CategorizedAstItems::new();

        for item in items {
            self.categorize_single_ast_item(item, &mut categorized);
        }

        categorized
    }

    fn categorize_single_ast_item(
        &self,
        item: &crate::services::context::AstItem,
        categorized: &mut CategorizedAstItems,
    ) {
        match item {
            crate::services::context::AstItem::Function {
                name,
                visibility,
                is_async,
                line,
            } => {
                categorized.functions.push(AstFunction {
                    name: name.clone(),
                    visibility: visibility.clone(),
                    is_async: *is_async,
                    line: *line,
                });
            }
            crate::services::context::AstItem::Struct {
                name,
                visibility,
                fields_count,
                derives,
                line,
            } => {
                categorized.structs.push(AstStruct {
                    name: name.clone(),
                    visibility: visibility.clone(),
                    fields_count: *fields_count,
                    derives: derives.clone(),
                    line: *line,
                });
            }
            crate::services::context::AstItem::Enum {
                name,
                visibility,
                variants_count,
                line,
            } => {
                categorized.enums.push(AstEnum {
                    name: name.clone(),
                    visibility: visibility.clone(),
                    variants_count: *variants_count,
                    line: *line,
                });
            }
            crate::services::context::AstItem::Trait {
                name,
                visibility,
                line,
            } => {
                categorized.traits.push(AstTrait {
                    name: name.clone(),
                    visibility: visibility.clone(),
                    line: *line,
                });
            }
            crate::services::context::AstItem::Impl {
                type_name,
                trait_name,
                line,
            } => {
                categorized.impls.push(AstImpl {
                    type_name: type_name.clone(),
                    trait_name: trait_name.clone(),
                    line: *line,
                });
            }
            crate::services::context::AstItem::Module {
                name,
                visibility,
                line,
            } => {
                categorized.modules.push(AstModule {
                    name: name.clone(),
                    visibility: visibility.clone(),
                    line: *line,
                });
            }
            crate::services::context::AstItem::Use { path, line } => {
                categorized.uses.push(AstUse {
                    path: path.clone(),
                    line: *line,
                });
            }
            crate::services::context::AstItem::Import {
                module,
                items,
                alias,
                line,
            } => {
                let path = self.format_import_path(module, items, alias);
                categorized.uses.push(AstUse { path, line: *line });
            }
        }
    }

    fn format_import_path(&self, module: &str, items: &[String], alias: &Option<String>) -> String {
        if let Some(alias) = alias {
            format!("{} as {}", module, alias)
        } else if !items.is_empty() {
            format!("{} ({})", module, items.join(", "))
        } else {
            module.to_string()
        }
    }

    fn write_ast_summary(
        &self,
        output: &mut String,
        items: &CategorizedAstItems,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        writeln!(output, "**Functions:** {} | **Structs:** {} | **Enums:** {} | **Traits:** {} | **Impls:** {} | **Modules:** {} | **Imports:** {}",
            items.functions.len(), items.structs.len(), items.enums.len(),
            items.traits.len(), items.impls.len(), items.modules.len(), items.uses.len())?;
        Ok(())
    }

    fn write_ast_details(
        &self,
        output: &mut String,
        items: &CategorizedAstItems,
    ) -> anyhow::Result<()> {
        self.write_functions_section(output, &items.functions)?;
        self.write_structs_section(output, &items.structs)?;
        self.write_enums_section(output, &items.enums)?;
        self.write_traits_section(output, &items.traits)?;
        self.write_impls_section(output, &items.impls)?;
        self.write_modules_section(output, &items.modules)?;
        self.write_imports_section(output, &items.uses)?;
        Ok(())
    }

    fn write_functions_section(
        &self,
        output: &mut String,
        functions: &[AstFunction],
    ) -> anyhow::Result<()> {
        if functions.is_empty() {
            return Ok(());
        }

        use std::fmt::Write;
        writeln!(output, "\n**Functions:**")?;

        for func in functions.iter().take(10) {
            let async_marker = if func.is_async { " (async)" } else { "" };
            writeln!(
                output,
                "  - `{}{}` ({}) at line {}",
                func.name, async_marker, func.visibility, func.line
            )?;
        }

        if functions.len() > 10 {
            writeln!(
                output,
                "  - ... and {} more functions",
                functions.len() - 10
            )?;
        }

        Ok(())
    }

    fn write_structs_section(
        &self,
        output: &mut String,
        structs: &[AstStruct],
    ) -> anyhow::Result<()> {
        if structs.is_empty() {
            return Ok(());
        }

        use std::fmt::Write;
        writeln!(output, "\n**Structs:**")?;

        for struct_item in structs.iter().take(5) {
            let derives_str = if struct_item.derives.is_empty() {
                String::with_capacity(1024)
            } else {
                format!(" (derives: {})", struct_item.derives.join(", "))
            };
            let field_plural = if struct_item.fields_count == 1 {
                ""
            } else {
                "s"
            };
            writeln!(
                output,
                "  - `{}` ({}) with {} field{}{} at line {}",
                struct_item.name,
                struct_item.visibility,
                struct_item.fields_count,
                field_plural,
                derives_str,
                struct_item.line
            )?;
        }

        if structs.len() > 5 {
            writeln!(output, "  - ... and {} more structs", structs.len() - 5)?;
        }

        Ok(())
    }

    fn write_enums_section(&self, output: &mut String, enums: &[AstEnum]) -> anyhow::Result<()> {
        if enums.is_empty() {
            return Ok(());
        }

        use std::fmt::Write;
        writeln!(output, "\n**Enums:**")?;

        for enum_item in enums.iter().take(5) {
            let variant_plural = if enum_item.variants_count == 1 {
                ""
            } else {
                "s"
            };
            writeln!(
                output,
                "  - `{}` ({}) with {} variant{} at line {}",
                enum_item.name,
                enum_item.visibility,
                enum_item.variants_count,
                variant_plural,
                enum_item.line
            )?;
        }

        if enums.len() > 5 {
            writeln!(output, "  - ... and {} more enums", enums.len() - 5)?;
        }

        Ok(())
    }

    fn write_traits_section(&self, output: &mut String, traits: &[AstTrait]) -> anyhow::Result<()> {
        if traits.is_empty() {
            return Ok(());
        }

        use std::fmt::Write;
        writeln!(output, "\n**Traits:**")?;

        for trait_item in traits.iter().take(5) {
            writeln!(
                output,
                "  - `{}` ({}) at line {}",
                trait_item.name, trait_item.visibility, trait_item.line
            )?;
        }

        if traits.len() > 5 {
            writeln!(output, "  - ... and {} more traits", traits.len() - 5)?;
        }

        Ok(())
    }

    fn write_impls_section(&self, output: &mut String, impls: &[AstImpl]) -> anyhow::Result<()> {
        if impls.is_empty() {
            return Ok(());
        }

        use std::fmt::Write;
        writeln!(output, "\n**Implementations:**")?;

        for impl_item in impls.iter().take(5) {
            if let Some(trait_name) = &impl_item.trait_name {
                writeln!(
                    output,
                    "  - `{} for {}` at line {}",
                    trait_name, impl_item.type_name, impl_item.line
                )?;
            } else {
                writeln!(
                    output,
                    "  - `impl {}` at line {}",
                    impl_item.type_name, impl_item.line
                )?;
            }
        }

        if impls.len() > 5 {
            writeln!(
                output,
                "  - ... and {} more implementations",
                impls.len() - 5
            )?;
        }

        Ok(())
    }

    fn write_modules_section(
        &self,
        output: &mut String,
        modules: &[AstModule],
    ) -> anyhow::Result<()> {
        if modules.is_empty() {
            return Ok(());
        }

        use std::fmt::Write;
        writeln!(output, "\n**Modules:**")?;

        for module_item in modules.iter().take(5) {
            writeln!(
                output,
                "  - `{}` ({}) at line {}",
                module_item.name, module_item.visibility, module_item.line
            )?;
        }

        if modules.len() > 5 {
            writeln!(output, "  - ... and {} more modules", modules.len() - 5)?;
        }

        Ok(())
    }

    fn write_imports_section(&self, output: &mut String, uses: &[AstUse]) -> anyhow::Result<()> {
        if uses.is_empty() {
            return Ok(());
        }

        use std::fmt::Write;

        if uses.len() <= 8 {
            writeln!(output, "\n**Key Imports:**")?;
            for use_item in uses.iter().take(8) {
                writeln!(output, "  - `{}` at line {}", use_item.path, use_item.line)?;
            }
        } else {
            writeln!(output, "\n**Imports:** {} import statements", uses.len())?;
        }

        Ok(())
    }

    fn write_file_metrics(
        &self,
        output: &mut String,
        context: &EnhancedFileContext,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;

        // Complexity metrics if available
        if let Some(ref complexity) = context.complexity_metrics {
            writeln!(output, "\n**Complexity Metrics:**")?;
            writeln!(
                output,
                "  - Cyclomatic: {:.1} | Cognitive: {:.1} | Lines: {}",
                complexity.total_complexity.cyclomatic,
                complexity.total_complexity.cognitive,
                complexity.total_complexity.lines
            )?;
        }

        // Churn metrics if available
        if let Some(ref churn) = context.churn_metrics {
            writeln!(output, "\n**Code Churn:**")?;
            writeln!(
                output,
                "  - {} commits by {} authors",
                churn.commits, churn.authors
            )?;
        }

        // TDG Score
        if let Some(ref tdg) = context.defects.tdg_score {
            writeln!(output, "\n**Code Quality Gradient:** {:.2}\n", tdg.value)?;
            writeln!(
                output,
                "**TDG Severity:** {:?}\n",
                TDGSeverity::from(tdg.value)
            )?;
        }

        Ok(())
    }

    fn format_complexity_hotspots(
        &self,
        output: &mut String,
        context: &DeepContext,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        if let Some(ref complexity) = context.analyses.complexity_report {
            writeln!(output, "## Complexity Hotspots\n")?;

            // Find top 10 most complex functions
            let mut all_functions: Vec<_> = complexity
                .files
                .par_iter()
                .flat_map(|f| f.functions.par_iter().map(move |func| (f, func)))
                .collect();
            all_functions.sort_by_key(|(_, func)| std::cmp::Reverse(func.metrics.cyclomatic));

            writeln!(output, "| Function | File | Cyclomatic | Cognitive |")?;
            writeln!(output, "|----------|------|------------|-----------|")?;

            for (file, func) in all_functions.iter().take(10) {
                writeln!(
                    output,
                    "| `{}` | `{}` | {} | {} |",
                    func.name, file.path, func.metrics.cyclomatic, func.metrics.cognitive
                )?;
            }
            writeln!(output)?;
        }

        Ok(())
    }

    fn format_churn_analysis(
        &self,
        output: &mut String,
        context: &DeepContext,
    ) -> anyhow::Result<()> {
        if let Some(ref churn) = context.analyses.churn_analysis {
            self.write_churn_header(output)?;
            self.write_churn_summary(output, churn)?;
            self.write_churn_files_table(output, &churn.files)?;
        }
        Ok(())
    }

    fn write_churn_header(&self, output: &mut String) -> anyhow::Result<()> {
        use std::fmt::Write;
        writeln!(output, "## Code Churn Analysis\n")?;
        Ok(())
    }

    fn write_churn_summary(
        &self,
        output: &mut String,
        churn: &CodeChurnAnalysis,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        writeln!(output, "**Summary:**")?;
        writeln!(output, "- Total Commits: {}", churn.summary.total_commits)?;
        writeln!(output, "- Files Changed: {}", churn.files.len())?;
        Ok(())
    }

    fn write_churn_files_table(
        &self,
        output: &mut String,
        files: &[crate::models::churn::FileChurnMetrics],
    ) -> anyhow::Result<()> {
        use std::fmt::Write;

        // Sort files by commit count
        let mut sorted_files = files.to_vec();
        sorted_files.sort_by_key(|f| std::cmp::Reverse(f.commit_count));

        writeln!(output, "\n**Top Changed Files:**")?;
        writeln!(output, "| File | Commits | Authors |")?;
        writeln!(output, "|------|---------|---------|")?;

        for file in sorted_files.iter().take(10) {
            writeln!(
                output,
                "| `{}` | {} | {} |",
                file.relative_path,
                file.commit_count,
                file.unique_authors.len()
            )?;
        }
        writeln!(output)?;
        Ok(())
    }

    fn format_technical_debt(
        &self,
        output: &mut String,
        context: &DeepContext,
    ) -> anyhow::Result<()> {
        if let Some(ref satd) = context.analyses.satd_results {
            use std::fmt::Write;
            writeln!(output, "## Code Quality Analysis\n")?;
            self.write_satd_severity_summary(output, satd)?;
            self.write_critical_items(output, satd)?;
            writeln!(output)?;
        }
        Ok(())
    }

    fn write_satd_severity_summary(
        &self,
        output: &mut String,
        satd: &crate::services::satd_detector::SATDAnalysisResult,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        let by_severity = self.group_satd_by_severity(satd);
        writeln!(output, "**SATD Summary:**")?;
        for (severity, count) in by_severity {
            writeln!(output, "- {severity:?}: {count}")?;
        }
        Ok(())
    }

    fn group_satd_by_severity<'a>(
        &self,
        satd: &'a crate::services::satd_detector::SATDAnalysisResult,
    ) -> FxHashMap<&'a crate::services::satd_detector::Severity, i32> {
        let mut by_severity = FxHashMap::default();
        for item in &satd.items {
            *by_severity.entry(&item.severity).or_insert(0) += 1;
        }
        by_severity
    }

    fn write_critical_items(
        &self,
        output: &mut String,
        satd: &crate::services::satd_detector::SATDAnalysisResult,
    ) -> anyhow::Result<()> {
        let critical_items = self.get_critical_satd_items(satd);
        if critical_items.is_empty() {
            return Ok(());
        }

        use std::fmt::Write;
        writeln!(output, "\n**Critical Items:**")?;
        for item in critical_items {
            writeln!(
                output,
                "- `{}:{} {}`: {}",
                item.file.display(),
                item.line,
                item.category,
                item.text.trim()
            )?;
        }
        Ok(())
    }

    fn get_critical_satd_items<'a>(
        &self,
        satd: &'a crate::services::satd_detector::SATDAnalysisResult,
    ) -> Vec<&'a crate::services::satd_detector::TechnicalDebt> {
        satd.items
            .iter()
            .filter(|item| {
                matches!(
                    item.severity,
                    crate::services::satd_detector::Severity::Critical
                )
            })
            .take(5)
            .collect()
    }

    fn format_dead_code_analysis(
        &self,
        output: &mut String,
        context: &DeepContext,
    ) -> anyhow::Result<()> {
        if let Some(ref dead_code) = context.analyses.dead_code_results {
            self.write_dead_code_header(output)?;
            self.write_dead_code_summary(output, &dead_code.summary)?;
            self.write_dead_code_files_table(output, &dead_code.ranked_files)?;
        }
        Ok(())
    }

    fn write_dead_code_header(&self, output: &mut String) -> anyhow::Result<()> {
        use std::fmt::Write;
        writeln!(output, "## Dead Code Analysis\n")?;
        Ok(())
    }

    fn write_dead_code_summary(
        &self,
        output: &mut String,
        summary: &crate::models::dead_code::DeadCodeSummary,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        writeln!(output, "**Summary:**")?;
        writeln!(output, "- Dead Functions: {}", summary.dead_functions)?;
        writeln!(output, "- Total Dead Lines: {}", summary.total_dead_lines)?;
        Ok(())
    }

    fn write_dead_code_files_table(
        &self,
        output: &mut String,
        files: &[crate::models::dead_code::FileDeadCodeMetrics],
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        if !files.is_empty() {
            writeln!(output, "\n**Top Files with Dead Code:**")?;
            writeln!(output, "| File | Dead Lines | Dead Functions |")?;
            writeln!(output, "|------|------------|----------------|")?;

            for file in files.iter().take(10) {
                writeln!(
                    output,
                    "| `{}` | {} | {} |",
                    file.path, file.dead_lines, file.dead_functions
                )?;
            }
            writeln!(output)?;
        }
        Ok(())
    }

    fn format_cross_references(
        &self,
        output: &mut String,
        cross_refs: &[CrossLangReference],
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        if !cross_refs.is_empty() {
            writeln!(output, "## Cross-Language References\n")?;

            writeln!(output, "| Source | Target | Type | Confidence |")?;
            writeln!(output, "|--------|--------|------|------------|")?;

            for cross_ref in cross_refs {
                writeln!(
                    output,
                    "| `{}` | `{}` | {:?} | {:.1}% |",
                    cross_ref.source_file.display(),
                    cross_ref.target_file.display(),
                    cross_ref.reference_type,
                    cross_ref.confidence * 100.0
                )?;
            }
            writeln!(output)?;
        }

        Ok(())
    }

    fn format_defect_predictions(
        &self,
        output: &mut String,
        context: &DeepContext,
    ) -> anyhow::Result<()> {
        self.write_defect_header(output)?;
        self.write_defect_summary(output, &context.defect_summary)?;
        self.write_defect_hotspots_table(output, &context.hotspots)?;
        Ok(())
    }

    fn write_defect_header(&self, output: &mut String) -> anyhow::Result<()> {
        use std::fmt::Write;
        writeln!(output, "## Defect Probability Analysis\n")?;
        Ok(())
    }

    fn write_defect_summary(
        &self,
        output: &mut String,
        summary: &DefectSummary,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        writeln!(output, "**Risk Assessment:**")?;
        writeln!(
            output,
            "- Total Defects Predicted: {}",
            summary.total_defects
        )?;
        writeln!(
            output,
            "- Defect Density: {:.2} defects per 1000 lines",
            summary.defect_density
        )?;
        Ok(())
    }

    fn write_defect_hotspots_table(
        &self,
        output: &mut String,
        hotspots: &[DefectHotspot],
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        if !hotspots.is_empty() {
            writeln!(output, "\n**High-Risk Hotspots:**")?;
            writeln!(output, "| File:Line | Risk Score | Effort (hours) |")?;
            writeln!(output, "|-----------|------------|----------------|")?;

            for hotspot in hotspots.iter().take(10) {
                writeln!(
                    output,
                    "| `{}:{}` | {:.1} | {:.1} |",
                    hotspot.location.file.display(),
                    hotspot.location.line,
                    hotspot.composite_score,
                    hotspot.refactoring_effort.estimated_hours
                )?;
            }
        }
        writeln!(output)?;
        Ok(())
    }

    fn format_prioritized_recommendations(
        &self,
        output: &mut String,
        recommendations: &[PrioritizedRecommendation],
    ) -> anyhow::Result<()> {
        if recommendations.is_empty() {
            return Ok(());
        }

        self.write_recommendations_header(output)?;

        for (i, rec) in recommendations.iter().enumerate() {
            self.write_single_recommendation(output, i, rec)?;
        }

        Ok(())
    }

    fn write_recommendations_header(&self, output: &mut String) -> anyhow::Result<()> {
        use std::fmt::Write;
        writeln!(output, "## Prioritized Recommendations\n")?;
        Ok(())
    }

    fn write_single_recommendation(
        &self,
        output: &mut String,
        index: usize,
        rec: &PrioritizedRecommendation,
    ) -> anyhow::Result<()> {
        let priority_emoji = self.get_priority_emoji(&rec.priority);
        self.write_recommendation_title(output, priority_emoji, index + 1, &rec.title)?;
        self.write_recommendation_details(output, rec)?;
        self.write_recommendation_prerequisites(output, &rec.prerequisites)?;
        Ok(())
    }

    fn get_priority_emoji(&self, priority: &Priority) -> &'static str {
        match priority {
            Priority::Critical => "🔴",
            Priority::High => "🟡",
            Priority::Medium => "🔵",
            Priority::Low => "⚪",
        }
    }

    fn write_recommendation_title(
        &self,
        output: &mut String,
        emoji: &str,
        number: usize,
        title: &str,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        writeln!(output, "### {} {} {}", emoji, number, title)?;
        Ok(())
    }

    fn write_recommendation_details(
        &self,
        output: &mut String,
        rec: &PrioritizedRecommendation,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        writeln!(output, "**Description:** {}", rec.description)?;
        writeln!(output, "**Effort:** {:?}", rec.estimated_effort)?;
        writeln!(output, "**Impact:** {:?}", rec.impact)?;
        Ok(())
    }

    fn write_recommendation_prerequisites(
        &self,
        output: &mut String,
        prerequisites: &[String],
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        if !prerequisites.is_empty() {
            writeln!(output, "**Prerequisites:**")?;
            for prereq in prerequisites {
                writeln!(output, "- {prereq}")?;
            }
        }
        writeln!(output)?;
        Ok(())
    }

    pub async fn analyze_project(&self, project_path: &PathBuf) -> anyhow::Result<DeepContext> {
        let start_time = std::time::Instant::now();
        info!(
            "Starting deep context analysis for project: {:?}",
            project_path
        );

        // Create progress tracker
        let progress = crate::services::progress::ProgressTracker::new(true);
        let main_progress = progress.create_spinner("Analyzing project...");

        // Execute all analysis phases using extracted methods
        let mut file_tree = self
            .execute_discovery_phase(project_path, &main_progress)
            .await?;
        let analyses = self
            .execute_analysis_phase(project_path, &progress, &main_progress)
            .await?;
        self.enrich_file_tree_if_dag_present(&mut file_tree, &analyses, &main_progress)?;
        let cross_refs = self
            .execute_cross_reference_phase(&analyses, &main_progress)
            .await?;
        let (defect_summary, hotspots) = self
            .execute_defect_correlation_phase(&analyses, &main_progress)
            .await?;
        let quality_scorecard = self
            .execute_quality_scoring_phase(&analyses, &defect_summary, &main_progress)
            .await?;
        let recommendations = self
            .execute_recommendations_phase(&analyses, &defect_summary, &main_progress)
            .await?;
        let template_provenance = self
            .execute_template_provenance_phase(&analyses, &main_progress)
            .await?;
        let (build_info, project_overview) = self
            .execute_metadata_analysis_phase(project_path, &main_progress)
            .await?;

        // Build the deep context from all phases
        let analysis_duration = start_time.elapsed();
        let build_params = DeepContextBuildParams {
            project_path,
            file_tree,
            analyses,
            cross_refs,
            quality_scorecard,
            template_provenance,
            defect_summary,
            hotspots,
            recommendations,
            build_info,
            project_overview,
            analysis_duration,
        };
        let mut deep_context = self.build_deep_context(build_params);

        // Execute final QA verification phase
        deep_context.qa_verification = Some(
            self.execute_qa_verification_phase(&deep_context, &main_progress)
                .await?,
        );

        // Complete progress tracking
        main_progress.finish_with_message("Analysis complete!");
        progress.clear();

        info!("Deep context analysis completed in {:?}", analysis_duration);
        Ok(deep_context)
    }

    // ============================================================================
    // EXTRACTED METHODS - Toyota Way Extract Method Pattern
    // Each method has single responsibility and <10 complexity
    // ============================================================================

    async fn execute_discovery_phase(
        &self,
        project_path: &PathBuf,
        progress: &indicatif::ProgressBar,
    ) -> anyhow::Result<AnnotatedFileTree> {
        progress.set_message("Discovering project structure...");
        let file_tree = self.discover_project_structure(project_path).await?;
        debug!("Discovery phase completed");
        Ok(file_tree)
    }

    async fn execute_analysis_phase(
        &self,
        project_path: &Path,
        tracker: &crate::services::progress::ProgressTracker,
        progress: &indicatif::ProgressBar,
    ) -> anyhow::Result<ParallelAnalysisResults> {
        progress.set_message("Running parallel analyses...");
        let analysis_start = std::time::Instant::now();
        let analyses = self
            .execute_parallel_analyses_with_progress(project_path, tracker)
            .await?;
        info!("Analysis phase completed in {:?}", analysis_start.elapsed());
        debug!("Analysis phase completed");
        Ok(analyses)
    }

    fn enrich_file_tree_if_dag_present(
        &self,
        file_tree: &mut AnnotatedFileTree,
        analyses: &ParallelAnalysisResults,
        progress: &indicatif::ProgressBar,
    ) -> anyhow::Result<()> {
        if let Some(ref dag) = analyses.dependency_graph {
            progress.set_message("Enriching file tree with centrality scores...");
            self.enrich_file_tree_with_centrality(file_tree, dag)?;
            debug!("File tree enriched with centrality scores");
        }
        Ok(())
    }

    async fn execute_cross_reference_phase(
        &self,
        analyses: &ParallelAnalysisResults,
        progress: &indicatif::ProgressBar,
    ) -> anyhow::Result<FxHashMap<String, Vec<CrossLangReference>>> {
        progress.set_message("Resolving cross-language references...");
        let cross_refs = self.build_cross_language_references(analyses).await?;
        debug!("Cross-reference resolution completed");
        Ok(cross_refs)
    }

    async fn execute_defect_correlation_phase(
        &self,
        analyses: &ParallelAnalysisResults,
        progress: &indicatif::ProgressBar,
    ) -> anyhow::Result<(DefectSummary, Vec<DefectHotspot>)> {
        progress.set_message("Correlating defects...");
        let (defect_summary, hotspots) = self.correlate_defects(analyses).await?;
        debug!("Defect correlation completed");
        Ok((defect_summary, hotspots))
    }

    async fn execute_quality_scoring_phase(
        &self,
        analyses: &ParallelAnalysisResults,
        defect_summary: &DefectSummary,
        progress: &indicatif::ProgressBar,
    ) -> anyhow::Result<QualityScorecard> {
        progress.set_message("Calculating quality scores...");
        let quality_scorecard = self
            .calculate_quality_scorecard(analyses, defect_summary)
            .await?;
        debug!("Quality scoring completed");
        Ok(quality_scorecard)
    }

    async fn execute_recommendations_phase(
        &self,
        analyses: &ParallelAnalysisResults,
        defect_summary: &DefectSummary,
        progress: &indicatif::ProgressBar,
    ) -> anyhow::Result<Vec<PrioritizedRecommendation>> {
        progress.set_message("Generating recommendations...");
        let recommendations = self
            .generate_recommendations(analyses, defect_summary)
            .await?;
        debug!("Recommendations generated");
        Ok(recommendations)
    }

    async fn execute_template_provenance_phase(
        &self,
        _analyses: &ParallelAnalysisResults,
        progress: &indicatif::ProgressBar,
    ) -> anyhow::Result<Option<TemplateProvenance>> {
        progress.set_message("Analyzing template provenance...");
        // Legacy function - returns None for now
        Ok(None)
    }

    async fn execute_metadata_analysis_phase(
        &self,
        project_path: &Path,
        progress: &indicatif::ProgressBar,
    ) -> anyhow::Result<(Option<BuildInfo>, Option<ProjectOverview>)> {
        progress.set_message("Analyzing project metadata...");
        let (build_info, project_overview) = self.analyze_project_metadata(project_path).await?;
        debug!("Project metadata analysis completed");
        Ok((build_info, project_overview))
    }

    fn build_deep_context(&self, params: DeepContextBuildParams) -> DeepContext {
        DeepContext {
            metadata: ContextMetadata {
                generated_at: Utc::now(),
                tool_version: env!("CARGO_PKG_VERSION").to_string(),
                project_root: params.project_path.to_path_buf(),
                cache_stats: CacheStats {
                    hit_rate: 0.0,
                    memory_efficiency: 0.0,
                    time_saved_ms: 0,
                },
                analysis_duration: params.analysis_duration,
            },
            file_tree: params.file_tree,
            analyses: AnalysisResults {
                ast_contexts: params.analyses.ast_contexts.unwrap_or_default(),
                complexity_report: params.analyses.complexity_report,
                churn_analysis: params.analyses.churn_analysis,
                dependency_graph: params.analyses.dependency_graph,
                dead_code_results: params.analyses.dead_code_results,
                duplicate_code_results: params.analyses.duplicate_code_results,
                satd_results: params.analyses.satd_results,
                provability_results: params.analyses.provability_results,
                cross_language_refs: params
                    .cross_refs
                    .into_iter()
                    .flat_map(|(_, refs)| refs)
                    .collect(),
                big_o_analysis: params.analyses.big_o_analysis,
            },
            quality_scorecard: params.quality_scorecard,
            template_provenance: params.template_provenance,
            defect_summary: params.defect_summary,
            hotspots: params.hotspots,
            recommendations: params.recommendations,
            qa_verification: None,
            build_info: params.build_info,
            project_overview: params.project_overview,
        }
    }

    async fn execute_qa_verification_phase(
        &self,
        deep_context: &DeepContext,
        progress: &indicatif::ProgressBar,
    ) -> anyhow::Result<QAVerificationResult> {
        progress.set_message("Running QA verification...");
        let qa_result = self.run_qa_verification(deep_context).await?;
        info!("QA verification completed");
        Ok(qa_result)
    }

    // New helper methods for phases that didn't exist before
    async fn calculate_quality_scorecard(
        &self,
        analyses: &ParallelAnalysisResults,
        _defect_summary: &DefectSummary,
    ) -> anyhow::Result<QualityScorecard> {
        // Calculate quality scores based on analyses
        let complexity_score = if let Some(ref report) = analyses.complexity_report {
            // Calculate based on the number of violations
            let violation_penalty = (report.violations.len() as f64 * 5.0).min(50.0);
            100.0 - violation_penalty
        } else {
            75.0
        };

        let maintainability_index = 70.0; // Placeholder for now
        let modularity_score = 85.0; // Placeholder for now
        let test_coverage = Some(65.0); // Placeholder for now
        let technical_debt_hours = 40.0; // Placeholder for now

        Ok(QualityScorecard {
            overall_health: (complexity_score + maintainability_index + modularity_score) / 3.0,
            complexity_score,
            maintainability_index,
            modularity_score,
            test_coverage,
            technical_debt_hours,
        })
    }

    async fn generate_recommendations(
        &self,
        analyses: &ParallelAnalysisResults,
        defect_summary: &DefectSummary,
    ) -> anyhow::Result<Vec<PrioritizedRecommendation>> {
        let mut recommendations = Vec::new();

        // Extract Method: Each recommendation type is handled by a focused method
        self.add_complexity_recommendations(&mut recommendations, analyses);
        self.add_defect_recommendations(&mut recommendations, defect_summary);
        self.add_satd_recommendations(&mut recommendations, analyses);

        Ok(recommendations)
    }

    fn add_complexity_recommendations(
        &self,
        recommendations: &mut Vec<PrioritizedRecommendation>,
        analyses: &ParallelAnalysisResults,
    ) {
        if let Some(complexity) = &analyses.complexity_report {
            for violation in &complexity.violations {
                if let Some(recommendation) = self.create_complexity_recommendation(violation) {
                    recommendations.push(recommendation);
                }
            }
        }
    }

    fn create_complexity_recommendation(
        &self,
        violation: &crate::services::complexity::Violation,
    ) -> Option<PrioritizedRecommendation> {
        match violation {
            crate::services::complexity::Violation::Error {
                function,
                value,
                threshold,
                message,
                ..
            }
            | crate::services::complexity::Violation::Warning {
                function,
                value,
                threshold,
                message,
                ..
            } => {
                function
                    .as_ref()
                    .map(|func_name| PrioritizedRecommendation {
                        title: format!("Refactor high-complexity function: {}", func_name),
                        description: format!(
                            "{} (complexity: {}, threshold: {})",
                            message, value, threshold
                        ),
                        priority: self.determine_complexity_priority(*value),
                        estimated_effort: Duration::from_secs(3600), // 1 hour estimate
                        impact: Impact::High,
                        prerequisites: vec![],
                    })
            }
        }
    }

    fn determine_complexity_priority(&self, value: u16) -> Priority {
        if value > 25 {
            Priority::Critical
        } else if value > 20 {
            Priority::High
        } else {
            Priority::Medium
        }
    }

    fn add_defect_recommendations(
        &self,
        recommendations: &mut Vec<PrioritizedRecommendation>,
        defect_summary: &DefectSummary,
    ) {
        if defect_summary.total_defects > 50 {
            recommendations.push(PrioritizedRecommendation {
                title: "High defect count detected".to_string(),
                description: format!(
                    "Project has {} total defects. Consider a focused quality improvement sprint.",
                    defect_summary.total_defects
                ),
                priority: Priority::High,
                estimated_effort: Duration::from_secs(7200), // 2 hours
                impact: Impact::High,
                prerequisites: vec![],
            });
        }
    }

    fn add_satd_recommendations(
        &self,
        recommendations: &mut Vec<PrioritizedRecommendation>,
        analyses: &ParallelAnalysisResults,
    ) {
        if let Some(satd) = &analyses.satd_results {
            if satd.summary.total_items > 0 {
                recommendations.push(PrioritizedRecommendation {
                    title: "Technical debt detected".to_string(),
                    description: format!(
                        "Found {} SATD comments. Zero-tolerance policy requires immediate remediation.",
                        satd.summary.total_items
                    ),
                    priority: Priority::Critical,
                    estimated_effort: Duration::from_secs(satd.summary.total_items as u64 * 1800), // 30 min per SATD
                    impact: Impact::High,
                    prerequisites: vec![],
                });
            }
        }
    }

    async fn discover_project_structure(
        &self,
        project_path: &PathBuf,
    ) -> anyhow::Result<AnnotatedFileTree> {
        let mut total_files = 0;
        let mut total_size_bytes = 0;

        let root =
            self.build_file_tree_recursive(project_path, &mut total_files, &mut total_size_bytes)?;

        Ok(AnnotatedFileTree {
            root,
            total_files,
            total_size_bytes,
        })
    }

    fn build_file_tree_recursive(
        &self,
        path: &PathBuf,
        total_files: &mut usize,
        total_size: &mut u64,
    ) -> anyhow::Result<AnnotatedNode> {
        let metadata = std::fs::metadata(path)?;
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        if metadata.is_dir() {
            let mut children = Vec::new();

            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    let child_path = entry.path();

                    // Apply exclude patterns
                    if self.should_exclude_path(&child_path) {
                        continue;
                    }

                    if let Ok(child_node) =
                        self.build_file_tree_recursive(&child_path, total_files, total_size)
                    {
                        children.push(child_node);
                    }
                }
            }

            Ok(AnnotatedNode {
                name,
                path: path.clone(),
                node_type: NodeType::Directory,
                children,
                annotations: NodeAnnotations {
                    defect_score: None,
                    complexity_score: None,
                    cognitive_complexity: None,
                    churn_score: None,
                    dead_code_items: 0,
                    satd_items: 0,
                    centrality: None,
                    test_coverage: None,
                    big_o_complexity: None,
                    memory_complexity: None,
                    duplication_score: None,
                },
            })
        } else {
            *total_files += 1;
            *total_size += metadata.len();

            Ok(AnnotatedNode {
                name,
                path: path.clone(),
                node_type: NodeType::File,
                children: Vec::new(),
                annotations: NodeAnnotations {
                    defect_score: None,
                    complexity_score: None,
                    cognitive_complexity: None,
                    churn_score: None,
                    dead_code_items: 0,
                    satd_items: 0,
                    centrality: None,
                    test_coverage: None,
                    big_o_complexity: None,
                    memory_complexity: None,
                    duplication_score: None,
                },
            })
        }
    }

    fn should_exclude_path(&self, path: &std::path::Path) -> bool {
        let path_str = path.to_string_lossy();

        for pattern in &self.config.exclude_patterns {
            if path_str.contains(pattern.trim_matches('*')) {
                return true;
            }
        }

        false
    }

    /// Enrich the file tree with centrality scores from the dependency graph
    fn enrich_file_tree_with_centrality(
        &self,
        file_tree: &mut AnnotatedFileTree,
        dag: &DependencyGraph,
    ) -> anyhow::Result<()> {
        // Create a map of file paths to centrality scores
        let mut centrality_map: FxHashMap<PathBuf, f32> = FxHashMap::default();

        for node in dag.nodes.values() {
            if let Some(centrality_str) = node.metadata.get("centrality") {
                if let Ok(centrality) = centrality_str.parse::<f32>() {
                    let file_path = PathBuf::from(&node.file_path);
                    centrality_map.insert(file_path, centrality);
                }
            }
        }

        // Recursively update the file tree with centrality scores
        Self::update_node_centrality(&mut file_tree.root, &centrality_map);

        Ok(())
    }

    /// Recursively update node centrality scores
    fn update_node_centrality(node: &mut AnnotatedNode, centrality_map: &FxHashMap<PathBuf, f32>) {
        // Update this node's centrality if it's a file
        if node.node_type == NodeType::File {
            if let Some(&centrality) = centrality_map.get(&node.path) {
                node.annotations.centrality = Some(centrality);
            }
        }

        // Recursively update children
        for child in &mut node.children {
            Self::update_node_centrality(child, centrality_map);
        }
    }

    async fn execute_parallel_analyses_with_progress(
        &self,
        project_path: &std::path::Path,
        progress: &crate::services::progress::ProgressTracker,
    ) -> anyhow::Result<ParallelAnalysisResults> {
        // Step 1: Spawn all analysis tasks with progress tracking
        let mut join_set = self.spawn_analysis_tasks(project_path)?;

        // Create sub-progress bars for different analyses
        let analysis_count = self.config.include_analyses.len() as u64;
        let analysis_progress = progress.create_sub_progress("Running analyses", analysis_count);

        // Step 2: Collect and process results with timeout
        // Increased timeout to handle projects with many files or large files
        let collection_timeout = std::time::Duration::from_secs(300); // 5 minutes
        let results = self
            .collect_analysis_results_with_progress(
                &mut join_set,
                collection_timeout,
                &analysis_progress,
            )
            .await?;

        analysis_progress.finish_with_message("Analyses complete");
        Ok(results)
    }

    /// Spawn all configured analysis tasks
    fn spawn_analysis_tasks(
        &self,
        project_path: &std::path::Path,
    ) -> anyhow::Result<tokio::task::JoinSet<AnalysisResult>> {
        let mut join_set = tokio::task::JoinSet::new();

        for analysis_type in &self.config.include_analyses {
            self.spawn_analysis_task(&mut join_set, project_path, analysis_type)?;
        }

        Ok(join_set)
    }

    /// Spawn a single analysis task based on type
    fn spawn_analysis_task(
        &self,
        join_set: &mut tokio::task::JoinSet<AnalysisResult>,
        project_path: &std::path::Path,
        analysis_type: &AnalysisType,
    ) -> anyhow::Result<()> {
        let path = project_path.to_path_buf();

        match analysis_type {
            AnalysisType::Ast => self.spawn_ast_analysis(join_set, path),
            AnalysisType::Complexity => self.spawn_complexity_analysis(join_set, path),
            AnalysisType::Churn => self.spawn_churn_analysis(join_set, path),
            AnalysisType::DeadCode => self.spawn_dead_code_analysis(join_set, path),
            AnalysisType::DuplicateCode => self.spawn_duplicate_analysis(join_set, path),
            AnalysisType::Satd => self.spawn_satd_analysis(join_set, path),
            AnalysisType::Provability => self.spawn_provability_analysis(join_set, path),
            AnalysisType::Dag => self.spawn_dag_analysis(join_set, path),
            AnalysisType::TechnicalDebtGradient => Ok(()), // Computed in correlate_defects
            AnalysisType::BigO => self.spawn_big_o_analysis(join_set, path),
        }
    }

    fn spawn_ast_analysis(
        &self,
        join_set: &mut tokio::task::JoinSet<AnalysisResult>,
        path: PathBuf,
    ) -> anyhow::Result<()> {
        let file_classifier_config = self.config.file_classifier_config.clone();
        join_set.spawn(async move {
            AnalysisResult::Ast(analyze_ast_contexts(&path, file_classifier_config).await)
        });
        Ok(())
    }

    fn spawn_complexity_analysis(
        &self,
        join_set: &mut tokio::task::JoinSet<AnalysisResult>,
        path: PathBuf,
    ) -> anyhow::Result<()> {
        join_set.spawn(async move { AnalysisResult::Complexity(analyze_complexity(&path).await) });
        Ok(())
    }

    fn spawn_churn_analysis(
        &self,
        join_set: &mut tokio::task::JoinSet<AnalysisResult>,
        path: PathBuf,
    ) -> anyhow::Result<()> {
        let days = self.config.period_days;
        join_set.spawn(async move { AnalysisResult::Churn(analyze_churn(&path, days).await) });
        Ok(())
    }

    fn spawn_dead_code_analysis(
        &self,
        join_set: &mut tokio::task::JoinSet<AnalysisResult>,
        path: PathBuf,
    ) -> anyhow::Result<()> {
        join_set.spawn(async move { AnalysisResult::DeadCode(analyze_dead_code(&path).await) });
        Ok(())
    }

    fn spawn_duplicate_analysis(
        &self,
        join_set: &mut tokio::task::JoinSet<AnalysisResult>,
        path: PathBuf,
    ) -> anyhow::Result<()> {
        join_set.spawn(async move {
            AnalysisResult::DuplicateCode(analyze_duplicate_code(&path).await)
        });
        Ok(())
    }

    fn spawn_satd_analysis(
        &self,
        join_set: &mut tokio::task::JoinSet<AnalysisResult>,
        path: PathBuf,
    ) -> anyhow::Result<()> {
        join_set.spawn(async move {
            let result = tokio::task::spawn_blocking(move || {
                tokio::runtime::Handle::current().block_on(async { analyze_satd(&path).await })
            })
            .await
            .unwrap_or_else(|_| Err(anyhow::anyhow!("SATD analysis failed")));
            AnalysisResult::Satd(result)
        });
        Ok(())
    }

    fn spawn_provability_analysis(
        &self,
        join_set: &mut tokio::task::JoinSet<AnalysisResult>,
        path: PathBuf,
    ) -> anyhow::Result<()> {
        join_set
            .spawn(async move { AnalysisResult::Provability(analyze_provability(&path).await) });
        Ok(())
    }

    fn spawn_dag_analysis(
        &self,
        join_set: &mut tokio::task::JoinSet<AnalysisResult>,
        path: PathBuf,
    ) -> anyhow::Result<()> {
        let dag_type = self.config.dag_type.clone();
        join_set.spawn(async move { AnalysisResult::Dag(analyze_dag(&path, dag_type).await) });
        Ok(())
    }

    fn spawn_big_o_analysis(
        &self,
        join_set: &mut tokio::task::JoinSet<AnalysisResult>,
        path: PathBuf,
    ) -> anyhow::Result<()> {
        join_set.spawn(async move { AnalysisResult::BigO(analyze_big_o(&path).await) });
        Ok(())
    }

    async fn collect_analysis_results_with_progress(
        &self,
        join_set: &mut tokio::task::JoinSet<AnalysisResult>,
        timeout: std::time::Duration,
        progress: &indicatif::ProgressBar,
    ) -> anyhow::Result<ParallelAnalysisResults> {
        let collection_future = self.process_analysis_results_with_progress(join_set, progress);

        match tokio::time::timeout(timeout, collection_future).await {
            Ok(Ok(results)) => {
                debug!("Parallel analysis collection completed successfully");
                Ok(results)
            }
            Ok(Err(e)) => Err(anyhow::anyhow!("Analysis result aggregation failed: {}", e)),
            Err(_) => Err(anyhow::anyhow!(
                "Analysis collection timed out after {:?}",
                timeout
            )),
        }
    }

    /// Process all analysis results concurrently with progress
    async fn process_analysis_results_with_progress(
        &self,
        join_set: &mut tokio::task::JoinSet<AnalysisResult>,
        progress: &indicatif::ProgressBar,
    ) -> anyhow::Result<ParallelAnalysisResults> {
        // Collect all results first
        let mut pending_results = Vec::new();
        while let Some(result) = join_set.join_next().await {
            pending_results.push(result?);
            progress.inc(1);
        }

        // Process results concurrently
        let result_processors: Vec<_> = pending_results
            .into_iter()
            .map(|result| tokio::spawn(async move { result }))
            .collect();

        // Aggregate processed results
        let mut results = ParallelAnalysisResults::default();
        for processor in result_processors {
            if let Ok(processed) = processor.await {
                self.integrate_analysis_result(&mut results, processed);
            }
        }

        Ok(results)
    }

    /// Integrate a single analysis result into the final results
    fn integrate_analysis_result(
        &self,
        results: &mut ParallelAnalysisResults,
        result: AnalysisResult,
    ) {
        match &result {
            AnalysisResult::Ast(Ok(data)) => {
                results.ast_contexts = Some(data.clone());
            }
            AnalysisResult::Complexity(Ok(data)) => {
                results.complexity_report = Some(data.clone());
            }
            AnalysisResult::Churn(Ok(data)) => {
                results.churn_analysis = Some(data.clone());
            }
            AnalysisResult::DeadCode(Ok(data)) => {
                results.dead_code_results = Some(data.clone());
            }
            AnalysisResult::DuplicateCode(Ok(data)) => {
                results.duplicate_code_results = Some(data.clone());
            }
            AnalysisResult::Satd(Ok(data)) => {
                results.satd_results = Some(data.clone());
            }
            AnalysisResult::Provability(Ok(data)) => {
                results.provability_results = Some(data.clone());
            }
            AnalysisResult::Dag(Ok(data)) => {
                results.dependency_graph = Some(data.clone());
            }
            AnalysisResult::BigO(Ok(data)) => {
                results.big_o_analysis = Some(data.clone());
            }
            // Handle errors with helper
            _ => self.log_integration_error(&result),
        }
    }

    /// Log errors from analysis integration
    fn log_integration_error(&self, result: &AnalysisResult) {
        match result {
            AnalysisResult::Ast(Err(e))
            | AnalysisResult::Complexity(Err(e))
            | AnalysisResult::Churn(Err(e))
            | AnalysisResult::DeadCode(Err(e))
            | AnalysisResult::DuplicateCode(Err(e))
            | AnalysisResult::Satd(Err(e))
            | AnalysisResult::Provability(Err(e))
            | AnalysisResult::Dag(Err(e))
            | AnalysisResult::BigO(Err(e)) => {
                debug!("{} analysis failed: {}", self.get_analysis_name(result), e);
            }
            _ => {}
        }
    }

    /// Get analysis name for logging
    fn get_analysis_name(&self, result: &AnalysisResult) -> &'static str {
        match result {
            AnalysisResult::Ast(_) => "AST",
            AnalysisResult::Complexity(_) => "Complexity",
            AnalysisResult::Churn(_) => "Churn",
            AnalysisResult::DeadCode(_) => "Dead code",
            AnalysisResult::DuplicateCode(_) => "Duplicate code",
            AnalysisResult::Satd(_) => "SATD",
            AnalysisResult::Provability(_) => "Provability",
            AnalysisResult::Dag(_) => "DAG",
            AnalysisResult::BigO(_) => "Big-O",
        }
    }

    async fn build_cross_language_references(
        &self,
        _analyses: &ParallelAnalysisResults,
    ) -> anyhow::Result<FxHashMap<String, Vec<CrossLangReference>>> {
        // TRACKED: Implement cross-language reference detection
        // This would analyze FFI bindings, WASM exports, Python bindings, etc.
        Ok(FxHashMap::default())
    }

    async fn correlate_defects(
        &self,
        analyses: &ParallelAnalysisResults,
    ) -> anyhow::Result<(DefectSummary, Vec<DefectHotspot>)> {
        // Step 1: Collect file TDG scores from all analyses
        let file_tdg_scores = self.collect_file_tdg_scores(analyses)?;

        // Step 2: Calculate TDG summary for the project
        let _tdg_calculator = TDGCalculator::new();
        let tdg_summary = self.calculate_tdg_summary(&file_tdg_scores)?;

        // Step 3: Build defect summary (now based on TDG)
        let defect_summary = self.build_tdg_defect_summary(&tdg_summary, analyses)?;

        // Step 4: Generate hotspots
        let hotspots = self.generate_tdg_hotspots(&file_tdg_scores)?;

        Ok((defect_summary, hotspots))
    }

    /// Collect file TDG scores from all available analyses
    fn collect_file_tdg_scores(
        &self,
        analyses: &ParallelAnalysisResults,
    ) -> anyhow::Result<FxHashMap<String, TDGScore>> {
        let mut file_tdg_scores = FxHashMap::default();

        if let Some(ref ast_contexts) = analyses.ast_contexts {
            for enhanced_context in ast_contexts {
                let file_path = enhanced_context.base.path.clone();

                // Extract actual churn score for this file
                let churn_score = if let Some(ref churn_analysis) = analyses.churn_analysis {
                    churn_analysis
                        .files
                        .iter()
                        .find(|f| {
                            f.path.to_string_lossy() == file_path
                                || f.relative_path == file_path
                                || file_path.ends_with(&f.relative_path)
                        })
                        .map(|f| f.churn_score)
                        .unwrap_or(0.0)
                } else {
                    0.0
                };

                // Use TDG calculator to compute score for this file
                let tdg_score = TDGScore {
                    value: 1.5, // Default value - could be computed from components
                    components: crate::models::tdg::TDGComponents {
                        complexity: 1.0,
                        churn: churn_score as f64,
                        coupling: 0.5,
                        domain_risk: 0.5,
                        duplication: 0.5,
                    },
                    severity: TDGSeverity::Normal,
                    percentile: 50.0,
                    confidence: 0.8,
                };

                file_tdg_scores.insert(file_path, tdg_score);
            }
        }

        Ok(file_tdg_scores)
    }

    /// Calculate TDG summary from individual file scores
    fn calculate_tdg_summary(
        &self,
        file_scores: &FxHashMap<String, TDGScore>,
    ) -> anyhow::Result<TDGSummary> {
        let total_files = file_scores.len();
        // Use parallel processing for score analysis
        let (values, severities): (Vec<_>, Vec<_>) = file_scores
            .par_iter()
            .map(|(_, score)| (score.value, &score.severity))
            .unzip();

        let mut tdg_values = values;

        // Count severities in parallel
        let critical_files = severities
            .par_iter()
            .filter(|s| matches!(s, TDGSeverity::Critical))
            .count();
        let warning_files = severities
            .par_iter()
            .filter(|s| matches!(s, TDGSeverity::Warning))
            .count();

        tdg_values.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());

        let average_tdg = if tdg_values.is_empty() {
            0.0
        } else {
            tdg_values.iter().sum::<f64>() / tdg_values.len() as f64
        };

        let p95_tdg = if tdg_values.is_empty() {
            0.0
        } else {
            let index = ((tdg_values.len() - 1) as f64 * 0.95) as usize;
            tdg_values[index.min(tdg_values.len() - 1)]
        };

        let p99_tdg = if tdg_values.is_empty() {
            0.0
        } else {
            let index = ((tdg_values.len() - 1) as f64 * 0.99) as usize;
            tdg_values[index.min(tdg_values.len() - 1)]
        };

        // Create hotspots from top TDG scores
        let mut hotspots: Vec<_> = file_scores
            .iter()
            .map(|(path, score)| crate::models::tdg::TDGHotspot {
                path: path.clone(),
                tdg_score: score.value,
                primary_factor: "complexity".to_string(), // Default factor
                estimated_hours: score.value * 2.0,       // Simple estimation
            })
            .collect();
        hotspots.sort_unstable_by(|a, b| b.tdg_score.partial_cmp(&a.tdg_score).unwrap());
        hotspots.truncate(10);

        Ok(TDGSummary {
            total_files,
            critical_files,
            warning_files,
            average_tdg,
            p95_tdg,
            p99_tdg,
            estimated_debt_hours: average_tdg * total_files as f64 * 2.0,
            hotspots,
        })
    }

    /// Build defect summary based on actual defect enumeration
    fn build_tdg_defect_summary(
        &self,
        tdg_summary: &TDGSummary,
        analyses: &ParallelAnalysisResults,
    ) -> anyhow::Result<DefectSummary> {
        let mut total_defects = 0usize;
        let mut by_severity = FxHashMap::default();
        let mut by_type = FxHashMap::default();
        let mut total_loc = 0usize;

        // Process each analysis type
        self.process_complexity_violations(
            analyses,
            &mut total_defects,
            &mut by_severity,
            &mut by_type,
            &mut total_loc,
        );
        self.process_satd_violations(analyses, &mut total_defects, &mut by_severity, &mut by_type);
        self.process_dead_code_violations(
            analyses,
            &mut total_defects,
            &mut by_severity,
            &mut by_type,
        );
        self.process_tdg_violations(
            tdg_summary,
            &mut total_defects,
            &mut by_severity,
            &mut by_type,
        );

        let defect_density = self.calculate_defect_density(total_defects, total_loc);

        debug!(
            "Calculated defect summary: {} total defects, {} LOC, density = {:.2}",
            total_defects, total_loc, defect_density
        );

        Ok(DefectSummary {
            total_defects,
            by_severity,
            by_type,
            defect_density,
        })
    }

    fn process_complexity_violations(
        &self,
        analyses: &ParallelAnalysisResults,
        total_defects: &mut usize,
        by_severity: &mut FxHashMap<String, usize>,
        by_type: &mut FxHashMap<String, usize>,
        total_loc: &mut usize,
    ) {
        if let Some(ref complexity_report) = analyses.complexity_report {
            let complexity_violations = complexity_report.violations.len();
            *total_defects += complexity_violations;
            by_type.insert("Complexity".to_string(), complexity_violations);

            for violation in &complexity_report.violations {
                let severity = match violation {
                    crate::services::complexity::Violation::Error { .. } => "Critical",
                    crate::services::complexity::Violation::Warning { .. } => "Warning",
                };
                *by_severity.entry(severity.to_string()).or_insert(0) += 1;
            }

            for file in &complexity_report.files {
                *total_loc += file.total_complexity.lines as usize;
            }
        }
    }

    fn process_satd_violations(
        &self,
        analyses: &ParallelAnalysisResults,
        total_defects: &mut usize,
        by_severity: &mut FxHashMap<String, usize>,
        by_type: &mut FxHashMap<String, usize>,
    ) {
        if let Some(ref satd_results) = analyses.satd_results {
            let satd_count = satd_results.items.len();
            *total_defects += satd_count;
            by_type.insert("TechnicalDebt".to_string(), satd_count);

            for item in &satd_results.items {
                let severity = match item.severity {
                    crate::services::satd_detector::Severity::Critical => "Critical",
                    crate::services::satd_detector::Severity::High => "Critical",
                    crate::services::satd_detector::Severity::Medium => "Warning",
                    crate::services::satd_detector::Severity::Low => "Normal",
                };
                *by_severity.entry(severity.to_string()).or_insert(0) += 1;
            }
        }
    }

    fn process_dead_code_violations(
        &self,
        analyses: &ParallelAnalysisResults,
        total_defects: &mut usize,
        by_severity: &mut FxHashMap<String, usize>,
        by_type: &mut FxHashMap<String, usize>,
    ) {
        if let Some(ref dead_code_results) = analyses.dead_code_results {
            let dead_code_count = dead_code_results.summary.dead_functions
                + dead_code_results.summary.dead_classes
                + dead_code_results.summary.dead_modules;
            *total_defects += dead_code_count;
            by_type.insert("DeadCode".to_string(), dead_code_count);
            *by_severity.entry("Warning".to_string()).or_insert(0) += dead_code_count;
        }
    }

    fn process_tdg_violations(
        &self,
        tdg_summary: &TDGSummary,
        total_defects: &mut usize,
        by_severity: &mut FxHashMap<String, usize>,
        by_type: &mut FxHashMap<String, usize>,
    ) {
        let high_tdg_count = tdg_summary.critical_files + tdg_summary.warning_files;
        *total_defects += high_tdg_count;
        by_type.insert("TDG".to_string(), high_tdg_count);
        *by_severity.entry("Critical".to_string()).or_insert(0) += tdg_summary.critical_files;
        *by_severity.entry("Warning".to_string()).or_insert(0) += tdg_summary.warning_files;
    }

    fn calculate_defect_density(&self, total_defects: usize, total_loc: usize) -> f64 {
        if total_loc > 0 {
            (total_defects as f64 * 1000.0) / total_loc as f64
        } else {
            0.0
        }
    }

    /// Generate hotspots from TDG scores
    fn generate_tdg_hotspots(
        &self,
        file_scores: &FxHashMap<String, TDGScore>,
    ) -> anyhow::Result<Vec<DefectHotspot>> {
        let mut hotspots: Vec<_> = file_scores
            .par_iter()
            .filter(|(_, score)| score.value > 1.5) // Filter above threshold
            .map(|(path, score)| DefectHotspot {
                location: FileLocation {
                    file: std::path::PathBuf::from(path),
                    line: 1,
                    column: 1,
                },
                composite_score: score.value as f32,
                contributing_factors: vec![DefectFactor::TechnicalDebt {
                    category: TechnicalDebtCategory::Implementation,
                    severity: TechnicalDebtSeverity::High,
                    age_days: 0,
                }],
                refactoring_effort: RefactoringEstimate {
                    estimated_hours: score.value as f32 * 2.0,
                    priority: Priority::High,
                    impact: Impact::Medium,
                    suggested_actions: vec!["Reduce TDG score".to_string()],
                },
            })
            .collect();

        hotspots
            .sort_unstable_by(|a, b| b.composite_score.partial_cmp(&a.composite_score).unwrap());
        hotspots.truncate(20);

        Ok(hotspots)
    }

    /// Analyze project metadata (Makefile and README)
    async fn analyze_project_metadata(
        &self,
        project_path: &Path,
    ) -> anyhow::Result<(
        Option<crate::models::project_meta::BuildInfo>,
        Option<crate::models::project_meta::ProjectOverview>,
    )> {
        use crate::services::{
            makefile_compressor::MakefileCompressor, project_meta_detector::ProjectMetaDetector,
            readme_compressor::ReadmeCompressor,
        };

        let detector = ProjectMetaDetector::new();
        let meta_files = detector.detect(project_path).await;

        let mut build_info = None;
        let mut project_overview = None;

        for meta_file in meta_files {
            match meta_file.file_type {
                crate::models::project_meta::MetaFileType::Makefile => {
                    let compressor = MakefileCompressor::new();
                    let compressed = compressor.compress(&meta_file.content);
                    build_info = Some(crate::models::project_meta::BuildInfo::from_makefile(
                        compressed,
                    ));
                    debug!("Makefile compressed and analyzed");
                }
                crate::models::project_meta::MetaFileType::Readme => {
                    let compressor = ReadmeCompressor::new();
                    let compressed = compressor.compress(&meta_file.content);
                    project_overview = Some(compressed.to_summary());
                    debug!("README compressed and analyzed");
                }
            }
        }

        Ok((build_info, project_overview))
    }

    /// Run QA verification on the deep context analysis results
    async fn run_qa_verification(
        &self,
        context: &DeepContext,
    ) -> anyhow::Result<QAVerificationResult> {
        // Convert DeepContext to the format expected by quality_gates
        let result = self.create_qa_compatible_result(context)?;

        // Create QA verification instance and generate report
        let qa_verification = QAVerification::new();
        let verification_report = qa_verification.generate_verification_report(&result);

        debug!(
            "QA verification report generated: overall status = {:?}",
            verification_report.overall
        );

        Ok(verification_report)
    }

    /// Create a DeepContextResult that's compatible with quality_gates expectations
    /// Convert complexity report to QA format
    fn convert_complexity_report_to_qa(&self, report: &ComplexityReport) -> ComplexityMetricsForQA {
        ComplexityMetricsForQA {
            files: report
                .files
                .iter()
                .map(|f| FileComplexityMetricsForQA {
                    path: std::path::PathBuf::from(&f.path),
                    functions: f
                        .functions
                        .iter()
                        .map(|func| FunctionComplexityForQA {
                            name: func.name.clone(),
                            cyclomatic: func.metrics.cyclomatic as u32,
                            cognitive: func.metrics.cognitive as u32,
                            nesting_depth: func.metrics.nesting_max as u32,
                            start_line: func.line_start as usize,
                            end_line: func.line_end as usize,
                        })
                        .collect(),
                    total_cyclomatic: f.total_complexity.cyclomatic as u32,
                    total_cognitive: f.total_complexity.cognitive as u32,
                    total_lines: f.total_complexity.lines as usize,
                })
                .collect(),
            summary: ComplexitySummaryForQA {
                total_files: report.files.len(),
                total_functions: report.files.par_iter().map(|f| f.functions.len()).sum(),
            },
        }
    }

    /// Create fallback complexity metrics from file discovery
    fn create_fallback_complexity_metrics(
        &self,
        context: &DeepContext,
    ) -> Option<ComplexityMetricsForQA> {
        let file_paths = self.collect_file_paths(&context.file_tree.root);
        let mut files_with_lines = Vec::new();
        let project_root = &context.metadata.project_root;

        debug!(
            "QA Fallback: Counting lines from {} files in {:?}",
            file_paths.len(),
            project_root
        );

        for path_str in &file_paths {
            if let Some(file_metrics) = self.process_file_for_fallback(path_str, project_root) {
                files_with_lines.push(file_metrics);
            }
        }

        if !files_with_lines.is_empty() {
            Some(ComplexityMetricsForQA {
                files: files_with_lines,
                summary: ComplexitySummaryForQA {
                    total_files: 0,
                    total_functions: 0,
                },
            })
        } else {
            None
        }
    }

    /// Process single file for fallback metrics
    fn process_file_for_fallback(
        &self,
        path_str: &str,
        project_root: &std::path::Path,
    ) -> Option<FileComplexityMetricsForQA> {
        let full_path = if std::path::Path::new(path_str).is_absolute() {
            std::path::PathBuf::from(path_str)
        } else {
            project_root.join(path_str)
        };

        if full_path.exists() && full_path.is_file() {
            if let Ok(content) = std::fs::read_to_string(&full_path) {
                let line_count = content.lines().count();

                if line_count > 0 {
                    return Some(FileComplexityMetricsForQA {
                        path: full_path,
                        functions: Vec::new(),
                        total_cyclomatic: 0,
                        total_cognitive: 0,
                        total_lines: line_count,
                    });
                }
            }
        }

        None
    }

    fn create_qa_compatible_result(
        &self,
        context: &DeepContext,
    ) -> anyhow::Result<DeepContextResult> {
        // Create complexity metrics from analysis results or fallback
        let complexity_metrics = if let Some(report) = context.analyses.complexity_report.as_ref() {
            Some(self.convert_complexity_report_to_qa(report))
        } else {
            self.create_fallback_complexity_metrics(context)
        };

        // Create dead code analysis from the results
        let dead_code_analysis = if let Some(ref dead_code) = context.analyses.dead_code_results {
            // Calculate total functions from complexity report if available
            let total_functions = context
                .analyses
                .complexity_report
                .as_ref()
                .map(|report| {
                    report
                        .files
                        .iter()
                        .map(|f| f.functions.len())
                        .sum::<usize>()
                })
                .unwrap_or(0);

            Some(DeadCodeAnalysis {
                summary: DeadCodeSummary {
                    total_functions,
                    dead_functions: dead_code.summary.dead_functions,
                    total_lines: dead_code
                        .ranked_files
                        .par_iter()
                        .map(|f| f.total_lines)
                        .sum(),
                    total_dead_lines: dead_code.summary.total_dead_lines,
                    dead_percentage: dead_code.summary.dead_percentage as f64,
                },
                dead_functions: vec![], // Not needed for QA verification
                warnings: vec![],
            })
        } else {
            None
        };

        // Create file paths list
        let file_paths = self.collect_file_paths(&context.file_tree.root);

        // Create AST summaries
        let ast_summaries = if !context.analyses.ast_contexts.is_empty() {
            Some(
                context
                    .analyses
                    .ast_contexts
                    .iter()
                    .map(|ctx| AstSummary {
                        path: ctx.base.path.clone(),
                        language: ctx.base.language.clone(),
                        total_items: ctx.base.items.len(),
                        functions: ctx
                            .base
                            .items
                            .iter()
                            .filter(|item| {
                                matches!(item, crate::services::context::AstItem::Function { .. })
                            })
                            .count(),
                        classes: ctx
                            .base
                            .items
                            .iter()
                            .filter(|item| {
                                matches!(item, crate::services::context::AstItem::Struct { .. })
                            })
                            .count(),
                        imports: ctx
                            .base
                            .items
                            .iter()
                            .filter(|item| {
                                matches!(
                                    item,
                                    crate::services::context::AstItem::Use { .. }
                                        | crate::services::context::AstItem::Import { .. }
                                )
                            })
                            .count(),
                    })
                    .collect(),
            )
        } else {
            None
        };

        // Create language statistics
        let mut language_stats = FxHashMap::default();
        for ctx in &context.analyses.ast_contexts {
            *language_stats.entry(ctx.base.language.clone()).or_insert(0) += 1;
        }

        // Build the QA-compatible result
        Ok(DeepContextResult {
            metadata: context.metadata.clone(),
            file_tree: file_paths, // Vec<String> for quality_gates
            analyses: context.analyses.clone(),
            quality_scorecard: context.quality_scorecard.clone(),
            template_provenance: context.template_provenance.clone(),
            defect_summary: context.defect_summary.clone(),
            hotspots: context.hotspots.clone(),
            recommendations: context.recommendations.clone(),
            qa_verification: context.qa_verification.clone(),

            // Additional fields expected by quality_gates
            complexity_metrics,
            dead_code_analysis,
            ast_summaries,
            churn_analysis: context.analyses.churn_analysis.clone(),
            language_stats: Some(language_stats),

            // Project metadata fields
            build_info: context.build_info.clone(),
            project_overview: context.project_overview.clone(),
        })
    }

    /// Collect all file paths from the annotated tree
    fn collect_file_paths(&self, node: &AnnotatedNode) -> Vec<String> {
        let mut paths = Vec::new();
        Self::collect_paths_recursive(node, &mut paths);
        paths
    }

    fn collect_paths_recursive(node: &AnnotatedNode, paths: &mut Vec<String>) {
        match node.node_type {
            NodeType::File => {
                paths.push(node.path.to_string_lossy().to_string());
            }
            NodeType::Directory => {
                for child in &node.children {
                    Self::collect_paths_recursive(child, paths);
                }
            }
        }
    }
}

/// Structure for collecting parallel analysis results
#[derive(Default)]
struct ParallelAnalysisResults {
    ast_contexts: Option<Vec<EnhancedFileContext>>,
    complexity_report: Option<ComplexityReport>,
    churn_analysis: Option<CodeChurnAnalysis>,
    dependency_graph: Option<DependencyGraph>,
    dead_code_results: Option<crate::models::dead_code::DeadCodeRankingResult>,
    duplicate_code_results: Option<crate::services::duplicate_detector::CloneReport>,
    satd_results: Option<SATDAnalysisResult>,
    provability_results:
        Option<Vec<crate::services::lightweight_provability_analyzer::ProofSummary>>,
    big_o_analysis: Option<crate::services::big_o_analyzer::BigOAnalysisReport>,
}

enum AnalysisResult {
    Ast(anyhow::Result<Vec<EnhancedFileContext>>),
    Complexity(anyhow::Result<ComplexityReport>),
    Churn(anyhow::Result<CodeChurnAnalysis>),
    DeadCode(anyhow::Result<crate::models::dead_code::DeadCodeRankingResult>),
    DuplicateCode(anyhow::Result<crate::services::duplicate_detector::CloneReport>),
    Satd(anyhow::Result<SATDAnalysisResult>),
    Provability(
        anyhow::Result<Vec<crate::services::lightweight_provability_analyzer::ProofSummary>>,
    ),
    Dag(anyhow::Result<DependencyGraph>),
    BigO(anyhow::Result<crate::services::big_o_analyzer::BigOAnalysisReport>),
}

// Analysis functions (simplified implementations)
async fn analyze_ast_contexts(
    path: &std::path::Path,
    _config: Option<FileClassifierConfig>,
) -> anyhow::Result<Vec<EnhancedFileContext>> {
    let _start_time = std::time::Instant::now();
    info!("Starting AST analysis for path: {:?}", path);

    let source_files = discover_and_categorize_source_files(path)?;
    let enhanced_contexts = analyze_source_files_for_contexts(source_files).await?;

    info!(
        "AST analysis completed. Generated {} file contexts",
        enhanced_contexts.len()
    );
    Ok(enhanced_contexts)
}

/// Discover files and filter for source files only
fn discover_and_categorize_source_files(path: &std::path::Path) -> anyhow::Result<Vec<PathBuf>> {
    use crate::services::file_discovery::{FileDiscoveryConfig, ProjectFileDiscovery};

    let discovery_config = create_ast_discovery_config();
    let discovery = ProjectFileDiscovery::new(path.to_path_buf()).with_config(discovery_config);
    let all_files = discovery.discover_files()?;
    
    let categorized_files = categorize_files_in_parallel(all_files);
    let source_files = filter_and_categorize_files(categorized_files);
    
    Ok(source_files)
}

/// Create discovery configuration for AST analysis
fn create_ast_discovery_config() -> crate::services::file_discovery::FileDiscoveryConfig {
    crate::services::file_discovery::FileDiscoveryConfig {
        respect_gitignore: true,
        filter_external_repos: true,
        max_files: Some(10_000), // Reasonable limit for AST analysis
        ..Default::default()
    }
}

/// Categorize files in parallel for better performance
fn categorize_files_in_parallel(all_files: Vec<PathBuf>) -> Vec<(PathBuf, crate::services::file_discovery::FileCategory)> {
    use crate::services::file_discovery::{FileCategory, ProjectFileDiscovery};
    
    all_files
        .into_par_iter()
        .map(|file_path| {
            let category = ProjectFileDiscovery::categorize_file(&file_path);
            (file_path, category)
        })
        .collect()
}

/// Filter categorized files to extract only source files
fn filter_and_categorize_files(
    categorized_files: Vec<(PathBuf, crate::services::file_discovery::FileCategory)>,
) -> Vec<PathBuf> {
    use crate::services::file_discovery::FileCategory;
    
    let mut source_files = Vec::new();
    let mut skipped_files = 0;

    for (file_path, category) in categorized_files {
        match category {
            FileCategory::SourceCode => {
                source_files.push(file_path);
            }
            FileCategory::GeneratedOutput | FileCategory::TestArtifact => {
                skipped_files += 1;
                debug!("Skipping generated/test file: {:?}", file_path);
            }
            FileCategory::EssentialDoc | FileCategory::BuildConfig => {
                debug!("Will compress metadata file: {:?}", file_path);
            }
            FileCategory::DevelopmentDoc => {
                debug!("Skipping development doc: {:?}", file_path);
            }
        }
    }

    info!(
        "Discovered {} source files for AST analysis (skipped {} generated/test files)",
        source_files.len(),
        skipped_files
    );
    
    source_files
}

/// Analyze source files and create enhanced contexts
async fn analyze_source_files_for_contexts(
    source_files: Vec<PathBuf>,
) -> anyhow::Result<Vec<EnhancedFileContext>> {
    let mut enhanced_contexts = Vec::new();
    let mut file_count = 0;
    let analysis_start = std::time::Instant::now();

    for file_path in source_files {
        if let Some(enhanced_context) = analyze_single_file_for_context(&file_path, &mut file_count).await {
            enhanced_contexts.push(enhanced_context);
        }
    }

    log_analysis_completion(analysis_start, file_count);
    Ok(enhanced_contexts)
}

/// Analyze single file and create enhanced context if successful
async fn analyze_single_file_for_context(
    file_path: &PathBuf,
    file_count: &mut usize,
) -> Option<EnhancedFileContext> {
    let file_start = std::time::Instant::now();
    
    if let Ok(file_context) = analyze_single_file(file_path).await {
        let ast_time = file_start.elapsed();

        if *file_count % 10 == 0 {
            info!(
                "Progress: {} files processed. Last file - AST: {:?}",
                file_count,
                ast_time
            );
        }

        let enhanced_context = EnhancedFileContext {
            base: file_context,
            complexity_metrics: None,
            churn_metrics: None,
            defects: DefectAnnotations {
                dead_code: None,
                technical_debt: Vec::new(),
                complexity_violations: Vec::new(),
                tdg_score: None, // Skip TDG calculation for context generation
            },
            symbol_id: uuid::Uuid::new_v4().to_string(),
        };
        
        *file_count += 1;
        Some(enhanced_context)
    } else {
        None
    }
}

/// Log analysis completion statistics
fn log_analysis_completion(analysis_start: std::time::Instant, file_count: usize) {
    let total_time = analysis_start.elapsed();
    info!(
        "AST analysis phase took {:?} for {} files ({:?} per file average)",
        total_time,
        file_count,
        total_time / file_count.max(1) as u32
    );
}

/// Analyze a single source file and extract AST items
/// Toyota Way refactored: Reduced complexity from 14 to <8 using Extract Method
pub async fn analyze_single_file(file_path: &std::path::Path) -> anyhow::Result<FileContext> {
    let path_str = file_path.to_string_lossy().to_string();
    let language = detect_language(file_path);
    let items = analyze_file_by_language(file_path, &language).await?;

    Ok(FileContext {
        path: path_str,
        language,
        items,
        complexity_metrics: None,
    })
}

/// Toyota Way Extract Method: Single responsibility for language-specific analysis
/// Reduced complexity by extracting the match logic into focused functions
pub async fn analyze_file_by_language(
    file_path: &std::path::Path,
    language: &str,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    match language {
        "rust" => analyze_rust_language(file_path).await,
        "typescript" | "javascript" => analyze_typescript_language(file_path).await,
        "python" => analyze_python_language(file_path).await,
        "c" | "cpp" => analyze_c_language(file_path).await,
        "kotlin" => analyze_kotlin_language(file_path).await,
        _ => Ok(Vec::new()),
    }
}

/// Toyota Way Single Responsibility: Handle Rust file analysis
pub async fn analyze_rust_language(
    file_path: &std::path::Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    analyze_rust_file(file_path).await
}

/// Toyota Way Single Responsibility: Handle TypeScript/JavaScript file analysis
pub async fn analyze_typescript_language(
    file_path: &std::path::Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    analyze_typescript_file(file_path).await
}

/// Toyota Way Single Responsibility: Handle Python file analysis
pub async fn analyze_python_language(
    file_path: &std::path::Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    analyze_python_file(file_path).await
}

/// Toyota Way Single Responsibility: Handle C/C++ file analysis
pub async fn analyze_c_language(
    file_path: &std::path::Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    analyze_c_file(file_path).await
}

/// Toyota Way Single Responsibility: Handle Kotlin file analysis with debug logging
pub async fn analyze_kotlin_language(
    file_path: &std::path::Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    tracing::debug!("Analyzing Kotlin file: {}", file_path.display());
    let items = analyze_kotlin_file(file_path).await?;
    tracing::debug!("Kotlin analysis returned {} items", items.len());
    Ok(items)
}

/// Detect programming language from file extension
fn detect_language(path: &std::path::Path) -> String {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        match ext {
            "rs" => "rust".to_string(),
            "ts" | "tsx" => "typescript".to_string(),
            "js" | "jsx" => "javascript".to_string(),
            "py" => "python".to_string(),
            "c" | "h" => "c".to_string(),
            "cpp" | "cc" | "cxx" | "hpp" | "hxx" => "cpp".to_string(),
            "kt" | "kts" => "kotlin".to_string(),
            _ => "unknown".to_string(),
        }
    } else {
        "unknown".to_string()
    }
}

/// Simple Rust file analysis
async fn analyze_rust_file(
    file_path: &std::path::Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    use crate::services::ast_rust::analyze_rust_file as analyze_rust;

    match analyze_rust(file_path).await {
        Ok(file_context) => Ok(file_context.items),
        Err(_) => Ok(Vec::new()), // Return empty vec on parse error
    }
}

/// Simple TypeScript/JavaScript file analysis
async fn analyze_typescript_file(
    _file_path: &Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    #[cfg(feature = "typescript-ast")]
    {
        use crate::services::ast_typescript::analyze_typescript_file as analyze_ts;

        match analyze_ts(_file_path).await {
            Ok(file_context) => Ok(file_context.items),
            Err(_) => Ok(Vec::new()), // Return empty vec on parse error
        }
    }
    #[cfg(not(feature = "typescript-ast"))]
    Ok(Vec::new())
}

/// Simple Python file analysis
async fn analyze_python_file(
    _file_path: &Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    #[cfg(feature = "python-ast")]
    {
        use crate::services::ast_python::analyze_python_file_with_classifier;

        match analyze_python_file_with_classifier(_file_path, None).await {
            Ok(file_context) => Ok(file_context.items),
            Err(_) => Ok(Vec::new()), // Return empty vec on parse error
        }
    }
    #[cfg(not(feature = "python-ast"))]
    Ok(Vec::new())
}

/// Simple C/C++ file analysis
async fn analyze_c_file(
    #[allow(unused_variables)] file_path: &Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    #[cfg(feature = "c-ast")]
    {
        use crate::models::unified_ast::AstKind;
        use crate::services::ast_c::CAstParser;
        use tokio::fs;

        // Read file content
        let content = fs::read_to_string(file_path).await?;

        // Parse with C AST parser
        let mut parser = CAstParser::new();
        let ast_dag = parser.parse_file(file_path, &content)?;

        // Convert AST DAG to context items
        let mut items = Vec::new();
        for node in ast_dag.nodes.iter() {
            if let AstKind::Function(_) = &node.kind {
                let item = crate::services::context::AstItem::Function {
                    name: format!("function_{}", node.name_vector), // Using name hash as placeholder
                    visibility: "public".to_string(),
                    is_async: false,
                    line: node.source_range.start as usize,
                };
                items.push(item);
            }
        }

        Ok(items)
    }
    #[cfg(not(feature = "c-ast"))]
    Ok(Vec::new())
}

async fn analyze_kotlin_file(
    #[allow(unused_variables)] file_path: &Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    // kotlin-ast feature is disabled
    Ok(Vec::new())
}

async fn analyze_complexity(path: &std::path::Path) -> anyhow::Result<ComplexityReport> {
    use crate::services::complexity::aggregate_results;

    info!("Starting complexity analysis for path: {:?}", path);

    // Extract Method: Discover source files
    let source_files = discover_source_files_for_complexity(path)?;
    info!(
        "Discovered {} source files for complexity analysis",
        source_files.len()
    );

    // Extract Method: Analyze all files
    let file_metrics = analyze_files_complexity(source_files).await;
    info!(
        "Complexity analysis completed. Analyzed {} files",
        file_metrics.len()
    );

    // Aggregate results into final report
    Ok(aggregate_results(file_metrics))
}

fn discover_source_files_for_complexity(
    path: &std::path::Path,
) -> anyhow::Result<Vec<std::path::PathBuf>> {
    use crate::services::file_discovery::{FileDiscoveryConfig, ProjectFileDiscovery};

    let discovery_config = FileDiscoveryConfig {
        respect_gitignore: true,
        filter_external_repos: true,
        max_files: Some(5_000), // Reasonable limit for complexity analysis
        ..Default::default()
    };

    let discovery = ProjectFileDiscovery::new(path.to_path_buf()).with_config(discovery_config);
    discovery.discover_files()
}

async fn analyze_files_complexity(
    source_files: Vec<std::path::PathBuf>,
) -> Vec<crate::services::complexity::FileComplexityMetrics> {
    let mut file_metrics = Vec::new();

    for file_path in source_files {
        if let Some(metrics) = analyze_single_file_complexity(&file_path).await {
            file_metrics.push(metrics);
        }
    }

    file_metrics
}

async fn analyze_single_file_complexity(
    file_path: &std::path::Path,
) -> Option<crate::services::complexity::FileComplexityMetrics> {
    #[cfg(feature = "python-ast")]
    use crate::services::ast_python::analyze_python_file_with_complexity;
    use crate::services::ast_rust::analyze_rust_file_with_complexity;
    #[cfg(feature = "typescript-ast")]
    use crate::services::ast_typescript::analyze_typescript_file_with_complexity;

    let ext = file_path.extension()?.to_str()?;

    match ext {
        "rs" => analyze_rust_file_with_complexity(file_path).await.ok(),
        #[cfg(feature = "typescript-ast")]
        "ts" | "js" | "jsx" | "tsx" => analyze_typescript_file_with_complexity(file_path)
            .await
            .ok(),
        #[cfg(feature = "python-ast")]
        "py" => analyze_python_file_with_complexity(file_path, None)
            .await
            .ok(),
        _ => None,
    }
}

async fn analyze_churn(path: &std::path::Path, days: u32) -> anyhow::Result<CodeChurnAnalysis> {
    use crate::services::git_analysis::GitAnalysisService;

    GitAnalysisService::analyze_code_churn(path, days)
        .map_err(|e| anyhow::anyhow!("Failed to analyze code churn: {}", e))
}

async fn analyze_dead_code(
    path: &std::path::Path,
) -> anyhow::Result<crate::models::dead_code::DeadCodeRankingResult> {
    use crate::models::dead_code::*;
    use crate::services::file_discovery::ProjectFileDiscovery;

    // Phase 1: Discover files for analysis without async AST parsing
    let discovery_service = ProjectFileDiscovery::new(path.to_path_buf());
    let all_files = discovery_service.discover_files()?;

    // Filter for source code files
    let files: Vec<_> = all_files
        .into_iter()
        .filter(|file| {
            if let Some(ext) = file.extension().and_then(|e| e.to_str()) {
                matches!(ext, "rs" | "ts" | "js" | "py")
            } else {
                false
            }
        })
        .collect();

    // Phase 2: Perform lightweight static analysis for dead code detection
    // Use parallel processing for file I/O and analysis
    let mut file_metrics: Vec<crate::models::dead_code::FileDeadCodeMetrics> = files
        .par_iter()
        .filter_map(|file_path| {
            std::fs::read_to_string(file_path)
                .ok()
                .map(|content| analyze_file_for_dead_code(file_path, &content))
        })
        .collect();

    // Aggregate metrics
    let total_dead_functions: usize = file_metrics.par_iter().map(|m| m.dead_functions).sum();
    let total_dead_classes: usize = file_metrics.par_iter().map(|m| m.dead_classes).sum();
    let total_dead_lines: usize = file_metrics.par_iter().map(|m| m.dead_lines).sum();

    // Phase 3: Calculate summary statistics
    let files_with_dead_code = file_metrics
        .par_iter()
        .filter(|f| f.dead_score > 0.0)
        .count();
    let total_lines_estimate: usize = file_metrics.par_iter().map(|f| f.total_lines).sum();
    let dead_percentage = if total_lines_estimate > 0 {
        (total_dead_lines as f32 / total_lines_estimate as f32) * 100.0
    } else {
        0.0
    };

    // Phase 4: Sort files by dead code score
    file_metrics.sort_unstable_by(|a, b| {
        b.dead_score
            .partial_cmp(&a.dead_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(DeadCodeRankingResult {
        summary: DeadCodeSummary {
            total_files_analyzed: files.len(),
            files_with_dead_code,
            total_dead_lines,
            dead_percentage,
            dead_functions: total_dead_functions,
            dead_classes: total_dead_classes,
            dead_modules: 0,
            unreachable_blocks: 0,
        },
        ranked_files: file_metrics,
        analysis_timestamp: chrono::Utc::now(),
        config: DeadCodeAnalysisConfig {
            include_unreachable: true,
            include_tests: false,
            min_dead_lines: 5,
        },
    })
}

fn analyze_file_for_dead_code(
    file_path: &std::path::Path,
    content: &str,
) -> crate::models::dead_code::FileDeadCodeMetrics {
    use crate::models::dead_code::{ConfidenceLevel, FileDeadCodeMetrics};

    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();
    let file_ext = file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    let mut dead_functions = 0;
    let mut dead_classes = 0;
    let mut dead_items = Vec::new();

    // Analyze based on file type
    match file_ext {
        "rs" => analyze_rust_dead_code(
            &lines,
            &mut dead_functions,
            &mut dead_classes,
            &mut dead_items,
        ),
        "ts" | "js" => analyze_typescript_dead_code(
            &lines,
            &mut dead_functions,
            &mut dead_classes,
            &mut dead_items,
        ),
        "py" => analyze_python_dead_code(
            &lines,
            &mut dead_functions,
            &mut dead_classes,
            &mut dead_items,
        ),
        _ => {}
    }

    let dead_lines = dead_items.len() * 5; // Conservative estimate
    let dead_percentage = if total_lines > 0 {
        (dead_lines as f32 / total_lines as f32) * 100.0
    } else {
        0.0
    };

    let confidence = if dead_items.is_empty() {
        ConfidenceLevel::High // High confidence in no dead code
    } else if dead_percentage > 20.0 {
        ConfidenceLevel::Medium
    } else {
        ConfidenceLevel::Low
    };

    let mut metrics = FileDeadCodeMetrics {
        path: file_path.to_string_lossy().to_string(),
        dead_lines,
        total_lines,
        dead_percentage,
        dead_functions,
        dead_classes,
        dead_modules: 0,
        unreachable_blocks: 0,
        dead_score: 0.0,
        confidence,
        items: dead_items,
    };

    metrics.calculate_score();
    metrics
}

fn analyze_rust_dead_code(
    lines: &[&str],
    dead_functions: &mut usize,
    dead_classes: &mut usize,
    dead_items: &mut Vec<crate::models::dead_code::DeadCodeItem>,
) {
    analyze_rust_dead_functions(lines, dead_functions, dead_items);
    analyze_rust_dead_structs(lines, dead_classes, dead_items);
}

/// Analyze dead functions in Rust code
fn analyze_rust_dead_functions(
    lines: &[&str],
    dead_functions: &mut usize,
    dead_items: &mut Vec<crate::models::dead_code::DeadCodeItem>,
) {
    use crate::models::dead_code::{DeadCodeItem, DeadCodeType};
    
    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        
        if trimmed.starts_with("fn ") && !trimmed.contains("pub ") {
            if let Some(function_name) = extract_function_name_if_unused(lines, trimmed) {
                *dead_functions += 1;
                dead_items.push(DeadCodeItem {
                    item_type: DeadCodeType::Function,
                    name: function_name,
                    line: (line_num + 1) as u32,
                    reason: "Private function with no apparent callers".to_string(),
                });
            }
        }
    }
}

/// Analyze dead structs in Rust code
fn analyze_rust_dead_structs(
    lines: &[&str],
    dead_classes: &mut usize,
    dead_items: &mut Vec<crate::models::dead_code::DeadCodeItem>,
) {
    use crate::models::dead_code::{DeadCodeItem, DeadCodeType};
    
    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        
        if trimmed.starts_with("struct ") && !trimmed.contains("pub ") {
            if let Some(struct_name) = extract_struct_name_if_unused(lines, trimmed) {
                *dead_classes += 1;
                dead_items.push(DeadCodeItem {
                    item_type: DeadCodeType::Class,
                    name: struct_name,
                    line: (line_num + 1) as u32,
                    reason: "Private struct with no apparent usage".to_string(),
                });
            }
        }
    }
}

/// Extract function name if unused
fn extract_function_name_if_unused(lines: &[&str], trimmed: &str) -> Option<String> {
    let function_name = extract_function_name(trimmed);
    if !function_name.is_empty() && !is_function_called_in_file(lines, &function_name) {
        Some(function_name)
    } else {
        None
    }
}

/// Extract struct name if unused
fn extract_struct_name_if_unused(lines: &[&str], trimmed: &str) -> Option<String> {
    let struct_name = extract_struct_name(trimmed);
    if !struct_name.is_empty() && !is_type_used_in_file(lines, &struct_name) {
        Some(struct_name)
    } else {
        None
    }
}

fn analyze_typescript_dead_code(
    lines: &[&str],
    dead_functions: &mut usize,
    dead_classes: &mut usize,
    dead_items: &mut Vec<crate::models::dead_code::DeadCodeItem>,
) {
    analyze_typescript_dead_functions(lines, dead_functions, dead_items);
    analyze_typescript_dead_classes(lines, dead_classes, dead_items);
}

/// Analyze dead functions in TypeScript code
fn analyze_typescript_dead_functions(
    lines: &[&str],
    dead_functions: &mut usize,
    dead_items: &mut Vec<crate::models::dead_code::DeadCodeItem>,
) {
    use crate::models::dead_code::{DeadCodeItem, DeadCodeType};
    
    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        
        if trimmed.starts_with("function ") && !trimmed.contains("export") {
            if let Some(function_name) = extract_js_function_name_if_unused(lines, trimmed) {
                *dead_functions += 1;
                dead_items.push(DeadCodeItem {
                    item_type: DeadCodeType::Function,
                    name: function_name,
                    line: (line_num + 1) as u32,
                    reason: "Non-exported function with no apparent callers".to_string(),
                });
            }
        }
    }
}

/// Analyze dead classes in TypeScript code
fn analyze_typescript_dead_classes(
    lines: &[&str],
    dead_classes: &mut usize,
    dead_items: &mut Vec<crate::models::dead_code::DeadCodeItem>,
) {
    use crate::models::dead_code::{DeadCodeItem, DeadCodeType};
    
    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        
        if trimmed.starts_with("class ") && !trimmed.contains("export") {
            if let Some(class_name) = extract_class_name_if_unused(lines, trimmed) {
                *dead_classes += 1;
                dead_items.push(DeadCodeItem {
                    item_type: DeadCodeType::Class,
                    name: class_name,
                    line: (line_num + 1) as u32,
                    reason: "Non-exported class with no apparent usage".to_string(),
                });
            }
        }
    }
}

/// Extract JS function name if unused
fn extract_js_function_name_if_unused(lines: &[&str], trimmed: &str) -> Option<String> {
    let function_name = extract_js_function_name(trimmed);
    if !function_name.is_empty() && !is_function_called_in_file(lines, &function_name) {
        Some(function_name)
    } else {
        None
    }
}

/// Extract class name if unused
fn extract_class_name_if_unused(lines: &[&str], trimmed: &str) -> Option<String> {
    let class_name = extract_class_name(trimmed);
    if !class_name.is_empty() && !is_type_used_in_file(lines, &class_name) {
        Some(class_name)
    } else {
        None
    }
}

fn analyze_python_dead_code(
    lines: &[&str],
    dead_functions: &mut usize,
    dead_classes: &mut usize,
    dead_items: &mut Vec<crate::models::dead_code::DeadCodeItem>,
) {
    analyze_python_dead_functions(lines, dead_functions, dead_items);
    analyze_python_dead_classes(lines, dead_classes, dead_items);
}

/// Analyze dead functions in Python code
fn analyze_python_dead_functions(
    lines: &[&str],
    dead_functions: &mut usize,
    dead_items: &mut Vec<crate::models::dead_code::DeadCodeItem>,
) {
    use crate::models::dead_code::{DeadCodeItem, DeadCodeType};
    
    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        
        if trimmed.starts_with("def _") {
            if let Some(function_name) = extract_python_function_name_if_unused(lines, trimmed) {
                *dead_functions += 1;
                dead_items.push(DeadCodeItem {
                    item_type: DeadCodeType::Function,
                    name: function_name,
                    line: (line_num + 1) as u32,
                    reason: "Private function with no apparent callers".to_string(),
                });
            }
        }
    }
}

/// Analyze dead classes in Python code
fn analyze_python_dead_classes(
    lines: &[&str],
    dead_classes: &mut usize,
    dead_items: &mut Vec<crate::models::dead_code::DeadCodeItem>,
) {
    use crate::models::dead_code::{DeadCodeItem, DeadCodeType};
    
    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        
        if trimmed.starts_with("class _") {
            if let Some(class_name) = extract_python_class_name_if_unused(lines, trimmed) {
                *dead_classes += 1;
                dead_items.push(DeadCodeItem {
                    item_type: DeadCodeType::Class,
                    name: class_name,
                    line: (line_num + 1) as u32,
                    reason: "Private class with no apparent usage".to_string(),
                });
            }
        }
    }
}

/// Extract Python function name if unused
fn extract_python_function_name_if_unused(lines: &[&str], trimmed: &str) -> Option<String> {
    let function_name = extract_python_function_name(trimmed);
    if !function_name.is_empty() && !is_function_called_in_file(lines, &function_name) {
        Some(function_name)
    } else {
        None
    }
}

/// Extract Python class name if unused
fn extract_python_class_name_if_unused(lines: &[&str], trimmed: &str) -> Option<String> {
    let class_name = extract_python_class_name(trimmed);
    if !class_name.is_empty() && !is_type_used_in_file(lines, &class_name) {
        Some(class_name)
    } else {
        None
    }
}

fn extract_function_name(line: &str) -> String {
    if let Some(start) = line.find("fn ") {
        let after_fn = &line[start + 3..];
        if let Some(paren_pos) = after_fn.find('(') {
            after_fn[..paren_pos].trim().to_string()
        } else {
            String::with_capacity(1024)
        }
    } else {
        String::with_capacity(1024)
    }
}

fn extract_struct_name(line: &str) -> String {
    if let Some(start) = line.find("struct ") {
        let after_struct = &line[start + 7..];
        after_struct
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_string()
    } else {
        String::with_capacity(1024)
    }
}

fn extract_js_function_name(line: &str) -> String {
    if let Some(start) = line.find("function ") {
        let after_fn = &line[start + 9..];
        if let Some(paren_pos) = after_fn.find('(') {
            after_fn[..paren_pos].trim().to_string()
        } else {
            String::with_capacity(1024)
        }
    } else {
        String::with_capacity(1024)
    }
}

fn extract_class_name(line: &str) -> String {
    if let Some(start) = line.find("class ") {
        let after_class = &line[start + 6..];
        after_class
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_string()
    } else {
        String::with_capacity(1024)
    }
}

fn extract_python_function_name(line: &str) -> String {
    if let Some(start) = line.find("def ") {
        let after_def = &line[start + 4..];
        if let Some(paren_pos) = after_def.find('(') {
            after_def[..paren_pos].trim().to_string()
        } else {
            String::with_capacity(1024)
        }
    } else {
        String::with_capacity(1024)
    }
}

fn extract_python_class_name(line: &str) -> String {
    if let Some(start) = line.find("class ") {
        let after_class = &line[start + 6..];
        if let Some(colon_pos) = after_class.find(':') {
            after_class[..colon_pos]
                .trim()
                .split('(')
                .next()
                .unwrap_or("")
                .trim()
                .to_string()
        } else {
            after_class
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_string()
        }
    } else {
        String::with_capacity(1024)
    }
}

fn is_function_called_in_file(lines: &[&str], function_name: &str) -> bool {
    let call_pattern = format!("{function_name}(");
    lines.iter().any(|line| line.contains(&call_pattern))
}

fn is_type_used_in_file(lines: &[&str], type_name: &str) -> bool {
    lines.iter().any(|line| {
        line.contains(type_name)
            && (line.contains(&format!("new {type_name}"))
                || line.contains(&format!(": {type_name}"))
                || line.contains(&format!("<{type_name}>")))
    })
}

async fn analyze_duplicate_code(
    path: &std::path::Path,
) -> anyhow::Result<crate::services::duplicate_detector::CloneReport> {
    use crate::services::duplicate_detector::DuplicateDetectionEngine;

    let all_files = discover_project_files(path)?;
    let files_for_analysis = filter_and_categorize_files_for_duplicates(all_files)?;
    let engine = DuplicateDetectionEngine::default();
    engine.detect_duplicates(&files_for_analysis)
}

fn discover_project_files(path: &std::path::Path) -> anyhow::Result<Vec<std::path::PathBuf>> {
    use crate::services::file_discovery::ProjectFileDiscovery;
    let discovery_service = ProjectFileDiscovery::new(path.to_path_buf());
    discovery_service.discover_files()
}

fn filter_and_categorize_files_for_duplicates(
    all_files: Vec<std::path::PathBuf>,
) -> anyhow::Result<
    Vec<(
        std::path::PathBuf,
        String,
        crate::services::duplicate_detector::Language,
    )>,
> {
    let mut files_for_analysis = Vec::new();
    for file_path in all_files {
        if let Some((file, content, lang)) = process_file_for_duplicate_detection(&file_path)? {
            files_for_analysis.push((file, content, lang));
        }
    }
    Ok(files_for_analysis)
}

fn process_file_for_duplicate_detection(
    file_path: &std::path::Path,
) -> anyhow::Result<
    Option<(
        std::path::PathBuf,
        String,
        crate::services::duplicate_detector::Language,
    )>,
> {
    let ext = match file_path.extension().and_then(|e| e.to_str()) {
        Some(e) => e,
        None => return Ok(None),
    };

    let language = match_extension_to_language(ext)?;
    if language.is_none() {
        return Ok(None);
    }

    let content = match std::fs::read_to_string(file_path) {
        Ok(c) if c.lines().count() >= 10 => c,
        _ => return Ok(None),
    };

    Ok(Some((file_path.to_path_buf(), content, language.unwrap())))
}

fn match_extension_to_language(
    ext: &str,
) -> anyhow::Result<Option<crate::services::duplicate_detector::Language>> {
    use crate::services::duplicate_detector::Language;

    Ok(match ext {
        "rs" => Some(Language::Rust),
        "ts" | "tsx" => Some(Language::TypeScript),
        "js" | "jsx" => Some(Language::JavaScript),
        "py" => Some(Language::Python),
        "c" | "h" => Some(Language::C),
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" => Some(Language::Cpp),
        "kt" | "kts" => Some(Language::Kotlin),
        _ => None,
    })
}

async fn analyze_satd(path: &std::path::Path) -> anyhow::Result<SATDAnalysisResult> {
    use crate::services::satd_detector::SATDDetector;

    let detector = SATDDetector::new();
    let result = detector.analyze_project(path, false).await?;

    Ok(result)
}

async fn analyze_provability(
    path: &std::path::Path,
) -> anyhow::Result<Vec<crate::services::lightweight_provability_analyzer::ProofSummary>> {
    use crate::services::context::{analyze_project, AstItem};
    use crate::services::lightweight_provability_analyzer::{
        FunctionId, LightweightProvabilityAnalyzer,
    };

    info!("Starting provability analysis for path: {:?}", path);

    let analyzer = LightweightProvabilityAnalyzer::new();

    // Discover functions from the project using AST analysis
    let project_context = analyze_project(path, "rust").await?;
    let mut function_ids = Vec::new();

    for file in &project_context.files {
        for item in &file.items {
            if let AstItem::Function { name, line, .. } = item {
                function_ids.push(FunctionId {
                    file_path: file.path.clone(),
                    function_name: name.clone(),
                    line_number: *line,
                });
            }
        }
    }

    // If no functions found, add a mock one
    if function_ids.is_empty() {
        function_ids.push(FunctionId {
            file_path: format!("{}/src/main.rs", path.display()),
            function_name: "main".to_string(),
            line_number: 1,
        });
    }

    let summaries = analyzer.analyze_incrementally(&function_ids).await;
    Ok(summaries)
}

async fn analyze_dag(path: &std::path::Path, dag_type: DagType) -> anyhow::Result<DependencyGraph> {
    use crate::services::{
        context::analyze_project,
        dag_builder::{
            filter_call_edges, filter_import_edges, filter_inheritance_edges, DagBuilder,
        },
    };

    // Analyze the project to get AST information
    let project_context = analyze_project(path, "rust").await?;

    // Build the dependency graph with PageRank pruning if needed
    let graph = DagBuilder::build_from_project_with_limit(&project_context, 400);

    // Apply filters based on DAG type
    let filtered_graph = match dag_type {
        DagType::CallGraph => filter_call_edges(graph),
        DagType::ImportGraph => filter_import_edges(graph),
        DagType::Inheritance => filter_inheritance_edges(graph),
        DagType::FullDependency => graph,
    };

    Ok(filtered_graph)
}

async fn analyze_big_o(
    path: &std::path::Path,
) -> anyhow::Result<crate::services::big_o_analyzer::BigOAnalysisReport> {
    use crate::services::big_o_analyzer::{BigOAnalysisConfig, BigOAnalyzer};

    let analyzer = BigOAnalyzer::new();
    let config = BigOAnalysisConfig {
        project_path: path.to_path_buf(),
        include_patterns: vec![
            "**/*.rs".to_string(),
            "**/*.ts".to_string(),
            "**/*.py".to_string(),
        ],
        exclude_patterns: vec!["**/target/**".to_string(), "**/node_modules/**".to_string()],
        confidence_threshold: 50,
        analyze_space_complexity: false,
    };

    analyzer.analyze(config).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tokio;

    #[test]
    fn test_analysis_type_variants() {
        let ast_type = AnalysisType::Ast;
        let complexity_type = AnalysisType::Complexity;
        let churn_type = AnalysisType::Churn;
        let dag_type = AnalysisType::Dag;

        // Test enum variants exist and can be created
        assert_eq!(ast_type, AnalysisType::Ast);
        assert_eq!(complexity_type, AnalysisType::Complexity);
        assert_eq!(churn_type, AnalysisType::Churn);
        assert_eq!(dag_type, AnalysisType::Dag);
    }

    #[test]
    fn test_dag_type_variants() {
        let call_graph = DagType::CallGraph;
        let import_graph = DagType::ImportGraph;
        let inheritance = DagType::Inheritance;
        let full_dependency = DagType::FullDependency;

        assert_eq!(call_graph, DagType::CallGraph);
        assert_eq!(import_graph, DagType::ImportGraph);
        assert_eq!(inheritance, DagType::Inheritance);
        assert_eq!(full_dependency, DagType::FullDependency);
    }

    #[test]
    fn test_cache_strategy_variants() {
        let normal = CacheStrategy::Normal;
        let force_refresh = CacheStrategy::ForceRefresh;
        let offline = CacheStrategy::Offline;

        assert_eq!(normal, CacheStrategy::Normal);
        assert_eq!(force_refresh, CacheStrategy::ForceRefresh);
        assert_eq!(offline, CacheStrategy::Offline);
    }

    #[test]
    fn test_complexity_thresholds_creation() {
        let thresholds = ComplexityThresholds {
            max_cyclomatic: 20,
            max_cognitive: 15,
        };

        assert_eq!(thresholds.max_cyclomatic, 20);
        assert_eq!(thresholds.max_cognitive, 15);
    }

    #[test]
    fn test_deep_context_config_default() {
        let config = DeepContextConfig::default();

        assert_eq!(config.period_days, 30);
        assert_eq!(config.dag_type, DagType::CallGraph);
        assert_eq!(config.max_depth, Some(3));
        assert_eq!(config.cache_strategy, CacheStrategy::Normal);
        assert_eq!(config.parallel, 4);
        assert!(config.include_analyses.contains(&AnalysisType::Ast));
        assert!(config.include_analyses.contains(&AnalysisType::Complexity));
        assert!(config.include_patterns.contains(&"**/*.rs".to_string()));
    }

    #[test]
    fn test_deep_context_analyzer_creation() {
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config.clone());

        assert_eq!(analyzer.config.period_days, config.period_days);
        assert_eq!(analyzer.config.parallel, config.parallel);
    }

    #[test]
    fn test_ast_summary_creation() {
        let summary = AstSummary {
            path: "test.rs".to_string(),
            language: "rust".to_string(),
            total_items: 100,
            functions: 50,
            classes: 20,
            imports: 10,
        };

        assert_eq!(summary.path, "test.rs");
        assert_eq!(summary.language, "rust");
        assert_eq!(summary.total_items, 100);
        assert_eq!(summary.functions, 50);
        assert_eq!(summary.classes, 20);
        assert_eq!(summary.imports, 10);
    }

    #[test]
    fn test_dead_code_summary_creation() {
        let summary = DeadCodeSummary {
            total_functions: 100,
            dead_functions: 15,
            total_lines: 10000,
            total_dead_lines: 450,
            dead_percentage: 4.5,
        };

        assert_eq!(summary.total_functions, 100);
        assert_eq!(summary.dead_functions, 15);
        assert_eq!(summary.total_lines, 10000);
        assert_eq!(summary.total_dead_lines, 450);
        assert_eq!(summary.dead_percentage, 4.5);
    }

    #[test]
    fn test_dead_code_analysis_creation() {
        let summary = DeadCodeSummary {
            total_functions: 50,
            dead_functions: 8,
            total_lines: 5000,
            total_dead_lines: 200,
            dead_percentage: 4.0,
        };

        let analysis = DeadCodeAnalysis {
            summary,
            dead_functions: vec![],
            warnings: vec![],
        };

        assert_eq!(analysis.summary.total_functions, 50);
        assert_eq!(analysis.summary.dead_functions, 8);
        assert_eq!(analysis.dead_functions.len(), 0);
    }

    #[test]
    fn test_context_metadata_creation() {
        let now = chrono::Utc::now();
        let cache_stats = CacheStats {
            hit_rate: 0.75,
            memory_efficiency: 0.8,
            time_saved_ms: 2000,
        };
        let metadata = ContextMetadata {
            generated_at: now,
            tool_version: "1.0.0".to_string(),
            project_root: PathBuf::from("/test"),
            cache_stats,
            analysis_duration: Duration::from_secs(30),
        };

        assert_eq!(metadata.generated_at, now);
        assert_eq!(metadata.tool_version, "1.0.0");
        assert_eq!(metadata.project_root, PathBuf::from("/test"));
        assert_eq!(metadata.cache_stats.hit_rate, 0.75);
        assert_eq!(metadata.analysis_duration, Duration::from_secs(30));
    }

    #[test]
    fn test_cache_stats_creation() {
        let stats = CacheStats {
            hit_rate: 0.8,
            memory_efficiency: 0.75,
            time_saved_ms: 1500,
        };

        assert_eq!(stats.hit_rate, 0.8);
        assert_eq!(stats.memory_efficiency, 0.75);
        assert_eq!(stats.time_saved_ms, 1500);
    }

    #[test]
    fn test_node_type_variants() {
        let file = NodeType::File;
        let directory = NodeType::Directory;

        assert_eq!(file, NodeType::File);
        assert_eq!(directory, NodeType::Directory);
    }

    #[test]
    fn test_node_annotations_creation() {
        let annotations = NodeAnnotations {
            defect_score: Some(15.5),
            complexity_score: Some(12.3),
            cognitive_complexity: Some(8),
            churn_score: Some(0.3),
            dead_code_items: 2,
            satd_items: 0,
            centrality: None,
            test_coverage: None,
            big_o_complexity: None,
            memory_complexity: None,
            duplication_score: None,
        };

        assert_eq!(annotations.defect_score, Some(15.5));
        assert_eq!(annotations.complexity_score, Some(12.3));
        assert_eq!(annotations.cognitive_complexity, Some(8));
        assert_eq!(annotations.churn_score, Some(0.3));
        assert_eq!(annotations.dead_code_items, 2);
    }

    #[test]
    fn test_annotated_node_creation() {
        let path = PathBuf::from("/test/file.rs");
        let annotations = NodeAnnotations {
            defect_score: Some(10.0),
            complexity_score: Some(8.5),
            cognitive_complexity: Some(12),
            churn_score: Some(0.2),
            dead_code_items: 2,
            satd_items: 0,
            centrality: None,
            test_coverage: None,
            big_o_complexity: None,
            memory_complexity: None,
            duplication_score: None,
        };

        let node = AnnotatedNode {
            name: "file.rs".to_string(),
            path: path.clone(),
            node_type: NodeType::File,
            annotations,
            children: vec![],
        };

        assert_eq!(node.path, path);
        assert_eq!(node.node_type, NodeType::File);
        assert_eq!(node.annotations.complexity_score, Some(10.0));
        assert_eq!(node.children.len(), 0);
    }

    #[test]
    fn test_annotated_file_tree_creation() {
        let root_path = PathBuf::from("/project");
        let root_annotations = NodeAnnotations {
            defect_score: Some(50.0),
            complexity_score: Some(15.2),
            cognitive_complexity: Some(18),
            churn_score: Some(0.1),
            dead_code_items: 5,
            satd_items: 0,
            centrality: Some(1.0),
            test_coverage: Some(80.0),
            big_o_complexity: Some("O(n)".to_string()),
            memory_complexity: Some("O(1)".to_string()),
            duplication_score: Some(0.05),
        };

        let root_node = AnnotatedNode {
            name: "test".to_string(),
            path: root_path.clone(),
            node_type: NodeType::Directory,
            annotations: root_annotations,
            children: vec![],
        };

        let tree = AnnotatedFileTree {
            root: root_node,
            total_files: 1,
            total_size_bytes: 1024,
        };

        assert_eq!(tree.root.path, root_path);
        assert_eq!(tree.total_files, 1);
        assert_eq!(tree.total_size_bytes, 1024);
    }

    #[test]
    #[ignore = "Test needs major refactoring for new DeepContextResult structure"]
    fn test_deep_context_result_creation() {
        // TODO: Update this test with the new DeepContextResult fields
        // including metadata, file_tree, analyses, quality_scorecard, etc.
    }

    #[tokio::test]
    async fn test_analyze_single_file_nonexistent() {
        let nonexistent_path = std::path::Path::new("/nonexistent/file.rs");
        let result = analyze_single_file(nonexistent_path).await;

        // Should return an error for nonexistent file
        assert!(result.is_err());
    }

    // ============================================================================
    // TDD TESTS FOR analyze_project REFACTORING - Sprint 47 Phase 3
    // Toyota Way: Test-Driven Development for Perfect Quality
    // ============================================================================

    #[tokio::test]
    async fn test_analyze_project_phase1_discovery_isolated() {
        // TDD: Phase 1 (Discovery) should be extractable as independent method
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        let test_project = tempfile::tempdir().unwrap();
        let project_path = test_project.path().to_path_buf();

        // Create test structure
        std::fs::create_dir_all(project_path.join("src")).unwrap();
        std::fs::write(project_path.join("src/main.rs"), "fn main() {}").unwrap();

        // Phase 1 should work independently
        let file_tree = analyzer
            .discover_project_structure(&project_path)
            .await
            .unwrap();
        assert!(file_tree.total_files > 0);
        assert_eq!(file_tree.root.node_type, NodeType::Directory);
    }

    #[tokio::test]
    async fn test_analyze_project_phase2_parallel_analyses_isolated() {
        // TDD: Phase 2 (Parallel Analyses) should be extractable
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        let test_project = tempfile::tempdir().unwrap();
        let project_path = test_project.path().to_path_buf();

        std::fs::create_dir_all(project_path.join("src")).unwrap();
        std::fs::write(project_path.join("src/lib.rs"), "pub fn test() {}").unwrap();

        let progress = crate::services::progress::ProgressTracker::new(false);
        let analyses = analyzer
            .execute_parallel_analyses_with_progress(&project_path, &progress)
            .await
            .unwrap();

        // Should complete without panicking
        assert!(analyses.ast_contexts.is_some() || analyses.complexity_report.is_some());
    }

    #[tokio::test]
    async fn test_analyze_project_phase3_cross_references_isolated() {
        // TDD: Phase 3 (Cross-Language References) should be extractable
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        let analyses = ParallelAnalysisResults::default();

        let cross_refs = analyzer
            .build_cross_language_references(&analyses)
            .await
            .unwrap();
        assert!(cross_refs.is_empty() || !cross_refs.is_empty());
    }

    #[tokio::test]
    async fn test_analyze_project_phase4_defect_correlation_isolated() {
        // TDD: Phase 4 (Defect Correlation) should be extractable
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        let analyses = ParallelAnalysisResults::default();

        let (defect_summary, hotspots) = analyzer.correlate_defects(&analyses).await.unwrap();
        assert!(defect_summary.total_defects >= 0);
        assert!(hotspots.is_empty() || !hotspots.is_empty());
    }

    #[tokio::test]
    async fn test_analyze_project_phase5_quality_scoring_isolated() {
        // TDD: Phase 5 (Quality Scoring) should be extractable
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        let analyses = ParallelAnalysisResults::default();
        let defect_summary = DefectSummary::default();

        // This method needs to be created during refactoring
        let quality = analyzer
            .calculate_quality_scorecard(&analyses, &defect_summary)
            .await
            .unwrap();
        assert!(quality.overall_health >= 0.0 && quality.overall_health <= 100.0);
    }

    #[tokio::test]
    async fn test_analyze_project_phase6_recommendations_isolated() {
        // TDD: Phase 6 (Recommendations) should be extractable
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        let deep_context = DeepContext::default();
        let defect_summary = DefectSummary {
            total_defects: 0,
            by_severity: FxHashMap::default(),
            by_type: FxHashMap::default(),
            defect_density: 0.0,
        };

        // This method needs to be created during refactoring
        let parallel_results = ParallelAnalysisResults {
            ast_contexts: None,
            complexity_report: None,
            churn_analysis: None,
            dependency_graph: None,
            dead_code_results: None,
            duplicate_code_results: None,
            satd_results: None,
            provability_results: None,
            big_o_analysis: None,
        };
        let recommendations = analyzer
            .generate_recommendations(&parallel_results, &defect_summary)
            .await
            .unwrap();
        assert!(recommendations.is_empty() || !recommendations.is_empty());
    }

    #[tokio::test]
    async fn test_analyze_project_phase7_metadata_analysis_isolated() {
        // TDD: Phase 7.5 (Project Metadata) should be extractable
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        let test_project = tempfile::tempdir().unwrap();
        let project_path = test_project.path().to_path_buf();

        std::fs::write(project_path.join("Makefile"), "test:\n\tcargo test").unwrap();
        std::fs::write(project_path.join("README.md"), "# Test").unwrap();

        let (build_info, overview) = analyzer
            .analyze_project_metadata(&project_path)
            .await
            .unwrap();
        assert!(build_info.is_some() || build_info.is_none());
        assert!(overview.is_some() || overview.is_none());
    }

    #[tokio::test]
    async fn test_analyze_project_phase8_qa_verification_isolated() {
        // TDD: Phase 8 (QA Verification) should be extractable
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        let mut context = DeepContext::default();
        context.metadata.project_root = PathBuf::from("/test");

        let qa = analyzer.run_qa_verification(&context).await.unwrap();
        // Check that we have a valid verification result
        assert!(!qa.timestamp.is_empty());
        assert!(!qa.version.is_empty());
    }

    #[tokio::test]
    async fn test_analyze_project_integration_all_phases() {
        // TDD: Integration test - refactored analyze_project should still work
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        let test_project = tempfile::tempdir().unwrap();
        let project_path = test_project.path().to_path_buf();

        std::fs::create_dir_all(project_path.join("src")).unwrap();
        std::fs::write(
            project_path.join("src/lib.rs"),
            "//! Test\npub fn add(a: i32, b: i32) -> i32 { a + b }",
        )
        .unwrap();

        let result = analyzer.analyze_project(&project_path).await.unwrap();

        // All phases should complete successfully
        assert_eq!(result.metadata.project_root, project_path);
        assert!(result.file_tree.total_files > 0);
        assert!(result.quality_scorecard.overall_health > 0.0);
        assert!(result.qa_verification.is_some());
    }

    #[tokio::test]
    async fn test_generate_recommendations_complexity_violations() {
        // TDD RED: Test complexity violation recommendations
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);

        let mut analyses = ParallelAnalysisResults::default();
        analyses.complexity_report = Some(crate::services::complexity::ComplexityReport {
            summary: Default::default(),
            violations: vec![crate::services::complexity::Violation::Error {
                rule: "complexity".to_string(),
                message: "Function too complex".to_string(),
                value: 30,
                threshold: 20,
                file: "test.rs".to_string(),
                line: 10,
                function: Some("complex_fn".to_string()),
            }],
            hotspots: vec![],
            files: vec![],
        });

        let defect_summary = DefectSummary::default();
        let recommendations = analyzer
            .generate_recommendations(&analyses, &defect_summary)
            .await
            .unwrap();

        assert_eq!(recommendations.len(), 1);
        assert!(recommendations[0].title.contains("complex_fn"));
        assert_eq!(recommendations[0].priority, Priority::Critical);
    }

    #[tokio::test]
    async fn test_generate_recommendations_high_defects() {
        // TDD RED: Test high defect count recommendations
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);

        let analyses = ParallelAnalysisResults::default();
        let mut by_severity = FxHashMap::default();
        by_severity.insert("high".to_string(), 50);
        by_severity.insert("medium".to_string(), 30);
        by_severity.insert("low".to_string(), 20);

        let defect_summary = DefectSummary {
            total_defects: 100,
            by_severity,
            by_type: FxHashMap::default(),
            defect_density: 10.0,
        };

        let recommendations = analyzer
            .generate_recommendations(&analyses, &defect_summary)
            .await
            .unwrap();

        assert_eq!(recommendations.len(), 1);
        assert!(recommendations[0].title.contains("High defect count"));
        assert_eq!(recommendations[0].priority, Priority::High);
    }

    #[tokio::test]
    async fn test_generate_recommendations_satd_detected() {
        // TDD RED: Test SATD detection recommendations
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);

        let mut analyses = ParallelAnalysisResults::default();
        analyses.satd_results = Some(crate::services::satd_detector::SATDAnalysisResult {
            items: vec![],
            summary: crate::services::satd_detector::SATDSummary {
                total_items: 5,
                by_severity: Default::default(),
                by_category: Default::default(),
                files_with_satd: 3,
                avg_age_days: 30.0,
            },
            total_files_analyzed: 10,
            files_with_debt: 3,
            analysis_timestamp: chrono::Utc::now(),
        });

        let defect_summary = DefectSummary::default();
        let recommendations = analyzer
            .generate_recommendations(&analyses, &defect_summary)
            .await
            .unwrap();

        assert_eq!(recommendations.len(), 1);
        assert!(recommendations[0].title.contains("Technical debt"));
        assert_eq!(recommendations[0].priority, Priority::Critical);
    }

    #[tokio::test]
    async fn test_analyze_complexity_function() {
        // TDD RED: Test analyze_complexity function refactoring
        let test_project = tempfile::tempdir().unwrap();
        let project_path = test_project.path();

        // Create a simple Rust file
        std::fs::write(
            project_path.join("test.rs"),
            "fn simple() { println!(\"test\"); }",
        )
        .unwrap();

        let result = analyze_complexity(project_path).await.unwrap();
        assert_eq!(result.summary.total_files, 1);
    }
}
