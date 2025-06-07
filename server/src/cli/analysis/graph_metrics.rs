//! Graph metrics analysis - uses the refactored GraphMetricsAnalyzer

use anyhow::Result;
use std::path::PathBuf;

#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_graph_metrics(
    project_path: PathBuf,
    metrics: Vec<crate::cli::GraphMetricType>,
    pagerank_seeds: Vec<String>,
    damping_factor: f32,
    max_iterations: usize,
    convergence_threshold: f64,
    export_graphml: bool,
    format: crate::cli::GraphMetricsOutputFormat,
    include: Option<String>,
    exclude: Option<String>,
    output: Option<PathBuf>,
    perf: bool,
    top_k: usize,
    min_centrality: f64,
) -> Result<()> {
    // Already refactored in main module
    crate::cli::handle_analyze_graph_metrics(
        project_path,
        metrics,
        pagerank_seeds,
        damping_factor,
        max_iterations,
        convergence_threshold,
        export_graphml,
        format,
        include,
        exclude,
        output,
        perf,
        top_k,
        min_centrality,
    )
    .await
}
