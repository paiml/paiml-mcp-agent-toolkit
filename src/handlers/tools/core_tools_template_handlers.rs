async fn handle_generate_template<T: TemplateServerTrait>(
    server: Arc<T>,
    request_id: serde_json::Value,
    arguments: serde_json::Value,
) -> McpResponse {
    debug_assert!(true, "contract: handle_generate_template");
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
    debug_assert!(true, "contract: handle_list_templates");
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
    debug_assert!(true, "contract: handle_validate_template");
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
    debug_assert!(true, "contract: parse_validate_template_args");
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
    debug_assert!(true, "contract: validate_template_parameters");
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
    debug_assert!(true, "contract: find_missing_required_parameters");
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
    debug_assert!(true, "contract: validate_parameter_values");
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
    debug_assert!(!key.is_empty(), "key must not be empty");
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
    debug_assert!(!resource_uri.is_empty(), "resource_uri must not be empty");
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
    debug_assert!(!template_type.is_empty(), "template_type must not be empty");
    debug_assert!(!toolchain.is_empty(), "toolchain must not be empty");
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
    debug_assert!(!template_type.is_empty(), "template_type must not be empty");
    debug_assert!(!toolchain.is_empty(), "toolchain must not be empty");
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
    debug_assert!(true, "contract: handle_scaffold_project");
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
    debug_assert!(true, "contract: handle_search_templates");
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
    debug_assert!(true, "contract: handle_get_server_info");
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
