/// Extracts patterns from AST
pub struct PatternExtractor {
    config: EntropyConfig,
}

impl PatternExtractor {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new(config: EntropyConfig) -> Self {
        Self { config }
    }

    /// Extract patterns from every source file under `project_path`.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn extract_patterns(&self, project_path: &Path) -> Result<PatternCollection> {
        if !project_path.exists() {
            anyhow::bail!(
                "entropy analysis path does not exist: {}",
                project_path.display()
            );
        }

        // Files arrive in path order (ProjectContext is a BTreeMap), so the walk
        // and everything derived from it is reproducible.
        let context = self.get_project_context(project_path).await?;

        let mut collection = PatternCollection::new();

        for (file_path, ast_data) in &context.files {
            if self.should_process_file(file_path) {
                self.extract_file_patterns(file_path, ast_data, &mut collection)?;
                collection.total_files += 1;
                // Measured, not estimated: the number of source lines we actually
                // read. Reporting 0 here for a populated crate (defect #650) made
                // the metrics block indistinguishable from an empty directory.
                collection.total_loc += Self::count_source_lines(ast_data);
            }
        }

        // Post-process to calculate variations
        self.calculate_pattern_variations(&mut collection);

        Ok(collection)
    }

    /// Count non-blank source lines in a file's contents.
    fn count_source_lines(content: &str) -> usize {
        content.lines().filter(|l| !l.trim().is_empty()).count()
    }

    /// Read the source files to analyze.
    ///
    /// DETERMINISM: this used to shell out to `pmat context <path> --format json`
    /// and only scan the directory when that call failed. That made the analyzed
    /// file set depend on whichever `pmat` happened to be on `$PATH` (a stale or
    /// differently-configured binary yields a different file set, and a context
    /// payload without `files[].content` silently yielded zero files). The
    /// directory scan is now the only path: same directory in, same file set out.
    async fn get_project_context(&self, project_path: &Path) -> Result<ProjectContext> {
        self.scan_source_files(project_path).await
    }

    /// Walk `project_path` and read every source file we know how to analyze.
    async fn scan_source_files(&self, project_path: &Path) -> Result<ProjectContext> {
        use std::fs;
        use walkdir::WalkDir;

        // BTreeMap: WalkDir yields entries in readdir order, which is not stable
        // across machines or runs; the map re-imposes path order.
        let mut files = BTreeMap::new();

        // Walk directory and read Rust files
        for entry in WalkDir::new(project_path)
            .follow_links(false)
            .into_iter()
            .filter_map(std::result::Result::ok)
        {
            let path = entry.path();

            // Process Rust and Ruchy files
            if let Some(extension) = path.extension() {
                if (extension == "rs" || extension == "ruchy" || extension == "rh")
                    && self.should_process_file(path)
                {
                    match fs::read_to_string(path) {
                        Ok(content) => {
                            files.insert(path.to_path_buf(), content);
                        }
                        Err(_) => continue, // Skip files we can't read
                    }
                }
            }
        }

        Ok(ProjectContext { files })
    }

    /// Check if file should be processed
    fn should_process_file(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        !self.config.exclude_paths.iter().any(|pattern| {
            glob::Pattern::new(pattern)
                .map(|p| p.matches(&path_str))
                .unwrap_or(false)
        })
    }

    /// Extract patterns from a single file's AST
    fn extract_file_patterns(
        &self,
        file_path: &Path,
        ast_data: &str,
        collection: &mut PatternCollection,
    ) -> Result<()> {
        // Extract patterns using regex-based AST pattern matching
        // Language-specific extraction based on file extension

        if let Some(extension) = file_path.extension().and_then(|s| s.to_str()) {
            match extension {
                "ruchy" | "rh" => {
                    // Ruchy-specific pattern extraction
                    self.extract_ruchy_actor_patterns(file_path, ast_data, collection)?;
                    self.extract_ruchy_pipeline_patterns(file_path, ast_data, collection)?;
                    self.extract_ruchy_message_passing_patterns(file_path, ast_data, collection)?;
                    self.extract_ruchy_error_handling_patterns(file_path, ast_data, collection)?;
                    self.extract_ruchy_pattern_matching_patterns(file_path, ast_data, collection)?;
                }
                "rs" => {
                    // Standard Rust pattern extraction
                    self.extract_error_handling_patterns(file_path, ast_data, collection)?;
                    self.extract_data_validation_patterns(file_path, ast_data, collection)?;
                    self.extract_resource_management_patterns(file_path, ast_data, collection)?;
                    self.extract_control_flow_patterns(file_path, ast_data, collection)?;
                    self.extract_data_transformation_patterns(file_path, ast_data, collection)?;
                    self.extract_api_call_patterns(file_path, ast_data, collection)?;
                }
                _ => {
                    // Generic pattern extraction for other languages
                    self.extract_control_flow_patterns(file_path, ast_data, collection)?;
                    self.extract_data_transformation_patterns(file_path, ast_data, collection)?;
                }
            }
        }

        Ok(())
    }
}
