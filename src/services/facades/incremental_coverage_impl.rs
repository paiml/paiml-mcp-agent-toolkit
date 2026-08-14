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
        // `--force-refresh` reached this struct and stopped there: the only
        // implementation of it lived on a duplicate handler that the CLI route
        // does not call, and that implementation was itself
        // `eprintln!("🧹 Clearing coverage cache...")` above
        // `// In real implementation, would clear the cache`. It now clears the
        // cache directory for real, through the same helper
        // `enforce extreme --clear-cache` uses.
        if request.force_refresh {
            let cache_path = request.cache_dir.clone().unwrap_or_else(|| {
                std::env::temp_dir().join("pmat_coverage_cache")
            });
            crate::cli::cache_clearing::clear_cache_directory_reporting(
                &cache_path,
                "--force-refresh",
            )?;
        }

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
        request: &IncrementalCoverageRequest,
    ) -> Result<Vec<ChangedFileCoverage>> {
        let measured = Self::load_line_coverage(project_path);

        // `--changed-files-only` ("Include only changed files") was carried into
        // this request and then ignored — the parameter was literally
        // `_request` — so the flag was accepted on every run and observable on
        // none. Restricting the run to the changed set is what the flag asks
        // for; WITHOUT it the run also reports the whole project's measured
        // coverage, which is the context the changed-file numbers are read
        // against.
        if !request.changed_files_only {
            Self::report_project_wide_coverage(measured.as_ref());
        }

        let mut coverage_data = Vec::new();

        for (path, status) in changed_files {
            if status != "M" && status != "A" {
                continue;
            }

            coverage_data.push(Self::file_coverage(path, measured.as_ref()));
        }

        Ok(coverage_data)
    }

    /// Project-wide measured coverage, printed only when the run is NOT limited
    /// to changed files (`--changed-files-only`).
    ///
    /// "No coverage artifact" is stated, never rendered as 0%: an absent
    /// measurement and a measured zero are different facts.
    pub(crate) fn report_project_wide_coverage(
        measured: Option<
            &std::collections::HashMap<String, std::collections::HashMap<usize, u64>>,
        >,
    ) {
        eprintln!("{}", Self::project_wide_coverage_line(measured));
    }

    /// The line [`Self::report_project_wide_coverage`] prints, as a value so it
    /// can be asserted on.
    pub(crate) fn project_wide_coverage_line(
        measured: Option<
            &std::collections::HashMap<String, std::collections::HashMap<usize, u64>>,
        >,
    ) -> String {
        let Some(files) = measured else {
            return "🌍 Project-wide coverage: not measured — no coverage artifact found \
                    (run the project's coverage tool first); --changed-files-only skips this"
                .to_string();
        };

        let (mut covered, mut total) = (0usize, 0usize);
        for hits in files.values() {
            total += hits.len();
            covered += hits.values().filter(|count| **count > 0).count();
        }

        if total == 0 {
            return format!(
                "🌍 Project-wide coverage: not measured — the coverage artifact lists {} file(s) \
                 but no lines",
                files.len()
            );
        }

        #[allow(clippy::cast_precision_loss)]
        let pct = (covered as f64 / total as f64) * 100.0;
        format!(
            "🌍 Project-wide coverage: {pct:.1}% ({covered}/{total} lines across {} measured files)",
            files.len()
        )
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
mod changed_files_only_tests {
    //! `--changed-files-only` was carried into `IncrementalCoverageRequest` and
    //! then dropped on the floor (`_request: &IncrementalCoverageRequest`), so
    //! `analyze incremental-coverage -b HEAD~1` and the same command with the
    //! flag produced byte-identical output over a real 8-file diff.
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_project_wide_line_states_absence_instead_of_zero() {
        let line = IncrementalCoverageFacade::project_wide_coverage_line(None);
        assert!(
            line.contains("not measured"),
            "no artifact must be reported as absent, not as 0%: {line}"
        );
        assert!(
            !line.contains("0.0%"),
            "an absent measurement must never render as a measured zero: {line}"
        );
    }

    #[test]
    fn test_project_wide_line_reports_measured_lines() {
        let mut files: HashMap<String, HashMap<usize, u64>> = HashMap::new();
        files.insert("src/a.rs".to_string(), HashMap::from([(1, 1), (2, 0)]));
        files.insert("src/b.rs".to_string(), HashMap::from([(1, 3), (2, 4)]));

        let line = IncrementalCoverageFacade::project_wide_coverage_line(Some(&files));

        assert!(line.contains("75.0%"), "3 of 4 lines are covered: {line}");
        assert!(line.contains("2 measured files"), "{line}");
    }

    #[test]
    fn test_project_wide_line_distinguishes_empty_artifact() {
        let files: HashMap<String, HashMap<usize, u64>> =
            HashMap::from([("src/a.rs".to_string(), HashMap::new())]);

        let line = IncrementalCoverageFacade::project_wide_coverage_line(Some(&files));

        assert!(
            line.contains("not measured") && line.contains("no lines"),
            "an artifact with no lines is not 0% coverage: {line}"
        );
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
