#![cfg_attr(coverage_nightly, coverage(off))]
//! ComputeBrick Pattern Detection for PMAT Compliance
//!
//! Extracted from comply_handlers.rs for file health compliance (CB-040).
//! Contains CB pattern detection functions and check_compute_brick.

mod dependency_checks;
mod lua_best_practices;
mod markdown_best_practices;
mod model_quality;
mod quality_checks;
mod rust_best_practices;
mod safety_checks;
mod scala_best_practices;
mod sql_best_practices;
mod types;
mod yaml_best_practices;

pub use dependency_checks::*;
pub use lua_best_practices::*;
pub use markdown_best_practices::*;
pub use model_quality::*;
pub use quality_checks::*;
pub use rust_best_practices::*;
pub use safety_checks::*;
pub use scala_best_practices::*;
pub use sql_best_practices::*;
pub use types::*;
pub use yaml_best_practices::*;

#[cfg(test)]
mod tests;
