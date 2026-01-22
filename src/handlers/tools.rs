use crate::models::churn::ChurnOutputFormat;

// Import handlers from extracted module (CB-040)
use crate::handlers::tools_advanced::{
    handle_analyze_dead_code, handle_analyze_deep_context, handle_analyze_lint_hotspot,
    handle_analyze_makefile_lint, handle_analyze_provability, handle_analyze_satd,
    handle_analyze_tdg, handle_quality_driven_development,
};
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
        tool_name if super::vectorized_tools::is_vectorized_tool(tool_name) => {
            super::vectorized_tools::handle_vectorized_tools(request_id, tool_params).await
        }
        _ => McpResponse::error(
            request_id,
            -32602,
            format!("Unknown tool: {}", tool_params.name),
        ),
    }
}

/// Check if a tool name is a template tool
pub fn is_template_tool(tool_name: &str) -> bool {
    matches!(
        tool_name,
        "generate_template"
            | "list_templates"
            | "validate_template"
            | "scaffold_project"
            | "search_templates"
    )
}

/// Check if a tool name is an analysis tool
pub fn is_analysis_tool(tool_name: &str) -> bool {
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
            | "analyze_satd"
            | "quality_driven_development"
            | "analyze_lint_hotspot"
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
    dispatch_analysis_tool(request_id, &tool_params.name, tool_params.arguments).await
}

/// Toyota Way: Extract Method - Dispatch analysis tools with grouped handling (complexity ≤8)
async fn dispatch_analysis_tool(
    request_id: serde_json::Value,
    tool_name: &str,
    arguments: serde_json::Value,
) -> McpResponse {
    // Group 1: Core analysis tools
    if let Some(response) =
        handle_core_analysis_tools(request_id.clone(), tool_name, arguments.clone()).await
    {
        return response;
    }

    // Group 2: Advanced analysis tools
    if let Some(response) =
        handle_advanced_analysis_tools(request_id.clone(), tool_name, arguments.clone()).await
    {
        return response;
    }

    // Group 3: Specialized analysis tools
    if let Some(response) =
        handle_specialized_analysis_tools(request_id.clone(), tool_name, arguments).await
    {
        return response;
    }

    // Unknown tool
    McpResponse::error(
        request_id,
        -32602,
        format!("Unsupported analysis tool: {tool_name}"),
    )
}

/// Toyota Way: Extract Method - Handle core analysis tools (complexity ≤5)
async fn handle_core_analysis_tools(
    request_id: serde_json::Value,
    tool_name: &str,
    arguments: serde_json::Value,
) -> Option<McpResponse> {
    match tool_name {
        "analyze_complexity" => Some(handle_analyze_complexity(request_id, arguments).await),
        "analyze_dead_code" => Some(handle_analyze_dead_code(request_id, arguments).await),
        "analyze_satd" => Some(handle_analyze_satd(request_id, arguments).await),
        "analyze_tdg" => Some(handle_analyze_tdg(request_id, arguments).await),
        _ => None,
    }
}

/// Toyota Way: Extract Method - Handle advanced analysis tools (complexity ≤5)
async fn handle_advanced_analysis_tools(
    request_id: serde_json::Value,
    tool_name: &str,
    arguments: serde_json::Value,
) -> Option<McpResponse> {
    match tool_name {
        "analyze_code_churn" => Some(handle_analyze_code_churn(request_id, arguments).await),
        "analyze_dag" => Some(handle_analyze_dag(request_id, arguments).await),
        "generate_context" => Some(handle_generate_context(request_id, arguments).await),
        "analyze_deep_context" => Some(handle_analyze_deep_context(request_id, arguments).await),
        "analyze_defect_probability" => {
            // Deprecated - redirect to TDG analysis
            Some(handle_analyze_tdg(request_id, arguments).await)
        }
        _ => None,
    }
}

/// Toyota Way: Extract Method - Handle specialized analysis tools (complexity ≤5)
async fn handle_specialized_analysis_tools(
    request_id: serde_json::Value,
    tool_name: &str,
    arguments: serde_json::Value,
) -> Option<McpResponse> {
    match tool_name {
        "analyze_system_architecture" => {
            Some(handle_analyze_system_architecture(request_id, arguments).await)
        }
        "analyze_makefile_lint" => {
            Some(handle_analyze_makefile_lint(request_id, Some(arguments)).await)
        }
        "analyze_provability" => {
            Some(handle_analyze_provability(request_id, Some(arguments)).await)
        }
        "analyze_lint_hotspot" => Some(handle_analyze_lint_hotspot(request_id, arguments).await),
        "quality_driven_development" => {
            Some(handle_quality_driven_development(request_id, arguments).await)
        }
        _ => None,
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
    let mut validation_errors = Vec::with_capacity(256);

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

// Helper to determine template variant
fn get_template_variant(template_type: &str, toolchain: &str) -> Option<&'static str> {
    match template_type {
        "makefile" | "readme" | "gitignore" => match toolchain {
            "rust" | "deno" | "python-uv" => Some("cli"),
            _ => None,
        },
        _ => None,
    }
}

// Helper to generate a single template
async fn generate_single_template<T: TemplateServerTrait>(
    server: &T,
    template_type: &str,
    toolchain: &str,
    parameters: serde_json::Map<String, serde_json::Value>,
) -> Result<serde_json::Value, String> {
    let variant = get_template_variant(template_type, toolchain)
        .ok_or_else(|| format!("No variant for {template_type} with {toolchain}"))?;

    let uri = format!("template://{template_type}/{toolchain}/{variant}");

    match template_service::generate_template(server, &uri, parameters).await {
        Ok(generated) => Ok(json!({
            "template": template_type,
            "filename": generated.filename,
            "content": generated.content,
            "checksum": generated.checksum,
        })),
        Err(e) => Err(e.to_string()),
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
                format!("Invalid scaffold_project arguments: {e}"),
            );
        }
    };

    let mut results = Vec::with_capacity(256);
    let mut errors = Vec::with_capacity(256);

    // Generate each requested template
    for template_type in &args.templates {
        match generate_single_template(
            server.as_ref(),
            template_type,
            &args.toolchain,
            args.parameters.clone(),
        )
        .await
        {
            Ok(result) => results.push(result),
            Err(error) => errors.push(json!({
                "template": template_type,
                "error": error,
            })),
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
            "name": "pmat",
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

/// Toyota Way: Extract Method - Handle code churn analysis (complexity ≤8)
async fn handle_analyze_code_churn(
    request_id: serde_json::Value,
    arguments: serde_json::Value,
) -> McpResponse {
    // Parse arguments
    let args = match parse_code_churn_args(arguments) {
        Ok(args) => args,
        Err(e) => {
            return McpResponse::error(
                request_id,
                -32602,
                format!("Invalid analyze_code_churn arguments: {e}"),
            );
        }
    };

    // Extract analysis parameters
    let (project_path, period_days, format) = extract_churn_parameters(&args);

    info!(
        "Analyzing code churn for {:?} over {} days",
        project_path, period_days
    );

    // Run analysis and format response
    run_and_format_churn_analysis(request_id, project_path, period_days, format).await
}

/// Toyota Way Helper: Parse code churn arguments
fn parse_code_churn_args(
    arguments: serde_json::Value,
) -> Result<AnalyzeCodeChurnArgs, serde_json::Error> {
    serde_json::from_value(arguments)
}

/// Toyota Way Helper: Extract churn analysis parameters
fn extract_churn_parameters(args: &AnalyzeCodeChurnArgs) -> (PathBuf, u32, ChurnOutputFormat) {
    let project_path = args.project_path.as_ref().map_or_else(
        || std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        PathBuf::from,
    );

    let period_days = args.period_days.unwrap_or(30);

    let format = args
        .format
        .as_deref()
        .and_then(|f| f.parse::<ChurnOutputFormat>().ok())
        .unwrap_or(ChurnOutputFormat::Summary);

    (project_path, period_days, format)
}

/// Toyota Way Helper: Run analysis and format response
async fn run_and_format_churn_analysis(
    request_id: serde_json::Value,
    project_path: PathBuf,
    period_days: u32,
    format: ChurnOutputFormat,
) -> McpResponse {
    match GitAnalysisService::analyze_code_churn(&project_path, period_days) {
        Ok(analysis) => {
            let content_text = format_churn_output(&analysis, &format);
            let result = build_churn_response(content_text, analysis, &format);
            McpResponse::success(request_id, result)
        }
        Err(e) => {
            error!("Code churn analysis failed: {}", e);
            McpResponse::error(request_id, -32000, e.to_string())
        }
    }
}

/// Toyota Way Helper: Format churn output based on requested format
fn format_churn_output(
    analysis: &crate::models::churn::CodeChurnAnalysis,
    format: &ChurnOutputFormat,
) -> String {
    match format {
        ChurnOutputFormat::Json => serde_json::to_string_pretty(&analysis).unwrap_or_default(),
        ChurnOutputFormat::Markdown => format_churn_as_markdown(analysis),
        ChurnOutputFormat::Csv => format_churn_as_csv(analysis),
        ChurnOutputFormat::Summary => format_churn_summary(analysis),
    }
}

/// Toyota Way Helper: Build churn response JSON
fn build_churn_response(
    content_text: String,
    analysis: crate::models::churn::CodeChurnAnalysis,
    format: &ChurnOutputFormat,
) -> serde_json::Value {
    json!({
        "content": [{
            "type": "text",
            "text": content_text
        }],
        "analysis": analysis,
        "format": format!("{:?}", format),
    })
}

/// Formats a code churn analysis into a human-readable summary
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::handlers::tools::format_churn_summary;
/// use pmat::models::churn::{CodeChurnAnalysis, ChurnSummary};
/// use std::path::PathBuf;
/// use std::collections::HashMap;
/// use chrono::Utc;
///
/// let analysis = CodeChurnAnalysis {
///     generated_at: Utc::now(),
///     period_days: 30,
///     repository_root: PathBuf::from("/project"),
///     files: vec![],
///     summary: ChurnSummary {
///         total_commits: 150,
///         total_files_changed: 45,
///         hotspot_files: vec![PathBuf::from("src/main.rs")],
///         stable_files: vec![PathBuf::from("README.md")],
///         author_contributions: HashMap::new(),
///     },
/// };
///
/// let summary = format_churn_summary(&analysis);
/// assert!(summary.contains("Period: 30 days"));
/// assert!(summary.contains("Total commits: 150"));
/// ```
#[must_use]
pub fn format_churn_summary(analysis: &crate::models::churn::CodeChurnAnalysis) -> String {
    let mut output = String::with_capacity(1024);

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

/// Formats a code churn analysis as a Markdown report
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::handlers::tools::format_churn_as_markdown;
/// use pmat::models::churn::{CodeChurnAnalysis, ChurnSummary};
/// use std::path::PathBuf;
/// use std::collections::HashMap;
/// use chrono::Utc;
///
/// let analysis = CodeChurnAnalysis {
///     generated_at: Utc::now(),
///     period_days: 7,
///     repository_root: PathBuf::from("/repo"),
///     files: vec![],
///     summary: ChurnSummary {
///         total_commits: 25,
///         total_files_changed: 12,
///         hotspot_files: vec![],
///         stable_files: vec![],
///         author_contributions: HashMap::new(),
///     },
/// };
///
/// let markdown = format_churn_as_markdown(&analysis);
/// assert!(markdown.contains("# Code Churn Analysis Report"));
/// assert!(markdown.contains("**Period:** 7 days"));
/// ```
#[must_use]
pub fn format_churn_as_markdown(analysis: &crate::models::churn::CodeChurnAnalysis) -> String {
    let mut output = String::with_capacity(1024);

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

/// Formats a code churn analysis as CSV data
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::handlers::tools::format_churn_as_csv;
/// use pmat::models::churn::{CodeChurnAnalysis, ChurnSummary, FileChurnMetrics};
/// use std::path::PathBuf;
/// use std::collections::HashMap;
/// use chrono::Utc;
///
/// let analysis = CodeChurnAnalysis {
///     generated_at: Utc::now(),
///     period_days: 30,
///     repository_root: PathBuf::from("/repo"),
///     files: vec![FileChurnMetrics {
///         path: PathBuf::from("/repo/src/main.rs"),
///         relative_path: "src/main.rs".to_string(),
///         commit_count: 5,
///         unique_authors: vec![],
///         additions: 100,
///         deletions: 50,
///         churn_score: 0.75,
///         last_modified: Utc::now(),
///         first_seen: Utc::now(),
///     }],
///     summary: ChurnSummary {
///         total_commits: 5,
///         total_files_changed: 1,
///         hotspot_files: vec![],
///         stable_files: vec![],
///         author_contributions: HashMap::new(),
///     },
/// };
///
/// let csv = format_churn_as_csv(&analysis);
/// assert!(csv.starts_with("file_path,commits,additions,deletions,churn_score,unique_authors,last_modified"));
/// assert!(csv.contains("src/main.rs,5,100,50,0.750,0"));
/// ```
#[must_use]
pub fn format_churn_as_csv(analysis: &crate::models::churn::CodeChurnAnalysis) -> String {
    let mut output = String::with_capacity(1024);

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
#[cfg(test)]
#[path = "tools_tests.rs"]
mod tests;
