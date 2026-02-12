#![cfg_attr(coverage_nightly, coverage(off))]
//! ComputeBrick Pattern Detection for PMAT Compliance
//!
//! Extracted from comply_handlers.rs for file health compliance (CB-040).
//! Contains CB pattern detection functions and check_compute_brick.

mod types;
mod safety_checks;
mod quality_checks;
mod dependency_checks;
mod rust_best_practices;

pub use types::*;
pub use safety_checks::*;
pub use quality_checks::*;
pub use dependency_checks::*;
pub use rust_best_practices::*;

#[cfg(test)]
mod tests;
