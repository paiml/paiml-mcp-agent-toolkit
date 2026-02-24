//! Search modes: raw, coverage-gaps, extract-candidates, suggest-rename, PTX, docs.

use super::options::*;
use crate::cli::QueryOutputFormat;
use crate::services::agent_context::{
    build_coverage_map, enrich_results_with_coverage, enrich_with_coverage_diff,
    format_coverage_summary, format_json, format_markdown, is_within_indexed_function, raw_search,
    suggest_renames, AgentContextIndex, QueryResult, RawSearchOptions, RawSearchOutput,
    RawSearchResult, RenameSignal, RenameSuggestion,
};
use std::path::PathBuf;

// ── Raw search mode ─────────────────────────────────────────────────────────
include!("modes_raw_search.rs");

// ── Coverage-gaps mode + PTX modes ──────────────────────────────────────────
include!("modes_coverage_gaps.rs");

// ── Suggest-rename mode ─────────────────────────────────────────────────────
include!("modes_rename.rs");

// ── Extract candidates mode ─────────────────────────────────────────────────
include!("modes_extract.rs");

// ── Document search helpers ─────────────────────────────────────────────────
include!("modes_docs.rs");
