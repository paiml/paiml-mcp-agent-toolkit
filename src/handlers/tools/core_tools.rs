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
