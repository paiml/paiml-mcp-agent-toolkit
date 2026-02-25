use super::engine::{is_test_function, parse_query_prefixes};
use super::enrichment::enrich_with_churn;
use super::formatters::{format_json, format_markdown, format_text, format_text_with_code};
use super::types::{QueryOptions, QueryResult, RankBy};
use crate::services::agent_context::function_index::DefinitionType;
use crate::services::agent_context::{AgentContextIndex, FunctionEntry, QualityMetrics};
use std::collections::{HashMap, HashSet};

// Shared helper constructors (create_test_entry, build_test_index)
include!("tests_helpers.rs");

// Basic QueryResult construction, formatters, and parse_query_prefixes tests
include!("tests_basic.rs");

// Query engine tests: scoped queries, filters, ranking, get_function, find_similar
include!("tests_query.rs");

// Scoring edge cases: empty corpus, no query terms, scoped scoring
include!("tests_scoring.rs");

// Caller summarization and production caller cap tests
include!("tests_callers.rs");

// Enrichment and formatter tests: churn, clones, entropy, faults, graph metrics
include!("tests_formatters.rs");

// Coverage metric formatter tests
include!("tests_coverage.rs");
