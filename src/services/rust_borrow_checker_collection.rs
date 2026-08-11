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
        // Gitignore-aware: this walk used to be a bare `WalkDir` with
        // `follow_links(true)`, so it descended into gitignored trees. On this
        // repo the ephemeral `.claude/worktrees/` checkouts — full copies of the
        // very files being scanned — supplied 90% of every proof-annotation
        // total (177305 of 195846). `follow_links` stays off for the same
        // reason: a symlinked copy is still a copy.
        let ignore_matcher = Self::build_ignore_matcher(project_root);
        for entry in WalkDir::new(project_root)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| {
                !Self::is_ignored(ignore_matcher.as_ref(), e.path(), e.file_type().is_dir())
            })
            .filter_map(std::result::Result::ok)
        {
            let path = entry.path();
            if Self::is_rust_file(path) {
                Self::process_single_rust_file(path, cache, rustc_version, collection_state).await;
            }
        }
    }

    /// Build a `.gitignore` matcher rooted at the project being analysed.
    ///
    /// Returns `None` when the project has no readable `.gitignore`; in that
    /// case nothing is filtered and the walk behaves as it always did.
    fn build_ignore_matcher(project_root: &Path) -> Option<ignore::gitignore::Gitignore> {
        let mut builder = ignore::gitignore::GitignoreBuilder::new(project_root);
        // add() returns Some(err) on failure — a missing .gitignore is normal.
        if builder.add(project_root.join(".gitignore")).is_some() {
            return None;
        }
        builder.build().ok()
    }

    /// True when `path` is excluded from the analysis: the git metadata
    /// directory, or anything the project's `.gitignore` excludes.
    fn is_ignored(
        matcher: Option<&ignore::gitignore::Gitignore>,
        path: &Path,
        is_dir: bool,
    ) -> bool {
        if is_dir && path.file_name().and_then(|n| n.to_str()) == Some(".git") {
            return true;
        }
        matcher.is_some_and(|m| m.matched_path_or_any_parents(path, is_dir).is_ignore())
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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod gitignore_walk_tests {
    use super::*;

    /// The proof walk used to be a bare `WalkDir`, so it descended into
    /// gitignored trees. On this repo the ephemeral `.claude/worktrees/`
    /// checkouts — copies of the files being analysed — supplied 177305 of
    /// 195846 reported proof annotations (90.5%).
    #[test]
    fn gitignored_trees_are_excluded_from_the_walk() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::write(root.join(".gitignore"), ".claude/worktrees\ntarget\n").unwrap();
        std::fs::create_dir_all(root.join(".claude/worktrees/copy")).unwrap();
        std::fs::create_dir_all(root.join("target/debug")).unwrap();
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("src/lib.rs"), "fn a() {}").unwrap();
        std::fs::write(root.join(".claude/worktrees/copy/lib.rs"), "fn a() {}").unwrap();

        let matcher = RustBorrowChecker::build_ignore_matcher(root);
        assert!(matcher.is_some(), "the fixture has a .gitignore");

        assert!(RustBorrowChecker::is_ignored(
            matcher.as_ref(),
            &root.join(".claude/worktrees"),
            true
        ));
        assert!(RustBorrowChecker::is_ignored(
            matcher.as_ref(),
            &root.join(".claude/worktrees/copy/lib.rs"),
            false
        ));
        assert!(RustBorrowChecker::is_ignored(
            matcher.as_ref(),
            &root.join("target/debug"),
            true
        ));
        assert!(RustBorrowChecker::is_ignored(
            matcher.as_ref(),
            &root.join(".git"),
            true
        ));
        assert!(!RustBorrowChecker::is_ignored(
            matcher.as_ref(),
            &root.join("src/lib.rs"),
            false
        ));
    }
}
