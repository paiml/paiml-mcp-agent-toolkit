use crate::models::{
    churn::CodeChurnAnalysis,
    dag::DependencyGraph,
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
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::Semaphore;
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheStrategy {
    Normal,
    ForceRefresh,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub language_stats: Option<HashMap<String, usize>>,

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMetadata {
    pub generated_at: DateTime<Utc>,
    pub tool_version: String,
    pub project_root: PathBuf,
    pub cache_stats: CacheStats,
    pub analysis_duration: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub hit_rate: f64,
    pub memory_efficiency: f64,
    pub time_saved_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotatedFileTree {
    pub root: AnnotatedNode,
    pub total_files: usize,
    pub total_size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotatedNode {
    pub name: String,
    pub path: PathBuf,
    pub node_type: NodeType,
    pub children: Vec<AnnotatedNode>,
    pub annotations: NodeAnnotations,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    Directory,
    File,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeAnnotations {
    pub defect_score: Option<f32>,
    pub complexity_score: Option<f32>,
    pub churn_score: Option<f32>,
    pub dead_code_items: usize,
    pub satd_items: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub parameters: HashMap<String, serde_json::Value>,
    pub drift_analysis: DriftAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftAnalysis {
    pub added_files: Vec<PathBuf>,
    pub modified_files: Vec<PathBuf>,
    pub deleted_files: Vec<PathBuf>,
    pub drift_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefectSummary {
    pub total_defects: usize,
    pub by_severity: HashMap<String, usize>,
    pub by_type: HashMap<String, usize>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

pub struct DeepContextAnalyzer {
    config: DeepContextConfig,
    #[allow(dead_code)] // Used for parallel execution control
    semaphore: Semaphore,
}

impl DeepContextAnalyzer {
    pub fn new(config: DeepContextConfig) -> Self {
        let semaphore = Semaphore::new(config.parallel);
        Self { config, semaphore }
    }

    /// Format as comprehensive markdown output using simple formatting
    pub async fn format_as_comprehensive_markdown(
        &self,
        context: &DeepContext,
    ) -> anyhow::Result<String> {
        // Simplified markdown formatting without formatting_pipeline
        let mut output = String::new();

        output.push_str("# Deep Context Analysis Report\n\n");

        // Project Overview (from README)
        if let Some(ref overview) = context.project_overview {
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

        // Build System (from Makefile)
        if let Some(ref build_info) = context.build_info {
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

        // Quality scorecard
        output.push_str("## Quality Scorecard\n\n");
        output.push_str(&format!(
            "- Overall Health: {:.1}%\n",
            context.quality_scorecard.overall_health
        ));
        output.push_str(&format!(
            "- Maintainability Index: {:.1}%\n",
            context.quality_scorecard.maintainability_index
        ));
        output.push_str(&format!(
            "- Technical Debt: {:.1} hours\n",
            context.quality_scorecard.technical_debt_hours
        ));
        output.push_str(&format!(
            "- Complexity Score: {:.1}%\n",
            context.quality_scorecard.complexity_score
        ));
        output.push('\n');

        // File tree
        output.push_str("## Project Structure\n\n");
        output.push_str("```\n");
        output.push_str(&format!(
            "Total Files: {}\nTotal Size: {} bytes\n",
            context.file_tree.total_files, context.file_tree.total_size_bytes
        ));
        output.push_str("\n```\n\n");

        // Analysis results
        output.push_str("## Analysis Results\n\n");

        if !context.analyses.ast_contexts.is_empty() {
            output.push_str(&format!(
                "### AST Analysis\n- Files analyzed: {}\n\n",
                context.analyses.ast_contexts.len()
            ));
        }

        if let Some(ref complexity) = context.analyses.complexity_report {
            output.push_str(&format!("### Complexity Analysis\n- Total files: {}\n- Total functions: {}\n- Median cyclomatic complexity: {:.1}\n\n",
                complexity.summary.total_files, complexity.summary.total_functions, complexity.summary.median_cyclomatic));
        }

        if let Some(ref churn) = context.analyses.churn_analysis {
            output.push_str(&format!(
                "### Code Churn\n- Files analyzed: {}\n- Total commits: {}\n\n",
                churn.files.len(),
                churn.summary.total_commits
            ));
        }

        // Recommendations
        if !context.recommendations.is_empty() {
            output.push_str("## Recommendations\n\n");
            for (i, rec) in context.recommendations.iter().enumerate() {
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

        Ok(output)
    }

    /// Legacy format method (kept for backward compatibility)
    pub fn format_as_comprehensive_markdown_legacy(
        &self,
        context: &DeepContext,
    ) -> anyhow::Result<String> {
        let mut output = String::new();

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
        use std::fmt::Write;

        // Quality scorecard summary
        writeln!(output, "\n## Quality Scorecard\n")?;
        writeln!(
            output,
            "- **Overall Health**: {} ({:.1}/100)",
            self.overall_health_emoji(context.quality_scorecard.overall_health),
            context.quality_scorecard.overall_health
        )?;
        writeln!(
            output,
            "- **Maintainability Index**: {:.1}",
            context.quality_scorecard.maintainability_index
        )?;
        writeln!(
            output,
            "- **Technical Debt**: {:.1} hours estimated",
            context.quality_scorecard.technical_debt_hours
        )?;

        // Project structure with annotations
        writeln!(output, "\n## Project Structure\n")?;
        writeln!(output, "```")?;
        self.format_annotated_tree(output, &context.file_tree)?;
        writeln!(output, "```\n")?;

        // Enhanced AST with complexity indicators
        if !context.analyses.ast_contexts.is_empty() {
            self.format_enhanced_ast_section(output, &context.analyses.ast_contexts)?;
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

        // Add complexity violations as SARIF results
        if let Some(ref complexity) = context.analyses.complexity_report {
            // Define complexity rules
            rules.push(json!({
                "id": "complexity/high-cyclomatic",
                "shortDescription": {"text": "High cyclomatic complexity"},
                "fullDescription": {"text": "Function has cyclomatic complexity above recommended threshold"},
                "defaultConfiguration": {"level": "warning"},
                "properties": {
                    "tags": ["complexity", "maintainability"]
                }
            }));

            rules.push(json!({
                "id": "complexity/high-cognitive",
                "shortDescription": {"text": "High cognitive complexity"},
                "fullDescription": {"text": "Function has cognitive complexity above recommended threshold"},
                "defaultConfiguration": {"level": "warning"},
                "properties": {
                    "tags": ["complexity", "maintainability"]
                }
            }));

            // Add results for high complexity functions
            for file in &complexity.files {
                for func in &file.functions {
                    if func.metrics.cyclomatic > 10 {
                        results.push(json!({
                            "ruleId": "complexity/high-cyclomatic",
                            "level": if func.metrics.cyclomatic > 20 { "error" } else { "warning" },
                            "message": {
                                "text": format!("Function '{}' has cyclomatic complexity of {}", func.name, func.metrics.cyclomatic)
                            },
                            "locations": [{
                                "physicalLocation": {
                                    "artifactLocation": {"uri": file.path.clone()},
                                    "region": {
                                        "startLine": func.line_start,
                                        "startColumn": 1,
                                        "endLine": func.line_end
                                    }
                                }
                            }],
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
                            "message": {
                                "text": format!("Function '{}' has cognitive complexity of {}", func.name, func.metrics.cognitive)
                            },
                            "locations": [{
                                "physicalLocation": {
                                    "artifactLocation": {"uri": file.path.clone()},
                                    "region": {
                                        "startLine": func.line_start,
                                        "startColumn": 1,
                                        "endLine": func.line_end
                                    }
                                }
                            }],
                            "properties": {
                                "cyclomatic_complexity": func.metrics.cyclomatic,
                                "cognitive_complexity": func.metrics.cognitive
                            }
                        }));
                    }
                }
            }
        }

        // Add SATD items as SARIF results
        if let Some(ref satd) = context.analyses.satd_results {
            rules.push(json!({
                "id": "debt/technical-debt",
                "shortDescription": {"text": "Technical debt item"},
                "fullDescription": {"text": "Self-admitted technical debt requiring attention"},
                "defaultConfiguration": {"level": "note"},
                "properties": {
                    "tags": ["debt", "maintainability"]
                }
            }));

            for item in &satd.items {
                let level = match item.severity {
                    crate::services::satd_detector::Severity::Critical => "error",
                    crate::services::satd_detector::Severity::High => "warning",
                    crate::services::satd_detector::Severity::Medium => "note",
                    crate::services::satd_detector::Severity::Low => "note",
                };

                results.push(json!({
                    "ruleId": "debt/technical-debt",
                    "level": level,
                    "message": {
                        "text": format!("{}: {}", item.category, item.text.trim())
                    },
                    "locations": [{
                        "physicalLocation": {
                            "artifactLocation": {"uri": item.file.to_string_lossy()},
                            "region": {
                                "startLine": item.line,
                                "startColumn": 1
                            }
                        }
                    }],
                    "properties": {
                        "category": format!("{:?}", item.category),
                        "severity": format!("{:?}", item.severity),
                        "debt_type": "self_admitted"
                    }
                }));
            }
        }

        // Add dead code as SARIF results
        if let Some(ref dead_code) = context.analyses.dead_code_results {
            rules.push(json!({
                "id": "dead-code/unused-code",
                "shortDescription": {"text": "Dead code detected"},
                "fullDescription": {"text": "Code that appears to be unused and can potentially be removed"},
                "defaultConfiguration": {"level": "warning"},
                "properties": {
                    "tags": ["dead-code", "maintainability"]
                }
            }));

            for file in &dead_code.ranked_files {
                if file.dead_functions > 0 {
                    results.push(json!({
                        "ruleId": "dead-code/unused-code",
                        "level": "warning",
                        "message": {
                            "text": format!("File contains {} dead functions and {} dead lines", 
                                file.dead_functions, file.dead_lines)
                        },
                        "locations": [{
                            "physicalLocation": {
                                "artifactLocation": {"uri": file.path.clone()},
                                "region": {"startLine": 1, "startColumn": 1}
                            }
                        }],
                        "properties": {
                            "dead_functions": file.dead_functions,
                            "dead_lines": file.dead_lines,
                            "dead_code_percentage": file.dead_lines as f64 / file.total_lines.max(1) as f64 * 100.0
                        }
                    }));
                }
            }
        }

        let sarif = json!({
            "version": "2.1.0",
            "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
            "runs": [{
                "tool": {
                    "driver": {
                        "name": "paiml-mcp-agent-toolkit",
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

        // Format node with annotations
        let mut node_display = node.name.clone();
        if matches!(node.node_type, NodeType::Directory) {
            node_display.push('/');
        }

        // Add annotations if present
        let mut annotations = Vec::new();
        if let Some(score) = node.annotations.defect_score {
            if score > 0.7 {
                annotations.push(format!("🔴{score:.1}"));
            } else if score > 0.4 {
                annotations.push(format!("🟡{score:.1}"));
            }
        }
        if node.annotations.satd_items > 0 {
            annotations.push(format!("📝{}", node.annotations.satd_items));
        }
        if node.annotations.dead_code_items > 0 {
            annotations.push(format!("💀{}", node.annotations.dead_code_items));
        }

        if !annotations.is_empty() {
            node_display.push_str(&format!(" [{}]", annotations.join(" ")));
        }

        writeln!(output, "{prefix}{connector}{node_display}")?;

        // Process children
        for (i, child) in node.children.iter().enumerate() {
            let is_last_child = i == node.children.len() - 1;
            self.format_tree_node(
                output,
                child,
                &format!("{prefix}{extension}"),
                is_last_child,
            )?;
        }

        Ok(())
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
            }
        }

        categorized
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
                String::new()
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
            writeln!(output, "\n**Technical Debt Gradient:** {:.2}\n", tdg.value)?;
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
                .iter()
                .flat_map(|f| f.functions.iter().map(move |func| (f, func)))
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
        use std::fmt::Write;
        if let Some(ref churn) = context.analyses.churn_analysis {
            writeln!(output, "## Code Churn Analysis\n")?;

            writeln!(output, "**Summary:**")?;
            writeln!(output, "- Total Commits: {}", churn.summary.total_commits)?;
            writeln!(output, "- Files Changed: {}", churn.files.len())?;

            // Top churned files
            let mut sorted_files = churn.files.clone();
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
        }

        Ok(())
    }

    fn format_technical_debt(
        &self,
        output: &mut String,
        context: &DeepContext,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        if let Some(ref satd) = context.analyses.satd_results {
            writeln!(output, "## Technical Debt Analysis\n")?;

            let mut by_severity = std::collections::HashMap::new();
            for item in &satd.items {
                *by_severity.entry(&item.severity).or_insert(0) += 1;
            }

            writeln!(output, "**SATD Summary:**")?;
            for (severity, count) in by_severity {
                writeln!(output, "- {severity:?}: {count}")?;
            }

            // Top critical debt items
            let critical_items: Vec<_> = satd
                .items
                .iter()
                .filter(|item| {
                    matches!(
                        item.severity,
                        crate::services::satd_detector::Severity::Critical
                    )
                })
                .take(5)
                .collect();

            if !critical_items.is_empty() {
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
            }
            writeln!(output)?;
        }

        Ok(())
    }

    fn format_dead_code_analysis(
        &self,
        output: &mut String,
        context: &DeepContext,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        if let Some(ref dead_code) = context.analyses.dead_code_results {
            writeln!(output, "## Dead Code Analysis\n")?;

            writeln!(output, "**Summary:**")?;
            writeln!(
                output,
                "- Dead Functions: {}",
                dead_code.summary.dead_functions
            )?;
            writeln!(
                output,
                "- Total Dead Lines: {}",
                dead_code.summary.total_dead_lines
            )?;

            if !dead_code.ranked_files.is_empty() {
                writeln!(output, "\n**Top Files with Dead Code:**")?;
                writeln!(output, "| File | Dead Lines | Dead Functions |")?;
                writeln!(output, "|------|------------|----------------|")?;

                for file in dead_code.ranked_files.iter().take(10) {
                    writeln!(
                        output,
                        "| `{}` | {} | {} |",
                        file.path, file.dead_lines, file.dead_functions
                    )?;
                }
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
        use std::fmt::Write;
        writeln!(output, "## Defect Probability Analysis\n")?;

        writeln!(output, "**Risk Assessment:**")?;
        writeln!(
            output,
            "- Total Defects Predicted: {}",
            context.defect_summary.total_defects
        )?;
        writeln!(
            output,
            "- Defect Density: {:.2} defects per 1000 lines",
            context.defect_summary.defect_density
        )?;

        if !context.hotspots.is_empty() {
            writeln!(output, "\n**High-Risk Hotspots:**")?;
            writeln!(output, "| File:Line | Risk Score | Effort (hours) |")?;
            writeln!(output, "|-----------|------------|----------------|")?;

            for hotspot in context.hotspots.iter().take(10) {
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
        use std::fmt::Write;
        if !recommendations.is_empty() {
            writeln!(output, "## Prioritized Recommendations\n")?;

            for (i, rec) in recommendations.iter().enumerate() {
                let priority_emoji = match rec.priority {
                    Priority::Critical => "🔴",
                    Priority::High => "🟡",
                    Priority::Medium => "🔵",
                    Priority::Low => "⚪",
                };

                writeln!(output, "### {} {} {}", priority_emoji, i + 1, rec.title)?;
                writeln!(output, "**Description:** {}", rec.description)?;
                writeln!(output, "**Effort:** {:?}", rec.estimated_effort)?;
                writeln!(output, "**Impact:** {:?}", rec.impact)?;

                if !rec.prerequisites.is_empty() {
                    writeln!(output, "**Prerequisites:**")?;
                    for prereq in &rec.prerequisites {
                        writeln!(output, "- {prereq}")?;
                    }
                }
                writeln!(output)?;
            }
        }

        Ok(())
    }

    pub async fn analyze_project(&self, project_path: &PathBuf) -> anyhow::Result<DeepContext> {
        let start_time = std::time::Instant::now();
        info!(
            "Starting deep context analysis for project: {:?}",
            project_path
        );

        // Phase 1: Discovery
        let file_tree = self.discover_project_structure(project_path).await?;
        debug!("Discovery phase completed");

        // Phase 2: Parallel analysis execution
        let analyses = self.execute_parallel_analyses(project_path).await?;
        debug!("Analysis phase completed");

        // Phase 3: Cross-language reference resolution
        let cross_refs = self.build_cross_language_references(&analyses).await?;
        debug!("Cross-reference resolution completed");

        // Phase 4: Defect correlation
        let (defect_summary, hotspots) = self.correlate_defects(&analyses).await?;
        debug!("Defect correlation completed");

        // Phase 5: Quality scoring
        let quality_scorecard = self.calculate_quality_scores(&analyses).await?;
        debug!("Quality scoring completed");

        // Phase 6: Generate recommendations
        let recommendations = self
            .generate_recommendations(&hotspots, &quality_scorecard)
            .await?;
        debug!("Recommendations generated");

        // Phase 7: Template provenance (if available)
        let template_provenance = self.analyze_template_provenance(project_path).await?;

        let analysis_duration = start_time.elapsed();
        info!("Deep context analysis completed in {:?}", analysis_duration);

        // Phase 7.5: Analyze project metadata (Makefile and README)
        let (build_info, project_overview) = self.analyze_project_metadata(project_path).await?;
        debug!("Project metadata analysis completed");

        // Create the deep context
        let mut deep_context = DeepContext {
            metadata: ContextMetadata {
                generated_at: Utc::now(),
                tool_version: env!("CARGO_PKG_VERSION").to_string(),
                project_root: project_path.clone(),
                cache_stats: CacheStats {
                    hit_rate: 0.0, // TODO: Implement cache statistics
                    memory_efficiency: 0.0,
                    time_saved_ms: 0,
                },
                analysis_duration,
            },
            file_tree,
            analyses: AnalysisResults {
                ast_contexts: analyses.ast_contexts.unwrap_or_default(),
                complexity_report: analyses.complexity_report,
                churn_analysis: analyses.churn_analysis,
                dependency_graph: analyses.dependency_graph,
                dead_code_results: analyses.dead_code_results,
                duplicate_code_results: analyses.duplicate_code_results,
                satd_results: analyses.satd_results,
                provability_results: analyses.provability_results,
                cross_language_refs: cross_refs,
            },
            quality_scorecard,
            template_provenance,
            defect_summary,
            hotspots,
            recommendations,
            qa_verification: None, // Will be populated next
            build_info,
            project_overview,
        };

        // Phase 8: Run QA verification
        deep_context.qa_verification = Some(self.run_qa_verification(&deep_context).await?);
        info!("QA verification completed");

        Ok(deep_context)
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
                    churn_score: None,
                    dead_code_items: 0,
                    satd_items: 0,
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
                    churn_score: None,
                    dead_code_items: 0,
                    satd_items: 0,
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

    async fn execute_parallel_analyses(
        &self,
        project_path: &std::path::Path,
    ) -> anyhow::Result<ParallelAnalysisResults> {
        // Step 1: Spawn all analysis tasks
        let mut join_set = self.spawn_analysis_tasks(project_path)?;

        // Step 2: Collect and process results with timeout
        let collection_timeout = std::time::Duration::from_secs(60);
        let results = self
            .collect_analysis_results(&mut join_set, collection_timeout)
            .await?;

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
            AnalysisType::Ast => {
                let file_classifier_config = self.config.file_classifier_config.clone();
                join_set.spawn(async move {
                    AnalysisResult::Ast(analyze_ast_contexts(&path, file_classifier_config).await)
                });
            }
            AnalysisType::Complexity => {
                join_set.spawn(async move {
                    AnalysisResult::Complexity(analyze_complexity(&path).await)
                });
            }
            AnalysisType::Churn => {
                let days = self.config.period_days;
                join_set
                    .spawn(async move { AnalysisResult::Churn(analyze_churn(&path, days).await) });
            }
            AnalysisType::DeadCode => {
                join_set
                    .spawn(async move { AnalysisResult::DeadCode(analyze_dead_code(&path).await) });
            }
            AnalysisType::DuplicateCode => {
                join_set.spawn(async move {
                    AnalysisResult::DuplicateCode(analyze_duplicate_code(&path).await)
                });
            }
            AnalysisType::Satd => {
                join_set.spawn(async move {
                    let result = tokio::task::spawn_blocking(move || {
                        tokio::runtime::Handle::current()
                            .block_on(async { analyze_satd(&path).await })
                    })
                    .await
                    .unwrap_or_else(|_| Err(anyhow::anyhow!("SATD analysis failed")));
                    AnalysisResult::Satd(result)
                });
            }
            AnalysisType::Provability => {
                join_set.spawn(async move {
                    AnalysisResult::Provability(analyze_provability(&path).await)
                });
            }
            AnalysisType::Dag => {
                let dag_type = self.config.dag_type.clone();
                join_set
                    .spawn(async move { AnalysisResult::Dag(analyze_dag(&path, dag_type).await) });
            }
            AnalysisType::TechnicalDebtGradient => {
                // DefectProbability is computed in correlate_defects, not as a separate analysis
            }
        }

        Ok(())
    }

    /// Collect and process analysis results with timeout
    async fn collect_analysis_results(
        &self,
        join_set: &mut tokio::task::JoinSet<AnalysisResult>,
        timeout: std::time::Duration,
    ) -> anyhow::Result<ParallelAnalysisResults> {
        let collection_future = self.process_analysis_results(join_set);

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

    /// Process all analysis results concurrently
    async fn process_analysis_results(
        &self,
        join_set: &mut tokio::task::JoinSet<AnalysisResult>,
    ) -> anyhow::Result<ParallelAnalysisResults> {
        // Collect all results first
        let mut pending_results = Vec::new();
        while let Some(result) = join_set.join_next().await {
            pending_results.push(result?);
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
        match result {
            AnalysisResult::Ast(Ok(ast_contexts)) => {
                results.ast_contexts = Some(ast_contexts);
            }
            AnalysisResult::Complexity(Ok(complexity)) => {
                results.complexity_report = Some(complexity);
            }
            AnalysisResult::Churn(Ok(churn)) => {
                results.churn_analysis = Some(churn);
            }
            AnalysisResult::DeadCode(Ok(dead_code)) => {
                results.dead_code_results = Some(dead_code);
            }
            AnalysisResult::DuplicateCode(Ok(duplicate_code)) => {
                results.duplicate_code_results = Some(duplicate_code);
            }
            AnalysisResult::Satd(Ok(satd)) => {
                results.satd_results = Some(satd);
            }
            AnalysisResult::Provability(Ok(provability)) => {
                results.provability_results = Some(provability);
            }
            AnalysisResult::Dag(Ok(dag)) => {
                results.dependency_graph = Some(dag);
            }
            AnalysisResult::Ast(Err(e)) => debug!("AST analysis failed: {}", e),
            AnalysisResult::Complexity(Err(e)) => debug!("Complexity analysis failed: {}", e),
            AnalysisResult::Churn(Err(e)) => debug!("Churn analysis failed: {}", e),
            AnalysisResult::DeadCode(Err(e)) => debug!("Dead code analysis failed: {}", e),
            AnalysisResult::DuplicateCode(Err(e)) => {
                debug!("Duplicate code analysis failed: {}", e)
            }
            AnalysisResult::Satd(Err(e)) => debug!("SATD analysis failed: {}", e),
            AnalysisResult::Provability(Err(e)) => debug!("Provability analysis failed: {}", e),
            AnalysisResult::Dag(Err(e)) => debug!("DAG analysis failed: {}", e),
        }
    }

    async fn build_cross_language_references(
        &self,
        _analyses: &ParallelAnalysisResults,
    ) -> anyhow::Result<Vec<CrossLangReference>> {
        // TODO: Implement cross-language reference detection
        // This would analyze FFI bindings, WASM exports, Python bindings, etc.
        Ok(Vec::new())
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
    ) -> anyhow::Result<std::collections::HashMap<String, TDGScore>> {
        use std::collections::HashMap;
        let mut file_tdg_scores = HashMap::new();

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
        file_scores: &std::collections::HashMap<String, TDGScore>,
    ) -> anyhow::Result<TDGSummary> {
        let total_files = file_scores.len();
        let mut critical_files = 0;
        let mut warning_files = 0;
        let mut tdg_values: Vec<f64> = Vec::new();

        for score in file_scores.values() {
            tdg_values.push(score.value);
            match score.severity {
                TDGSeverity::Critical => critical_files += 1,
                TDGSeverity::Warning => warning_files += 1,
                TDGSeverity::Normal => {}
            }
        }

        tdg_values.sort_by(|a, b| a.partial_cmp(b).unwrap());

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
        hotspots.sort_by(|a, b| b.tdg_score.partial_cmp(&a.tdg_score).unwrap());
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
        // Enumerate actual defects from all analysis sources
        let mut total_defects = 0usize;
        let mut by_severity = HashMap::new();
        let mut by_type = HashMap::new();
        let mut total_loc = 0usize;

        // Count complexity violations
        if let Some(ref complexity_report) = analyses.complexity_report {
            let complexity_violations = complexity_report.violations.len();
            total_defects += complexity_violations;
            by_type.insert("Complexity".to_string(), complexity_violations);

            // Count violations by severity
            for violation in &complexity_report.violations {
                let severity = match violation {
                    crate::services::complexity::Violation::Error { .. } => "Critical",
                    crate::services::complexity::Violation::Warning { .. } => "Warning",
                };
                *by_severity.entry(severity.to_string()).or_insert(0) += 1;
            }

            // Count total lines of code from file metrics
            for file in &complexity_report.files {
                total_loc += file.total_complexity.lines as usize;
            }
        }

        // Count SATD items
        if let Some(ref satd_results) = analyses.satd_results {
            let satd_count = satd_results.items.len();
            total_defects += satd_count;
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

        // Count dead code items
        if let Some(ref dead_code_results) = analyses.dead_code_results {
            let dead_code_count = dead_code_results.summary.dead_functions
                + dead_code_results.summary.dead_classes
                + dead_code_results.summary.dead_modules;
            total_defects += dead_code_count;
            by_type.insert("DeadCode".to_string(), dead_code_count);

            // Dead code is typically warning level
            *by_severity.entry("Warning".to_string()).or_insert(0) += dead_code_count;
        }

        // Count high TDG scores as defects
        let high_tdg_count = tdg_summary.critical_files + tdg_summary.warning_files;
        total_defects += high_tdg_count;
        by_type.insert("TDG".to_string(), high_tdg_count);
        *by_severity.entry("Critical".to_string()).or_insert(0) += tdg_summary.critical_files;
        *by_severity.entry("Warning".to_string()).or_insert(0) += tdg_summary.warning_files;

        // Calculate proper defect density: defects per 1000 lines of code
        let defect_density = if total_loc > 0 {
            (total_defects as f64 * 1000.0) / total_loc as f64
        } else {
            0.0
        };

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

    /// Generate hotspots from TDG scores
    fn generate_tdg_hotspots(
        &self,
        file_scores: &std::collections::HashMap<String, TDGScore>,
    ) -> anyhow::Result<Vec<DefectHotspot>> {
        let mut hotspots: Vec<_> = file_scores
            .iter()
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

        hotspots.sort_by(|a, b| b.composite_score.partial_cmp(&a.composite_score).unwrap());
        hotspots.truncate(20);

        Ok(hotspots)
    }

    /// Extract complexity metrics for a specific file
    #[allow(dead_code)]
    fn extract_complexity_metrics(
        &self,
        file_path: &str,
        analyses: &ParallelAnalysisResults,
    ) -> anyhow::Result<(f32, u32, u32)> {
        if let Some(ref complexity_report) = analyses.complexity_report {
            let file_complexity = complexity_report
                .files
                .iter()
                .find(|f| f.path == file_path)
                .map(|f| {
                    let avg_cyclomatic = if f.functions.is_empty() {
                        1.0
                    } else {
                        f.functions
                            .iter()
                            .map(|func| func.metrics.cyclomatic as f32)
                            .sum::<f32>()
                            / f.functions.len() as f32
                    };
                    let max_cyclomatic = f
                        .functions
                        .iter()
                        .map(|func| func.metrics.cyclomatic as u32)
                        .max()
                        .unwrap_or(1);
                    let max_cognitive = f
                        .functions
                        .iter()
                        .map(|func| func.metrics.cognitive as u32)
                        .max()
                        .unwrap_or(1);
                    (avg_cyclomatic, max_cyclomatic, max_cognitive)
                });

            if let Some((avg_complexity, max_cyclomatic, max_cognitive)) = file_complexity {
                Ok((avg_complexity, max_cyclomatic, max_cognitive))
            } else {
                Ok((1.0, 1, 1))
            }
        } else {
            Ok((1.0, 1, 1))
        }
    }

    /// Extract churn metrics for a specific file
    #[allow(dead_code)]
    fn extract_churn_metrics(
        &self,
        file_path: &str,
        analyses: &ParallelAnalysisResults,
    ) -> anyhow::Result<f32> {
        if let Some(ref churn_analysis) = analyses.churn_analysis {
            let churn_score = churn_analysis
                .files
                .iter()
                .find(|f| f.relative_path == file_path || f.relative_path.ends_with(file_path))
                .map(|f| {
                    // Normalize churn score based on commit count and recency
                    let max_commits = churn_analysis
                        .files
                        .iter()
                        .map(|file| file.commit_count)
                        .max()
                        .unwrap_or(1) as f32;
                    f.commit_count as f32 / max_commits
                })
                .unwrap_or(0.0);
            Ok(churn_score)
        } else {
            Ok(0.0)
        }
    }

    /// Build FileMetrics struct from collected data (LEGACY - DISABLED)
    #[allow(dead_code)]
    fn build_file_metrics(
        &self,
        _file_path: &str,
        _complexity_score: f32,
        _cyclomatic: u32,
        _cognitive: u32,
        _churn_score: f32,
        enhanced_context: &EnhancedFileContext,
    ) -> anyhow::Result<()> {
        // Estimate lines of code from AST items
        let _estimated_loc = enhanced_context.base.items.len() * 10;

        // Calculate efferent coupling from imports
        let _efferent_coupling = enhanced_context
            .base
            .items
            .iter()
            .filter_map(|item| match item {
                crate::services::context::AstItem::Use { .. } => Some(1.0),
                _ => None,
            })
            .sum::<f32>()
            .max(1.0);

        // Legacy function disabled
        Ok(())
    }

    /// Calculate defect probabilities for all files (LEGACY - DISABLED)
    #[allow(dead_code)]
    fn calculate_defect_probabilities(
        &self,
        _file_metrics_map: &std::collections::HashMap<String, ()>,
        _calculator: &(),
    ) -> anyhow::Result<()> {
        // Legacy function disabled
        Ok(())
    }

    /// Build defect summary from project analysis (LEGACY - DISABLED)
    #[allow(dead_code)]
    fn build_defect_summary(
        &self,
        _project_analysis: &(),
        _analyses: &ParallelAnalysisResults,
        _file_metrics_map: &std::collections::HashMap<String, ()>,
    ) -> anyhow::Result<DefectSummary> {
        // Legacy function disabled - return dummy data
        Ok(DefectSummary {
            total_defects: 0,
            by_severity: HashMap::new(),
            by_type: HashMap::new(),
            defect_density: 0.0,
        })
    }

    /// Generate defect hotspots from high-risk files (LEGACY - DISABLED)
    #[allow(dead_code)]
    fn generate_defect_hotspots(
        &self,
        _project_analysis: &(),
        _analyses: &ParallelAnalysisResults,
        _file_metrics_map: &std::collections::HashMap<String, ()>,
    ) -> anyhow::Result<Vec<DefectHotspot>> {
        // Legacy function disabled
        Ok(Vec::new())
    }

    /// Calculate quality scores (LEGACY - DISABLED)
    #[allow(dead_code)]
    async fn calculate_quality_scores(
        &self,
        _analyses: &ParallelAnalysisResults,
    ) -> anyhow::Result<QualityScorecard> {
        Ok(QualityScorecard {
            overall_health: 75.0,
            complexity_score: 80.0,
            maintainability_index: 70.0,
            modularity_score: 85.0,
            test_coverage: Some(65.0),
            technical_debt_hours: 40.0,
        })
    }

    /// Generate recommendations (LEGACY - DISABLED)
    #[allow(dead_code)]
    async fn generate_recommendations(
        &self,
        _hotspots: &[DefectHotspot],
        _quality: &QualityScorecard,
    ) -> anyhow::Result<Vec<PrioritizedRecommendation>> {
        Ok(Vec::new())
    }

    /// Analyze template provenance (LEGACY - DISABLED)
    #[allow(dead_code)]
    async fn analyze_template_provenance(
        &self,
        _path: &std::path::Path,
    ) -> anyhow::Result<Option<TemplateProvenance>> {
        Ok(None)
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
    fn create_qa_compatible_result(
        &self,
        context: &DeepContext,
    ) -> anyhow::Result<DeepContextResult> {
        // For QA verification, we need to ensure certain fields are populated
        // that the quality_gates module expects

        // Create complexity metrics from the analysis results
        let complexity_metrics =
            context
                .analyses
                .complexity_report
                .as_ref()
                .map(|report| ComplexityMetricsForQA {
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
                        total_functions: report.files.iter().map(|f| f.functions.len()).sum(),
                    },
                });

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
                    total_lines: dead_code.ranked_files.iter().map(|f| f.total_lines).sum(),
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
                                matches!(item, crate::services::context::AstItem::Use { .. })
                            })
                            .count(),
                    })
                    .collect(),
            )
        } else {
            None
        };

        // Create language statistics
        let mut language_stats = HashMap::new();
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
}

// Analysis functions (simplified implementations)
async fn analyze_ast_contexts(
    path: &std::path::Path,
    _config: Option<FileClassifierConfig>,
) -> anyhow::Result<Vec<EnhancedFileContext>> {
    use crate::services::file_discovery::{FileDiscoveryConfig, ProjectFileDiscovery};

    info!("Starting AST analysis for path: {:?}", path);

    let mut enhanced_contexts = Vec::new();

    // Use proper file discovery that respects .gitignore
    let discovery_config = FileDiscoveryConfig {
        respect_gitignore: true,
        filter_external_repos: true,
        max_files: Some(10_000), // Reasonable limit for AST analysis
        ..Default::default()
    };

    let discovery = ProjectFileDiscovery::new(path.to_path_buf()).with_config(discovery_config);

    let all_files = discovery.discover_files()?;

    // Filter files based on category
    let mut source_files = Vec::new();
    let mut essential_files = Vec::new();
    let mut skipped_files = 0;

    for file_path in all_files {
        use crate::services::file_discovery::FileCategory;
        let category = ProjectFileDiscovery::categorize_file(&file_path);

        match category {
            FileCategory::SourceCode => {
                source_files.push(file_path);
            }
            FileCategory::GeneratedOutput | FileCategory::TestArtifact => {
                skipped_files += 1;
                debug!("Skipping generated/test file: {:?}", file_path);
            }
            FileCategory::EssentialDoc | FileCategory::BuildConfig => {
                // Collect for compression and inclusion in report
                essential_files.push((file_path.clone(), category));
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

    // Initialize TDG calculator for per-file analysis
    let tdg_calculator = crate::services::tdg_calculator::TDGCalculator::new();

    // Analyze each discovered source file
    for file_path in source_files {
        if let Ok(file_context) = analyze_single_file(&file_path).await {
            // Calculate TDG score for this file
            let tdg_score = (tdg_calculator.calculate_file(&file_path).await).ok();

            let enhanced_context = EnhancedFileContext {
                base: file_context,
                complexity_metrics: None,
                churn_metrics: None,
                defects: DefectAnnotations {
                    dead_code: None,
                    technical_debt: Vec::new(),
                    complexity_violations: Vec::new(),
                    tdg_score,
                },
                symbol_id: uuid::Uuid::new_v4().to_string(),
            };
            enhanced_contexts.push(enhanced_context);
        }
    }

    info!(
        "AST analysis completed. Generated {} file contexts",
        enhanced_contexts.len()
    );
    Ok(enhanced_contexts)
}

/// Analyze a single source file and extract AST items
async fn analyze_single_file(file_path: &std::path::Path) -> anyhow::Result<FileContext> {
    let path_str = file_path.to_string_lossy().to_string();
    let language = detect_language(file_path);
    let mut items = Vec::new();

    // Simple AST analysis based on file extension
    match language.as_str() {
        "rust" => {
            items = analyze_rust_file(file_path).await?;
        }
        "typescript" | "javascript" => {
            items = analyze_typescript_file(file_path).await?;
        }
        "python" => {
            items = analyze_python_file(file_path).await?;
        }
        "c" | "cpp" => {
            items = analyze_c_file(file_path).await?;
        }
        _ => {}
    }

    Ok(FileContext {
        path: path_str,
        language,
        items,
        complexity_metrics: None,
    })
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

async fn analyze_complexity(path: &std::path::Path) -> anyhow::Result<ComplexityReport> {
    #[cfg(feature = "python-ast")]
    use crate::services::ast_python::analyze_python_file_with_complexity;
    use crate::services::ast_rust::analyze_rust_file_with_complexity;
    #[cfg(feature = "typescript-ast")]
    use crate::services::ast_typescript::analyze_typescript_file_with_complexity;
    use crate::services::complexity::aggregate_results;
    use crate::services::file_discovery::{FileDiscoveryConfig, ProjectFileDiscovery};

    info!("Starting complexity analysis for path: {:?}", path);

    // Use proper file discovery that respects .gitignore
    let discovery_config = FileDiscoveryConfig {
        respect_gitignore: true,
        filter_external_repos: true,
        max_files: Some(5_000), // Reasonable limit for complexity analysis
        ..Default::default()
    };

    let discovery = ProjectFileDiscovery::new(path.to_path_buf()).with_config(discovery_config);

    let source_files = discovery.discover_files()?;
    info!(
        "Discovered {} source files for complexity analysis",
        source_files.len()
    );

    let mut file_metrics = Vec::new();

    // Analyze each discovered source file for complexity
    for file_path in source_files {
        if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
            let file_complexity = match ext {
                "rs" => (analyze_rust_file_with_complexity(&file_path).await).ok(),
                #[cfg(feature = "typescript-ast")]
                "ts" | "js" | "jsx" | "tsx" => {
                    (analyze_typescript_file_with_complexity(&file_path).await).ok()
                }
                #[cfg(feature = "python-ast")]
                "py" => (analyze_python_file_with_complexity(&file_path).await).ok(),
                _ => None,
            };

            if let Some(metrics) = file_complexity {
                file_metrics.push(metrics);
            }
        }
    }

    info!(
        "Complexity analysis completed. Analyzed {} files",
        file_metrics.len()
    );

    // Aggregate results into final report
    Ok(aggregate_results(file_metrics))
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
    let mut file_metrics = Vec::new();
    let mut total_dead_functions = 0;
    let mut total_dead_classes = 0;
    let mut total_dead_lines = 0;

    for file_path in &files {
        if let Ok(content) = std::fs::read_to_string(file_path) {
            let metrics = analyze_file_for_dead_code(file_path, &content);
            total_dead_functions += metrics.dead_functions;
            total_dead_classes += metrics.dead_classes;
            total_dead_lines += metrics.dead_lines;
            file_metrics.push(metrics);
        }
    }

    // Phase 3: Calculate summary statistics
    let files_with_dead_code = file_metrics.iter().filter(|f| f.dead_score > 0.0).count();
    let total_lines_estimate: usize = file_metrics.iter().map(|f| f.total_lines).sum();
    let dead_percentage = if total_lines_estimate > 0 {
        (total_dead_lines as f32 / total_lines_estimate as f32) * 100.0
    } else {
        0.0
    };

    // Phase 4: Sort files by dead code score
    file_metrics.sort_by(|a, b| {
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
    use crate::models::dead_code::{DeadCodeItem, DeadCodeType};

    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Look for unused functions (simple heuristic: private functions without callers in same file)
        if trimmed.starts_with("fn ") && !trimmed.contains("pub ") {
            let function_name = extract_function_name(trimmed);
            if !function_name.is_empty() && !is_function_called_in_file(lines, &function_name) {
                *dead_functions += 1;
                dead_items.push(DeadCodeItem {
                    item_type: DeadCodeType::Function,
                    name: function_name,
                    line: (line_num + 1) as u32,
                    reason: "Private function with no apparent callers".to_string(),
                });
            }
        }

        // Look for unused structs
        if trimmed.starts_with("struct ") && !trimmed.contains("pub ") {
            let struct_name = extract_struct_name(trimmed);
            if !struct_name.is_empty() && !is_type_used_in_file(lines, &struct_name) {
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

fn analyze_typescript_dead_code(
    lines: &[&str],
    dead_functions: &mut usize,
    dead_classes: &mut usize,
    dead_items: &mut Vec<crate::models::dead_code::DeadCodeItem>,
) {
    use crate::models::dead_code::{DeadCodeItem, DeadCodeType};

    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Look for private functions
        if trimmed.starts_with("function ") && !trimmed.contains("export") {
            let function_name = extract_js_function_name(trimmed);
            if !function_name.is_empty() && !is_function_called_in_file(lines, &function_name) {
                *dead_functions += 1;
                dead_items.push(DeadCodeItem {
                    item_type: DeadCodeType::Function,
                    name: function_name,
                    line: (line_num + 1) as u32,
                    reason: "Non-exported function with no apparent callers".to_string(),
                });
            }
        }

        // Look for private classes
        if trimmed.starts_with("class ") && !trimmed.contains("export") {
            let class_name = extract_class_name(trimmed);
            if !class_name.is_empty() && !is_type_used_in_file(lines, &class_name) {
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

fn analyze_python_dead_code(
    lines: &[&str],
    dead_functions: &mut usize,
    dead_classes: &mut usize,
    dead_items: &mut Vec<crate::models::dead_code::DeadCodeItem>,
) {
    use crate::models::dead_code::{DeadCodeItem, DeadCodeType};

    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Look for private functions (starting with underscore)
        if trimmed.starts_with("def _") {
            let function_name = extract_python_function_name(trimmed);
            if !function_name.is_empty() && !is_function_called_in_file(lines, &function_name) {
                *dead_functions += 1;
                dead_items.push(DeadCodeItem {
                    item_type: DeadCodeType::Function,
                    name: function_name,
                    line: (line_num + 1) as u32,
                    reason: "Private function with no apparent callers".to_string(),
                });
            }
        }

        // Look for private classes
        if trimmed.starts_with("class _") {
            let class_name = extract_python_class_name(trimmed);
            if !class_name.is_empty() && !is_type_used_in_file(lines, &class_name) {
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

fn extract_function_name(line: &str) -> String {
    if let Some(start) = line.find("fn ") {
        let after_fn = &line[start + 3..];
        if let Some(paren_pos) = after_fn.find('(') {
            after_fn[..paren_pos].trim().to_string()
        } else {
            String::new()
        }
    } else {
        String::new()
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
        String::new()
    }
}

fn extract_js_function_name(line: &str) -> String {
    if let Some(start) = line.find("function ") {
        let after_fn = &line[start + 9..];
        if let Some(paren_pos) = after_fn.find('(') {
            after_fn[..paren_pos].trim().to_string()
        } else {
            String::new()
        }
    } else {
        String::new()
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
        String::new()
    }
}

fn extract_python_function_name(line: &str) -> String {
    if let Some(start) = line.find("def ") {
        let after_def = &line[start + 4..];
        if let Some(paren_pos) = after_def.find('(') {
            after_def[..paren_pos].trim().to_string()
        } else {
            String::new()
        }
    } else {
        String::new()
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
        String::new()
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
    use crate::services::duplicate_detector::{DuplicateDetectionEngine, Language};
    use crate::services::file_discovery::ProjectFileDiscovery;

    // Phase 1: Discover source files
    let discovery_service = ProjectFileDiscovery::new(path.to_path_buf());
    let all_files = discovery_service.discover_files()?;

    // Phase 2: Filter and categorize files by language
    let mut files_for_analysis = Vec::new();

    for file_path in all_files {
        if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
            let language = match ext {
                "rs" => Some(Language::Rust),
                "ts" | "tsx" => Some(Language::TypeScript),
                "js" | "jsx" => Some(Language::JavaScript),
                "py" => Some(Language::Python),
                "c" | "h" => Some(Language::C),
                "cpp" | "cc" | "cxx" | "hpp" | "hxx" => Some(Language::Cpp),
                _ => None,
            };

            if let Some(lang) = language {
                if let Ok(content) = std::fs::read_to_string(&file_path) {
                    // Skip very small files (likely not worth analyzing)
                    if content.lines().count() >= 10 {
                        files_for_analysis.push((file_path, content, lang));
                    }
                }
            }
        }
    }

    // Phase 3: Run duplicate detection
    let engine = DuplicateDetectionEngine::default();
    let report = engine.detect_duplicates(&files_for_analysis)?;

    Ok(report)
}

async fn analyze_satd(_path: &std::path::Path) -> anyhow::Result<SATDAnalysisResult> {
    Ok(SATDAnalysisResult {
        items: Vec::new(),
        summary: crate::services::satd_detector::SATDSummary {
            total_items: 0,
            by_severity: std::collections::HashMap::new(),
            by_category: std::collections::HashMap::new(),
            files_with_satd: 0,
            avg_age_days: 0.0,
        },
        total_files_analyzed: 0,
        files_with_debt: 0,
        analysis_timestamp: chrono::Utc::now(),
    })
}

async fn analyze_provability(
    path: &std::path::Path,
) -> anyhow::Result<Vec<crate::services::lightweight_provability_analyzer::ProofSummary>> {
    use crate::services::lightweight_provability_analyzer::{
        FunctionId, LightweightProvabilityAnalyzer,
    };

    info!("Starting provability analysis for path: {:?}", path);

    let analyzer = LightweightProvabilityAnalyzer::new();

    // Mock function discovery from project path - in a full implementation,
    // this would use AST analysis to discover all functions
    let function_ids = vec![FunctionId {
        file_path: format!("{}/src/main.rs", path.display()),
        function_name: "main".to_string(),
        line_number: 1,
    }];

    let summaries = analyzer.analyze_incrementally(&function_ids).await;
    Ok(summaries)
}

async fn analyze_dag(
    _path: &std::path::Path,
    _dag_type: DagType,
) -> anyhow::Result<DependencyGraph> {
    Ok(DependencyGraph::default())
}
