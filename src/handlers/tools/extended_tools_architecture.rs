// System architecture analysis handlers (extracted from extended_tools.rs for CB-040)

#[derive(Debug, Deserialize, Serialize)]
struct AnalyzeSystemArchitectureArgs {
    project_path: Option<String>,
    format: Option<String>,
    show_complexity: Option<bool>,
}

// Helper function to convert DAG node type to CallNodeType
fn convert_node_type(
    dag_type: &crate::models::dag::NodeType,
) -> crate::services::canonical_query::CallNodeType {
    use crate::services::canonical_query::CallNodeType;
    match dag_type {
        crate::models::dag::NodeType::Function => CallNodeType::Function,
        crate::models::dag::NodeType::Class => CallNodeType::Struct,
        crate::models::dag::NodeType::Module => CallNodeType::Module,
        crate::models::dag::NodeType::Trait => CallNodeType::Trait,
        crate::models::dag::NodeType::Interface => CallNodeType::Trait,
    }
}

// Helper function to convert DAG edge type to CallEdgeType
fn convert_edge_type(
    dag_type: &crate::models::dag::EdgeType,
) -> crate::services::canonical_query::CallEdgeType {
    use crate::services::canonical_query::CallEdgeType;
    match dag_type {
        crate::models::dag::EdgeType::Calls => CallEdgeType::FunctionCall,
        crate::models::dag::EdgeType::Imports => CallEdgeType::ModuleImport,
        crate::models::dag::EdgeType::Inherits => CallEdgeType::TraitImpl,
        crate::models::dag::EdgeType::Implements => CallEdgeType::TraitImpl,
        crate::models::dag::EdgeType::Uses => CallEdgeType::FunctionCall,
    }
}

// Helper function to build call graph from DAG
fn build_call_graph(
    dag_result: &crate::models::dag::DependencyGraph,
) -> crate::services::canonical_query::CallGraph {
    use crate::services::canonical_query::{CallEdge, CallGraph, CallNode};

    let call_nodes: Vec<CallNode> = dag_result
        .nodes
        .iter()
        .map(|(node_id, node_info)| CallNode {
            id: node_id.clone(),
            name: node_info.label.clone(),
            module_path: node_info
                .metadata
                .get("module_path")
                .cloned()
                .unwrap_or_else(|| node_info.file_path.clone()),
            node_type: convert_node_type(&node_info.node_type),
        })
        .collect();

    let call_edges: Vec<CallEdge> = dag_result
        .edges
        .iter()
        .map(|edge| CallEdge {
            from: edge.from.clone(),
            to: edge.to.clone(),
            edge_type: convert_edge_type(&edge.edge_type),
            weight: edge.weight,
        })
        .collect();

    CallGraph {
        nodes: call_nodes,
        edges: call_edges,
    }
}

// Helper function to build complexity map
fn build_complexity_map(
    complexity_report: Option<&crate::services::complexity::ComplexityReport>,
) -> rustc_hash::FxHashMap<String, crate::services::complexity::ComplexityMetrics> {
    use crate::services::complexity::ComplexityMetrics;
    use rustc_hash::FxHashMap;

    let mut complexity_map = FxHashMap::default();

    if let Some(report) = complexity_report {
        for file in &report.files {
            for func in &file.functions {
                let key = format!("{}::{}", file.path, func.name);
                complexity_map.insert(
                    key,
                    ComplexityMetrics {
                        cyclomatic: func.metrics.cyclomatic,
                        cognitive: func.metrics.cognitive,
                        nesting_max: func.metrics.nesting_max,
                        lines: func.metrics.lines,
                        halstead: func.metrics.halstead,
                    },
                );
            }
        }
    }

    complexity_map
}

// Helper function to format result
fn format_architecture_result(
    result: &crate::services::canonical_query::QueryResult,
    format: Option<&str>,
) -> String {
    match format {
        Some("json") => serde_json::to_string_pretty(result).unwrap_or_default(),
        _ => format!("# System Architecture Analysis\n\n{}", result.diagram),
    }
}

/// Toyota Way: Extract Method - Handle system architecture analysis (complexity <=8)
async fn handle_analyze_system_architecture(
    request_id: serde_json::Value,
    arguments: serde_json::Value,
) -> McpResponse {
    // Parse arguments
    let (args, project_path) = match parse_architecture_analysis_args(arguments) {
        Ok(result) => result,
        Err(e) => {
            return McpResponse::error(
                request_id,
                -32602,
                format!("Invalid analyze_system_architecture arguments: {e}"),
            );
        }
    };

    info!("Analyzing system architecture for {:?}", project_path);

    // Run deep context analysis
    let deep_context = match run_architecture_deep_context_analysis(&project_path).await {
        Ok(ctx) => ctx,
        Err(e) => {
            return McpResponse::error(
                request_id,
                -32000,
                format!("Failed to analyze project: {e}"),
            );
        }
    };

    // Build analysis context
    let context = match build_architecture_analysis_context(&project_path, &deep_context) {
        Ok(ctx) => ctx,
        Err(e) => {
            return McpResponse::error(request_id, -32000, e);
        }
    };

    // Execute and format results
    execute_architecture_query_and_respond(request_id, args, context, &deep_context)
}

/// Toyota Way: Extract Method - Parse architecture analysis arguments (complexity <=5)
///
/// R22-1 / D101: project_path is required. Missing, null, and
/// empty/whitespace values are rejected with an error that the caller maps
/// to JSON-RPC `-32602` — we never silently default to the server's cwd.
fn parse_architecture_analysis_args(
    arguments: serde_json::Value,
) -> Result<(AnalyzeSystemArchitectureArgs, PathBuf), Box<dyn std::error::Error>> {
    let args: AnalyzeSystemArchitectureArgs = serde_json::from_value(arguments)?;

    let project_path = require_project_path(args.project_path.clone())
        .map_err(Box::<dyn std::error::Error>::from)?;

    Ok((args, project_path))
}

/// Toyota Way: Extract Method - Run deep context analysis for architecture (complexity <=5)
async fn run_architecture_deep_context_analysis(
    project_path: &Path,
) -> Result<crate::services::deep_context::DeepContext, Box<dyn std::error::Error>> {
    use crate::services::deep_context::{DeepContextAnalyzer, DeepContextConfig};

    let config = DeepContextConfig {
        include_analyses: vec![
            crate::services::deep_context::AnalysisType::Ast,
            crate::services::deep_context::AnalysisType::Complexity,
            crate::services::deep_context::AnalysisType::Dag,
        ],
        ..Default::default()
    };

    let analyzer = DeepContextAnalyzer::new(config);
    Ok(analyzer
        .analyze_project(&project_path.to_path_buf())
        .await?)
}

/// Toyota Way: Extract Method - Build architecture analysis context (complexity <=6)
fn build_architecture_analysis_context(
    project_path: &Path,
    deep_context: &crate::services::deep_context::DeepContext,
) -> Result<crate::services::canonical_query::AnalysisContext, String> {
    use crate::services::canonical_query::AnalysisContext;

    let dag_result = deep_context
        .analyses
        .dependency_graph
        .clone()
        .ok_or_else(|| "Failed to generate dependency graph".to_string())?;

    let call_graph = build_call_graph(&dag_result);
    let complexity_map = build_complexity_map(deep_context.analyses.complexity_report.as_ref());

    Ok(AnalysisContext {
        project_path: project_path.to_path_buf(),
        ast_dag: dag_result,
        call_graph,
        complexity_map,
        churn_analysis: deep_context.analyses.churn_analysis.clone(),
    })
}

/// Toyota Way: Extract Method - Execute architecture query and respond (complexity <=8)
fn execute_architecture_query_and_respond(
    request_id: serde_json::Value,
    args: AnalyzeSystemArchitectureArgs,
    context: crate::services::canonical_query::AnalysisContext,
    deep_context: &crate::services::deep_context::DeepContext,
) -> McpResponse {
    use crate::services::canonical_query::{CanonicalQuery, SystemArchitectureQuery};

    let query = SystemArchitectureQuery;
    match query.execute(&context) {
        Ok(result) => {
            let content_text = format_architecture_result(&result, args.format.as_deref());

            let response = json!({
                "content": [{
                    "type": "text",
                    "text": content_text
                }],
                "result": result,
                "format": args.format.unwrap_or_else(|| "mermaid".to_string()),
                "metadata": {
                    "nodes": result.metadata.nodes,
                    "edges": result.metadata.edges,
                    "analysis_time_ms": result.metadata.analysis_time_ms,
                    "complexity_hotspots": deep_context.analyses.complexity_report
                        .as_ref()
                        .map_or(0, |r| r.hotspots.len()),
                }
            });

            McpResponse::success(request_id, response)
        }
        Err(e) => {
            error!("System architecture analysis failed: {}", e);
            McpResponse::error(request_id, -32000, e.to_string())
        }
    }
}
