//! CI/CD integration for real-time ML model learning
//!
//! Provides continuous learning from mutation test results in CI/CD pipelines,
//! enabling the ML predictor to improve accuracy over time based on actual
//! test suite behavior.

use super::ml_predictor::{SurvivabilityPredictor, TrainingData};
use super::types::*;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use chrono::{DateTime, Utc};

/// CI/CD learning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CiCdLearningConfig {
    /// Directory for storing training data
    pub data_dir: PathBuf,

    /// Directory for storing model versions
    pub model_dir: PathBuf,

    /// Minimum samples before retraining
    pub min_samples_for_training: usize,

    /// Maximum training data to keep
    pub max_training_samples: usize,

    /// Auto-train on data collection
    pub auto_train: bool,

    /// Model versioning enabled
    pub versioning_enabled: bool,
}

impl Default for CiCdLearningConfig {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from(".pmat/training_data"),
            model_dir: PathBuf::from(".pmat/models"),
            min_samples_for_training: 50,
            max_training_samples: 10000,
            auto_train: true,
            versioning_enabled: true,
        }
    }
}

/// Training data batch from CI/CD run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingBatch {
    /// Batch ID (timestamp-based)
    pub id: String,

    /// CI/CD run metadata
    pub metadata: CiCdMetadata,

    /// Training samples from this run
    pub samples: Vec<TrainingData>,

    /// Timestamp of collection
    pub collected_at: DateTime<Utc>,
}

/// CI/CD run metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CiCdMetadata {
    /// CI/CD system (github, gitlab, jenkins, etc.)
    pub system: String,

    /// Repository name
    pub repository: String,

    /// Branch name
    pub branch: String,

    /// Commit hash
    pub commit: String,

    /// Build/run ID
    pub build_id: String,
}

/// Model version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelVersion {
    /// Version number (incremental)
    pub version: u32,

    /// Timestamp of training
    pub trained_at: DateTime<Utc>,

    /// Number of training samples used
    pub sample_count: usize,

    /// Model accuracy (from cross-validation)
    pub accuracy: f64,

    /// Model file path
    pub file_path: PathBuf,

    /// Training metadata
    pub metadata: Option<CiCdMetadata>,
}

/// CI/CD learning manager
pub struct CiCdLearningManager {
    config: CiCdLearningConfig,
    predictor: SurvivabilityPredictor,
    current_version: Option<ModelVersion>,
}

impl CiCdLearningManager {
    /// Create new CI/CD learning manager
    pub fn new(config: CiCdLearningConfig) -> Self {
        Self {
            config,
            predictor: SurvivabilityPredictor::new(),
            current_version: None,
        }
    }

    /// Collect training data from mutation results
    pub async fn collect_training_data(
        &mut self,
        results: &[MutationResult],
        metadata: CiCdMetadata,
    ) -> Result<TrainingBatch> {
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
    pub async fn train_incremental(&mut self, training_data: &[TrainingData]) -> Result<ModelVersion> {
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
        let accuracy = self
            .predictor
            .cross_validate(samples, 5)
            .unwrap_or(0.0);

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
    pub fn predictor(&self) -> &SurvivabilityPredictor {
        &self.predictor
    }

    /// Get current model version
    pub fn current_version(&self) -> Option<&ModelVersion> {
        self.current_version.as_ref()
    }

    /// Save training batch to disk
    async fn save_training_batch(&self, batch: &TrainingBatch) -> Result<()> {
        tokio::fs::create_dir_all(&self.config.data_dir).await?;

        let file_path = self.config.data_dir.join(format!("batch_{}.json", batch.id));
        let json = serde_json::to_string_pretty(batch)?;

        tokio::fs::write(file_path, json)
            .await
            .context("Failed to save training batch")
    }

    /// Load all training data from disk
    async fn load_all_training_data(&self) -> Result<Vec<TrainingData>> {
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
        self.current_version
            .as_ref()
            .map(|v| v.version + 1)
            .unwrap_or(1)
    }

    /// Get model file path for version
    fn get_model_path(&self, version: u32) -> PathBuf {
        self.config.model_dir.join(format!("model_v{}.bin", version))
    }

    /// Clean old training data (keep only recent batches)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_config() -> CiCdLearningConfig {
        CiCdLearningConfig {
            data_dir: PathBuf::from("/tmp/pmat_test/training_data"),
            model_dir: PathBuf::from("/tmp/pmat_test/models"),
            min_samples_for_training: 5,
            max_training_samples: 100,
            auto_train: false, // Disable for tests
            versioning_enabled: true,
        }
    }

    fn create_test_metadata() -> CiCdMetadata {
        CiCdMetadata {
            system: "github".to_string(),
            repository: "test/repo".to_string(),
            branch: "main".to_string(),
            commit: "abc123".to_string(),
            build_id: "12345".to_string(),
        }
    }

    fn create_test_mutant() -> Mutant {
        Mutant {
            id: "test_1".to_string(),
            original_file: PathBuf::from("test.rs"),
            mutated_source: "fn test() { }".to_string(),
            location: SourceLocation {
                line: 1,
                column: 1,
                end_line: 1,
                end_column: 10,
            },
            operator: MutationOperatorType::ArithmeticReplacement,
            hash: "test_hash".to_string(),
            status: MutantStatus::Pending,
        }
    }

    #[test]
    fn test_ci_cd_config_default() {
        let config = CiCdLearningConfig::default();
        assert_eq!(config.min_samples_for_training, 50);
        assert_eq!(config.max_training_samples, 10000);
        assert!(config.auto_train);
        assert!(config.versioning_enabled);
    }

    #[test]
    fn test_ci_cd_learning_manager_creation() {
        let config = create_test_config();
        let manager = CiCdLearningManager::new(config);
        assert!(manager.current_version().is_none());
    }

    #[test]
    fn test_training_batch_creation() {
        let metadata = create_test_metadata();
        let mutant = create_test_mutant();

        let results = vec![MutationResult {
            mutant,
            status: MutantStatus::Killed,
            test_failures: vec!["test1".to_string()],
            execution_time_ms: 100,
            error_message: None,
        }];

        let samples: Vec<TrainingData> = results
            .iter()
            .map(|r| TrainingData {
                mutant: r.mutant.clone(),
                was_killed: r.status == MutantStatus::Killed,
                test_failures: r.test_failures.clone(),
                execution_time_ms: r.execution_time_ms,
            })
            .collect();

        assert_eq!(samples.len(), 1);
        assert!(samples[0].was_killed);
    }

    #[test]
    fn test_model_version_increment() {
        let config = create_test_config();
        let mut manager = CiCdLearningManager::new(config);

        assert_eq!(manager.get_next_version(), 1);

        manager.current_version = Some(ModelVersion {
            version: 5,
            trained_at: Utc::now(),
            sample_count: 100,
            accuracy: 0.85,
            file_path: PathBuf::from("/tmp/model_v5.bin"),
            metadata: None,
        });

        assert_eq!(manager.get_next_version(), 6);
    }

    #[actix_rt::test]
    async fn test_collect_training_data() {
        let config = create_test_config();
        let mut manager = CiCdLearningManager::new(config);
        let metadata = create_test_metadata();
        let mutant = create_test_mutant();

        let results = vec![
            MutationResult {
                mutant: mutant.clone(),
                status: MutantStatus::Killed,
                test_failures: vec!["test1".to_string()],
                execution_time_ms: 100,
                error_message: None,
            },
            MutationResult {
                mutant,
                status: MutantStatus::Survived,
                test_failures: vec![],
                execution_time_ms: 50,
                error_message: None,
            },
        ];

        let batch = manager
            .collect_training_data(&results, metadata)
            .await
            .unwrap();

        assert_eq!(batch.samples.len(), 2);
        assert!(batch.samples[0].was_killed);
        assert!(!batch.samples[1].was_killed);
    }
}
