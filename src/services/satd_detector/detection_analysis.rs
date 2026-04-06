// SATD project analysis: project scanning, directory analysis, and result aggregation.

impl SATDDetector {
    /// Analyze project for SATD patterns
    /// Toyota Way: Extract Method - reduced complexity from 25-><=8
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn analyze_project(
        &self,
        root: &Path,
        include_tests: bool,
    ) -> Result<SATDAnalysisResult, TemplateError> {
        let files = self.find_source_files(root).await?;
        let mut analysis_stats = ProjectAnalysisStats::new();

        self.process_project_files(&files, include_tests, &mut analysis_stats)
            .await;
        let avg_age_days = self
            .calculate_project_debt_age(&analysis_stats.all_debts, root)
            .await;

        Ok(self.build_analysis_result(analysis_stats, avg_age_days))
    }

    /// Toyota Way: Extract Method - process all files in project (complexity <=8)
    async fn process_project_files(
        &self,
        files: &[std::path::PathBuf],
        include_tests: bool,
        stats: &mut ProjectAnalysisStats,
    ) {
        for file_path in files {
            if self.should_skip_file(file_path, include_tests).await {
                continue;
            }

            stats.total_files_analyzed += 1;
            self.process_single_file(file_path, stats).await;
        }
    }

    /// Toyota Way: Extract Method - check if file should be skipped (complexity <=8)
    async fn should_skip_file(&self, file_path: &Path, include_tests: bool) -> bool {
        // Skip test files if not requested
        if !include_tests && self.is_test_file(file_path) {
            return true;
        }

        // Skip minified/vendor files
        if self.is_minified_or_vendor_file(file_path) {
            return true;
        }

        // Check file size constraints
        if let Ok(metadata) = tokio::fs::metadata(file_path).await {
            if metadata.len() > crate::services::file_classifier::LARGE_FILE_THRESHOLD as u64 {
                eprintln!("Warning: Skipped: {} (large file >500KB)", file_path.display());
                return true;
            }

            if metadata.len() > 1_000_000 && self.is_likely_minified_content(file_path).await {
                eprintln!("Warning: Skipped: {} (minified content)", file_path.display());
                return true;
            }
        }

        false
    }

    /// Toyota Way: Extract Method - process individual file (complexity <=8)
    async fn process_single_file(&self, file_path: &Path, stats: &mut ProjectAnalysisStats) {
        match tokio::fs::read_to_string(file_path).await {
            Ok(content) => {
                if content.len() > 10_000_000 {
                    eprintln!(
                        "Warning: Skipping large file {}: {} bytes",
                        file_path.display(),
                        content.len()
                    );
                    return;
                }

                match self.extract_from_content(&content, file_path) {
                    Ok(debts) => {
                        if !debts.is_empty() {
                            stats.files_with_debt += 1;
                        }
                        stats.all_debts.extend(debts);
                    }
                    Err(_e) => {
                        // Silently skip files that fail parsing (e.g., line too long)
                        // Analysis continues successfully with remaining files
                        // BUG-010: Removed noisy warning that interleaved with progress
                    }
                }
            }
            Err(_e) => {
                // Silently skip unreadable files
                // BUG-010: Removed noisy warning that interleaved with progress
            }
        }
    }

    /// Toyota Way: Extract Method - calculate debt age (complexity <=3)
    async fn calculate_project_debt_age(&self, debts: &[TechnicalDebt], root: &Path) -> f64 {
        if !debts.is_empty() && root.join(".git").exists() {
            self.calculate_average_debt_age(debts, root)
                .await
                .unwrap_or(0.0)
        } else {
            0.0
        }
    }

    /// Toyota Way: Extract Method - build analysis result (complexity <=5)
    fn build_analysis_result(
        &self,
        stats: ProjectAnalysisStats,
        avg_age_days: f64,
    ) -> SATDAnalysisResult {
        SATDAnalysisResult {
            items: stats.all_debts.clone(),
            summary: SATDSummary {
                total_items: stats.all_debts.len(),
                by_severity: self.group_debts_by_severity(&stats.all_debts),
                by_category: self.group_debts_by_category(&stats.all_debts),
                files_with_satd: stats.files_with_debt,
                avg_age_days,
            },
            total_files_analyzed: stats.total_files_analyzed,
            files_with_debt: stats.files_with_debt,
            analysis_timestamp: chrono::Utc::now(),
        }
    }

    /// Toyota Way: Extract Method - group debts by severity (complexity <=3)
    fn group_debts_by_severity(
        &self,
        debts: &[TechnicalDebt],
    ) -> std::collections::HashMap<String, usize> {
        let mut map = std::collections::HashMap::with_capacity(3);
        for debt in debts {
            *map.entry(format!("{:?}", debt.severity)).or_insert(0) += 1;
        }
        map
    }

    /// Toyota Way: Extract Method - group debts by category (complexity <=3)
    fn group_debts_by_category(
        &self,
        debts: &[TechnicalDebt],
    ) -> std::collections::HashMap<String, usize> {
        let mut map = std::collections::HashMap::with_capacity(5);
        for debt in debts {
            *map.entry(format!("{:?}", debt.category)).or_insert(0) += 1;
        }
        map
    }

    /// Analyze debt in a directory recursively (excluding test files by default)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn analyze_directory(
        &self,
        root: &Path,
    ) -> Result<Vec<TechnicalDebt>, TemplateError> {
        self.analyze_directory_with_tests(root, false).await
    }

    /// Analyze debt in a directory recursively with test file inclusion control
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn analyze_directory_with_tests(
        &self,
        root: &Path,
        include_tests: bool,
    ) -> Result<Vec<TechnicalDebt>, TemplateError> {
        let mut all_debts = Vec::new();
        let files = self.find_source_files(root).await?;

        for file_path in files {
            if self
                .should_skip_file_for_analysis(&file_path, include_tests)
                .await
            {
                continue;
            }

            let debts = self.process_file_for_debts(&file_path).await;
            all_debts.extend(debts);
        }

        Ok(all_debts)
    }

    async fn should_skip_file_for_analysis(&self, file_path: &Path, include_tests: bool) -> bool {
        // Skip test files unless explicitly requested
        if !include_tests && self.is_test_file(file_path) {
            return true;
        }

        // Skip minified/vendor files
        if self.is_minified_or_vendor_file(file_path) {
            return true;
        }

        // Check file size and minification for large files
        self.should_skip_large_file(file_path).await
    }

    async fn should_skip_large_file(&self, file_path: &Path) -> bool {
        if let Ok(metadata) = tokio::fs::metadata(file_path).await {
            if metadata.len() > 1_000_000 && self.is_likely_minified_content(file_path).await {
                return true;
            }
        }
        false
    }

    async fn process_file_for_debts(&self, file_path: &Path) -> Vec<TechnicalDebt> {
        match tokio::fs::read_to_string(file_path).await {
            Ok(content) => self.extract_debts_from_content(&content, file_path),
            Err(_e) => {
                // Silently skip unreadable files
                // BUG-010: Removed noisy warning that interleaved with progress
                Vec::new()
            }
        }
    }

    fn extract_debts_from_content(&self, content: &str, file_path: &Path) -> Vec<TechnicalDebt> {
        // Validate file size before processing
        if content.len() > 10_000_000 {
            eprintln!(
                "Warning: Skipping large file {}: {} bytes",
                file_path.display(),
                content.len()
            );
            return Vec::new();
        }

        // Silently skip files that fail parsing (BUG-010: Removed noisy warning)
        self.extract_from_content(content, file_path)
            .unwrap_or_default()
    }
}
