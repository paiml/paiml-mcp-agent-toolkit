use crate::models::error::TemplateError;
use crate::models::template::{GeneratedTemplate, TemplateResource};
use crate::services::context::{analyze_project, format_context_as_markdown};
use crate::services::renderer;
use crate::TemplateServerTrait;
use serde_json::Map;
use sha2::{Digest, Sha256};
use std::path::Path;
use std::sync::Arc;

pub async fn get_template_content<T: TemplateServerTrait>(
    server: &T,
    uri: &str,
) -> Result<String, TemplateError> {
    server
        .get_template_content(uri)
        .await
        .map(|arc_str| arc_str.to_string())
        .map_err(|_| TemplateError::NotFound(format!("Template content not found: {}", uri)))
}

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
        .map_err(|_| TemplateError::NotFound(format!("Template content not found: {}", uri)))?;

    // Render template
    let rendered =
        renderer::render_template(server.get_renderer(), &template_content, parameters.clone())?;

    // Calculate checksum
    let mut hasher = Sha256::new();
    hasher.update(rendered.as_bytes());
    let checksum = hex::encode(hasher.finalize());

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
    let checksum = hex::encode(hasher.finalize());

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
                uri: format!("template://context/{}/ast", toolchain),
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

pub async fn list_templates<T: TemplateServerTrait>(
    server: &T,
    toolchain: Option<&str>,
    category: Option<&str>,
) -> Result<Vec<Arc<TemplateResource>>, TemplateError> {
    let prefix = build_template_prefix(category, toolchain);
    let mut templates = server.list_templates(&prefix).await.map_err(|_| {
        TemplateError::NotFound(format!("Failed to list templates with prefix: {}", prefix))
    })?;

    // Filter by toolchain if specified but category is not
    // (when both are specified, the prefix already handles filtering)
    if let Some(tc) = toolchain {
        if category.is_none() {
            templates.retain(|t| t.toolchain.as_str() == tc);
        }
    }

    // Sort by toolchain priority and version
    templates.sort_by(|a, b| {
        a.toolchain
            .priority()
            .cmp(&b.toolchain.priority())
            .then_with(|| b.semantic_version.cmp(&a.semantic_version))
    });

    Ok(templates)
}

pub async fn list_all_resources<T: TemplateServerTrait>(
    server: &T,
) -> Result<Vec<Arc<TemplateResource>>, TemplateError> {
    list_templates(server, None, None).await
}

fn parse_template_uri(uri: &str) -> Result<(&str, &str, &str), TemplateError> {
    let parts: Vec<&str> = uri
        .strip_prefix("template://")
        .ok_or_else(|| TemplateError::InvalidUri {
            uri: uri.to_string(),
        })?
        .split('/')
        .collect();

    if parts.len() != 3 {
        return Err(TemplateError::InvalidUri {
            uri: uri.to_string(),
        });
    }

    Ok((parts[0], parts[1], parts[2]))
}

fn build_template_prefix(category: Option<&str>, toolchain: Option<&str>) -> String {
    match (category, toolchain) {
        (None, None) => String::new(), // Empty prefix to match all
        (Some(cat), None) => format!("{}/", cat),
        (Some(cat), Some(tc)) => format!("{}/{}/", cat, tc),
        (None, Some(_)) => String::new(), // Invalid case, return empty
    }
}

fn extract_filename(category: &str) -> String {
    match category {
        "makefile" => "Makefile".to_string(),
        "readme" => "README.md".to_string(),
        "gitignore" => ".gitignore".to_string(),
        _ => format!("{}.txt", category),
    }
}

fn validate_parameters(
    specs: &[crate::models::template::ParameterSpec],
    provided: &Map<String, serde_json::Value>,
) -> Result<(), TemplateError> {
    for spec in specs {
        if spec.required && !provided.contains_key(&spec.name) {
            return Err(TemplateError::ValidationError {
                parameter: spec.name.clone(),
                reason: "required parameter missing".to_string(),
            });
        }

        if let Some(value) = provided.get(&spec.name) {
            // Add validation logic based on param_type
            if let Some(pattern) = &spec.validation_pattern {
                // Validate against regex pattern
                let regex =
                    regex::Regex::new(pattern).map_err(|_| TemplateError::ValidationError {
                        parameter: spec.name.clone(),
                        reason: "invalid validation pattern".to_string(),
                    })?;

                if let Some(str_value) = value.as_str() {
                    if !regex.is_match(str_value) {
                        return Err(TemplateError::ValidationError {
                            parameter: spec.name.clone(),
                            reason: format!("value does not match pattern: {}", pattern),
                        });
                    }
                }
            }
        }
    }

    Ok(())
}

// Additional functions for CLI support
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

        let uri = format!("template://{}/{}/{}", template_type, toolchain, variant);

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

pub async fn search_templates<T: TemplateServerTrait>(
    server: Arc<T>,
    query: &str,
    toolchain: Option<&str>,
) -> Result<Vec<SearchResult>, TemplateError> {
    let templates = list_templates(server.as_ref(), toolchain, None).await?;
    let query_lower = query.to_lowercase();

    let mut results: Vec<SearchResult> = templates
        .into_iter()
        .filter_map(|template| {
            let mut matches = Vec::new();
            let mut relevance = 0.0;

            // Check name
            if template.name.to_lowercase().contains(&query_lower) {
                matches.push(format!("name: {}", template.name));
                relevance += if template.name.to_lowercase() == query_lower {
                    10.0
                } else {
                    5.0
                };
            }

            // Check description
            if template.description.to_lowercase().contains(&query_lower) {
                matches.push("description".to_string());
                relevance += 3.0;
            }

            // Check parameter names
            for param in &template.parameters {
                if param.name.to_lowercase().contains(&query_lower) {
                    matches.push(format!("parameter: {}", param.name));
                    relevance += 1.0;
                }
            }

            if !matches.is_empty() {
                Some(SearchResult {
                    template: (*template).clone(),
                    relevance,
                    matches,
                })
            } else {
                None
            }
        })
        .collect();

    // Sort by relevance (highest first)
    results.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap());

    Ok(results)
}

pub async fn validate_template<T: TemplateServerTrait>(
    server: Arc<T>,
    uri: &str,
    parameters: &serde_json::Value,
) -> Result<ValidationResult, TemplateError> {
    let metadata =
        server
            .get_template_metadata(uri)
            .await
            .map_err(|_| TemplateError::TemplateNotFound {
                uri: uri.to_string(),
            })?;

    let mut errors = Vec::new();

    // Convert parameters to Map if it's an object
    let params_map = if let serde_json::Value::Object(map) = parameters {
        map
    } else {
        errors.push(ValidationError {
            field: "parameters".to_string(),
            message: "Parameters must be an object".to_string(),
        });
        return Ok(ValidationResult {
            valid: false,
            errors,
        });
    };

    // Check required parameters
    for param in &metadata.parameters {
        if param.required && !params_map.contains_key(&param.name) {
            errors.push(ValidationError {
                field: param.name.clone(),
                message: "Required parameter missing".to_string(),
            });
        }
    }

    // Validate parameter values
    for (key, value) in params_map {
        if let Some(param_spec) = metadata.parameters.iter().find(|p| p.name == *key) {
            if let Some(pattern) = &param_spec.validation_pattern {
                if let Ok(regex) = regex::Regex::new(pattern) {
                    if let Some(str_val) = value.as_str() {
                        if !regex.is_match(str_val) {
                            errors.push(ValidationError {
                                field: key.clone(),
                                message: format!("Does not match pattern: {}", pattern),
                            });
                        }
                    }
                }
            }
        } else {
            errors.push(ValidationError {
                field: key.clone(),
                message: "Unknown parameter".to_string(),
            });
        }
    }

    Ok(ValidationResult {
        valid: errors.is_empty(),
        errors,
    })
}

// Result types for CLI
#[derive(Debug)]
pub struct ScaffoldResult {
    pub files: Vec<GeneratedFile>,
    pub errors: Vec<ScaffoldError>,
}

#[derive(Debug)]
pub struct GeneratedFile {
    pub path: String,
    pub content: String,
    pub checksum: String,
}

#[derive(Debug)]
pub struct ScaffoldError {
    pub template: String,
    pub error: String,
}

#[derive(Debug)]
pub struct SearchResult {
    pub template: crate::models::template::TemplateResource,
    pub relevance: f32,
    pub matches: Vec<String>,
}

#[derive(Debug)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
}

#[derive(Debug)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}
