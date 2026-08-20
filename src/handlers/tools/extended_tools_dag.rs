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
    let dag_type = parse_dag_type(args.dag_type.as_deref());

    // #1020: this handler built the graph through `build_from_project_with_limit`
    // — which cuts to the 400-edge Mermaid budget and drops every node the
    // surviving edges do not touch — and then filtered to `Calls` edges that
    // nothing on this path ever produced. `call-graph` was therefore empty for
    // EVERY project, at any size. It now runs the same pipeline as the CLI and
    // the pmcp tool, so all three answer the same question the same way.
    let (filtered_graph, _stats) = crate::services::dag_pipeline::build_typed_dag(
        &project_context,
        &project_path,
        dag_edge_types(&dag_type),
    )
    .await;

    Ok(generate_dag_output(&filtered_graph, args, dag_type))
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

/// The edges a `--dag-type` selects; `None` keeps everything.
fn dag_edge_types(
    dag_type: &crate::cli::DagType,
) -> Option<&'static [crate::models::dag::EdgeType]> {
    use crate::cli::DagType;
    use crate::models::dag::EdgeType;

    match dag_type {
        DagType::CallGraph => Some(&[EdgeType::Calls]),
        DagType::ImportGraph => Some(&[EdgeType::Imports]),
        DagType::Inheritance => Some(&[EdgeType::Inherits, EdgeType::Implements]),
        DagType::FullDependency => None,
    }
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
