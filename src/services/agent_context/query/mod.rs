#![cfg_attr(coverage_nightly, coverage(off))]
//! Query Engine for Agent Context
//!
//! Provides semantic search with quality filtering over the function index.

mod coverage;
mod engine;
mod enrichment;
mod formatters;
pub(crate) mod raw_search;
mod types;

pub use coverage::{
    build_coverage_map, compute_impact_score, enrich_results_with_coverage,
    enrich_with_coverage, enrich_with_coverage_diff, format_coverage_summary,
};
pub use enrichment::{
    build_churn_map, enrich_results_with_churn, enrich_results_with_duplicates,
    enrich_results_with_entropy, enrich_results_with_faults, enrich_with_churn,
};
pub use formatters::{format_json, format_markdown, format_text, format_text_with_code};
pub use types::{CaseSensitivity, QueryOptions, QueryResult, RankBy, SearchMode};

// Engine methods are impl'd on AgentContextIndex directly (in engine.rs),
// so they're available wherever AgentContextIndex is imported.

#[cfg(test)]
mod tests;
