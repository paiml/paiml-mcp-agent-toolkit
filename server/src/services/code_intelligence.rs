//! Unified code intelligence interface
//!
//! Provides a comprehensive analysis interface that combines DAG representation,
//! duplicate detection, dead code analysis, and more into a single API.

use crate::models::unified_ast::AstDag;
use crate::services::{
    context::analyze_project,
    dag_builder::DagBuilder,
    dead_code_analyzer::{DeadCodeAnalyzer, DeadCodeReport},
    duplicate_detector::CloneReport,
    mermaid_generator::{MermaidGenerator, MermaidOptions},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Analysis request configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisRequest {
    pub project_path: String,
    pub analysis_types: Vec<AnalysisType>,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub max_depth: Option<usize>,
    pub parallel: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AnalysisType {
    DuplicateDetection,
    DeadCodeAnalysis,
    ComplexityMetrics,
    DependencyGraph,
    DefectPrediction,
    NameSimilarity,
}

impl AnalysisRequest {
    pub fn cache_key(&self) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(self.project_path.as_bytes());
        for t in &self.analysis_types {
            hasher.update(format!("{t:?}").as_bytes());
        }
        format!("{:x}", hasher.finalize())
    }
}

/// Comprehensive analysis report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisReport {
    pub duplicates: Option<CloneReport>,
    pub dead_code: Option<DeadCodeReport>,
    pub complexity_metrics: Option<ComplexityReport>,
    pub dependency_graph: Option<DependencyGraphReport>,
    pub defect_predictions: Option<Vec<DefectScore>>,
    pub graph_metrics: Option<GraphMetricsReport>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityReport {
    pub total_files: usize,
    pub average_complexity: f32,
    pub hotspots: Vec<ComplexityHotspot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityHotspot {
    pub file_path: String,
    pub function_name: String,
    pub cyclomatic_complexity: u32,
    pub cognitive_complexity: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraphReport {
    pub nodes: usize,
    pub edges: usize,
    pub circular_dependencies: Vec<Vec<String>>,
    pub mermaid_diagram: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefectScore {
    pub entity: String,
    pub score: f32,
    pub confidence: f32,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetricsReport {
    pub centrality_scores: Vec<CentralityScore>,
    pub clustering_coefficient: f32,
    pub modularity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CentralityScore {
    pub node: String,
    pub degree: f32,
    pub betweenness: f32,
    pub closeness: f32,
    pub pagerank: f32,
}

/// Unified cache for analysis results
pub struct UnifiedCache {
    cache: Arc<RwLock<lru::LruCache<String, AnalysisReport>>>,
}

impl UnifiedCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(lru::LruCache::new(
                std::num::NonZeroUsize::new(capacity).unwrap(),
            ))),
        }
    }

    pub async fn get(&self, key: &str) -> Option<AnalysisReport> {
        self.cache.read().await.peek(key).cloned()
    }

    pub async fn put(&self, key: String, report: AnalysisReport) {
        self.cache.write().await.put(key, report);
    }
}

/// Main code intelligence interface
pub struct CodeIntelligence {
    dag: Arc<RwLock<AstDag>>,
    deadcode: Arc<RwLock<DeadCodeAnalyzer>>,
    cache: Arc<UnifiedCache>,
}

impl Default for CodeIntelligence {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeIntelligence {
    pub fn new() -> Self {
        let dag = Arc::new(RwLock::new(AstDag::new()));

        Self {
            dag: dag.clone(),
            deadcode: Arc::new(RwLock::new(DeadCodeAnalyzer::new(10000))), // Initial capacity
            cache: Arc::new(UnifiedCache::new(100)),
        }
    }

    /// Perform comprehensive analysis based on request
    pub async fn analyze_comprehensive(
        &self,
        req: AnalysisRequest,
    ) -> anyhow::Result<AnalysisReport> {
        let cache_key = req.cache_key();

        // Check cache first
        if let Some(cached) = self.cache.get(&cache_key).await {
            return Ok(cached);
        }

        let mut report = AnalysisReport {
            duplicates: None,
            dead_code: None,
            complexity_metrics: None,
            dependency_graph: None,
            defect_predictions: None,
            graph_metrics: None,
            timestamp: Utc::now(),
        };

        // First, analyze the project and build the AST DAG
        self.analyze_project(&req.project_path).await?;

        // Handle dependency graph analysis directly
        if req.analysis_types.contains(&AnalysisType::DependencyGraph) {
            // Create project context for dependency graph generation
            if let Ok(project_context) =
                analyze_project(std::path::Path::new(&req.project_path), "rust").await
            {
                // Build dependency graph using DagBuilder with limit
                let dependency_graph =
                    DagBuilder::build_from_project_with_limit(&project_context, 50);

                // Generate Mermaid diagram
                let mermaid_options = MermaidOptions {
                    max_depth: req.max_depth,
                    filter_external: false,
                    group_by_module: true,
                    show_complexity: true,
                };
                let mermaid_generator = MermaidGenerator::new(mermaid_options);
                let mermaid_diagram = mermaid_generator.generate(&dependency_graph);

                // Store results in report
                report.dependency_graph = Some(DependencyGraphReport {
                    nodes: dependency_graph.nodes.len(),
                    edges: dependency_graph.edges.len(),
                    circular_dependencies: Vec::new(), // TODO: Implement cycle detection
                    mermaid_diagram,
                });
            }
        }

        // Run other requested analyses in parallel
        let futures = self.build_analysis_futures(&req, &mut report);

        // Wait for all analyses to complete
        futures::future::join_all(futures).await;

        // Cache the results
        self.cache.put(cache_key, report.clone()).await;

        Ok(report)
    }

    /// Analyze project and build AST DAG
    async fn analyze_project(&self, project_path: &str) -> anyhow::Result<()> {
        use crate::models::unified_ast::{AstKind, Language, NodeMetadata, UnifiedAstNode};
        use crate::services::context::analyze_project as analyze_project_context;
        use std::path::Path;

        // Analyze the project using existing context analysis
        let project_context = analyze_project_context(
            Path::new(project_path),
            "rust", // Default to Rust for now, could be auto-detected
        )
        .await?;

        let mut dag = self.dag.write().await;

        // Convert project context to unified AST nodes
        for file in &project_context.files {
            for item in &file.items {
                use crate::services::context::AstItem;

                let node = match item {
                    AstItem::Function { name, line, .. } => {
                        let mut node = UnifiedAstNode::new(
                            AstKind::Function(crate::models::unified_ast::FunctionKind::Regular),
                            Language::Rust,
                        );
                        node.parent = crate::models::unified_ast::INVALID_NODE_KEY;
                        node.first_child = crate::models::unified_ast::INVALID_NODE_KEY;
                        node.next_sibling = crate::models::unified_ast::INVALID_NODE_KEY;
                        node.source_range = (*line as u32)..(*line as u32 + 1);
                        node.semantic_hash = name
                            .as_bytes()
                            .iter()
                            .fold(0u64, |acc, &b| acc.wrapping_mul(31).wrapping_add(b as u64));
                        node.structural_hash = 0x12345678;
                        node
                    }
                    AstItem::Struct {
                        name,
                        line,
                        fields_count,
                        ..
                    } => {
                        let mut node = UnifiedAstNode::new(
                            AstKind::Class(crate::models::unified_ast::ClassKind::Struct),
                            Language::Rust,
                        );
                        node.parent = crate::models::unified_ast::INVALID_NODE_KEY;
                        node.first_child = crate::models::unified_ast::INVALID_NODE_KEY;
                        node.next_sibling = crate::models::unified_ast::INVALID_NODE_KEY;
                        node.source_range = (*line as u32)..(*line as u32 + 1);
                        node.semantic_hash = name
                            .as_bytes()
                            .iter()
                            .fold(0u64, |acc, &b| acc.wrapping_mul(31).wrapping_add(b as u64));
                        node.structural_hash = 0x87654321;
                        node.name_vector = *fields_count as u64;
                        node.metadata = NodeMetadata {
                            raw: *fields_count as u64,
                        };
                        node
                    }
                    AstItem::Trait { name, line, .. } => {
                        let mut node = UnifiedAstNode::new(
                            AstKind::Class(crate::models::unified_ast::ClassKind::Trait),
                            Language::Rust,
                        );
                        node.parent = crate::models::unified_ast::INVALID_NODE_KEY;
                        node.first_child = crate::models::unified_ast::INVALID_NODE_KEY;
                        node.next_sibling = crate::models::unified_ast::INVALID_NODE_KEY;
                        node.source_range = (*line as u32)..(*line as u32 + 1);
                        node.semantic_hash = name
                            .as_bytes()
                            .iter()
                            .fold(0u64, |acc, &b| acc.wrapping_mul(31).wrapping_add(b as u64));
                        node.structural_hash = 0x13579BDF;
                        node
                    }
                    AstItem::Module { name, line, .. } => {
                        let mut node = UnifiedAstNode::new(
                            AstKind::Module(crate::models::unified_ast::ModuleKind::File),
                            Language::Rust,
                        );
                        node.parent = crate::models::unified_ast::INVALID_NODE_KEY;
                        node.first_child = crate::models::unified_ast::INVALID_NODE_KEY;
                        node.next_sibling = crate::models::unified_ast::INVALID_NODE_KEY;
                        node.source_range = (*line as u32)..(*line as u32 + 1);
                        node.semantic_hash = name
                            .as_bytes()
                            .iter()
                            .fold(0u64, |acc, &b| acc.wrapping_mul(31).wrapping_add(b as u64));
                        node.structural_hash = 0x24681ACE;
                        node
                    }
                    AstItem::Enum {
                        name,
                        line,
                        variants_count,
                        ..
                    } => {
                        let mut node = UnifiedAstNode::new(
                            AstKind::Class(crate::models::unified_ast::ClassKind::Enum),
                            Language::Rust,
                        );
                        node.parent = crate::models::unified_ast::INVALID_NODE_KEY;
                        node.first_child = crate::models::unified_ast::INVALID_NODE_KEY;
                        node.next_sibling = crate::models::unified_ast::INVALID_NODE_KEY;
                        node.source_range = (*line as u32)..(*line as u32 + 1);
                        node.semantic_hash = name
                            .as_bytes()
                            .iter()
                            .fold(0u64, |acc, &b| acc.wrapping_mul(31).wrapping_add(b as u64));
                        node.structural_hash = 0x97531BDF;
                        node.name_vector = *variants_count as u64;
                        node.metadata = NodeMetadata {
                            raw: *variants_count as u64,
                        };
                        node
                    }
                    _ => continue, // Skip other types for now
                };

                dag.nodes.push(node);
            }
        }

        // Increment generation counter
        dag.generation
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        Ok(())
    }

    /// Build analysis futures based on request
    fn build_analysis_futures<'a>(
        &'a self,
        req: &'a AnalysisRequest,
        _report: &'a mut AnalysisReport,
    ) -> Vec<std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>>> {
        let mut futures = Vec::new();

        for analysis_type in &req.analysis_types {
            match analysis_type {
                AnalysisType::DuplicateDetection => {
                    // Duplicate detection is handled in the deep context analysis
                    // No additional processing needed here
                }

                AnalysisType::DeadCodeAnalysis => {
                    let deadcode = self.deadcode.clone();
                    let dag = self.dag.clone();
                    futures.push(Box::pin(async move {
                        let dag_guard = dag.read().await;
                        let mut analyzer = deadcode.write().await;
                        let _dead_report = analyzer.analyze(&dag_guard);
                        // TODO: Store in report
                    })
                        as std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>);
                }

                AnalysisType::DependencyGraph => {
                    // Dependency graph is handled synchronously in analyze_comprehensive
                    // No future needed here
                }

                // TODO: Implement other analysis types
                _ => {}
            }
        }

        futures
    }

    /// Get current DAG statistics
    pub async fn get_dag_stats(&self) -> (usize, u32) {
        let dag = self.dag.read().await;
        (dag.nodes.len(), dag.generation())
    }
}

/// Enhanced DAG analysis specifically for the CLI
pub async fn analyze_dag_enhanced(
    project_path: &str,
    _dag_type: crate::cli::DagType,
    max_depth: Option<usize>,
    _filter_external: bool,
    _show_complexity: bool,
    include_duplicates: bool,
    include_dead_code: bool,
) -> anyhow::Result<String> {
    let intelligence = CodeIntelligence::new();

    // Build analysis request
    let mut analysis_types = vec![AnalysisType::DependencyGraph];
    if include_duplicates {
        analysis_types.push(AnalysisType::DuplicateDetection);
    }
    if include_dead_code {
        analysis_types.push(AnalysisType::DeadCodeAnalysis);
    }

    let request = AnalysisRequest {
        project_path: project_path.to_string(),
        analysis_types,
        include_patterns: vec![],
        exclude_patterns: vec![],
        max_depth,
        parallel: true,
    };

    // Run comprehensive analysis
    let report = intelligence.analyze_comprehensive(request).await?;

    // Format output based on dag_type
    let mut output = String::new();

    // Add dependency graph if available
    if let Some(dep_graph) = &report.dependency_graph {
        output.push_str(&dep_graph.mermaid_diagram);
        output.push_str("\n\n");
    }

    // Add duplicate detection results if requested
    if include_duplicates {
        if let Some(duplicates) = &report.duplicates {
            output.push_str("%% Duplicate Detection Results:\n");
            output.push_str(&format!(
                "%% Total clone groups: {}\n",
                duplicates.summary.clone_groups
            ));
            output.push_str(&format!(
                "%% Clone coverage: {:.1}%\n",
                duplicates.summary.duplication_ratio * 100.0
            ));
        }
    }

    // Add dead code analysis results if requested
    if include_dead_code {
        if let Some(dead_code) = &report.dead_code {
            output.push_str("%% Dead Code Analysis:\n");
            output.push_str(&format!(
                "%% Dead code percentage: {:.1}%\n",
                dead_code.summary.percentage_dead
            ));
            output.push_str(&format!(
                "%% Dead functions: {}\n",
                dead_code.dead_functions.len()
            ));
            output.push_str(&format!(
                "%% Dead classes: {}\n",
                dead_code.dead_classes.len()
            ));
        }
    }

    // Add statistics
    let (node_count, generation) = intelligence.get_dag_stats().await;
    output.push_str("\n%% Graph Statistics:\n");
    output.push_str(&format!("%% Total nodes: {node_count}\n"));
    output.push_str(&format!("%% Generation: {generation}\n"));
    output.push_str(&format!("%% Analysis timestamp: {}\n", Utc::now()));

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analysis_request_cache_key() {
        let req = AnalysisRequest {
            project_path: "/test/project".to_string(),
            analysis_types: vec![AnalysisType::DuplicateDetection],
            include_patterns: vec![],
            exclude_patterns: vec![],
            max_depth: None,
            parallel: true,
        };

        let key1 = req.cache_key();
        let key2 = req.cache_key();

        assert_eq!(key1, key2);
    }

    #[tokio::test]
    async fn test_unified_cache() {
        let cache = UnifiedCache::new(10);

        let report = AnalysisReport {
            duplicates: None,
            dead_code: None,
            complexity_metrics: None,
            dependency_graph: None,
            defect_predictions: None,
            graph_metrics: None,
            timestamp: Utc::now(),
        };

        cache.put("test_key".to_string(), report.clone()).await;

        let cached = cache.get("test_key").await;
        assert!(cached.is_some());
    }

    #[tokio::test]
    async fn test_code_intelligence_creation() {
        let intelligence = CodeIntelligence::new();
        let (nodes, gen) = intelligence.get_dag_stats().await;

        assert_eq!(nodes, 0);
        assert_eq!(gen, 0);
    }
}
