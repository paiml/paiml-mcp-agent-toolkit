#![cfg_attr(coverage_nightly, coverage(off))]
//! Mermaid diagram generator for dependency graphs
//!
//! This module generates Mermaid-compatible diagrams from dependency graphs,
//! providing visual representations of code structure, dependencies, and complexity.
//! It implements intelligent graph simplification to handle large codebases while
//! maintaining diagram readability and avoiding Mermaid's rendering limits.
//!
//! # Features
//!
//! - **PageRank-based Selection**: Prioritizes important nodes using `PageRank` algorithm
//! - **Module Grouping**: Organizes nodes by module for better visual hierarchy
//! - **Complexity Visualization**: Optional complexity metrics in node labels
//! - **Edge Type Styling**: Different styles for imports, calls, and inheritance
//! - **Automatic Simplification**: Reduces graph size while preserving key relationships
//!
//! # Diagram Types
//!
//! - **Dependency Graphs**: Show module and file dependencies
//! - **Call Graphs**: Display function call relationships
//! - **Architecture Diagrams**: High-level system structure
//! - **Complexity Heatmaps**: Visualize complexity distribution
//!
//! # Example
//!
//! ```no_run
//! use pmat::services::mermaid_generator::{MermaidGenerator, MermaidOptions};
//! use pmat::models::dag::DependencyGraph;
//!
//! let options = MermaidOptions {
//!     max_depth: Some(3),
//!     filter_external: true,
//!     group_by_module: true,
//!     show_complexity: true,
//! };
//!
//! let generator = MermaidGenerator::new(options);
//! let graph = DependencyGraph::new(); // Your dependency graph
//!
//! let mermaid_code = generator.generate(&graph);
//! println!("```mermaid\n{}\n```", mermaid_code);
//!
//! // Use with fixed size configuration
//! use pmat::services::fixed_graph_builder::{GraphConfig, GroupingStrategy};
//! let config = GraphConfig {
//!     max_nodes: 30,
//!     max_edges: 200,
//!     grouping: GroupingStrategy::Module,
//! };
//! let diagram = generator.generate_with_config(&graph, &config);
//! ```

mod formatters;
mod generator;
mod node_helpers;
mod types;

pub use types::{MermaidGenerator, MermaidOptions};

#[cfg(test)]
mod tests;

#[cfg(test)]
mod tests_part2;

#[cfg(test)]
mod tests_part3;

#[cfg(test)]
mod property_tests;
