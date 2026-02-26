#![cfg_attr(coverage_nightly, coverage(off))]
//! README and documentation hallucination detection CLI handlers
//!
//! Validates AI-generated documentation against codebase facts using semantic
//! entropy-based hallucination detection.
//!
//! Based on peer-reviewed research:
//! - Semantic Entropy (Farquhar et al., Nature 2024)
//! - MIND framework (IJCAI 2025)
//! - Unified Detection Framework (Complex & Intelligent Systems 2025)

mod execution;
pub(crate) mod output;
mod types;

pub use types::{OutputFormat, ValidateReadmeCmd};

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_edge_cases;
#[cfg(test)]
mod tests_output;
