#![cfg_attr(coverage_nightly, coverage(off))]
//! Data processing functions for demo analysis results.

use anyhow::Result;

/// Extract actual analysis results and timings from demo report
#[allow(dead_code)]
pub(crate) fn extract_analysis_from_demo_report(
    demo_report: &crate::demo::DemoReport,
) -> (
    Option<crate::services::complexity::ComplexityReport>,
    Option<crate::models::dag::DependencyGraph>,
    (u64, u64, u64, u64), // timings: (ast, complexity, dag, churn)
) {
    let mut complexity_result = None;
    let mut dag_result = None;
    let mut timings = (0u64, 0u64, 0u64, 0u64);

    for step in &demo_report.steps {
        process_demo_step(step, &mut complexity_result, &mut dag_result, &mut timings);
    }

    (complexity_result, dag_result, timings)
}

/// Process a single demo step (cognitive complexity <=8)
fn process_demo_step(
    step: &crate::demo::DemoStep,
    complexity_result: &mut Option<crate::services::complexity::ComplexityReport>,
    dag_result: &mut Option<crate::models::dag::DependencyGraph>,
    timings: &mut (u64, u64, u64, u64),
) {
    match step.capability {
        "AST Context Analysis" => process_ast_step(step, timings),
        "Code Complexity Analysis" => process_complexity_step(step, complexity_result, timings),
        "DAG Visualization" => process_dag_step(step, dag_result, timings),
        "Code Churn Analysis" => process_churn_step(step, timings),
        _ => {} // Unknown capability - skip
    }
}

/// Process AST context analysis step (cognitive complexity 1)
fn process_ast_step(step: &crate::demo::DemoStep, timings: &mut (u64, u64, u64, u64)) {
    timings.0 = step.elapsed_ms;
}

/// Process complexity analysis step (cognitive complexity <=6)
fn process_complexity_step(
    step: &crate::demo::DemoStep,
    complexity_result: &mut Option<crate::services::complexity::ComplexityReport>,
    timings: &mut (u64, u64, u64, u64),
) {
    timings.1 = step.elapsed_ms;

    if let Some(result) = &step.response.result {
        if let Some(complexity_report) = extract_complexity_from_result(result) {
            *complexity_result = Some(complexity_report);
        }
    }
}

/// Process DAG visualization step (cognitive complexity <=6)
fn process_dag_step(
    step: &crate::demo::DemoStep,
    dag_result: &mut Option<crate::models::dag::DependencyGraph>,
    timings: &mut (u64, u64, u64, u64),
) {
    timings.2 = step.elapsed_ms;

    if let Some(result) = &step.response.result {
        if let Some(dag) = extract_dag_from_result(result) {
            *dag_result = Some(dag);
        }
    }
}

/// Process code churn analysis step (cognitive complexity 1)
fn process_churn_step(step: &crate::demo::DemoStep, timings: &mut (u64, u64, u64, u64)) {
    timings.3 = step.elapsed_ms;
}

/// Extract complexity report from JSON result (cognitive complexity <=5)
pub(crate) fn extract_complexity_from_result(
    result: &serde_json::Value,
) -> Option<crate::services::complexity::ComplexityReport> {
    let complexity_data = serde_json::from_value::<serde_json::Value>(result.clone()).ok()?;
    let report_value = complexity_data.get("report")?;
    serde_json::from_value::<crate::services::complexity::ComplexityReport>(report_value.clone())
        .ok()
}

/// Extract DAG from JSON result (cognitive complexity <=4)
pub(crate) fn extract_dag_from_result(
    result: &serde_json::Value,
) -> Option<crate::models::dag::DependencyGraph> {
    let dag_data = serde_json::from_value::<serde_json::Value>(result.clone()).ok()?;
    parse_dag_data(&dag_data)
}

#[allow(dead_code)]
pub(crate) fn parse_dag_data(
    dag_data: &serde_json::Value,
) -> Option<crate::models::dag::DependencyGraph> {
    // Try to extract basic graph structure from the actual response format
    let node_count = dag_data.get("nodes")?.as_u64().unwrap_or(0) as usize;
    let edge_count = dag_data.get("edges")?.as_u64().unwrap_or(0) as usize;

    // Create a minimal graph structure
    if node_count > 0 || edge_count > 0 {
        return Some(crate::models::dag::DependencyGraph {
            nodes: (0..node_count)
                .map(|i| {
                    let node_id = format!("node_{i}");
                    (
                        node_id.clone(),
                        crate::models::dag::NodeInfo {
                            id: node_id,
                            label: format!("Module {i}"),
                            node_type: crate::models::dag::NodeType::Module,
                            file_path: format!("module_{i}.rs"),
                            line_number: 1,
                            complexity: 1,
                            metadata: Default::default(),
                        },
                    )
                })
                .collect(),
            edges: (0..edge_count)
                .map(|i| crate::models::dag::Edge {
                    from: format!("node_{}", i % node_count),
                    to: format!("node_{}", (i + 1) % node_count),
                    edge_type: crate::models::dag::EdgeType::Imports,
                    weight: 1,
                })
                .collect(),
        });
    }
    None
}

// Helper functions for web demo analyses
#[allow(dead_code)]
pub(crate) async fn analyze_context(
    repo_path: &std::path::Path,
) -> Result<crate::services::context::ProjectContext> {
    crate::services::context::analyze_project(repo_path, "rust")
        .await
        .map_err(|e| anyhow::anyhow!("Error analyzing project: {e}"))
}

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
pub(crate) async fn analyze_churn(
    repo_path: &std::path::Path,
) -> Result<crate::models::churn::CodeChurnAnalysis> {
    crate::services::git_analysis::GitAnalysisService::analyze_code_churn(repo_path, 30)
        .map_err(|e| anyhow::anyhow!("Error analyzing churn: {e}"))
}

#[allow(dead_code)]
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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // parse_dag_data tests
    // ============================================================

    #[test]
    fn test_parse_dag_data_with_valid_data() {
        let dag_data = serde_json::json!({
            "nodes": 5,
            "edges": 4
        });

        let result = parse_dag_data(&dag_data);
        assert!(result.is_some());

        let dag = result.unwrap();
        assert_eq!(dag.nodes.len(), 5);
        assert_eq!(dag.edges.len(), 4);
    }

    #[test]
    fn test_parse_dag_data_with_zero_nodes() {
        let dag_data = serde_json::json!({
            "nodes": 0,
            "edges": 0
        });

        let result = parse_dag_data(&dag_data);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_dag_data_missing_nodes() {
        let dag_data = serde_json::json!({
            "edges": 3
        });

        let result = parse_dag_data(&dag_data);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_dag_data_missing_edges() {
        let dag_data = serde_json::json!({
            "nodes": 3
        });

        let result = parse_dag_data(&dag_data);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_dag_data_node_structure() {
        let dag_data = serde_json::json!({
            "nodes": 3,
            "edges": 2
        });

        let result = parse_dag_data(&dag_data).unwrap();

        // Verify node structure
        for (id, node) in &result.nodes {
            assert!(id.starts_with("node_"));
            assert!(node.label.starts_with("Module "));
            assert!(node.file_path.starts_with("module_"));
            assert_eq!(node.line_number, 1);
            assert_eq!(node.complexity, 1);
        }
    }

    #[test]
    fn test_parse_dag_data_edge_structure() {
        let dag_data = serde_json::json!({
            "nodes": 3,
            "edges": 2
        });

        let result = parse_dag_data(&dag_data).unwrap();

        // Verify edge structure
        for edge in &result.edges {
            assert!(edge.from.starts_with("node_"));
            assert!(edge.to.starts_with("node_"));
            assert_eq!(edge.weight, 1);
        }
    }

    // ============================================================
    // extract_complexity_from_result tests
    // ============================================================

    #[test]
    fn test_extract_complexity_from_result_missing_report() {
        let result = serde_json::json!({
            "status": "ok"
        });

        let complexity = extract_complexity_from_result(&result);
        assert!(complexity.is_none());
    }

    #[test]
    fn test_extract_complexity_from_result_empty_object() {
        let result = serde_json::json!({});
        let complexity = extract_complexity_from_result(&result);
        assert!(complexity.is_none());
    }

    // ============================================================
    // extract_dag_from_result tests
    // ============================================================

    #[test]
    fn test_extract_dag_from_result_valid() {
        let result = serde_json::json!({
            "nodes": 2,
            "edges": 1
        });

        let dag = extract_dag_from_result(&result);
        assert!(dag.is_some());
    }

    #[test]
    fn test_extract_dag_from_result_invalid() {
        let result = serde_json::json!({
            "invalid": "data"
        });

        let dag = extract_dag_from_result(&result);
        assert!(dag.is_none());
    }

    // ============================================================
    // Edge case tests
    // ============================================================

    #[test]
    fn test_parse_dag_data_large_counts() {
        let dag_data = serde_json::json!({
            "nodes": 100,
            "edges": 200
        });

        let result = parse_dag_data(&dag_data).unwrap();
        assert_eq!(result.nodes.len(), 100);
        assert_eq!(result.edges.len(), 200);
    }

    #[test]
    fn test_parse_dag_data_with_one_node() {
        let dag_data = serde_json::json!({
            "nodes": 1,
            "edges": 0
        });

        let result = parse_dag_data(&dag_data);
        assert!(result.is_some());

        let dag = result.unwrap();
        assert_eq!(dag.nodes.len(), 1);
        assert_eq!(dag.edges.len(), 0);
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_parse_dag_data_node_edge_relationship(
            node_count in 1u64..50u64,
            edge_count in 0u64..100u64
        ) {
            let dag_data = serde_json::json!({
                "nodes": node_count,
                "edges": edge_count
            });

            let result = parse_dag_data(&dag_data);
            prop_assert!(result.is_some());

            let dag = result.unwrap();
            prop_assert_eq!(dag.nodes.len(), node_count as usize);
            prop_assert_eq!(dag.edges.len(), edge_count as usize);
        }

        #[test]
        fn test_protocol_trace_response_preservation(
            protocol_name in "[a-z]{1,10}",
            key in "[a-z]{1,10}",
            value in "[a-z0-9]{1,20}"
        ) {
            let response = serde_json::json!({ &key: &value });
            let trace = super::super::orchestration::ProtocolTrace {
                protocol_name: protocol_name.clone(),
                response: response.clone(),
            };

            prop_assert_eq!(trace.protocol_name, protocol_name);
            prop_assert_eq!(&trace.response[&key], &value);
        }
    }
}
