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
            | "analyze_satd"
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
        format!("Unsupported analysis tool: {}", tool_name),
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
        .ok_or_else(|| format!("No variant for {} with {}", template_type, toolchain))?;

    let uri = format!("template://{}/{}/{}", template_type, toolchain, variant);

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
    let project_path = args
        .project_path
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

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
/// ```rust
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
/// ```rust
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
/// ```rust
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
    use crate::services::complexity::*;

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
    project_path
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
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

    let project_path = args
        .project_path
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

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
    match format {
        "json" => serde_json::to_string_pretty(deep_context).unwrap_or_default(),
        _ => {
            use crate::services::deep_context::{DeepContextAnalyzer, DeepContextConfig};
            let analyzer = DeepContextAnalyzer::new(DeepContextConfig::default());
            analyzer
                .format_as_comprehensive_markdown(deep_context)
                .await
                .unwrap_or_else(|_| "Error formatting deep context".to_string())
        }
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
            return McpResponse::error(request_id, -32000, e.to_string());
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

    let project_path = args
        .project_path
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

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

fn get_relative_path(path: &Path, project_path: &Path) -> String {
    path.strip_prefix(project_path)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
}

fn calculate_cyclomatic_complexity(content: &str) -> u32 {
    let control_flow_keywords = ["if", "else", "for", "while", "match", "loop", "?"];
    control_flow_keywords
        .iter()
        .map(|kw| content.matches(kw).count() as u32)
        .sum::<u32>()
        + 1
}

fn calculate_cognitive_complexity(cyclomatic_complexity: u32) -> u32 {
    (cyclomatic_complexity as f32 * 1.5) as u32
}

fn calculate_duplicate_ratio(lines: &[&str]) -> f32 {
    let mut line_counts = std::collections::HashMap::new();
    let mut duplicate_lines = 0;

    // Count non-empty, non-comment lines
    for line in lines {
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with("//") {
            *line_counts.entry(trimmed).or_insert(0) += 1;
        }
    }

    // Count duplicates
    for count in line_counts.values() {
        if *count > 1 {
            duplicate_lines += count - 1;
        }
    }

    if !lines.is_empty() {
        duplicate_lines as f32 / lines.len() as f32
    } else {
        0.0
    }
}

fn calculate_efferent_coupling(content: &str) -> f32 {
    content
        .lines()
        .filter(|line| line.trim().starts_with("use "))
        .count() as f32
}

fn is_public_declaration(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with("pub fn")
        || trimmed.starts_with("pub struct")
        || trimmed.starts_with("pub enum")
        || trimmed.starts_with("pub trait")
        || trimmed.starts_with("pub mod")
}

fn calculate_afferent_coupling(content: &str) -> f32 {
    content
        .lines()
        .filter(|line| is_public_declaration(line))
        .count() as f32
}

fn get_churn_score(relative_path: &str, churn_map: &std::collections::HashMap<String, f32>) -> f32 {
    churn_map.get(relative_path).copied().unwrap_or(0.1)
}

// Helper function to calculate file metrics
async fn calculate_file_metrics(
    path: PathBuf,
    project_path: PathBuf,
    churn_map: std::collections::HashMap<String, f32>,
) -> crate::services::defect_probability::FileMetrics {
    use crate::services::defect_probability::FileMetrics;

    let relative_path = get_relative_path(&path, &project_path);
    let content = tokio::fs::read_to_string(&path).await.unwrap_or_default();
    let lines: Vec<&str> = content.lines().collect();
    let lines_of_code = lines.len();

    let cyclomatic_complexity = calculate_cyclomatic_complexity(&content);
    let cognitive_complexity = calculate_cognitive_complexity(cyclomatic_complexity);
    let churn_score = get_churn_score(&relative_path, &churn_map);
    let duplicate_ratio = calculate_duplicate_ratio(&lines);
    let efferent_coupling = calculate_efferent_coupling(&content);
    let afferent_coupling = calculate_afferent_coupling(&content);

    FileMetrics {
        file_path: relative_path,
        churn_score,
        complexity: cyclomatic_complexity as f32,
        duplicate_ratio,
        afferent_coupling,
        efferent_coupling,
        lines_of_code,
        cyclomatic_complexity,
        cognitive_complexity,
    }
}

#[allow(dead_code)]
/// Toyota Way: Extract Method - Handle defect probability analysis (complexity ≤8)
async fn handle_analyze_defect_probability(
    request_id: serde_json::Value,
    arguments: serde_json::Value,
) -> McpResponse {
    // Parse arguments
    let (args, project_path) = match parse_defect_probability_args(arguments) {
        Ok(result) => result,
        Err(e) => {
            return McpResponse::error(
                request_id,
                -32602,
                format!("Invalid analyze_defect_probability arguments: {e}"),
            );
        }
    };

    info!("Analyzing defect probability for {:?}", project_path);

    // Build churn map from git analysis
    let churn_map = build_churn_map(&project_path);

    // Discover and analyze files
    let file_metrics =
        match discover_and_analyze_files(&project_path, churn_map, request_id.clone()).await {
            Ok(metrics) => metrics,
            Err(response) => return response,
        };

    // Calculate defect probabilities and create response
    create_defect_probability_response(request_id, args, file_metrics)
}

/// Toyota Way: Extract Method - Parse defect probability arguments (complexity ≤5)
fn parse_defect_probability_args(
    arguments: serde_json::Value,
) -> Result<(AnalyzeDefectProbabilityArgs, PathBuf), Box<dyn std::error::Error>> {
    let args: AnalyzeDefectProbabilityArgs = serde_json::from_value(arguments)?;

    let project_path = args
        .project_path
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    Ok((args, project_path))
}

/// Toyota Way: Extract Method - Build churn map from git analysis (complexity ≤5)
fn build_churn_map(project_path: &Path) -> std::collections::HashMap<String, f32> {
    use crate::services::git_analysis::GitAnalysisService;

    let churn_analysis = GitAnalysisService::analyze_code_churn(project_path, 30).ok();
    churn_analysis
        .map(|analysis| {
            analysis
                .files
                .into_iter()
                .map(|f| (f.relative_path, f.churn_score))
                .collect()
        })
        .unwrap_or_default()
}

/// Toyota Way: Extract Method - Discover and analyze files (complexity ≤8)
async fn discover_and_analyze_files(
    project_path: &Path,
    churn_map: std::collections::HashMap<String, f32>,
    request_id: serde_json::Value,
) -> Result<Vec<crate::services::defect_probability::FileMetrics>, McpResponse> {
    use crate::services::file_discovery::ProjectFileDiscovery;
    use futures::stream::{self, StreamExt};

    // Discover files
    let discovery = ProjectFileDiscovery::new(project_path.to_path_buf());
    let discovered_files = match discovery.discover_files() {
        Ok(files) => files,
        Err(e) => {
            error!("Failed to discover files: {}", e);
            return Err(McpResponse::error(
                request_id,
                -32603,
                format!("Failed to discover files: {e}"),
            ));
        }
    };

    // Process files in parallel
    let metrics_futures: Vec<_> = discovered_files
        .into_iter()
        .filter(|path| path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rs"))
        .map(|path| {
            let project_path = project_path.to_path_buf();
            let churn_map = churn_map.clone();
            calculate_file_metrics(path, project_path, churn_map)
        })
        .collect();

    // Execute futures concurrently
    let file_metrics = stream::iter(metrics_futures)
        .buffer_unordered(8)
        .collect()
        .await;

    Ok(file_metrics)
}

/// Toyota Way: Extract Method - Create defect probability response (complexity ≤6)
fn create_defect_probability_response(
    request_id: serde_json::Value,
    args: AnalyzeDefectProbabilityArgs,
    file_metrics: Vec<crate::services::defect_probability::FileMetrics>,
) -> McpResponse {
    use crate::services::defect_probability::{DefectProbabilityCalculator, ProjectDefectAnalysis};

    let calculator = DefectProbabilityCalculator::new();
    let scores = calculator.calculate_batch(&file_metrics);
    let analysis = ProjectDefectAnalysis::from_scores(scores);

    let content_text = format_defect_probability_output(&args, &analysis);

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

/// Toyota Way: Extract Method - Format defect probability output (complexity ≤5)
fn format_defect_probability_output(
    args: &AnalyzeDefectProbabilityArgs,
    analysis: &crate::services::defect_probability::ProjectDefectAnalysis,
) -> String {
    match args.format.as_deref() {
        Some("json") => serde_json::to_string_pretty(analysis).unwrap_or_default(),
        _ => format!(
            "# Defect Probability Analysis\n\nTotal files: {}\nHigh-risk files: {}\nMedium-risk files: {}\nAverage probability: {:.2}",
            analysis.total_files,
            analysis.high_risk_files.len(),
            analysis.medium_risk_files.len(),
            analysis.average_probability
        ),
    }
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

/// Toyota Way: Extract Method - Handle dead code analysis (complexity ≤8)
async fn handle_analyze_dead_code(
    request_id: serde_json::Value,
    arguments: serde_json::Value,
) -> McpResponse {
    // Parse arguments
    let (args, project_path) = match parse_dead_code_args(arguments) {
        Ok(result) => result,
        Err(e) => {
            return McpResponse::error(
                request_id,
                -32602,
                format!("Invalid analyze_dead_code arguments: {e}"),
            );
        }
    };

    info!("Analyzing dead code for {:?}", project_path);

    // Run dead code analysis
    let mut result = match run_dead_code_analysis(&project_path, &args).await {
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

    // Format and respond
    format_and_respond_dead_code(request_id, args, result)
}

/// Toyota Way: Extract Method - Parse dead code arguments (complexity ≤5)
fn parse_dead_code_args(
    arguments: serde_json::Value,
) -> Result<(AnalyzeDeadCodeArgs, PathBuf), Box<dyn std::error::Error>> {
    let args: AnalyzeDeadCodeArgs = serde_json::from_value(arguments)?;

    let project_path = args
        .project_path
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    Ok((args, project_path))
}

/// Toyota Way: Extract Method - Run dead code analysis (complexity ≤6)
async fn run_dead_code_analysis(
    project_path: &Path,
    args: &AnalyzeDeadCodeArgs,
) -> Result<crate::models::dead_code::DeadCodeRankingResult, Box<dyn std::error::Error>> {
    use crate::models::dead_code::DeadCodeAnalysisConfig;
    use crate::services::dead_code_analyzer::DeadCodeAnalyzer;

    let mut analyzer = DeadCodeAnalyzer::new(10000);

    let config = DeadCodeAnalysisConfig {
        include_unreachable: args.include_unreachable.unwrap_or(false),
        include_tests: args.include_tests.unwrap_or(false),
        min_dead_lines: args.min_dead_lines.unwrap_or(10),
    };

    Ok(analyzer.analyze_with_ranking(project_path, config).await?)
}

/// Toyota Way: Extract Method - Format and respond with dead code results (complexity ≤8)
fn format_and_respond_dead_code(
    request_id: serde_json::Value,
    args: AnalyzeDeadCodeArgs,
    result: crate::models::dead_code::DeadCodeRankingResult,
) -> McpResponse {
    let format = args.format.as_deref().unwrap_or("summary");

    let content_text = match format_dead_code_output(&result, format) {
        Ok(content) => content,
        Err(e) => {
            return McpResponse::error(request_id, -32000, format!("Failed to format output: {e}"));
        }
    };

    let response = build_dead_code_response(format, content_text, &result);
    McpResponse::success(request_id, response)
}

/// Toyota Way: Extract Method - Build dead code response JSON (complexity ≤5)
fn build_dead_code_response(
    format: &str,
    content_text: String,
    result: &crate::models::dead_code::DeadCodeRankingResult,
) -> serde_json::Value {
    json!({
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
    })
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

/// Toyota Way: Extract Method - Format dead code analysis as summary text for MCP (complexity ≤8)
fn format_dead_code_summary_mcp(
    result: &crate::models::dead_code::DeadCodeRankingResult,
) -> anyhow::Result<String> {
    let mut output = String::with_capacity(1024);

    output.push_str("# Dead Code Analysis Summary\n\n");
    format_dead_code_summary_stats(&mut output, &result.summary);
    format_top_dead_code_files(&mut output, &result.ranked_files);

    Ok(output)
}

/// Toyota Way: Extract Method - Format summary statistics section (complexity ≤5)
fn format_dead_code_summary_stats(
    output: &mut String,
    summary: &crate::models::dead_code::DeadCodeSummary,
) {
    output.push_str(&format!(
        "**Total files analyzed:** {}\n",
        summary.total_files_analyzed
    ));

    let files_with_dead_percentage = if summary.total_files_analyzed > 0 {
        (summary.files_with_dead_code as f32 / summary.total_files_analyzed as f32) * 100.0
    } else {
        0.0
    };

    output.push_str(&format!(
        "**Files with dead code:** {} ({:.1}%)\n",
        summary.files_with_dead_code, files_with_dead_percentage
    ));
    output.push_str(&format!(
        "**Total dead lines:** {} ({:.1}% of codebase)\n",
        summary.total_dead_lines, summary.dead_percentage
    ));
    output.push_str(&format!("**Dead functions:** {}\n", summary.dead_functions));
    output.push_str(&format!("**Dead classes:** {}\n", summary.dead_classes));
    output.push_str(&format!("**Dead modules:** {}\n", summary.dead_modules));
    output.push_str(&format!(
        "**Unreachable blocks:** {}\n\n",
        summary.unreachable_blocks
    ));
}

/// Toyota Way: Extract Method - Format top files with dead code (complexity ≤8)
fn format_top_dead_code_files(
    output: &mut String,
    ranked_files: &[crate::models::dead_code::FileDeadCodeMetrics],
) {
    if !ranked_files.is_empty() {
        let top_count = ranked_files.len().min(5);
        output.push_str(&format!("## Top {top_count} Files with Most Dead Code\n\n"));

        for (i, file_metrics) in ranked_files.iter().take(top_count).enumerate() {
            format_dead_code_file_entry(output, i + 1, file_metrics);
        }
    }
}

/// Toyota Way: Extract Method - Format individual file entry (complexity ≤5)
fn format_dead_code_file_entry(
    output: &mut String,
    index: usize,
    file_metrics: &crate::models::dead_code::FileDeadCodeMetrics,
) {
    let confidence_text = get_confidence_level_text(file_metrics.confidence);

    output.push_str(&format!(
        "{}. **{}** (Score: {:.1}) [{}confidence]\n",
        index, file_metrics.path, file_metrics.dead_score, confidence_text
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

/// Toyota Way: Extract Method - Get confidence level text (complexity ≤3)
fn get_confidence_level_text(
    confidence: crate::models::dead_code::ConfidenceLevel,
) -> &'static str {
    match confidence {
        crate::models::dead_code::ConfidenceLevel::High => "HIGH ",
        crate::models::dead_code::ConfidenceLevel::Medium => "MEDIUM ",
        crate::models::dead_code::ConfidenceLevel::Low => "LOW ",
    }
}

/// Format dead code analysis as SARIF for MCP
fn format_dead_code_as_sarif_mcp(
    result: &crate::models::dead_code::DeadCodeRankingResult,
) -> anyhow::Result<String> {
    use serde_json::json;

    let mut results = Vec::with_capacity(256);

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
                    "name": "pmat",
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
/// Toyota Way: Extract Method - Format dead code analysis as markdown (complexity ≤8)
fn format_dead_code_as_markdown_mcp(
    result: &crate::models::dead_code::DeadCodeRankingResult,
) -> anyhow::Result<String> {
    let mut output = String::with_capacity(1024);

    write_dead_code_header(&mut output, &result.analysis_timestamp);
    write_dead_code_summary_section(&mut output, &result.summary);
    write_dead_code_top_files_section(&mut output, &result.ranked_files);

    Ok(output)
}

/// Toyota Way: Extract Method - Write dead code report header (complexity ≤3)
fn write_dead_code_header(output: &mut String, timestamp: &chrono::DateTime<chrono::Utc>) {
    output.push_str("# Dead Code Analysis Report\n\n");
    output.push_str(&format!(
        "**Analysis Date:** {}\n\n",
        timestamp.format("%Y-%m-%d %H:%M:%S UTC")
    ));
}

/// Toyota Way: Extract Method - Write dead code summary section (complexity ≤8)
fn write_dead_code_summary_section(
    output: &mut String,
    summary: &crate::models::dead_code::DeadCodeSummary,
) {
    output.push_str("## Summary\n\n");

    output.push_str(&format!(
        "- **Total files analyzed:** {}\n",
        summary.total_files_analyzed
    ));

    let files_with_dead_percentage = calculate_dead_files_percentage(summary);
    output.push_str(&format!(
        "- **Files with dead code:** {} ({:.1}%)\n",
        summary.files_with_dead_code, files_with_dead_percentage
    ));

    write_dead_code_metrics(output, summary);
}

/// Toyota Way: Extract Method - Calculate dead files percentage (complexity ≤3)
fn calculate_dead_files_percentage(summary: &crate::models::dead_code::DeadCodeSummary) -> f32 {
    if summary.total_files_analyzed > 0 {
        (summary.files_with_dead_code as f32 / summary.total_files_analyzed as f32) * 100.0
    } else {
        0.0
    }
}

/// Toyota Way: Extract Method - Write dead code metrics (complexity ≤5)
fn write_dead_code_metrics(
    output: &mut String,
    summary: &crate::models::dead_code::DeadCodeSummary,
) {
    output.push_str(&format!(
        "- **Total dead lines:** {} ({:.1}% of codebase)\n",
        summary.total_dead_lines, summary.dead_percentage
    ));
    output.push_str(&format!(
        "- **Dead functions:** {}\n",
        summary.dead_functions
    ));
    output.push_str(&format!("- **Dead classes:** {}\n", summary.dead_classes));
    output.push_str(&format!("- **Dead modules:** {}\n", summary.dead_modules));
    output.push_str(&format!(
        "- **Unreachable blocks:** {}\n\n",
        summary.unreachable_blocks
    ));
}

/// Toyota Way: Extract Method - Write top dead code files section (complexity ≤8)
fn write_dead_code_top_files_section(
    output: &mut String,
    ranked_files: &[crate::models::dead_code::FileDeadCodeMetrics],
) {
    if !ranked_files.is_empty() {
        write_dead_code_table_header(output);
        write_dead_code_table_rows(output, ranked_files);
        output.push('\n');
    }
}

/// Toyota Way: Extract Method - Write dead code table header (complexity ≤3)
fn write_dead_code_table_header(output: &mut String) {
    output.push_str("## Top Files with Dead Code\n\n");
    output.push_str(
        "| Rank | File | Dead Lines | Percentage | Functions | Classes | Score | Confidence |\n",
    );
    output.push_str(
        "|------|------|------------|------------|-----------|---------|-------|------------|\n",
    );
}

/// Toyota Way: Extract Method - Write dead code table rows (complexity ≤5)
fn write_dead_code_table_rows(
    output: &mut String,
    ranked_files: &[crate::models::dead_code::FileDeadCodeMetrics],
) {
    for (i, file_metrics) in ranked_files.iter().enumerate() {
        write_single_dead_code_row(output, i + 1, file_metrics);
    }
}

/// Toyota Way: Extract Method - Write single dead code table row (complexity ≤5)
fn write_single_dead_code_row(
    output: &mut String,
    rank: usize,
    file_metrics: &crate::models::dead_code::FileDeadCodeMetrics,
) {
    let confidence_text = format_confidence_emoji(file_metrics.confidence);

    output.push_str(&format!(
        "| {:>4} | `{}` | {:>10} | {:>9.1}% | {:>9} | {:>7} | {:>5.1} | {} |\n",
        rank,
        file_metrics.path,
        file_metrics.dead_lines,
        file_metrics.dead_percentage,
        file_metrics.dead_functions,
        file_metrics.dead_classes,
        file_metrics.dead_score,
        confidence_text
    ));
}

/// Toyota Way: Extract Method - Format confidence level with emoji (complexity ≤3)
fn format_confidence_emoji(confidence: crate::models::dead_code::ConfidenceLevel) -> &'static str {
    match confidence {
        crate::models::dead_code::ConfidenceLevel::High => "🔴 High",
        crate::models::dead_code::ConfidenceLevel::Medium => "🟡 Medium",
        crate::models::dead_code::ConfidenceLevel::Low => "🟢 Low",
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct AnalyzeTdgArgs {
    project_path: Option<String>,
    format: Option<String>,
    threshold: Option<f64>,
    include_components: Option<bool>,
    max_results: Option<usize>,
}

/// Toyota Way: Extract Method - Handle TDG analysis (complexity ≤8)
async fn handle_analyze_tdg(
    request_id: serde_json::Value,
    arguments: serde_json::Value,
) -> McpResponse {
    // Parse arguments
    let args = match parse_tdg_args(arguments) {
        Ok(args) => args,
        Err(e) => {
            return McpResponse::error(
                request_id,
                -32602,
                format!("Invalid analyze_tdg arguments: {e}"),
            );
        }
    };

    // Extract project path
    let project_path = extract_tdg_project_path(&args);
    info!("Analyzing Technical Debt Gradient for {:?}", project_path);

    // Run analysis and format response
    run_and_format_tdg_analysis(request_id, project_path, args.format).await
}

/// Toyota Way Helper: Parse TDG arguments
fn parse_tdg_args(arguments: serde_json::Value) -> Result<AnalyzeTdgArgs, serde_json::Error> {
    serde_json::from_value(arguments)
}

/// Toyota Way Helper: Extract TDG project path
fn extract_tdg_project_path(args: &AnalyzeTdgArgs) -> PathBuf {
    args.project_path
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

/// Toyota Way Helper: Run TDG analysis and format response
async fn run_and_format_tdg_analysis(
    request_id: serde_json::Value,
    project_path: PathBuf,
    format: Option<String>,
) -> McpResponse {
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

    // Format and respond
    format_and_respond_tdg(request_id, analysis, format)
}

/// Toyota Way Helper: Format TDG analysis and build response
fn format_and_respond_tdg(
    request_id: serde_json::Value,
    analysis: crate::models::tdg::TDGSummary,
    format: Option<String>,
) -> McpResponse {
    // Format output
    let content_text = match format.as_deref() {
        Some("json") => serde_json::to_string_pretty(&analysis).unwrap_or_default(),
        _ => format_tdg_summary(&analysis),
    };

    let result = json!({
        "content": [{
            "type": "text",
            "text": content_text
        }],
        "analysis": analysis,
        "format": format.unwrap_or_else(|| "summary".to_string()),
    });

    McpResponse::success(request_id, result)
}

/// Toyota Way: Extract Method - Format TDG summary (complexity ≤8)
fn format_tdg_summary(summary: &crate::models::tdg::TDGSummary) -> String {
    let mut output = String::with_capacity(1024);

    output.push_str("# Technical Debt Gradient Analysis\n\n");

    // Build each section
    append_tdg_summary_section(&mut output, summary);
    append_tdg_hotspots_section(&mut output, summary);
    append_tdg_severity_section(&mut output, summary);

    output
}

/// Toyota Way Helper: Append TDG summary statistics
fn append_tdg_summary_section(output: &mut String, summary: &crate::models::tdg::TDGSummary) {
    output.push_str("## Summary\n\n");
    output.push_str(&format!("**Total files:** {}\n", summary.total_files));

    // Calculate and append percentages
    let critical_pct = calculate_percentage(summary.critical_files, summary.total_files);
    let warning_pct = calculate_percentage(summary.warning_files, summary.total_files);

    output.push_str(&format!(
        "**Critical files:** {} ({:.1}%)\n",
        summary.critical_files, critical_pct
    ));
    output.push_str(&format!(
        "**Warning files:** {} ({:.1}%)\n",
        summary.warning_files, warning_pct
    ));

    // Append metrics
    append_tdg_metrics(output, summary);
}

/// Toyota Way Helper: Append TDG metrics
fn append_tdg_metrics(output: &mut String, summary: &crate::models::tdg::TDGSummary) {
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
}

/// Toyota Way Helper: Append TDG hotspots table
fn append_tdg_hotspots_section(output: &mut String, summary: &crate::models::tdg::TDGSummary) {
    if summary.hotspots.is_empty() {
        return;
    }

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

/// Toyota Way Helper: Append TDG severity distribution
fn append_tdg_severity_section(output: &mut String, summary: &crate::models::tdg::TDGSummary) {
    output.push_str("## Severity Distribution\n\n");
    output.push_str(&format!(
        "- 🔴 Critical (>2.5): {} files\n",
        summary.critical_files
    ));
    output.push_str(&format!(
        "- 🟡 Warning (1.5-2.5): {} files\n",
        summary.warning_files
    ));

    let normal_files = summary
        .total_files
        .saturating_sub(summary.critical_files + summary.warning_files);
    output.push_str(&format!("- 🟢 Normal (<1.5): {} files\n", normal_files));
}

/// Toyota Way Helper: Calculate percentage safely
fn calculate_percentage(part: usize, total: usize) -> f64 {
    if total > 0 {
        (part as f64 / total as f64) * 100.0
    } else {
        0.0
    }
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

    let project_path = resolve_deep_context_project_path(args.project_path.clone());
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

fn resolve_deep_context_project_path(project_path: Option<String>) -> PathBuf {
    project_path
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

fn default_project_path() -> String {
    ".".to_string()
}

fn default_top_files() -> usize {
    10
}

fn get_default_analysis_types() -> Vec<crate::services::deep_context::AnalysisType> {
    use crate::services::deep_context::AnalysisType;
    vec![
        AnalysisType::Ast,
        AnalysisType::Complexity,
        AnalysisType::Churn,
    ]
}

fn parse_analysis_type_string(s: &str) -> Option<crate::services::deep_context::AnalysisType> {
    use crate::services::deep_context::AnalysisType;

    match s {
        "ast" => Some(AnalysisType::Ast),
        "complexity" => Some(AnalysisType::Complexity),
        "churn" => Some(AnalysisType::Churn),
        "dag" => Some(AnalysisType::Dag),
        "dead_code" => Some(AnalysisType::DeadCode),
        "satd" => Some(AnalysisType::Satd),
        "tdg" => Some(AnalysisType::TechnicalDebtGradient),
        _ => None,
    }
}

fn parse_analysis_types(
    include_analyses: Option<Vec<String>>,
) -> Vec<crate::services::deep_context::AnalysisType> {
    match include_analyses {
        Some(analyses) => analyses
            .iter()
            .filter_map(|s| parse_analysis_type_string(s))
            .collect(),
        None => get_default_analysis_types(),
    }
}

fn parse_deep_context_dag_type(dag_type: Option<String>) -> crate::services::deep_context::DagType {
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
        dag_type: parse_deep_context_dag_type(args.dag_type.clone()),
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
                    "name": "pmat",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/paiml/mcp-agent-toolkit"
                }
            },
            "results": []
        }]
    });

    serde_json::to_string_pretty(&sarif).unwrap_or_default()
}

/// Toyota Way: Extract Method - Format deep context analysis as markdown (complexity ≤8)
fn format_deep_context_as_markdown(context: &crate::services::deep_context::DeepContext) -> String {
    use crate::cli::formatting_helpers::*;

    let mut output = String::with_capacity(1024);
    output.push_str("# Deep Context Analysis\n\n");

    // Reuse helper functions from cli module
    output.push_str(&format_executive_summary(context));
    format_essential_metadata(&mut output, context);

    // Quality Scorecard and other sections
    output.push_str(&format_quality_scorecard(context));
    output.push_str(&format_defect_summary(context));
    output.push_str(&format_recommendations(context));

    format_analysis_results(&mut output, context);
    format_deep_context_recommendations(&mut output, context);

    output
}

/// Toyota Way: Extract Method - Format essential project metadata section (complexity ≤5)
fn format_essential_metadata(
    output: &mut String,
    context: &crate::services::deep_context::DeepContext,
) {
    use crate::cli::formatting_helpers::*;

    if context.project_overview.is_some() || context.build_info.is_some() {
        output.push_str("\n## Essential Project Metadata\n\n");

        if let Some(ref overview) = context.project_overview {
            output.push_str(&format_project_overview(overview));
        }

        if let Some(ref build_info) = context.build_info {
            output.push_str(&format_build_info(build_info));
        }
    }
}

/// Toyota Way: Extract Method - Format analysis results section (complexity ≤8)
fn format_analysis_results(
    output: &mut String,
    context: &crate::services::deep_context::DeepContext,
) {
    output.push_str("\n## Analysis Results\n\n");
    output.push_str(&format!(
        "**Total Defects:** {}\n",
        context.defect_summary.total_defects
    ));
    output.push_str(&format!(
        "**Defect Density:** {:.2}\n",
        context.defect_summary.defect_density
    ));

    format_defects_by_type(output, &context.defect_summary.by_type);
    format_defects_by_severity(output, &context.defect_summary.by_severity);

    output.push_str(&format!(
        "**Total Files:** {}\n\n",
        context.file_tree.total_files
    ));
}

/// Toyota Way: Extract Method - Format defects by type (complexity ≤5)
fn format_defects_by_type(output: &mut String, by_type: &rustc_hash::FxHashMap<String, usize>) {
    if !by_type.is_empty() {
        output.push_str("**By Type:**\n");
        for (defect_type, count) in by_type {
            output.push_str(&format!("- {defect_type}: {count}\n"));
        }
    }
}

/// Toyota Way: Extract Method - Format defects by severity (complexity ≤5)
fn format_defects_by_severity(
    output: &mut String,
    by_severity: &rustc_hash::FxHashMap<String, usize>,
) {
    if !by_severity.is_empty() {
        output.push_str("**By Severity:**\n");
        for (severity, count) in by_severity {
            output.push_str(&format!("- {severity}: {count}\n"));
        }
    }
}

/// Toyota Way: Extract Method - Format recommendations section (complexity ≤5)
fn format_deep_context_recommendations(
    output: &mut String,
    context: &crate::services::deep_context::DeepContext,
) {
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
}

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

fn parse_makefile_lint_args(
    arguments: Option<serde_json::Value>,
) -> Result<MakefileLintArgs, String> {
    match arguments {
        Some(args) => serde_json::from_value(args)
            .map_err(|e| format!("Invalid analyze_makefile_lint arguments: {e}")),
        None => Err("Missing required arguments for analyze_makefile_lint".to_string()),
    }
}

async fn execute_makefile_linting(
    makefile_path: &std::path::Path,
) -> Result<crate::services::makefile_linter::LintResult, String> {
    use crate::services::makefile_linter;

    makefile_linter::lint_makefile(makefile_path)
        .await
        .map_err(|e| format!("Makefile linting failed: {e}"))
}

fn map_severity(severity: &crate::services::makefile_linter::Severity) -> &'static str {
    use crate::services::makefile_linter::Severity;

    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Performance => "performance",
        Severity::Info => "info",
    }
}

fn format_violation(violation: &crate::services::makefile_linter::Violation) -> serde_json::Value {
    json!({
        "rule": violation.rule,
        "severity": map_severity(&violation.severity),
        "line": violation.span.line,
        "column": violation.span.column,
        "message": violation.message,
        "fix_hint": violation.fix_hint,
    })
}

fn count_violations_by_severity(
    violations: &[crate::services::makefile_linter::Violation],
    _target_severity: crate::services::makefile_linter::Severity,
) -> usize {
    violations
        .iter()
        .filter(|v| matches!(&v.severity, _target_severity))
        .count()
}

fn build_makefile_analysis(
    args: &MakefileLintArgs,
    lint_result: &crate::services::makefile_linter::LintResult,
) -> serde_json::Value {
    use crate::services::makefile_linter::Severity;

    json!({
        "path": args.path,
        "violations": lint_result.violations.iter().map(format_violation).collect::<Vec<_>>(),
        "quality_score": lint_result.quality_score,
        "rules_applied": args.rules,
        "total_violations": lint_result.violations.len(),
        "error_count": count_violations_by_severity(&lint_result.violations, Severity::Error),
        "warning_count": count_violations_by_severity(&lint_result.violations, Severity::Warning),
    })
}

async fn handle_analyze_makefile_lint(
    request_id: serde_json::Value,
    arguments: Option<serde_json::Value>,
) -> McpResponse {
    let args = match parse_makefile_lint_args(arguments) {
        Ok(args) => args,
        Err(e) => return McpResponse::error(request_id, -32602, e),
    };

    let makefile_path = std::path::Path::new(&args.path);
    info!("Analyzing Makefile at {:?}", makefile_path);

    let lint_result = match execute_makefile_linting(makefile_path).await {
        Ok(result) => result,
        Err(e) => return McpResponse::error(request_id, -32000, e),
    };

    let analysis = build_makefile_analysis(&args, &lint_result);
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

#[derive(Deserialize)]
struct SatdArgs {
    #[serde(default = "default_project_path")]
    project_path: String,
    #[serde(default)]
    strict: bool,
    #[serde(default = "default_true")]
    exclude_tests: bool,
    #[serde(default)]
    critical_only: bool,
    #[serde(default = "default_summary_format")]
    format: String,
}

fn default_true() -> bool {
    true
}

fn default_summary_format() -> String {
    "summary".to_string()
}

fn parse_satd_args(arguments: serde_json::Value) -> Result<SatdArgs, String> {
    serde_json::from_value(arguments).map_err(|e| format!("Invalid analyze_satd arguments: {e}"))
}

fn create_satd_detector(strict: bool) -> crate::services::satd_detector::SATDDetector {
    use crate::services::satd_detector::SATDDetector;

    if strict {
        SATDDetector::new_strict()
    } else {
        SATDDetector::new()
    }
}

async fn execute_satd_analysis(
    args: &SatdArgs,
) -> Result<crate::services::satd_detector::SATDAnalysisResult, String> {
    use std::path::Path;

    let detector = create_satd_detector(args.strict);
    let project_path = Path::new(&args.project_path);

    detector
        .analyze_project(project_path, !args.exclude_tests)
        .await
        .map_err(|e| format!("Failed to analyze SATD: {e}"))
}

fn filter_satd_items(
    mut result: crate::services::satd_detector::SATDAnalysisResult,
    critical_only: bool,
) -> (
    crate::services::satd_detector::SATDAnalysisResult,
    Vec<crate::services::satd_detector::TechnicalDebt>,
) {
    use crate::services::satd_detector::Severity;

    let items = if critical_only {
        std::mem::take(&mut result.items)
            .into_iter()
            .filter(|item| matches!(item.severity, Severity::Critical))
            .collect::<Vec<_>>()
    } else {
        std::mem::take(&mut result.items)
    };

    (result, items)
}

fn format_satd_json_output(
    args: &SatdArgs,
    result: &crate::services::satd_detector::SATDAnalysisResult,
    items: &[crate::services::satd_detector::TechnicalDebt],
) -> serde_json::Value {
    json!({
        "project_path": args.project_path,
        "total_debt_items": result.summary.total_items,
        "debt_density": (result.summary.total_items as f64 / result.total_files_analyzed.max(1) as f64),
        "critical_items": result.summary.by_severity.get("Critical").copied().unwrap_or(0),
        "categories": result.summary.by_category,
        "items": items.iter().map(|item| json!({
            "file": item.file.display().to_string(),
            "line": item.line,
            "column": item.column,
            "category": format!("{:?}", item.category),
            "severity": format!("{:?}", item.severity),
            "text": item.text,
        })).collect::<Vec<_>>(),
    })
}

fn build_satd_summary_header(
    result: &crate::services::satd_detector::SATDAnalysisResult,
) -> String {
    let mut summary = String::from("SATD Analysis Summary\n");
    summary.push_str("====================\n");
    summary.push_str(&format!(
        "Total debt items: {}\n",
        result.summary.total_items
    ));
    summary.push_str(&format!(
        "Debt density: {:.2} per KLOC\n",
        (result.summary.total_items as f64 / result.total_files_analyzed.max(1) as f64)
    ));
    summary.push_str(&format!(
        "Critical items: {}\n",
        result
            .summary
            .by_severity
            .get("Critical")
            .copied()
            .unwrap_or(0)
    ));
    summary.push_str("\nTop files with technical debt:\n");
    summary
}

fn group_and_sort_satd_items(
    items: &[crate::services::satd_detector::TechnicalDebt],
) -> Vec<(
    &std::path::Path,
    Vec<&crate::services::satd_detector::TechnicalDebt>,
)> {
    use std::collections::HashMap;

    let mut files_map: HashMap<
        &std::path::Path,
        Vec<&crate::services::satd_detector::TechnicalDebt>,
    > = HashMap::new();

    for item in items {
        files_map.entry(&item.file).or_default().push(item);
    }

    let mut sorted_files: Vec<_> = files_map.into_iter().collect();
    sorted_files.sort_by_key(|(_, items)| -(items.len() as i32));
    sorted_files
}

fn format_satd_summary_output(
    result: &crate::services::satd_detector::SATDAnalysisResult,
    items: &[crate::services::satd_detector::TechnicalDebt],
) -> serde_json::Value {
    let mut summary = build_satd_summary_header(result);
    let sorted_files = group_and_sort_satd_items(items);

    for (path, file_items) in sorted_files.iter().take(10) {
        summary.push_str(&format!(
            "  {} - {} items\n",
            path.display(),
            file_items.len()
        ));
    }

    json!({
        "formatted_output": summary,
        "stats": {
            "total_items": result.summary.total_items,
            "critical_items": result.summary.by_severity.get("Critical").copied().unwrap_or(0),
            "debt_density": (result.summary.total_items as f64 / result.total_files_analyzed.max(1) as f64),
        }
    })
}

fn format_satd_output(
    args: &SatdArgs,
    result: &crate::services::satd_detector::SATDAnalysisResult,
    items: &[crate::services::satd_detector::TechnicalDebt],
) -> serde_json::Value {
    match args.format.as_str() {
        "json" => format_satd_json_output(args, result, items),
        _ => format_satd_summary_output(result, items),
    }
}

async fn handle_analyze_satd(
    request_id: serde_json::Value,
    arguments: serde_json::Value,
) -> McpResponse {
    let args = match parse_satd_args(arguments) {
        Ok(args) => args,
        Err(e) => return McpResponse::error(request_id, -32602, e),
    };

    info!("Analyzing SATD for project: {:?}", args.project_path);

    let result = match execute_satd_analysis(&args).await {
        Ok(result) => result,
        Err(e) => return McpResponse::error(request_id, -32603, e),
    };

    let (result, items) = filter_satd_items(result, args.critical_only);
    let output = format_satd_output(&args, &result, &items);

    McpResponse::success(request_id, output)
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct LintHotspotArgs {
    #[serde(default = "default_project_path")]
    project_path: String,
    #[serde(default = "default_top_files")]
    top_files: usize,
    #[serde(default = "default_min_violations")]
    min_violations: usize,
    #[serde(default)]
    include: Option<String>,
    #[serde(default)]
    exclude: Option<String>,
    #[serde(default = "default_table_format")]
    format: String,
}

fn default_min_violations() -> usize {
    1
}

fn default_table_format() -> String {
    "table".to_string()
}

fn parse_lint_hotspot_args(arguments: serde_json::Value) -> Result<LintHotspotArgs, String> {
    serde_json::from_value(arguments)
        .map_err(|e| format!("Invalid analyze_lint_hotspot arguments: {e}"))
}

async fn execute_lint_hotspot_analysis(
    args: &LintHotspotArgs,
    project_path: &Path,
) -> Result<std::path::PathBuf, String> {
    use crate::cli::handlers::lint_hotspot_handlers::handle_analyze_lint_hotspot;
    use crate::cli::LintHotspotOutputFormat;

    let temp_file = tempfile::NamedTempFile::new()
        .map_err(|e| format!("Failed to create temporary file: {e}"))?;
    let output_path = temp_file.path().to_path_buf();

    handle_analyze_lint_hotspot(
        project_path.to_path_buf(),
        None,
        LintHotspotOutputFormat::Json,
        100.0,
        0.0,
        false,
        false,
        false,
        Some(output_path.clone()),
        false,
        String::new(),
        args.top_files,
        Vec::new(),
        Vec::new(),
    )
    .await
    .map_err(|e| format!("Failed to analyze lint hotspots: {e}"))?;

    Ok(output_path)
}

async fn read_and_parse_lint_output(
    output_path: &std::path::Path,
) -> Result<serde_json::Value, String> {
    let json_output = tokio::fs::read_to_string(output_path)
        .await
        .map_err(|e| format!("Failed to read temporary file: {e}"))?;

    serde_json::from_str(&json_output).map_err(|e| format!("Failed to parse JSON output: {e}"))
}

struct LintHotspotData {
    hotspots: Vec<serde_json::Value>,
    total_files: usize,
    total_violations: usize,
    average_violations_per_file: f64,
}

fn extract_lint_data(lint_data: &serde_json::Value) -> LintHotspotData {
    LintHotspotData {
        hotspots: lint_data["hotspots"].as_array().unwrap_or(&vec![]).clone(),
        total_files: lint_data["total_files_analyzed"].as_u64().unwrap_or(0) as usize,
        total_violations: lint_data["total_violations"].as_u64().unwrap_or(0) as usize,
        average_violations_per_file: lint_data["average_violations_per_file"]
            .as_f64()
            .unwrap_or(0.0),
    }
}

fn format_lint_hotspot_output(args: &LintHotspotArgs, data: &LintHotspotData) -> serde_json::Value {
    match args.format.as_str() {
        "json" => format_json_output(args, data),
        "csv" => format_csv_output(),
        _ => format_table_output(data),
    }
}

fn format_json_output(args: &LintHotspotArgs, data: &LintHotspotData) -> serde_json::Value {
    json!({
        "project_path": args.project_path,
        "total_files_analyzed": data.total_files,
        "total_violations": data.total_violations,
        "average_violations_per_file": data.average_violations_per_file,
        "hotspots": data.hotspots,
    })
}

fn format_csv_output() -> serde_json::Value {
    json!({
        "formatted_output": "file_path,violations,lines_of_code,defect_density\n",
        "content_type": "text/csv"
    })
}

fn format_table_output(data: &LintHotspotData) -> serde_json::Value {
    let mut table = String::from("Lint Hotspot Analysis\n");
    table.push_str("====================\n");
    table.push_str(&format!("Total files analyzed: {}\n", data.total_files));
    table.push_str(&format!("Total violations: {}\n", data.total_violations));
    table.push_str(&format!(
        "Average violations per file: {:.2}\n\n",
        data.average_violations_per_file
    ));
    table.push_str("No hotspots found.\n");

    json!({
        "formatted_output": table,
        "stats": {
            "total_files": data.total_files,
            "total_violations": data.total_violations,
            "average_violations_per_file": data.average_violations_per_file,
        }
    })
}

async fn handle_analyze_lint_hotspot(
    request_id: serde_json::Value,
    arguments: serde_json::Value,
) -> McpResponse {
    let args = match parse_lint_hotspot_args(arguments) {
        Ok(args) => args,
        Err(e) => return McpResponse::error(request_id, -32602, e),
    };

    info!(
        "Analyzing lint hotspots for project: {:?}",
        args.project_path
    );

    let project_path = std::path::PathBuf::from(args.project_path.clone());

    let output_path = match execute_lint_hotspot_analysis(&args, &project_path).await {
        Ok(path) => path,
        Err(e) => return McpResponse::error(request_id, -32603, e),
    };

    let lint_data = match read_and_parse_lint_output(&output_path).await {
        Ok(data) => data,
        Err(e) => return McpResponse::error(request_id, -32603, e),
    };

    let extracted_data = extract_lint_data(&lint_data);
    let output = format_lint_hotspot_output(&args, &extracted_data);

    McpResponse::success(request_id, output)
}
