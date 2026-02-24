#![cfg_attr(coverage_nightly, coverage(off))]
//! File Split Service — Semantic file splitting using Louvain community detection
//!
//! Analyzes a file's function entries in the agent context index, builds an intra-file
//! call graph, runs Louvain community detection, and names each cluster using
//! suggest-rename signal functions.
//!
//! # Algorithm
//!
//! 1. `index.file_index[path]` → get all function indices in file
//! 2. Build intra-file call graph from `calls`/`called_by` (keep only edges where
//!    both endpoints are in the same file)
//! 3. Convert to `UndirectedGraph` for Louvain
//! 4. Run `LouvainDetector::detect_communities()`
//! 5. Map communities → function entries → clusters with estimated line counts
//! 6. Name each cluster using suggest-rename signals
//! 7. Calculate impact (scan index for importers)

use crate::graph::community::LouvainDetector;
use crate::graph::types::{NodeData, UndirectedGraph};
use crate::services::agent_context::query::suggest_rename::{
    find_context_word, try_common_prefix, try_doc_comment_consensus, try_dominant_type,
    try_function_theme,
};
use crate::services::agent_context::{AgentContextIndex, FunctionEntry};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

// ── Sub-modules via include! ───────────────────────────────────────────────

include!("file_split_types.rs");
include!("file_split_algorithm.rs");
include!("file_split_graph.rs");
include!("file_split_naming.rs");
include!("file_split_analysis.rs");
include!("file_split_tests.rs");
