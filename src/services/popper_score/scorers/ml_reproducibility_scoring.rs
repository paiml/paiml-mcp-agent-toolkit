impl MLReproducibilityScorer {
    /// Detect if the project is an ML project
    pub fn is_ml_project(&self, project_path: &Path) -> bool {
        has_ml_files(project_path)
            || has_ml_requirements(project_path)
            || has_rust_ml_crates(project_path)
            || has_ml_readme(project_path)
    }

    /// F1: Random Seed Fixing (2 points)
    fn score_random_seed_fixing(&self, project_path: &Path) -> PopperSubScore {
        let mut earned: f64 = 0.0;
        let max: f64 = 2.0;
        let mut description = Vec::new();

        let content = read_python_files_in_dirs(project_path);
        let seed_patterns = [
            "random.seed", "np.random.seed", "torch.manual_seed",
            "tf.random.set_seed", "set_seed", "SEED =", "seed =",
        ];
        if seed_patterns.iter().any(|p| content.contains(p)) {
            earned += 1.0;
            description.push("seed setting in code");
        }

        if file_contains_any(&project_path.join("README.md"), &["seed", "reproducib"]) {
            earned += 1.0;
            description.push("seed documented");
        }

        if description.is_empty() {
            description.push("no random seed management");
        }

        PopperSubScore::new("F1", "Random Seed Fixing", earned.min(max), max, &description.join(", "))
    }

    /// F2: Model Versioning (2 points)
    fn score_model_versioning(&self, project_path: &Path) -> PopperSubScore {
        let mut earned: f64 = 0.0;
        let max: f64 = 2.0;
        let mut description = Vec::new();

        let versioning_files = [
            ("dvc.yaml", "DVC configured"), ("dvc.lock", "DVC configured"),
            (".dvc", "DVC configured"), ("mlflow.yaml", "MLflow configured"),
            ("MLproject", "MLflow configured"), ("wandb/", "W&B configured"),
            (".wandb", "W&B configured"),
        ];
        for (file, desc) in versioning_files {
            if project_path.join(file).exists() {
                earned += 1.0;
                description.push(desc);
                break;
            }
        }

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

        PopperSubScore::new("F2", "Model Versioning", earned.min(max), max, &description.join(", "))
    }

    /// F3: Dataset Documentation (1 point)
    fn score_dataset_documentation(&self, project_path: &Path) -> PopperSubScore {
        let mut earned: f64 = 0.0;
        let max: f64 = 1.0;
        let mut description = Vec::new();

        let data_docs = [
            "data/README.md", "datasets/README.md", "DATA.md",
            "DATASHEET.md", "data_card.md", "model_card.md",
        ];
        for doc in data_docs {
            if project_path.join(doc).exists() {
                earned += 1.0;
                description.push("dataset documentation found");
                break;
            }
        }

        if earned < 1.0 && file_contains_any(
            &project_path.join("README.md"),
            &["## data", "## dataset", "data source"],
        ) {
            earned += 1.0;
            description.push("data section in README");
        }

        if description.is_empty() {
            description.push("no dataset documentation");
        }

        PopperSubScore::new("F3", "Dataset Documentation", earned.min(max), max, &description.join(", "))
    }
}
