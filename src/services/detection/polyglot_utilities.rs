// Polyglot detector utility methods: framework detection, scanning, complexity estimation
// Included from polyglot.rs — no `use` imports or `#!` attributes allowed

impl PolyglotDetector {
    fn calculate_recommendation_score(
        &self,
        languages: &[LanguageStats],
        dependencies: &[CrossLanguageDependency],
    ) -> f64 {
        let language_diversity = languages.len() as f64 * 0.2;
        let dependency_complexity = dependencies.len() as f64 * 0.1;
        let avg_complexity = languages.iter().map(|l| l.complexity_score).sum::<f64>()
            / languages.len().max(1) as f64;

        (language_diversity + dependency_complexity + (1.0 - avg_complexity / 100.0))
            .clamp(0.0, 1.0)
    }

    fn estimate_file_complexity(&self, content: &str) -> f64 {
        let lines = content.lines().count();
        let branches = content.matches("if").count()
            + content.matches("for").count()
            + content.matches("while").count();
        let functions = content.matches("fn ").count()
            + content.matches("function").count()
            + content.matches("def ").count();

        (lines + branches * 2 + functions) as f64 / 10.0
    }

    fn detect_frameworks_in_content(&self, content: &str, language: &str) -> Vec<String> {
        match language {
            "Rust" => detect_rust_frameworks(content),
            "TypeScript" | "JavaScript" => detect_js_frameworks(content),
            "Python" => detect_python_frameworks(content),
            _ => Vec::new(),
        }
    }

    fn scan_project_directory(&self, dir: &Path) -> Result<Vec<std::path::PathBuf>> {
        debug_assert!(dir.exists(), "dir must exist: {}", dir.display());
        let mut files = Vec::new();
        if !dir.is_dir() {
            return Ok(files);
        }
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if self.detect_language(&path).is_some() {
                    files.push(path);
                }
            } else if path.is_dir() && !self.should_skip_directory(&path) {
                let mut subdir_files = self.scan_project_directory(&path)?;
                files.append(&mut subdir_files);
            }
        }
        Ok(files)
    }

    fn should_skip_directory(&self, path: &Path) -> bool {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        path.file_name()
            .and_then(|n| n.to_str())
            .map_or(true, |name| {
                matches!(
                    name,
                    ".git" | "node_modules" | "target" | "__pycache__" | ".idea" | ".vscode"
                )
            })
    }
}

fn detect_rust_frameworks(content: &str) -> Vec<String> {
    let mut frameworks = Vec::new();
    if content.contains("tokio") { frameworks.push("Tokio".to_string()); }
    if content.contains("serde") { frameworks.push("Serde".to_string()); }
    if content.contains("actix") { frameworks.push("Actix".to_string()); }
    frameworks
}

fn detect_js_frameworks(content: &str) -> Vec<String> {
    let mut frameworks = Vec::new();
    if content.contains("React") { frameworks.push("React".to_string()); }
    if content.contains("Express") { frameworks.push("Express".to_string()); }
    if content.contains("Vue") { frameworks.push("Vue".to_string()); }
    frameworks
}

fn detect_python_frameworks(content: &str) -> Vec<String> {
    let mut frameworks = Vec::new();
    if content.contains("django") { frameworks.push("Django".to_string()); }
    if content.contains("flask") { frameworks.push("Flask".to_string()); }
    if content.contains("pandas") { frameworks.push("Pandas".to_string()); }
    frameworks
}
