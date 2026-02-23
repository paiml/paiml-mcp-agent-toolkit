#![cfg_attr(coverage_nightly, coverage(off))]
//! Comprehensive Analysis Handler
//!
//! Orchestrates multiple analysis types using the facade pattern.

pub(crate) mod handler;
pub(crate) mod helpers;
pub(crate) mod output;
pub(crate) mod types;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod tests_part2;

#[cfg(test)]
mod property_tests;

pub use handler::handle_analyze_comprehensive;
pub use types::ComprehensiveAnalysisConfig;
