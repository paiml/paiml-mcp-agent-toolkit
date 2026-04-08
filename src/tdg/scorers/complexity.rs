//! Complexity scoring for TDG analysis.
//!
//! Split into submodules via include!():
//! - complexity_structural.rs: StructuralComplexityScorer impl + Scorer trait
//! - complexity_semantic.rs: SemanticComplexityScorer impl + Scorer trait
//! - complexity_tests.rs: Unit tests

use anyhow::Result;
use tree_sitter::{Node, Tree};
use crate::tdg::{Language, MetricCategory, PenaltyTracker, TdgConfig};
use super::{Scorer, walk_tree, count_nodes_of_kind, max_depth};

// ---------------------------------------------------------------------------
// Struct definitions
// ---------------------------------------------------------------------------

/// Structural complexity scorer.
pub struct StructuralComplexityScorer;

/// Semantic complexity scorer.
pub struct SemanticComplexityScorer;

// ---------------------------------------------------------------------------
// Implementations (via include!())
// ---------------------------------------------------------------------------

include!("complexity_structural.rs");
include!("complexity_semantic.rs");

// ---------------------------------------------------------------------------
// Tests (via include!())
// ---------------------------------------------------------------------------

include!("complexity_tests.rs");
