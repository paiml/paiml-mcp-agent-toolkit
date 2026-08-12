#![cfg_attr(coverage_nightly, coverage(off))]
//! Function Index - RAG Index for Agent Context
//!
//! Builds a searchable index of all functions in a project with quality annotations.

mod build;
pub(crate) mod helpers;
pub(crate) mod sqlite_backend;
mod types;

pub(crate) use types::DefinitionType;
pub use types::{
    AgentContextIndex, FunctionEntry, GraphMetrics, IndexManifest, IndexStats, QualityMetrics,
    TDG_SCALE,
};

// Re-export items used by sibling test modules (query/tests.rs)
#[cfg(test)]
pub(crate) use helpers::{
    build_call_graph, build_indices, compute_graph_metrics, compute_name_frequency,
};

#[cfg(test)]
mod sqlite_falsification_tests;
#[cfg(test)]
mod sqlite_falsification_tests_part2;
#[cfg(test)]
mod tests;
