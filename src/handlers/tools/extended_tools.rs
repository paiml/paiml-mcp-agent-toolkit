
fn calculate_relevance(template: &crate::models::template::TemplateResource, query: &str) -> f32 {
    let mut score = 0.0;

    // Exact match in name gets highest score
    if template.name.to_lowercase() == query {
        score += 10.0;
    } else if template.name.to_lowercase().contains(query) {
        score += 5.0;
    }

    // Match in description
    if template.description.to_lowercase().contains(query) {
        score += 3.0;
    }

    // Match in parameter names
    for param in &template.parameters {
        if param.name.to_lowercase().contains(query) {
            score += 1.0;
        }
    }

    score
}

#[derive(Debug, Deserialize, Serialize)]
struct AnalyzeComplexityArgs {
    project_path: Option<String>,
    toolchain: Option<String>,
    format: Option<String>,
    max_cyclomatic: Option<u16>,
    max_cognitive: Option<u16>,
    include: Option<Vec<String>>,
    top_files: Option<usize>,
}

fn parse_complexity_args(arguments: serde_json::Value) -> Result<AnalyzeComplexityArgs, String> {
    serde_json::from_value(arguments)
        .map_err(|e| format!("Invalid analyze_complexity arguments: {e}"))
}

struct ComplexityAnalysisContext {
    project_path: PathBuf,
    toolchain: String,
    _thresholds: crate::services::complexity::ComplexityThresholds,
}

fn prepare_complexity_analysis(args: &AnalyzeComplexityArgs) -> ComplexityAnalysisContext {
    let project_path = resolve_project_path_complexity(args.project_path.clone());
    let toolchain = detect_toolchain(&args.toolchain, &project_path);
    let thresholds = build_complexity_thresholds(args);

    ComplexityAnalysisContext {
        project_path,
        toolchain,
        _thresholds: thresholds,
    }
}

#[allow(dead_code)]
async fn perform_complexity_analysis(
    context: &ComplexityAnalysisContext,
    args: &AnalyzeComplexityArgs,
) -> (crate::services::complexity::ComplexityReport, usize) {
    use crate::services::complexity::aggregate_results;

    let (file_metrics, file_count) =
        analyze_project_files(&context.project_path, &context.toolchain, args).await;

    let report = aggregate_results(file_metrics);
    (report, file_count)
}

fn generate_complexity_content(
    report: &crate::services::complexity::ComplexityReport,
    file_metrics: &[crate::services::complexity::FileComplexityMetrics],
    args: &AnalyzeComplexityArgs,
) -> String {
    if let Some(top_files_count) = args.top_files {
        if top_files_count > 0 {
            generate_ranked_content(file_metrics, top_files_count, args)
        } else {
            format_complexity_output(report, args)
        }
    } else {
        format_complexity_output(report, args)
    }
}

fn generate_ranked_content(
    file_metrics: &[crate::services::complexity::FileComplexityMetrics],
    top_files_count: usize,
    args: &AnalyzeComplexityArgs,
) -> String {
    use crate::services::ranking::{rank_files_by_complexity, ComplexityRanker};

    let ranker = ComplexityRanker::default();
    let rankings = rank_files_by_complexity(file_metrics, top_files_count, &ranker);
    format_complexity_rankings(&rankings, args)
}

fn build_complexity_response(
    request_id: serde_json::Value,
    content_text: String,
    report: &crate::services::complexity::ComplexityReport,
    toolchain: &str,
    file_count: usize,
    args: &AnalyzeComplexityArgs,
) -> McpResponse {
    let result = json!({
        "content": [{
            "type": "text",
            "text": content_text
        }],
        "report": report,
        "toolchain": toolchain,
        "files_analyzed": file_count,
        "format": args.format.as_deref().unwrap_or("summary"),
        "top_files": args.top_files,
    });

    McpResponse::success(request_id, result)
}

async fn handle_analyze_complexity(
    request_id: serde_json::Value,
    arguments: serde_json::Value,
) -> McpResponse {
    let args = match parse_complexity_args(arguments) {
        Ok(args) => args,
        Err(e) => return McpResponse::error(request_id, -32602, e),
    };

    let context = prepare_complexity_analysis(&args);

    info!(
        "Analyzing complexity for {:?} using {} toolchain",
        context.project_path, context.toolchain
    );

    let (file_metrics, file_count) =
        analyze_project_files(&context.project_path, &context.toolchain, &args).await;

    let report = crate::services::complexity::aggregate_results(file_metrics.clone());
    let content_text = generate_complexity_content(&report, &file_metrics, &args);

    build_complexity_response(
        request_id,
        content_text,
        &report,
        &context.toolchain,
        file_count,
        &args,
    )
}

fn resolve_project_path_complexity(project_path_arg: Option<String>) -> PathBuf {
    project_path_arg.map_or_else(
        || std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        PathBuf::from,
    )
}

fn detect_toolchain(toolchain_arg: &Option<String>, project_path: &Path) -> String {
    if let Some(t) = toolchain_arg {
        t.clone()
    } else if project_path.join("Cargo.toml").exists() {
        "rust".to_string()
    } else if project_path.join("package.json").exists() || project_path.join("deno.json").exists()
    {
        "deno".to_string()
    } else if project_path.join("pyproject.toml").exists()
        || project_path.join("requirements.txt").exists()
    {
        "python-uv".to_string()
    } else {
        "rust".to_string() // default
    }
}

fn build_complexity_thresholds(
    args: &AnalyzeComplexityArgs,
) -> crate::services::complexity::ComplexityThresholds {
    use crate::services::complexity::ComplexityThresholds;

    let mut thresholds = ComplexityThresholds::default();
    if let Some(max) = args.max_cyclomatic {
        thresholds.cyclomatic_error = max;
        thresholds.cyclomatic_warn = (max * 3 / 4).max(1);
    }
    if let Some(max) = args.max_cognitive {
        thresholds.cognitive_error = max;
        thresholds.cognitive_warn = (max * 3 / 4).max(1);
    }
    thresholds
}

async fn analyze_project_files(
    project_path: &Path,
    toolchain: &str,
    args: &AnalyzeComplexityArgs,
) -> (
    Vec<crate::services::complexity::FileComplexityMetrics>,
    usize,
) {
    use crate::services::file_discovery::ProjectFileDiscovery;

    let mut file_metrics = Vec::with_capacity(256);
    let mut file_count = 0;

    // Use ProjectFileDiscovery which properly respects .gitignore files
    let discovery = ProjectFileDiscovery::new(project_path.to_path_buf());
    let discovered_files = match discovery.discover_files() {
        Ok(files) => files,
        Err(e) => {
            error!("Failed to discover files: {}", e);
            return (file_metrics, file_count);
        }
    };

    for path in discovered_files {
        if path.is_dir() || !should_analyze_file(&path, toolchain) {
            continue;
        }

        if !matches_include_filters(&path, &args.include) {
            continue;
        }

        file_count += 1;

        if let Some(metrics) = analyze_file_complexity(&path, toolchain).await {
            file_metrics.push(metrics);
        }
    }

    (file_metrics, file_count)
}

fn should_analyze_file(path: &Path, toolchain: &str) -> bool {
    match toolchain {
        "rust" => path.extension().and_then(|s| s.to_str()) == Some("rs"),
        "deno" => matches!(
            path.extension().and_then(|s| s.to_str()),
            Some("ts" | "tsx" | "js" | "jsx")
        ),
        "python-uv" => path.extension().and_then(|s| s.to_str()) == Some("py"),
        _ => false,
    }
}

fn matches_include_filters(path: &Path, include_patterns: &Option<Vec<String>>) -> bool {
    let Some(ref patterns) = include_patterns else {
        return true;
    };

    if patterns.is_empty() {
        return true;
    }

    let path_str = path.to_string_lossy();
    patterns
        .iter()
        .any(|pattern| matches_pattern(&path_str, pattern))
}

fn matches_pattern(path_str: &str, pattern: &str) -> bool {
    if pattern.contains("**") {
        // Match any path containing the pattern after **
        let parts: Vec<&str> = pattern.split("**").collect();
        if parts.len() == 2 {
            path_str.contains(parts[1].trim_start_matches('/'))
        } else {
            false
        }
    } else if pattern.starts_with("*.") {
        // Match by extension
        path_str.ends_with(&pattern[1..])
    } else {
        // Direct substring match
        path_str.contains(pattern)
    }
}

async fn analyze_file_complexity(
    path: &Path,
    toolchain: &str,
) -> Option<crate::services::complexity::FileComplexityMetrics> {
    match toolchain {
        "rust" => {
            use crate::services::ast_rust;
            ast_rust::analyze_rust_file_with_complexity(path).await.ok()
        }
        "deno" => {
            #[cfg(feature = "typescript-ast")]
            {
                use crate::services::ast_typescript;
                ast_typescript::analyze_typescript_file_with_complexity(path)
                    .await
                    .ok()
            }
            #[cfg(not(feature = "typescript-ast"))]
            None
        }
        "python-uv" => {
            #[cfg(feature = "python-ast")]
            {
                use crate::services::ast_python;
                ast_python::analyze_python_file_with_complexity(path, None)
                    .await
                    .ok()
            }
            #[cfg(not(feature = "python-ast"))]
            None
        }
        _ => None,
    }
}

fn format_complexity_output(
    report: &crate::services::complexity::ComplexityReport,
    args: &AnalyzeComplexityArgs,
) -> String {
    use crate::services::complexity::{
        format_as_sarif, format_complexity_report, format_complexity_summary,
    };

    let format = args.format.as_deref().unwrap_or("summary");
    match format {
        "full" => format_complexity_report(report),
        "json" => serde_json::to_string_pretty(report).unwrap_or_default(),
        "sarif" => match format_as_sarif(report) {
            Ok(sarif) => sarif,
            Err(_) => "Error generating SARIF format".to_string(),
        },
        _ => format_complexity_summary(report), // default to summary
    }
}

fn format_complexity_rankings(
    rankings: &[(String, crate::services::ranking::CompositeComplexityScore)],
    args: &AnalyzeComplexityArgs,
) -> String {
    use crate::services::ranking::{ComplexityRanker, FileRanker};

    let format = args.format.as_deref().unwrap_or("summary");
    if format == "json" {
        let ranker = ComplexityRanker::default();
        let rankings_json = serde_json::json!({
            "analysis_type": ranker.ranking_type(),
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "top_files": {
                "requested": rankings.len(),
                "returned": rankings.len()
            },
            "rankings": rankings.iter().enumerate().map(|(i, (file, score))| {
                serde_json::json!({
                    "rank": i + 1,
                    "file": file,
                    "metrics": {
                        "functions": score.function_count,
                        "max_cyclomatic": score.cyclomatic_max,
                        "avg_cognitive": score.cognitive_avg,
                        "halstead_effort": score.halstead_effort,
                        "total_score": score.total_score
                    }
                })
            }).collect::<Vec<_>>()
        });
        serde_json::to_string_pretty(&rankings_json).unwrap_or_default()
    } else {
        // Table format (default)
        let mut output = String::with_capacity(1024);
        output.push_str(&format!("## Top {} Complexity Files\n\n", rankings.len()));
        output.push_str("| Rank | File                               | Functions | Max Cyclomatic | Avg Cognitive | Halstead | Score |\n");
        output.push_str("|------|------------------------------------|-----------|--------------  |---------------|----------|-------|\n");

        for (i, (file, score)) in rankings.iter().enumerate() {
            output.push_str(&format!(
                "| {:>4} | {:<50} | {:>9} | {:>14} | {:>13.1} | {:>11.1} | {:>11.1} |\n",
                i + 1,
                file,
                score.function_count,
                score.cyclomatic_max,
                score.cognitive_avg,
                score.halstead_effort,
                score.total_score
            ));
        }
        output.push('\n');
        output
    }
}

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

    match execute_dag_analysis(&args).await {
        Ok(result) => McpResponse::success(request_id, result),
        Err(e) => McpResponse::error(request_id, -32000, format!("DAG analysis failed: {e}")),
    }
}

/// Toyota Way: Extract Method pattern for DAG analysis
async fn execute_dag_analysis(args: &AnalyzeDagArgs) -> anyhow::Result<serde_json::Value> {
    use crate::services::context::analyze_project;
    let project_path = resolve_project_path(&args.project_path);
    let project_context = analyze_project(&project_path, "rust").await?;
    let graph = build_dag_graph(&project_context);
    let dag_type = parse_dag_type(args.dag_type.as_deref());
    let filtered_graph = apply_dag_filters(graph, dag_type.clone());
    let output = generate_dag_output(&filtered_graph, args, dag_type);
    Ok(output)
}

fn resolve_project_path(project_path: &Option<String>) -> PathBuf {
    project_path.as_ref().map_or_else(
        || std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        PathBuf::from,
    )
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

#[derive(Debug, Deserialize, Serialize)]
struct GenerateContextArgs {
    toolchain: Option<String>,
    project_path: Option<String>,
    format: Option<String>,
    debug: Option<bool>,
    debug_output: Option<PathBuf>,
    skip_vendor: Option<bool>,
    max_line_length: Option<usize>,
}

/// Toyota Way: Extract Method - Handle context generation (complexity ≤8)
async fn handle_generate_context(
    request_id: serde_json::Value,
    arguments: serde_json::Value,
) -> McpResponse {
    // Parse and validate arguments
    let (args, project_path) = match parse_generate_context_args(arguments) {
        Ok(result) => result,
        Err(e) => {
            return McpResponse::error(
                request_id,
                -32602,
                format!("Invalid generate_context arguments: {e}"),
            );
        }
    };

    info!("Generating comprehensive context for {:?}", project_path);

    // Configure and run analysis
    let config = build_context_generation_config(&args);
    let deep_context = match run_deep_context_analysis_with_config(&project_path, config).await {
        Ok(ctx) => ctx,
        Err(e) => {
            return McpResponse::error(
                request_id,
                -32000,
                format!("Failed to analyze project: {e}"),
            );
        }
    };

    // Format and respond
    format_and_respond_context(request_id, args, deep_context).await
}

/// Toyota Way: Extract Method - Parse context generation arguments (complexity ≤5)
fn parse_generate_context_args(
    arguments: serde_json::Value,
) -> Result<(GenerateContextArgs, PathBuf), Box<dyn std::error::Error>> {
    let args: GenerateContextArgs = serde_json::from_value(arguments)?;

    let project_path = args.project_path.as_ref().map_or_else(
        || std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        PathBuf::from,
    );

    Ok((args, project_path))
}

/// Toyota Way: Extract Method - Build context generation config (complexity ≤6)
fn build_context_generation_config(
    args: &GenerateContextArgs,
) -> crate::services::deep_context::DeepContextConfig {
    use crate::services::deep_context::DeepContextConfig;
    use crate::services::file_classifier::FileClassifierConfig;

    let mut config = DeepContextConfig::default();

    // Configure FileClassifier settings if debug options are provided
    if should_configure_file_classifier(args) {
        let file_classifier_config = FileClassifierConfig {
            skip_vendor: args.skip_vendor.unwrap_or(true),
            max_line_length: args.max_line_length.unwrap_or(10_000),
            max_file_size: 1_048_576, // 1MB default
        };
        config.file_classifier_config = Some(file_classifier_config);
    }

    config
}

/// Toyota Way: Extract Method - Check if file classifier config needed (complexity ≤3)
fn should_configure_file_classifier(args: &GenerateContextArgs) -> bool {
    args.debug.unwrap_or(false)
        || args.skip_vendor.unwrap_or(false)
        || args.max_line_length.is_some()
}

/// Toyota Way: Extract Method - Run deep context analysis with config (complexity ≤5)
async fn run_deep_context_analysis_with_config(
    project_path: &Path,
    config: crate::services::deep_context::DeepContextConfig,
) -> Result<crate::services::deep_context::DeepContext, Box<dyn std::error::Error>> {
    use crate::services::deep_context::DeepContextAnalyzer;

    let analyzer = DeepContextAnalyzer::new(config);
    Ok(analyzer
        .analyze_project(&project_path.to_path_buf())
        .await?)
}

/// Toyota Way: Extract Method - Format and respond with context (complexity ≤8)
async fn format_and_respond_context(
    request_id: serde_json::Value,
    args: GenerateContextArgs,
    deep_context: crate::services::deep_context::DeepContext,
) -> McpResponse {
    let format = args.format.as_deref().unwrap_or("markdown");
    let content = format_context_content(format, &deep_context).await;

    let result = build_context_response(&args, format, content, &deep_context);
    McpResponse::success(request_id, result)
}

/// Toyota Way: Extract Method - Format context content (complexity ≤5)
async fn format_context_content(
    format: &str,
    deep_context: &crate::services::deep_context::DeepContext,
) -> String {
    if format == "json" {
        serde_json::to_string_pretty(deep_context).unwrap_or_default()
    } else {
        use crate::services::deep_context::{DeepContextAnalyzer, DeepContextConfig};
        let analyzer = DeepContextAnalyzer::new(DeepContextConfig::default());
        analyzer
            .format_as_comprehensive_markdown(deep_context)
            .await
            .unwrap_or_else(|_| "Error formatting deep context".to_string())
    }
}

/// Toyota Way: Extract Method - Build context response JSON (complexity ≤5)
fn build_context_response(
    args: &GenerateContextArgs,
    format: &str,
    content: String,
    deep_context: &crate::services::deep_context::DeepContext,
) -> serde_json::Value {
    json!({
        "content": [{
            "type": "text",
            "text": content
        }],
        "toolchain": args.toolchain.as_deref().unwrap_or("auto-detected"),
        "format": format,
        "analysis_metadata": {
            "generated_at": deep_context.metadata.generated_at,
            "tool_version": deep_context.metadata.tool_version,
            "analysis_duration_ms": deep_context.metadata.analysis_duration.as_millis(),
            "total_files": deep_context.file_tree.total_files,
            "total_size_bytes": deep_context.file_tree.total_size_bytes,
        },
        "quality_scorecard": {
            "overall_health": deep_context.quality_scorecard.overall_health,
            "complexity_score": deep_context.quality_scorecard.complexity_score,
            "maintainability_index": deep_context.quality_scorecard.maintainability_index,
            "technical_debt_hours": deep_context.quality_scorecard.technical_debt_hours,
        }
    })
}

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

/// Toyota Way: Extract Method - Handle system architecture analysis (complexity ≤8)
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

/// Toyota Way: Extract Method - Parse architecture analysis arguments (complexity ≤5)
fn parse_architecture_analysis_args(
    arguments: serde_json::Value,
) -> Result<(AnalyzeSystemArchitectureArgs, PathBuf), Box<dyn std::error::Error>> {
    let args: AnalyzeSystemArchitectureArgs = serde_json::from_value(arguments)?;

    let project_path = args.project_path.as_ref().map_or_else(
        || std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        PathBuf::from,
    );

    Ok((args, project_path))
}

/// Toyota Way: Extract Method - Run deep context analysis for architecture (complexity ≤5)
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

/// Toyota Way: Extract Method - Build architecture analysis context (complexity ≤6)
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

/// Toyota Way: Extract Method - Execute architecture query and respond (complexity ≤8)
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



// Tests extracted to tools_tests.rs for file health compliance (CB-040)
