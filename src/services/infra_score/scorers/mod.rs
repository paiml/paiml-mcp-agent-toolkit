#![cfg_attr(coverage_nightly, coverage(off))]
//! Infra-score scorers — 5 base dimensions (100 pts) + 1 bonus (10 pts).
//!
//! Each scorer implements `InfraScorer` and evaluates one dimension
//! of CI/CD infrastructure quality.

pub mod build_reliability;
pub mod deployment_release;
pub mod provable_contracts;
pub mod quality_pipeline;
pub mod supply_chain;
pub mod workflow_architecture;

pub use build_reliability::BuildReliabilityScorer;
pub use deployment_release::DeploymentReleaseScorer;
pub use provable_contracts::ProvableContractsScorer;
pub use quality_pipeline::QualityPipelineScorer;
pub use supply_chain::SupplyChainScorer;
pub use workflow_architecture::WorkflowArchitectureScorer;

use crate::services::infra_score::models::InfraCategoryScore;
use async_trait::async_trait;
use std::path::Path;

/// Trait for all infra scoring modules
#[async_trait]
pub trait InfraScorer: Send + Sync {
    /// Name of the category (e.g., "Workflow Architecture")
    fn category_name(&self) -> &str;

    /// Maximum points available for this category
    fn max_score(&self) -> f64;

    /// Execute scoring for this category
    async fn score(&self, repo_path: &Path) -> anyhow::Result<InfraCategoryScore>;
}

/// Read all workflow YAML files from .github/workflows/
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn read_workflow_files(repo_path: &Path) -> Vec<(String, String)> {
    let workflows_dir = repo_path.join(".github/workflows");
    let mut files = Vec::new();

    if !workflows_dir.is_dir() {
        return files;
    }

    if let Ok(entries) = std::fs::read_dir(&workflows_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if (ext == "yml" || ext == "yaml") && path.is_file() {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        let name = path
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default();
                        files.push((name, content));
                    }
                }
            }
        }
    }

    files
}

/// Read Cargo.toml content if it exists
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn read_cargo_toml(repo_path: &Path) -> Option<String> {
    let path = repo_path.join("Cargo.toml");
    std::fs::read_to_string(path).ok()
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_read_workflow_files_empty_dir() {
        let tmp = TempDir::new().unwrap();
        let files = read_workflow_files(tmp.path());
        assert!(files.is_empty());
    }

    #[test]
    fn test_read_workflow_files_with_yml() {
        let tmp = TempDir::new().unwrap();
        let wf_dir = tmp.path().join(".github/workflows");
        fs::create_dir_all(&wf_dir).unwrap();
        fs::write(wf_dir.join("ci.yml"), "name: CI\non: push").unwrap();
        fs::write(wf_dir.join("nightly.yaml"), "name: Nightly").unwrap();
        fs::write(wf_dir.join("README.md"), "not a workflow").unwrap();

        let files = read_workflow_files(tmp.path());
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_read_cargo_toml_exists() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();
        let content = read_cargo_toml(tmp.path());
        assert!(content.is_some());
        assert!(content.unwrap().contains("[package]"));
    }

    #[test]
    fn test_read_cargo_toml_missing() {
        let tmp = TempDir::new().unwrap();
        let content = read_cargo_toml(tmp.path());
        assert!(content.is_none());
    }
}
