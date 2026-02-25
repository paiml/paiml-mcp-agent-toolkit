#![cfg_attr(coverage_nightly, coverage(off))]
// Semantic Search Engine
// PMAT-SEARCH-004: High-level orchestration for code search
//
// GREEN Phase: Full implementation with LOCAL embeddings (zero API keys)
// Uses aprender TF-IDF for vectorization - no external dependencies

use super::chunker::{chunk_code, ChunkType, Language};
use super::turso_vector_db::{EmbeddingEntry, TursoVectorDB};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::time::Instant;
use walkdir::WalkDir;

/// Search mode
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchMode {
    SemanticOnly,
    KeywordOnly,
    Hybrid,
}

/// Search query
#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub query: String,
    pub mode: SearchMode,
    pub language_filter: Option<String>,
    pub file_pattern: Option<String>,
    pub chunk_type_filter: Option<ChunkType>,
    pub limit: usize,
}

/// Search result
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub file_path: String,
    pub chunk_name: String,
    pub chunk_type: String,
    pub language: String,
    pub similarity_score: f64,
    pub snippet: String,
    pub start_line: usize,
    pub end_line: usize,
}

/// Index statistics
#[derive(Debug, Clone)]
pub struct IndexStats {
    pub total_files: usize,
    pub total_chunks: usize,
    pub created: usize,
    pub updated: usize,
    pub skipped: usize,
    pub duration_ms: u64,
}

/// Hash-based local embedder (Send + Sync safe)
/// Uses feature hashing (hashing trick) for fixed-dimension embeddings
/// Provides pure Rust embeddings with no external API dependencies
#[derive(Debug)]
pub struct LocalEmbedder {
    /// Fixed embedding dimension
    dimension: usize,
    /// Term frequencies for IDF calculation
    document_frequencies: RwLock<HashMap<String, usize>>,
    /// Total document count
    doc_count: RwLock<usize>,
}

/// Semantic search engine using LOCAL embeddings (no API keys required)
pub struct SemanticSearchEngine {
    vector_db: Arc<TursoVectorDB>,
    embedder: Arc<RwLock<LocalEmbedder>>,
}

// --- Implementation split into include files ---

include!("search_engine_embedder.rs");
include!("search_engine_core.rs");
include!("search_engine_tests.rs");
