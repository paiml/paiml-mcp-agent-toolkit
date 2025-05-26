use semver::Version;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateResource {
    pub uri: String,
    pub name: String,
    pub description: String,
    pub toolchain: Toolchain,
    pub category: TemplateCategory,
    pub parameters: Vec<ParameterSpec>,
    pub s3_object_key: String,
    pub content_hash: String,
    pub semantic_version: Version,
    pub dependency_graph: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Toolchain {
    #[serde(rename = "rust")]
    RustCli { cargo_features: Vec<String> },
    #[serde(rename = "deno")]
    DenoTypescript { deno_version: String },
    #[serde(rename = "python-uv")]
    PythonUv { python_version: String },
}

impl Toolchain {
    pub fn priority(&self) -> u8 {
        match self {
            Toolchain::RustCli { .. } => 1,
            Toolchain::DenoTypescript { .. } => 2,
            Toolchain::PythonUv { .. } => 3,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Toolchain::RustCli { .. } => "rust",
            Toolchain::DenoTypescript { .. } => "deno",
            Toolchain::PythonUv { .. } => "python-uv",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TemplateCategory {
    Makefile,
    Readme,
    Gitignore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSpec {
    pub name: String,
    pub param_type: ParameterType,
    pub required: bool,
    pub default_value: Option<String>,
    pub validation_pattern: Option<String>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParameterType {
    ProjectName,
    SemVer,
    GitHubUsername,
    LicenseIdentifier,
    Boolean,
    String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeneratedTemplate {
    pub content: String,
    pub filename: String,
    pub checksum: String,
    pub toolchain: Toolchain,
}
