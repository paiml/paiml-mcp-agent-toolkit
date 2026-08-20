// ProofAnnotator implementation methods
// Included from proof_annotator.rs - do NOT add `use` imports or `#!` attributes here.

impl ProofAnnotator {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new(symbol_table: Arc<SymbolTable>) -> Self {
        Self {
            sources: Vec::new(),
            cache: Arc::new(RwLock::new(ProofCache::new())),
            symbol_table,
            collection_errors: std::sync::atomic::AtomicUsize::new(0),
            files_processed: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    /// How many files the last [`collect_proofs`](Self::collect_proofs) failed
    /// to read or parse, and therefore did not contribute annotations.
    ///
    /// Zero before any collection has run.
    #[must_use]
    pub fn collection_errors(&self) -> usize {
        self.collection_errors
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// How many files the last [`collect_proofs`](Self::collect_proofs) read.
    ///
    /// The denominator of the annotation total: zero means nothing was scanned,
    /// which is not the same fact as "scanned and found none".
    ///
    /// Zero before any collection has run.
    #[must_use]
    pub fn files_processed(&self) -> usize {
        self.files_processed
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Add a proof source to the annotator
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn add_source<T: ProofSource + 'static>(&mut self, source: T) {
        self.sources.push(Box::new(source));
    }

    /// Collect proof annotations from all sources in parallel
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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

        // How many files were actually read, taken before the merge consumes
        // the per-source metrics. `max` rather than `sum`: every source walks
        // the same tree, so summing would report more files than exist; what
        // this has to answer is "was anything opened at all", and the largest
        // single source's count is the honest floor for that.
        let files_read = all_results
            .iter()
            .map(|r| r.metrics.files_processed)
            .max()
            .unwrap_or(0);

        // Merge results with conflict resolution
        let (proof_map, total_errors) = self.merge_with_conflict_resolution(all_results);

        self.files_processed
            .store(files_read, std::sync::atomic::Ordering::Relaxed);

        // The per-file failures used to end here, as a single `warn!` line on
        // stderr and nothing else: the stdout report claimed a total with no
        // hint that N files had been skipped. Retained on the annotator so the
        // renderers can disclose it alongside the total.
        self.collection_errors
            .store(total_errors, std::sync::atomic::Ordering::Relaxed);

        let elapsed = start.elapsed();
        let total_annotations = proof_map.values().map(std::vec::Vec::len).sum::<usize>();

        info!(
            "Proof collection completed in {}ms: {} annotations from {} sources",
            elapsed.as_millis(),
            total_annotations,
            self.sources.len()
        );

        proof_map
    }

    /// Merge results from multiple sources with conflict resolution.
    ///
    /// Returns the merged map and the number of per-file errors the sources
    /// reported, which the caller records on the annotator.
    fn merge_with_conflict_resolution(
        &self,
        results: Vec<ProofCollectionResult>,
    ) -> (ProofMap, usize) {
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
                                u32::from(existing[idx].assumptions.is_empty()),
                            );

                            let new_score = (
                                annotation.confidence_level as u32,
                                method_rank(&annotation.method),
                                u32::from(annotation.assumptions.is_empty()),
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

        (proof_map, total_errors)
    }

    /// Get cache statistics
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn cache_stats(&self) -> CacheStats {
        let cache = self.cache.read();
        CacheStats {
            size: cache.size(),
            files_tracked: cache.file_times.len(),
        }
    }

    /// Clear the cache
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn clear_cache(&self) {
        self.cache.write().clear();
    }
}

impl std::fmt::Debug for ProofAnnotator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProofAnnotator")
            .field("sources_count", &self.sources.len())
            .field("cache_stats", &self.cache_stats())
            .finish()
    }
}
