// ExportService and report creation helpers.
// Included from export.rs — no `use` imports or inner attributes allowed.

impl ExportService {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        let mut exporters: std::collections::HashMap<String, Box<dyn Exporter>> =
            std::collections::HashMap::new();

        exporters.insert("markdown".to_string(), Box::new(MarkdownExporter));
        exporters.insert("json".to_string(), Box::new(JsonExporter::new(true)));
        exporters.insert("sarif".to_string(), Box::new(SarifExporter));

        Self { exporters }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Export.
    pub fn export(&self, format: &str, report: &ExportReport) -> Result<String> {
        self.exporters
            .get(format)
            .ok_or_else(|| anyhow::anyhow!("Unsupported export format: {format}"))?
            .export(report)
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    /// Save to file.
    pub fn save_to_file(&self, format: &str, report: &ExportReport, path: &Path) -> Result<()> {
        let content = self.export(format, report)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Supported formats.
    pub fn supported_formats(&self) -> Vec<&str> {
        self.exporters
            .keys()
            .map(std::string::String::as_str)
            .collect()
    }
}

impl Default for ExportService {
    fn default() -> Self {
        Self::new()
    }
}

// Helper to create ExportReport from analysis results
#[must_use]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
/// Create export report.
pub fn create_export_report(
    repo_name: &str,
    dag: &DependencyGraph,
    complexity: Option<&ComplexityReport>,
    churn: Option<&CodeChurnAnalysis>,
    mermaid_diagram: &str,
    analysis_time_ms: u64,
) -> ExportReport {
    create_full_export_report(
        repo_name,
        dag,
        complexity,
        churn,
        mermaid_diagram,
        analysis_time_ms,
        vec![], // empty ast_contexts for backward compatibility
        None,   // no SATD analysis
        None,   // no dead code summary
        vec![], // no cross references
        None,   // no quality scorecard
        None,   // no defect summary
    )
}

// Full helper to create comprehensive ExportReport
#[allow(clippy::too_many_arguments)]
#[must_use]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
/// Create full export report.
pub fn create_full_export_report(
    repo_name: &str,
    dag: &DependencyGraph,
    complexity: Option<&ComplexityReport>,
    churn: Option<&CodeChurnAnalysis>,
    mermaid_diagram: &str,
    analysis_time_ms: u64,
    ast_contexts: Vec<FileContext>,
    satd_analysis: Option<SATDAnalysisResult>,
    dead_code_results: Option<DeadCodeRankingResult>,
    cross_references: Vec<CrossLangReference>,
    quality_scorecard: Option<QualityScorecard>,
    defect_summary: Option<DefectSummary>,
    // tdg_analysis: Option<TDGAnalysis>, // Disabled due to compilation errors
) -> ExportReport {
    let complexity_analysis = if let Some(c) = complexity {
        let hotspots = c
            .files
            .iter()
            .flat_map(|file| {
                file.functions.iter().map(move |func| Hotspot {
                    file: format!("{}::{}", file.path, func.name),
                    complexity: u32::from(func.metrics.cyclomatic),
                    churn_score: u32::from(func.metrics.cognitive),
                })
            })
            .collect();

        ComplexityAnalysis {
            hotspots,
            total_files: c.files.len(),
            average_complexity: f64::from(c.summary.median_cyclomatic),
            technical_debt_hours: c.summary.technical_debt_hours as u32,
        }
    } else {
        ComplexityAnalysis {
            hotspots: vec![],
            total_files: 0,
            average_complexity: 0.0,
            technical_debt_hours: 0,
        }
    };

    // Create mermaid graphs map
    let mut mermaid_graphs = HashMap::new();
    mermaid_graphs.insert("main".to_string(), mermaid_diagram.to_string());

    // Create default metadata if not provided
    let metadata = ContextMetadata {
        generated_at: Utc::now(),
        tool_version: env!("CARGO_PKG_VERSION").to_string(),
        project_root: std::path::PathBuf::from(repo_name),
        cache_stats: crate::services::deep_context::CacheStats {
            hit_rate: 0.0,
            memory_efficiency: 0.0,
            time_saved_ms: 0,
        },
        analysis_duration: std::time::Duration::from_millis(analysis_time_ms),
    };

    ExportReport {
        repository: repo_name.to_string(),
        timestamp: Utc::now(),
        metadata,
        ast_contexts,
        dependency_graph: dag.clone(),
        complexity_analysis,
        churn_analysis: churn.cloned(),
        satd_analysis,
        dead_code_results,
        cross_references,
        quality_scorecard,
        defect_summary,
        // tdg_analysis, // Disabled due to compilation errors
        mermaid_graphs,
        summary: ProjectSummary {
            total_nodes: dag.nodes.len(),
            total_edges: dag.edges.len(),
            analyzed_files: complexity.map_or(0, |c| c.files.len()),
            analysis_time_ms,
        },
    }
}
