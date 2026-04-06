impl CiCdLearningManager {
    /// Create new CI/CD learning manager
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(config: CiCdLearningConfig) -> Self {
        Self {
            config,
            predictor: SurvivabilityPredictor::new(),
            current_version: None,
        }
    }

    /// Collect training data from mutation results
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn collect_training_data(
        &mut self,
        results: &[MutationResult],
        metadata: CiCdMetadata,
    ) -> Result<TrainingBatch> {
        debug_assert!(!results.is_empty(), "results must not be empty");
        // Convert results to training data
        let samples: Vec<TrainingData> = results
            .iter()
            .filter(|r| matches!(r.status, MutantStatus::Killed | MutantStatus::Survived))
            .map(|r| TrainingData {
                mutant: r.mutant.clone(),
                was_killed: r.status == MutantStatus::Killed,
                test_failures: r.test_failures.clone(),
                execution_time_ms: r.execution_time_ms,
            })
            .collect();

        let batch = TrainingBatch {
            id: format!("{}", Utc::now().timestamp()),
            metadata,
            samples,
            collected_at: Utc::now(),
        };

        // Persist batch
        self.save_training_batch(&batch).await?;

        // Auto-train if configured
        if self.config.auto_train {
            let all_samples = self.load_all_training_data().await?;
            if all_samples.len() >= self.config.min_samples_for_training {
                self.train_incremental(&all_samples).await?;
            }
        }

        Ok(batch)
    }

    /// Train model incrementally with new data
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn train_incremental(
        &mut self,
        training_data: &[TrainingData],
    ) -> Result<ModelVersion> {
        debug_assert!(!training_data.is_empty(), "training_data must not be empty");
        // Limit to max_training_samples (keep most recent)
        let samples = if training_data.len() > self.config.max_training_samples {
            &training_data[training_data.len() - self.config.max_training_samples..]
        } else {
            training_data
        };

        // Train predictor
        self.predictor
            .train(samples)
            .context("Failed to train predictor")?;

        // Validate with cross-validation
        let accuracy = self.predictor.cross_validate(samples, 5).unwrap_or(0.0);

        // Create model version
        let version = self.get_next_version();
        let model_version = ModelVersion {
            version,
            trained_at: Utc::now(),
            sample_count: samples.len(),
            accuracy,
            file_path: self.get_model_path(version),
            metadata: None,
        };

        // Save model if versioning enabled
        if self.config.versioning_enabled {
            self.save_model_version(&model_version).await?;
        }

        self.current_version = Some(model_version.clone());

        Ok(model_version)
    }

    /// Load latest model version
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn load_latest_model(&mut self) -> Result<Option<ModelVersion>> {
        let versions = self.list_model_versions().await?;

        if let Some(latest) = versions.last() {
            self.predictor = SurvivabilityPredictor::load(&latest.file_path)?;
            self.current_version = Some(latest.clone());
            Ok(Some(latest.clone()))
        } else {
            Ok(None)
        }
    }

    /// Get predictor reference
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn predictor(&self) -> &SurvivabilityPredictor {
        &self.predictor
    }

    /// Get current model version
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn current_version(&self) -> Option<&ModelVersion> {
        self.current_version.as_ref()
    }

    /// Save training batch to disk
    async fn save_training_batch(&self, batch: &TrainingBatch) -> Result<()> {
        debug_assert!(true, "contract: save_training_batch");
        tokio::fs::create_dir_all(&self.config.data_dir).await?;

        let file_path = self
            .config
            .data_dir
            .join(format!("batch_{}.json", batch.id));
        let json = serde_json::to_string_pretty(batch)?;

        tokio::fs::write(file_path, json)
            .await
            .context("Failed to save training batch")
    }

    /// Load all training data from disk
    async fn load_all_training_data(&self) -> Result<Vec<TrainingData>> {
        debug_assert!(true, "contract: load_all_training_data");
        let mut all_samples = Vec::new();

        if !self.config.data_dir.exists() {
            return Ok(all_samples);
        }

        let mut entries = tokio::fs::read_dir(&self.config.data_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = tokio::fs::read_to_string(&path).await?;
                if let Ok(batch) = serde_json::from_str::<TrainingBatch>(&content) {
                    all_samples.extend(batch.samples);
                }
            }
        }

        Ok(all_samples)
    }

    /// Save model version to disk
    async fn save_model_version(&self, version: &ModelVersion) -> Result<()> {
        debug_assert!(true, "contract: save_model_version");
        tokio::fs::create_dir_all(&self.config.model_dir).await?;

        // Save predictor
        self.predictor.save(&version.file_path)?;

        // Save version metadata
        let metadata_path = self
            .config
            .model_dir
            .join(format!("version_{}.json", version.version));
        let json = serde_json::to_string_pretty(version)?;
        tokio::fs::write(metadata_path, json).await?;

        Ok(())
    }

    /// List all model versions
    async fn list_model_versions(&self) -> Result<Vec<ModelVersion>> {
        debug_assert!(true, "contract: list_model_versions");
        let mut versions = Vec::new();

        if !self.config.model_dir.exists() {
            return Ok(versions);
        }

        let mut entries = tokio::fs::read_dir(&self.config.model_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with("version_") && n.ends_with(".json"))
                .unwrap_or(false)
            {
                let content = tokio::fs::read_to_string(&path).await?;
                if let Ok(version) = serde_json::from_str::<ModelVersion>(&content) {
                    versions.push(version);
                }
            }
        }

        versions.sort_by_key(|v| v.version);

        Ok(versions)
    }

    /// Get next version number
    fn get_next_version(&self) -> u32 {
        debug_assert!(true, "contract: get_next_version");
        self.current_version
            .as_ref()
            .map(|v| v.version + 1)
            .unwrap_or(1)
    }

    /// Get model file path for version
    fn get_model_path(&self, version: u32) -> PathBuf {
        self.config
            .model_dir
            .join(format!("model_v{}.bin", version))
    }

    /// Clean old training data (keep only recent batches)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn cleanup_old_data(&self, keep_batches: usize) -> Result<usize> {
        if !self.config.data_dir.exists() {
            return Ok(0);
        }

        let mut batches = Vec::new();
        let mut entries = tokio::fs::read_dir(&self.config.data_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                batches.push(path);
            }
        }

        batches.sort();

        if batches.len() <= keep_batches {
            return Ok(0);
        }

        let to_remove = &batches[..batches.len() - keep_batches];
        let mut removed = 0;

        for path in to_remove {
            if tokio::fs::remove_file(path).await.is_ok() {
                removed += 1;
            }
        }

        Ok(removed)
    }
}
