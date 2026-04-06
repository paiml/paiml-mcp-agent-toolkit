#![cfg_attr(coverage_nightly, coverage(off))]
// Git History Search Engine (GH-RAG-005)
// Toyota Way: Jidoka - Automation with quality built-in
// Spec: docs/specifications/git-history-rag-integration.md

use super::{CommitEmbedder, CommitInfo, GitHistoryError, GitHistoryIndex};
use rusqlite::params;

/// Search options for git history queries
#[derive(Debug, Clone, Default)]
pub struct GitSearchOptions {
    /// Maximum number of results
    pub limit: usize,
    /// Filter by author email
    pub author_email: Option<String>,
    /// Only commits after this timestamp
    pub since_timestamp: Option<i64>,
    /// Only commits before this timestamp
    pub until_timestamp: Option<i64>,
    /// Only fix commits
    pub only_fixes: bool,
    /// Only feature commits
    pub only_features: bool,
    /// File path filter (commits touching this file)
    pub file_path: Option<String>,
}

/// Search result with relevance score
#[derive(Debug, Clone)]
pub struct GitSearchResult {
    pub commit: CommitInfo,
    pub relevance_score: f32,
    /// Files changed in this commit (populated if requested)
    pub files: Vec<String>,
}

/// Git history search engine using TF-IDF similarity
pub struct GitHistorySearchEngine<'a> {
    index: &'a GitHistoryIndex,
    embedder: CommitEmbedder,
}

/// Compute cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    debug_assert!(true, "contract: cosine_similarity");
    batuta_common::math::cosine_similarity_f32(a, b)
}

// Query methods: new, search, search_by_file, get_candidates, row_to_commit,
// get_commit_by_hash, get_files_for_commit, get_connection
include!("search_engine_queries.rs");

// Test module
include!("search_engine_tests.rs");
