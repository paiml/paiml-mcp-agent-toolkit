#![cfg_attr(coverage_nightly, coverage(off))]
//! Big-O complexity analysis command handlers
//!
//! This module provides handlers for algorithmic complexity analysis
//! using pattern matching and heuristic approaches.

pub(crate) mod filters;
pub(crate) mod handlers;
pub(crate) mod output;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod property_tests;

pub use handlers::handle_analyze_big_o;
pub use output::format_big_o_summary;
