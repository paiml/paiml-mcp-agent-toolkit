use crate::models::error::TemplateError;
use crate::models::template::{GeneratedTemplate, TemplateResource};
use crate::services::renderer;
use crate::TemplateServerTrait;
use serde_json::Map;
use sha2::{Digest, Sha256};
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
    let (category, _toolchain, _variant) = parse_template_uri(uri)?;

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
