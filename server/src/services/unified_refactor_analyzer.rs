use crate::models::refactor::{MetricSet, RefactorOp};
use crate::models::unified_ast::UnifiedAstNode;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[async_trait]
pub trait UnifiedAnalyzer: Send + Sync {
    /// Compute metrics for a given AST node
    async fn compute_metrics(&self, node: &UnifiedAstNode) -> Result<MetricSet, AnalyzerError>;

    /// Suggest refactoring operations based on metrics
    async fn suggest_refactors(
        &self,
        metrics: &MetricSet,
    ) -> Result<Vec<RefactorPlan>, AnalyzerError>;

    /// Apply a transformation to an AST node
    async fn apply_transform(
        &self,
        ast: &UnifiedAstNode,
        plan: &RefactorPlan,
    ) -> Result<AstDelta, AnalyzerError>;

    /// Update metrics incrementally based on changes
    async fn update_incremental(&self, delta: &AstDelta) -> Result<MetricDelta, AnalyzerError>;

    /// Get the language this analyzer supports
    fn language(&self) -> Language;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactorPlan {
    pub operation: RefactorOp,
    pub confidence: f32,
    pub estimated_improvement: EstimatedImprovement,
    pub risk_level: RiskLevel,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EstimatedImprovement {
    pub complexity_reduction: (u16, u16), // (cyclomatic, cognitive)
    pub tdg_improvement: f32,
    pub lines_saved: i32,
    pub maintainability_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,    // Safe, minimal impact
    Medium, // Some test changes expected
    High,   // Significant behavioral changes
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstDelta {
    pub nodes_added_count: u32,
    pub nodes_removed: Vec<NodeId>,
    pub nodes_modified: Vec<NodeModification>,
    pub file_hash_before: u64,
    pub file_hash_after: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeModification {
    pub node_id: NodeId,
    pub old_content: String,
    pub new_content: String,
    pub source_range_changed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDelta {
    pub complexity_change: (i16, i16), // Can be negative for improvements
    pub tdg_change: f32,
    pub dead_code_change: i32,
    pub satd_change: i32,
    pub provability_change: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct NodeId(pub u64);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Language {
    Rust,
    TypeScript,
    Python,
    C,
    Cpp,
    Java,
    Go,
    Kotlin,
    Swift,
    JavaScript,
    Other(u8),
}

#[derive(Debug, thiserror::Error)]
pub enum AnalyzerError {
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Invalid AST structure: {0}")]
    InvalidAst(String),
    #[error("Transformation failed: {0}")]
    TransformationFailed(String),
}

pub struct AnalyzerPool {
    analyzers: std::collections::HashMap<Language, Arc<dyn UnifiedAnalyzer>>,
}

impl AnalyzerPool {
    pub fn new() -> Self {
        Self {
            analyzers: std::collections::HashMap::new(),
        }
    }

    pub fn register(&mut self, analyzer: Arc<dyn UnifiedAnalyzer>) {
        self.analyzers.insert(analyzer.language(), analyzer);
    }

    pub fn get(&self, language: Language) -> Option<Arc<dyn UnifiedAnalyzer>> {
        self.analyzers.get(&language).cloned()
    }

    pub fn languages(&self) -> Vec<Language> {
        self.analyzers.keys().copied().collect()
    }
}

impl Default for AnalyzerPool {
    fn default() -> Self {
        Self::new()
    }
}

// Rust analyzer implementation
pub struct RustAnalyzer;

impl RustAnalyzer {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl UnifiedAnalyzer for RustAnalyzer {
    async fn compute_metrics(&self, node: &UnifiedAstNode) -> Result<MetricSet, AnalyzerError> {
        // Create a mutable analyzer instance for this computation
        let mut analyzer = crate::services::verified_complexity::VerifiedComplexityAnalyzer::new();

        // Analyze the function complexity
        let complexity = analyzer.analyze_function(node);

        // Calculate TDG based on complexity
        let tdg_score = ((complexity.cyclomatic as f32 / 10.0) + (complexity.cognitive as f32 / 15.0)) / 2.0;
        
        // Analyze dead code by checking for unused functions
        let dead_code = vec![false; 10]; // Simplified: assume 10 symbols, none dead
        
        // Count SATD markers in the node's text (would need actual text in real impl)
        let satd_count = 0; // Would need to analyze actual source text
        
        // Simple provability score based on complexity
        let provability = 1.0 / (1.0 + complexity.cyclomatic as f32 / 20.0);
        
        Ok(MetricSet {
            complexity: (complexity.cyclomatic as u16, complexity.cognitive as u16),
            tdg_score,
            dead_code,
            satd_count,
            provability,
        })
    }

    async fn suggest_refactors(
        &self,
        metrics: &MetricSet,
    ) -> Result<Vec<RefactorPlan>, AnalyzerError> {
        let mut plans = Vec::new();

        // High complexity suggests function extraction
        if metrics.complexity.0 > 15 || metrics.complexity.1 > 20 {
            plans.push(RefactorPlan {
                operation: RefactorOp::ExtractFunction {
                    name: "extracted_helper".to_string(),
                    start: crate::models::refactor::BytePos {
                        byte: 0,
                        line: 1,
                        column: 1,
                    },
                    end: crate::models::refactor::BytePos {
                        byte: 100,
                        line: 10,
                        column: 1,
                    },
                    params: vec!["param1".to_string(), "param2".to_string()],
                },
                confidence: 0.8,
                estimated_improvement: EstimatedImprovement {
                    complexity_reduction: (
                        (metrics.complexity.0 * 3 / 4),
                        (metrics.complexity.1 * 2 / 3),
                    ),
                    tdg_improvement: 0.3,
                    lines_saved: -5, // Adding function might increase total lines
                    maintainability_score: 0.4,
                },
                risk_level: RiskLevel::Medium,
                explanation: "Extract complex logic into separate function to improve readability and testability".to_string(),
            });
        }

        // High TDG suggests simplification
        if metrics.tdg_score > 2.0 {
            plans.push(RefactorPlan {
                operation: RefactorOp::SimplifyExpression {
                    expr: "complex_expression".to_string(),
                    simplified: "simplified_expression".to_string(),
                },
                confidence: 0.6,
                estimated_improvement: EstimatedImprovement {
                    complexity_reduction: (2, 3),
                    tdg_improvement: 0.5,
                    lines_saved: 2,
                    maintainability_score: 0.3,
                },
                risk_level: RiskLevel::Low,
                explanation: "Simplify complex expressions to reduce code complexity".to_string(),
            });
        }

        Ok(plans)
    }

    async fn apply_transform(
        &self,
        ast: &UnifiedAstNode,
        plan: &RefactorPlan,
    ) -> Result<AstDelta, AnalyzerError> {
        // Simulate applying a transformation
        let mut nodes_modified = Vec::new();
        let mut nodes_removed = Vec::new();
        let nodes_added_count = match &plan.operation {
            RefactorOp::ExtractFunction { name, .. } => {
                // Simulating extracting a function
                nodes_modified.push(NodeModification {
                    node_id: NodeId(ast.source_range.start as u64),
                    old_content: "inline_code".to_string(),
                    new_content: format!("{}()", name),
                    source_range_changed: true,
                });
                1 // One new function node added
            }
            RefactorOp::FlattenNesting { .. } => {
                // Simulating flattening nested conditions
                nodes_modified.push(NodeModification {
                    node_id: NodeId(ast.source_range.start as u64),
                    old_content: "if (a) { if (b) { } }".to_string(),
                    new_content: "if (a && b) { }".to_string(),
                    source_range_changed: true,
                });
                0
            }
            RefactorOp::RemoveSatd { .. } => {
                // Removing a SATD comment
                nodes_removed.push(NodeId(ast.source_range.start as u64));
                0
            }
            _ => 0,
        };
        
        Ok(AstDelta {
            nodes_added_count,
            nodes_removed,
            nodes_modified,
            file_hash_before: 12345, // Would calculate actual hash
            file_hash_after: 12346,  // Would calculate actual hash
        })
    }

    async fn update_incremental(&self, delta: &AstDelta) -> Result<MetricDelta, AnalyzerError> {
        // Calculate metric changes based on the AST delta
        let nodes_modified_count = delta.nodes_modified.len() as i16;
        let nodes_removed_count = delta.nodes_removed.len() as i16;
        let nodes_added_count = delta.nodes_added_count as i16;
        
        // Estimate complexity changes based on node changes
        let complexity_change = (
            -nodes_removed_count + (nodes_added_count / 2), // Cyclomatic
            -nodes_removed_count + (nodes_added_count / 3), // Cognitive
        );
        
        // TDG changes based on modifications
        let tdg_change = -0.1 * nodes_modified_count as f32;
        
        // Dead code might be removed
        let dead_code_change = -nodes_removed_count as i32;
        
        // SATD changes (removing nodes might remove SATD)
        let satd_change = if nodes_removed_count > 0 { -1 } else { 0 };
        
        // Provability improves with simplification
        let provability_change = if nodes_modified_count > 0 { 0.05 } else { 0.0 };
        
        Ok(MetricDelta {
            complexity_change,
            tdg_change,
            dead_code_change,
            satd_change,
            provability_change,
        })
    }

    fn language(&self) -> Language {
        Language::Rust
    }
}

impl Default for RustAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl std::hash::Hash for Language {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        if let Language::Other(val) = self {
            val.hash(state);
        }
    }
}

impl Language {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "rs" => Language::Rust,
            "ts" | "tsx" => Language::TypeScript,
            "js" | "jsx" => Language::JavaScript,
            "py" => Language::Python,
            "c" => Language::C,
            "cpp" | "cc" | "cxx" => Language::Cpp,
            "java" => Language::Java,
            "go" => Language::Go,
            "kt" | "kts" => Language::Kotlin,
            "swift" => Language::Swift,
            _ => Language::Other(0),
        }
    }
}
