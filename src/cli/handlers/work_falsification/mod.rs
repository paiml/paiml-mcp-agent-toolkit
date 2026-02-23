#![cfg_attr(coverage_nightly, coverage(off))]
//! Work Falsification: Popperian Falsification Executor
//!
//! Runs all falsification tests and BLOCKS completion if ANY fail.
//! Based on Karl Popper's demarcation criterion: claims must be falsifiable.
//!
//! **O(1) CRITICAL:** All checks read from cached metrics (<100ms total).
//! Cache is populated by pre-commit hooks and CI pipelines.
//!
//! Based on: docs/specifications/improve-pmat-work.md

pub mod advanced_checks;
pub mod cache;
pub mod churn_checks;
pub mod core_checks;
pub mod formal_checks;
pub mod runner;
pub mod supply_chain_checks;
mod tests;
pub mod types;

// Re-export public API
pub use cache::capture_baseline;
pub use runner::run_falsification_tests;
pub use types::{ClaimResult, FalsificationReport};
