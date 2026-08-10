/// Facade for incremental coverage analysis operations
#[derive(Clone)]
pub struct IncrementalCoverageFacade {
    
    registry: Arc<ServiceRegistry>,
}

impl IncrementalCoverageFacade {
    /// Create a new incremental coverage facade
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(registry: Arc<ServiceRegistry>) -> Self {
        Self { registry }
    }

    /// Perform incremental coverage analysis on a project
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn analyze_project(
        &self,
        request: IncrementalCoverageRequest,
    ) -> Result<IncrementalCoverageResult> {
        // Get changed files between branches
        let changed_files = self
            .get_changed_files(
                &request.project_path,
                &request.base_branch,
                request.target_branch.as_deref(),
            )
            .await?;

        // Analyze coverage for changed files
        let coverage_data = self
            .analyze_coverage_changes(&request.project_path, &changed_files, &request)
            .await?;

        // Build result
        Ok(self.build_coverage_result(coverage_data, changed_files, &request))
    }

    /// Get changed files between branches
    async fn get_changed_files(
        &self,
        project_path: &Path,
        base_branch: &str,
        target_branch: Option<&str>,
    ) -> Result<Vec<(PathBuf, String)>> {
        use crate::cli::coverage_helpers::get_changed_files_for_coverage;

        get_changed_files_for_coverage(project_path, base_branch, target_branch).await
    }

    /// Line coverage for each changed file, read from the coverage artifact the
    /// project already has on disk.
    ///
    /// This used to be `coverage_before = 0.75`, `coverage_after = 0.85`,
    /// `lines 85/100` for every file, under `// Mock coverage analysis for now`
    /// — identical numbers whatever the project contained. Now it reads the same
    /// lcov/llvm-cov artifact `pmat context` reads, and reports "not measured"
    /// when there is none instead of a plausible default.
    ///
    /// Every changed file is looked up, not just the `--top-files` display
    /// slice: the loop used to `break` at `top_files`, and the summary was then
    /// derived from that truncated vector while `total_files` came from the
    /// full changed-file list, so `files_not_measured` simply echoed
    /// `--top-files` (3 -> 3, 10 -> 10, 50 -> 50) and 279 of 289 files were
    /// unaccounted for. The lookup is a hash-map hit, so there is nothing to
    /// save by stopping early.
    async fn analyze_coverage_changes(
        &self,
        project_path: &Path,
        changed_files: &[(PathBuf, String)],
        _request: &IncrementalCoverageRequest,
    ) -> Result<Vec<ChangedFileCoverage>> {
        let measured = Self::load_line_coverage(project_path);
        let mut coverage_data = Vec::new();

        for (path, status) in changed_files {
            if status != "M" && status != "A" {
                continue;
            }

            coverage_data.push(Self::file_coverage(path, measured.as_ref()));
        }

        Ok(coverage_data)
    }

    /// Line-hit counts for the project, or `None` when no coverage run has
    /// produced an artifact. Never runs a coverage tool.
    fn load_line_coverage(
        project_path: &Path,
    ) -> Option<std::collections::HashMap<String, std::collections::HashMap<usize, u64>>> {
        crate::services::agent_context::query::discover_line_coverage(project_path)
    }

    /// Coverage for one changed file. Absent from the artifact ⇒ not measured.
    fn file_coverage(
        path: &Path,
        measured: Option<
            &std::collections::HashMap<String, std::collections::HashMap<usize, u64>>,
        >,
    ) -> ChangedFileCoverage {
        let key = path.to_string_lossy().to_string();
        let lines = measured.and_then(|m| m.get(&key));

        let (coverage_after, lines_covered, lines_total) = match lines {
            Some(hits) if !hits.is_empty() => {
                let total = hits.len();
                let covered = hits.values().filter(|count| **count > 0).count();
                #[allow(clippy::cast_precision_loss)]
                let pct = (covered as f64 / total as f64) * 100.0;
                (Some(pct), covered, total)
            }
            _ => (None, 0, 0),
        };

        ChangedFileCoverage {
            file_path: key,
            // Not measurable: this needs coverage for the base branch, which is
            // not on disk. It used to be reported as 0.75 / 0.0.
            coverage_before: None,
            coverage_after,
            coverage_delta: None,
            status: CoverageStatus::NotMeasured,
            lines_covered,
            lines_total,
        }
    }

    /// Build the final coverage result
    fn build_coverage_result(
        &self,
        coverage_data: Vec<ChangedFileCoverage>,
        changed_files: Vec<(PathBuf, String)>,
        request: &IncrementalCoverageRequest,
    ) -> IncrementalCoverageResult {
        let total_files = changed_files.len();
        let measured: Vec<f64> = coverage_data
            .iter()
            .filter_map(|f| f.coverage_after)
            .collect();

        let covered_files = measured.iter().filter(|pct| **pct > 0.0).count();
        // Counted against ALL changed files, so the summary reconciles:
        // measured + not_measured == total_files. Deleted/renamed files, which
        // get no coverage row at all, are unmeasured too.
        let files_not_measured = total_files.saturating_sub(measured.len());

        // Absent, not zero: "no coverage artifact" is not "0% covered".
        #[allow(clippy::cast_precision_loss)]
        let coverage_percentage = if measured.is_empty() {
            None
        } else {
            Some(measured.iter().sum::<f64>() / measured.len() as f64)
        };

        // Both sides of the comparison are percentages in 0-100 now. The
        // threshold used to be rendered as `threshold * 100.0`, turning the
        // documented default of 80.0 into "8000.0%" (GH #658).
        let files_above_threshold = measured
            .iter()
            .filter(|pct| **pct >= request.coverage_threshold)
            .count();
        let files_below_threshold = measured.len() - files_above_threshold;

        let coverage_text = coverage_percentage
            .map_or_else(|| "not measured".to_string(), |pct| format!("{pct:.1}%"));

        let summary = format!(
            "Analyzed {} changed files: {} covered (mean {}), {} above threshold ({:.1}%), \
             {} below threshold, {} not measured",
            total_files,
            covered_files,
            coverage_text,
            files_above_threshold,
            request.coverage_threshold,
            files_below_threshold,
            files_not_measured
        );

        // --top-files is a display limit only; it must not change any count above.
        let displayed = if request.top_files == 0 {
            coverage_data
        } else {
            coverage_data.into_iter().take(request.top_files).collect()
        };

        IncrementalCoverageResult {
            total_files,
            covered_files,
            coverage_percentage,
            files_above_threshold,
            files_below_threshold,
            files_not_measured,
            changed_files: displayed,
            summary,
        }
    }

    /// Quick coverage analysis with defaults
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn quick_analysis(
        &self,
        project_path: PathBuf,
        base_branch: String,
    ) -> Result<IncrementalCoverageResult> {
        let request = IncrementalCoverageRequest {
            project_path,
            base_branch,
            target_branch: None,
            // Percent, matching `--coverage-threshold`'s documented default.
            coverage_threshold: 80.0,
            changed_files_only: true,
            detailed: false,
            cache_dir: None,
            force_refresh: false,
            top_files: 10,
        };

        self.analyze_project(request).await
    }
}

#[cfg(test)]
mod top_files_accounting_tests {
    //! `--top-files` is a display limit; it must not leak into the summary.
    use super::*;
    use crate::services::service_registry::ServiceRegistry;

    fn request(top_files: usize) -> IncrementalCoverageRequest {
        IncrementalCoverageRequest {
            project_path: PathBuf::from("."),
            base_branch: "master".to_string(),
            target_branch: None,
            coverage_threshold: 80.0,
            changed_files_only: true,
            detailed: false,
            cache_dir: None,
            force_refresh: false,
            top_files,
        }
    }

    /// `--top-files N` used to make `files_not_measured` come back as exactly N
    /// (3 -> 3, 10 -> 10, 50 -> 50) against a constant total_files of 289,
    /// because the summary was derived from the truncated display vector.
    #[test]
    fn test_top_files_truncates_the_display_list_not_the_counts() {
        let facade = IncrementalCoverageFacade::new(Arc::new(ServiceRegistry::new()));

        let changed: Vec<(PathBuf, String)> = (0..25)
            .map(|i| (PathBuf::from(format!("src/f{i}.rs")), "M".to_string()))
            .collect();
        let coverage_data: Vec<ChangedFileCoverage> = changed
            .iter()
            .map(|(path, _)| IncrementalCoverageFacade::file_coverage(path, None))
            .collect();

        let result = facade.build_coverage_result(coverage_data, changed, &request(3));

        assert_eq!(result.total_files, 25);
        assert_eq!(
            result.changed_files.len(),
            3,
            "--top-files 3 must cap the displayed list"
        );
        assert_eq!(
            result.files_not_measured, 25,
            "no coverage artifact ⇒ all 25 changed files are unmeasured, not --top-files of them"
        );
        assert_eq!(
            result.files_above_threshold + result.files_below_threshold + result.files_not_measured,
            result.total_files,
            "the summary must reconcile with total_files"
        );
    }
}
