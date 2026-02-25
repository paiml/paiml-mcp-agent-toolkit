/// Enhanced DAG analysis specifically for CLI with formatted output.
///
/// This is a high-level convenience function that wraps `CodeIntelligence::analyze_comprehensive`
/// and formats the results as a Mermaid diagram with optional analysis annotations.
///
/// # Parameters
///
/// * `project_path` - Path to the project root directory
/// * `_dag_type` - Type of DAG to generate (currently unused, defaults to dependency graph)
/// * `max_depth` - Maximum depth for graph traversal (None = unlimited)
/// * `_filter_external` - Whether to filter external dependencies (currently unused)
/// * `_show_complexity` - Whether to show complexity metrics (currently unused)
/// * `include_duplicates` - Whether to include duplicate detection results
/// * `include_dead_code` - Whether to include dead code analysis results
///
/// # Returns
///
/// A formatted string containing:
/// - Mermaid diagram of the dependency graph
/// - Optional duplicate detection summary
/// - Optional dead code analysis summary
/// - Graph statistics and metadata
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::services::code_intelligence::analyze_dag_enhanced;
/// use pmat::cli::DagType;
/// use tempfile::tempdir;
/// use std::fs;
///
/// # tokio_test::block_on(async {
/// // Create a simple Rust project
/// let dir = tempdir().expect("tempdir");
/// let main_rs = dir.path().join("main.rs");
/// fs::write(&main_rs, "fn main() { println!(\"Hello!\"); }").expect("write");
///
/// let result = analyze_dag_enhanced(
///     &dir.path().to_string_lossy(),
///     DagType::FullDependency,
///     Some(5),
///     false,
///     true,
///     false, // No duplicates
///     false, // No dead code
/// ).await;
///
/// assert!(result.is_ok());
/// let output = result.expect("analysis succeeded");
///
/// // Should contain graph statistics
/// assert!(output.contains("Graph Statistics:"));
/// assert!(output.contains("Total nodes:"));
/// assert!(output.contains("Generation:"));
/// assert!(output.contains("Analysis timestamp:"));
/// # });
/// ```
pub async fn analyze_dag_enhanced(
    project_path: &str,
    _dag_type: crate::cli::DagType,
    max_depth: Option<usize>,
    _filter_external: bool,
    _show_complexity: bool,
    include_duplicates: bool,
    include_dead_code: bool,
) -> anyhow::Result<String> {
    let intelligence = CodeIntelligence::new();

    // Build analysis request
    let mut analysis_types = vec![AnalysisType::DependencyGraph];
    if include_duplicates {
        analysis_types.push(AnalysisType::DuplicateDetection);
    }
    if include_dead_code {
        analysis_types.push(AnalysisType::DeadCodeAnalysis);
    }

    let request = AnalysisRequest {
        project_path: project_path.to_string(),
        analysis_types,
        include_patterns: vec![],
        exclude_patterns: vec![],
        max_depth,
        parallel: true,
    };

    // Run comprehensive analysis
    let report = intelligence.analyze_comprehensive(request).await?;

    // Format output based on dag_type
    let mut output = String::new();

    // Add dependency graph if available
    if let Some(dep_graph) = &report.dependency_graph {
        output.push_str(&dep_graph.mermaid_diagram);
        output.push_str("\n\n");
    }

    // Add duplicate detection results if requested
    if include_duplicates {
        if let Some(duplicates) = &report.duplicates {
            output.push_str("%% Duplicate Detection Results:\n");
            output.push_str(&format!(
                "%% Total clone groups: {}\n",
                duplicates.summary.clone_groups
            ));
            output.push_str(&format!(
                "%% Clone coverage: {:.1}%\n",
                duplicates.summary.duplication_ratio * 100.0
            ));
        }
    }

    // Add dead code analysis results if requested
    if include_dead_code {
        if let Some(dead_code) = &report.dead_code {
            output.push_str("%% Dead Code Analysis:\n");
            output.push_str(&format!(
                "%% Dead code percentage: {:.1}%\n",
                dead_code.summary.percentage_dead
            ));
            output.push_str(&format!(
                "%% Dead functions: {}\n",
                dead_code.dead_functions.len()
            ));
            output.push_str(&format!(
                "%% Dead classes: {}\n",
                dead_code.dead_classes.len()
            ));
        }
    }

    // Add statistics
    let (node_count, generation) = intelligence.get_dag_stats().await;
    output.push_str("\n%% Graph Statistics:\n");
    output.push_str(&format!("%% Total nodes: {node_count}\n"));
    output.push_str(&format!("%% Generation: {generation}\n"));
    output.push_str(&format!("%% Analysis timestamp: {}\n", Utc::now()));

    Ok(output)
}
