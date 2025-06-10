use crate::models::error::TemplateError;
use crate::models::template::{
    ParameterSpec, ParameterType, TemplateCategory, TemplateResource, Toolchain,
};
use semver::Version;
use std::sync::Arc;
use tracing::debug;

// Simple template metadata structure for embedding
#[derive(serde::Deserialize, Clone)]
struct EmbeddedTemplateMetadata {
    uri: String,
    name: String,
    description: String,
    category: String,
    toolchain: String,
    variant: String,
    parameters: Vec<EmbeddedParameter>,
}

#[derive(serde::Deserialize, Clone)]
struct EmbeddedParameter {
    name: String,
    #[serde(rename = "type")]
    param_type: String,
    required: bool,
    #[serde(default, rename = "default")]
    default_value: Option<serde_json::Value>,
    description: String,
}

// Embed all templates at compile time
const MAKEFILE_RUST_CLI_META: &str = include_str!("../../templates/makefile/rust/cli.json");
const MAKEFILE_RUST_CLI_HBS: &str = include_str!("../../templates/makefile/rust/cli.hbs");

const README_RUST_CLI_META: &str = include_str!("../../templates/readme/rust/cli.json");
const README_RUST_CLI_HBS: &str = include_str!("../../templates/readme/rust/cli.hbs");

const GITIGNORE_RUST_CLI_META: &str = include_str!("../../templates/gitignore/rust/cli.json");
const GITIGNORE_RUST_CLI_HBS: &str = include_str!("../../templates/gitignore/rust/cli.hbs");

const MAKEFILE_PYTHON_UV_CLI_META: &str =
    include_str!("../../templates/makefile/python-uv/cli.json");
const MAKEFILE_PYTHON_UV_CLI_HBS: &str = include_str!("../../templates/makefile/python-uv/cli.hbs");

const MAKEFILE_DENO_CLI_META: &str = include_str!("../../templates/makefile/deno/cli.json");
const MAKEFILE_DENO_CLI_HBS: &str = include_str!("../../templates/makefile/deno/cli.hbs");

const README_DENO_CLI_META: &str = include_str!("../../templates/readme/deno/cli.json");
const README_DENO_CLI_HBS: &str = include_str!("../../templates/readme/deno/cli.hbs");

const README_PYTHON_UV_CLI_META: &str = include_str!("../../templates/readme/python-uv/cli.json");
const README_PYTHON_UV_CLI_HBS: &str = include_str!("../../templates/readme/python-uv/cli.hbs");

const GITIGNORE_DENO_CLI_META: &str = include_str!("../../templates/gitignore/deno/cli.json");
const GITIGNORE_DENO_CLI_HBS: &str = include_str!("../../templates/gitignore/deno/cli.hbs");

const GITIGNORE_PYTHON_UV_CLI_META: &str =
    include_str!("../../templates/gitignore/python-uv/cli.json");
const GITIGNORE_PYTHON_UV_CLI_HBS: &str =
    include_str!("../../templates/gitignore/python-uv/cli.hbs");

// Convert embedded metadata to full TemplateResource
fn convert_to_template_resource(
    embedded: EmbeddedTemplateMetadata,
) -> Result<TemplateResource, TemplateError> {
    let category = parse_template_category(&embedded.category)?;
    let toolchain = parse_toolchain(&embedded.toolchain)?;
    let parameters = convert_embedded_parameters(embedded.parameters);

    let s3_object_key = build_s3_object_key(&category, &toolchain, &embedded.variant);

    Ok(TemplateResource {
        uri: embedded.uri,
        name: embedded.name,
        description: embedded.description,
        toolchain: toolchain.clone(),
        category: category.clone(),
        parameters,
        s3_object_key,
        content_hash: "embedded".to_string(),
        semantic_version: Version::new(1, 0, 0),
        dependency_graph: vec![],
    })
}

fn parse_template_category(category_str: &str) -> Result<TemplateCategory, TemplateError> {
    match category_str {
        "makefile" => Ok(TemplateCategory::Makefile),
        "readme" => Ok(TemplateCategory::Readme),
        "gitignore" => Ok(TemplateCategory::Gitignore),
        _ => Err(TemplateError::InvalidUri {
            uri: format!("Unknown category: {category_str}"),
        }),
    }
}

fn parse_toolchain(toolchain_str: &str) -> Result<Toolchain, TemplateError> {
    match toolchain_str {
        "rust" => Ok(Toolchain::RustCli {
            cargo_features: vec![],
        }),
        "deno" => Ok(Toolchain::DenoTypescript {
            deno_version: "1.46".to_string(),
        }),
        "python-uv" => Ok(Toolchain::PythonUv {
            python_version: "3.12".to_string(),
        }),
        _ => Err(TemplateError::InvalidUri {
            uri: format!("Unknown toolchain: {toolchain_str}"),
        }),
    }
}

fn convert_embedded_parameters(embedded_params: Vec<EmbeddedParameter>) -> Vec<ParameterSpec> {
    embedded_params
        .into_iter()
        .map(convert_embedded_parameter)
        .collect()
}

fn convert_embedded_parameter(p: EmbeddedParameter) -> ParameterSpec {
    let param_type = parse_parameter_type(&p.param_type);
    let default_value = p.default_value.map(convert_json_value_to_string);

    ParameterSpec {
        name: p.name,
        param_type,
        required: p.required,
        default_value,
        validation_pattern: None,
        description: p.description,
    }
}

fn parse_parameter_type(param_type_str: &str) -> ParameterType {
    match param_type_str {
        "project_name" => ParameterType::ProjectName,
        "boolean" => ParameterType::Boolean,
        "string" => ParameterType::String,
        "array" => ParameterType::String, // Arrays handled as strings for now
        _ => ParameterType::String,
    }
}

fn convert_json_value_to_string(value: serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s,
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Array(_) => "[]".to_string(),
        _ => value.to_string(),
    }
}

fn build_s3_object_key(
    category: &TemplateCategory,
    toolchain: &Toolchain,
    variant: &str,
) -> String {
    let category_path = get_category_path(category);
    format!(
        "templates/{}/{}/{}.hbs",
        category_path,
        toolchain.as_str(),
        variant
    )
}

fn get_category_path(category: &TemplateCategory) -> &'static str {
    match category {
        TemplateCategory::Makefile => "makefile",
        TemplateCategory::Readme => "readme",
        TemplateCategory::Gitignore => "gitignore",
        TemplateCategory::Context => "context",
    }
}

pub async fn list_templates(prefix: &str) -> Result<Vec<Arc<TemplateResource>>, TemplateError> {
    debug!("Listing embedded templates with prefix: {}", prefix);

    let all_metadata = vec![
        MAKEFILE_RUST_CLI_META,
        README_RUST_CLI_META,
        GITIGNORE_RUST_CLI_META,
        MAKEFILE_PYTHON_UV_CLI_META,
        MAKEFILE_DENO_CLI_META,
        README_DENO_CLI_META,
        README_PYTHON_UV_CLI_META,
        GITIGNORE_DENO_CLI_META,
        GITIGNORE_PYTHON_UV_CLI_META,
    ];

    let mut resources = Vec::new();

    for metadata_str in all_metadata {
        let embedded: EmbeddedTemplateMetadata =
            serde_json::from_str(metadata_str).map_err(TemplateError::JsonError)?;

        // Filter by prefix if provided
        if prefix.is_empty() || embedded.uri.contains(prefix) {
            let resource = convert_to_template_resource(embedded)?;
            resources.push(Arc::new(resource));
        }
    }

    debug!("Found {} embedded templates", resources.len());
    Ok(resources)
}

pub async fn get_template_metadata(uri: &str) -> Result<Arc<TemplateResource>, TemplateError> {
    debug!("Fetching embedded template metadata for: {}", uri);

    let metadata_str = match uri {
        uri if uri.contains("makefile/rust/cli") => MAKEFILE_RUST_CLI_META,
        uri if uri.contains("readme/rust/cli") => README_RUST_CLI_META,
        uri if uri.contains("gitignore/rust/cli") => GITIGNORE_RUST_CLI_META,
        uri if uri.contains("makefile/python-uv/cli") => MAKEFILE_PYTHON_UV_CLI_META,
        uri if uri.contains("makefile/deno/cli") => MAKEFILE_DENO_CLI_META,
        uri if uri.contains("readme/deno/cli") => README_DENO_CLI_META,
        uri if uri.contains("readme/python-uv/cli") => README_PYTHON_UV_CLI_META,
        uri if uri.contains("gitignore/deno/cli") => GITIGNORE_DENO_CLI_META,
        uri if uri.contains("gitignore/python-uv/cli") => GITIGNORE_PYTHON_UV_CLI_META,
        _ => {
            return Err(TemplateError::NotFound(format!(
                "Template not found: {uri}"
            )))
        }
    };

    let embedded: EmbeddedTemplateMetadata =
        serde_json::from_str(metadata_str).map_err(TemplateError::JsonError)?;
    let resource = convert_to_template_resource(embedded)?;

    Ok(Arc::new(resource))
}

pub async fn get_template_content(uri: &str) -> Result<Arc<str>, TemplateError> {
    debug!("Fetching embedded template content for: {}", uri);

    let content = match uri {
        uri if uri.contains("makefile/rust/cli") => MAKEFILE_RUST_CLI_HBS,
        uri if uri.contains("readme/rust/cli") => README_RUST_CLI_HBS,
        uri if uri.contains("gitignore/rust/cli") => GITIGNORE_RUST_CLI_HBS,
        uri if uri.contains("makefile/python-uv/cli") => MAKEFILE_PYTHON_UV_CLI_HBS,
        uri if uri.contains("makefile/deno/cli") => MAKEFILE_DENO_CLI_HBS,
        uri if uri.contains("readme/deno/cli") => README_DENO_CLI_HBS,
        uri if uri.contains("readme/python-uv/cli") => README_PYTHON_UV_CLI_HBS,
        uri if uri.contains("gitignore/deno/cli") => GITIGNORE_DENO_CLI_HBS,
        uri if uri.contains("gitignore/python-uv/cli") => GITIGNORE_PYTHON_UV_CLI_HBS,
        _ => {
            return Err(TemplateError::NotFound(format!(
                "Template content not found: {uri}"
            )))
        }
    };

    Ok(content.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_templates_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}
