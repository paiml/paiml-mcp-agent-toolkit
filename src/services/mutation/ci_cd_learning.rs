#![cfg_attr(coverage_nightly, coverage(off))]
//! CI/CD integration for real-time ML model learning
//!
//! Provides continuous learning from mutation test results in CI/CD pipelines,
//! enabling the ML predictor to improve accuracy over time based on actual
//! test suite behavior.

use super::ml_predictor::{SurvivabilityPredictor, TrainingData};
use super::types::*;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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

include!("ci_cd_learning_methods.rs");
include!("ci_cd_learning_tests.rs");
