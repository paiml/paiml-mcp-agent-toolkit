#![cfg_attr(coverage_nightly, coverage(off))]
//! Template registry for managing agent templates.

use super::error::{ScaffoldError, ScaffoldResult};
use super::generator::TemplateGenerator;
use super::templates::{AgentTemplate, MCPServerTemplate, StateMachineTemplate};
use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use url::Url;

/// Registry for agent templates.
pub struct TemplateRegistry {
    /// Built-in templates.
    builtin: HashMap<String, Arc<dyn TemplateGenerator>>,
    /// Custom templates from filesystem.
    custom: HashMap<String, PathBuf>,
    /// Remote templates from URLs.
    remote: HashMap<String, Url>,
}

impl TemplateRegistry {
    /// Create a new template registry with built-in templates.
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        let mut builtin = HashMap::new();

        // Register built-in templates
        builtin.insert(
            "mcp-server".to_string(),
            Arc::new(MCPServerTemplate::default()) as Arc<dyn TemplateGenerator>,
        );
        builtin.insert(
            "state-machine".to_string(),
            Arc::new(StateMachineTemplate::default()) as Arc<dyn TemplateGenerator>,
        );
        builtin.insert(
            "calculator".to_string(),
            Arc::new(DeterministicCalculatorTemplate::default()) as Arc<dyn TemplateGenerator>,
        );
        builtin.insert(
            "hybrid".to_string(),
            Arc::new(HybridAnalyzerTemplate::default()) as Arc<dyn TemplateGenerator>,
        );

        Self {
            builtin,
            custom: HashMap::new(),
            remote: HashMap::new(),
        }
    }

    /// Get a template generator by template type.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get(&self, template: &AgentTemplate) -> ScaffoldResult<Arc<dyn TemplateGenerator>> {
        match template {
            AgentTemplate::MCPToolServer => self
                .builtin
                .get("mcp-server")
                .cloned()
                .ok_or_else(|| ScaffoldError::TemplateNotFound("mcp-server".to_string())),
            AgentTemplate::StateMachineWorkflow => self
                .builtin
                .get("state-machine")
                .cloned()
                .ok_or_else(|| ScaffoldError::TemplateNotFound("state-machine".to_string())),
            AgentTemplate::DeterministicCalculator => self
                .builtin
                .get("calculator")
                .cloned()
                .ok_or_else(|| ScaffoldError::TemplateNotFound("calculator".to_string())),
            AgentTemplate::HybridAnalyzer => self
                .builtin
                .get("hybrid")
                .cloned()
                .ok_or_else(|| ScaffoldError::TemplateNotFound("hybrid".to_string())),
            AgentTemplate::CustomAgent(path) => self.load_custom_template(path),
        }
    }

    /// List all available template names.
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn list_available(&self) -> Vec<String> {
        let mut templates: Vec<String> = self.builtin.keys().cloned().collect();
        templates.extend(self.custom.keys().cloned());
        templates.extend(self.remote.keys().cloned());
        templates.sort();
        templates
    }

    /// Register a custom template from a filesystem path.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn register_custom(&mut self, name: impl Into<String>, path: impl Into<PathBuf>) {
        self.custom.insert(name.into(), path.into());
    }

    /// Register a remote template from a URL.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn register_remote(&mut self, name: impl Into<String>, url: Url) {
        self.remote.insert(name.into(), url);
    }

    /// Fetch a remote template.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn fetch_remote(&self, name: &str) -> ScaffoldResult<Arc<dyn TemplateGenerator>> {
        let url = self
            .remote
            .get(name)
            .ok_or_else(|| ScaffoldError::TemplateNotFound(format!("Remote template '{name}'")))?;

        // Remote templates require network access which is disabled for security
        // Templates should be installed locally or use built-in templates
        Err(ScaffoldError::NetworkError(format!(
            "Remote template '{url}' requires network access. Please use a local or built-in template"
        )))
    }

    /// Validate a template file.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn validate_template_file(&self, path: &Path) -> Result<()> {
        if !path.exists() {
            return Err(ScaffoldError::TemplateNotFound(format!("{}", path.display())).into());
        }

        // In a real implementation, this would validate the template structure
        Ok(())
    }

    /// Load a custom template from a path.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    fn load_custom_template(&self, path: &Path) -> ScaffoldResult<Arc<dyn TemplateGenerator>> {
        if !path.exists() {
            return Err(ScaffoldError::TemplateNotFound(format!(
                "{}",
                path.display()
            )));
        }

        // Load custom template from filesystem
        // Currently returns a basic template generator for custom paths
        if path.is_file()
            && path
                .extension()
                .is_some_and(|ext| ext == "toml" || ext == "yaml")
        {
            // Return a placeholder error for custom templates
            // Custom templates would be loaded and parsed here in production
            Err(ScaffoldError::InvalidTemplate(format!(
                "Custom template support requires template file parsing: {}",
                path.display()
            )))
        } else {
            Err(ScaffoldError::InvalidTemplate(format!(
                "Template file must be a TOML or YAML file: {}",
                path.display()
            )))
        }
    }

    /// Check if a template exists.
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn has_template(&self, name: &str) -> bool {
        self.builtin.contains_key(name)
            || self.custom.contains_key(name)
            || self.remote.contains_key(name)
    }

    /// Get template metadata.
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_template_info(&self, name: &str) -> Option<TemplateInfo> {
        self.builtin.get(name).map(|gen| TemplateInfo {
            name: gen.name().to_string(),
            description: gen.description().to_string(),
            source: TemplateSource::Builtin,
        })
    }
}

impl Default for TemplateRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about a template.
#[derive(Debug, Clone)]
pub struct TemplateInfo {
    /// Template name.
    pub name: String,
    /// Template description.
    pub description: String,
    /// Template source.
    pub source: TemplateSource,
}

/// Source of a template.
#[derive(Debug, Clone)]
pub enum TemplateSource {
    /// Built-in template.
    Builtin,
    /// Custom template from filesystem.
    Custom(PathBuf),
    /// Remote template from URL.
    Remote(Url),
}

// Placeholder implementations for additional templates

use super::context::AgentContext;
use super::generator::GeneratedFiles;
use async_trait::async_trait;

/// Deterministic calculator template.
struct DeterministicCalculatorTemplate {
    name: String,
    description: String,
}

impl Default for DeterministicCalculatorTemplate {
    fn default() -> Self {
        Self {
            name: "calculator".to_string(),
            description: "Deterministic calculator agent with verified operations".to_string(),
        }
    }
}

#[async_trait]
impl TemplateGenerator for DeterministicCalculatorTemplate {
    fn generate(&self, ctx: &AgentContext) -> Result<GeneratedFiles> {
        let mut files = GeneratedFiles::new();

        // Generate basic calculator template
        files.add_text_file(
            "Cargo.toml",
            format!(
                r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1.0"
"#,
                ctx.name
            ),
        );

        files.add_text_file(
            "src/main.rs",
            format!(
                r#"//! {} - Deterministic Calculator Agent

fn main() {{
    println!("Calculator agent: {{}}", "{}");
}}
"#,
                ctx.name, ctx.name
            ),
        );

        Ok(files)
    }

    fn validate_context(&self, ctx: &AgentContext) -> Result<()> {
        if ctx.name.is_empty() {
            anyhow::bail!("Agent name is required");
        }
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }
}

/// Hybrid analyzer template.
struct HybridAnalyzerTemplate {
    name: String,
    description: String,
}

impl Default for HybridAnalyzerTemplate {
    fn default() -> Self {
        Self {
            name: "hybrid".to_string(),
            description: "Hybrid agent with deterministic core and probabilistic wrapper"
                .to_string(),
        }
    }
}

#[async_trait]
impl TemplateGenerator for HybridAnalyzerTemplate {
    fn generate(&self, ctx: &AgentContext) -> Result<GeneratedFiles> {
        if ctx.deterministic_core.is_none() || ctx.probabilistic_wrapper.is_none() {
            anyhow::bail!("Hybrid agents require both deterministic core and probabilistic wrapper specifications");
        }

        let mut files = GeneratedFiles::new();

        // Generate hybrid agent template
        files.add_text_file(
            "Cargo.toml",
            format!(
                r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1.0"
async-trait = "0.1"
tokio = {{ version = "1.40", features = ["full"] }}
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
"#,
                ctx.name
            ),
        );

        files.add_text_file(
            "src/main.rs",
            format!(
                r#"//! {} - Hybrid Analyzer Agent

#[tokio::main]
async fn main() {{
    println!("Hybrid analyzer agent: {{}}", "{}");
}}
"#,
                ctx.name, ctx.name
            ),
        );

        files.add_text_file(
            "src/core.rs",
            r#"//! Deterministic core implementation.

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
/// Deterministic analyze.
pub fn deterministic_analyze(input: &str) -> String {
    // Deterministic implementation
    format!("Analyzed: {}", input)
}
"#
            .to_string(),
        );

        files.add_text_file(
            "src/wrapper.rs",
            r#"//! Probabilistic wrapper implementation.

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn probabilistic_enhance(input: &str) -> String {
    // Probabilistic enhancement
    format!("Enhanced: {}", input)
}
"#
            .to_string(),
        );

        Ok(files)
    }

    fn validate_context(&self, ctx: &AgentContext) -> Result<()> {
        if ctx.name.is_empty() {
            anyhow::bail!("Agent name is required");
        }
        if ctx.deterministic_core.is_none() || ctx.probabilistic_wrapper.is_none() {
            anyhow::bail!("Hybrid agents require both core and wrapper specifications");
        }
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::scaffold::agent::templates::AgentTemplate;

    #[test]
    fn test_registry_creation() {
        let registry = TemplateRegistry::new();
        assert!(registry.has_template("mcp-server"));
        assert!(registry.has_template("state-machine"));
        assert!(registry.has_template("calculator"));
        assert!(registry.has_template("hybrid"));
    }

    #[test]
    fn test_list_available() {
        let registry = TemplateRegistry::new();
        let templates = registry.list_available();
        assert!(templates.contains(&"mcp-server".to_string()));
        assert!(templates.contains(&"state-machine".to_string()));
        assert_eq!(templates.len(), 4);
    }

    #[test]
    fn test_get_template() {
        let registry = TemplateRegistry::new();
        let result = registry.get(&AgentTemplate::MCPToolServer);
        assert!(result.is_ok());

        let result = registry.get(&AgentTemplate::StateMachineWorkflow);
        assert!(result.is_ok());
    }

    #[test]
    fn test_register_custom() {
        let mut registry = TemplateRegistry::new();
        registry.register_custom("my-template", "/path/to/template");
        assert!(registry.has_template("my-template"));
    }

    #[test]
    fn test_register_remote() {
        let mut registry = TemplateRegistry::new();
        let url = Url::parse("https://example.com/template").unwrap();
        registry.register_remote("remote-template", url);
        assert!(registry.has_template("remote-template"));
    }

    #[test]
    fn test_template_info() {
        let registry = TemplateRegistry::new();
        let info = registry.get_template_info("mcp-server");
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.name, "mcp-server");
        assert!(matches!(info.source, TemplateSource::Builtin));
    }

    // --- PMAT-635 additions: cover TemplateRegistry + embedded templates ---

    use crate::scaffold::agent::context::AgentContext;
    use crate::scaffold::agent::features::QualityLevel;
    use crate::scaffold::agent::generator::FileContent;
    use crate::scaffold::agent::hybrid::{CoreSpec, WrapperSpec};

    fn text_file<'a>(files: &'a GeneratedFiles, path: &str) -> &'a str {
        match files.files.get(Path::new(path)) {
            Some(FileContent::Text(s)) => s.as_str(),
            _ => panic!("expected text file at {path}"),
        }
    }

    fn scaffold_err<T>(r: ScaffoldResult<T>) -> ScaffoldError {
        match r {
            Ok(_) => panic!("expected ScaffoldError, got Ok"),
            Err(e) => e,
        }
    }

    fn anyhow_err<T>(r: Result<T>) -> anyhow::Error {
        match r {
            Ok(_) => panic!("expected anyhow::Error, got Ok"),
            Err(e) => e,
        }
    }

    fn ctx_named(name: &str) -> AgentContext {
        AgentContext {
            name: name.to_string(),
            template_type: AgentTemplate::MCPToolServer,
            features: std::collections::HashSet::new(),
            quality_level: QualityLevel::Standard,
            deterministic_core: None,
            probabilistic_wrapper: None,
        }
    }

    fn ctx_hybrid(name: &str) -> AgentContext {
        AgentContext {
            name: name.to_string(),
            template_type: AgentTemplate::HybridAnalyzer,
            features: std::collections::HashSet::new(),
            quality_level: QualityLevel::Standard,
            deterministic_core: Some(CoreSpec::default()),
            probabilistic_wrapper: Some(WrapperSpec::default()),
        }
    }

    #[test]
    fn test_get_calculator_template_arm() {
        let registry = TemplateRegistry::new();
        let gen = registry
            .get(&AgentTemplate::DeterministicCalculator)
            .expect("calculator arm");
        assert_eq!(gen.name(), "calculator");
    }

    #[test]
    fn test_get_hybrid_template_arm() {
        let registry = TemplateRegistry::new();
        let gen = registry
            .get(&AgentTemplate::HybridAnalyzer)
            .expect("hybrid arm");
        assert_eq!(gen.name(), "hybrid");
    }

    #[test]
    fn test_get_custom_agent_nonexistent_path() {
        let registry = TemplateRegistry::new();
        let missing = std::path::PathBuf::from("/tmp/definitely/does/not/exist/template.toml");
        let err = scaffold_err(registry.get(&AgentTemplate::CustomAgent(missing)));
        assert!(matches!(err, ScaffoldError::TemplateNotFound(_)));
    }

    #[test]
    fn test_get_custom_agent_toml_placeholder_err() {
        let tmp = tempfile::TempDir::new().unwrap();
        let p = tmp.path().join("custom.toml");
        std::fs::write(&p, "name = \"x\"").unwrap();
        let registry = TemplateRegistry::new();
        let err = scaffold_err(registry.get(&AgentTemplate::CustomAgent(p)));
        assert!(matches!(err, ScaffoldError::InvalidTemplate(_)));
        assert!(err.to_string().contains("Custom template support requires"));
    }

    #[test]
    fn test_get_custom_agent_yaml_placeholder_err() {
        let tmp = tempfile::TempDir::new().unwrap();
        let p = tmp.path().join("custom.yaml");
        std::fs::write(&p, "name: x").unwrap();
        let registry = TemplateRegistry::new();
        let err = scaffold_err(registry.get(&AgentTemplate::CustomAgent(p)));
        assert!(matches!(err, ScaffoldError::InvalidTemplate(_)));
    }

    #[test]
    fn test_get_custom_agent_wrong_extension_err() {
        let tmp = tempfile::TempDir::new().unwrap();
        let p = tmp.path().join("not-supported.md");
        std::fs::write(&p, "# oops").unwrap();
        let registry = TemplateRegistry::new();
        let err = scaffold_err(registry.get(&AgentTemplate::CustomAgent(p)));
        assert!(matches!(err, ScaffoldError::InvalidTemplate(_)));
        assert!(err.to_string().contains("must be a TOML or YAML file"));
    }

    #[test]
    fn test_get_custom_agent_directory_is_invalid() {
        // A directory path exists but is not a .toml/.yaml file, so the code
        // takes the else-branch of `path.is_file() && ext == toml|yaml`.
        let tmp = tempfile::TempDir::new().unwrap();
        let registry = TemplateRegistry::new();
        let err = scaffold_err(registry.get(&AgentTemplate::CustomAgent(tmp.path().to_path_buf())));
        assert!(matches!(err, ScaffoldError::InvalidTemplate(_)));
    }

    #[test]
    fn test_validate_template_file_missing() {
        let registry = TemplateRegistry::new();
        let missing = std::path::PathBuf::from("/tmp/definitely/nope.toml");
        let err = anyhow_err(registry.validate_template_file(&missing));
        assert!(err.to_string().contains("nope.toml"));
    }

    #[test]
    fn test_validate_template_file_existing_ok() {
        let tmp = tempfile::TempDir::new().unwrap();
        let p = tmp.path().join("exists.toml");
        std::fs::write(&p, "").unwrap();
        let registry = TemplateRegistry::new();
        registry.validate_template_file(&p).expect("file exists");
    }

    #[tokio::test]
    async fn test_fetch_remote_unregistered_name() {
        let registry = TemplateRegistry::new();
        let err = scaffold_err(registry.fetch_remote("does-not-exist").await);
        assert!(matches!(err, ScaffoldError::TemplateNotFound(_)));
    }

    #[tokio::test]
    async fn test_fetch_remote_registered_returns_network_error() {
        let mut registry = TemplateRegistry::new();
        let url = Url::parse("https://example.com/t").unwrap();
        registry.register_remote("remote-x", url);
        let err = scaffold_err(registry.fetch_remote("remote-x").await);
        assert!(
            matches!(err, ScaffoldError::NetworkError(_)),
            "expected NetworkError, got {err:?}"
        );
        assert!(err.to_string().contains("requires network access"));
    }

    #[test]
    fn test_get_template_info_none_for_unknown() {
        let registry = TemplateRegistry::new();
        assert!(registry.get_template_info("not-a-builtin").is_none());
    }

    #[test]
    fn test_get_template_info_none_for_custom() {
        // Custom-registered templates don't have TemplateInfo yet — only builtins do.
        let mut registry = TemplateRegistry::new();
        registry.register_custom("my-c", "/some/path");
        assert!(registry.get_template_info("my-c").is_none());
    }

    #[test]
    fn test_default_equals_new() {
        let a = TemplateRegistry::default();
        let b = TemplateRegistry::new();
        // Both must expose the same 4 built-in names.
        let mut al = a.list_available();
        let mut bl = b.list_available();
        al.sort();
        bl.sort();
        assert_eq!(al, bl);
        assert_eq!(al.len(), 4);
    }

    #[test]
    fn test_list_available_is_sorted_and_includes_custom_remote() {
        let mut registry = TemplateRegistry::new();
        registry.register_custom("zz-custom", "/p");
        registry.register_remote("aa-remote", Url::parse("https://x.example").unwrap());
        let list = registry.list_available();
        let mut sorted = list.clone();
        sorted.sort();
        assert_eq!(list, sorted, "list_available must be sorted");
        assert!(list.contains(&"zz-custom".to_string()));
        assert!(list.contains(&"aa-remote".to_string()));
        assert_eq!(list.len(), 6);
    }

    #[test]
    fn test_template_source_variants_construct() {
        let c = TemplateSource::Custom(PathBuf::from("/p"));
        let r = TemplateSource::Remote(Url::parse("https://x.example").unwrap());
        assert!(matches!(c, TemplateSource::Custom(_)));
        assert!(matches!(r, TemplateSource::Remote(_)));
    }

    // --- Embedded DeterministicCalculatorTemplate ---

    #[test]
    fn test_calculator_template_default_fields() {
        let t = DeterministicCalculatorTemplate::default();
        assert_eq!(t.name(), "calculator");
        assert!(t.description().contains("Deterministic calculator"));
    }

    #[test]
    fn test_calculator_template_generate_produces_two_files() {
        let t = DeterministicCalculatorTemplate::default();
        let files = t.generate(&ctx_named("calc_agent")).expect("generate");
        assert_eq!(files.file_count(), 2);
        assert!(text_file(&files, "Cargo.toml").contains("name = \"calc_agent\""));
        assert!(text_file(&files, "src/main.rs").contains("calc_agent"));
    }

    #[test]
    fn test_calculator_template_validate_context_empty_name_err() {
        let t = DeterministicCalculatorTemplate::default();
        let err = anyhow_err(t.validate_context(&ctx_named("")));
        assert!(err.to_string().contains("name is required"));
    }

    #[test]
    fn test_calculator_template_validate_context_nonempty_ok() {
        let t = DeterministicCalculatorTemplate::default();
        t.validate_context(&ctx_named("ok")).expect("ok");
    }

    // --- Embedded HybridAnalyzerTemplate ---

    #[test]
    fn test_hybrid_template_default_fields() {
        let t = HybridAnalyzerTemplate::default();
        assert_eq!(t.name(), "hybrid");
        assert!(t.description().contains("Hybrid agent"));
    }

    #[test]
    fn test_hybrid_template_generate_requires_core_and_wrapper() {
        let t = HybridAnalyzerTemplate::default();
        let err = anyhow_err(t.generate(&ctx_named("bad")));
        assert!(err.to_string().contains("deterministic core"));
    }

    #[test]
    fn test_hybrid_template_generate_produces_four_files() {
        let t = HybridAnalyzerTemplate::default();
        let files = t.generate(&ctx_hybrid("hyb_agent")).expect("generate");
        assert_eq!(files.file_count(), 4);
        for path in ["Cargo.toml", "src/main.rs", "src/core.rs", "src/wrapper.rs"] {
            assert!(files.contains_file(Path::new(path)), "missing {path}");
        }
        assert!(text_file(&files, "Cargo.toml").contains("name = \"hyb_agent\""));
    }

    #[test]
    fn test_hybrid_template_validate_context_empty_name_err() {
        let t = HybridAnalyzerTemplate::default();
        let mut ctx = ctx_hybrid("x");
        ctx.name = String::new();
        let err = anyhow_err(t.validate_context(&ctx));
        assert!(err.to_string().contains("name is required"));
    }

    #[test]
    fn test_hybrid_template_validate_context_missing_specs_err() {
        let t = HybridAnalyzerTemplate::default();
        // Non-empty name but missing specs → second branch.
        let err = anyhow_err(t.validate_context(&ctx_named("nm")));
        assert!(err.to_string().contains("core and wrapper"));
    }

    #[test]
    fn test_hybrid_template_validate_context_ok() {
        let t = HybridAnalyzerTemplate::default();
        t.validate_context(&ctx_hybrid("ok")).expect("ok");
    }
}
