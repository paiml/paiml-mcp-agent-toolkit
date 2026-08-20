#![cfg_attr(coverage_nightly, coverage(off))]
//! Proof annotation collection service with parallel processing
//!
//! This service orchestrates the collection of proof annotations from multiple sources
//! including Rust borrow checker, companion files, and inline code annotations.

use crate::models::unified_ast::{
    ConfidenceLevel, Location, ProofAnnotation, ProofMap, PropertyType, VerificationMethod,
};
use crate::services::symbol_table::SymbolTable;
use parking_lot::RwLock;
use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::task::JoinSet;
use tracing::{debug, error, info, warn};

/// Errors that can occur during proof collection
#[derive(Debug, Error)]
pub enum ProofCollectionError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error in file {path}: {message}")]
    Parse {
        path: std::path::PathBuf,
        message: String,
    },

    #[error("Invalid metadata: {0}")]
    InvalidMetadata(String),

    #[error("Proof source error: {0}")]
    ProofSource(String),
}

/// Result of proof collection from a single source
#[derive(Debug)]
pub struct ProofCollectionResult {
    pub annotations: Vec<(Location, ProofAnnotation)>,
    pub errors: Vec<ProofCollectionError>,
    pub metrics: CollectionMetrics,
}

/// Metrics for proof collection performance
#[derive(Debug, Default)]
pub struct CollectionMetrics {
    pub files_processed: usize,
    pub annotations_found: usize,
    pub cache_hits: usize,
    pub duration_ms: u64,
}

/// Trait for proof sources that can collect annotations
pub trait ProofSource: Send + Sync {
    /// Collect proof annotations from the project
    fn collect(
        &self,
        project_root: &Path,
        cache: &Arc<RwLock<ProofCache>>,
        symbol_table: &Arc<SymbolTable>,
    ) -> Pin<
        Box<dyn Future<Output = Result<ProofCollectionResult, ProofCollectionError>> + Send + '_>,
    >;

    /// Clone the proof source (needed for dynamic dispatch)
    fn clone_box(&self) -> Box<dyn ProofSource>;
}

/// Simple in-memory cache for proof annotations
#[derive(Debug, Default)]
pub struct ProofCache {
    /// Cache key is content hash + file modification time
    cache: std::collections::HashMap<String, Vec<ProofAnnotation>>,
    /// File modification times for cache invalidation
    file_times: std::collections::HashMap<std::path::PathBuf, std::time::SystemTime>,
}

/// Main proof annotation service
pub struct ProofAnnotator {
    sources: Vec<Box<dyn ProofSource>>,
    cache: Arc<RwLock<ProofCache>>,
    symbol_table: Arc<SymbolTable>,
    /// Files the last `collect_proofs` could not read or parse.
    ///
    /// These used to be counted, logged once at `warn!` and thrown away, so a
    /// run over this repo skipped files whose annotations are simply absent
    /// from the report — and the report said "Total proofs: N" as if it had
    /// seen everything. Retained so the renderers can disclose the gap.
    collection_errors: std::sync::atomic::AtomicUsize,
    /// Files the last `collect_proofs` actually read — the denominator of
    /// "Total proofs: N".
    ///
    /// Every source already measures this as `CollectionMetrics::files_processed`
    /// and the merge threw it away, which left `analyze proof-annotations`
    /// unable to tell "scanned the tree, found no annotations" from "there was
    /// nothing to scan": both printed "Total proofs: 0" and exited 0 (#1015).
    files_processed: std::sync::atomic::AtomicUsize,
}

/// Cache statistics
#[derive(Debug)]
pub struct CacheStats {
    pub size: usize,
    pub files_tracked: usize,
}

/// Mock proof source for testing
#[derive(Debug, Clone)]
pub struct MockProofSource {
    pub name: String,
    pub delay_ms: u64,
    pub annotation_count: usize,
}

// --- Implementation files ---
include!("proof_annotator_cache.rs");
include!("proof_annotator_core.rs");
include!("proof_annotator_mock.rs");
include!("proof_annotator_tests.rs");
