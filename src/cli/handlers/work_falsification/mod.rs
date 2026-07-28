#![cfg_attr(coverage_nightly, coverage(off))]
//! Work Falsification: Popperian Falsification Executor
//!
//! Runs all falsification tests and BLOCKS completion if ANY fail.
//! Based on Karl Popper's demarcation criterion: claims must be falsifiable.
//!
//! **O(1) CRITICAL:** On the hot path every check reads cached metrics
//! (<100ms total). The one exception is the supply-chain claim: when its cache
//! is stale or absent it costs one bounded `cargo deny check` (see
//! [`deny_refresh`]), because nothing else has ever written that cache and the
//! claim was otherwise unsatisfiable (GH #629).
//!
//! Based on: docs/specifications/improve-pmat-work.md

pub mod advanced_checks;
pub mod cache;
pub mod churn_checks;
pub mod core_checks;
pub mod deny_refresh;
pub mod formal_checks;
pub mod lint_refresh;
pub mod pmat_owned_state;
pub mod pre_run_tree;
pub mod runner;
pub mod supply_chain_checks;
mod tests;
pub mod types;

// Re-export public API
pub use cache::capture_baseline;
pub use pre_run_tree::{snapshot_working_tree, TreeSnapshot};
pub use runner::run_falsification_tests;
pub use types::{ClaimResult, FalsificationReport};
