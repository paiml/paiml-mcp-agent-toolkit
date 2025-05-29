use crate::models::churn::ChurnOutputFormat;
use crate::models::mcp::{
    GenerateTemplateArgs, ListTemplatesArgs, McpRequest, McpResponse, ScaffoldProjectArgs,
    SearchTemplatesArgs, ToolCallParams, ValidateTemplateArgs,
};
use crate::services::git_analysis::GitAnalysisService;
use crate::services::template_service;
use crate::TemplateServerTrait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, info};

pub async fn handle_tool_call<T: TemplateServerTrait>(
    server: Arc<T>,
    request: McpRequest,
) -> McpResponse {
    let params = match request.params {
        Some(p) => p,
        None => {
            return McpResponse::error(
                request.id,
                -32602,
                "Invalid params: missing tool call parameters".to_string(),
            );
        }
    };

    let tool_params: ToolCallParams = match serde_json::from_value(params) {
        Ok(p) => p,
        Err(e) => {
            return McpResponse::error(request.id, -32602, format!("Invalid params: {}", e));
        }
    };

    match tool_params.name.as_str() {
        "get_server_info" => handle_get_server_info(request.id).await,
        "generate_template" => {
            handle_generate_template(server, request.id, tool_params.arguments).await
        }
        "list_templates" => handle_list_templates(server, request.id, tool_params.arguments).await,
        "validate_template" => {
            handle_validate_template(server, request.id, tool_params.arguments).await
        }
        "scaffold_project" => {
            handle_scaffold_project(server, request.id, tool_params.arguments).await
        }
        "search_templates" => {
            handle_search_templates(server, request.id, tool_params.arguments).await
        }
        "analyze_code_churn" => handle_analyze_code_churn(request.id, tool_params.arguments).await,
        "analyze_complexity" => handle_analyze_complexity(request.id, tool_params.arguments).await,
        "analyze_dag" => handle_analyze_dag(request.id, tool_params.arguments).await,
        "generate_context" => handle_generate_context(request.id, tool_params.arguments).await,
        _ => McpResponse::error(
            request.id,
            -32602,
            format!("Unknown tool: {}", tool_params.name),
        ),
    }
}

async fn handle_generate_template<T: TemplateServerTrait>(
    server: Arc<T>,
    request_id: serde_json::Value,
    arguments: serde_json::Value,
) -> McpResponse {
    let args: GenerateTemplateArgs = match serde_json::from_value(arguments) {
        Ok(a) => a,
        Err(e) => {
            // Check if it's specifically a missing parameters field
            let error_message = if e.to_string().contains("missing field `parameters`") {
                "Missing required field: parameters".to_string()
            } else {
                format!("Invalid generate_template arguments: {}", e)
            };
            return McpResponse::error(request_id, -32602, error_message);
        }
    };

    info!("Generating template: {}", args.resource_uri);

    match template_service::generate_template(server.as_ref(), &args.resource_uri, args.parameters)
        .await
    {
        Ok(generated) => {
            let result = json!({
                "content": [{
                    "type": "text",
                    "text": generated.content
                }],
                "filename": generated.filename,
                "checksum": generated.checksum,
                "toolchain": generated.toolchain,
            });
            McpResponse::success(request_id, result)
        }
        Err(e) => {
            error!("Template generation failed: {}", e);
            McpResponse::error(request_id, e.to_mcp_code(), e.to_string())
        }
    }
}

async fn handle_list_templates<T: TemplateServerTrait>(
    server: Arc<T>,
    request_id: serde_json::Value,
    arguments: serde_json::Value,
) -> McpResponse {
    let args: ListTemplatesArgs = match serde_json::from_value(arguments) {
        Ok(a) => a,
        Err(e) => {
            return McpResponse::error(
                request_id,
                -32602,
                format!("Invalid list_templates arguments: {}", e),
            );
        }
    };

    match template_service::list_templates(
        server.as_ref(),
        args.toolchain.as_deref(),
        args.category.as_deref(),
    )
    .await
    {
        Ok(templates) => {
            let template_list: Vec<_> = templates
                .into_iter()
                .map(|t| {
                    json!({
                        "uri": t.uri,
                        "name": t.name,
                        "description": t.description,
                        "category": t.category,
                        "toolchain": t.toolchain,
                    })
                })
                .collect();

            let result = json!({
                "content": [{
                    "type": "text",
                    "text": format!("Found {} templates", template_list.len())
                }],
                "templates": template_list,
                "count": template_list.len(),
            });
            McpResponse::success(request_id, result)
        }
        Err(e) => {
            error!("Template listing failed: {}", e);
            McpResponse::error(request_id, -32000, e.to_string())
        }
    }
}

async fn handle_validate_template<T: TemplateServerTrait>(
    server: Arc<T>,
    request_id: serde_json::Value,
    arguments: serde_json::Value,
) -> McpResponse {
    let args: ValidateTemplateArgs = match serde_json::from_value(arguments) {
        Ok(a) => a,
        Err(e) => {
            return McpResponse::error(
                request_id,
                -32602,
                format!("Invalid validate_template arguments: {}", e),
            );
        }
    };

    // Get template metadata to validate parameters
    match server.get_template_metadata(&args.resource_uri).await {
        Ok(metadata) => {
            let mut validation_errors = Vec::new();
            let mut missing_required = Vec::new();

            // Check required parameters
            for param in &metadata.parameters {
                if param.required && !args.parameters.contains_key(&param.name) {
                    missing_required.push(&param.name);
                }
            }

            // Validate parameter values
            for (key, value) in &args.parameters {
                if let Some(param_spec) = metadata.parameters.iter().find(|p| p.name == *key) {
                    if let Some(pattern) = &param_spec.validation_pattern {
                        if let Ok(regex) = regex::Regex::new(pattern) {
                            if let Some(str_val) = value.as_str() {
                                if !regex.is_match(str_val) {
                                    validation_errors.push(format!(
                                        "Parameter '{}' does not match pattern: {}",
                                        key, pattern
                                    ));
                                }
                            }
                        }
                    }
                } else {
                    validation_errors.push(format!("Unknown parameter: {}", key));
                }
            }

            let is_valid = missing_required.is_empty() && validation_errors.is_empty();

            let result = json!({
                "content": [{
                    "type": "text",
                    "text": if is_valid {
                        "Template parameters are valid".to_string()
                    } else {
                        format!("Validation failed: {} errors",
                            missing_required.len() + validation_errors.len())
                    }
                }],
                "valid": is_valid,
                "missing_required": missing_required,
                "validation_errors": validation_errors,
                "template_uri": args.resource_uri,
            });

            McpResponse::success(request_id, result)
        }
        Err(_) => McpResponse::error(
            request_id,
            -32000,
            format!("Template not found: {}", args.resource_uri),
        ),
    }
}

async fn handle_scaffold_project<T: TemplateServerTrait>(
    server: Arc<T>,
    request_id: serde_json::Value,
    arguments: serde_json::Value,
) -> McpResponse {
    let args: ScaffoldProjectArgs = match serde_json::from_value(arguments) {
        Ok(a) => a,
        Err(e) => {
            return McpResponse::error(
                request_id,
                -32602,
                format!("Invalid scaffold_project arguments: {}", e),
            );
        }
    };

    let mut results = Vec::new();
    let mut errors = Vec::new();

    // Generate each requested template
    for template_type in &args.templates {
        let uri = format!("template://{}/{}/", template_type, args.toolchain);

        // Find the appropriate variant based on project type in parameters
        let variant = match template_type.as_str() {
            "makefile" | "readme" | "gitignore" => match args.toolchain.as_str() {
                "rust" | "deno" | "python-uv" => "cli",
                _ => continue,
            },
            _ => continue,
        };

        let full_uri = format!("{}{}", uri, variant);

        match template_service::generate_template(
            server.as_ref(),
            &full_uri,
            args.parameters.clone(),
        )
        .await
        {
            Ok(generated) => {
                results.push(json!({
                    "template": template_type,
                    "filename": generated.filename,
                    "content": generated.content,
                    "checksum": generated.checksum,
                }));
            }
            Err(e) => {
                errors.push(json!({
                    "template": template_type,
                    "error": e.to_string(),
                }));
            }
        }
    }

    let result = json!({
        "content": [{
            "type": "text",
            "text": format!(
                "Scaffolded {} templates successfully, {} errors",
                results.len(),
                errors.len()
            )
        }],
        "generated": results,
        "errors": errors,
        "toolchain": args.toolchain,
    });

    McpResponse::success(request_id, result)
}

async fn handle_search_templates<T: TemplateServerTrait>(
    server: Arc<T>,
    request_id: serde_json::Value,
    arguments: serde_json::Value,
) -> McpResponse {
    let args: SearchTemplatesArgs = match serde_json::from_value(arguments) {
        Ok(a) => a,
        Err(e) => {
            return McpResponse::error(
                request_id,
                -32602,
                format!("Invalid search_templates arguments: {}", e),
            );
        }
    };

    // Get all templates, optionally filtered by toolchain
    match template_service::list_templates(server.as_ref(), args.toolchain.as_deref(), None).await {
        Ok(templates) => {
            let query_lower = args.query.to_lowercase();

            // Search in template name, description, and parameter names
            let matching_templates: Vec<_> = templates
                .into_iter()
                .filter(|t| {
                    t.name.to_lowercase().contains(&query_lower)
                        || t.description.to_lowercase().contains(&query_lower)
                        || t.parameters
                            .iter()
                            .any(|p| p.name.to_lowercase().contains(&query_lower))
                })
                .map(|t| {
                    json!({
                        "uri": t.uri,
                        "name": t.name,
                        "description": t.description,
                        "category": t.category,
                        "toolchain": t.toolchain,
                        "relevance": calculate_relevance(&t, &query_lower),
                    })
                })
                .collect();

            let result = json!({
                "content": [{
                    "type": "text",
                    "text": format!("Found {} templates matching '{}'", matching_templates.len(), args.query)
                }],
                "templates": matching_templates,
                "query": args.query,
                "count": matching_templates.len(),
            });

            McpResponse::success(request_id, result)
        }
        Err(e) => {
            error!("Template search failed: {}", e);
            McpResponse::error(request_id, -32000, e.to_string())
        }
    }
}

async fn handle_get_server_info(request_id: serde_json::Value) -> McpResponse {
    let result = json!({
        "content": [{
            "type": "text",
            "text": "PAIML MCP Agent Toolkit - Professional project scaffolding toolkit created by Pragmatic AI Labs"
        }],
        "serverInfo": {
            "name": "paiml-mcp-agent-toolkit",
            "version": env!("CARGO_PKG_VERSION"),
            "vendor": "Pragmatic AI Labs (paiml.com)",
            "author": "Pragmatic AI Labs",
            "description": "Professional project scaffolding toolkit that generates Makefiles, README.md files, and .gitignore files for Rust, Deno, and Python projects. Created by Pragmatic AI Labs to streamline project setup with best practices.",
            "website": "https://paiml.com",
            "capabilities": [
                "Generate individual project files (Makefile, README.md, .gitignore)",
                "Scaffold complete projects with all files at once",
                "Support for Rust CLI/library projects",
                "Support for Deno/TypeScript applications",
                "Support for Python UV projects",
                "Smart subdirectory creation for organized project structure"
            ],
            "supportedTemplates": ["makefile", "readme", "gitignore"],
            "supportedToolchains": ["rust", "deno", "python-uv"],
        }
    });

    McpResponse::success(request_id, result)
}

#[derive(Debug, Deserialize, Serialize)]
struct AnalyzeCodeChurnArgs {
    project_path: Option<String>,
    period_days: Option<u32>,
    format: Option<String>,
}

async fn handle_analyze_code_churn(
    request_id: serde_json::Value,
    arguments: serde_json::Value,
) -> McpResponse {
    let args: AnalyzeCodeChurnArgs = match serde_json::from_value(arguments) {
        Ok(a) => a,
        Err(e) => {
            return McpResponse::error(
                request_id,
                -32602,
                format!("Invalid analyze_code_churn arguments: {}", e),
            );
        }
    };

    let project_path = args
        .project_path
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let period_days = args.period_days.unwrap_or(30);
    let format = args
        .format
        .as_deref()
        .and_then(|f| f.parse::<ChurnOutputFormat>().ok())
        .unwrap_or(ChurnOutputFormat::Summary);

    info!(
        "Analyzing code churn for {:?} over {} days",
        project_path, period_days
    );

    match GitAnalysisService::analyze_code_churn(&project_path, period_days) {
        Ok(analysis) => {
            let content_text = match format {
                ChurnOutputFormat::Json => {
                    serde_json::to_string_pretty(&analysis).unwrap_or_default()
                }
                ChurnOutputFormat::Markdown => format_churn_as_markdown(&analysis),
                ChurnOutputFormat::Csv => format_churn_as_csv(&analysis),
                ChurnOutputFormat::Summary => format_churn_summary(&analysis),
            };

            let result = json!({
                "content": [{
                    "type": "text",
                    "text": content_text
                }],
                "analysis": analysis,
                "format": format!("{:?}", format),
            });

            McpResponse::success(request_id, result)
        }
        Err(e) => {
            error!("Code churn analysis failed: {}", e);
            McpResponse::error(request_id, -32000, e.to_string())
        }
    }
}

pub fn format_churn_summary(analysis: &crate::models::churn::CodeChurnAnalysis) -> String {
    let mut output = String::new();

    output.push_str("# Code Churn Analysis\n\n");
    output.push_str(&format!("Period: {} days\n", analysis.period_days));
    output.push_str(&format!(
        "Total files changed: {}\n",
        analysis.summary.total_files_changed
    ));
    output.push_str(&format!(
        "Total commits: {}\n\n",
        analysis.summary.total_commits
    ));

    if !analysis.summary.hotspot_files.is_empty() {
        output.push_str("## Hotspot Files (High Churn)\n");
        for (i, file) in analysis.summary.hotspot_files.iter().take(5).enumerate() {
            output.push_str(&format!("{}. {}\n", i + 1, file.display()));
        }
        output.push('\n');
    }

    if !analysis.summary.stable_files.is_empty() {
        output.push_str("## Stable Files (Low Churn)\n");
        for (i, file) in analysis.summary.stable_files.iter().take(5).enumerate() {
            output.push_str(&format!("{}. {}\n", i + 1, file.display()));
        }
    }

    output
}

pub fn format_churn_as_markdown(analysis: &crate::models::churn::CodeChurnAnalysis) -> String {
    let mut output = String::new();

    output.push_str("# Code Churn Analysis Report\n\n");
    output.push_str(&format!(
        "**Generated:** {}\n",
        analysis.generated_at.format("%Y-%m-%d %H:%M:%S UTC")
    ));
    output.push_str(&format!(
        "**Repository:** {}\n",
        analysis.repository_root.display()
    ));
    output.push_str(&format!("**Period:** {} days\n\n", analysis.period_days));

    output.push_str("## Summary\n\n");
    output.push_str(&format!(
        "- Total files changed: {}\n",
        analysis.summary.total_files_changed
    ));
    output.push_str(&format!(
        "- Total commits: {}\n",
        analysis.summary.total_commits
    ));
    output.push_str(&format!(
        "- Unique contributors: {}\n\n",
        analysis.summary.author_contributions.len()
    ));

    output.push_str("## Top 10 Files by Churn Score\n\n");
    output.push_str("| File | Commits | Changes | Churn Score | Authors |\n");
    output.push_str("|------|---------|---------|-------------|----------|\n");

    for file in analysis.files.iter().take(10) {
        output.push_str(&format!(
            "| {} | {} | +{} -{}  | {:.2} | {} |\n",
            file.relative_path,
            file.commit_count,
            file.additions,
            file.deletions,
            file.churn_score,
            file.unique_authors.len()
        ));
    }

    output
}

pub fn format_churn_as_csv(analysis: &crate::models::churn::CodeChurnAnalysis) -> String {
    let mut output = String::new();

    output.push_str(
        "file_path,commits,additions,deletions,churn_score,unique_authors,last_modified\n",
    );

    for file in &analysis.files {
        output.push_str(&format!(
            "{},{},{},{},{:.3},{},{}\n",
            file.relative_path,
            file.commit_count,
            file.additions,
            file.deletions,
            file.churn_score,
            file.unique_authors.len(),
            file.last_modified.format("%Y-%m-%d")
        ));
    }

    output
}

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
}

async fn handle_analyze_complexity(
    request_id: serde_json::Value,
    arguments: serde_json::Value,
) -> McpResponse {
    let args: AnalyzeComplexityArgs = match serde_json::from_value(arguments) {
        Ok(a) => a,
        Err(e) => {
            return McpResponse::error(
                request_id,
                -32602,
                format!("Invalid analyze_complexity arguments: {}", e),
            );
        }
    };

    let project_path = args
        .project_path
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    // Detect toolchain if not specified
    let detected_toolchain = if let Some(t) = args.toolchain {
        t
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
    };

    info!(
        "Analyzing complexity for {:?} using {} toolchain",
        project_path, detected_toolchain
    );

    // Import complexity analysis functionality
    use crate::services::complexity::*;
    use walkdir::WalkDir;

    // Custom thresholds
    let mut thresholds = ComplexityThresholds::default();
    if let Some(max) = args.max_cyclomatic {
        thresholds.cyclomatic_error = max;
        thresholds.cyclomatic_warn = (max * 3 / 4).max(1);
    }
    if let Some(max) = args.max_cognitive {
        thresholds.cognitive_error = max;
        thresholds.cognitive_warn = (max * 3 / 4).max(1);
    }

    // Analyze files
    let mut file_metrics = Vec::new();
    let mut file_count = 0;

    for entry in WalkDir::new(&project_path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Skip directories and non-source files
        if path.is_dir() {
            continue;
        }

        // Check file extension based on toolchain
        let should_analyze = match detected_toolchain.as_str() {
            "rust" => path.extension().and_then(|s| s.to_str()) == Some("rs"),
            "deno" => matches!(
                path.extension().and_then(|s| s.to_str()),
                Some("ts") | Some("tsx") | Some("js") | Some("jsx")
            ),
            "python-uv" => path.extension().and_then(|s| s.to_str()) == Some("py"),
            _ => false,
        };

        if !should_analyze {
            continue;
        }

        // Apply include filters if specified (simple pattern matching)
        if let Some(ref include_patterns) = args.include {
            if !include_patterns.is_empty() {
                let path_str = path.to_string_lossy();
                let matches_filter = include_patterns.iter().any(|pattern| {
                    // Simple glob-like matching
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
                });
                if !matches_filter {
                    continue;
                }
            }
        }

        file_count += 1;

        // Analyze file complexity
        match detected_toolchain.as_str() {
            "rust" => {
                use crate::services::ast_rust;
                if let Ok(metrics) = ast_rust::analyze_rust_file_with_complexity(path).await {
                    file_metrics.push(metrics);
                }
            }
            "deno" => {
                use crate::services::ast_typescript;
                if let Ok(metrics) =
                    ast_typescript::analyze_typescript_file_with_complexity(path).await
                {
                    file_metrics.push(metrics);
                }
            }
            "python-uv" => {
                use crate::services::ast_python;
                if let Ok(metrics) = ast_python::analyze_python_file_with_complexity(path).await {
                    file_metrics.push(metrics);
                }
            }
            _ => {}
        }
    }

    // Aggregate results
    let report = aggregate_results(file_metrics);

    // Format output based on requested format
    let format = args.format.as_deref().unwrap_or("summary");
    let content_text = match format {
        "full" => format_complexity_report(&report),
        "json" => serde_json::to_string_pretty(&report).unwrap_or_default(),
        "sarif" => match format_as_sarif(&report) {
            Ok(sarif) => sarif,
            Err(_) => "Error generating SARIF format".to_string(),
        },
        _ => format_complexity_summary(&report), // default to summary
    };

    let result = json!({
        "content": [{
            "type": "text",
            "text": content_text
        }],
        "report": report,
        "toolchain": detected_toolchain,
        "files_analyzed": file_count,
        "format": format,
    });

    McpResponse::success(request_id, result)
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
                format!("Invalid analyze_dag arguments: {}", e),
            );
        }
    };

    let project_path = args
        .project_path
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    // Analyze the project to get AST information
    use crate::cli::DagType;
    use crate::services::{
        context::analyze_project,
        dag_builder::{
            filter_call_edges, filter_import_edges, filter_inheritance_edges, DagBuilder,
        },
        mermaid_generator::{MermaidGenerator, MermaidOptions},
    };

    // We'll analyze as Rust by default, but could be enhanced to detect toolchain
    let project_context = match analyze_project(&project_path, "rust").await {
        Ok(context) => context,
        Err(e) => {
            return McpResponse::error(
                request_id,
                -32000,
                format!("Failed to analyze project: {}", e),
            );
        }
    };

    // Build the dependency graph
    let graph = DagBuilder::build_from_project(&project_context);

    // Parse dag type
    let dag_type = args
        .dag_type
        .as_deref()
        .and_then(|t| match t {
            "call-graph" => Some(DagType::CallGraph),
            "import-graph" => Some(DagType::ImportGraph),
            "inheritance" => Some(DagType::Inheritance),
            "full-dependency" => Some(DagType::FullDependency),
            _ => None,
        })
        .unwrap_or(DagType::CallGraph);

    // Apply filters based on DAG type
    let filtered_graph = match dag_type {
        DagType::CallGraph => filter_call_edges(graph),
        DagType::ImportGraph => filter_import_edges(graph),
        DagType::Inheritance => filter_inheritance_edges(graph),
        DagType::FullDependency => graph,
    };

    // Generate Mermaid output
    let generator = MermaidGenerator::new(MermaidOptions {
        max_depth: args.max_depth,
        filter_external: args.filter_external.unwrap_or(false),
        show_complexity: args.show_complexity.unwrap_or(false),
        ..Default::default()
    });

    let mermaid_output = generator.generate(&filtered_graph);

    // Add stats as comments
    let output_with_stats = format!(
        "{}\n%% Graph Statistics:\n%% Nodes: {}\n%% Edges: {}\n",
        mermaid_output,
        filtered_graph.nodes.len(),
        filtered_graph.edges.len()
    );

    let result = json!({
        "content": [{
            "type": "text",
            "text": output_with_stats
        }],
        "graph_type": format!("{:?}", dag_type),
        "nodes": filtered_graph.nodes.len(),
        "edges": filtered_graph.edges.len(),
    });

    McpResponse::success(request_id, result)
}

#[derive(Debug, Deserialize, Serialize)]
struct GenerateContextArgs {
    toolchain: String,
    project_path: Option<String>,
    format: Option<String>,
}

async fn handle_generate_context(
    request_id: serde_json::Value,
    arguments: serde_json::Value,
) -> McpResponse {
    let args: GenerateContextArgs = match serde_json::from_value(arguments) {
        Ok(a) => a,
        Err(e) => {
            return McpResponse::error(
                request_id,
                -32602,
                format!("Invalid generate_context arguments: {}", e),
            );
        }
    };

    let project_path = args
        .project_path
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    use crate::services::cache::{config::CacheConfig, persistent_manager::PersistentCacheManager};
    use crate::services::context::{
        analyze_project_with_persistent_cache, format_context_as_markdown,
    };

    // Create a persistent cache manager for cross-session caching
    let cache_config = CacheConfig::default();
    let cache_manager = match PersistentCacheManager::with_default_dir(cache_config) {
        Ok(manager) => Arc::new(manager),
        Err(e) => {
            return McpResponse::error(
                request_id,
                -32000,
                format!("Failed to create cache manager: {}", e),
            );
        }
    };

    // Analyze the project with caching
    let context = match analyze_project_with_persistent_cache(
        &project_path,
        &args.toolchain,
        Some(cache_manager.clone()),
    )
    .await
    {
        Ok(ctx) => ctx,
        Err(e) => {
            return McpResponse::error(
                request_id,
                -32000,
                format!("Failed to analyze project: {}", e),
            );
        }
    };

    // Get cache diagnostics
    let diagnostics = cache_manager.get_diagnostics();

    // Format the output
    let format = args.format.as_deref().unwrap_or("markdown");
    let content = match format {
        "json" => serde_json::to_string_pretty(&context).unwrap_or_default(),
        _ => format_context_as_markdown(&context), // default to markdown
    };

    let result = json!({
        "content": [{
            "type": "text",
            "text": content
        }],
        "toolchain": args.toolchain,
        "format": format,
        "cache_diagnostics": {
            "hit_rate": diagnostics.effectiveness.overall_hit_rate,
            "memory_efficiency": diagnostics.effectiveness.memory_efficiency,
            "time_saved_ms": diagnostics.effectiveness.time_saved_ms,
        }
    });

    McpResponse::success(request_id, result)
}
