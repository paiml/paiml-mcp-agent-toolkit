#![cfg_attr(coverage_nightly, coverage(off))]
//! Canonical query framework for standardized code analysis.
//!
//! This module provides a unified interface for executing various code analysis
//! queries in a consistent, cacheable manner. It enables building complex
//! analysis pipelines by composing simple, reusable query components.
//!
//! # Architecture
//!
//! The framework consists of:
//! - `CanonicalQuery` trait: Standard interface for all analysis queries
//! - `AnalysisContext`: Shared context containing AST, call graphs, and metrics
//! - Query implementations: Specific analyses (complexity, coupling, defects)
//! - Result caching: Automatic caching based on query ID and project path
//!
//! # Query Types
//!
//! - **Structural Queries**: AST analysis, dependency graphs, call graphs
//! - **Metric Queries**: Complexity, coupling, cohesion measurements
//! - **Quality Queries**: Defect prediction, code smell detection
//! - **Evolution Queries**: Code churn, hotspot analysis
//!
//! # Example
//!
//! ```ignore
//! use pmat::services::canonical_query::{CanonicalQuery, AnalysisContext, QueryResult};
//! use std::path::Path;
//!
//! // Define a custom query
//! struct HighComplexityQuery {
//!     threshold: u32,
//! }
//!
//! impl CanonicalQuery for HighComplexityQuery {
//!     fn query_id(&self) -> &'static str {
//!         "high_complexity_functions"
//!     }
//!
//!     fn execute(&self, ctx: &AnalysisContext) -> anyhow::Result<QueryResult> {
//!         let mut results = Vec::new();
//!         
//!         for (name, metrics) in &ctx.complexity_map {
//!             if metrics.cyclomatic > self.threshold as u16 {
//!                 results.push(serde_json::json!({
//!                     "function": name,
//!                     "complexity": metrics.cyclomatic,
//!                 }));
//!             }
//!         }
//!         
//!         // In real implementation, would return proper QueryResult with diagram
//!         todo!("Create proper QueryResult")
//!     }
//! }
//! ```

use anyhow::Result;
use chrono::{DateTime, Utc};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Core trait for canonical analysis queries
pub trait CanonicalQuery: Send + Sync {
    fn query_id(&self) -> &'static str;
    fn execute(&self, ctx: &AnalysisContext) -> Result<QueryResult>;
    fn cache_key(&self, project_path: &Path) -> String {
        format!("{}:{}", self.query_id(), project_path.display())
    }
}

/// Analysis context containing all data needed for queries
#[derive(Debug, Clone)]
pub struct AnalysisContext {
    pub project_path: std::path::PathBuf,
    pub ast_dag: crate::models::dag::DependencyGraph,
    pub call_graph: CallGraph,
    pub complexity_map: FxHashMap<String, crate::services::complexity::ComplexityMetrics>,
    pub churn_analysis: Option<crate::models::churn::CodeChurnAnalysis>,
}

/// Call graph representation for component relationship analysis
#[derive(Debug, Clone, Default)]
pub struct CallGraph {
    pub nodes: Vec<CallNode>,
    pub edges: Vec<CallEdge>,
}

#[derive(Debug, Clone)]
/// Call node.
pub struct CallNode {
    pub id: String,
    pub name: String,
    pub module_path: String,
    pub node_type: CallNodeType,
}

#[derive(Debug, Clone)]
/// Type classification for call node.
pub enum CallNodeType {
    Function,
    Method,
    Struct,
    Module,
    Trait,
}

#[derive(Debug, Clone)]
/// Call edge.
pub struct CallEdge {
    pub from: String,
    pub to: String,
    pub edge_type: CallEdgeType,
    pub weight: u32,
}

#[derive(Debug, Clone)]
/// Type classification for call edge.
pub enum CallEdgeType {
    FunctionCall,
    MethodCall,
    StructInstantiation,
    TraitImpl,
    ModuleImport,
}

/// Query result with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub diagram: String,
    pub metadata: GraphMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Metadata for graph.
pub struct GraphMetadata {
    pub nodes: usize,
    pub edges: usize,
    pub max_depth: usize,
    pub timestamp: DateTime<Utc>,
    pub query_version: String,
    pub analysis_time_ms: u64,
}

/// System architecture analysis query
pub struct SystemArchitectureQuery;

/// Architectural component representation
#[derive(Debug, Clone)]
pub struct Component {
    pub id: String,
    pub label: String,
    pub nodes: Vec<String>,
    pub complexity: f64,
    pub loc: usize,
    pub functions: usize,
}

/// Component relationship edge
#[derive(Debug, Clone)]
pub struct ComponentEdge {
    pub from: String,
    pub to: String,
    pub edge_type: ComponentEdgeType,
    pub weight: u32,
}

#[derive(Debug)]
/// Type classification for component edge.
pub enum ComponentEdgeType {
    Import,
    Call,
    Inheritance,
    Composition,
}

/// Component metrics aggregated from individual functions/modules
#[derive(Debug, Clone)]
pub struct ComponentMetrics {
    pub total_complexity: f64,
    pub avg_complexity: f64,
    pub max_complexity: f64,
    pub total_loc: usize,
    pub function_count: usize,
}

// --- Core impl: component detection, relationship inference, metric aggregation ---
include!("canonical_query_impl.rs");

// --- Diagram generation: Mermaid rendering, utility functions, trait impls ---
include!("canonical_query_diagram.rs");

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // Basic type tests and trait tests
    include!("canonical_query_tests.rs");

    // Relationship and metric tests
    include!("canonical_query_tests_part2.rs");
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    include!("canonical_query_proptests.rs");
}
