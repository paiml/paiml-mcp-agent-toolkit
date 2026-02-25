//! Query Handler - Semantic code search for agents (PMAT-470)
//!
//! Provides RAG-powered code search with quality annotations.
//! Designed as a grep replacement for AI agents.

#![cfg_attr(coverage_nightly, coverage(off))]

mod enrichments;
mod formatting;
mod git_history;
mod indexing;
mod modes;
mod options;

use crate::cli::QueryOutputFormat;
use std::path::PathBuf;

use enrichments::{apply_all_enrichments, fetch_git_data, merge_raw_results};
use formatting::emit_query_output;
use indexing::{
    backfill_results_source, collect_siblings, emit_index_stats, load_query_index,
    prepare_index_for_mode,
};
use modes::{
    emit_docs_section, handle_coverage_gaps_mode, handle_docs_search,
    handle_extract_candidates_mode, handle_ptx_modes, handle_raw_search_mode,
    handle_suggest_rename_mode,
};
use options::{
    apply_post_enrichment_sort, apply_result_filters, build_query_options, MergeContext,
    QueryProfile,
};

include!("query_execution.rs");

#[cfg(test)]
mod tests {
    use super::*;
    use super::git_history::{
        classify_commit_type, compute_decay_score, compute_impact_risk, format_timestamp,
        parse_git_log,
    };
    use super::options::{FileAnnotation, FileHotspot};
    use tempfile::TempDir;

    include!("query_tests.rs");
}
