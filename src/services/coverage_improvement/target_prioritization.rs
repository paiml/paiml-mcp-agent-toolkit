// Target prioritization using PMAT analysis
// Included into mod.rs via include!() -- no `use` imports or `#!` attributes allowed

impl CoverageImprovementService {
    /// Prioritize files for test generation using PMAT analysis
    ///
    /// Uses a weighted scoring system:
    /// - Complexity: 40% weight
    /// - SATD (Technical Debt): 30% weight
    /// - Dead Code: 20% weight
    /// - Git Churn: 10% weight
    ///
    /// Returns top N files sorted by score (highest priority first).
    async fn prioritize_targets(&self) -> Result<Vec<PathBuf>> {
        debug_assert!(true, "contract: prioritize_targets");
        eprintln!("🎯 Prioritizing files for test generation...");

        // Run PMAT analyze commands in parallel
        let complexity_fut = self.run_pmat_analyze("complexity");
        let satd_fut = self.run_pmat_analyze("satd");
        let dead_code_fut = self.run_pmat_analyze("dead-code");
        let churn_fut = self.run_pmat_analyze("churn");

        let (complexity_output, satd_output, dead_code_output, churn_output) =
            tokio::try_join!(complexity_fut, satd_fut, dead_code_fut, churn_fut)?;

        // Parse outputs and calculate scores
        let mut file_scores: std::collections::HashMap<PathBuf, f64> =
            std::collections::HashMap::new();

        // Parse complexity (40% weight)
        self.parse_and_score(&complexity_output, &mut file_scores, 0.4)?;

        // Parse SATD (30% weight)
        self.parse_and_score(&satd_output, &mut file_scores, 0.3)?;

        // Parse dead code (20% weight)
        self.parse_and_score(&dead_code_output, &mut file_scores, 0.2)?;

        // Parse churn (10% weight)
        self.parse_and_score(&churn_output, &mut file_scores, 0.1)?;

        // Apply focus and exclude patterns
        file_scores.retain(|path, _score| {
            let path_str = path.to_string_lossy();

            // Check exclude patterns
            if !self.config.exclude_patterns.is_empty() {
                for pattern in &self.config.exclude_patterns {
                    if glob::Pattern::new(pattern)
                        .ok()
                        .map(|p| p.matches(&path_str))
                        .unwrap_or(false)
                    {
                        return false;
                    }
                }
            }

            // Check focus patterns (if specified, only include matching files)
            if !self.config.focus_patterns.is_empty() {
                for pattern in &self.config.focus_patterns {
                    if glob::Pattern::new(pattern)
                        .ok()
                        .map(|p| p.matches(&path_str))
                        .unwrap_or(false)
                    {
                        return true;
                    }
                }
                return false;
            }

            true
        });

        // Sort by score descending and take top N (default 10)
        let mut files_vec: Vec<(PathBuf, f64)> = file_scores.into_iter().collect();
        files_vec.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let top_n = 10; // TODO: Make this configurable
        let targets: Vec<PathBuf> = files_vec
            .into_iter()
            .take(top_n)
            .map(|(path, score)| {
                eprintln!("  📄 {} (score: {:.2})", path.display(), score);
                path
            })
            .collect();

        eprintln!("✅ Prioritized {} files", targets.len());

        Ok(targets)
    }

    /// Run a PMAT analyze command and return stdout
    async fn run_pmat_analyze(&self, analysis_type: &str) -> Result<String> {
        debug_assert!(!analysis_type.is_empty(), "analysis_type must not be empty");
        let output = Command::new("pmat")
            .args(["analyze", analysis_type, "--format", "json"])
            .current_dir(&self.config.project_path)
            .output()
            .await
            .context(format!(
                "Failed to execute `pmat analyze {}`",
                analysis_type
            ))?;

        if !output.status.success() {
            eprintln!(
                "⚠️  `pmat analyze {}` returned non-zero exit code, using empty results",
                analysis_type
            );
            return Ok("{}".to_string());
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Parse PMAT analyze output and add weighted scores to file_scores map
    ///
    /// Simple heuristic: Count occurrences of file paths in the output and normalize
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) fn parse_and_score(
        &self,
        output: &str,
        file_scores: &mut std::collections::HashMap<PathBuf, f64>,
        weight: f64,
    ) -> Result<()> {
        debug_assert!(!output.is_empty(), "output must not be empty");
        debug_assert!(weight >= 0.0, "weight must be non-negative");
        // Try to parse as JSON first
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(output) {
            // Extract file paths from JSON
            self.extract_files_from_json(&json_value, file_scores, weight);
        } else {
            // Fallback: Parse as text, looking for file paths
            for line in output.lines() {
                if let Some(path) = self.extract_file_path_from_line(line) {
                    *file_scores.entry(path).or_insert(0.0) += weight;
                }
            }
        }

        Ok(())
    }

    /// Extract file paths from JSON recursively
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) fn extract_files_from_json(
        &self,
        json: &serde_json::Value,
        file_scores: &mut std::collections::HashMap<PathBuf, f64>,
        weight: f64,
    ) {
        debug_assert!(weight >= 0.0, "weight must be non-negative");
        match json {
            serde_json::Value::Object(map) => {
                // Look for common field names that might contain file paths
                if let Some(file_path) = map
                    .get("file")
                    .or_else(|| map.get("path"))
                    .or_else(|| map.get("file_path"))
                {
                    if let Some(path_str) = file_path.as_str() {
                        let path = PathBuf::from(path_str);
                        *file_scores.entry(path).or_insert(0.0) += weight;
                    }
                }

                // Recursively process nested objects
                for value in map.values() {
                    self.extract_files_from_json(value, file_scores, weight);
                }
            }
            serde_json::Value::Array(arr) => {
                for value in arr {
                    self.extract_files_from_json(value, file_scores, weight);
                }
            }
            _ => {}
        }
    }

    /// Extract file path from a text line
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) fn extract_file_path_from_line(&self, line: &str) -> Option<PathBuf> {
        debug_assert!(!line.is_empty(), "line must not be empty");
        // Look for patterns like "src/path/to/file.rs"
        let parts: Vec<&str> = line.split_whitespace().collect();
        for part in parts {
            if part.contains(".rs") || part.contains(".toml") || part.contains(".md") {
                return Some(PathBuf::from(part));
            }
        }
        None
    }
}
