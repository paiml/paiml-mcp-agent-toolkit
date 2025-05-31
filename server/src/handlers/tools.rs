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
        "analyze_system_architecture" => {
            handle_analyze_system_architecture(request.id, tool_params.arguments).await
        }
        "analyze_defect_probability" => {
            handle_analyze_defect_probability(request.id, tool_params.arguments).await
        }
        "analyze_dead_code" => handle_analyze_dead_code(request.id, tool_params.arguments).await,
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
    let args = match parse_validate_template_args(arguments) {
        Ok(args) => args,
        Err(e) => {
            return McpResponse::error(
                request_id,
                -32602,
                format!("Invalid validate_template arguments: {}", e),
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
            validation_errors.push(format!("Unknown parameter: {}", key));
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
                        "Parameter '{}' does not match pattern: {}",
                        key, pattern
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
                format!("Invalid analyze_complexity arguments: {}", e),
            );
        }
    };

    let project_path = resolve_project_path(args.project_path.clone());
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

fn resolve_project_path(project_path_arg: Option<String>) -> PathBuf {
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
    use walkdir::WalkDir;

    let mut file_metrics = Vec::new();
    let mut file_count = 0;

    for entry in WalkDir::new(project_path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if path.is_dir() || !should_analyze_file(path, toolchain) {
            continue;
        }

        if !matches_include_filters(path, &args.include) {
            continue;
        }

        file_count += 1;

        if let Some(metrics) = analyze_file_complexity(path, toolchain).await {
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
            use crate::services::ast_typescript;
            ast_typescript::analyze_typescript_file_with_complexity(path)
                .await
                .ok()
        }
        "python-uv" => {
            use crate::services::ast_python;
            ast_python::analyze_python_file_with_complexity(path)
                .await
                .ok()
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

#[derive(Debug, Deserialize, Serialize)]
struct AnalyzeSystemArchitectureArgs {
    project_path: Option<String>,
    format: Option<String>,
    show_complexity: Option<bool>,
}

async fn handle_analyze_system_architecture(
    request_id: serde_json::Value,
    arguments: serde_json::Value,
) -> McpResponse {
    let args: AnalyzeSystemArchitectureArgs = match serde_json::from_value(arguments) {
        Ok(a) => a,
        Err(e) => {
            return McpResponse::error(
                request_id,
                -32602,
                format!("Invalid analyze_system_architecture arguments: {}", e),
            );
        }
    };

    let project_path = args
        .project_path
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    info!("Analyzing system architecture for {:?}", project_path);

    use crate::services::canonical_query::{
        AnalysisContext, CallGraph, CanonicalQuery, SystemArchitectureQuery,
    };
    use crate::services::context::analyze_project;
    use crate::services::dag_builder::DagBuilder;
    use std::collections::HashMap;

    // Build analysis context
    let context_result = match analyze_project(&project_path, "rust").await {
        Ok(ctx) => ctx,
        Err(e) => {
            return McpResponse::error(
                request_id,
                -32000,
                format!("Failed to analyze project: {}", e),
            );
        }
    };

    let dag_result = DagBuilder::build_from_project(&context_result);

    // Convert to analysis context
    let context = AnalysisContext {
        project_path: project_path.clone(),
        ast_dag: dag_result,
        call_graph: CallGraph::default(), // TODO: Build actual call graph
        complexity_map: HashMap::new(),
        churn_analysis: None, // Optional
    };

    let query = SystemArchitectureQuery;
    match query.execute(&context) {
        Ok(result) => {
            let content_text = match args.format.as_deref() {
                Some("json") => serde_json::to_string_pretty(&result).unwrap_or_default(),
                _ => format!("# System Architecture Analysis\n\n{}", result.diagram),
            };

            let response = json!({
                "content": [{
                    "type": "text",
                    "text": content_text
                }],
                "result": result,
                "format": args.format.unwrap_or_else(|| "mermaid".to_string()),
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
                format!("Invalid analyze_defect_probability arguments: {}", e),
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
    use walkdir::WalkDir;

    let calculator = DefectProbabilityCalculator::new();
    let mut file_metrics = Vec::new();

    // Get complexity data for better defect probability calculation
    // (This is simplified - in real implementation we'd get complexity and churn data)
    for entry in WalkDir::new(&project_path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rs") {
            let relative_path = path
                .strip_prefix(&project_path)
                .unwrap_or(path)
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
                format!("Invalid analyze_dead_code arguments: {}", e),
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
                format!("Dead code analysis failed: {}", e),
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
            return McpResponse::error(
                request_id,
                -32000,
                format!("Failed to format output: {}", e),
            );
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
        output.push_str(&format!(
            "## Top {} Files with Most Dead Code\n\n",
            top_count
        ));

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
