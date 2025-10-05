//! Scaffolding system for generating projects and agents.
//!
//! # TICKET-PMAT-5001: Core ScaffoldEngine
//! Extended to include core scaffolding engine for project creation.

pub mod agent;
pub mod config;
pub mod errors;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod property_tests;

// Re-export existing agent scaffolding
pub use agent::{
    scaffold_agent, AgentContext, AgentContextBuilder, AgentFeature, AgentTemplate,
    InteractiveScaffolder, QualityLevel, TemplateRegistry,
};

// TICKET-PMAT-5001: Core ScaffoldEngine exports
pub use config::{ScaffoldConfig, Template, AgentFramework, WasmFramework, Feature, QualityGateConfig};
pub use errors::{ScaffoldError, Result};

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
            .args(&["init"])
            .current_dir(project_dir)
            .output()
            .map_err(|e| ScaffoldError::GitError(format!("Failed to run git: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ScaffoldError::GitError(format!("git init failed: {}", stderr)));
        }

        Ok(())
    }

    /// Full scaffolding workflow (to be expanded in later tickets)
    ///
    /// # Complexity
    /// - Time: O(n) where n is number of files in template
    /// - Cyclomatic: 5 (validation, creation, git, error handling)
    pub fn scaffold(&self, config: ScaffoldConfig) -> Result<PathBuf> {
        self.validate_config(&config)?;
        let project_dir = self.create_directory(&config.project_name)?;
        self.init_git(&project_dir)?;
        Ok(project_dir)
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
