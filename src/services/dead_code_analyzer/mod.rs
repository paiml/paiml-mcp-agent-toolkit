#![cfg_attr(coverage_nightly, coverage(off))]
//! Dead code detection with cross-reference analysis
//!
//! This module provides advanced dead code detection capabilities through multi-level
//! reachability analysis, cross-language reference tracking, and dynamic dispatch resolution.
//! It identifies unreachable code segments, unused functions, and unreferenced variables
//! across multiple programming languages.
//!
//! # Architecture
//!
//! The dead code analyzer consists of several key components:
//! - **`HierarchicalBitSet`**: Efficient bitmap-based reachability tracking using Roaring bitmaps
//! - **`CrossLangReferenceGraph`**: Graph structure for tracking references across languages
//! - **`VTableResolver`**: Dynamic dispatch resolution for object-oriented languages
//! - **`DeadCodeAnalyzer`**: Main analysis engine that orchestrates the detection process
//!
//! # Analysis Process
//!
//! 1. **Build Reference Graph**: Constructs a cross-language reference graph from AST nodes
//! 2. **Mark Entry Points**: Identifies entry points (main functions, exported symbols, etc.)
//! 3. **Reachability Analysis**: Performs breadth-first traversal to mark reachable code
//! 4. **Dynamic Resolution**: Resolves virtual calls and dynamic dispatch targets
//! 5. **Dead Code Identification**: Reports unreachable code segments
//!
//! # Features
//!
//! - Cross-language reference tracking (Rust, Python, JavaScript, etc.)
//! - Dynamic dispatch resolution for OOP languages
//! - Hierarchical bitmap for efficient memory usage
//! - Confidence scoring for uncertain references
//! - Integration with dependency graphs and AST analysis
//!
//! # Example Usage
//!
//! ```rust
//! use pmat::services::dead_code_analyzer::DeadCodeAnalyzer;
//! use pmat::models::unified_ast::AstDag;
//!
//! // Create analyzer with node capacity
//! let mut analyzer = DeadCodeAnalyzer::new(1000);
//!
//! // Create AST DAG for analysis
//! let ast_dag = AstDag::new();
//!
//! // Perform dead code analysis
//! let report = analyzer.analyze(&ast_dag);
//!
//! if report.dead_functions.is_empty() {
//!     println!("No dead code found!");
//! } else {
//!     println!("Found {} dead functions", report.dead_functions.len());
//!     for func in &report.dead_functions {
//!         println!("  - {} at {}:{}", func.name, func.file_path, func.line_number);
//!     }
//! }
//! ```

pub mod types;
pub mod cfg_detection;
pub mod reference_graph;
pub mod analysis;

// Re-export all public items from submodules
pub use types::{
    CoverageData, CrossLangReferenceGraph, DeadCodeItem, DeadCodeReport, DeadCodeSummary,
    DeadCodeType, HierarchicalBitSet, ReferenceEdge, ReferenceNode, ReferenceType,
    UnreachableBlock, VTableResolver,
};

pub use analysis::DeadCodeAnalyzer;
#[cfg(test)]
pub(crate) use analysis::{
    calculate_dead_percentage,
    classify_dead_functions_pure,
    collect_functions_from_context,
    compute_reachability,
    detect_function_calls_in_lines,
};
