//! Project metadata models for PMAT compliance system (GH-96)
//!
//! Tracks project's PMAT version, schema versions, and compliance state.
//! Stored in `.pmat/project.toml`

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Project metadata stored in .pmat/project.toml
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectMetadata {
    /// PMAT metadata
    #[serde(rename = "pmat")]
    pub pmat: PmatMetadata,

    /// Compliance tracking
    #[serde(rename = "compliance", default)]
    pub compliance: ComplianceMetadata,
}

/// PMAT version and schema information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PmatMetadata {
    /// PMAT binary version used to create/update this project
    pub version: String,

    /// Last compliance check timestamp (RFC3339)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_compliance_check: Option<String>,

    /// Project metadata schema version
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
}

/// Compliance state tracking
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ComplianceMetadata {
    /// Breaking changes that have been accepted/migrated
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub breaking_changes_accepted: Vec<String>,

    /// Last migration timestamp (RFC3339)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_migration: Option<String>,

    /// Migration history (version -> timestamp)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub migration_history: Vec<MigrationRecord>,
}

/// Migration history record
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MigrationRecord {
    /// Version migrated from
    pub from_version: String,

    /// Version migrated to
    pub to_version: String,

    /// Migration timestamp (RFC3339)
    pub timestamp: String,

    /// Whether migration was successful
    pub success: bool,
}

fn default_schema_version() -> String {
    "1.0".to_string()
}

impl ProjectMetadata {
    /// Create new project metadata with current PMAT version
    pub fn new(pmat_version: impl Into<String>) -> Self {
        Self {
            pmat: PmatMetadata {
                version: pmat_version.into(),
                last_compliance_check: Some(chrono::Utc::now().to_rfc3339()),
                schema_version: default_schema_version(),
            },
            compliance: ComplianceMetadata::default(),
        }
    }

    /// Load project metadata from .pmat/project.toml
    pub fn load(project_path: &Path) -> Result<Self> {
        let path = Self::get_path(project_path);
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;

        toml::from_str(&content).with_context(|| format!("Failed to parse {}", path.display()))
    }

    /// Save project metadata to .pmat/project.toml
    pub fn save(&self, project_path: &Path) -> Result<()> {
        let path = Self::get_path(project_path);

        // Ensure .pmat directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {}", parent.display()))?;
        }

        let content =
            toml::to_string_pretty(self).context("Failed to serialize project metadata")?;

        fs::write(&path, content).with_context(|| format!("Failed to write {}", path.display()))
    }

    /// Check if project metadata exists
    pub fn exists(project_path: &Path) -> bool {
        Self::get_path(project_path).exists()
    }

    /// Get path to project.toml
    pub fn get_path(project_path: &Path) -> PathBuf {
        project_path.join(".pmat").join("project.toml")
    }

    /// Update last compliance check timestamp
    pub fn update_compliance_check(&mut self) {
        self.pmat.last_compliance_check = Some(chrono::Utc::now().to_rfc3339());
    }

    /// Record a migration
    pub fn record_migration(&mut self, from: String, to: String, success: bool) {
        self.compliance.migration_history.push(MigrationRecord {
            from_version: from,
            to_version: to.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            success,
        });

        if success {
            self.pmat.version = to;
            self.compliance.last_migration = Some(chrono::Utc::now().to_rfc3339());
        }
    }

    /// Accept a breaking change
    pub fn accept_breaking_change(&mut self, version: String) {
        if !self.compliance.breaking_changes_accepted.contains(&version) {
            self.compliance.breaking_changes_accepted.push(version);
        }
    }

    /// Check if a breaking change has been accepted
    pub fn is_breaking_change_accepted(&self, version: &str) -> bool {
        self.compliance
            .breaking_changes_accepted
            .contains(&version.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_new_project_metadata() {
        let metadata = ProjectMetadata::new("2.205.0");
        assert_eq!(metadata.pmat.version, "2.205.0");
        assert_eq!(metadata.pmat.schema_version, "1.0");
        assert!(metadata.pmat.last_compliance_check.is_some());
        assert!(metadata.compliance.breaking_changes_accepted.is_empty());
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let metadata = ProjectMetadata::new("2.205.0");

        metadata.save(temp_dir.path()).unwrap();
        assert!(ProjectMetadata::exists(temp_dir.path()));

        let loaded = ProjectMetadata::load(temp_dir.path()).unwrap();
        assert_eq!(loaded.pmat.version, metadata.pmat.version);
        assert_eq!(loaded.pmat.schema_version, metadata.pmat.schema_version);
    }

    #[test]
    fn test_record_migration() {
        let mut metadata = ProjectMetadata::new("2.150.0");
        metadata.record_migration("2.150.0".to_string(), "2.205.0".to_string(), true);

        assert_eq!(metadata.pmat.version, "2.205.0");
        assert_eq!(metadata.compliance.migration_history.len(), 1);
        assert!(metadata.compliance.last_migration.is_some());

        let migration = &metadata.compliance.migration_history[0];
        assert_eq!(migration.from_version, "2.150.0");
        assert_eq!(migration.to_version, "2.205.0");
        assert!(migration.success);
    }

    #[test]
    fn test_accept_breaking_change() {
        let mut metadata = ProjectMetadata::new("2.150.0");
        metadata.accept_breaking_change("2.180.0".to_string());
        metadata.accept_breaking_change("2.195.0".to_string());

        assert!(metadata.is_breaking_change_accepted("2.180.0"));
        assert!(metadata.is_breaking_change_accepted("2.195.0"));
        assert!(!metadata.is_breaking_change_accepted("2.200.0"));
        assert_eq!(metadata.compliance.breaking_changes_accepted.len(), 2);
    }

    #[test]
    fn test_update_compliance_check() {
        let mut metadata = ProjectMetadata::new("2.205.0");
        let first_check = metadata.pmat.last_compliance_check.clone();

        // Wait a tiny bit to ensure timestamp changes
        std::thread::sleep(std::time::Duration::from_millis(10));

        metadata.update_compliance_check();
        let second_check = metadata.pmat.last_compliance_check.clone();

        assert_ne!(first_check, second_check);
    }

    #[test]
    fn test_get_path() {
        let path = Path::new("/tmp/test-project");
        let expected = path.join(".pmat").join("project.toml");
        assert_eq!(ProjectMetadata::get_path(path), expected);
    }

    #[test]
    fn test_serialization() {
        let metadata = ProjectMetadata::new("2.205.0");
        let toml = toml::to_string_pretty(&metadata).unwrap();

        assert!(toml.contains("[pmat]"));
        assert!(toml.contains("version = \"2.205.0\""));
        assert!(toml.contains("schema_version = \"1.0\""));
        assert!(toml.contains("[compliance]"));
    }
}
