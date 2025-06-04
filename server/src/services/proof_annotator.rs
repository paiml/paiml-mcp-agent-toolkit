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

impl ProofCache {
    pub fn new() -> Self {
        Self {
            cache: std::collections::HashMap::new(),
            file_times: std::collections::HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&Vec<ProofAnnotation>> {
        self.cache.get(key)
    }

    pub fn insert(&mut self, key: String, annotations: Vec<ProofAnnotation>) {
        self.cache.insert(key, annotations);
    }

    pub fn is_file_cached(&self, path: &Path) -> bool {
        if let Ok(metadata) = std::fs::metadata(path) {
            if let Ok(modified) = metadata.modified() {
                if let Some(cached_time) = self.file_times.get(path) {
                    return modified <= *cached_time;
                }
            }
        }
        false
    }

    pub fn update_file_time(&mut self, path: std::path::PathBuf) {
        if let Ok(metadata) = std::fs::metadata(&path) {
            if let Ok(modified) = metadata.modified() {
                self.file_times.insert(path, modified);
            }
        }
    }

    pub fn clear(&mut self) {
        self.cache.clear();
        self.file_times.clear();
    }

    pub fn size(&self) -> usize {
        self.cache.len()
    }
}

/// Main proof annotation service
pub struct ProofAnnotator {
    sources: Vec<Box<dyn ProofSource>>,
    cache: Arc<RwLock<ProofCache>>,
    symbol_table: Arc<SymbolTable>,
}

impl ProofAnnotator {
    pub fn new(symbol_table: Arc<SymbolTable>) -> Self {
        Self {
            sources: Vec::new(),
            cache: Arc::new(RwLock::new(ProofCache::new())),
            symbol_table,
        }
    }

    /// Add a proof source to the annotator
    pub fn add_source<T: ProofSource + 'static>(&mut self, source: T) {
        self.sources.push(Box::new(source));
    }

    /// Collect proof annotations from all sources in parallel
    pub async fn collect_proofs(&self, project_root: &Path) -> ProofMap {
        let start = Instant::now();
        let mut join_set = JoinSet::new();

        info!(
            "Starting proof collection with {} sources",
            self.sources.len()
        );

        // Launch parallel collection tasks
        for (index, source) in self.sources.iter().enumerate() {
            let root = project_root.to_owned();
            let source_clone = source.clone_box();
            let cache = self.cache.clone();
            let symbols = self.symbol_table.clone();

            join_set.spawn(async move {
                debug!("Starting proof collection from source {}", index);
                let result = source_clone.collect(&root, &cache, &symbols).await;
                debug!(
                    "Completed proof collection from source {}: {:?}",
                    index,
                    result.as_ref().map(|r| r.metrics.annotations_found)
                );
                (index, result)
            });
        }

        // Collect results from all sources
        let mut all_results = Vec::new();
        while let Some(task_result) = join_set.join_next().await {
            match task_result {
                Ok((index, Ok(result))) => {
                    debug!(
                        "Source {} collected {} annotations",
                        index,
                        result.annotations.len()
                    );
                    all_results.push(result);
                }
                Ok((index, Err(e))) => {
                    error!("Proof collection failed for source {}: {}", index, e);
                }
                Err(e) => {
                    error!("Task panic during proof collection: {}", e);
                }
            }
        }

        // Merge results with conflict resolution
        let proof_map = self.merge_with_conflict_resolution(all_results);

        let elapsed = start.elapsed();
        let total_annotations = proof_map.values().map(|v| v.len()).sum::<usize>();

        info!(
            "Proof collection completed in {}ms: {} annotations from {} sources",
            elapsed.as_millis(),
            total_annotations,
            self.sources.len()
        );

        proof_map
    }

    /// Merge results from multiple sources with conflict resolution
    fn merge_with_conflict_resolution(&self, results: Vec<ProofCollectionResult>) -> ProofMap {
        let mut proof_map: ProofMap = std::collections::HashMap::new();
        let mut total_errors = 0;

        // Define verification method hierarchy for conflict resolution
        let method_rank = |m: &VerificationMethod| -> u32 {
            match m {
                VerificationMethod::FormalProof { .. } => 4,
                VerificationMethod::ModelChecking { bounded: false } => 3,
                VerificationMethod::ModelChecking { bounded: true } => 2,
                VerificationMethod::StaticAnalysis { .. } => 2,
                VerificationMethod::AbstractInterpretation => 2,
                VerificationMethod::BorrowChecker => 1,
            }
        };

        for result in results {
            total_errors += result.errors.len();

            for (loc, annotation) in result.annotations {
                let loc_clone = loc.clone();
                proof_map
                    .entry(loc)
                    .and_modify(|existing| {
                        // Complex deduplication: same property, different methods
                        let key = (&annotation.property_proven, &annotation.specification_id);

                        if let Some(idx) = existing
                            .iter()
                            .position(|a| (&a.property_proven, &a.specification_id) == key)
                        {
                            let existing_score = (
                                existing[idx].confidence_level as u32,
                                method_rank(&existing[idx].method),
                                existing[idx].assumptions.is_empty() as u32,
                            );

                            let new_score = (
                                annotation.confidence_level as u32,
                                method_rank(&annotation.method),
                                annotation.assumptions.is_empty() as u32,
                            );

                            if new_score > existing_score {
                                debug!(
                                "Replacing {:?} proof with higher confidence {:?} proof at {:?}",
                                existing[idx].method, annotation.method, loc_clone
                            );
                                existing[idx] = annotation.clone();
                            }
                        } else {
                            existing.push(annotation.clone());
                        }
                    })
                    .or_insert_with(|| vec![annotation]);
            }
        }

        if total_errors > 0 {
            warn!(
                "Encountered {} errors during proof collection",
                total_errors
            );
        }

        proof_map
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        let cache = self.cache.read();
        CacheStats {
            size: cache.size(),
            files_tracked: cache.file_times.len(),
        }
    }

    /// Clear the cache
    pub fn clear_cache(&self) {
        self.cache.write().clear();
    }
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

impl MockProofSource {
    pub fn new(name: String, delay_ms: u64, annotation_count: usize) -> Self {
        Self {
            name,
            delay_ms,
            annotation_count,
        }
    }
}

impl ProofSource for MockProofSource {
    fn clone_box(&self) -> Box<dyn ProofSource> {
        Box::new(self.clone())
    }

    fn collect(
        &self,
        _project_root: &Path,
        _cache: &Arc<RwLock<ProofCache>>,
        _symbol_table: &Arc<SymbolTable>,
    ) -> Pin<
        Box<dyn Future<Output = Result<ProofCollectionResult, ProofCollectionError>> + Send + '_>,
    > {
        let delay_ms = self.delay_ms;
        let annotation_count = self.annotation_count;
        let tool_name = self.name.clone();

        Box::pin(async move {
            // Simulate some work
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;

            let mut annotations = Vec::new();

            // Generate mock annotations with unique file names per source
            for i in 0..annotation_count {
                let location = Location::new(
                    std::path::PathBuf::from(format!("{}_{}.rs", tool_name, i)),
                    i as u32 * 10,
                    (i as u32 + 1) * 10,
                );

                let annotation = ProofAnnotation {
                    annotation_id: uuid::Uuid::new_v4(),
                    property_proven: PropertyType::MemorySafety,
                    specification_id: None,
                    method: VerificationMethod::BorrowChecker,
                    tool_name: tool_name.clone(),
                    tool_version: "1.0.0".to_string(),
                    confidence_level: ConfidenceLevel::Medium,
                    assumptions: vec![],
                    evidence_type:
                        crate::models::unified_ast::EvidenceType::ImplicitTypeSystemGuarantee,
                    evidence_location: None,
                    date_verified: chrono::Utc::now(),
                };

                annotations.push((location, annotation));
            }

            Ok(ProofCollectionResult {
                annotations,
                errors: Vec::new(),
                metrics: CollectionMetrics {
                    files_processed: annotation_count,
                    annotations_found: annotation_count,
                    cache_hits: 0,
                    duration_ms: delay_ms,
                },
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_proof_annotator_basic() {
        let symbol_table = Arc::new(SymbolTable::new());
        let mut annotator = ProofAnnotator::new(symbol_table);

        // Add a mock source
        annotator.add_source(MockProofSource::new("test".to_string(), 10, 2));

        let project_root = PathBuf::from(".");
        let proof_map = annotator.collect_proofs(&project_root).await;

        assert_eq!(proof_map.len(), 2);
    }

    #[tokio::test]
    async fn test_proof_annotator_parallel_sources() {
        let symbol_table = Arc::new(SymbolTable::new());
        let mut annotator = ProofAnnotator::new(symbol_table);

        // Add multiple mock sources with different file names to avoid conflicts
        annotator.add_source(MockProofSource::new("source1".to_string(), 50, 1));
        annotator.add_source(MockProofSource::new("source2".to_string(), 30, 1));
        annotator.add_source(MockProofSource::new("source3".to_string(), 20, 1));

        let start = Instant::now();
        let project_root = PathBuf::from(".");
        let proof_map = annotator.collect_proofs(&project_root).await;
        let elapsed = start.elapsed();

        // Should complete in roughly the time of the slowest source (50ms)
        // rather than the sum of all sources (100ms)
        assert!(elapsed.as_millis() < 100);

        // Each source generates 1 annotation with unique file names, so no conflicts
        assert_eq!(proof_map.len(), 3, "Should have 3 unique annotations");
    }

    #[tokio::test]
    async fn test_proof_cache() {
        let mut cache = ProofCache::new();

        let annotations = vec![ProofAnnotation {
            annotation_id: uuid::Uuid::new_v4(),
            property_proven: PropertyType::MemorySafety,
            specification_id: None,
            method: VerificationMethod::BorrowChecker,
            tool_name: "test".to_string(),
            tool_version: "1.0.0".to_string(),
            confidence_level: ConfidenceLevel::High,
            assumptions: vec![],
            evidence_type: crate::models::unified_ast::EvidenceType::ImplicitTypeSystemGuarantee,
            evidence_location: None,
            date_verified: chrono::Utc::now(),
        }];

        cache.insert("test_key".to_string(), annotations.clone());

        assert_eq!(cache.get("test_key"), Some(&annotations));
        assert_eq!(cache.size(), 1);

        cache.clear();
        assert_eq!(cache.size(), 0);
    }
}
