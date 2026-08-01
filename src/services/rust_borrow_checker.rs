#![cfg_attr(coverage_nightly, coverage(off))]
//! Rust borrow checker proof source for safety guarantees
//!
//! This module provides proof annotations based on Rust's type system guarantees,
//! including memory safety, thread safety, and termination properties.
//!
//! Split into submodules via include!():
//! - rust_borrow_checker_analysis.rs: AST analysis and unsafe detection
//! - rust_borrow_checker_annotations.rs: ProofAnnotation factory methods
//! - rust_borrow_checker_collection.rs: File processing, caching, ProofSource impl
//! - rust_borrow_checker_tests.rs: Unit and property tests

use crate::models::unified_ast::{
    ConfidenceLevel, EvidenceType, Location, ProofAnnotation, PropertyType, VerificationMethod,
};
use crate::services::proof_annotator::{
    CollectionMetrics, ProofCache, ProofCollectionError, ProofCollectionResult, ProofSource,
};
use crate::services::symbol_table::SymbolTable;
use parking_lot::RwLock;
use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;
use tracing::{debug, info, warn};
use walkdir::WalkDir;

#[cfg(feature = "rust-ast")]
use syn::{Item, ItemFn, ItemImpl, Type};

/// Rust borrow checker proof source
#[derive(Debug, Clone)]
pub struct RustBorrowChecker {
    rustc_version: String,
    rustc_channel: String,
}

/// Internal state for proof collection process
struct CollectionState {
    annotations: Vec<(Location, ProofAnnotation)>,
    errors: Vec<ProofCollectionError>,
    files_processed: usize,
}

impl CollectionState {
    fn new() -> Self {
        Self {
            annotations: Vec::new(),
            errors: Vec::new(),
            files_processed: 0,
        }
    }
}

impl RustBorrowChecker {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    /// # Errors
    ///
    /// Currently infallible; the signature is kept for callers.
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // These name the analysis that ACTUALLY runs. This used to report
        // tool_name "rustc-stable" and tool_version "1.70.0 (unknown)", but pmat
        // never invokes rustc -- it parses with syn and reasons about the
        // result. Attributing findings to a compiler that never ran, on a
        // version nobody queried, is fabricated provenance on a proof artifact.
        Ok(Self {
            rustc_version: env!("CARGO_PKG_VERSION").to_string(),
            rustc_channel: "syn-static-analysis".to_string(),
        })
    }
}

impl Default for RustBorrowChecker {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            rustc_version: "unknown".to_string(),
            rustc_channel: "stable".to_string(),
        })
    }
}

// Analysis: unsafe detection, thread safety analysis, file analysis, item analysis
include!("rust_borrow_checker_analysis.rs");

// Annotation factories: memory safety, thread safety, termination, auto trait
include!("rust_borrow_checker_annotations.rs");

// Collection: file processing, caching, ProofSource trait impl
include!("rust_borrow_checker_collection.rs");

// Tests: unit tests and property tests
include!("rust_borrow_checker_tests.rs");
