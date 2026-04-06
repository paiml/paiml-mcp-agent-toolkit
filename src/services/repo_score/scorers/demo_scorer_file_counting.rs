// DemoScorer file counting helpers - extracted for complexity budget

fn is_hidden_dir(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|s| s.starts_with('.'))
}

fn has_extension(path: &Path, ext: &str) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e == ext)
}

impl DemoScorer {
    /// Count files with specific extension
    async fn count_files_by_extension(&self, repo_path: &Path, ext: &str) -> usize {
        let mut count = 0;
        let Ok(entries) = std::fs::read_dir(repo_path) else {
            return 0;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && has_extension(&path, ext) {
                count += 1;
            } else if path.is_dir() && !is_hidden_dir(&path) {
                count += Box::pin(self.count_files_by_extension(&path, ext)).await;
            }
        }
        count
    }

    /// Count code files (rs, py, js, ts, go)
    async fn count_code_files(&self, repo_path: &Path) -> usize {
        let mut count = 0;
        let code_exts = ["rs", "py", "js", "ts", "go", "rb", "java", "c", "cpp"];
        for ext in code_exts {
            count += self.count_files_by_extension(repo_path, ext).await;
        }
        count
    }
}
