#![cfg_attr(coverage_nightly, coverage(off))]
//! Simplified Deep Context Analysis - Phase 4 implementation
//!
//! A streamlined deep context analysis implementation that focuses on
//! integrating with existing services without complex dependencies.

mod analyzer;
mod function_names;
mod heuristics;
mod language_complexity;
mod types;

pub use types::{
    ComplexityMetrics, FileComplexityDetail, SimpleAnalysisConfig, SimpleAnalysisReport,
    SimpleDeepContext,
};

#[cfg(test)]
mod tests;
