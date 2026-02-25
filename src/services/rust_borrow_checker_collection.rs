// Proof collection and caching methods for RustBorrowChecker
// Included by rust_borrow_checker.rs - no `use` imports or `#!` attributes

impl RustBorrowChecker {
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
            .filter_map(std::result::Result::ok)
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
                cache_hits: 0, // TRACKED: Track cache hits properly
                duration_ms: duration.as_millis() as u64,
            },
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
