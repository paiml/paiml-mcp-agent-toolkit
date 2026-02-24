#![cfg_attr(coverage_nightly, coverage(off))]
//! Local Semantic Analysis Service
//!
//! Pure Rust semantic search, topic modeling, and clustering.
//! **Zero external API dependencies** - no OpenAI, no internet required.
//!
//! # Architecture (Toyota Way - Jidoka)
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                  Semantic Search Stack                       │
//! │                  (Pure Rust - Zero API Keys)                 │
//! ├─────────────────────────────────────────────────────────────┤
//! │  aprender 0.14.0     │ TF-IDF, LDA, K-means, DBSCAN         │
//! │  trueno-rag          │ Hybrid retrieval, RRF fusion         │
//! │  trueno-graph        │ PageRank, BFS, Louvain clustering    │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Peer-Reviewed Foundation
//!
//! | Algorithm | Citation |
//! |-----------|----------|
//! | TF-IDF | Manning et al. (2008) "Introduction to IR" |
//! | LDA | Blei et al. (2003) JMLR |
//! | K-means | MacQueen (1967) Berkeley Symposium |
//! | DBSCAN | Ester et al. (1996) KDD |
//! | BM25 | Robertson & Zaragoza (2009) F&T in IR |
//! | RRF | Cormack et al. (2009) SIGIR |
//! | PageRank | Page et al. (1999) Stanford |
//!
//! # Usage
//!
//! ```rust,no_run
//! use pmat::services::local_semantic::LocalSemanticEngine;
//!
//! let mut engine = LocalSemanticEngine::new();
//!
//! // Index codebase
//! engine.index_directory(std::path::Path::new("."), None)?;
//!
//! // Extract topics (LDA)
//! let topics = engine.extract_topics(5, None)?;
//!
//! // Cluster code (K-means)
//! let clusters = engine.cluster("kmeans", Some(5))?;
//! # Ok::<(), String>(())
//! ```
//!
//! # Specification
//!
//! See: `docs/specifications/semantic-search-feature.md`

use aprender::cluster::{AgglomerativeClustering, KMeans, DBSCAN};
use aprender::primitives::Matrix;
use aprender::text::tokenize::WhitespaceTokenizer;
use aprender::text::topic::LatentDirichletAllocation;
use aprender::text::vectorize::TfidfVectorizer;
use aprender::traits::UnsupervisedEstimator;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Local semantic analysis engine using aprender
pub struct LocalSemanticEngine {
    /// Collected code documents
    documents: Vec<CodeDocument>,
    /// Document-term matrix (f64 for LDA)
    dtm: Option<Matrix<f64>>,
    /// Vocabulary mapping (word -> index)
    vocabulary: HashMap<String, usize>,
    /// Reverse vocabulary (index -> word)
    reverse_vocabulary: Vec<String>,
}

/// A code document for analysis
#[derive(Debug, Clone)]
pub struct CodeDocument {
    pub file_path: PathBuf,
    pub content: String,
    pub language: String,
}

/// Result of topic extraction
#[derive(Debug, Clone)]
pub struct LocalTopicResult {
    pub topics: Vec<LocalTopic>,
    pub num_documents: usize,
}

/// A single topic with top terms
#[derive(Debug, Clone)]
pub struct LocalTopic {
    pub id: usize,
    pub top_terms: Vec<(String, f64)>,
    pub document_count: usize,
}

/// Result of clustering
#[derive(Debug, Clone)]
pub struct LocalClusterResult {
    pub clusters: Vec<LocalCluster>,
    pub method: String,
    pub num_documents: usize,
}

/// A single cluster
#[derive(Debug, Clone)]
pub struct LocalCluster {
    pub id: usize,
    pub files: Vec<PathBuf>,
    pub size: usize,
}

// --- Implementation (split into include files) ---

include!("local_semantic_engine.rs");
include!("local_semantic_tests.rs");
