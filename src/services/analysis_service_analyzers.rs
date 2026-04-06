// Analysis methods for AnalysisService: complexity, SATD, and dead code analyzers
// Included from analysis_service.rs - shares parent scope (no use imports allowed)

impl AnalysisService {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) async fn analyze_complexity(
        &self,
        _path: &Path,
        _options: &AnalysisOptions,
    ) -> Result<ComplexityResults> {
        debug_assert!(_path.exists(), "_path must exist: {}", _path.display());
        // Implementation would call the actual complexity analyzer
        // This is a simplified version
        Ok(ComplexityResults {
            total_files: 10,
            average_complexity: 5.5,
            max_complexity: 15,
            violations: vec![],
        })
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) async fn analyze_satd(
        &self,
        path: &Path,
        _options: &AnalysisOptions,
    ) -> Result<SatdResults> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
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
        debug_assert!(path.exists(), "path must exist: {}", path.display());
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
