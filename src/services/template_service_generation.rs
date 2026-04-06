// Template generation and context creation functions.
// Includes generate_template, generate_context, get_template_content, and scaffold_project.

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn get_template_content<T: TemplateServerTrait>(
    server: &T,
    uri: &str,
) -> Result<String, TemplateError> {
    server
        .get_template_content(uri)
        .await
        .map(|arc_str| arc_str.to_string())
        .map_err(|_| TemplateError::NotFound(format!("Template content not found: {uri}")))
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn generate_template<T: TemplateServerTrait>(
    server: &T,
    uri: &str,
    parameters: Map<String, serde_json::Value>,
) -> Result<GeneratedTemplate, TemplateError> {
    // Parse and validate URI
    let (category, toolchain, _variant) = parse_template_uri(uri)?;

    // Handle context generation separately
    if category == "context" {
        return generate_context(toolchain, parameters).await;
    }

    // Get template metadata
    let metadata =
        server
            .get_template_metadata(uri)
            .await
            .map_err(|_| TemplateError::TemplateNotFound {
                uri: uri.to_string(),
            })?;

    // Validate parameters
    validate_parameters(&metadata.parameters, &parameters)?;

    // Get template content - using the URI instead of s3_object_key
    let template_content = server
        .get_template_content(uri)
        .await
        .map_err(|_| TemplateError::NotFound(format!("Template content not found: {uri}")))?;

    // Render template
    let rendered =
        renderer::render_template(server.get_renderer(), &template_content, parameters.clone())?;

    // Calculate checksum
    let mut hasher = Sha256::new();
    hasher.update(rendered.as_bytes());
    let checksum = hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();

    // Extract project_name to use as subdirectory
    let project_name = parameters
        .get("project_name")
        .and_then(|v| v.as_str())
        .unwrap_or("project");

    Ok(GeneratedTemplate {
        content: rendered,
        filename: format!("{}/{}", project_name, extract_filename(category)),
        checksum,
        toolchain: metadata.toolchain.clone(),
    })
}

async fn generate_context(
    toolchain: &str,
    parameters: Map<String, serde_json::Value>,
) -> Result<GeneratedTemplate, TemplateError> {
    // Get project path from parameters
    let project_path = parameters
        .get("project_path")
        .and_then(|v| v.as_str())
        .unwrap_or(".");

    // Analyze the project
    let context = analyze_project(Path::new(project_path), toolchain).await?;

    // Format as markdown
    let content = format_context_as_markdown(&context);

    // Calculate checksum
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let checksum = hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();

    // Create toolchain enum
    let toolchain_enum = match toolchain {
        "rust" => crate::models::template::Toolchain::RustCli {
            cargo_features: vec![],
        },
        "deno" => crate::models::template::Toolchain::DenoTypescript {
            deno_version: "1.46".to_string(),
        },
        "python-uv" => crate::models::template::Toolchain::PythonUv {
            python_version: "3.12".to_string(),
        },
        _ => {
            return Err(TemplateError::InvalidUri {
                uri: format!("template://context/{toolchain}/ast"),
            })
        }
    };

    Ok(GeneratedTemplate {
        content,
        filename: "CONTEXT.md".to_string(),
        checksum,
        toolchain: toolchain_enum,
    })
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn scaffold_project<T: TemplateServerTrait>(
    server: Arc<T>,
    toolchain: &str,
    templates: Vec<String>,
    parameters: serde_json::Value,
) -> Result<ScaffoldResult, TemplateError> {
    let mut files = Vec::new();
    let mut errors = Vec::new();

    // Convert parameters to Map if it's an object
    let params_map = if let serde_json::Value::Object(map) = parameters {
        map
    } else {
        return Err(TemplateError::ValidationError {
            parameter: "parameters".to_string(),
            reason: "Parameters must be an object".to_string(),
        });
    };

    // Generate each requested template
    for template_type in &templates {
        let variant = match template_type.as_str() {
            "makefile" | "readme" | "gitignore" => match toolchain {
                "rust" | "deno" | "python-uv" => "cli",
                _ => continue,
            },
            _ => continue,
        };

        let uri = format!("template://{template_type}/{toolchain}/{variant}");

        match generate_template(server.as_ref(), &uri, params_map.clone()).await {
            Ok(generated) => {
                files.push(GeneratedFile {
                    path: generated.filename,
                    content: generated.content,
                    checksum: generated.checksum,
                });
            }
            Err(e) => {
                errors.push(ScaffoldError {
                    template: template_type.clone(),
                    error: e.to_string(),
                });
            }
        }
    }

    Ok(ScaffoldResult { files, errors })
}
