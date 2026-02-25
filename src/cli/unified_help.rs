#![cfg_attr(coverage_nightly, coverage(off))]
//! Unified Help Service - RAG-powered intelligent help
//!
//! Combines:
//! - **NLP**: Tokenization, stemming, BM25 scoring
//! - **trueno-graph**: PageRank for command importance ranking
//!
//! # Architecture
//!
//! ```text
//! User Query
//!     |
//!     +-->  NLP (tokenize, stem, BM25)
//!     |        |
//!     |        v
//!     +-->  trueno-graph PageRank (importance ranking)
//!              |
//!              v
//!         Ranked Results with Context
//! ```
//!
//! # References
//!
//! - Specification: docs/specifications/unified-cli-mcp-help-integration.md
//! - GitHub Issue: #118
//! - Citations: Lewis et al. (2020) RAG, Teyton et al. (2013) PageRank

use crate::cli::registry::{CommandMetadata, CommandRegistry};
use std::collections::{HashMap, HashSet};

// Re-export for convenience
pub use trueno_graph::storage::csr::{CsrGraph, NodeId};

/// NLP processor for semantic help matching.
/// Uses simple but effective tokenization, stemming, and BM25 scoring.
pub struct HelpNlpProcessor {
    /// Stop words to filter out
    stop_words: HashSet<String>,
}

/// Graph-based command importance ranking using trueno-graph.
pub struct CommandGraph {
    /// CSR graph for efficient traversal
    graph: CsrGraph,
    /// Command name to node ID mapping
    command_to_node: HashMap<String, NodeId>,
    /// Node ID to command name mapping
    node_to_command: HashMap<NodeId, String>,
    /// Cached PageRank scores
    importance_scores: HashMap<String, f32>,
    /// Next available node ID
    next_node_id: u32,
}

/// Search result from unified help
#[derive(Debug, Clone)]
pub struct HelpSearchResult {
    /// Command name
    pub command: String,
    /// Short description
    pub description: String,
    /// Relevance score (0-1)
    pub relevance: f32,
    /// Importance score from PageRank
    pub importance: f32,
    /// Combined score
    pub combined_score: f32,
    /// Matched snippet
    pub snippet: String,
}

/// Response from unified help lookup
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum HelpResponse {
    /// Exact match found
    Exact(CommandMetadata),
    /// Fuzzy match suggestion
    DidYouMean { suggestion: String, confidence: f32 },
    /// Search results
    SearchResults {
        query: String,
        results: Vec<HelpSearchResult>,
    },
}

/// Unified help service combining NLP, Graph, and RAG.
pub struct UnifiedHelpService {
    registry: CommandRegistry,
    nlp: HelpNlpProcessor,
    graph: CommandGraph,
    /// Indexed command documents for search
    command_docs: HashMap<String, String>,
}

// HelpNlpProcessor: tokenization, stemming, BM25 scoring
include!("unified_help_nlp.rs");

// CommandGraph: graph construction and PageRank importance ranking
include!("unified_help_graph.rs");

// UnifiedHelpService: lookup, search, fuzzy matching, and levenshtein distance
include!("unified_help_service.rs");

// Tests: NLP, graph, service, and levenshtein tests
include!("unified_help_tests.rs");
