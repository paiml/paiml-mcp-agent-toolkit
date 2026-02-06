#![cfg_attr(coverage_nightly, coverage(off))]
//! Type definitions for the Function Index.
//!
//! Contains all struct and enum definitions used by the agent context index.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Quality metrics for a function
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QualityMetrics {
    /// TDG score (lower is better, 0-10 scale)
    pub tdg_score: f32,
    /// TDG grade (A, B, C, D, F)
    pub tdg_grade: String,
    /// Cyclomatic complexity
    pub complexity: u32,
    /// Cognitive complexity
    pub cognitive_complexity: u32,
    /// Big-O runtime estimate
    pub big_o: String,
    /// SATD marker count (TODO, FIXME, HACK)
    pub satd_count: u32,
    /// Lines of code
    pub loc: u32,
    /// Git commit count for the file (churn indicator)
    #[serde(default)]
    pub commit_count: u32,
    /// Churn score (0.0-1.0, higher = more volatile)
    #[serde(default)]
    pub churn_score: f32,
}

/// Definition type for indexed items (issue #150)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DefinitionType {
    #[default]
    Function,
    Struct,
    Enum,
    Trait,
    TypeAlias,
}

/// A function/type entry in the index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionEntry {
    /// File path relative to project root
    pub file_path: String,
    /// Function/type name
    pub function_name: String,
    /// Full signature/definition
    pub signature: String,
    /// Type of definition (function, struct, enum, trait, type alias)
    #[serde(default)]
    pub definition_type: DefinitionType,
    /// Documentation comment (if any)
    pub doc_comment: Option<String>,
    /// Full function source code
    pub source: String,
    /// Starting line number (1-indexed)
    pub start_line: usize,
    /// Ending line number
    pub end_line: usize,
    /// Programming language
    pub language: String,
    /// Quality metrics
    pub quality: QualityMetrics,
    /// Content checksum for incremental updates
    pub checksum: String,
    // === Cached annotations (computed at build time, not query time) ===
    /// Git commit count for this file
    #[serde(default)]
    pub commit_count: u32,
    /// Churn score 0.0-1.0 (higher = more volatile)
    #[serde(default)]
    pub churn_score: f32,
    /// Number of code clones/duplicates
    #[serde(default)]
    pub clone_count: u32,
    /// Pattern diversity 0.0-1.0 (lower = more repetitive)
    #[serde(default)]
    pub pattern_diversity: f32,
    /// Fault pattern annotations from batuta
    #[serde(default)]
    pub fault_annotations: Vec<String>,
}

/// Index manifest with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexManifest {
    /// Version of the index format
    pub version: String,
    /// Timestamp when index was built
    pub built_at: String,
    /// Project root path
    pub project_root: String,
    /// Number of functions indexed
    pub function_count: usize,
    /// Number of files processed
    pub file_count: usize,
    /// Languages detected
    pub languages: Vec<String>,
    /// Average TDG score
    pub avg_tdg_score: f32,
    /// SHA256 checksums for each source file (for incremental updates)
    #[serde(default)]
    pub file_checksums: HashMap<String, String>,
    /// Number of files reparsed in last incremental update (0 = no changes)
    #[serde(default)]
    pub last_incremental_changes: usize,
}

/// Serialized payload for the index (v1.3.0+ with cached indices)
#[derive(Serialize, Deserialize)]
pub(super) struct IndexPayload {
    pub(super) functions: Vec<FunctionEntry>,
    pub(super) corpus: Vec<String>,
    pub(super) calls: HashMap<usize, Vec<usize>>,
    pub(super) called_by: HashMap<usize, Vec<usize>>,
    // v1.3.0: Cached indices to avoid rebuild on load
    #[serde(default)]
    pub(super) name_index: HashMap<String, Vec<usize>>,
    #[serde(default)]
    pub(super) file_index: HashMap<String, Vec<usize>>,
    #[serde(default)]
    pub(super) graph_metrics: Vec<GraphMetrics>,
    #[serde(default)]
    pub(super) corpus_lower: Vec<String>,
    #[serde(default)]
    pub(super) name_frequency: HashMap<String, f32>,
}

/// Result of build_indices function
#[derive(Debug, Clone, Default)]
pub(crate) struct BuildIndicesResult {
    pub name_index: HashMap<String, Vec<usize>>,
    pub file_index: HashMap<String, Vec<usize>>,
    pub corpus: Vec<String>,
}

/// Index statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    /// Total functions indexed
    pub total_functions: usize,
    /// Functions by language
    pub by_language: HashMap<String, usize>,
    /// Functions by TDG grade
    pub by_grade: HashMap<String, usize>,
    /// Average complexity
    pub avg_complexity: f32,
    /// Index size in bytes
    pub index_size_bytes: u64,
}

/// Graph metrics for ranking functions by importance
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct GraphMetrics {
    /// PageRank score (higher = more important, transitively called)
    pub pagerank: f32,
    /// Degree centrality (direct callers + callees)
    pub centrality: f32,
    /// In-degree (number of direct callers)
    pub in_degree: u32,
    /// Out-degree (number of direct callees)
    pub out_degree: u32,
}

/// Agent Context Index - searchable function database
#[derive(Clone)]
pub struct AgentContextIndex {
    /// All indexed functions
    pub(crate) functions: Vec<FunctionEntry>,
    /// Function name -> indices for O(1) lookup
    pub(crate) name_index: HashMap<String, Vec<usize>>,
    /// File path -> indices
    pub(crate) file_index: HashMap<String, Vec<usize>>,
    /// Document corpus for BM25-style search
    pub(crate) corpus: Vec<String>,
    /// Pre-computed lowercase corpus (avoids per-query lowercasing of 42K+ docs)
    pub(crate) corpus_lower: Vec<String>,
    /// Name frequency for generic name demotion (name -> fraction of total functions)
    pub(crate) name_frequency: HashMap<String, f32>,
    /// Caller graph: func_idx -> indices of functions it calls
    pub(crate) calls: HashMap<usize, Vec<usize>>,
    /// Callee graph: func_idx -> indices of functions that call it
    pub(crate) called_by: HashMap<usize, Vec<usize>>,
    /// Graph metrics per function (PageRank, centrality)
    pub(crate) graph_metrics: Vec<GraphMetrics>,
    /// Project root
    pub(crate) project_root: PathBuf,
    /// Manifest
    pub(crate) manifest: IndexManifest,
}
