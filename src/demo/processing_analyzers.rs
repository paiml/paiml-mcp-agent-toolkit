// Helper functions for web demo analyses
#[allow(dead_code)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) async fn analyze_context(
    repo_path: &std::path::Path,
) -> Result<crate::services::context::ProjectContext> {
    crate::services::context::analyze_project(repo_path, "rust")
        .await
        .map_err(|e| anyhow::anyhow!("Error analyzing project: {e}"))
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) async fn analyze_complexity(
    repo_path: &std::path::Path,
) -> Result<crate::services::complexity::ComplexityReport> {
    use crate::services::ast_rust::analyze_rust_file_with_complexity;
    use crate::services::complexity::aggregate_results;
    use walkdir::WalkDir;

    let mut file_metrics = Vec::new();

    for entry in WalkDir::new(repo_path)
        .follow_links(false)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rs") {
            if let Ok(metrics) = analyze_rust_file_with_complexity(path).await {
                file_metrics.push(metrics);
            }
        }
    }

    Ok(aggregate_results(file_metrics))
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) async fn analyze_dag(
    repo_path: &std::path::Path,
) -> Result<crate::models::dag::DependencyGraph> {
    use crate::services::dag_builder::DagBuilder;

    let context = crate::services::context::analyze_project(repo_path, "rust")
        .await
        .map_err(|e| anyhow::anyhow!("Error analyzing project: {e}"))?;
    let graph = DagBuilder::build_from_project(&context);

    Ok(graph)
}

#[allow(dead_code)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) async fn analyze_churn(
    repo_path: &std::path::Path,
) -> Result<crate::models::churn::CodeChurnAnalysis> {
    crate::services::git_analysis::GitAnalysisService::analyze_code_churn(repo_path, 30)
        .map_err(|e| anyhow::anyhow!("Error analyzing churn: {e}"))
}

#[allow(dead_code)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) async fn analyze_system_architecture(
    repo_path: &std::path::Path,
) -> Result<crate::services::canonical_query::QueryResult> {
    use crate::services::canonical_query::{
        AnalysisContext, CallGraph, CanonicalQuery, SystemArchitectureQuery,
    };
    use rustc_hash::FxHashMap;

    // Build analysis context
    let _context_result = analyze_context(repo_path).await?;
    let dag_result = analyze_dag(repo_path).await?;
    let complexity_result = analyze_complexity(repo_path).await?;
    let churn_result = analyze_churn(repo_path).await.ok(); // Optional

    // Convert complexity report to map
    let mut complexity_map = FxHashMap::default();
    for file in &complexity_result.files {
        for function in &file.functions {
            complexity_map.insert(function.name.clone(), function.metrics);
        }
    }

    let context = AnalysisContext {
        project_path: repo_path.to_path_buf(),
        ast_dag: dag_result,
        call_graph: CallGraph::default(), // TRACKED: Build actual call graph
        complexity_map,
        churn_analysis: churn_result,
    };

    let query = SystemArchitectureQuery;
    query
        .execute(&context)
        .map_err(|e| anyhow::anyhow!("Error analyzing architecture: {e}"))
}

#[allow(dead_code)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) async fn analyze_defect_probability(
    repo_path: &std::path::Path,
) -> Result<crate::services::defect_probability::ProjectDefectAnalysis> {
    use crate::services::defect_probability::{
        DefectProbabilityCalculator, FileMetrics, ProjectDefectAnalysis,
    };
    use walkdir::WalkDir;

    let calculator = DefectProbabilityCalculator::new();
    let mut file_metrics = Vec::new();

    // Get complexity and churn data
    let complexity_result = analyze_complexity(repo_path).await?;
    let churn_result = analyze_churn(repo_path).await.ok();

    // Build churn map for quick lookup
    let churn_map: std::collections::HashMap<String, f32> = churn_result
        .as_ref()
        .map(|churn| {
            churn
                .files
                .iter()
                .map(|f| (f.relative_path.clone(), f.churn_score))
                .collect()
        })
        .unwrap_or_default();

    // Analyze each Rust file
    for entry in WalkDir::new(repo_path)
        .follow_links(false)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rs") {
            let relative_path = path
                .strip_prefix(repo_path)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            // Find complexity data for this file
            if let Some(file_complexity) = complexity_result
                .files
                .iter()
                .find(|f| f.path == relative_path)
            {
                let churn_score = churn_map.get(&relative_path).copied().unwrap_or(0.0);

                // Aggregate complexity from all functions in file
                let total_complexity: f32 = file_complexity
                    .functions
                    .iter()
                    .map(|f| f32::from(f.metrics.cyclomatic))
                    .sum();
                let avg_complexity = if file_complexity.functions.is_empty() {
                    1.0
                } else {
                    total_complexity / file_complexity.functions.len() as f32
                };

                let max_cyclomatic = file_complexity
                    .functions
                    .iter()
                    .map(|f| f.metrics.cyclomatic)
                    .max()
                    .unwrap_or(1);

                let max_cognitive = file_complexity
                    .functions
                    .iter()
                    .map(|f| f.metrics.cognitive)
                    .max()
                    .unwrap_or(1);

                let total_loc: usize = file_complexity
                    .functions
                    .iter()
                    .map(|f| f.metrics.lines as usize)
                    .sum();

                let metrics = FileMetrics {
                    file_path: relative_path,
                    churn_score,
                    complexity: avg_complexity,
                    duplicate_ratio: 0.0, // TRACKED: Implement duplication detection
                    afferent_coupling: 0.0, // TRACKED: Implement coupling analysis
                    efferent_coupling: 0.0,
                    lines_of_code: total_loc,
                    cyclomatic_complexity: u32::from(max_cyclomatic),
                    cognitive_complexity: u32::from(max_cognitive),
                };

                file_metrics.push(metrics);
            }
        }
    }

    let scores = calculator.calculate_batch(&file_metrics);
    Ok(ProjectDefectAnalysis::from_scores(scores))
}
