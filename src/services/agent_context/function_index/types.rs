#![cfg_attr(coverage_nightly, coverage(off))]
//! Type definitions for the Function Index.
//!
//! Contains all struct and enum definitions used by the agent context index.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Identifier of the TDG scale persisted `tdg_score` values were written under.
///
/// Stored in `IndexManifest::tdg_scale` (blob) and in the SQLite `metadata`
/// table under the `tdg_scale` key. An index whose marker is absent or
/// different was written under a DIFFERENT scale — before v3.30.0 the index
/// stored a 0-10 LOWER-is-better debt number — and must be REBUILT, never
/// reinterpreted: reading a stored `0.12` as 0.12/100 would turn the best
/// possible legacy score into an F. Absent is not "fine", it is unknown.
pub const TDG_SCALE: &str = "tdg-0-100-higher-is-better";

/// Quality metrics for a function
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QualityMetrics {
    /// TDG score on the 0-100 HIGHER-is-better scale `pmat tdg` reports.
    ///
    /// Was a 0-10 lower-is-better debt number before v3.30.0; see [`TDG_SCALE`].
    pub tdg_score: f32,
    /// TDG grade, one of `crate::tdg::Grade` (`A+`, `A`, `A-`, … `D`, `F`)
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
    /// Contract verification level (L0-L5, None if no contract)
    #[serde(default)]
    pub contract_level: Option<String>,
    /// Contract equation name (from #[contract] attribute)
    #[serde(default)]
    pub contract_equation: Option<String>,
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
    /// C/C++ forward declaration (header file prototype)
    Declaration,
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
    /// Linked definition file:line for declarations (e.g., header → implementation)
    #[serde(default)]
    pub linked_definition: Option<String>,
}

/// What the index recorded about a source file the last time it read it.
///
/// The checksum alone cannot decide whether a file changed without reading it:
/// the mtime fast path skips the read entirely, and mtime is writable by any
/// process (`touch -d`, `os.utime`, a tarball restore), so a rewrite can carry
/// an mtime that predates the index and be silently served from a stale
/// checksum. `len` and (on unix) `ctime` are the two stat fields a rewrite
/// cannot hold still: truncating and writing advances ctime, and content of a
/// different size changes len. Both are compared before the read is skipped.
///
/// `len == 0 && ctime == 0` means UNKNOWN, not "an empty file created at the
/// epoch" — that is what every manifest written before this record existed
/// deserialises to (see the `Deserialize` impl below), and unknown stats must
/// force a re-hash rather than authorise a skip.
#[derive(Debug, Clone, Serialize, PartialEq, Eq, Default)]
pub struct FileRecord {
    /// SHA256 of the file contents as of the last read
    pub checksum: String,
    /// File length in bytes as of the last read (0 = unknown)
    pub len: u64,
    /// Unix ctime (inode change time), NANOSECONDS since the epoch, as of the
    /// last read (0 = unknown). See `file_stat_fields` for why not seconds.
    pub ctime: i64,
}

impl FileRecord {
    /// A record carrying only a checksum — stats unknown, so no skip is allowed.
    pub fn from_checksum(checksum: String) -> Self {
        Self {
            checksum,
            len: 0,
            ctime: 0,
        }
    }

    /// True when this record carries stat evidence a skip decision may rest on.
    pub fn has_stats(&self) -> bool {
        self.len != 0 || self.ctime != 0
    }
}

impl<'de> Deserialize<'de> for FileRecord {
    /// Accepts both the current object form and the bare-checksum string every
    /// pre-CRUX-07 manifest holds. A legacy entry loads with unknown stats, so
    /// the file is re-hashed once and rewritten with stats on the next save.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Repr {
            Legacy(String),
            Full {
                checksum: String,
                #[serde(default)]
                len: u64,
                #[serde(default)]
                ctime: i64,
            },
        }
        Ok(match Repr::deserialize(deserializer)? {
            Repr::Legacy(checksum) => Self::from_checksum(checksum),
            Repr::Full {
                checksum,
                len,
                ctime,
            } => Self {
                checksum,
                len,
                ctime,
            },
        })
    }
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
    /// Average TDG score (0-100, higher is better)
    pub avg_tdg_score: f32,
    /// Scale the persisted `tdg_score` values were written under.
    ///
    /// Empty (the serde default, which is what every pre-v3.30.0 index
    /// deserialises to) means UNKNOWN, and unknown must fail loudly rather
    /// than be read on today's scale. See [`TDG_SCALE`].
    #[serde(default)]
    pub tdg_scale: String,
    /// Per-file record (checksum + stat evidence) for incremental updates.
    ///
    /// Values written before CRUX-07 were bare checksum strings; they still
    /// load, with unknown stats. See [`FileRecord`].
    #[serde(default)]
    pub file_checksums: HashMap<String, FileRecord>,
    /// Number of files reparsed in last incremental update (0 = no changes)
    #[serde(default)]
    pub last_incremental_changes: usize,
}

/// Serialized payload for the index (v1.4.0 with cached indices)
///
/// v1.3.0: Added cached indices (name_index, file_index, graph_metrics, corpus_lower, name_frequency)
/// v1.4.0: corpus_lower no longer persisted (computed lazily on load, saves ~50MB)
#[derive(Serialize, Deserialize)]
pub(super) struct IndexPayload {
    pub(super) functions: Vec<FunctionEntry>,
    pub(super) corpus: Vec<String>,
    pub(super) calls: HashMap<usize, Vec<usize>>,
    pub(super) called_by: HashMap<usize, Vec<usize>>,
    // v1.3.0+: Cached indices to avoid rebuild on load
    #[serde(default)]
    pub(super) name_index: HashMap<String, Vec<usize>>,
    #[serde(default)]
    pub(super) file_index: HashMap<String, Vec<usize>>,
    #[serde(default)]
    pub(super) graph_metrics: Vec<GraphMetrics>,
    /// v1.4.0: Empty on save, computed lazily on load from `corpus`
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
    /// Path to SQLite FTS5 database (if available)
    pub(crate) db_path: Option<PathBuf>,
    /// Files with module-level `coverage(off)` annotation (cached from index build)
    pub(crate) coverage_off_files: HashSet<String>,
}
