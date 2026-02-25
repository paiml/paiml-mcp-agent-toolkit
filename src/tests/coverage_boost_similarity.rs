#![cfg_attr(coverage_nightly, coverage(off))]
//! Coverage boost tests for services/similarity module
//! Tests for SimilarityDetector, Winnowing, and related types to achieve full coverage
//!
//! Split into include files for maintainability:
//! - similarity_config_clone_type_tests.inc.rs  (~223 lines): SimilarityConfig + CloneType
//! - similarity_detector_tests.inc.rs           (~351 lines): SimilarityDetector methods
//! - winnowing_location_block_tests.inc.rs      (~280 lines): Winnowing + Location + SimilarBlock
//! - entropy_priority_metrics_tests.inc.rs      (~386 lines): Entropy, Priority, Refactoring, Metrics, Report
//! - edge_case_integration_tests.inc.rs         (~326 lines): Hash collision, empty input, integration, boundary

use crate::services::similarity::{
    CloneType, ComprehensiveReport, EntropyBlock, EntropyReport, Location, Metrics, Priority,
    RefactoringHint, SimilarBlock, SimilarityConfig, SimilarityDetector, Winnowing,
};
use std::path::PathBuf;

// SimilarityConfig and CloneType tests
include!("similarity_config_clone_type_tests.inc.rs");

// SimilarityDetector method tests
include!("similarity_detector_tests.inc.rs");

// Winnowing, Location, and SimilarBlock tests
include!("winnowing_location_block_tests.inc.rs");

// EntropyReport, EntropyBlock, Priority, RefactoringHint, Metrics, ComprehensiveReport tests
include!("entropy_priority_metrics_tests.inc.rs");

// Hash collision, empty input, integration, and boundary tests
include!("edge_case_integration_tests.inc.rs");
