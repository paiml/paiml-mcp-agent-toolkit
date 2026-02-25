#![cfg_attr(coverage_nightly, coverage(off))]

use super::helpers::*;
use super::types::*;
use crate::services::semantic::{chunk_code, Language};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

// ── Part 1: Helper function unit tests ─────────────────────────────────────
// detect_language, complexity, satd, big_o, grade, ignored_dir, sha256,
// name_frequency, doc_comments, keywords, identifiers, quality metrics,
// return type, count_params, TDG/grade boundaries, normalize_source
include!("tests_helpers.rs");

// ── Part 2: Index building, call graph, graph metrics ──────────────────────
// build_indices, build_call_graph, get_calls_and_called_by,
// find_function_index, compute_graph_metrics
include!("tests_indexing.rs");

// ── Part 3: Persistence (save/load, SQLite, incremental, workspace) ────────
// save_load_roundtrip, load_prefers_sqlite, load_fails, incremental_build,
// workspace_siblings, checksums, corpus_lower_lazy
include!("tests_persistence.rs");

// ── Part 4: Build scenarios, code clones, patterns, fault detection ────────
// merge_fast, manifest, build_with_multiple_files, empty/binary/ignored,
// detect_code_clones, pattern_diversity, detect_fault_patterns,
// name_index_capped, build_filters_test_functions
include!("tests_build_and_clones.rs");

// ── Part 5: Index accessors, cross-project call graph, PTX ─────────────────
// make_test_index, stats, get_by_name/file, corpus/project_root accessors,
// is_generic_callee, is_test_chunk, call_graph_excludes_generic,
// cross-project graph, PTX role classification, PTX diagnostics
include!("tests_cross_project_ptx.rs");
