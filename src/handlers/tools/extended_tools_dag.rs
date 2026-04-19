// DAG analysis handlers (extracted from extended_tools.rs for CB-040)

#[derive(Debug, Deserialize, Serialize)]
struct AnalyzeDagArgs {
    project_path: Option<String>,
    dag_type: Option<String>,
    max_depth: Option<usize>,
    filter_external: Option<bool>,
    show_complexity: Option<bool>,
}

async fn handle_analyze_dag(
    request_id: serde_json::Value,
    arguments: serde_json::Value,
) -> McpResponse {
    let args: AnalyzeDagArgs = match serde_json::from_value(arguments) {
        Ok(a) => a,
        Err(e) => {
            return McpResponse::error(
                request_id,
                -32602,
                format!("Invalid analyze_dag arguments: {e}"),
            );
        }
    };

    // R22-1 / D101 + R22-2 / D102: require explicit project_path (reject
    // null/missing/empty) and glob-expand via shared `services::path_glob`
    // so downstream `analyze_project` sees a concrete directory.
    let project_path = match resolve_project_path(&args.project_path) {
        Ok(p) => p,
        Err(msg) => return McpResponse::error(request_id, -32602, msg),
    };

    match execute_dag_analysis(&args, project_path).await {
        Ok(result) => McpResponse::success(request_id, result),
        Err(e) => McpResponse::error(request_id, -32000, format!("DAG analysis failed: {e}")),
    }
}

/// Toyota Way: Extract Method pattern for DAG analysis
///
/// R22-1 / D101 + R22-2 / D102: project_path is validated and glob-expanded
/// in `resolve_project_path`; this receives the already-validated PathBuf.
async fn execute_dag_analysis(
    args: &AnalyzeDagArgs,
    project_path: PathBuf,
) -> anyhow::Result<serde_json::Value> {
    use crate::services::context::analyze_project;
    let project_context = analyze_project(&project_path, "rust").await?;
    let graph = build_dag_graph(&project_context);
    let dag_type = parse_dag_type(args.dag_type.as_deref());
    let filtered_graph = apply_dag_filters(graph, dag_type.clone());
    let output = generate_dag_output(&filtered_graph, args, dag_type);
    Ok(output)
}

/// R22-1 / D101 + R22-2 / D102: Validate and glob-expand `project_path`.
///
/// D101: reject null/missing/empty values so an MCP client can't cause the
/// DAG analysis to silently run against the server's cwd.
/// D102: expand shell-style globs via the shared `services::path_glob`
/// helper, failing loud on empty expansion.
fn resolve_project_path(project_path: &Option<String>) -> Result<PathBuf, String> {
    let _validated = require_project_path(project_path.clone())?;
    let raw = project_path
        .as_deref()
        .expect("require_project_path returned Ok for None");
    match resolve_project_path_with_globs(raw) {
        ResolvedProjectPath::Concrete(p) => Ok(p),
        e @ ResolvedProjectPath::EmptyGlob(_) => Err(e.into_error_message()),
    }
}


fn build_dag_graph(
    project_context: &crate::services::context::ProjectContext,
) -> crate::models::dag::DependencyGraph {
    use crate::services::dag_builder::DagBuilder;
    DagBuilder::build_from_project_with_limit(project_context, 50)
}

fn parse_dag_type(dag_type_str: Option<&str>) -> crate::cli::DagType {
    use crate::cli::DagType;
    dag_type_str
        .and_then(|t| match t {
            "call-graph" => Some(DagType::CallGraph),
            "import-graph" => Some(DagType::ImportGraph),
            "inheritance" => Some(DagType::Inheritance),
            "full-dependency" => Some(DagType::FullDependency),
            _ => None,
        })
        .unwrap_or(DagType::CallGraph)
}

fn apply_dag_filters(
    graph: crate::models::dag::DependencyGraph,
    dag_type: crate::cli::DagType,
) -> crate::models::dag::DependencyGraph {
    use crate::cli::DagType;
    use crate::services::dag_builder::{
        filter_call_edges, filter_import_edges, filter_inheritance_edges,
    };

    match dag_type {
        DagType::CallGraph => filter_call_edges(graph),
        DagType::ImportGraph => filter_import_edges(graph),
        DagType::Inheritance => filter_inheritance_edges(graph),
        DagType::FullDependency => graph,
    }
}

fn generate_dag_output(
    filtered_graph: &crate::models::dag::DependencyGraph,
    args: &AnalyzeDagArgs,
    dag_type: crate::cli::DagType,
) -> serde_json::Value {
    use crate::services::mermaid_generator::{MermaidGenerator, MermaidOptions};

    let generator = MermaidGenerator::new(MermaidOptions {
        max_depth: args.max_depth,
        filter_external: args.filter_external.unwrap_or(false),
        show_complexity: args.show_complexity.unwrap_or(false),
        ..Default::default()
    });

    let mermaid_output = generator.generate(filtered_graph);
    let output_with_stats = format!(
        "{}\n%% Graph Statistics:\n%% Nodes: {}\n%% Edges: {}\n",
        mermaid_output,
        filtered_graph.nodes.len(),
        filtered_graph.edges.len()
    );

    json!({
        "content": [{
            "type": "text",
            "text": output_with_stats
        }],
        "graph_type": format!("{:?}", dag_type),
        "nodes": filtered_graph.nodes.len(),
        "edges": filtered_graph.edges.len(),
    })
}
