#![cfg_attr(coverage_nightly, coverage(off))]
//! Dependency graph builder for constructing code structure DAGs.
//!
//! This module builds directed acyclic graphs (DAGs) representing the structure
//! and dependencies within a codebase. It processes AST items to create nodes
//! and edges that capture relationships like function calls, type inheritance,
//! module imports, and data flow.
//!
//! # Graph Construction Process
//!
//! 1. **Node Collection**: First pass collects all entities (functions, types, modules)
//! 2. **Relationship Analysis**: Second pass creates edges based on code relationships
//! 3. **Graph Optimization**: Prunes edges to stay within visualization limits
//! 4. **Semantic Naming**: Applies deterministic naming for stable graph generation
//!
//! # Edge Types
//!
//! - **Calls**: Function/method invocations
//! - **Imports**: Module dependencies
//! - **Inherits**: Class/trait inheritance
//! - **Implements**: Interface implementations
//! - **Uses**: Type usage relationships
//! - **`DataFlow`**: Data dependencies between components
//!
//! # Example
//!
//! ```ignore
//! use pmat::services::dag_builder::DagBuilder;
//! use pmat::services::context::ProjectContext;
//!
//! # fn example(project: &ProjectContext) {
//! // Build dependency graph from project context
//! let graph = DagBuilder::build_from_project(project);
//!
//! println!("Graph has {} nodes and {} edges",
//!          graph.nodes.len(), graph.edges.len());
//!
//! // Find all functions that call a specific function
//! let callers = graph.edges.iter()
//!     .filter(|e| e.to == "myFunction" && e.edge_type == EdgeType::Calls)
//!     .map(|e| &e.from)
//!     .collect::<Vec<_>>();
//!
//! println!("Functions calling myFunction: {:?}", callers);
//! # }
//! ```ignore

use crate::models::dag::{DependencyGraph, Edge, EdgeType, NodeInfo, NodeType};
use crate::services::context::{AstItem, FileContext, ProjectContext};
use crate::services::semantic_naming::SemanticNamer;
use rustc_hash::{FxHashMap, FxHashSet};

/// Builder for constructing dag instances.
pub struct DagBuilder {
    graph: DependencyGraph,
    // Track functions by name for call resolution
    function_map: FxHashMap<String, String>, // function_name -> full_id
    // Track types for inheritance resolution
    type_map: FxHashMap<String, String>, // type_name -> full_id
    // Semantic namer for deterministic display names
    namer: SemanticNamer,
}

// Core: constructor, build_from_project, finalize_graph, Default impl
include!("dag_builder_core.rs");

// Node collection: collect_nodes from FileContext AST items
include!("dag_builder_node_collection.rs");

// Relationship processing: edges, imports, inheritance, helper methods
include!("dag_builder_relationships.rs");

// Graph algorithms: filtering, PageRank scoring, pruning
include!("dag_builder_graph_algorithms.rs");

// Tests
include!("dag_builder_tests.rs");
