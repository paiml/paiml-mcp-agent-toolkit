impl TdgAnalyzerAst {
    pub async fn analyze_file(&self, path: &Path) -> Result<TdgScore> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        self.analyze_file_with_priority(path, OperationPriority::Medium)
            .await
    }

    pub async fn analyze_file_with_priority(
        &self,
        path: &Path,
        priority: OperationPriority,
    ) -> Result<TdgScore> {
        let start_time = SystemTime::now();
        let language = Language::from_extension(path);

        // Toyota Way Extract Method: Resource allocation
        let _resource_allocation = self.request_analysis_resources(path, priority).await?;

        let source = fs::read_to_string(path)?;
        let content_hash = blake3::hash(source.as_bytes());

        // Toyota Way Extract Method: Cache check and return if hit
        if let Some(cached_score) = self
            .check_cache_and_return(&content_hash, language, path, start_time)
            .await?
        {
            return Ok(cached_score);
        }

        // Toyota Way Extract Method: Fresh analysis and storage
        let score = self
            .perform_analysis_and_store(path, &source, language, content_hash, start_time)
            .await?;

        Ok(score)
    }

    /// Toyota Way Extract Method: Request analysis resources if controller available
    async fn request_analysis_resources(
        &self,
        path: &Path,
        priority: OperationPriority,
    ) -> Result<Option<crate::tdg::resource_control::ResourceAllocation>> {
        if let Some(controller) = &self.resource_controller {
            let estimated_memory = self.estimate_analysis_memory(path)?;
            Ok(Some(
                controller
                    .request_resources(
                        format!("analyze_{}", path.display()),
                        crate::tdg::resource_control::OperationType::Analysis,
                        priority,
                        estimated_memory,
                    )
                    .await?,
            ))
        } else {
            Ok(None)
        }
    }

    /// Toyota Way Extract Method: Check cache and return score if hit
    async fn check_cache_and_return(
        &self,
        content_hash: &blake3::Hash,
        language: Language,
        path: &Path,
        start_time: SystemTime,
    ) -> Result<Option<TdgScore>> {
        if let Some(storage) = &self.storage {
            if let Some(hot_entry) = storage.get_hot(content_hash) {
                // Record performance sample for cache hit
                if let Some(adaptive) = &self.adaptive_manager {
                    let duration = start_time.elapsed().unwrap_or_default();
                    let sample = adaptive.create_sample(duration, true, 0).await;
                    adaptive.record_sample(sample).await?;
                }

                // Return cached score with updated timestamp
                let mut cached_score = TdgScore {
                    total: hot_entry.total_score,
                    grade: Grade::from_score(hot_entry.total_score),
                    language,
                    confidence: language.confidence(),
                    file_path: Some(path.to_path_buf()),
                    ..Default::default()
                };
                cached_score.calculate_total();
                return Ok(Some(cached_score));
            }
        }
        Ok(None)
    }

    /// Toyota Way Extract Method: Perform fresh analysis and store results
    async fn perform_analysis_and_store(
        &self,
        path: &Path,
        source: &str,
        language: Language,
        content_hash: blake3::Hash,
        start_time: SystemTime,
    ) -> Result<TdgScore> {
        // Perform fresh analysis
        let analysis_start = SystemTime::now();
        let score = self.analyze_source(source, language, Some(path.to_path_buf()))?;
        let analysis_duration = analysis_start.elapsed().unwrap_or_default();

        // Store in tiered storage if enabled
        self.store_analysis_record(path, &score, content_hash, analysis_duration, language)
            .await?;

        // Record performance sample for fresh analysis
        if let Some(adaptive) = &self.adaptive_manager {
            let total_duration = start_time.elapsed().unwrap_or_default();
            let sample = adaptive.create_sample(total_duration, false, 0).await;
            adaptive.record_sample(sample).await?;
        }

        Ok(score)
    }

    /// Toyota Way Extract Method: Store analysis record in tiered storage
    async fn store_analysis_record(
        &self,
        path: &Path,
        score: &TdgScore,
        content_hash: blake3::Hash,
        analysis_duration: Duration,
        language: Language,
    ) -> Result<()> {
        if let Some(storage) = &self.storage {
            let file_metadata = fs::metadata(path)?;
            let record = FullTdgRecord {
                identity: FileIdentity {
                    path: path.to_path_buf(),
                    content_hash,
                    size_bytes: file_metadata.len(),
                    modified_time: file_metadata.modified().unwrap_or(SystemTime::now()),
                },
                score: score.clone(),
                components: ComponentScores {
                    complexity_breakdown: std::collections::HashMap::new(),
                    duplication_sources: Vec::new(),
                    coupling_dependencies: Vec::new(),
                    doc_missing_items: Vec::new(),
                    consistency_violations: Vec::new(),
                },
                semantic_sig: SemanticSignature {
                    ast_structure_hash: u64::from_le_bytes(
                        content_hash.as_bytes()[0..8]
                            .try_into()
                            .expect("slice with incorrect length"),
                    ),
                    identifier_pattern: String::new(),
                    control_flow_pattern: String::new(),
                    import_dependencies: Vec::new(),
                },
                metadata: AnalysisMetadata {
                    analyzer_version: env!("CARGO_PKG_VERSION").to_string(),
                    analysis_duration_ms: analysis_duration.as_millis() as u64,
                    language_confidence: language.confidence(),
                    analysis_timestamp: SystemTime::now(),
                    cache_hit: false,
                },
                git_context: self.git_context.clone(), // Sprint 65: Git-commit correlation
            };

            storage.store(record).await?;
        }
        Ok(())
    }

    /// Analyze file with commit priority (for git hooks, CI/CD)
    pub async fn analyze_file_commit(&self, path: &Path) -> Result<TdgScore> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let _guard = if let Some(scheduler) = &self.scheduler {
            Some(
                scheduler
                    .schedule_commit(path.to_path_buf())
                    .await
                    .map_err(|e| anyhow::anyhow!("Scheduling failed: {e}"))?,
            )
        } else {
            None
        };

        self.analyze_file_with_priority(path, OperationPriority::Critical)
            .await
    }

    /// Analyze file with background priority (for daemon, IDE plugins)
    pub async fn analyze_file_background(&self, path: &Path) -> Result<TdgScore> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let _guard = if let Some(scheduler) = &self.scheduler {
            Some(
                scheduler
                    .schedule_background(path.to_path_buf())
                    .await
                    .map_err(|e| anyhow::anyhow!("Scheduling failed: {e}"))?,
            )
        } else {
            None
        };

        self.analyze_file_with_priority(path, OperationPriority::Low)
            .await
    }
}
