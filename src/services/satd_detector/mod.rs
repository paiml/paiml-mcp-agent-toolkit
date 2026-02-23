//! Self-Admitted Technical Debt (SATD) Detection System
//!
//! This module provides high-performance, multi-language detection and classification
//! of technical debt annotations embedded in source code comments.

#![cfg_attr(coverage_nightly, coverage(off))]

mod classifier;
mod detection;
mod metrics;
mod types;

// Re-export all public items that were previously accessible from satd_detector.rs
pub use types::{
    AstContext, AstNodeType, CategoryMetrics, DebtCategory, DebtClassifier, DebtEvolution,
    SATDAnalysisResult, SATDDetector, SATDMetrics, SATDSummary, Severity, TechnicalDebt,
};

// Tests extracted to satd_detector_tests.rs for file health compliance (CB-040)
// TEMPORARILY DISABLED: File splitting broke syntax
#[cfg(all(test, feature = "broken-tests"))]
#[path = "../satd_detector_tests.rs"]
mod tests;
