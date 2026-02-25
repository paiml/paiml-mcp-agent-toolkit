#![cfg_attr(coverage_nightly, coverage(off))]
// Hybrid Search Engine with RRF
// PMAT-SEARCH-005: Combine keyword (ripgrep) and vector search
//
// GREEN Phase: Full implementation
//
// This module is split into include files:
// - hybrid_search_engine.rs: HybridSearchEngine impl methods
// - hybrid_search_bm25.rs: BM25 search engine (trueno-rag integration)
// - hybrid_search_tests.rs: Unit tests for BM25 and RRF

use super::search_engine::{SearchQuery, SearchResult, SemanticSearchEngine};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use trueno_rag::index::BM25Index;
use trueno_rag::{Chunk, ChunkId, DocumentId, SparseIndex};

/// Search mode for hybrid engine
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HybridSearchMode {
    KeywordOnly,
    VectorOnly,
    Hybrid,
}

/// Hybrid search query
#[derive(Debug, Clone)]
pub struct HybridSearchQuery {
    pub query: String,
    pub mode: HybridSearchMode,
    pub keyword_weight: f64,
    pub vector_weight: f64,
    pub language_filter: Option<String>,
    pub file_pattern: Option<String>,
    pub limit: usize,
}

/// Hybrid search result
#[derive(Debug, Clone)]
pub struct HybridSearchResult {
    pub file_path: String,
    pub chunk_name: String,
    pub chunk_type: String,
    pub language: String,
    pub start_line: usize,
    pub end_line: usize,
    pub keyword_score: f64,
    pub vector_score: f64,
    pub hybrid_score: f64,
    pub snippet: String,
}

/// Keyword match from ripgrep
#[derive(Debug, Clone)]
pub struct KeywordMatch {
    file_path: String,
    line_number: usize,
    content: String,
}

/// Hybrid search engine combining keyword and vector search
pub struct HybridSearchEngine {
    semantic_engine: Arc<SemanticSearchEngine>,
    search_root: std::path::PathBuf,
}

// HybridSearchEngine implementation: constructor, search dispatch,
// keyword/vector/hybrid search, RRF merging, filtering, utilities
include!("hybrid_search_engine.rs");

// BM25 search engine using trueno-rag: struct, impl, Default
include!("hybrid_search_bm25.rs");

// Unit tests for BM25 and RRF
include!("hybrid_search_tests.rs");
