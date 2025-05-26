use crate::models::mcp::{
    GenerateTemplateArgs, ListTemplatesArgs, McpRequest, McpResponse, ScaffoldProjectArgs,
    SearchTemplatesArgs, ToolCallParams, ValidateTemplateArgs,
};
use crate::services::template_service;
use crate::TemplateServerTrait;
use serde_json::json;
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
