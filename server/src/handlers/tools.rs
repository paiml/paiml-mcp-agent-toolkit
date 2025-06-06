use crate::models::churn::ChurnOutputFormat;
use crate::models::mcp::{
    GenerateTemplateArgs, ListTemplatesArgs, McpRequest, McpResponse, ScaffoldProjectArgs,
    SearchTemplatesArgs, ToolCallParams, ValidateTemplateArgs,
};
use crate::models::template::{ParameterSpec, TemplateResource};
use crate::services::git_analysis::GitAnalysisService;
use crate::services::template_service;
use crate::TemplateServerTrait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{error, info};

pub async fn handle_tool_call<T: TemplateServerTrait>(
    server: Arc<T>,
    request: McpRequest,
) -> McpResponse {
    let tool_params = match parse_tool_call_params(request.params, &request.id) {
        Ok(params) => params,
        Err(response) => return *response,
    };

    dispatch_tool_call(server, request.id, tool_params).await
}

fn parse_tool_call_params(
    params: Option<serde_json::Value>,
    request_id: &serde_json::Value,
) -> Result<ToolCallParams, Box<McpResponse>> {
    let params = match params {
        Some(p) => p,
        None => {
            return Err(Box::new(McpResponse::error(
                request_id.clone(),
                -32602,
                "Invalid params: missing tool call parameters".to_string(),
            )));
        }
    };

    match serde_json::from_value(params) {
        Ok(p) => Ok(p),
        Err(e) => Err(Box::new(McpResponse::error(
            request_id.clone(),
            -32602,
            format!("Invalid params: {e}"),
        ))),
    }
}

async fn dispatch_tool_call<T: TemplateServerTrait>(
    server: Arc<T>,
    request_id: serde_json::Value,
    tool_params: ToolCallParams,
) -> McpResponse {
    match tool_params.name.as_str() {
        "get_server_info" => handle_get_server_info(request_id).await,
        tool_name if is_template_tool(tool_name) => {
            handle_template_tools(server, request_id, tool_params).await
        }
        tool_name if is_analysis_tool(tool_name) => {
            handle_analysis_tools(request_id, tool_params).await
        }
        _ => McpResponse::error(
            request_id,
            -32602,
            format!("Unknown tool: {}", tool_params.name),
        ),
    }
}

fn is_template_tool(tool_name: &str) -> bool {
    matches!(
        tool_name,
        "generate_template"
            | "list_templates"
            | "validate_template"
            | "scaffold_project"
            | "search_templates"
    )
}

fn is_analysis_tool(tool_name: &str) -> bool {
    matches!(
        tool_name,
        "analyze_code_churn"
            | "analyze_complexity"
            | "analyze_dag"
            | "generate_context"
            | "analyze_system_architecture"
            | "analyze_defect_probability"
            | "analyze_dead_code"
            | "analyze_deep_context"
            | "analyze_tdg"
            | "analyze_makefile_lint"
            | "analyze_provability"
    )
}

async fn handle_template_tools<T: TemplateServerTrait>(
    server: Arc<T>,
    request_id: serde_json::Value,
    tool_params: ToolCallParams,
) -> McpResponse {
    match tool_params.name.as_str() {
        "generate_template" => {
            handle_generate_template(server, request_id, tool_params.arguments).await
        }
        "list_templates" => handle_list_templates(server, request_id, tool_params.arguments).await,
        "validate_template" => {
            handle_validate_template(server, request_id, tool_params.arguments).await
        }
        "scaffold_project" => {
            handle_scaffold_project(server, request_id, tool_params.arguments).await
        }
        "search_templates" => {
            handle_search_templates(server, request_id, tool_params.arguments).await
        }
        _ => McpResponse::error(
            request_id,
            -32602,
            format!("Unsupported template tool: {}", tool_params.name),
        ),
    }
}

async fn handle_analysis_tools(
    request_id: serde_json::Value,
    tool_params: ToolCallParams,
) -> McpResponse {
    match tool_params.name.as_str() {
        "analyze_code_churn" => handle_analyze_code_churn(request_id, tool_params.arguments).await,
        "analyze_complexity" => handle_analyze_complexity(request_id, tool_params.arguments).await,
        "analyze_dag" => handle_analyze_dag(request_id, tool_params.arguments).await,
        "generate_context" => handle_generate_context(request_id, tool_params.arguments).await,
        "analyze_system_architecture" => {
            handle_analyze_system_architecture(request_id, tool_params.arguments).await
        }
        "analyze_defect_probability" => {
            // Deprecated - redirect to TDG analysis
            handle_analyze_tdg(request_id, tool_params.arguments).await
        }
        "analyze_dead_code" => handle_analyze_dead_code(request_id, tool_params.arguments).await,
        "analyze_deep_context" => {
            handle_analyze_deep_context(request_id, tool_params.arguments).await
        }
        "analyze_tdg" => handle_analyze_tdg(request_id, tool_params.arguments).await,
        "analyze_makefile_lint" => {
            handle_analyze_makefile_lint(request_id, Some(tool_params.arguments)).await
        }
        "analyze_provability" => {
            handle_analyze_provability(request_id, Some(tool_params.arguments)).await
        }
        _ => McpResponse::error(
            request_id,
            -32602,
            format!("Unsupported analysis tool: {}", tool_params.name),
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
                format!("Invalid generate_template arguments: {e}")
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
                format!("Invalid list_templates arguments: {e}"),
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
    let args = match parse_validate_template_args(arguments) {
        Ok(args) => args,
        Err(e) => {
            return McpResponse::error(
                request_id,
                -32602,
                format!("Invalid validate_template arguments: {e}"),
            )
        }
    };

    match server.get_template_metadata(&args.resource_uri).await {
        Ok(template_resource) => {
            let validation_result =
                validate_template_parameters(&args.parameters, &template_resource);
            create_validation_response(request_id, validation_result, &args.resource_uri)
        }
        Err(_) => McpResponse::error(
            request_id,
            -32000,
            format!("Template not found: {}", args.resource_uri),
        ),
    }
}

fn parse_validate_template_args(
    arguments: serde_json::Value,
) -> Result<ValidateTemplateArgs, serde_json::Error> {
    serde_json::from_value(arguments)
}

struct ValidationResult {
    missing_required: Vec<String>,
    validation_errors: Vec<String>,
}

fn validate_template_parameters(
    parameters: &serde_json::Map<String, serde_json::Value>,
    template_resource: &TemplateResource,
) -> ValidationResult {
    let missing_required =
        find_missing_required_parameters(parameters, &template_resource.parameters);
    let validation_errors = validate_parameter_values(parameters, &template_resource.parameters);

    ValidationResult {
        missing_required,
        validation_errors,
    }
}

fn find_missing_required_parameters(
    parameters: &serde_json::Map<String, serde_json::Value>,
    parameter_specs: &[ParameterSpec],
) -> Vec<String> {
    parameter_specs
        .iter()
        .filter(|param| param.required && !parameters.contains_key(&param.name))
        .map(|param| param.name.clone())
        .collect()
}

fn validate_parameter_values(
    parameters: &serde_json::Map<String, serde_json::Value>,
    parameter_specs: &[ParameterSpec],
) -> Vec<String> {
    let mut validation_errors = Vec::new();

    for (key, value) in parameters {
        if let Some(param_spec) = parameter_specs.iter().find(|p| p.name == *key) {
            if let Some(error) = validate_single_parameter(key, value, param_spec) {
                validation_errors.push(error);
            }
        } else {
            validation_errors.push(format!("Unknown parameter: {key}"));
        }
    }

    validation_errors
}

fn validate_single_parameter(
    key: &str,
    value: &serde_json::Value,
    param_spec: &ParameterSpec,
) -> Option<String> {
    if let Some(pattern) = &param_spec.validation_pattern {
        if let Ok(regex) = regex::Regex::new(pattern) {
            if let Some(str_val) = value.as_str() {
                if !regex.is_match(str_val) {
                    return Some(format!(
                        "Parameter '{key}' does not match pattern: {pattern}"
                    ));
                }
            }
        }
    }
    None
}

fn create_validation_response(
    request_id: serde_json::Value,
    validation_result: ValidationResult,
    resource_uri: &str,
) -> McpResponse {
    let is_valid = validation_result.missing_required.is_empty()
        && validation_result.validation_errors.is_empty();

    let result = json!({
        "content": [{
            "type": "text",
            "text": if is_valid {
                "Template parameters are valid".to_string()
            } else {
                format!("Validation failed: {} errors",
                    validation_result.missing_required.len() + validation_result.validation_errors.len())
            }
        }],
        "valid": is_valid,
        "missing_required": validation_result.missing_required,
        "validation_errors": validation_result.validation_errors,
        "template_uri": resource_uri,
    });

    McpResponse::success(request_id, result)
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
                format!("Invalid scaffold_project arguments: {e}"),
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

        let full_uri = format!("{uri}{variant}");

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
                format!("Invalid search_templates arguments: {e}"),
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
                format!("Invalid analyze_code_churn arguments: {e}"),
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
    top_files: Option<usize>,
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
                format!("Invalid analyze_complexity arguments: {e}"),
            );
        }
    };

    let project_path = resolve_project_path_complexity(args.project_path.clone());
    let detected_toolchain = detect_toolchain(&args.toolchain, &project_path);

    info!(
        "Analyzing complexity for {:?} using {} toolchain",
        project_path, detected_toolchain
    );

    let _thresholds = build_complexity_thresholds(&args);
    let (file_metrics, file_count) =
        analyze_project_files(&project_path, &detected_toolchain, &args).await;

    // Import complexity analysis functionality
    use crate::services::complexity::*;
    use crate::services::ranking::{rank_files_by_complexity, ComplexityRanker};

    let report = aggregate_results(file_metrics.clone());

    // Handle top_files ranking if requested
    let content_text = if let Some(top_files_count) = args.top_files {
        if top_files_count > 0 {
            let ranker = ComplexityRanker::default();
            let rankings = rank_files_by_complexity(&file_metrics, top_files_count, &ranker);
            format_complexity_rankings(&rankings, &args)
        } else {
            format_complexity_output(&report, &args)
        }
    } else {
        format_complexity_output(&report, &args)
    };

    let result = json!({
        "content": [{
            "type": "text",
            "text": content_text
        }],
        "report": report,
        "toolchain": detected_toolchain,
        "files_analyzed": file_count,
        "format": args.format.as_deref().unwrap_or("summary"),
        "top_files": args.top_files,
    });

    McpResponse::success(request_id, result)
}

fn resolve_project_path_complexity(project_path_arg: Option<String>) -> PathBuf {
    project_path_arg
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
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

    let mut file_metrics = Vec::new();
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
            Some("ts") | Some("tsx") | Some("js") | Some("jsx")
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
                ast_python::analyze_python_file_with_complexity(path)
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
    use crate::services::complexity::*;

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
    match format {
        "json" => {
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
        }
        _ => {
            // Table format (default)
            let mut output = String::new();
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
                format!("Failed to analyze project: {e}"),
            );
        }
    };

    // Build the dependency graph with pruning limit
    let graph = DagBuilder::build_from_project_with_limit(&project_context, 50);

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
    toolchain: Option<String>,
    project_path: Option<String>,
    format: Option<String>,
    debug: Option<bool>,
    debug_output: Option<PathBuf>,
    skip_vendor: Option<bool>,
    max_line_length: Option<usize>,
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
                format!("Invalid generate_context arguments: {e}"),
            );
        }
    };

    let project_path = args
        .project_path
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    info!("Generating comprehensive context for {:?}", project_path);

    // Use the proven deep context analyzer for comprehensive analysis
    use crate::services::deep_context::{DeepContextAnalyzer, DeepContextConfig};
    use crate::services::file_classifier::FileClassifierConfig;

    // Create analyzer and run analysis using proven implementation
    let mut config = DeepContextConfig::default();

    // Configure FileClassifier settings if debug options are provided
    if args.debug.unwrap_or(false)
        || args.skip_vendor.unwrap_or(false)
        || args.max_line_length.is_some()
    {
        let file_classifier_config = FileClassifierConfig {
            skip_vendor: args.skip_vendor.unwrap_or(true),
            max_line_length: args.max_line_length.unwrap_or(10_000),
            max_file_size: 1_048_576, // 1MB default
        };
        config.file_classifier_config = Some(file_classifier_config);
    }

    let analyzer = DeepContextAnalyzer::new(config);

    let deep_context = match analyzer.analyze_project(&project_path).await {
        Ok(ctx) => ctx,
        Err(e) => {
            return McpResponse::error(
                request_id,
                -32000,
                format!("Failed to analyze project: {e}"),
            );
        }
    };

    // Format the output
    let format = args.format.as_deref().unwrap_or("markdown");
    let content = match format {
        "json" => serde_json::to_string_pretty(&deep_context).unwrap_or_default(),
        _ => {
            // Use the comprehensive formatter that includes README and Makefile
            use crate::services::deep_context::{DeepContextAnalyzer, DeepContextConfig};
            let analyzer = DeepContextAnalyzer::new(DeepContextConfig::default());
            analyzer
                .format_as_comprehensive_markdown(&deep_context)
                .await
                .unwrap_or_else(|_| "Error formatting deep context".to_string())
        }
    };

    let result = json!({
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
    });

    McpResponse::success(request_id, result)
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
) -> std::collections::HashMap<String, crate::services::complexity::ComplexityMetrics> {
    use crate::services::complexity::ComplexityMetrics;
    use std::collections::HashMap;

    let mut complexity_map = HashMap::new();

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

async fn handle_analyze_system_architecture(
    request_id: serde_json::Value,
    arguments: serde_json::Value,
) -> McpResponse {
    use crate::services::canonical_query::{
        AnalysisContext, CanonicalQuery, SystemArchitectureQuery,
    };
    use crate::services::deep_context::{DeepContextAnalyzer, DeepContextConfig};

    // Parse arguments
    let args: AnalyzeSystemArchitectureArgs = match serde_json::from_value(arguments) {
        Ok(a) => a,
        Err(e) => {
            return McpResponse::error(
                request_id,
                -32602,
                format!("Invalid analyze_system_architecture arguments: {e}"),
            );
        }
    };

    let project_path = args
        .project_path
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    info!("Analyzing system architecture for {:?}", project_path);

    // Create analyzer config
    let config = DeepContextConfig {
        include_analyses: vec![
            crate::services::deep_context::AnalysisType::Ast,
            crate::services::deep_context::AnalysisType::Complexity,
            crate::services::deep_context::AnalysisType::Dag,
        ],
        ..Default::default()
    };

    // Run analysis
    let analyzer = DeepContextAnalyzer::new(config);
    let deep_context = match analyzer.analyze_project(&project_path).await {
        Ok(ctx) => ctx,
        Err(e) => {
            return McpResponse::error(
                request_id,
                -32000,
                format!("Failed to analyze project: {e}"),
            );
        }
    };

    // Extract dependency graph
    let dag_result = match deep_context.analyses.dependency_graph {
        Some(dag) => dag,
        None => {
            return McpResponse::error(
                request_id,
                -32000,
                "Failed to generate dependency graph".to_string(),
            );
        }
    };

    // Build components for analysis context
    let call_graph = build_call_graph(&dag_result);
    let complexity_map = build_complexity_map(deep_context.analyses.complexity_report.as_ref());

    // Create analysis context
    let context = AnalysisContext {
        project_path: project_path.clone(),
        ast_dag: dag_result,
        call_graph,
        complexity_map,
        churn_analysis: deep_context.analyses.churn_analysis,
    };

    // Execute query
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
                        .map(|r| r.hotspots.len())
                        .unwrap_or(0),
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

#[derive(Debug, Deserialize, Serialize)]
struct AnalyzeDefectProbabilityArgs {
    project_path: Option<String>,
    format: Option<String>,
}

#[allow(dead_code)]
async fn handle_analyze_defect_probability(
    request_id: serde_json::Value,
    arguments: serde_json::Value,
) -> McpResponse {
    let args: AnalyzeDefectProbabilityArgs = match serde_json::from_value(arguments) {
        Ok(a) => a,
        Err(e) => {
            return McpResponse::error(
                request_id,
                -32602,
                format!("Invalid analyze_defect_probability arguments: {e}"),
            );
        }
    };

    let project_path = args
        .project_path
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    info!("Analyzing defect probability for {:?}", project_path);

    use crate::services::defect_probability::{
        DefectProbabilityCalculator, FileMetrics, ProjectDefectAnalysis,
    };
    use crate::services::file_discovery::ProjectFileDiscovery;

    let calculator = DefectProbabilityCalculator::new();
    let mut file_metrics = Vec::new();

    // Use ProjectFileDiscovery which properly respects .gitignore files
    let discovery = ProjectFileDiscovery::new(project_path.clone());
    let discovered_files = match discovery.discover_files() {
        Ok(files) => files,
        Err(e) => {
            error!("Failed to discover files: {}", e);
            return McpResponse::error(
                request_id,
                -32603,
                format!("Failed to discover files: {e}"),
            );
        }
    };

    for path in discovered_files {
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rs") {
            let relative_path = path
                .strip_prefix(&project_path)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();

            // Simple metrics for demonstration
            let metrics = FileMetrics {
                file_path: relative_path,
                churn_score: 0.1, // Placeholder
                complexity: 5.0,  // Placeholder
                duplicate_ratio: 0.0,
                afferent_coupling: 1.0,
                efferent_coupling: 2.0,
                lines_of_code: 100, // Placeholder
                cyclomatic_complexity: 5,
                cognitive_complexity: 8,
            };

            file_metrics.push(metrics);
        }
    }

    let scores = calculator.calculate_batch(&file_metrics);
    let analysis = ProjectDefectAnalysis::from_scores(scores);

    let content_text = match args.format.as_deref() {
        Some("json") => serde_json::to_string_pretty(&analysis).unwrap_or_default(),
        _ => format!(
            "# Defect Probability Analysis\n\nTotal files: {}\nHigh-risk files: {}\nMedium-risk files: {}\nAverage probability: {:.2}",
            analysis.total_files,
            analysis.high_risk_files.len(),
            analysis.medium_risk_files.len(),
            analysis.average_probability
        ),
    };

    let result = json!({
        "content": [{
            "type": "text",
            "text": content_text
        }],
        "analysis": analysis,
        "format": args.format.unwrap_or_else(|| "summary".to_string()),
    });

    McpResponse::success(request_id, result)
}

#[derive(Debug, Deserialize, Serialize)]
struct AnalyzeDeadCodeArgs {
    project_path: Option<String>,
    format: Option<String>,
    top_files: Option<usize>,
    include_unreachable: Option<bool>,
    min_dead_lines: Option<usize>,
    include_tests: Option<bool>,
}

async fn handle_analyze_dead_code(
    request_id: serde_json::Value,
    arguments: serde_json::Value,
) -> McpResponse {
    let args: AnalyzeDeadCodeArgs = match serde_json::from_value(arguments) {
        Ok(a) => a,
        Err(e) => {
            return McpResponse::error(
                request_id,
                -32602,
                format!("Invalid analyze_dead_code arguments: {e}"),
            );
        }
    };

    let project_path = args
        .project_path
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    info!("Analyzing dead code for {:?}", project_path);

    use crate::models::dead_code::DeadCodeAnalysisConfig;
    use crate::services::dead_code_analyzer::DeadCodeAnalyzer;

    // Create analyzer with a reasonable capacity
    let mut analyzer = DeadCodeAnalyzer::new(10000);

    // Configure analysis
    let config = DeadCodeAnalysisConfig {
        include_unreachable: args.include_unreachable.unwrap_or(false),
        include_tests: args.include_tests.unwrap_or(false),
        min_dead_lines: args.min_dead_lines.unwrap_or(10),
    };

    // Run analysis with ranking
    let mut result = match analyzer.analyze_with_ranking(&project_path, config).await {
        Ok(r) => r,
        Err(e) => {
            return McpResponse::error(
                request_id,
                -32000,
                format!("Dead code analysis failed: {e}"),
            );
        }
    };

    // Apply top_files limit if specified
    if let Some(limit) = args.top_files {
        result.ranked_files.truncate(limit);
    }

    // Format output based on requested format
    let format = args.format.as_deref().unwrap_or("summary");
    let content_text = match format_dead_code_output(&result, format) {
        Ok(content) => content,
        Err(e) => {
            return McpResponse::error(request_id, -32000, format!("Failed to format output: {e}"));
        }
    };

    let response = json!({
        "content": [{
            "type": "text",
            "text": content_text
        }],
        "result": result,
        "format": format,
        "files_analyzed": result.summary.total_files_analyzed,
        "files_with_dead_code": result.summary.files_with_dead_code,
        "total_dead_lines": result.summary.total_dead_lines,
        "dead_percentage": result.summary.dead_percentage,
    });

    McpResponse::success(request_id, response)
}

/// Format dead code analysis output for MCP response
fn format_dead_code_output(
    result: &crate::models::dead_code::DeadCodeRankingResult,
    format: &str,
) -> anyhow::Result<String> {
    use crate::cli::DeadCodeOutputFormat;

    let output_format = match format {
        "summary" => DeadCodeOutputFormat::Summary,
        "json" => DeadCodeOutputFormat::Json,
        "sarif" => DeadCodeOutputFormat::Sarif,
        "markdown" => DeadCodeOutputFormat::Markdown,
        _ => DeadCodeOutputFormat::Summary,
    };

    // Use the existing formatting functions from CLI module
    match output_format {
        DeadCodeOutputFormat::Summary => {
            // Import the function from the CLI module
            format_dead_code_summary_mcp(result)
        }
        DeadCodeOutputFormat::Json => Ok(serde_json::to_string_pretty(result)?),
        DeadCodeOutputFormat::Sarif => format_dead_code_as_sarif_mcp(result),
        DeadCodeOutputFormat::Markdown => format_dead_code_as_markdown_mcp(result),
    }
}

/// Format dead code analysis as summary text for MCP
fn format_dead_code_summary_mcp(
    result: &crate::models::dead_code::DeadCodeRankingResult,
) -> anyhow::Result<String> {
    let mut output = String::new();

    output.push_str("# Dead Code Analysis Summary\n\n");
    output.push_str(&format!(
        "**Total files analyzed:** {}\n",
        result.summary.total_files_analyzed
    ));
    output.push_str(&format!(
        "**Files with dead code:** {} ({:.1}%)\n",
        result.summary.files_with_dead_code,
        if result.summary.total_files_analyzed > 0 {
            (result.summary.files_with_dead_code as f32
                / result.summary.total_files_analyzed as f32)
                * 100.0
        } else {
            0.0
        }
    ));
    output.push_str(&format!(
        "**Total dead lines:** {} ({:.1}% of codebase)\n",
        result.summary.total_dead_lines, result.summary.dead_percentage
    ));
    output.push_str(&format!(
        "**Dead functions:** {}\n",
        result.summary.dead_functions
    ));
    output.push_str(&format!(
        "**Dead classes:** {}\n",
        result.summary.dead_classes
    ));
    output.push_str(&format!(
        "**Dead modules:** {}\n",
        result.summary.dead_modules
    ));
    output.push_str(&format!(
        "**Unreachable blocks:** {}\n\n",
        result.summary.unreachable_blocks
    ));

    // Show top files if available
    if !result.ranked_files.is_empty() {
        let top_count = result.ranked_files.len().min(5);
        output.push_str(&format!("## Top {top_count} Files with Most Dead Code\n\n"));

        for (i, file_metrics) in result.ranked_files.iter().take(top_count).enumerate() {
            let confidence_text = match file_metrics.confidence {
                crate::models::dead_code::ConfidenceLevel::High => "HIGH",
                crate::models::dead_code::ConfidenceLevel::Medium => "MEDIUM",
                crate::models::dead_code::ConfidenceLevel::Low => "LOW",
            };

            output.push_str(&format!(
                "{}. **{}** (Score: {:.1}) [{}confidence]\n",
                i + 1,
                file_metrics.path,
                file_metrics.dead_score,
                confidence_text
            ));
            output.push_str(&format!(
                "   - {} dead lines ({:.1}% of file)\n",
                file_metrics.dead_lines, file_metrics.dead_percentage
            ));
            if file_metrics.dead_functions > 0 || file_metrics.dead_classes > 0 {
                output.push_str(&format!(
                    "   - {} functions, {} classes\n",
                    file_metrics.dead_functions, file_metrics.dead_classes
                ));
            }
            output.push('\n');
        }
    }

    Ok(output)
}

/// Format dead code analysis as SARIF for MCP
fn format_dead_code_as_sarif_mcp(
    result: &crate::models::dead_code::DeadCodeRankingResult,
) -> anyhow::Result<String> {
    use serde_json::json;

    let mut results = Vec::new();

    for file_metrics in &result.ranked_files {
        for item in &file_metrics.items {
            results.push(json!({
                "ruleId": format!("dead-code-{}", format!("{:?}", item.item_type).to_lowercase()),
                "level": "info",
                "message": {
                    "text": format!("Dead {} '{}': {}",
                        format!("{:?}", item.item_type).to_lowercase(),
                        item.name,
                        item.reason
                    )
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": file_metrics.path
                        },
                        "region": {
                            "startLine": item.line
                        }
                    }
                }]
            }));
        }
    }

    let sarif = json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "paiml-mcp-agent-toolkit",
                    "version": "0.1.0",
                    "informationUri": "https://github.com/paiml/mcp-agent-toolkit"
                }
            },
            "results": results
        }]
    });

    Ok(serde_json::to_string_pretty(&sarif)?)
}

/// Format dead code analysis as Markdown for MCP
fn format_dead_code_as_markdown_mcp(
    result: &crate::models::dead_code::DeadCodeRankingResult,
) -> anyhow::Result<String> {
    let mut output = String::new();

    output.push_str("# Dead Code Analysis Report\n\n");
    output.push_str(&format!(
        "**Analysis Date:** {}\n\n",
        result.analysis_timestamp.format("%Y-%m-%d %H:%M:%S UTC")
    ));

    // Summary section
    output.push_str("## Summary\n\n");
    output.push_str(&format!(
        "- **Total files analyzed:** {}\n",
        result.summary.total_files_analyzed
    ));
    output.push_str(&format!(
        "- **Files with dead code:** {} ({:.1}%)\n",
        result.summary.files_with_dead_code,
        if result.summary.total_files_analyzed > 0 {
            (result.summary.files_with_dead_code as f32
                / result.summary.total_files_analyzed as f32)
                * 100.0
        } else {
            0.0
        }
    ));
    output.push_str(&format!(
        "- **Total dead lines:** {} ({:.1}% of codebase)\n",
        result.summary.total_dead_lines, result.summary.dead_percentage
    ));
    output.push_str(&format!(
        "- **Dead functions:** {}\n",
        result.summary.dead_functions
    ));
    output.push_str(&format!(
        "- **Dead classes:** {}\n",
        result.summary.dead_classes
    ));
    output.push_str(&format!(
        "- **Dead modules:** {}\n",
        result.summary.dead_modules
    ));
    output.push_str(&format!(
        "- **Unreachable blocks:** {}\n\n",
        result.summary.unreachable_blocks
    ));

    // Top files section
    if !result.ranked_files.is_empty() {
        output.push_str("## Top Files with Dead Code\n\n");
        output.push_str("| Rank | File | Dead Lines | Percentage | Functions | Classes | Score | Confidence |\n");
        output.push_str("|------|------|------------|------------|-----------|---------|-------|------------|\n");

        for (i, file_metrics) in result.ranked_files.iter().enumerate() {
            let confidence_text = match file_metrics.confidence {
                crate::models::dead_code::ConfidenceLevel::High => "🔴 High",
                crate::models::dead_code::ConfidenceLevel::Medium => "🟡 Medium",
                crate::models::dead_code::ConfidenceLevel::Low => "🟢 Low",
            };

            output.push_str(&format!(
                "| {:>4} | `{}` | {:>10} | {:>9.1}% | {:>9} | {:>7} | {:>5.1} | {} |\n",
                i + 1,
                file_metrics.path,
                file_metrics.dead_lines,
                file_metrics.dead_percentage,
                file_metrics.dead_functions,
                file_metrics.dead_classes,
                file_metrics.dead_score,
                confidence_text
            ));
        }
        output.push('\n');
    }

    Ok(output)
}

#[derive(Debug, Deserialize, Serialize)]
struct AnalyzeTdgArgs {
    project_path: Option<String>,
    format: Option<String>,
    threshold: Option<f64>,
    include_components: Option<bool>,
    max_results: Option<usize>,
}

async fn handle_analyze_tdg(
    request_id: serde_json::Value,
    arguments: serde_json::Value,
) -> McpResponse {
    let args: AnalyzeTdgArgs = match serde_json::from_value(arguments) {
        Ok(a) => a,
        Err(e) => {
            return McpResponse::error(
                request_id,
                -32602,
                format!("Invalid analyze_tdg arguments: {e}"),
            );
        }
    };

    let project_path = args
        .project_path
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    info!("Analyzing Technical Debt Gradient for {:?}", project_path);

    use crate::services::tdg_calculator::TDGCalculator;

    // Create TDG calculator (primary service for analysis)
    let calculator = TDGCalculator::new();

    // Run TDG analysis
    let analysis = match calculator.analyze_directory(&project_path).await {
        Ok(analysis) => analysis,
        Err(e) => {
            return McpResponse::error(request_id, -32000, format!("TDG analysis failed: {e}"));
        }
    };

    // Format output
    let content_text = match args.format.as_deref() {
        Some("json") => serde_json::to_string_pretty(&analysis).unwrap_or_default(),
        _ => format_tdg_summary(&analysis),
    };

    let result = json!({
        "content": [{
            "type": "text",
            "text": content_text
        }],
        "analysis": analysis,
        "format": args.format.unwrap_or_else(|| "summary".to_string()),
    });

    McpResponse::success(request_id, result)
}

fn format_tdg_summary(summary: &crate::models::tdg::TDGSummary) -> String {
    let mut output = String::new();

    output.push_str("# Technical Debt Gradient Analysis\n\n");

    // Summary
    output.push_str("## Summary\n\n");
    output.push_str(&format!("**Total files:** {}\n", summary.total_files));
    output.push_str(&format!(
        "**Critical files:** {} ({:.1}%)\n",
        summary.critical_files,
        if summary.total_files > 0 {
            (summary.critical_files as f64 / summary.total_files as f64) * 100.0
        } else {
            0.0
        }
    ));
    output.push_str(&format!(
        "**Warning files:** {} ({:.1}%)\n",
        summary.warning_files,
        if summary.total_files > 0 {
            (summary.warning_files as f64 / summary.total_files as f64) * 100.0
        } else {
            0.0
        }
    ));
    output.push_str(&format!("**Average TDG:** {:.2}\n", summary.average_tdg));
    output.push_str(&format!(
        "**95th percentile TDG:** {:.2}\n",
        summary.p95_tdg
    ));
    output.push_str(&format!(
        "**99th percentile TDG:** {:.2}\n",
        summary.p99_tdg
    ));
    output.push_str(&format!(
        "**Estimated technical debt:** {:.0} hours\n\n",
        summary.estimated_debt_hours
    ));

    // Hotspots
    if !summary.hotspots.is_empty() {
        output.push_str("## Top Hotspots\n\n");
        output.push_str("| File | TDG Score | Primary Factor | Estimated Hours |\n");
        output.push_str("|------|-----------|----------------|----------------|\n");

        for hotspot in &summary.hotspots {
            output.push_str(&format!(
                "| {} | {:.2} | {} | {:.0} |\n",
                hotspot.path, hotspot.tdg_score, hotspot.primary_factor, hotspot.estimated_hours
            ));
        }
        output.push('\n');
    }

    output.push_str("## Severity Distribution\n\n");
    output.push_str(&format!(
        "- 🔴 Critical (>2.5): {} files\n",
        summary.critical_files
    ));
    output.push_str(&format!(
        "- 🟡 Warning (1.5-2.5): {} files\n",
        summary.warning_files
    ));
    output.push_str(&format!(
        "- 🟢 Normal (<1.5): {} files\n",
        summary.total_files - summary.critical_files - summary.warning_files
    ));

    output
}

#[derive(Debug, Deserialize, Serialize)]
struct AnalyzeDeepContextArgs {
    project_path: Option<String>,
    format: Option<String>,
    include_analyses: Option<Vec<String>>,
    exclude_analyses: Option<Vec<String>>,
    period_days: Option<u32>,
    dag_type: Option<String>,
    max_depth: Option<usize>,
    include_pattern: Option<Vec<String>>,
    exclude_pattern: Option<Vec<String>>,
    cache_strategy: Option<String>,
    parallel: Option<usize>,
}

async fn handle_analyze_deep_context(
    request_id: serde_json::Value,
    arguments: serde_json::Value,
) -> McpResponse {
    let args = match parse_deep_context_args(arguments) {
        Ok(args) => args,
        Err(e) => return McpResponse::error(request_id, -32602, e),
    };

    let project_path = resolve_project_path(args.project_path.clone());
    info!("Running deep context analysis for {:?}", project_path);

    let config = build_deep_context_config(&args);
    let analyzer = create_deep_context_analyzer(config);

    match analyzer.analyze_project(&project_path).await {
        Ok(context) => {
            let result = format_deep_context_response(&context, &args);
            McpResponse::success(request_id, result)
        }
        Err(e) => {
            error!("Deep context analysis failed: {}", e);
            McpResponse::error(request_id, -32000, e.to_string())
        }
    }
}

fn parse_deep_context_args(arguments: serde_json::Value) -> Result<AnalyzeDeepContextArgs, String> {
    serde_json::from_value(arguments)
        .map_err(|e| format!("Invalid analyze_deep_context arguments: {e}"))
}

fn resolve_project_path(project_path: Option<String>) -> PathBuf {
    project_path
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

fn parse_analysis_types(
    include_analyses: Option<Vec<String>>,
) -> Vec<crate::services::deep_context::AnalysisType> {
    use crate::services::deep_context::AnalysisType;

    if let Some(analyses) = include_analyses {
        analyses
            .into_iter()
            .filter_map(|s| match s.as_str() {
                "ast" => Some(AnalysisType::Ast),
                "complexity" => Some(AnalysisType::Complexity),
                "churn" => Some(AnalysisType::Churn),
                "dag" => Some(AnalysisType::Dag),
                "dead_code" => Some(AnalysisType::DeadCode),
                "satd" => Some(AnalysisType::Satd),
                "tdg" => Some(AnalysisType::TechnicalDebtGradient),
                _ => None,
            })
            .collect()
    } else {
        vec![
            AnalysisType::Ast,
            AnalysisType::Complexity,
            AnalysisType::Churn,
        ]
    }
}

fn parse_dag_type(dag_type: Option<String>) -> crate::services::deep_context::DagType {
    use crate::services::deep_context::DagType;

    match dag_type.as_deref() {
        Some("import-graph") => DagType::ImportGraph,
        Some("inheritance") => DagType::Inheritance,
        Some("full-dependency") => DagType::FullDependency,
        Some("call-graph") | None => DagType::CallGraph,
        _ => DagType::CallGraph,
    }
}

fn parse_cache_strategy(
    cache_strategy: Option<String>,
) -> crate::services::deep_context::CacheStrategy {
    use crate::services::deep_context::CacheStrategy;

    match cache_strategy.as_deref() {
        Some("force-refresh") => CacheStrategy::ForceRefresh,
        Some("offline") => CacheStrategy::Offline,
        Some("normal") | None => CacheStrategy::Normal,
        _ => CacheStrategy::Normal,
    }
}

fn build_deep_context_config(
    args: &AnalyzeDeepContextArgs,
) -> crate::services::deep_context::DeepContextConfig {
    use crate::services::deep_context::{ComplexityThresholds, DeepContextConfig};

    DeepContextConfig {
        include_analyses: parse_analysis_types(args.include_analyses.clone()),
        period_days: args.period_days.unwrap_or(30),
        dag_type: parse_dag_type(args.dag_type.clone()),
        complexity_thresholds: Some(ComplexityThresholds {
            max_cyclomatic: 10,
            max_cognitive: 15,
        }),
        max_depth: args.max_depth,
        include_patterns: args.include_pattern.clone().unwrap_or_default(),
        exclude_patterns: args.exclude_pattern.clone().unwrap_or_default(),
        cache_strategy: parse_cache_strategy(args.cache_strategy.clone()),
        parallel: args.parallel.unwrap_or(4),
        file_classifier_config: None, // Default to None for deep context analysis
    }
}

fn create_deep_context_analyzer(
    config: crate::services::deep_context::DeepContextConfig,
) -> crate::services::deep_context::DeepContextAnalyzer {
    crate::services::deep_context::DeepContextAnalyzer::new(config)
}

fn format_deep_context_response(
    context: &crate::services::deep_context::DeepContext,
    args: &AnalyzeDeepContextArgs,
) -> serde_json::Value {
    let format = args.format.as_deref().unwrap_or("markdown");
    let content_text = match format {
        "json" => serde_json::to_string_pretty(context).unwrap_or_default(),
        "sarif" => format_deep_context_as_sarif(context),
        _ => {
            // Note: This is a sync context, so we can't easily use async here
            // The format_deep_context_as_markdown function has been updated to include
            // README and Makefile metadata when available
            format_deep_context_as_markdown(context)
        }
    };

    json!({
        "content": [{
            "type": "text",
            "text": content_text
        }],
        "context": context,
        "format": format!("{:?}", format),
        "analysis_duration_ms": context.metadata.analysis_duration.as_millis(),
    })
}

fn format_deep_context_as_sarif(_context: &crate::services::deep_context::DeepContext) -> String {
    // Simple SARIF implementation for MCP
    use serde_json::json;

    let sarif = json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "paiml-mcp-agent-toolkit",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/paiml/mcp-agent-toolkit"
                }
            },
            "results": []
        }]
    });

    serde_json::to_string_pretty(&sarif).unwrap_or_default()
}

fn format_deep_context_as_markdown(context: &crate::services::deep_context::DeepContext) -> String {
    use crate::cli::formatting_helpers::*;

    let mut output = String::new();
    output.push_str("# Deep Context Analysis\n\n");

    // Reuse helper functions from cli module
    output.push_str(&format_executive_summary(context));

    // Essential Project Metadata (README and Makefile)
    if context.project_overview.is_some() || context.build_info.is_some() {
        output.push_str("\n## Essential Project Metadata\n\n");

        if let Some(ref overview) = context.project_overview {
            output.push_str(&format_project_overview(overview));
        }

        if let Some(ref build_info) = context.build_info {
            output.push_str(&format_build_info(build_info));
        }
    }

    // Quality Scorecard and other sections
    output.push_str(&format_quality_scorecard(context));
    output.push_str(&format_defect_summary(context));
    output.push_str(&format_recommendations(context));

    // Continue with remaining sections specific to this function
    output.push_str("\n## Analysis Results\n\n");
    output.push_str(&format!(
        "**Total Defects:** {}\n",
        context.defect_summary.total_defects
    ));
    output.push_str(&format!(
        "**Defect Density:** {:.2}\n",
        context.defect_summary.defect_density
    ));

    // Show defects by type
    if !context.defect_summary.by_type.is_empty() {
        output.push_str("**By Type:**\n");
        for (defect_type, count) in &context.defect_summary.by_type {
            output.push_str(&format!("- {defect_type}: {count}\n"));
        }
    }

    // Show defects by severity
    if !context.defect_summary.by_severity.is_empty() {
        output.push_str("**By Severity:**\n");
        for (severity, count) in &context.defect_summary.by_severity {
            output.push_str(&format!("- {severity}: {count}\n"));
        }
    }
    output.push_str(&format!(
        "**Total Files:** {}\n\n",
        context.file_tree.total_files
    ));

    // Recommendations
    if !context.recommendations.is_empty() {
        output.push_str("## Recommendations\n\n");
        for (i, rec) in context.recommendations.iter().take(5).enumerate() {
            output.push_str(&format!(
                "{}. **{}** (Priority: {:?})\n",
                i + 1,
                rec.title,
                rec.priority
            ));
            output.push_str(&format!("   {}\n\n", rec.description));
        }
    }

    output
}

async fn handle_analyze_makefile_lint(
    request_id: serde_json::Value,
    arguments: Option<serde_json::Value>,
) -> McpResponse {
    #[derive(Deserialize)]
    struct MakefileLintArgs {
        path: String,
        #[serde(default)]
        rules: Vec<String>,
        #[serde(default)]
        #[allow(dead_code)]
        fix: bool,
        #[serde(default)]
        #[allow(dead_code)]
        gnu_version: String,
    }

    let args: MakefileLintArgs = match arguments {
        Some(args) => match serde_json::from_value(args) {
            Ok(args) => args,
            Err(e) => {
                return McpResponse::error(
                    request_id,
                    -32602,
                    format!("Invalid analyze_makefile_lint arguments: {e}"),
                );
            }
        },
        None => {
            return McpResponse::error(
                request_id,
                -32602,
                "Missing required arguments for analyze_makefile_lint".to_string(),
            );
        }
    };

    let makefile_path = std::path::Path::new(&args.path);

    info!("Analyzing Makefile at {:?}", makefile_path);

    // Use the existing makefile linter service
    use crate::services::makefile_linter;

    let lint_result = match makefile_linter::lint_makefile(makefile_path).await {
        Ok(result) => result,
        Err(e) => {
            return McpResponse::error(request_id, -32000, format!("Makefile linting failed: {e}"));
        }
    };

    let analysis = json!({
        "path": args.path,
        "violations": lint_result.violations.iter().map(|v| json!({
            "rule": v.rule,
            "severity": match v.severity {
                makefile_linter::Severity::Error => "error",
                makefile_linter::Severity::Warning => "warning",
                makefile_linter::Severity::Performance => "performance",
                makefile_linter::Severity::Info => "info",
            },
            "line": v.span.line,
            "column": v.span.column,
            "message": v.message,
            "fix_hint": v.fix_hint,
        })).collect::<Vec<_>>(),
        "quality_score": lint_result.quality_score,
        "rules_applied": args.rules,
        "total_violations": lint_result.violations.len(),
        "error_count": lint_result.violations.iter().filter(|v| matches!(v.severity, makefile_linter::Severity::Error)).count(),
        "warning_count": lint_result.violations.iter().filter(|v| matches!(v.severity, makefile_linter::Severity::Warning)).count(),
    });

    McpResponse::success(request_id, analysis)
}

async fn handle_analyze_provability(
    request_id: serde_json::Value,
    arguments: Option<serde_json::Value>,
) -> McpResponse {
    #[derive(Deserialize)]
    struct ProvabilityArgs {
        project_path: String,
        #[serde(default)]
        functions: Option<Vec<String>>,
        #[serde(default)]
        analysis_depth: Option<usize>,
    }

    let args: ProvabilityArgs = match arguments {
        Some(args) => match serde_json::from_value(args) {
            Ok(args) => args,
            Err(e) => {
                return McpResponse::error(
                    request_id,
                    -32602,
                    format!("Invalid analyze_provability arguments: {e}"),
                );
            }
        },
        None => {
            return McpResponse::error(
                request_id,
                -32602,
                "Missing required arguments for analyze_provability".to_string(),
            );
        }
    };

    info!("Analyzing provability for project: {:?}", args.project_path);

    // Use the existing provability analyzer service
    use crate::services::lightweight_provability_analyzer::{
        FunctionId, LightweightProvabilityAnalyzer,
    };

    let analyzer = LightweightProvabilityAnalyzer::new();

    // Extract functions from parameters or mock discovery from project path
    let functions = if let Some(function_names) = args.functions {
        function_names
            .into_iter()
            .enumerate()
            .map(|(i, name)| FunctionId {
                file_path: format!("{}/src/lib.rs", args.project_path),
                function_name: name,
                line_number: i * 10, // Mock line numbers
            })
            .collect()
    } else {
        // Mock function discovery from project path
        vec![FunctionId {
            file_path: format!("{}/src/main.rs", args.project_path),
            function_name: "main".to_string(),
            line_number: 1,
        }]
    };

    let summaries = analyzer.analyze_incrementally(&functions).await;

    let average_score = if summaries.is_empty() {
        0.0
    } else {
        summaries.iter().map(|s| s.provability_score).sum::<f64>() / summaries.len() as f64
    };

    let analysis = json!({
        "project_path": args.project_path,
        "analysis_depth": args.analysis_depth.unwrap_or(10),
        "functions_analyzed": summaries.len(),
        "average_provability_score": average_score,
        "summaries": summaries.iter().map(|s| json!({
            "function_id": format!("{}:{}", s.version, "main"), // Mock function ID
            "provability_score": s.provability_score,
            "verified_properties": s.verified_properties,
            "analysis_time_us": s.analysis_time_us,
        })).collect::<Vec<_>>(),
        "confidence_breakdown": {
            "high_confidence": summaries.iter().filter(|s| s.provability_score > 0.8).count(),
            "medium_confidence": summaries.iter().filter(|s| s.provability_score > 0.5 && s.provability_score <= 0.8).count(),
            "low_confidence": summaries.iter().filter(|s| s.provability_score <= 0.5).count(),
        }
    });

    McpResponse::success(request_id, analysis)
}
