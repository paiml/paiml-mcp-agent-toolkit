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
        let files = self.discover_files(root, include_tests).await?;
        let mut analysis_stats = ProjectAnalysisStats::new();

        self.process_project_files(&files, include_tests, &mut analysis_stats)
            .await;
        let avg_age_days = self
            .calculate_project_debt_age(&analysis_stats.all_debts, root)
            .await;

        Ok(self.build_analysis_result(analysis_stats, avg_age_days))
    }

    /// Discover the files an analysis will read.
    ///
    /// `find_source_files` filters every candidate through
    /// `is_valid_source_file`, which is `is_source_file() && !is_test_file()`:
    /// test files are dropped during DISCOVERY. So `--include-tests` — whose
    /// only job is to add them — had nothing left to add, and pointing satd
    /// straight at a `tests/` directory reported 0 violations. When tests are
    /// wanted the walk below applies the same directory exclusions and the same
    /// source-file test, minus that drop.
    async fn discover_files(
        &self,
        root: &Path,
        include_tests: bool,
    ) -> Result<Vec<std::path::PathBuf>, TemplateError> {
        if !include_tests {
            return self.find_source_files(root).await;
        }
        let mut files = Vec::new();
        self.collect_files_including_tests(root, &mut files).await?;
        Ok(files)
    }

    fn collect_files_including_tests<'a>(
        &'a self,
        dir: &'a Path,
        files: &'a mut Vec<std::path::PathBuf>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), TemplateError>> + Send + 'a>>
    {
        Box::pin(async move {
            if dir.is_file() {
                if self.is_source_file(dir) {
                    files.push(dir.to_path_buf());
                }
                return Ok(());
            }
            if !dir.is_dir() {
                return Ok(());
            }

            let mut entries = tokio::fs::read_dir(dir).await.map_err(TemplateError::Io)?;
            while let Some(entry) = entries.next_entry().await.map_err(TemplateError::Io)? {
                let path = entry.path();
                if path.is_dir() {
                    if !self.should_skip_directory(&path) {
                        self.collect_files_including_tests(&path, files).await?;
                    }
                } else if self.is_source_file(&path) {
                    files.push(path);
                }
            }
            Ok(())
        })
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

        // Files whose every line is suppressed by `should_exclude_file` were
        // read, scanned and thrown away one line at a time, so they still
        // counted towards `total_files_analyzed`. A file nothing can be
        // reported from was not analysed; saying otherwise is what makes
        // "excluded everything" look like "measured clean" (#923).
        if self.should_exclude_file(file_path) {
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
        let files = self.discover_files(root, include_tests).await?;
        let discovered = files.len();
        let mut analyzed = 0usize;

        for file_path in files {
            if self
                .should_skip_file_for_analysis(&file_path, include_tests)
                .await
            {
                continue;
            }

            analyzed += 1;
            let debts = self.process_file_for_debts(&file_path).await;
            all_debts.extend(debts);
        }

        if analyzed == 0 {
            return Err(Self::nothing_measured(root, discovered));
        }

        Ok(all_debts)
    }

    /// The refusal returned when a walk analysed nothing.
    ///
    /// #923, second half: this entry point returns `Vec<TechnicalDebt>`, and an
    /// empty vec from it is the *only* thing the CLI sees — so "every candidate
    /// was excluded" and "the code is clean" arrived as the same value and were
    /// rendered as the same sentence, `Found 0 SATD violations in 0 files`,
    /// with exit 0. On the real tree `analyze satd -p <repo>/examples` printed
    /// exactly that over 113 `.rs` files holding 10 marker-leading TODO/FIXME
    /// comments. A gate cannot pass on a measurement that was never taken, so
    /// the absence of a measurement is reported as such, the same way a
    /// nonexistent path already is (`ensure_analysis_path_exists`).
    fn nothing_measured(root: &Path, discovered: usize) -> TemplateError {
        let reason = if discovered == 0 {
            format!(
                "no source files were found under {}, so no SATD measurement was taken. \
                 This is not a clean result.",
                root.display()
            )
        } else {
            format!(
                "all {discovered} source file(s) under {} were skipped — test, example, \
                 fuzz, vendored, generated, minified or oversized — so no SATD \
                 measurement was taken. This is not a clean result: point the analysis \
                 at the project root, or pass --include-tests to measure test code.",
                root.display()
            )
        };
        TemplateError::ValidationError {
            parameter: "path".to_string(),
            reason,
        }
    }

    async fn should_skip_file_for_analysis(&self, file_path: &Path, include_tests: bool) -> bool {
        // Skip test files unless explicitly requested
        if !include_tests && self.is_test_file(file_path) {
            return true;
        }

        // See `should_skip_file`: an excluded file is not an analysed file.
        if self.should_exclude_file(file_path) {
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
