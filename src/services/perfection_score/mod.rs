#![cfg_attr(coverage_nightly, coverage(off))]
//! Perfection Score Service (master-plan-pmat-work-system.md)
//!
//! Aggregates 8 quality metrics into a unified 200-point score:
//! - TDG (40 pts)
//! - Repo Score (30 pts)
//! - Rust Project Score (30 pts)
//! - Popper Score (25 pts)
//! - Test Coverage (25 pts)
//! - Mutation Score (20 pts)
//! - Documentation (15 pts)
//! - Performance (15 pts)
//!
//! PMAT-454: All output normalized to 0-100 scale

pub mod calculator;
pub mod types;

mod calculator_tests;
mod coverage_tests;
mod property_tests;
mod tests;

pub use calculator::PerfectionScoreCalculator;
pub use types::{CategoryScore, CategoryWeights, PerfectionScoreResult, MAX_PERFECTION_SCORE};
