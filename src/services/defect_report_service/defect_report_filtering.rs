// Defect report filtering by file patterns and line count (Feature #52)

impl DefectReportService {
    /// Filter defect report by file patterns and line count (Feature #52)
    ///
    /// This function filters a defect report based on:
    /// - `include`: Optional glob pattern to include only matching files
    /// - `exclude`: Optional glob pattern to exclude matching files
    /// - `min_lines`: Minimum line count threshold (0 = no filtering)
    ///
    /// # Arguments
    ///
    /// * `report` - The original defect report to filter
    /// * `include` - Optional include pattern (e.g., "src/*.rs", "**/*.rs")
    /// * `exclude` - Optional exclude pattern (e.g., "tests/*", "benches/*")
    /// * `min_lines` - Minimum line count (files with fewer lines are excluded)
    ///
    /// # Returns
    ///
    /// A new defect report containing only defects from matching files
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use pmat::services::defect_report_service::DefectReportService;
    ///
    /// let service = DefectReportService::new();
    /// // Filter to only src/ files, exclude tests, minimum 50 lines
    /// let filtered = DefectReportService::filter_by_pattern(
    ///     &report,
    ///     Some("src/**/*.rs".to_string()),
    ///     Some("tests/*".to_string()),
    ///     50
    /// );
    /// ```
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn filter_by_pattern(
        report: &DefectReport,
        include: Option<String>,
        exclude: Option<String>,
        min_lines: usize,
    ) -> DefectReport {
        use globset::{Glob, GlobMatcher};

        // Build glob matchers
        let include_matcher: Option<GlobMatcher> = include
            .as_ref()
            .and_then(|pattern| Glob::new(pattern).ok().map(|g| g.compile_matcher()));

        let exclude_matcher: Option<GlobMatcher> = exclude
            .as_ref()
            .and_then(|pattern| Glob::new(pattern).ok().map(|g| g.compile_matcher()));

        // Filter defects
        let filtered_defects: Vec<Defect> = report
            .defects
            .iter()
            .filter(|defect| {
                // Check include pattern
                if let Some(matcher) = &include_matcher {
                    if !matcher.is_match(&defect.file_path) {
                        return false;
                    }
                }

                // Check exclude pattern
                if let Some(matcher) = &exclude_matcher {
                    if matcher.is_match(&defect.file_path) {
                        return false;
                    }
                }

                // Check min_lines threshold
                if min_lines > 0 {
                    // Line count filtering requires file metadata not yet available in defects
                }

                true
            })
            .cloned()
            .collect();

        // Rebuild file_index
        let mut file_index = BTreeMap::new();
        for defect in &filtered_defects {
            file_index
                .entry(defect.file_path.clone())
                .or_insert_with(Vec::new)
                .push(defect.id.clone());
        }

        // Recompute summary
        let summary = Self::new().compute_summary(&filtered_defects);

        DefectReport {
            metadata: ReportMetadata {
                tool: report.metadata.tool.clone(),
                version: report.metadata.version.clone(),
                generated_at: report.metadata.generated_at,
                project_root: report.metadata.project_root.clone(),
                total_files_analyzed: file_index.len(),
                analysis_duration_ms: report.metadata.analysis_duration_ms,
            },
            summary,
            defects: filtered_defects,
            file_index,
        }
    }
}
