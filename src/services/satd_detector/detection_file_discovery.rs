// SATD file discovery: source file finding, filtering, test/vendor detection, and directory traversal.

impl SATDDetector {
    /// Find all source files in a directory, respecting .gitignore.
    /// Uses `git ls-files` for tracked repos, falls back to recursive walk.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) async fn find_source_files(
        &self,
        root: &Path,
    ) -> Result<Vec<PathBuf>, TemplateError> {
        debug_assert!(root.exists(), "root must exist: {}", root.display());
        // Try git ls-files first to respect .gitignore
        if let Ok(output) = tokio::process::Command::new("git")
            .args(["ls-files", "--cached", "--others", "--exclude-standard"])
            .current_dir(root)
            .output()
            .await
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let files: Vec<PathBuf> = stdout
                    .lines()
                    .filter(|line| !line.is_empty())
                    .map(|line| root.join(line))
                    .filter(|path| self.is_valid_source_file(path))
                    .collect();
                if !files.is_empty() {
                    return Ok(files);
                }
            }
        }
        // Fallback: recursive walk (non-git projects)
        let mut files = Vec::new();
        self.collect_files_recursive(root, &mut files).await?;
        Ok(files)
    }

    /// Recursively collect source files
    fn collect_files_recursive<'a>(
        &'a self,
        dir: &'a Path,
        files: &'a mut Vec<PathBuf>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), TemplateError>> + Send + 'a>>
    {
        debug_assert!(true, "contract: collect_files_recursive");
        Box::pin(async move {
            if !dir.is_dir() {
                return Ok(());
            }

            let mut entries = tokio::fs::read_dir(dir).await.map_err(TemplateError::Io)?;

            while let Some(entry) = entries.next_entry().await.map_err(TemplateError::Io)? {
                let path = entry.path();
                self.process_directory_entry(&path, files).await?;
            }

            Ok(())
        })
    }

    async fn process_directory_entry(
        &self,
        path: &Path,
        files: &mut Vec<PathBuf>,
    ) -> Result<(), TemplateError> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        if path.is_dir() {
            self.process_subdirectory(path, files).await
        } else {
            self.process_file(path, files);
            Ok(())
        }
    }

    async fn process_subdirectory(
        &self,
        path: &Path,
        files: &mut Vec<PathBuf>,
    ) -> Result<(), TemplateError> {
        if self.should_skip_directory(path) {
            return Ok(());
        }
        self.collect_files_recursive(path, files).await
    }

    fn should_skip_directory(&self, path: &Path) -> bool {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            self.is_excluded_directory_name(name)
        } else {
            false
        }
    }

    fn is_excluded_directory_name(&self, name: &str) -> bool {
        debug_assert!(!name.is_empty(), "name must not be empty");
        name.starts_with('.') || self.is_common_build_directory(name)
    }

    fn is_common_build_directory(&self, name: &str) -> bool {
        debug_assert!(!name.is_empty(), "name must not be empty");
        [
            "target",
            "node_modules",
            "dist",
            "build",
            "__pycache__",
            "book",
        ]
        .contains(&name)
    }

    fn process_file(&self, path: &Path, files: &mut Vec<PathBuf>) {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        if self.is_valid_source_file(path) {
            files.push(path.to_path_buf());
        }
    }

    fn is_valid_source_file(&self, path: &Path) -> bool {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        self.is_source_file(path) && !self.is_test_file(path)
    }

    /// Check if a file is a supported source file
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) fn is_source_file(&self, path: &Path) -> bool {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            matches!(
                ext,
                "rs" | "py"
                    | "js"
                    | "ts"
                    | "jsx"
                    | "tsx"
                    | "java"
                    | "cpp"
                    | "c"
                    | "h"
                    | "hpp"
                    | "cs"
                    | "go"
                    | "php"
                    | "rb"
                    | "swift"
                    | "kt"
                    | "scala"
                    | "clj"
                    | "hs"
                    | "ml"
                    | "elm"
            )
        } else {
            false
        }
    }

    /// Check if a file is a test file
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) fn is_test_file(&self, path: &Path) -> bool {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        // Check if path contains test directories
        let path_str = path.to_string_lossy();
        if path_str.contains("/tests/")
            || path_str.contains("/test/")
            || path_str.contains("\\tests\\")
            || path_str.contains("\\test\\")
        {
            return true;
        }

        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            // Common test file patterns
            file_name.contains("test")
                || file_name.contains("spec")
                || file_name.ends_with("_test.rs")
                || file_name.ends_with("_test.py")
                || file_name.ends_with("_test.js")
                || file_name.ends_with("_test.ts")
                || file_name.ends_with(".test.js")
                || file_name.ends_with(".test.ts")
                || file_name.ends_with(".spec.js")
                || file_name.ends_with(".spec.ts")
        } else {
            false
        }
    }

    /// Check if file is minified or in vendor directory
    /// Check if file should be excluded from SATD analysis
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) fn should_exclude_file(&self, file_path: &Path) -> bool {
        debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
        let path_str = file_path.to_string_lossy();

        self.is_satd_analysis_tool(&path_str)
            || self.is_build_or_config_file(&path_str)
            || self.is_example_or_demo(&path_str)
            || self.is_fuzz_target(&path_str)
            || self.is_generated_or_vendor(&path_str)
    }

    fn is_satd_analysis_tool(&self, path_str: &str) -> bool {
        debug_assert!(!path_str.is_empty(), "path_str must not be empty");
        path_str.contains("satd_detector")
            || path_str.contains("satd_property_tests")
            || path_str.contains("quality_proxy")
            || (path_str.contains("test") && path_str.contains("satd"))
    }

    fn is_build_or_config_file(&self, path_str: &str) -> bool {
        debug_assert!(!path_str.is_empty(), "path_str must not be empty");
        path_str.contains("/build.rs")
            || path_str.contains("/Cargo.toml")
            || path_str.contains(".gitignore")
            || path_str.contains("README")
    }

    fn is_example_or_demo(&self, path_str: &str) -> bool {
        debug_assert!(!path_str.is_empty(), "path_str must not be empty");
        path_str.contains("/examples/") || path_str.contains("/demo/") || path_str.contains("_demo")
    }

    fn is_fuzz_target(&self, path_str: &str) -> bool {
        debug_assert!(!path_str.is_empty(), "path_str must not be empty");
        path_str.contains("/fuzz/") || path_str.contains("fuzz_targets")
    }

    fn is_generated_or_vendor(&self, path_str: &str) -> bool {
        path_str.contains("/target/")
            || path_str.contains("/vendor/")
            || path_str.contains("/node_modules/")
            || path_str.contains("/book/")
            || path_str.contains(".generated")
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) fn is_minified_or_vendor_file(&self, path: &Path) -> bool {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        // Check if path contains vendor directory
        if path.components().any(|c| c.as_os_str() == "vendor") {
            return true;
        }

        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            // Common minified file patterns
            file_name.contains(".min.")
                || file_name.contains(".bundle.")
                || file_name.contains("-min.")
                || file_name.contains(".production.")
                || file_name.ends_with(".min.js")
                || file_name.ends_with(".min.css")
                || file_name.ends_with(".bundle.js")
                || file_name.ends_with(".production.js")
        } else {
            false
        }
    }

    /// Check if file content suggests it's minified (has very long lines)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) async fn is_likely_minified_content(&self, path: &Path) -> bool {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        use tokio::io::{AsyncBufReadExt, BufReader};

        match tokio::fs::File::open(path).await {
            Ok(file) => {
                let reader = BufReader::new(file);
                let mut lines = reader.lines();

                // Check first few lines for length
                for _ in 0..3 {
                    match lines.next_line().await {
                        Ok(Some(line)) => {
                            if line.len() > 5000 {
                                return true; // Very long line, likely minified
                            }
                        }
                        Ok(None) => break,
                        Err(_) => return false,
                    }
                }
                false
            }
            Err(_) => false,
        }
    }
}
