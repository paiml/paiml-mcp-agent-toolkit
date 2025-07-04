use crate::models::dag::DependencyGraph;
use crate::services::dag_builder::DagBuilder;
use crate::services::file_discovery::ProjectFileDiscovery;
use anyhow::Result;
use std::path::{Path, PathBuf};

/// Represents a project for analysis
pub struct Project {
    root: PathBuf,
    file_discovery: ProjectFileDiscovery,
}

impl Project {
    /// Create a new project instance
    pub fn new(root: &Path) -> Result<Self> {
        Ok(Self {
            root: root.to_path_buf(),
            file_discovery: ProjectFileDiscovery::new(root.to_path_buf()),
        })
    }

    /// Get the project root path
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Get all source files in the project
    pub fn source_files(&self) -> Vec<PathBuf> {
        self.file_discovery
            .discover_files()
            .unwrap_or_default()
            .into_iter()
            .filter(|f| self.is_source_file(f))
            .collect()
    }

    /// Build a dependency graph for the project
    pub async fn build_dependency_graph(&self) -> Result<DependencyGraph> {
        use crate::services::context::{ProjectContext, ProjectSummary};

        // Create a project context for the DAG builder
        let files = self.source_files();
        let project_context = ProjectContext {
            project_type: "rust".to_string(), // TODO: detect project type
            files: vec![],                    // TODO: convert files to FileContext
            summary: ProjectSummary {
                total_files: files.len(),
                total_functions: 0,
                total_structs: 0,
                total_enums: 0,
                total_traits: 0,
                total_impls: 0,
                dependencies: vec![],
            },
        };

        Ok(DagBuilder::build_from_project(&project_context))
    }

    /// Check if a file is a source file
    fn is_source_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            matches!(
                ext.to_str().unwrap_or(""),
                "rs" | "py"
                    | "js"
                    | "ts"
                    | "tsx"
                    | "jsx"
                    | "java"
                    | "kt"
                    | "cpp"
                    | "c"
                    | "h"
                    | "hpp"
            )
        } else {
            false
        }
    }
}
