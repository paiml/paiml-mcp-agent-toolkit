#![cfg_attr(coverage_nightly, coverage(off))]
// Reciprocal Rank Fusion (GH-RAG-006)
// Toyota Way: Jidoka - Automation with quality built-in
// Citation: [SIGIR-2022] SPLADE v2 - Sparse Lexical and Expansion Model
// Spec: docs/specifications/git-history-rag-integration.md

use std::collections::HashMap;

/// Reciprocal Rank Fusion (RRF) implementation
/// Formula: score(d) = Σ 1/(k + rank(d))
/// Citation: [SIGIR-2022] Formal, T., et al. "SPLADE v2"
pub struct RrfFusion {
    /// Constant k in RRF formula (default: 60)
    /// Higher k values smooth out the impact of rank differences
    k: f32,
}

/// A document with a ranking score
#[derive(Debug, Clone)]
pub struct RankedDocument {
    /// Unique identifier (file_path:function_name or commit_hash)
    pub id: String,
    /// Original relevance score (for display)
    pub original_score: f32,
    /// Source of this result ("code" or "git")
    pub source: String,
    /// Additional metadata
    pub metadata: DocumentMetadata,
}

/// Metadata carried through fusion
#[derive(Debug, Clone, Default)]
pub struct DocumentMetadata {
    /// File path (for code) or commit hash (for git)
    pub path: String,
    /// Function name (for code) or commit subject (for git)
    pub name: String,
    /// Line number (for code) or timestamp (for git as i64)
    pub line_or_timestamp: i64,
    /// Related commit hashes (for code results enriched with git)
    pub related_commits: Vec<String>,
}

/// Result of RRF fusion
#[derive(Debug, Clone)]
pub struct FusedResult {
    /// Document identifier
    pub id: String,
    /// Combined RRF score
    pub rrf_score: f32,
    /// Original scores from each source
    pub source_scores: HashMap<String, f32>,
    /// Metadata from the highest-scoring source
    pub metadata: DocumentMetadata,
    /// Source that contributed most
    pub primary_source: String,
}

impl Default for RrfFusion {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for accumulating RRF scores
struct FusedResultBuilder {
    id: String,
    total_rrf: f32,
    source_scores: HashMap<String, f32>,
    best_metadata: DocumentMetadata,
    best_score: f32,
    primary_source: String,
}

// --- Submodule includes ---
// Scoring methods: new(), with_k(), fuse(), calculate_improvement(), mean_reciprocal_rank()
include!("rrf_fusion_scoring.rs");
// Unit tests
include!("rrf_fusion_tests.rs");
