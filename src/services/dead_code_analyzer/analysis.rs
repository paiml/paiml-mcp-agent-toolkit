//! Core analysis logic for dead code detection.
//!
//! Contains the main `DeadCodeAnalyzer` struct, classification functions,
//! and pure utility functions for reachability and dead code identification.

use super::cfg_detection::is_cfg_gated;
use super::types::{
    CoverageData, CrossLangReferenceGraph, DeadCodeItem, DeadCodeReport, DeadCodeSummary,
    DeadCodeType, HierarchicalBitSet, VTableResolver,
};
use crate::models::dag::DependencyGraph;
use crate::models::unified_ast::{AstDag, NodeKey};
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Arc;

/// Main dead code analyzer
pub struct DeadCodeAnalyzer {
    // Multi-level reachability
    pub(crate) reachability: Arc<RwLock<HierarchicalBitSet>>,

    // Cross-language reference tracking
    pub(crate) references: Arc<RwLock<CrossLangReferenceGraph>>,

    // Dynamic dispatch resolution
    #[allow(dead_code)]
    pub(crate) vtable_analysis: Arc<RwLock<VTableResolver>>,

    // Test coverage integration
    coverage_map: Option<Arc<CoverageData>>,

    // Entry points (main functions, exported APIs, etc.)
    pub(crate) entry_points: Arc<RwLock<HashSet<NodeKey>>>,
}

// Constructor, with_coverage, analyze, analyze_dependency_graph
include!("analysis_core.rs");

// classify_dead_code, classify_dead_code_from_dep_graph
include!("analysis_classification.rs");

// analyze_project_context
include!("analysis_project_context.rs");

// analyze_with_ranking, aggregate_by_file
include!("analysis_ranking.rs");

// Pure functions: compute_reachability, find_containing_function,
// find_calls_in_line, detect_function_calls_in_lines,
// classify_dead_functions_pure, collect_functions_from_context,
// calculate_dead_percentage
include!("analysis_pure_functions.rs");

// Unit tests, pure function tests, property tests
include!("analysis_tests.rs");
