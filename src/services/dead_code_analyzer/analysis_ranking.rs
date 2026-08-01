impl DeadCodeAnalyzer {
    /// Analyze dead code with ranking functionality
    ///
    /// Performs comprehensive dead code analysis on a project directory,
    /// identifying unused functions, classes, and other code elements.
    /// Returns ranked results with scoring and filtering capabilities.
    ///
    /// # Examples
    ///
    /// ```
    /// use pmat::services::dead_code_analyzer::DeadCodeAnalyzer;
    /// use pmat::models::dead_code::DeadCodeAnalysisConfig;
    /// use std::path::Path;
    /// use tempfile::TempDir;
    /// use std::fs;
    ///
    /// # tokio_test::block_on(async {
    /// let temp_dir = TempDir::new().expect("internal error");
    /// let test_file = temp_dir.path().join("test.rs");
    /// fs::write(&test_file, r#"
    /// fn used_function() -> i32 { 42 }
    /// fn unused_function() -> i32 { 100 }
    /// fn main() { println!("{}", used_function()); }
    /// "#).expect("internal error");
    ///
    /// let mut analyzer = DeadCodeAnalyzer::new(1000);
    /// let config = DeadCodeAnalysisConfig::default();
    /// let result = analyzer.analyze_with_ranking(temp_dir.path(), config).await.expect("internal error");
    ///
    /// assert!(result.summary.total_files_analyzed > 0);
    /// # });
    /// ```
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn analyze_with_ranking(
        &mut self,
        project_path: &Path,
        config: crate::models::dead_code::DeadCodeAnalysisConfig,
    ) -> anyhow::Result<crate::models::dead_code::DeadCodeRankingResult> {
        use crate::services::context::analyze_project_for_dead_code;
        use chrono::Utc;

        // 1. Use optimized dead code analysis that only scans relevant source files
        let project_context = analyze_project_for_dead_code(project_path, "rust").await?;

        // Track total files analyzed
        let total_files_in_project = project_context.files.len();

        // 2. (DAG building not needed for this implementation)

        // 3. Perform dead code analysis using the project context directly
        let report = self.analyze_project_context(&project_context)?;

        // 4. Aggregate by file and create ranking metrics
        let mut file_metrics = self.aggregate_by_file(&report, &project_context, &config)?;

        // 5. Calculate scores and sort
        for metrics in &mut file_metrics {
            metrics.calculate_score();
        }
        file_metrics.sort_by(|a, b| {
            b.dead_score
                .partial_cmp(&a.dead_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // 6. Apply filters
        if !config.include_tests {
            file_metrics.retain(|f| !f.path.contains("test"));
        }
        file_metrics.retain(|f| f.dead_lines >= config.min_dead_lines);

        let mut summary = crate::models::dead_code::DeadCodeSummary::from_files(&file_metrics);
        // Update total files analyzed to reflect actual project files
        summary.total_files_analyzed = total_files_in_project;

        Ok(crate::models::dead_code::DeadCodeRankingResult {
            summary,
            ranked_files: file_metrics,
            analysis_timestamp: Utc::now(),
            config,
        })
    }

    /// Aggregate dead code by file
    fn aggregate_by_file(
        &self,
        report: &DeadCodeReport,
        project_context: &crate::services::context::ProjectContext,
        config: &crate::models::dead_code::DeadCodeAnalysisConfig,
    ) -> anyhow::Result<Vec<crate::models::dead_code::FileDeadCodeMetrics>> {
        use std::collections::HashMap;

        let mut file_map: HashMap<String, crate::models::dead_code::FileDeadCodeMetrics> =
            HashMap::new();

        // Process dead functions
        for dead_item in &report.dead_functions {
            let file_path = dead_item.file_path.clone();
            let entry = file_map
                .entry(file_path.clone())
                .or_insert_with(|| crate::models::dead_code::FileDeadCodeMetrics::new(file_path));

            entry.add_item(crate::models::dead_code::DeadCodeItem {
                item_type: crate::models::dead_code::DeadCodeType::Function,
                name: dead_item.name.clone(),
                line: dead_item.line_number,
                reason: dead_item.reason.clone(),
            });
        }

        // Process dead classes
        for dead_item in &report.dead_classes {
            let file_path = dead_item.file_path.clone();
            let entry = file_map
                .entry(file_path.clone())
                .or_insert_with(|| crate::models::dead_code::FileDeadCodeMetrics::new(file_path));

            entry.add_item(crate::models::dead_code::DeadCodeItem {
                item_type: crate::models::dead_code::DeadCodeType::Class,
                name: dead_item.name.clone(),
                line: dead_item.line_number,
                reason: dead_item.reason.clone(),
            });
        }

        // Process dead variables
        for dead_item in &report.dead_variables {
            let file_path = dead_item.file_path.clone();
            let entry = file_map
                .entry(file_path.clone())
                .or_insert_with(|| crate::models::dead_code::FileDeadCodeMetrics::new(file_path));

            entry.add_item(crate::models::dead_code::DeadCodeItem {
                item_type: crate::models::dead_code::DeadCodeType::Variable,
                name: dead_item.name.clone(),
                line: dead_item.line_number,
                reason: dead_item.reason.clone(),
            });
        }

        // Process unreachable blocks if requested
        if config.include_unreachable {
            for unreachable in &report.unreachable_code {
                let file_path = unreachable.file_path.clone();
                let entry = file_map.entry(file_path.clone()).or_insert_with(|| {
                    crate::models::dead_code::FileDeadCodeMetrics::new(file_path)
                });

                // The real extent is known here, so it replaces `add_item`'s flat
                // 3-line estimate rather than being added on top of it.
                //
                // `unreachable_blocks` is NOT incremented here: `add_item` below
                // already increments it, so doing both counted every block twice
                // and the summary reported exactly double the item list it heads.
                let unreachable_lines = (unreachable.end_line - unreachable.start_line + 1) as usize;

                entry.add_item(crate::models::dead_code::DeadCodeItem {
                    item_type: crate::models::dead_code::DeadCodeType::UnreachableCode,
                    name: format!(
                        "unreachable block {}-{}",
                        unreachable.start_line, unreachable.end_line
                    ),
                    line: unreachable.start_line,
                    reason: unreachable.reason.clone(),
                });

                // Swap the flat estimate `add_item` just billed for the extent
                // actually reported by the analyzer.
                entry.dead_lines = entry.dead_lines.saturating_sub(
                    crate::models::dead_code::UNREACHABLE_BLOCK_LINE_ESTIMATE,
                ) + unreachable_lines;
            }
        }

        // Calculate total lines and percentages for each file.
        //
        // The line count is READ, never estimated. It used to be
        // `file_info.items.len() * 10` ("rough estimate: 10 lines per item")
        // whenever the file appeared in the project context, and
        // `dead_percentage` was then computed against that invented total —
        // the same defect class as `analyze dead-code`'s hardcoded
        // `total_lines: 100`. When the file cannot be read the count stays 0
        // and `update_percentage` leaves the percentage at 0.0 rather than
        // dividing by a number nobody measured.
        for (file_path, metrics) in &mut file_map {
            if let Ok(content) = std::fs::read_to_string(file_path) {
                metrics.total_lines = content.lines().count();
            }

            // `add_item` bills flat per-item line estimates against this measured
            // total, so the estimate could exceed the file it describes and yield
            // a dead_percentage above 100. Dead code cannot occupy more lines than
            // the file physically has.
            if metrics.total_lines > 0 {
                metrics.dead_lines = metrics.dead_lines.min(metrics.total_lines);
            }

            metrics.update_percentage();
        }

        Ok(file_map.into_values().collect())
    }
}
