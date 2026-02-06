#![cfg_attr(coverage_nightly, coverage(off))]
//! Function Index - RAG Index for Agent Context
//!
//! Builds a searchable index of all functions in a project with quality annotations.

mod build;
pub(crate) mod helpers;
mod types;

pub use types::{
    AgentContextIndex, DefinitionType, FunctionEntry, GraphMetrics,
    IndexManifest, IndexStats, QualityMetrics,
};

// Re-export pub(crate) items used by sibling test modules
pub(crate) use helpers::{build_call_graph, build_indices, compute_graph_metrics, compute_name_frequency};

#[cfg(test)]
mod tests;
