//! Category F: ML/AI Reproducibility (5 points) - CONDITIONAL
//!
//! Modern science standards for machine learning projects.
//! This category is N/A for non-ML projects.
//!
//! ## Sub-categories
//!
//! | ID | Name | Points | Description |
//! |----|------|--------|-------------|
//! | F1 | Random Seed Fixing | 2 | Deterministic training |
//! | F2 | Model Versioning | 2 | DVC, MLflow, or equivalent |
//! | F3 | Dataset Documentation | 1 | Data provenance |
//!
//! ## N/A Handling
//!
//! If a project is determined to be non-ML:
//! - Returns `is_applicable = false`
//! - Score is excluded from normalization denominator
//! - This prevents "free points" for non-ML projects
//!
//! ## Academic Foundation
//!
//! - NeurIPS ML Reproducibility Checklist [25]
//! - MLCommons Benchmarking [26]
//! - Pineau et al. (2021): ICLR Guidelines [27]

use crate::services::popper_score::models::{PopperCategoryScore, PopperFinding, PopperSubScore};
use crate::services::popper_score::scorer::{PopperScorer, PopperScorerResult};
use std::path::Path;

/// Patterns that indicate a project uses ML/AI
const ML_INDICATORS: &[&str] = &[
    "torch",
    "tensorflow",
    "keras",
    "sklearn",
    "scikit-learn",
    "pytorch",
    "jax",
    "huggingface",
    "transformers",
    "model",
    "neural",
    "training",
    "inference",
    "dataset",
    "ml",
    "machine learning",
    "deep learning",
    "llm",
    "embedding",
];

/// Scorer for Category F: ML/AI Reproducibility (5 points)
///
/// This is a **conditional** category - N/A for non-ML projects.
pub struct MLReproducibilityScorer;

impl MLReproducibilityScorer {
    /// Create a new ML reproducibility scorer
    pub fn new() -> Self {
        Self
    }

    /// Detect if the project is an ML project
    pub fn is_ml_project(&self, project_path: &Path) -> bool {
        // Check for ML-specific files
        let ml_files = [
            "model.py",
            "train.py",
            "training.py",
            "inference.py",
            "model.onnx",
            "model.pt",
            "model.pth",
            "model.h5",
            "model.keras",
            "dvc.yaml",
            "mlflow.yaml",
            "wandb/",
            "checkpoints/",
        ];

        for file in ml_files {
            if project_path.join(file).exists() {
                return true;
            }
        }

        // Check requirements.txt or pyproject.toml for ML dependencies
        let req_files = ["requirements.txt", "pyproject.toml", "setup.py"];
        for req_file in req_files {
            let path = project_path.join(req_file);
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let content_lower = content.to_lowercase();
                    for indicator in ML_INDICATORS {
                        if content_lower.contains(indicator) {
                            return true;
                        }
                    }
                }
            }
        }

        // Check Cargo.toml for Rust ML crates
        let cargo_toml = project_path.join("Cargo.toml");
        if cargo_toml.exists() {
            if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
                let content_lower = content.to_lowercase();
                let rust_ml_crates = [
                    "candle",
                    "burn",
                    "tch",
                    "ort",
                    "tract",
                    "linfa",
                    "smartcore",
                    "ndarray",
                ];
                for crate_name in rust_ml_crates {
                    if content_lower.contains(crate_name) {
                        return true;
                    }
                }
            }
        }

        // Check README for ML mentions
        let readme = project_path.join("README.md");
        if readme.exists() {
            if let Ok(content) = std::fs::read_to_string(&readme) {
                let content_lower = content.to_lowercase();
                for indicator in ML_INDICATORS {
                    if content_lower.contains(indicator) {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// F1: Random Seed Fixing (2 points)
    ///
    /// Checks for:
    /// - Seed setting in training code (1 point)
    /// - Documented seed in README (1 point)
    fn score_random_seed_fixing(&self, project_path: &Path) -> PopperSubScore {
        let mut earned: f64 = 0.0;
        let max: f64 = 2.0;
        let mut description = Vec::new();

        // Check for seed setting patterns in Python files
        if let Ok(content) = self.read_python_files(project_path) {
            let seed_patterns = [
                "random.seed",
                "np.random.seed",
                "torch.manual_seed",
                "tf.random.set_seed",
                "set_seed",
                "SEED =",
                "seed =",
            ];
            for pattern in seed_patterns {
                if content.contains(pattern) {
                    earned += 1.0;
                    description.push("seed setting in code");
                    break;
                }
            }
        }

        // Check README for seed documentation
        let readme = project_path.join("README.md");
        if readme.exists() {
            if let Ok(content) = std::fs::read_to_string(&readme) {
                let content_lower = content.to_lowercase();
                if content_lower.contains("seed") || content_lower.contains("reproducib") {
                    earned += 1.0;
                    description.push("seed documented");
                }
            }
        }

        if description.is_empty() {
            description.push("no random seed management");
        }

        PopperSubScore::new(
            "F1",
            "Random Seed Fixing",
            earned.min(max),
            max,
            &description.join(", "),
        )
    }

    /// F2: Model Versioning (2 points)
    ///
    /// Checks for:
    /// - DVC, MLflow, or equivalent (1 point)
    /// - Model checkpoints versioning (1 point)
    fn score_model_versioning(&self, project_path: &Path) -> PopperSubScore {
        let mut earned: f64 = 0.0;
        let max: f64 = 2.0;
        let mut description = Vec::new();

        // Check for versioning tools
        let versioning_files = [
            ("dvc.yaml", "DVC configured"),
            ("dvc.lock", "DVC configured"),
            (".dvc", "DVC configured"),
            ("mlflow.yaml", "MLflow configured"),
            ("MLproject", "MLflow configured"),
            ("wandb/", "W&B configured"),
            (".wandb", "W&B configured"),
        ];

        for (file, desc) in versioning_files {
            if project_path.join(file).exists() {
                earned += 1.0;
                description.push(desc);
                break;
            }
        }

        // Check for model registry or artifact storage
        let artifact_dirs = ["models/", "checkpoints/", "artifacts/", "saved_models/"];
        for dir in artifact_dirs {
            if project_path.join(dir).exists() {
                earned += 1.0;
                description.push("model artifacts tracked");
                break;
            }
        }

        if description.is_empty() {
            description.push("no model versioning");
        }

        PopperSubScore::new(
            "F2",
            "Model Versioning",
            earned.min(max),
            max,
            &description.join(", "),
        )
    }

    /// F3: Dataset Documentation (1 point)
    ///
    /// Checks for:
    /// - Data documentation or datasheet (1 point)
    fn score_dataset_documentation(&self, project_path: &Path) -> PopperSubScore {
        let mut earned: f64 = 0.0;
        let max: f64 = 1.0;
        let mut description = Vec::new();

        // Check for data documentation
        let data_docs = [
            "data/README.md",
            "datasets/README.md",
            "DATA.md",
            "DATASHEET.md",
            "data_card.md",
            "model_card.md",
        ];

        for doc in data_docs {
            if project_path.join(doc).exists() {
                earned += 1.0;
                description.push("dataset documentation found");
                break;
            }
        }

        // Check README for data section
        if earned < 1.0 {
            let readme = project_path.join("README.md");
            if readme.exists() {
                if let Ok(content) = std::fs::read_to_string(&readme) {
                    let content_lower = content.to_lowercase();
                    if content_lower.contains("## data")
                        || content_lower.contains("## dataset")
                        || content_lower.contains("data source")
                    {
                        earned += 1.0;
                        description.push("data section in README");
                    }
                }
            }
        }

        if description.is_empty() {
            description.push("no dataset documentation");
        }

        PopperSubScore::new(
            "F3",
            "Dataset Documentation",
            earned.min(max),
            max,
            &description.join(", "),
        )
    }

    /// Read all Python files in the project
    fn read_python_files(&self, project_path: &Path) -> Result<String, std::io::Error> {
        let mut content = String::new();

        // Read Python files in root and common directories
        let dirs = [".", "src", "scripts", "train"];
        for dir in dirs {
            let path = project_path.join(dir);
            if path.exists() && path.is_dir() {
                if let Ok(entries) = std::fs::read_dir(&path) {
                    for entry in entries.flatten() {
                        let file_path = entry.path();
                        if file_path.extension().map(|e| e == "py").unwrap_or(false) {
                            if let Ok(file_content) = std::fs::read_to_string(&file_path) {
                                content.push_str(&file_content);
                            }
                        }
                    }
                }
            }
        }

        Ok(content)
    }
}

impl Default for MLReproducibilityScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl PopperScorer for MLReproducibilityScorer {
    fn name(&self) -> &str {
        "ML/AI Reproducibility"
    }

    fn category_id(&self) -> char {
        'F'
    }

    fn max_points(&self) -> f64 {
        5.0
    }

    fn score(&self, project_path: &Path) -> PopperScorerResult<PopperCategoryScore> {
        let is_ml = self.is_ml_project(project_path);

        let mut category = PopperCategoryScore::new(self.name(), 0.0, self.max_points());
        category.is_applicable = is_ml;

        if !is_ml {
            category.add_finding(PopperFinding::info(
                "Non-ML project - Category F excluded from scoring",
            ));
            return Ok(category);
        }

        // Score each sub-category
        let f1 = self.score_random_seed_fixing(project_path);
        let f2 = self.score_model_versioning(project_path);
        let f3 = self.score_dataset_documentation(project_path);

        // Add findings based on scores
        if f1.earned < 1.0 {
            category.add_finding(PopperFinding::warning(
                "Random seed not fixed - training may not be reproducible",
                2.0 - f1.earned,
            ));
        }

        if f2.earned < 1.0 {
            category.add_finding(PopperFinding::warning(
                "Model versioning missing - consider using DVC or MLflow",
                2.0 - f2.earned,
            ));
        }

        if f3.earned < 1.0 {
            category.add_finding(PopperFinding::warning(
                "Dataset documentation missing - add a data card or README",
                1.0 - f3.earned,
            ));
        }

        if f1.earned + f2.earned + f3.earned >= 4.0 {
            category.add_finding(PopperFinding::positive(
                "Strong ML reproducibility practices",
            ));
        }

        // Add sub-scores
        category.add_sub_score(f1);
        category.add_sub_score(f2);
        category.add_sub_score(f3);

        Ok(category)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_ml_reproducibility_scorer_basics() {
        let scorer = MLReproducibilityScorer::new();
        assert_eq!(scorer.name(), "ML/AI Reproducibility");
        assert_eq!(scorer.category_id(), 'F');
        assert_eq!(scorer.max_points(), 5.0);
        assert!(!scorer.is_gateway());
    }

    #[test]
    fn test_non_ml_project_returns_not_applicable() {
        let temp_dir = tempdir().unwrap();

        // Create a non-ML project
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"cli\"",
        )
        .unwrap();
        fs::write(temp_dir.path().join("README.md"), "# CLI Tool").unwrap();

        let scorer = MLReproducibilityScorer::new();
        let result = scorer.score(temp_dir.path()).unwrap();

        assert!(!result.is_applicable);
    }

    #[test]
    fn test_ml_project_detected() {
        let temp_dir = tempdir().unwrap();

        // Create requirements.txt with PyTorch
        fs::write(
            temp_dir.path().join("requirements.txt"),
            "torch==2.0.0\nnumpy\n",
        )
        .unwrap();

        let scorer = MLReproducibilityScorer::new();
        assert!(scorer.is_ml_project(temp_dir.path()));
    }

    #[test]
    fn test_ml_project_with_seed() {
        let temp_dir = tempdir().unwrap();

        // Create ML project markers
        fs::write(
            temp_dir.path().join("requirements.txt"),
            "torch\ntransformers\n",
        )
        .unwrap();

        // Create training script with seed
        fs::write(
            temp_dir.path().join("train.py"),
            r#"
import torch
import random
import numpy as np

SEED = 42
random.seed(SEED)
np.random.seed(SEED)
torch.manual_seed(SEED)
"#,
        )
        .unwrap();

        let scorer = MLReproducibilityScorer::new();
        let result = scorer.score(temp_dir.path()).unwrap();

        assert!(result.is_applicable);
        let f1 = result.sub_scores.iter().find(|s| s.id == "F1").unwrap();
        assert!(f1.earned >= 1.0);
    }

    #[test]
    fn test_ml_project_with_dvc() {
        let temp_dir = tempdir().unwrap();

        // Create ML project with DVC
        fs::write(temp_dir.path().join("requirements.txt"), "torch\n").unwrap();
        fs::write(temp_dir.path().join("dvc.yaml"), "stages:\n  train:\n").unwrap();

        let scorer = MLReproducibilityScorer::new();
        let result = scorer.score(temp_dir.path()).unwrap();

        assert!(result.is_applicable);
        let f2 = result.sub_scores.iter().find(|s| s.id == "F2").unwrap();
        assert!(f2.earned >= 1.0);
    }

    #[test]
    fn test_rust_ml_project_detected() {
        let temp_dir = tempdir().unwrap();

        // Create Rust ML project with candle
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"ml\"\n\n[dependencies]\ncandle-core = \"0.3\"",
        )
        .unwrap();

        let scorer = MLReproducibilityScorer::new();
        assert!(scorer.is_ml_project(temp_dir.path()));
    }
}
