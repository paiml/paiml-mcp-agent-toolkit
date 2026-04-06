// DemoScorer: find_demo_files helper
// Locates demo/example source files in the repository

const DEMO_EXTENSIONS: &[&str] = &["rs", "py", "js", "ts", "go", "rb", "sh"];

const DEMO_ROOT_NAMES: &[&str] = &[
    "demo.rs", "demo.py", "demo.js", "demo.ts", "example.rs", "example.py",
];

impl DemoScorer {
    /// Find demo/example files in the repository
    async fn find_demo_files(&self, repo_path: &Path) -> Vec<std::path::PathBuf> {
        debug_assert!(repo_path.exists(), "repo_path must exist: {}", repo_path.display());
        let mut demo_files = vec![];

        let examples_dirs = ["examples", "demos", "demo", "samples"];
        for dir in examples_dirs {
            let dir_path = repo_path.join(dir);
            if !dir_path.exists() {
                continue;
            }
            if let Ok(entries) = std::fs::read_dir(&dir_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if !path.is_file() {
                        continue;
                    }
                    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                    if DEMO_EXTENSIONS.contains(&ext) {
                        demo_files.push(path);
                    }
                }
            }
        }

        for name in DEMO_ROOT_NAMES {
            let path = repo_path.join(name);
            if path.exists() {
                demo_files.push(path);
            }
        }

        demo_files
    }
}
