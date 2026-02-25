use anyhow::Result;
use std::collections::{HashMap, HashSet};
use tree_sitter::{Node, Tree};
use crate::tdg::{Language, MetricCategory, PenaltyTracker, TdgConfig};
use super::{Scorer, walk_tree, get_node_text};

pub struct CouplingAnalyzer;

struct DependencyGraph {
    edges: HashMap<String, HashSet<String>>,
}

// CouplingAnalyzer methods: afferent/efferent coupling, abstractness,
// visibility checks, import extraction, builtin type detection, dependency graph building
include!("coupling_analysis.rs");

// Scorer trait implementation: scoring logic with penalty tracking
include!("coupling_scorer.rs");

// DependencyGraph methods: topological sort, DFS traversal, depth calculation
include!("coupling_graph.rs");

// Unit tests for CouplingAnalyzer analysis methods and DependencyGraph operations
include!("coupling_tests.rs");

// Scorer trait tests, metric integration tests, and property-based tests
include!("coupling_tests_scorer.rs");
