// ML Reproducibility helper functions
// Included from ml_reproducibility.rs — no `use` imports or `#!` attributes allowed

const RUST_ML_CRATES: &[&str] = &[
    "candle", "burn", "tch", "ort", "tract", "linfa", "smartcore", "ndarray",
];

fn has_ml_files(project_path: &Path) -> bool {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let ml_files = [
        "model.py", "train.py", "training.py", "inference.py",
        "model.onnx", "model.pt", "model.pth", "model.h5",
        "model.keras", "dvc.yaml", "mlflow.yaml", "wandb/", "checkpoints/",
    ];
    ml_files.iter().any(|f| project_path.join(f).exists())
}

fn file_contains_any(path: &Path, patterns: &[&str]) -> bool {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    std::fs::read_to_string(path).is_ok_and(|content| {
        let lower = content.to_lowercase();
        patterns.iter().any(|p| lower.contains(p))
    })
}

fn has_ml_requirements(project_path: &Path) -> bool {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    ["requirements.txt", "pyproject.toml", "setup.py"]
        .iter()
        .any(|f| {
            let path = project_path.join(f);
            path.exists() && file_contains_any(&path, ML_INDICATORS)
        })
}

fn has_rust_ml_crates(project_path: &Path) -> bool {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let cargo_toml = project_path.join("Cargo.toml");
    cargo_toml.exists() && file_contains_any(&cargo_toml, RUST_ML_CRATES)
}

fn has_ml_readme(project_path: &Path) -> bool {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let readme = project_path.join("README.md");
    readme.exists() && file_contains_any(&readme, ML_INDICATORS)
}

fn read_python_files_in_dirs(project_path: &Path) -> String {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let mut content = String::new();
    for dir in [".", "src", "scripts", "train"] {
        let path = project_path.join(dir);
        if path.is_dir() {
            collect_python_content(&path, &mut content);
        }
    }
    content
}

fn collect_python_content(dir: &Path, content: &mut String) {
    debug_assert!(dir.exists(), "dir must exist: {}", dir.display());
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let file_path = entry.path();
        if file_path.extension().is_some_and(|e| e == "py") {
            if let Ok(file_content) = std::fs::read_to_string(&file_path) {
                content.push_str(&file_content);
            }
        }
    }
}
