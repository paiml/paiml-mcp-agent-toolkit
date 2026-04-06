// Incremental analysis methods for IncrementalChurnAnalyzer
// Included from incremental_churn.rs - do NOT add `use` imports or `#!` attributes here.

impl IncrementalChurnAnalyzer {
    /// Get churn metrics for a specific file (lazy evaluation)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn get_file_churn(
        &self,
        file_path: &Path,
    ) -> Result<FileChurnMetrics, TemplateError> {
        // Check cache first
        let relative_path = file_path
            .strip_prefix(&self.project_root)
            .unwrap_or(file_path)
            .to_path_buf();

        if let Some(entry) = self.cache.get(&relative_path) {
            // Check if file has been modified since cache entry
            if self.is_cache_valid(&entry, file_path).await? {
                return Ok(entry.metrics.clone());
            }
        }

        // Compute churn for this specific file
        let metrics = self.compute_file_churn(file_path).await?;

        // Cache the result
        let commit_hash = self.get_current_commit_hash().await?;
        let entry = ChurnCacheEntry {
            metrics: metrics.clone(),
            last_modified: Utc::now(),
            git_commit_hash: commit_hash,
        };
        self.cache.insert(relative_path, entry);

        Ok(metrics)
    }

    /// Get churn analysis for multiple files (incremental)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn analyze_incremental(
        &self,
        files: Vec<PathBuf>,
        period_days: u32,
    ) -> Result<CodeChurnAnalysis, TemplateError> {
        let mut file_metrics = Vec::new();
        let mut uncached_files = Vec::new();

        // Separate cached and uncached files
        for file in files {
            let relative_path = file
                .strip_prefix(&self.project_root)
                .unwrap_or(&file)
                .to_path_buf();

            if let Some(entry) = self.cache.get(&relative_path) {
                if self.is_cache_valid(&entry, &file).await? {
                    file_metrics.push(entry.metrics.clone());
                    continue;
                }
            }
            uncached_files.push(file);
        }

        // Batch analyze uncached files
        if !uncached_files.is_empty() {
            let new_metrics = self
                .batch_compute_churn(&uncached_files, period_days)
                .await?;

            // Cache new results
            let commit_hash = self.get_current_commit_hash().await?;
            for metrics in &new_metrics {
                let relative_path = metrics
                    .path
                    .strip_prefix(&self.project_root)
                    .unwrap_or(&metrics.path)
                    .to_path_buf();

                let entry = ChurnCacheEntry {
                    metrics: metrics.clone(),
                    last_modified: Utc::now(),
                    git_commit_hash: commit_hash.clone(),
                };
                self.cache.insert(relative_path, entry);
            }

            file_metrics.extend(new_metrics);
        }

        // Generate summary
        let summary = self.generate_summary(&file_metrics);

        Ok(CodeChurnAnalysis {
            generated_at: Utc::now(),
            period_days,
            repository_root: self.project_root.clone(),
            files: file_metrics,
            summary,
        })
    }

    /// Check if cache entry is still valid
    async fn is_cache_valid(
        &self,
        entry: &ChurnCacheEntry,
        file_path: &Path,
    ) -> Result<bool, TemplateError> {
        // Check if file has been modified in git since cache entry
        let current_hash = self.get_file_last_commit_hash(file_path).await?;
        Ok(current_hash == entry.git_commit_hash)
    }
}
