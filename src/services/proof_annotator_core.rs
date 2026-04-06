// ProofAnnotator implementation methods
// Included from proof_annotator.rs - do NOT add `use` imports or `#!` attributes here.

impl ProofAnnotator {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(symbol_table: Arc<SymbolTable>) -> Self {
        Self {
            sources: Vec::new(),
            cache: Arc::new(RwLock::new(ProofCache::new())),
            symbol_table,
        }
    }

    /// Add a proof source to the annotator
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn add_source<T: ProofSource + 'static>(&mut self, source: T) {
        self.sources.push(Box::new(source));
    }

    /// Collect proof annotations from all sources in parallel
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn collect_proofs(&self, project_root: &Path) -> ProofMap {
        debug_assert!(project_root.exists(), "project_root must exist: {}", project_root.display());
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
        let total_annotations = proof_map.values().map(std::vec::Vec::len).sum::<usize>();

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
        debug_assert!(!results.is_empty(), "results must not be empty");
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

        proof_map
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
