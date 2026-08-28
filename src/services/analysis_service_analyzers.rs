// Analysis methods for AnalysisService: complexity, SATD, and dead code analyzers
// Included from analysis_service.rs - shares parent scope (no use imports allowed)

impl AnalysisService {
    /// There is no complexity analyzer behind this service, and it now says so.
    ///
    /// #1090 / T7: this body was `Ok(ComplexityResults { total_files: 10,
    /// average_complexity: 5.5, max_complexity: 15, violations: vec![] })`. The
    /// same three constants came back for an empty directory and for this
    /// repository, because neither `_path` nor `_options` was ever read — a
    /// value that is identical for every input measures nothing.
    ///
    /// It did not stay a harmless placeholder. `QualityGateService::check_complexity`
    /// called this, discarded the result, and reported "All functions within
    /// complexity limit", so `pmat agent`'s quality gate answered PASSED for a
    /// tree it had not opened. A fabricated number is worse than no number
    /// precisely because it survives being passed on.
    ///
    /// So this refuses rather than inventing. The real analyzer lives behind
    /// `pmat analyze complexity` / `pmat quality-gate --checks complexity`
    /// (`src/services/complexity/`); wiring it into this service is a separate
    /// change, and until it lands the honest answer here is "this service does
    /// not know".
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) async fn analyze_complexity(
        &self,
        _path: &Path,
        _options: &AnalysisOptions,
    ) -> Result<ComplexityResults> {
        anyhow::bail!(
            "complexity is not_measured: AnalysisService has no complexity analyzer wired in \
             (it returned three fixed constants for every path it was ever handed). Run \
             `pmat analyze complexity` or `pmat quality-gate --checks complexity`, which read \
             the tree."
        )
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) async fn analyze_satd(
        &self,
        path: &Path,
        _options: &AnalysisOptions,
    ) -> Result<SatdResults> {
        // Use the actual SATD detector
        let results = self
            .satd_detector
            .analyze_project(path, true)
            .await
            .map_err(|e| anyhow::anyhow!("SATD analysis failed: {e}"))?;

        // Convert TechnicalDebt to SatdViolation
        let violations: Vec<SatdViolation> = results
            .items
            .into_iter()
            .map(|debt| SatdViolation {
                file: debt.file.to_string_lossy().to_string(),
                line: debt.line as usize,
                comment: debt.text,
                category: format!("{:?}", debt.category),
            })
            .collect();

        Ok(SatdResults {
            total_files: results.total_files_analyzed,
            total_satd: violations.len(),
            violations,
        })
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) async fn analyze_dead_code(
        &self,
        path: &Path,
        options: &AnalysisOptions,
    ) -> Result<DeadCodeResults> {
        use crate::models::dead_code::DeadCodeAnalysisConfig;

        let config = DeadCodeAnalysisConfig {
            include_unreachable: true, // Include all dead code
            include_tests: options.include_tests,
            min_dead_lines: 1, // Include even single-line dead code
        };

        // Create a new analyzer instance (DeadCodeAnalyzer doesn't implement Clone)
        let mut analyzer = DeadCodeAnalyzer::new(DeadCodeAnalyzer::DEFAULT_CAPACITY);
        let analysis_result = analyzer.analyze_with_ranking(path, config).await?;

        // Convert ranked files to unused items
        let unused_items: Vec<UnusedItem> = analysis_result
            .ranked_files
            .into_iter()
            .flat_map(|file| {
                file.items.into_iter().map(move |item| UnusedItem {
                    file: file.path.clone(),
                    item: item.name.clone(),
                    line: item.line as usize,
                    item_type: format!("{:?}", item.item_type),
                })
            })
            .collect();

        let total_files = analysis_result.summary.total_files_analyzed;
        let dead_code_count = unused_items.len();
        let dead_code_percentage = if total_files > 0 {
            (dead_code_count as f64 / total_files as f64) * 100.0
        } else {
            0.0
        };

        Ok(DeadCodeResults {
            total_files,
            dead_code_count,
            dead_code_percentage,
            unused_items,
        })
    }
}
