use crate::models::{churn::CodeChurnAnalysis, dag::DependencyGraph};
use crate::services::context::FileContext;
use crate::services::{
    ast_python, ast_typescript,
    complexity::{ComplexityReport, FileComplexityMetrics},
    dead_code_analyzer::DeadCodeAnalyzer,
    defect_probability::{DefectProbabilityCalculator, FileMetrics, ProjectDefectAnalysis},
    git_analysis::GitAnalysisService,
    satd_detector::{SATDAnalysisResult, SATDDetector},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::Semaphore;
use tracing::{debug, info, instrument};

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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnalysisType {
    Ast,
    Complexity,
    Churn,
    Dag,
    DeadCode,
    Satd,
    DefectProbability,
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
    pub satd_results: Option<SATDAnalysisResult>,
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
    pub defect_probability: f32,
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
        cyclomatic: u32,
        cognitive: u32,
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
                AnalysisType::DefectProbability,
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

        writeln!(output, "# Deep Context: {}", project_name)?;
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
                annotations.push(format!("🔴{:.1}", score));
            } else if score > 0.4 {
                annotations.push(format!("🟡{:.1}", score));
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

        writeln!(output, "{}{}{}", prefix, connector, node_display)?;

        // Process children
        for (i, child) in node.children.iter().enumerate() {
            let is_last_child = i == node.children.len() - 1;
            self.format_tree_node(
                output,
                child,
                &format!("{}{}", prefix, extension),
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

        // Defect probability
        writeln!(
            output,
            "\n**Defect Probability:** {:.1}%\n",
            context.defects.defect_probability * 100.0
        )?;

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
                writeln!(output, "- {:?}: {}", severity, count)?;
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
                        writeln!(output, "- {}", prereq)?;
                    }
                }
                writeln!(output)?;
            }
        }

        Ok(())
    }

    #[instrument(level = "info", skip(self))]
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

        Ok(DeepContext {
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
                satd_results: analyses.satd_results,
                cross_language_refs: cross_refs,
            },
            quality_scorecard,
            template_provenance,
            defect_summary,
            hotspots,
            recommendations,
        })
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
        let results = self.collect_analysis_results(&mut join_set, collection_timeout).await?;

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
                join_set.spawn(async move { AnalysisResult::Ast(analyze_ast_contexts(&path).await) });
            }
            AnalysisType::Complexity => {
                join_set.spawn(async move { AnalysisResult::Complexity(analyze_complexity(&path).await) });
            }
            AnalysisType::Churn => {
                let days = self.config.period_days;
                join_set.spawn(async move { AnalysisResult::Churn(analyze_churn(&path, days).await) });
            }
            AnalysisType::DeadCode => {
                join_set.spawn(async move { AnalysisResult::DeadCode(analyze_dead_code(&path).await) });
            }
            AnalysisType::Satd => {
                join_set.spawn(async move {
                    let result = tokio::task::spawn_blocking(move || {
                        tokio::runtime::Handle::current().block_on(async { analyze_satd(&path).await })
                    })
                    .await
                    .unwrap_or_else(|_| Err(anyhow::anyhow!("SATD analysis failed")));
                    AnalysisResult::Satd(result)
                });
            }
            AnalysisType::Dag => {
                let dag_type = self.config.dag_type.clone();
                join_set.spawn(async move { AnalysisResult::Dag(analyze_dag(&path, dag_type).await) });
            }
            AnalysisType::DefectProbability => {
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
            AnalysisResult::Satd(Ok(satd)) => {
                results.satd_results = Some(satd);
            }
            AnalysisResult::Dag(Ok(dag)) => {
                results.dependency_graph = Some(dag);
            }
            AnalysisResult::Ast(Err(e)) => debug!("AST analysis failed: {}", e),
            AnalysisResult::Complexity(Err(e)) => debug!("Complexity analysis failed: {}", e),
            AnalysisResult::Churn(Err(e)) => debug!("Churn analysis failed: {}", e),
            AnalysisResult::DeadCode(Err(e)) => debug!("Dead code analysis failed: {}", e),
            AnalysisResult::Satd(Err(e)) => debug!("SATD analysis failed: {}", e),
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
        // Step 1: Collect file metrics from all analyses
        let file_metrics_map = self.collect_file_metrics(analyses)?;

        // Step 2: Calculate defect probabilities
        let calculator = DefectProbabilityCalculator::new();
        let project_analysis =
            self.calculate_defect_probabilities(&file_metrics_map, &calculator)?;

        // Step 3: Build defect summary
        let defect_summary =
            self.build_defect_summary(&project_analysis, analyses, &file_metrics_map)?;

        // Step 4: Generate hotspots
        let hotspots =
            self.generate_defect_hotspots(&project_analysis, analyses, &file_metrics_map)?;

        Ok((defect_summary, hotspots))
    }

    /// Collect file metrics from all available analyses
    fn collect_file_metrics(
        &self,
        analyses: &ParallelAnalysisResults,
    ) -> anyhow::Result<std::collections::HashMap<String, FileMetrics>> {
        use std::collections::HashMap;
        let mut file_metrics_map = HashMap::new();

        if let Some(ref ast_contexts) = analyses.ast_contexts {
            for enhanced_context in ast_contexts {
                let file_path = enhanced_context.base.path.clone();

                // Extract complexity metrics for this file
                let (complexity_score, cyclomatic, cognitive) =
                    self.extract_complexity_metrics(&file_path, analyses)?;

                // Extract churn metrics for this file
                let churn_score = self.extract_churn_metrics(&file_path, analyses)?;

                // Build file metrics
                let file_metrics = self.build_file_metrics(
                    &file_path,
                    complexity_score,
                    cyclomatic,
                    cognitive,
                    churn_score,
                    enhanced_context,
                )?;

                file_metrics_map.insert(file_path, file_metrics);
            }
        }

        Ok(file_metrics_map)
    }

    /// Extract complexity metrics for a specific file
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

    /// Build FileMetrics struct from collected data
    fn build_file_metrics(
        &self,
        file_path: &str,
        complexity_score: f32,
        cyclomatic: u32,
        cognitive: u32,
        churn_score: f32,
        enhanced_context: &EnhancedFileContext,
    ) -> anyhow::Result<FileMetrics> {
        // Estimate lines of code from AST items
        let estimated_loc = enhanced_context.base.items.len() * 10;

        // Calculate efferent coupling from imports
        let efferent_coupling = enhanced_context
            .base
            .items
            .iter()
            .filter_map(|item| match item {
                crate::services::context::AstItem::Use { .. } => Some(1.0),
                _ => None,
            })
            .sum::<f32>()
            .max(1.0);

        Ok(FileMetrics {
            file_path: file_path.to_string(),
            churn_score,
            complexity: complexity_score,
            duplicate_ratio: 0.1, // Default assumption - would need actual duplication analysis
            afferent_coupling: 1.0, // Default - would come from DAG analysis
            efferent_coupling,
            lines_of_code: estimated_loc,
            cyclomatic_complexity: cyclomatic,
            cognitive_complexity: cognitive,
        })
    }

    /// Calculate defect probabilities for all files
    fn calculate_defect_probabilities(
        &self,
        file_metrics_map: &std::collections::HashMap<String, FileMetrics>,
        calculator: &DefectProbabilityCalculator,
    ) -> anyhow::Result<ProjectDefectAnalysis> {
        let file_scores: Vec<_> = file_metrics_map
            .values()
            .map(|metrics| (metrics.file_path.clone(), calculator.calculate(metrics)))
            .collect();

        Ok(ProjectDefectAnalysis::from_scores(file_scores))
    }

    /// Build defect summary from project analysis
    fn build_defect_summary(
        &self,
        project_analysis: &ProjectDefectAnalysis,
        analyses: &ParallelAnalysisResults,
        file_metrics_map: &std::collections::HashMap<String, FileMetrics>,
    ) -> anyhow::Result<DefectSummary> {
        use std::collections::HashMap;

        let mut defect_summary = DefectSummary {
            total_defects: project_analysis.high_risk_files.len()
                + project_analysis.medium_risk_files.len(),
            by_severity: HashMap::new(),
            by_type: HashMap::new(),
            defect_density: 0.0,
        };

        // Populate severity breakdown
        defect_summary
            .by_severity
            .insert("high".to_string(), project_analysis.high_risk_files.len());
        defect_summary.by_severity.insert(
            "medium".to_string(),
            project_analysis.medium_risk_files.len(),
        );
        defect_summary.by_severity.insert(
            "low".to_string(),
            project_analysis.total_files
                - project_analysis.high_risk_files.len()
                - project_analysis.medium_risk_files.len(),
        );

        // Count defects by type from specific analyses
        if let Some(ref dead_code) = analyses.dead_code_results {
            defect_summary
                .by_type
                .insert("dead_code".to_string(), dead_code.summary.dead_functions);
        }

        if let Some(ref satd) = analyses.satd_results {
            defect_summary
                .by_type
                .insert("technical_debt".to_string(), satd.items.len());
        }

        // Calculate defect density (defects per 1000 lines of code)
        let total_loc: usize = file_metrics_map.values().map(|m| m.lines_of_code).sum();
        defect_summary.defect_density = if total_loc > 0 {
            (defect_summary.total_defects as f64 / total_loc as f64) * 1000.0
        } else {
            0.0
        };

        Ok(defect_summary)
    }

    /// Generate defect hotspots from high-risk files
    fn generate_defect_hotspots(
        &self,
        project_analysis: &ProjectDefectAnalysis,
        analyses: &ParallelAnalysisResults,
        file_metrics_map: &std::collections::HashMap<String, FileMetrics>,
    ) -> anyhow::Result<Vec<DefectHotspot>> {
        let mut hotspots = Vec::new();

        for (file_path, defect_score) in project_analysis.get_top_risk_files(20) {
            // Create contributing factors
            let contributing_factors = self.create_contributing_factors(
                file_path,
                defect_score,
                analyses,
                file_metrics_map,
            )?;

            // Calculate refactoring effort and priority
            let (estimated_hours, priority, impact) =
                self.calculate_refactoring_estimates(defect_score);

            hotspots.push(DefectHotspot {
                location: FileLocation {
                    file: std::path::PathBuf::from(file_path),
                    line: 1, // Would need more sophisticated line-level analysis
                    column: 1,
                },
                composite_score: defect_score.probability,
                contributing_factors,
                refactoring_effort: RefactoringEstimate {
                    estimated_hours,
                    priority,
                    impact,
                    suggested_actions: defect_score.recommendations.clone(),
                },
            });
        }

        // Sort hotspots by composite score (highest risk first)
        hotspots.sort_by(|a, b| b.composite_score.partial_cmp(&a.composite_score).unwrap());

        Ok(hotspots)
    }

    /// Create contributing factors for a defect hotspot
    fn create_contributing_factors(
        &self,
        file_path: &str,
        defect_score: &crate::services::defect_probability::DefectScore,
        analyses: &ParallelAnalysisResults,
        file_metrics_map: &std::collections::HashMap<String, FileMetrics>,
    ) -> anyhow::Result<Vec<DefectFactor>> {
        let mut contributing_factors = Vec::new();

        for (factor_name, contribution) in &defect_score.contributing_factors {
            match factor_name.as_str() {
                "complexity" => {
                    if let Some(file_metrics) = file_metrics_map.get(file_path) {
                        contributing_factors.push(DefectFactor::Complexity {
                            cyclomatic: file_metrics.cyclomatic_complexity,
                            cognitive: file_metrics.cognitive_complexity,
                            violations: defect_score.recommendations.clone(),
                        });
                    }
                }
                "churn" => {
                    if let Some(ref churn_analysis) = analyses.churn_analysis {
                        if let Some(churn_file) = churn_analysis.files.iter().find(|f| {
                            f.relative_path == file_path || f.relative_path.ends_with(file_path)
                        }) {
                            contributing_factors.push(DefectFactor::ChurnRisk {
                                commits: churn_file.commit_count as u32,
                                authors: churn_file.unique_authors.len() as u32,
                                defect_correlation: *contribution,
                            });
                        }
                    }
                }
                "dead_code" => {
                    contributing_factors.push(DefectFactor::DeadCode {
                        confidence: ConfidenceLevel::Medium,
                        reason: "Detected through static analysis".to_string(),
                    });
                }
                _ => {}
            }
        }

        Ok(contributing_factors)
    }

    /// Calculate refactoring effort estimates
    fn calculate_refactoring_estimates(
        &self,
        defect_score: &crate::services::defect_probability::DefectScore,
    ) -> (f32, Priority, Impact) {
        let estimated_hours = match defect_score.risk_level {
            crate::services::defect_probability::RiskLevel::High => defect_score.probability * 8.0,
            crate::services::defect_probability::RiskLevel::Medium => {
                defect_score.probability * 4.0
            }
            crate::services::defect_probability::RiskLevel::Low => defect_score.probability * 2.0,
        };

        let priority = match defect_score.risk_level {
            crate::services::defect_probability::RiskLevel::High => Priority::High,
            crate::services::defect_probability::RiskLevel::Medium => Priority::Medium,
            crate::services::defect_probability::RiskLevel::Low => Priority::Low,
        };

        let impact = if defect_score.probability > 0.8 {
            Impact::High
        } else if defect_score.probability > 0.5 {
            Impact::Medium
        } else {
            Impact::Low
        };

        (estimated_hours, priority, impact)
    }

    async fn calculate_quality_scores(
        &self,
        analyses: &ParallelAnalysisResults,
    ) -> anyhow::Result<QualityScorecard> {
        let mut complexity_score = 100.0;
        let mut maintainability_index = 100.0;
        let modularity_score = 100.0; // TODO: Calculate from DAG analysis
        let test_coverage = 50.0; // Default coverage assumption
        let mut technical_debt_hours = 0.0;

        // Calculate complexity score
        if let Some(ref complexity) = analyses.complexity_report {
            // Simplified scoring: inverse of median complexity (NO AVERAGES per spec)
            let median_complexity = complexity.summary.median_cyclomatic as f64;
            complexity_score = 100.0 / (1.0 + median_complexity / 10.0);
        }

        // Calculate technical debt hours
        if let Some(ref satd) = analyses.satd_results {
            // Estimate hours based on SATD items
            technical_debt_hours = satd.items.len() as f64 * 0.5; // 30 minutes per item
        }

        // Calculate maintainability index
        if let Some(ref churn) = analyses.churn_analysis {
            // Factor in churn rate
            let avg_churn = churn.summary.total_commits as f64 / churn.files.len().max(1) as f64;
            maintainability_index = 100.0 / (1.0 + avg_churn / 20.0);
        }

        let overall_health = (complexity_score * 0.3
            + maintainability_index * 0.3
            + modularity_score * 0.2
            + test_coverage * 0.2)
            .min(100.0);

        Ok(QualityScorecard {
            overall_health,
            complexity_score,
            maintainability_index,
            modularity_score,
            test_coverage: Some(test_coverage),
            technical_debt_hours,
        })
    }

    async fn generate_recommendations(
        &self,
        _hotspots: &[DefectHotspot],
        scorecard: &QualityScorecard,
    ) -> anyhow::Result<Vec<PrioritizedRecommendation>> {
        let mut recommendations = Vec::new();

        // Generate recommendations based on quality scores
        if scorecard.complexity_score < 70.0 {
            recommendations.push(PrioritizedRecommendation {
                title: "Reduce Code Complexity".to_string(),
                description: "Several functions exceed complexity thresholds. Consider refactoring complex functions into smaller, more focused units.".to_string(),
                priority: Priority::High,
                estimated_effort: Duration::from_secs(8 * 3600), // 8 hours
                impact: Impact::High,
                prerequisites: vec!["Identify most complex functions".to_string()],
            });
        }

        if scorecard.technical_debt_hours > 20.0 {
            recommendations.push(PrioritizedRecommendation {
                title: "Address Technical Debt".to_string(),
                description: format!("Found {:.1} hours of estimated technical debt. Prioritize critical SATD items.", scorecard.technical_debt_hours),
                priority: Priority::Medium,
                estimated_effort: Duration::from_secs((scorecard.technical_debt_hours * 3600.0) as u64),
                impact: Impact::Medium,
                prerequisites: vec!["Review SATD analysis".to_string()],
            });
        }

        if scorecard.maintainability_index < 60.0 {
            recommendations.push(PrioritizedRecommendation {
                title: "Improve Code Maintainability".to_string(),
                description:
                    "High churn rate detected. Consider stabilizing frequently changed code paths."
                        .to_string(),
                priority: Priority::Medium,
                estimated_effort: Duration::from_secs(16 * 3600), // 16 hours
                impact: Impact::High,
                prerequisites: vec!["Analyze churn patterns".to_string()],
            });
        }

        Ok(recommendations)
    }

    async fn analyze_template_provenance(
        &self,
        project_path: &std::path::Path,
    ) -> anyhow::Result<Option<TemplateProvenance>> {
        let scaffold_file = project_path.join(".paiml-scaffold.json");

        if scaffold_file.exists() {
            // TODO: Implement template provenance analysis
            // This would parse the scaffold metadata and compare current state
            Ok(None)
        } else {
            Ok(None)
        }
    }
}

#[derive(Default)]
struct ParallelAnalysisResults {
    ast_contexts: Option<Vec<EnhancedFileContext>>,
    complexity_report: Option<ComplexityReport>,
    churn_analysis: Option<CodeChurnAnalysis>,
    dependency_graph: Option<DependencyGraph>,
    dead_code_results: Option<crate::models::dead_code::DeadCodeRankingResult>,
    satd_results: Option<SATDAnalysisResult>,
}

enum AnalysisResult {
    Ast(anyhow::Result<Vec<EnhancedFileContext>>),
    Complexity(anyhow::Result<ComplexityReport>),
    Churn(anyhow::Result<CodeChurnAnalysis>),
    DeadCode(anyhow::Result<crate::models::dead_code::DeadCodeRankingResult>),
    #[allow(dead_code)] // Will be used when SATD analysis is re-enabled
    Satd(anyhow::Result<SATDAnalysisResult>),
    Dag(anyhow::Result<DependencyGraph>),
}

// Analysis helper functions

async fn analyze_ast_contexts(
    project_path: &std::path::Path,
) -> anyhow::Result<Vec<EnhancedFileContext>> {
    use crate::services::context::analyze_rust_file;
    use walkdir::WalkDir;

    let mut enhanced_contexts = Vec::new();

    // Analyze all Rust files in the project
    for entry in WalkDir::new(project_path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Skip non-Rust files and target directory
        if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }

        if path.to_string_lossy().contains("/target/") {
            continue;
        }

        // Analyze the file
        match analyze_rust_file(path).await {
            Ok(file_context) => {
                enhanced_contexts.push(EnhancedFileContext {
                    base: file_context,
                    complexity_metrics: None, // Will be filled by complexity analysis
                    churn_metrics: None,      // Will be filled by churn analysis
                    defects: DefectAnnotations {
                        dead_code: None,
                        technical_debt: Vec::new(),
                        complexity_violations: Vec::new(),
                        defect_probability: 0.0,
                    },
                    symbol_id: format!("rust::{}", path.to_string_lossy()),
                });
            }
            Err(e) => {
                debug!("Failed to analyze Rust file {:?}: {}", path, e);
            }
        }
    }

    // Also analyze TypeScript and Python files
    for entry in WalkDir::new(project_path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let extension = path.extension().and_then(|s| s.to_str());
        match extension {
            Some("ts") | Some("js") | Some("tsx") | Some("jsx") => {
                if let Ok(file_context) = analyze_typescript_file(path).await {
                    enhanced_contexts.push(EnhancedFileContext {
                        base: file_context,
                        complexity_metrics: None,
                        churn_metrics: None,
                        defects: DefectAnnotations {
                            dead_code: None,
                            technical_debt: Vec::new(),
                            complexity_violations: Vec::new(),
                            defect_probability: 0.0,
                        },
                        symbol_id: format!("typescript::{}", path.to_string_lossy()),
                    });
                }
            }
            Some("py") => {
                if let Ok(file_context) = analyze_python_file(path).await {
                    enhanced_contexts.push(EnhancedFileContext {
                        base: file_context,
                        complexity_metrics: None,
                        churn_metrics: None,
                        defects: DefectAnnotations {
                            dead_code: None,
                            technical_debt: Vec::new(),
                            complexity_violations: Vec::new(),
                            defect_probability: 0.0,
                        },
                        symbol_id: format!("python::{}", path.to_string_lossy()),
                    });
                }
            }
            _ => continue,
        }
    }

    Ok(enhanced_contexts)
}

async fn analyze_typescript_file(path: &std::path::Path) -> anyhow::Result<FileContext> {
    ast_typescript::analyze_typescript_file(path)
        .await
        .map_err(|e| anyhow::anyhow!("TypeScript analysis failed: {}", e))
}

async fn analyze_python_file(path: &std::path::Path) -> anyhow::Result<FileContext> {
    ast_python::analyze_python_file(path)
        .await
        .map_err(|e| anyhow::anyhow!("Python analysis failed: {}", e))
}

async fn analyze_complexity(project_path: &std::path::Path) -> anyhow::Result<ComplexityReport> {
    use crate::services::complexity::aggregate_results;
    use walkdir::WalkDir;

    let mut file_metrics = Vec::new();

    // Simple implementation - analyze Rust files
    for entry in WalkDir::new(project_path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }

        // Skip target and other excluded directories
        if path.to_string_lossy().contains("/target/") {
            continue;
        }

        if let Ok(Some(metrics)) = analyze_rust_file_complexity(path).await {
            file_metrics.push(metrics);
        }
    }

    Ok(aggregate_results(file_metrics))
}

async fn analyze_rust_file_complexity(
    path: &std::path::Path,
) -> anyhow::Result<Option<FileComplexityMetrics>> {
    use crate::services::ast_rust;

    // Try to use existing complexity analysis
    match ast_rust::analyze_rust_file_with_complexity(path).await {
        Ok(metrics) => Ok(Some(metrics)),
        Err(_) => {
            // Fallback to simple implementation
            Ok(Some(FileComplexityMetrics {
                path: path.to_string_lossy().to_string(),
                total_complexity: crate::services::complexity::ComplexityMetrics::default(),
                functions: Vec::new(),
                classes: Vec::new(),
            }))
        }
    }
}

async fn analyze_churn(
    project_path: &std::path::Path,
    days: u32,
) -> anyhow::Result<CodeChurnAnalysis> {
    GitAnalysisService::analyze_code_churn(project_path, days)
        .map_err(|e| anyhow::anyhow!("Churn analysis failed: {}", e))
}

async fn analyze_dead_code(
    project_path: &std::path::Path,
) -> anyhow::Result<crate::models::dead_code::DeadCodeRankingResult> {
    use crate::models::dead_code::DeadCodeAnalysisConfig;

    let mut analyzer = DeadCodeAnalyzer::new(10000);
    let config = DeadCodeAnalysisConfig {
        include_unreachable: true,
        include_tests: false,
        min_dead_lines: 10,
    };

    analyzer.analyze_with_ranking(project_path, config).await
}

#[allow(dead_code)] // Will be used when SATD analysis is re-enabled
async fn analyze_satd(project_path: &std::path::Path) -> anyhow::Result<SATDAnalysisResult> {
    let detector = SATDDetector::new();
    detector
        .analyze_project(project_path, false)
        .await
        .map_err(|e| anyhow::anyhow!("SATD analysis failed: {}", e))
}

async fn analyze_dag(
    project_path: &std::path::Path,
    dag_type: DagType,
) -> anyhow::Result<DependencyGraph> {
    use crate::services::dag_builder::DagBuilder;

    match dag_type {
        DagType::CallGraph => {
            // Use existing context analysis and filter for call relationships
            let context = crate::services::context::analyze_project(project_path, "rust")
                .await
                .map_err(|e| anyhow::anyhow!("Context analysis failed: {}", e))?;
            let graph = DagBuilder::build_from_project(&context);
            Ok(crate::services::dag_builder::filter_call_edges(graph))
        }
        DagType::ImportGraph => {
            // Use existing context analysis and filter for import relationships
            let context = crate::services::context::analyze_project(project_path, "rust")
                .await
                .map_err(|e| anyhow::anyhow!("Context analysis failed: {}", e))?;
            let graph = DagBuilder::build_from_project(&context);
            Ok(crate::services::dag_builder::filter_import_edges(graph))
        }
        DagType::Inheritance => {
            // Use existing context analysis and filter for inheritance relationships
            let context = crate::services::context::analyze_project(project_path, "rust")
                .await
                .map_err(|e| anyhow::anyhow!("Context analysis failed: {}", e))?;
            let graph = DagBuilder::build_from_project(&context);
            Ok(crate::services::dag_builder::filter_inheritance_edges(
                graph,
            ))
        }
        DagType::FullDependency => {
            // Use existing context analysis for full dependency graph
            let context = crate::services::context::analyze_project(project_path, "rust")
                .await
                .map_err(|e| anyhow::anyhow!("Context analysis failed: {}", e))?;
            Ok(DagBuilder::build_from_project(&context))
        }
    }
}
