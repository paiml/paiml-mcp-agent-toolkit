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
    let category_str = embedded.category.clone();
    let toolchain_str = embedded.toolchain.clone();
    let variant = embedded.variant.clone();

    let category = match category_str.as_str() {
        "makefile" => TemplateCategory::Makefile,
        "readme" => TemplateCategory::Readme,
        "gitignore" => TemplateCategory::Gitignore,
        _ => {
            return Err(TemplateError::InvalidUri {
                uri: format!("Unknown category: {}", category_str),
            })
        }
    };

    let toolchain = match toolchain_str.as_str() {
        "rust" => Toolchain::RustCli {
            cargo_features: vec![],
        },
        "deno" => Toolchain::DenoTypescript {
            deno_version: "1.46".to_string(),
        },
        "python-uv" => Toolchain::PythonUv {
            python_version: "3.12".to_string(),
        },
        _ => {
            return Err(TemplateError::InvalidUri {
                uri: format!("Unknown toolchain: {}", toolchain_str),
            })
        }
    };

    let parameters = embedded
        .parameters
        .into_iter()
        .map(|p| {
            let param_type = match p.param_type.as_str() {
                "project_name" => ParameterType::ProjectName,
                "boolean" => ParameterType::Boolean,
                "string" => ParameterType::String,
                "array" => ParameterType::String, // Arrays handled as strings for now
                _ => ParameterType::String,
            };

            ParameterSpec {
                name: p.name,
                param_type,
                required: p.required,
                default_value: p.default_value.map(|v| match v {
                    serde_json::Value::String(s) => s,
                    serde_json::Value::Bool(b) => b.to_string(),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Array(_) => "[]".to_string(),
                    _ => v.to_string(),
                }),
                validation_pattern: None,
                description: p.description,
            }
        })
        .collect();

    Ok(TemplateResource {
        uri: embedded.uri,
        name: embedded.name,
        description: embedded.description,
        toolchain: toolchain.clone(),
        category: category.clone(),
        parameters,
        s3_object_key: format!(
            "templates/{}/{}/{}.hbs",
            match &category {
                TemplateCategory::Makefile => "makefile",
                TemplateCategory::Readme => "readme",
                TemplateCategory::Gitignore => "gitignore",
                TemplateCategory::Context => "context",
            },
            toolchain.as_str(),
            variant
        ),
        content_hash: "embedded".to_string(),
        semantic_version: Version::new(1, 0, 0),
        dependency_graph: vec![],
    })
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
                "Template not found: {}",
                uri
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
                "Template content not found: {}",
                uri
            )))
        }
    };

    Ok(content.into())
}
