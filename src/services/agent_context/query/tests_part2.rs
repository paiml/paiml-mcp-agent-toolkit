#![cfg_attr(coverage_nightly, coverage(off))]

use super::engine::glob_matches;
use super::enrichment::{
    build_churn_map, enrich_results_with_churn, enrich_results_with_duplicates,
    enrich_results_with_entropy, enrich_results_with_faults, enrich_with_churn,
};
use super::formatters::{format_markdown, format_text, format_text_with_code};
use super::types::{CaseSensitivity, QueryOptions, QueryResult, SearchMode};
use crate::services::agent_context::function_index::DefinitionType;
use crate::services::agent_context::{AgentContextIndex, FunctionEntry, QualityMetrics};
use std::collections::{HashMap, HashSet};

include!("tests_part2_helpers.rs");
include!("tests_part2_formatters.rs");
include!("tests_part2_query_enrichment.rs");
include!("tests_part2_sync_enrichment.rs");
include!("tests_part2_async_enrichment.rs");
include!("tests_part2_fault_annotations.rs");
