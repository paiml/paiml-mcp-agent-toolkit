#![cfg_attr(coverage_nightly, coverage(off))]
//! Query Engine for Agent Context
//!
//! Provides semantic search with quality filtering over the function index.

mod coverage;
pub(crate) mod coverage_exclusion;
mod engine;
mod enrichment;
pub(crate) mod extract_candidates;
mod formatters;
pub(crate) mod ptx_diagnostics;
pub(crate) mod ptx_flow;
pub(crate) mod raw_search;
pub(crate) mod suggest_rename;
mod types;

pub(crate) use coverage::discover_line_coverage;
pub use coverage::{
    build_coverage_map, compute_impact_score, enrich_results_with_coverage, enrich_with_coverage,
    enrich_with_coverage_diff, format_coverage_summary, load_workspace_coverage,
};
// coverage_exclusion types are used via crate::services::agent_context::query::coverage_exclusion
pub use enrichment::{
    build_churn_map, enrich_results_with_churn, enrich_results_with_duplicates,
    enrich_results_with_entropy, enrich_results_with_faults, enrich_with_churn,
};
pub use formatters::{format_json, format_markdown, format_text, format_text_with_code};
pub use suggest_rename::{suggest_renames, RenameSignal, RenameSuggestion};
pub use types::{
    normalize_definition_type, CaseSensitivity, QueryOptions, QueryResult, RankBy, SearchMode,
    DEFINITION_TYPE_VALUES,
};

// Engine methods are impl'd on AgentContextIndex directly (in engine.rs),
// so they're available wherever AgentContextIndex is imported.

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_part2;
