#![cfg_attr(coverage_nightly, coverage(off))]
use semver::Version;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Template resource.
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
/// Toolchain.
pub enum Toolchain {
    #[serde(rename = "rust")]
    RustCli { cargo_features: Vec<String> },
    #[serde(rename = "deno")]
    DenoTypescript { deno_version: String },
    #[serde(rename = "python-uv")]
    PythonUv { python_version: String },
}

impl Toolchain {
    /// Returns the priority of the toolchain (lower number = higher priority)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::template::Toolchain;
    ///
    /// let rust = Toolchain::RustCli {
    ///     cargo_features: vec!["serde".to_string()]
    /// };
    /// assert_eq!(rust.priority(), 1);
    ///
    /// let deno = Toolchain::DenoTypescript {
    ///     deno_version: "1.38".to_string()
    /// };
    /// assert_eq!(deno.priority(), 2);
    /// ```
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn priority(&self) -> u8 {
        match self {
            Toolchain::RustCli { .. } => 1,
            Toolchain::DenoTypescript { .. } => 2,
            Toolchain::PythonUv { .. } => 3,
        }
    }

    /// Returns the string identifier for the toolchain
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::template::Toolchain;
    ///
    /// let rust = Toolchain::RustCli {
    ///     cargo_features: vec![]
    /// };
    /// assert_eq!(rust.as_str(), "rust");
    ///
    /// let python = Toolchain::PythonUv {
    ///     python_version: "3.11".to_string()
    /// };
    /// assert_eq!(python.as_str(), "python-uv");
    /// ```
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn as_str(&self) -> &'static str {
        match self {
            Toolchain::RustCli { .. } => "rust",
            Toolchain::DenoTypescript { .. } => "deno",
            Toolchain::PythonUv { .. } => "python-uv",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
/// Category classification for template.
pub enum TemplateCategory {
    Makefile,
    Readme,
    Gitignore,
    Context,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Parameter spec.
pub struct ParameterSpec {
    pub name: String,
    pub param_type: ParameterType,
    pub required: bool,
    pub default_value: Option<String>,
    pub validation_pattern: Option<String>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
/// Type classification for parameter.
pub enum ParameterType {
    ProjectName,
    SemVer,
    GitHubUsername,
    LicenseIdentifier,
    Boolean,
    String,
}

#[derive(Debug, Serialize, Deserialize)]
/// Template for generated generation.
pub struct GeneratedTemplate {
    pub content: String,
    pub filename: String,
    pub checksum: String,
    pub toolchain: Toolchain,
}

#[derive(Debug, Serialize, Deserialize)]
/// Response from template operation.
pub struct TemplateResponse {
    pub content: String,
}

// --- Test modules (split into include files) ---
include!("template_tests.rs");
include!("template_property_tests.rs");
include!("template_coverage_tests.rs");
