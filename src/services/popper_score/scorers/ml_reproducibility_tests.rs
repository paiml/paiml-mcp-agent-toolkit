#[cfg_attr(coverage_nightly, coverage(off))]
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
