#![cfg_attr(coverage_nightly, coverage(off))]

//! CLI handler for `pmat deps-audit` command
//!
//! Analyzes dependencies and suggests removals/replacements with
//! Sovereign AI stack (trueno ecosystem) alternatives.
//!
//! Uses trueno-graph for dependency graph analysis:
//! - PageRank for criticality scoring
//! - Transitive dependency counts
//! - Bridge/orphan detection

pub mod classify;
pub mod graph;
pub mod handler;
pub mod output;
pub mod pareto;
pub mod parser;
pub mod types;

mod tests;

// Re-export the public API
pub use handler::handle_deps_audit;
pub use types::{DepAnalysis, DepCategory, DepsAuditReport, ParetoEffort, ParetoEntry, SortMode};
