use anyhow::Result;
use std::collections::HashMap;
use tree_sitter::{Node, Tree};
use crate::tdg::{Language, MetricCategory, PenaltyTracker, TdgConfig};
use super::{Scorer, walk_tree, get_node_text};

/// Duplication detector.
pub struct DuplicationDetector {
    min_token_sequence: usize,
    similarity_threshold: f32,
}

impl DuplicationDetector {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self {
            min_token_sequence: 50,
            similarity_threshold: 0.85,
        }
    }
}

#[derive(Clone, Debug)]
struct Token {
    kind: String,
    text: String,
    normalized: String,
}

#[derive(Clone, Debug)]
struct TokenSequence {
    tokens: Vec<Token>,
    start_byte: usize,
    end_byte: usize,
}

#[derive(Debug)]
enum CloneType {
    Exact,
    Renamed,
    Modified,
}

#[derive(Debug)]
struct CloneSet {
    clones: Vec<(CloneType, Vec<TokenSequence>)>,
}

// Token extraction, clone detection, and similarity analysis
include!("duplication_analysis.rs");

// Scorer trait implementation and CloneSet methods
include!("duplication_scoring.rs");

// Unit tests — data structures, similarity, LCS
include!("duplication_tests.rs");

// Unit tests — normalization, hashing, scoring, clone detection
include!("duplication_tests_scoring.rs");

// Property-based tests
include!("duplication_property_tests.rs");
