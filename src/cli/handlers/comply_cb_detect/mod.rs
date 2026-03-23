#![cfg_attr(coverage_nightly, coverage(off))]
//! ComputeBrick Pattern Detection for PMAT Compliance
//!
//! Extracted from comply_handlers.rs for file health compliance (CB-040).
//! Contains CB pattern detection functions and check_compute_brick.

mod dependency_checks;
mod lean_best_practices;
mod lua_best_practices;
mod markdown_best_practices;
mod model_quality;
mod quality_checks;
mod rust_best_practices;
mod rust_best_practices_extended;
mod safety_checks;
mod scala_best_practices;
mod spec_work_traceability;
mod sql_best_practices;
mod stale_paths;
mod types;
mod yaml_best_practices;

pub use dependency_checks::*;
pub use lean_best_practices::*;
pub use lua_best_practices::*;
pub use markdown_best_practices::*;
pub use model_quality::*;
pub use quality_checks::*;
pub use rust_best_practices::*;
pub use rust_best_practices_extended::*;
pub use safety_checks::*;
pub use scala_best_practices::*;
pub use spec_work_traceability::*;
pub use sql_best_practices::*;
pub use stale_paths::*;
pub use types::*;
pub use yaml_best_practices::*;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_part2;
#[cfg(test)]
mod tests_part3;
