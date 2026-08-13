// SATD file discovery: source file finding, filtering, test/vendor detection, and directory traversal.

impl SATDDetector {
    /// Find all source files in a directory, respecting .gitignore.
    /// Uses `git ls-files` for tracked repos, falls back to recursive walk.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) async fn find_source_files(
        &self,
        root: &Path,
    ) -> Result<Vec<PathBuf>, TemplateError> {
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
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            self.is_excluded_directory_name(name)
        } else {
            false
        }
    }

    fn is_excluded_directory_name(&self, name: &str) -> bool {
        name.starts_with('.') || self.is_common_build_directory(name)
    }

    fn is_common_build_directory(&self, name: &str) -> bool {
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
        if self.is_valid_source_file(path) {
            files.push(path.to_path_buf());
        }
    }

    fn is_valid_source_file(&self, path: &Path) -> bool {
        self.is_source_file(path) && !self.is_test_file(path)
    }

    /// Check if a file is a supported source file
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) fn is_source_file(&self, path: &Path) -> bool {
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
    ///
    /// The directory test runs against the project-relative path (#923): it
    /// used to run against the absolute one, so a checkout that merely sat
    /// under a directory named `tests/` — any CI runner or monorepo with such
    /// a segment — classified every file in the project as test code and
    /// reported the whole tree clean.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) fn is_test_file(&self, path: &Path) -> bool {
        // Check if path contains test directories
        let path_str = source_scope::project_relative_str(path);
        if source_scope::has_dir_component(&path_str, &["tests", "test"]) {
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
    ///
    /// Every predicate below reads the path RELATIVE TO ITS OWN PROJECT ROOT
    /// (see [`source_scope`]). They used to read the absolute path, so an
    /// ancestor directory named `examples`, `demo`, `fuzz`, `vendor`, `book`
    /// or `target` — none of which the analysed project chose — excluded the
    /// entire tree and reported "0 violations in 0 files" with exit 0 (#923).
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) fn should_exclude_file(&self, file_path: &Path) -> bool {
        let path_str = source_scope::project_relative_str(file_path);

        self.is_satd_analysis_tool(&path_str)
            || self.is_build_or_config_file(&path_str)
            || self.is_example_or_demo(&path_str)
            || self.is_fuzz_target(&path_str)
            || self.is_generated_or_vendor(&path_str)
    }

    fn is_satd_analysis_tool(&self, path_str: &str) -> bool {
        path_str.contains("satd_detector")
            || path_str.contains("satd_property_tests")
            || path_str.contains("quality_proxy")
            || (path_str.contains("test") && path_str.contains("satd"))
    }

    /// The package's own build script and manifest — the ones that sit beside
    /// `src/`, not every file that happens to be called `build.rs`.
    ///
    /// #925's false negative was exactly this: `src/services/context_impl/
    /// build.rs` holds a production `// TODO: Implement call graph edge
    /// extraction in future iteration`, and `contains("/build.rs")` excluded
    /// the whole file — so the one real marker the issue looked for was
    /// invisible while 57 pieces of prose were reported.
    fn is_build_or_config_file(&self, path_str: &str) -> bool {
        matches!(path_str, "/build.rs" | "/Cargo.toml")
            || path_str.ends_with(".gitignore")
            || path_str.ends_with("README.md")
    }

    fn is_example_or_demo(&self, path_str: &str) -> bool {
        source_scope::has_dir_component(path_str, &["examples", "demo"])
            || path_str.contains("_demo")
    }

    fn is_fuzz_target(&self, path_str: &str) -> bool {
        source_scope::has_dir_component(path_str, &["fuzz", "fuzz_targets"])
    }

    fn is_generated_or_vendor(&self, path_str: &str) -> bool {
        source_scope::has_dir_component(path_str, &["target", "vendor", "node_modules", "book"])
            || path_str.contains(".generated")
    }

    /// Whether `path` is vendored or minified.
    ///
    /// The rule itself lives in [`source_scope::is_vendored_or_minified`] and
    /// is shared with the Known-Defects walk, which needed exactly this
    /// predicate once it learned to read JavaScript (#926) — this method used
    /// to BE the rule, and four of its eight name patterns
    /// (`ends_with(".min.js")`, `".min.css"`, `".bundle.js"`,
    /// `".production.js"`) were already dead, subsumed by the `contains(".min.")`
    /// / `contains(".bundle.")` / `contains(".production.")` tests above them.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) fn is_minified_or_vendor_file(&self, path: &Path) -> bool {
        source_scope::is_vendored_or_minified(path)
    }

    /// Check if file content suggests it's minified (has very long lines)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) async fn is_likely_minified_content(&self, path: &Path) -> bool {
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
