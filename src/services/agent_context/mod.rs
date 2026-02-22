//! Agent Context - RAG-Powered Code Search for Agents
//!
//! This module provides semantic code search with quality annotations,
//! designed for use by AI agents (Claude Code, Cline, etc.) instead of grep.
//!
//! # Architecture (Toyota Way - Jidoka)
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                  Agent Context Pipeline                          │
//! ├─────────────────────────────────────────────────────────────────┤
//! │  1. AST Parsing     │ tree-sitter extracts functions            │
//! │  2. TDG Annotation  │ Complexity, SATD, Big-O scores            │
//! │  3. Embedding       │ trueno-rag BM25 + optional vectors        │
//! │  4. Query           │ Semantic search with quality filtering    │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Why This Exists
//!
//! Agents grep/glob the codebase, which:
//! - Returns 500+ irrelevant matches
//! - Has no quality context (complexity, TDG grade)
//! - Wastes tokens on raw text
//!
//! This module provides:
//! - Semantic search ("find error handling")
//! - Quality-ranked results (TDG grade, complexity)
//! - Structured output (signatures, docs, metrics)
//!
//! # Usage
//!
//! ```rust,no_run
//! use pmat::services::agent_context::{AgentContextIndex, QueryOptions};
//!
//! // Build index
//! let index = AgentContextIndex::build(std::path::Path::new("."))?;
//!
//! // Query with quality filters
//! let results = index.query("error handling", QueryOptions {
//!     limit: 5,
//!     min_grade: Some("B".to_string()),
//!     max_complexity: Some(15),
//!     ..Default::default()
//! })?;
//!
//! for r in results {
//!     println!("{} - {} (TDG: {}, Complexity: {})",
//!         r.file_path, r.function_name, r.tdg_grade, r.complexity);
//! }
//! # Ok::<(), String>(())
//! ```
//!
//! # Specification
//!
//! See: `docs/specifications/improve-context.md`

pub(crate) mod document_index;
pub(crate) mod function_index;
pub(crate) mod query;

pub(crate) use document_index::DocumentResult;
pub(crate) use function_index::helpers::parse_workspace_siblings;
pub(crate) use function_index::sqlite_backend::persist_entropy_violations;
pub(crate) use function_index::sqlite_backend::persist_provability_scores;
pub(crate) use function_index::sqlite_backend::persist_quality_violations;
pub use function_index::{
    AgentContextIndex, FunctionEntry, GraphMetrics, IndexManifest, IndexStats, QualityMetrics,
};
pub use query::coverage_exclusion::{classify_exclusions, CoverageExclusion, ExclusionSummary};
pub use query::ptx_diagnostics::{
    format_ptx_diagnostics_json, format_ptx_diagnostics_text, run_ptx_diagnostics,
    PtxDiagnosticResult,
};
pub use query::ptx_flow::{
    classify_ptx_role, format_ptx_flow_json, format_ptx_flow_text, trace_ptx_dataflow, PtxFlowNode,
    PtxFlowResult, PtxRole,
};
pub use query::raw_search::{
    is_within_indexed_function, raw_search, FileMatchCount, RawSearchOptions, RawSearchOutput,
    RawSearchResult,
};
pub use query::{
    build_churn_map, build_coverage_map, compute_impact_score, enrich_results_with_churn,
    enrich_results_with_coverage, enrich_results_with_duplicates, enrich_results_with_entropy,
    enrich_results_with_faults, enrich_with_churn, enrich_with_coverage, enrich_with_coverage_diff,
    format_coverage_summary, format_json, format_markdown, format_text, format_text_with_code,
    load_workspace_coverage, suggest_renames, CaseSensitivity, QueryOptions, QueryResult, RankBy,
    RenameSignal, RenameSuggestion, SearchMode,
};
