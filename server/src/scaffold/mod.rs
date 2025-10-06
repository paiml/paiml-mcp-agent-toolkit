//! Scaffolding system for generating projects and agents.
//!
//! # TICKET-PMAT-5001: Core ScaffoldEngine
//! Extended to include core scaffolding engine for project creation.

pub mod agent;
pub mod ci; // TICKET-PMAT-5022: GitHub Actions workflow generation
pub mod config;
pub mod errors;
pub mod hooks; // TICKET-PMAT-5005
pub mod template; // TICKET-PMAT-5002

#[cfg(test)]
mod tests;

#[cfg(test)]
mod property_tests;

// Re-export existing agent scaffolding
pub use agent::{
    scaffold_agent, AgentContext, AgentContextBuilder, AgentFeature, AgentTemplate,
    InteractiveScaffolder, QualityLevel,
    TemplateRegistry as AgentTemplateRegistry,
};

// TICKET-PMAT-5001: Core ScaffoldEngine exports
pub use config::{ScaffoldConfig, TemplateType, AgentFramework, WasmFramework, Feature, QualityGateConfig};
pub use errors::{ScaffoldError, Result};

// TICKET-PMAT-5002: Template system exports
pub use template::{Template, TemplateRegistry};

use std::path::{Path, PathBuf};
use std::process::Command;
use std::fs;

/// Core scaffolding engine for creating new projects from templates
pub struct ScaffoldEngine {
    template_dir: PathBuf,
}

impl ScaffoldEngine {
    /// Create a new scaffolding engine
    pub fn new() -> Result<Self> {
        Ok(Self {
            template_dir: PathBuf::from("templates"),
        })
    }

    /// Validate scaffolding configuration
    ///
    /// # Complexity
    /// - Time: O(1) - constant-time validation
    /// - Cyclomatic: 3 (input validation branches)
    pub fn validate_config(&self, config: &ScaffoldConfig) -> Result<()> {
        validate_project_name(&config.project_name)?;
        Ok(())
    }

    /// Create project directory structure
    ///
    /// # Complexity
    /// - Time: O(1) - single directory creation
    /// - Cyclomatic: 2 (success/error)
    pub fn create_directory(&self, name: &str) -> Result<PathBuf> {
        let path = PathBuf::from(name);

        if path.exists() {
            return Err(ScaffoldError::DirectoryExists(path));
        }

        fs::create_dir_all(&path)
            .map_err(ScaffoldError::IoError)?;

        Ok(path)
    }

    /// Initialize git repository in project directory
    ///
    /// # Complexity
    /// - Time: O(1) - single git command
    /// - Cyclomatic: 2 (success/error)
    pub fn init_git(&self, project_dir: &Path) -> Result<()> {
        let output = Command::new("git")
            .args(["init"])
            .current_dir(project_dir)
            .output()
            .map_err(|e| ScaffoldError::GitError(format!("Failed to run git: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ScaffoldError::GitError(format!("git init failed: {}", stderr)));
        }

        Ok(())
    }

    /// Full scaffolding workflow (TICKET-PMAT-5005)
    ///
    /// # Complexity
    /// - Time: O(n*m) where n=templates, m=avg template size
    /// - Cyclomatic: 6 (validation, creation, git, structure, files, hooks)
    pub fn scaffold(&self, config: ScaffoldConfig) -> Result<PathBuf> {
        self.validate_config(&config)?;
        let project_dir = self.create_directory(&config.project_name)?;
        self.init_git(&project_dir)?;

        // TICKET-PMAT-5004: Generate project structure and files
        let registry = self.get_template_registry(&config.template_type);
        self.create_project_structure(&project_dir, &config.template_type)?;
        self.generate_files(&project_dir, &registry, &config)?;

        // TICKET-PMAT-5005: Install pre-commit hooks
        self.install_hooks(&project_dir, &config)?;

        Ok(project_dir)
    }

    /// Get template registry based on project type
    ///
    /// # Complexity
    /// - Time: O(1)
    /// - Cyclomatic: 3
    fn get_template_registry(&self, template_type: &TemplateType) -> TemplateRegistry {
        match template_type {
            TemplateType::Agent { .. } => TemplateRegistry::with_pforge_templates(),
            TemplateType::Wasm { .. } => TemplateRegistry::with_wasm_templates(),
            _ => TemplateRegistry::new(),
        }
    }

    /// Create project directory structure
    ///
    /// # Complexity
    /// - Time: O(1) - fixed number of directories
    /// - Cyclomatic: 3
    fn create_project_structure(
        &self,
        project_dir: &Path,
        template_type: &TemplateType,
    ) -> Result<()> {
        match template_type {
            TemplateType::Agent { .. } => {
                fs::create_dir_all(project_dir.join("src/handlers"))?;
                fs::create_dir_all(project_dir.join("tests"))?;
                fs::create_dir_all(project_dir.join("docs"))?;
            }
            TemplateType::Wasm { .. } => {
                fs::create_dir_all(project_dir.join("src"))?;
                fs::create_dir_all(project_dir.join("tests"))?;
                fs::create_dir_all(project_dir.join("benches"))?;
            }
            _ => {}
        }
        Ok(())
    }

    /// Generate all files from templates
    ///
    /// # Complexity
    /// - Time: O(n*m) where n=templates, m=avg template size
    /// - Cyclomatic: 4
    fn generate_files(
        &self,
        project_dir: &Path,
        registry: &TemplateRegistry,
        config: &ScaffoldConfig,
    ) -> Result<()> {
        // Prepare variables for template rendering
        let vars = self.prepare_template_vars(config);

        // Render and write each template
        for template_name in registry.list() {
            let template = registry.get(&template_name)?;
            let rendered = template.render(&vars)?;

            let file_path = self.get_file_path(
                project_dir,
                &template_name,
                &config.template_type,
            );

            self.write_file(&file_path, &rendered)?;
        }

        Ok(())
    }

    /// Prepare template variables from config
    ///
    /// # Complexity
    /// - Time: O(1)
    /// - Cyclomatic: 1
    fn prepare_template_vars(&self, config: &ScaffoldConfig) -> std::collections::HashMap<String, String> {
        use std::collections::HashMap;

        let mut vars = HashMap::new();
        vars.insert("project_name".into(), config.project_name.clone());
        vars.insert("author".into(), "Developer".into());
        vars.insert("description".into(), format!("{} project", config.project_name));

        // Add handler-specific vars
        vars.insert("handler_name".into(), "Example".into());
        vars.insert("handler_description".into(), "Example handler".into());

        vars
    }

    /// Get file path for template
    ///
    /// # Complexity
    /// - Time: O(1)
    /// - Cyclomatic: 6
    fn get_file_path(
        &self,
        project_dir: &Path,
        template_name: &str,
        _template_type: &TemplateType,
    ) -> PathBuf {
        match template_name {
            "pforge.yaml" => project_dir.join("pforge.yaml"),
            "Cargo.toml" => project_dir.join("Cargo.toml"),
            "Makefile" => project_dir.join("Makefile"),
            "README.md" => project_dir.join("README.md"),
            "handler.rs" => project_dir.join("src/handlers/example.rs"),
            "lib.rs" => project_dir.join("src/lib.rs"),
            "vfs.rs" => project_dir.join("src/vfs.rs"),
            _ => project_dir.join(template_name),
        }
    }

    /// Write file to disk
    ///
    /// # Complexity
    /// - Time: O(n) where n is content length
    /// - Cyclomatic: 2
    fn write_file(&self, path: &Path, content: &str) -> Result<()> {
        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(path, content)
            .map_err(ScaffoldError::IoError)?;

        Ok(())
    }

    /// Install pre-commit hooks (TICKET-PMAT-5005)
    ///
    /// # Complexity
    /// - Time: O(1)
    /// - Cyclomatic: 2
    fn install_hooks(&self, project_dir: &Path, config: &ScaffoldConfig) -> Result<()> {
        use crate::scaffold::hooks::{HookConfig, generate_pre_commit_hook, install_pre_commit_hook};

        let hook_config = HookConfig {
            project_type: config.template_type.clone(),
            quality_gates: config.quality_gates.clone(),
        };

        let script = generate_pre_commit_hook(&hook_config)?;
        install_pre_commit_hook(project_dir, &script)?;

        Ok(())
    }
}

impl Default for ScaffoldEngine {
    fn default() -> Self {
        Self::new().expect("ScaffoldEngine::new should not fail")
    }
}

/// Validate project name according to filesystem constraints
///
/// # Complexity
/// - Time: O(n) where n is length of name
/// - Cyclomatic: 1 (delegates to is_valid_name)
fn validate_project_name(name: &str) -> Result<()> {
    if is_valid_name(name) {
        Ok(())
    } else {
        Err(ScaffoldError::InvalidProjectName(name.into()))
    }
}

/// Check if project name is valid
///
/// # Rules
/// - Not empty
/// - Length < 256 characters
/// - No path separators or null bytes
///
/// # Complexity
/// - Time: O(n) where n is length of name
/// - Cyclomatic: 4 (empty, length, contains checks)
fn is_valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() < 256
        && !name.contains(['/', '\\', '\0'])
}
