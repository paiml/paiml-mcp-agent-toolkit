use crate::models::{churn::CodeChurnAnalysis, dag::DependencyGraph};
use crate::services::context::FileContext;
use crate::services::{
    complexity::{ComplexityReport, FileComplexityMetrics},
    dead_code_analyzer::DeadCodeAnalyzer,
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
        use tokio::task::JoinSet;

        let mut join_set = JoinSet::new();

        // AST Analysis
        if self.config.include_analyses.contains(&AnalysisType::Ast) {
            let path = project_path.to_path_buf();
            join_set.spawn(async move { AnalysisResult::Ast(analyze_ast_contexts(&path).await) });
        }

        // Complexity Analysis
        if self
            .config
            .include_analyses
            .contains(&AnalysisType::Complexity)
        {
            let path = project_path.to_path_buf();
            join_set
                .spawn(async move { AnalysisResult::Complexity(analyze_complexity(&path).await) });
        }

        // Churn Analysis
        if self.config.include_analyses.contains(&AnalysisType::Churn) {
            let path = project_path.to_path_buf();
            let days = self.config.period_days;
            join_set.spawn(async move { AnalysisResult::Churn(analyze_churn(&path, days).await) });
        }

        // Dead Code Analysis
        if self
            .config
            .include_analyses
            .contains(&AnalysisType::DeadCode)
        {
            let path = project_path.to_path_buf();
            join_set.spawn(async move { AnalysisResult::DeadCode(analyze_dead_code(&path).await) });
        }

        // SATD Analysis
        if self.config.include_analyses.contains(&AnalysisType::Satd) {
            let path = project_path.to_path_buf();
            join_set.spawn(async move {
                // Use spawn_blocking to avoid Send issues with SATD detector
                let result = tokio::task::spawn_blocking(move || {
                    tokio::runtime::Handle::current().block_on(async { analyze_satd(&path).await })
                })
                .await
                .unwrap_or_else(|_| Err(anyhow::anyhow!("SATD analysis failed")));
                AnalysisResult::Satd(result)
            });
        }

        // Collect results
        let mut results = ParallelAnalysisResults::default();

        while let Some(result) = join_set.join_next().await {
            match result? {
                AnalysisResult::Ast(Ok(ast_contexts)) => results.ast_contexts = Some(ast_contexts),
                AnalysisResult::Complexity(Ok(complexity)) => {
                    results.complexity_report = Some(complexity)
                }
                AnalysisResult::Churn(Ok(churn)) => results.churn_analysis = Some(churn),
                AnalysisResult::DeadCode(Ok(dead_code)) => {
                    results.dead_code_results = Some(dead_code)
                }
                AnalysisResult::Satd(Ok(satd)) => results.satd_results = Some(satd),
                AnalysisResult::Ast(Err(e)) => debug!("AST analysis failed: {}", e),
                AnalysisResult::Complexity(Err(e)) => debug!("Complexity analysis failed: {}", e),
                AnalysisResult::Churn(Err(e)) => debug!("Churn analysis failed: {}", e),
                AnalysisResult::DeadCode(Err(e)) => debug!("Dead code analysis failed: {}", e),
                AnalysisResult::Satd(Err(e)) => debug!("SATD analysis failed: {}", e),
            }
        }

        Ok(results)
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
        let mut defect_summary = DefectSummary {
            total_defects: 0,
            by_severity: HashMap::new(),
            by_type: HashMap::new(),
            defect_density: 0.0,
        };

        let hotspots = Vec::new();

        // Count defects from dead code analysis
        if let Some(ref dead_code) = analyses.dead_code_results {
            defect_summary.total_defects += dead_code.summary.dead_functions;
            defect_summary
                .by_type
                .insert("dead_code".to_string(), dead_code.summary.dead_functions);
        }

        // Count defects from SATD analysis
        if let Some(ref satd) = analyses.satd_results {
            defect_summary.total_defects += satd.items.len();
            defect_summary
                .by_type
                .insert("technical_debt".to_string(), satd.items.len());
        }

        // Generate hotspots (simplified implementation)
        // TODO: Implement sophisticated defect correlation

        Ok((defect_summary, hotspots))
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
}

// Analysis helper functions

async fn analyze_ast_contexts(
    _project_path: &std::path::Path,
) -> anyhow::Result<Vec<EnhancedFileContext>> {
    // TODO: Implement AST analysis with enhancement
    Ok(Vec::new())
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
