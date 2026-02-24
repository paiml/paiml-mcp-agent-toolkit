/// Extracts patterns from AST
pub struct PatternExtractor {
    config: EntropyConfig,
}

impl PatternExtractor {
    #[must_use]
    pub fn new(config: EntropyConfig) -> Self {
        Self { config }
    }

    /// Extract patterns from project using pmat context
    pub async fn extract_patterns(&self, project_path: &Path) -> Result<PatternCollection> {
        // Get project context with AST
        let context = self.get_project_context(project_path).await?;

        let mut collection = PatternCollection::new();

        // Process each file's AST
        for (file_path, ast_data) in context.files {
            if self.should_process_file(&file_path) {
                self.extract_file_patterns(&file_path, &ast_data, &mut collection)?;
                collection.total_files += 1;
            }
        }

        // Post-process to calculate variations
        self.calculate_pattern_variations(&mut collection);

        Ok(collection)
    }

    /// Get project context using pmat context command
    async fn get_project_context(&self, project_path: &Path) -> Result<ProjectContext> {
        use std::collections::HashMap;
        use tokio::process::Command;

        // Execute pmat context command to get actual project context
        let output = Command::new("pmat")
            .arg("context")
            .arg(project_path)
            .arg("--format")
            .arg("json")
            .arg("--skip-expensive-metrics")
            .output()
            .await?;

        if !output.status.success() {
            // Fall back to directory scanning if pmat context fails
            return self.scan_directory_fallback(project_path).await;
        }

        let context_json = String::from_utf8(output.stdout)?;

        // Parse the context JSON and extract file information
        let context_value: serde_json::Value = serde_json::from_str(&context_json)?;
        let mut files = HashMap::new();

        // Extract file contents from context
        if let Some(file_tree) = context_value.get("files") {
            if let Some(file_array) = file_tree.as_array() {
                for file_info in file_array {
                    if let (Some(path), Some(content)) = (
                        file_info.get("path").and_then(|p| p.as_str()),
                        file_info.get("content").and_then(|c| c.as_str()),
                    ) {
                        let path_buf = PathBuf::from(path);
                        files.insert(path_buf, content.to_string());
                    }
                }
            }
        }

        Ok(ProjectContext { files })
    }

    /// Fallback method to scan directory when pmat context fails
    async fn scan_directory_fallback(&self, project_path: &Path) -> Result<ProjectContext> {
        use std::fs;
        use walkdir::WalkDir;

        let mut files = HashMap::new();

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
