//! Rust borrow checker proof source for safety guarantees
//!
//! This module provides proof annotations based on Rust's type system guarantees,
//! including memory safety, thread safety, and termination properties.

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
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Use a simple default version since we don't have rustc_version crate
        let version = "1.70.0 (unknown)".to_string();
        let channel = "stable".to_string();

        Ok(Self {
            rustc_version: version,
            rustc_channel: channel,
        })
    }

    /// Check if an item contains unsafe code
    #[cfg(feature = "rust-ast")]
    fn contains_unsafe(&self, item_fn: &ItemFn) -> bool {
        // Simple check for unsafe keyword
        item_fn.sig.unsafety.is_some()
    }

    #[cfg(not(feature = "rust-ast"))]
    fn contains_unsafe(&self, _content: &str) -> bool {
        // Without syn, do a simple text search
        _content.contains("unsafe")
    }

    /// Check if an impl block contains unsafe code
    #[cfg(feature = "rust-ast")]
    fn contains_unsafe_impl(&self, item_impl: &ItemImpl) -> bool {
        item_impl.unsafety.is_some()
    }

    /// Analyze thread safety via trait bounds and type analysis
    #[cfg(feature = "rust-ast")]
    fn analyze_thread_safety(&self, item_fn: &ItemFn) -> Option<ProofAnnotation> {
        // Conservative analysis: only if all parameters appear to be Send+Sync
        let params_likely_send_sync = item_fn.sig.inputs.iter().all(|arg| {
            match arg {
                syn::FnArg::Typed(pat_type) => {
                    // Simple heuristic: check if the type looks like it implements Send+Sync
                    self.type_likely_implements_send_sync(&pat_type.ty)
                }
                _ => true, // self is Send+Sync if the type is
            }
        });

        if params_likely_send_sync {
            Some(self.create_thread_safety_annotation())
        } else {
            None
        }
    }

    /// Simple heuristic to check if a type likely implements Send+Sync
    #[cfg(all(feature = "rust-ast", feature = "quote"))]
    fn type_likely_implements_send_sync(&self, ty: &Type) -> bool {
        match ty {
            Type::Path(path) => {
                let path_str = quote::quote!(#path).to_string();
                // Common Send+Sync types
                matches!(
                    path_str.as_str(),
                    "String"
                        | "i32"
                        | "u32"
                        | "i64"
                        | "u64"
                        | "f32"
                        | "f64"
                        | "bool"
                        | "char"
                        | "usize"
                        | "isize"
                        | "Vec"
                        | "HashMap"
                        | "BTreeMap"
                        | "Arc"
                        | "Mutex"
                        | "RwLock"
                )
            }
            Type::Reference(_) => true, // &T is Send+Sync if T is
            _ => false,                 // Conservative default
        }
    }

    /// Fallback implementation without quote
    #[cfg(all(feature = "rust-ast", not(feature = "quote")))]
    fn type_likely_implements_send_sync(&self, _ty: &Type) -> bool {
        // Conservative default when we can't analyze the type
        false
    }

    /// Check if a trait path is an auto trait (Send, Sync, etc.)
    #[cfg(all(feature = "rust-ast", feature = "quote"))]
    fn is_auto_trait(&self, trait_path: &syn::Path) -> bool {
        let path_str = quote::quote!(#trait_path).to_string();
        matches!(
            path_str.as_str(),
            "Send" | "Sync" | "Unpin" | "UnwindSafe" | "RefUnwindSafe"
        )
    }

    /// Fallback implementation without quote
    #[cfg(all(feature = "rust-ast", not(feature = "quote")))]
    fn is_auto_trait(&self, trait_path: &syn::Path) -> bool {
        // Simple check based on the last segment
        if let Some(segment) = trait_path.segments.last() {
            matches!(
                segment.ident.to_string().as_str(),
                "Send" | "Sync" | "Unpin" | "UnwindSafe" | "RefUnwindSafe"
            )
        } else {
            false
        }
    }

    /// Create memory safety annotation
    fn memory_safety_annotation(&self) -> ProofAnnotation {
        ProofAnnotation {
            annotation_id: uuid::Uuid::new_v4(),
            property_proven: PropertyType::MemorySafety,
            specification_id: None,
            method: VerificationMethod::BorrowChecker,
            tool_name: format!("rustc-{}", self.rustc_channel),
            tool_version: self.rustc_version.clone(),
            confidence_level: ConfidenceLevel::High,
            assumptions: vec![
                "Safe Rust subset".to_string(),
                "No compiler bugs".to_string(),
            ],
            evidence_type: EvidenceType::ImplicitTypeSystemGuarantee,
            evidence_location: None,
            date_verified: chrono::Utc::now(),
        }
    }

    /// Create thread safety annotation
    fn create_thread_safety_annotation(&self) -> ProofAnnotation {
        ProofAnnotation {
            annotation_id: uuid::Uuid::new_v4(),
            property_proven: PropertyType::ThreadSafety,
            specification_id: None,
            method: VerificationMethod::BorrowChecker,
            tool_name: format!("rustc-{}", self.rustc_channel),
            tool_version: self.rustc_version.clone(),
            confidence_level: ConfidenceLevel::High,
            assumptions: vec![
                "Send + Sync bounds satisfied".to_string(),
                "No interior mutability without synchronization".to_string(),
            ],
            evidence_type: EvidenceType::ImplicitTypeSystemGuarantee,
            evidence_location: None,
            date_verified: chrono::Utc::now(),
        }
    }

    /// Create const fn termination annotation
    fn const_fn_termination(&self) -> ProofAnnotation {
        ProofAnnotation {
            annotation_id: uuid::Uuid::new_v4(),
            property_proven: PropertyType::Termination,
            specification_id: None,
            method: VerificationMethod::BorrowChecker,
            tool_name: format!("rustc-{}", self.rustc_channel),
            tool_version: self.rustc_version.clone(),
            confidence_level: ConfidenceLevel::High,
            assumptions: vec!["const fn restrictions guarantee termination".to_string()],
            evidence_type: EvidenceType::ImplicitTypeSystemGuarantee,
            evidence_location: None,
            date_verified: chrono::Utc::now(),
        }
    }

    /// Create auto trait annotation
    #[cfg(all(feature = "rust-ast", feature = "quote"))]
    fn auto_trait_annotation(&self, trait_path: &syn::Path) -> ProofAnnotation {
        let trait_name = quote::quote!(#trait_path).to_string();
        let property = match trait_name.as_str() {
            "Send" | "Sync" => PropertyType::ThreadSafety,
            _ => PropertyType::MemorySafety,
        };

        ProofAnnotation {
            annotation_id: uuid::Uuid::new_v4(),
            property_proven: property,
            specification_id: Some(format!("auto_trait_{trait_name}")),
            method: VerificationMethod::BorrowChecker,
            tool_name: format!("rustc-{}", self.rustc_channel),
            tool_version: self.rustc_version.clone(),
            confidence_level: ConfidenceLevel::High,
            assumptions: vec![format!("{} auto trait implementation", trait_name)],
            evidence_type: EvidenceType::ImplicitTypeSystemGuarantee,
            evidence_location: None,
            date_verified: chrono::Utc::now(),
        }
    }

    /// Create auto trait annotation without quote
    #[cfg(all(feature = "rust-ast", not(feature = "quote")))]
    fn auto_trait_annotation(&self, trait_path: &syn::Path) -> ProofAnnotation {
        let trait_name = if let Some(segment) = trait_path.segments.last() {
            segment.ident.to_string()
        } else {
            "Unknown".to_string()
        };

        let property = match trait_name.as_str() {
            "Send" | "Sync" => PropertyType::ThreadSafety,
            _ => PropertyType::MemorySafety,
        };

        ProofAnnotation {
            annotation_id: uuid::Uuid::new_v4(),
            property_proven: property,
            specification_id: Some(format!("auto_trait_{}", trait_name)),
            method: VerificationMethod::BorrowChecker,
            tool_name: format!("rustc-{}", self.rustc_channel),
            tool_version: self.rustc_version.clone(),
            confidence_level: ConfidenceLevel::High,
            assumptions: vec![format!("{} auto trait implementation", trait_name)],
            evidence_type: EvidenceType::ImplicitTypeSystemGuarantee,
            evidence_location: None,
            date_verified: chrono::Utc::now(),
        }
    }

    /// Analyze a Rust file and extract proof annotations
    #[cfg(feature = "rust-ast")]
    fn analyze_rust_file(
        &self,
        file_path: &Path,
    ) -> Result<Vec<(Location, ProofAnnotation)>, ProofCollectionError> {
        let content = std::fs::read_to_string(file_path).map_err(ProofCollectionError::Io)?;

        let syntax = syn::parse_file(&content).map_err(|e| ProofCollectionError::Parse {
            path: file_path.to_owned(),
            message: format!("Syntax error: {e}"),
        })?;

        let mut annotations = Vec::new();

        for item in &syntax.items {
            let item_annotations = self.analyze_item(item, file_path);
            annotations.extend(item_annotations);
        }

        Ok(annotations)
    }

    /// Analyze an item and generate proof annotations
    #[cfg(feature = "rust-ast")]
    fn analyze_item(&self, item: &Item, file_path: &Path) -> Vec<(Location, ProofAnnotation)> {
        let mut annotations = Vec::new();

        match item {
            Item::Fn(item_fn) if !self.contains_unsafe(item_fn) => {
                // Extract location from span (simplified)
                let start = 0u32; // Would need proper span handling
                let end = 100u32;
                let loc = Location::new(file_path.to_owned(), start, end);

                // Memory safety guarantee for safe functions
                annotations.push((loc.clone(), self.memory_safety_annotation()));

                // Thread safety analysis
                if let Some(thread_safety) = self.analyze_thread_safety(item_fn) {
                    annotations.push((loc.clone(), thread_safety));
                }

                // Termination analysis for const fn
                if item_fn.sig.constness.is_some() {
                    annotations.push((loc, self.const_fn_termination()));
                }
            }
            Item::Impl(item_impl) if !self.contains_unsafe_impl(item_impl) => {
                // Analyze impl blocks for trait safety guarantees
                if let Some((_, trait_path, _)) = &item_impl.trait_ {
                    if self.is_auto_trait(trait_path) {
                        let start = 0u32;
                        let end = 100u32;
                        let loc = Location::new(file_path.to_owned(), start, end);
                        annotations.push((loc, self.auto_trait_annotation(trait_path)));
                    }
                }
            }
            _ => {}
        }

        annotations
    }

    /// Analyze a Rust file without syn (fallback)
    #[cfg(not(feature = "rust-ast"))]
    fn analyze_rust_file_simple(
        &self,
        file_path: &Path,
    ) -> Result<Vec<(Location, ProofAnnotation)>, ProofCollectionError> {
        let content = std::fs::read_to_string(file_path).map_err(ProofCollectionError::Io)?;

        let mut annotations = Vec::new();

        // Simple text-based analysis
        if !self.contains_unsafe(&content) {
            // If no unsafe code found, assume memory safety
            let loc = Location::new(file_path.to_owned(), 0, content.len() as u32);
            annotations.push((loc, self.memory_safety_annotation()));
        }

        Ok(annotations)
    }

    /// Process all Rust files in the project directory
    async fn process_rust_files(
        project_root: &Path,
        cache: &Arc<RwLock<ProofCache>>,
        rustc_version: &str,
        collection_state: &mut CollectionState,
    ) {
        for entry in WalkDir::new(project_root)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if Self::is_rust_file(path) {
                Self::process_single_rust_file(path, cache, rustc_version, collection_state).await;
            }
        }
    }

    /// Check if a path represents a Rust source file
    fn is_rust_file(path: &Path) -> bool {
        path.extension().and_then(|s| s.to_str()) == Some("rs")
    }

    /// Process a single Rust file with caching
    async fn process_single_rust_file(
        path: &Path,
        cache: &Arc<RwLock<ProofCache>>,
        rustc_version: &str,
        collection_state: &mut CollectionState,
    ) {
        let cache_key = format!(
            "rust_borrow_checker:{}:{}",
            rustc_version,
            path.to_string_lossy()
        );

        // Try to get cached results first
        if Self::try_get_cached_results(path, &cache_key, cache, collection_state) {
            return;
        }

        // Analyze the file if not cached
        Self::analyze_and_cache_file(path, &cache_key, cache, collection_state).await;
    }

    /// Try to retrieve cached analysis results
    fn try_get_cached_results(
        path: &Path,
        cache_key: &str,
        cache: &Arc<RwLock<ProofCache>>,
        collection_state: &mut CollectionState,
    ) -> bool {
        let cache_guard = cache.read();
        if cache_guard.is_file_cached(path) {
            if let Some(cached_annotations) = cache_guard.get(cache_key) {
                debug!("Using cached analysis for {:?}", path);
                for annotation in cached_annotations {
                    let loc = Location::new(path.to_owned(), 0, 100);
                    collection_state.annotations.push((loc, annotation.clone()));
                }
                collection_state.files_processed += 1;
                return true;
            }
        }
        false
    }

    /// Analyze file and cache the results
    async fn analyze_and_cache_file(
        path: &Path,
        cache_key: &str,
        cache: &Arc<RwLock<ProofCache>>,
        collection_state: &mut CollectionState,
    ) {
        #[cfg(feature = "rust-ast")]
        let file_result = RustBorrowChecker::default().analyze_rust_file(path);

        #[cfg(not(feature = "rust-ast"))]
        let file_result = RustBorrowChecker::default().analyze_rust_file_simple(path);

        match file_result {
            Ok(file_annotations) => {
                debug!(
                    "Analyzed {:?}: {} annotations",
                    path,
                    file_annotations.len()
                );
                Self::cache_analysis_results(cache_key, &file_annotations, path, cache);
                collection_state.annotations.extend(file_annotations);
                collection_state.files_processed += 1;
            }
            Err(e) => {
                warn!("Failed to analyze {:?}: {}", path, e);
                collection_state.errors.push(e);
            }
        }
    }

    /// Cache analysis results for future use
    fn cache_analysis_results(
        cache_key: &str,
        file_annotations: &[(Location, ProofAnnotation)],
        path: &Path,
        cache: &Arc<RwLock<ProofCache>>,
    ) {
        let cache_annotations: Vec<ProofAnnotation> = file_annotations
            .iter()
            .map(|(_, annotation)| annotation.clone())
            .collect();

        let mut cache_guard = cache.write();
        cache_guard.insert(cache_key.to_string(), cache_annotations);
        cache_guard.update_file_time(path.to_owned());
    }

    /// Finalize collection and build result
    fn finalize_collection(
        start: std::time::Instant,
        collection_state: CollectionState,
    ) -> Result<ProofCollectionResult, ProofCollectionError> {
        let duration = start.elapsed();
        let annotations_count = collection_state.annotations.len();

        info!(
            "Rust borrow checker analysis completed: {} files, {} annotations, {}ms",
            collection_state.files_processed,
            annotations_count,
            duration.as_millis()
        );

        Ok(ProofCollectionResult {
            annotations: collection_state.annotations,
            errors: collection_state.errors,
            metrics: CollectionMetrics {
                files_processed: collection_state.files_processed,
                annotations_found: annotations_count,
                cache_hits: 0, // TODO: Track cache hits properly
                duration_ms: duration.as_millis() as u64,
            },
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

impl ProofSource for RustBorrowChecker {
    fn clone_box(&self) -> Box<dyn ProofSource> {
        Box::new(self.clone())
    }

    fn collect(
        &self,
        project_root: &Path,
        cache: &Arc<RwLock<ProofCache>>,
        _symbol_table: &Arc<SymbolTable>,
    ) -> Pin<
        Box<dyn Future<Output = Result<ProofCollectionResult, ProofCollectionError>> + Send + '_>,
    > {
        let project_root = project_root.to_owned();
        let cache = cache.clone();
        let rustc_version = self.rustc_version.clone();

        Box::pin(async move {
            let start = std::time::Instant::now();
            info!(
                "Starting Rust borrow checker analysis for {:?}",
                project_root
            );

            let mut collection_state = CollectionState::new();

            // Process all Rust files in the project
            Self::process_rust_files(&project_root, &cache, &rustc_version, &mut collection_state)
                .await;

            Self::finalize_collection(start, collection_state)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_rust_borrow_checker_creation() {
        let checker = RustBorrowChecker::new();
        assert!(checker.is_ok());
    }

    #[tokio::test]
    async fn test_rust_borrow_checker_collect() {
        let temp_dir = TempDir::new().unwrap();
        let rust_file = temp_dir.path().join("test.rs");

        // Create a simple Rust file
        std::fs::write(
            &rust_file,
            r#"
            fn safe_function() {
                let x = 42;
                println!("{}", x);
            }
        "#,
        )
        .unwrap();

        let checker = RustBorrowChecker::default();
        let cache = Arc::new(RwLock::new(ProofCache::new()));
        let symbol_table = Arc::new(SymbolTable::new());

        let result = checker
            .collect(temp_dir.path(), &cache, &symbol_table)
            .await;
        assert!(result.is_ok());

        let collection_result = result.unwrap();
        assert_eq!(collection_result.metrics.files_processed, 1);
        assert!(!collection_result.annotations.is_empty());
    }

    #[test]
    fn test_memory_safety_annotation() {
        let checker = RustBorrowChecker::default();
        let annotation = checker.memory_safety_annotation();

        assert_eq!(annotation.property_proven, PropertyType::MemorySafety);
        assert_eq!(annotation.method, VerificationMethod::BorrowChecker);
        assert_eq!(annotation.confidence_level, ConfidenceLevel::High);
        assert_eq!(annotation.tool_name, "rustc-stable");
    }

    #[test]
    fn test_thread_safety_annotation() {
        let checker = RustBorrowChecker::default();
        let annotation = checker.create_thread_safety_annotation();

        assert_eq!(annotation.property_proven, PropertyType::ThreadSafety);
        assert_eq!(annotation.method, VerificationMethod::BorrowChecker);
        assert_eq!(annotation.confidence_level, ConfidenceLevel::High);
    }
}
