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
mod lua_best_practices;
mod sql_best_practices;
mod yaml_best_practices;
mod markdown_best_practices;
mod model_quality;

pub use types::*;
pub use safety_checks::*;
pub use quality_checks::*;
pub use dependency_checks::*;
pub use rust_best_practices::*;
pub use lua_best_practices::*;
pub use sql_best_practices::*;
pub use yaml_best_practices::*;
pub use markdown_best_practices::*;
pub use model_quality::*;

#[cfg(test)]
mod tests;
