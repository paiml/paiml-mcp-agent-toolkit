impl TdgAnalyzerAst {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn analyze_file(&self, path: &Path) -> Result<TdgScore> {
        self.analyze_file_with_priority(path, OperationPriority::Medium)
            .await
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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
            if storage.get_hot(content_hash).is_some() {
                // Record performance sample for cache hit
                if let Some(adaptive) = &self.adaptive_manager {
                    let duration = start_time.elapsed().unwrap_or_default();
                    let sample = adaptive.create_sample(duration, true, 0).await;
                    adaptive.record_sample(sample).await?;
                }

                // A hot-cache entry carries only `total_score` and a grade byte, so
                // the score used to be rebuilt as `TdgScore { total, ..Default::default() }`
                // — and `TdgScore::default()` seeds every component with its category
                // MAXIMUM (25+20+20+15+10+10 = 100). The `calculate_total()` call that
                // followed re-derived `total` from those maxima, so every content-hash
                // cache HIT was rewritten to 100.0 / A+, discarding the measured score.
                // `pmat tdg compare <p> <p>` showed it plainly: source 1 88.0 (A-),
                // source 2 (the cache hit) 100.0 (A+), "Winner: source2".
                //
                // Rebuild from the persisted full record, which carries the real
                // component breakdown. If that record cannot be read, treat it as a
                // miss and re-analyze rather than inventing a breakdown we never
                // measured.
                if let Some(record) = storage.retrieve_full(content_hash).await.ok().flatten() {
                    let mut cached_score = record.score;
                    cached_score.language = language;
                    cached_score.confidence = language.confidence();
                    cached_score.file_path = Some(path.to_path_buf());
                    return Ok(Some(cached_score));
                }
                return Ok(None);
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn analyze_file_commit(&self, path: &Path) -> Result<TdgScore> {
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn analyze_file_background(&self, path: &Path) -> Result<TdgScore> {
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

/// Regression tests for the hot-cache score reconstruction.
///
/// A cache HIT used to be rebuilt from `TdgScore::default()` (whose components
/// are the category maxima) and then re-totalled, so the second analysis of the
/// same content always came back as 100.0 / A+ regardless of what the first
/// analysis measured.
#[cfg(test)]
mod hot_cache_score_regression_tests {
    use super::*;

    /// A file with enough branching and no documentation that it cannot score a
    /// perfect 100 — the test is only meaningful when the measured score differs
    /// from the maxed-out default.
    const IMPERFECT_SOURCE: &str = r#"
pub fn classify(a: i32, b: i32, c: i32) -> i32 {
    let mut acc = 0;
    for i in 0..a {
        if i % 2 == 0 && b > 0 {
            acc += i * b;
        } else if i % 3 == 0 || c < 0 {
            acc -= i;
        } else {
            match i % 5 {
                0 => acc += 1,
                1 => acc -= 1,
                2 => acc *= 2,
                _ => acc = acc.saturating_add(c),
            }
        }
    }
    while acc > 1000 {
        acc /= 2;
    }
    acc
}

pub fn classify_again(a: i32, b: i32, c: i32) -> i32 {
    let mut acc = 0;
    for i in 0..a {
        if i % 2 == 0 && b > 0 {
            acc += i * b;
        } else if i % 3 == 0 || c < 0 {
            acc -= i;
        } else {
            match i % 5 {
                0 => acc += 1,
                1 => acc -= 1,
                2 => acc *= 2,
                _ => acc = acc.saturating_add(c),
            }
        }
    }
    acc
}
"#;

    #[tokio::test]
    async fn cache_hit_returns_the_measured_score_not_a_perfect_100() {
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("hot.rs");
        fs::write(&file, IMPERFECT_SOURCE).expect("write fixture");

        let analyzer = TdgAnalyzerAst::with_in_memory_storage(TdgConfig::default());

        let first = analyzer.analyze_file(&file).await.expect("first analysis");
        assert!(
            first.total < 99.9,
            "fixture must not score a perfect 100 or the regression is untestable (got {})",
            first.total
        );

        // Same content hash => hot-cache hit.
        let second = analyzer
            .analyze_file(&file)
            .await
            .expect("second analysis (cache hit)");

        assert!(
            (second.total - first.total).abs() < 0.01,
            "cache hit returned {} but the file measures {}",
            second.total,
            first.total
        );
        assert_eq!(
            second.grade, first.grade,
            "cache hit regraded the file from {:?} to {:?}",
            first.grade, second.grade
        );
    }
}
