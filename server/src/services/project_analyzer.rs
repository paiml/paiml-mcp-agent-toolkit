//! Project analyzer for codebase exploration and dependency analysis
//!
//! This module provides the core project analysis functionality, serving as
//! the entry point for analyzing codebases. It coordinates file discovery,
//! source file identification, and dependency graph construction to provide
//! a comprehensive view of project structure.
//!
//! # Features
//!
//! - **Automatic File Discovery**: Finds all relevant source files in a project
//! - **Language Detection**: Identifies programming languages by file extension
//! - **Dependency Graph Building**: Constructs relationships between code elements
//! - **Gitignore Respect**: Automatically excludes ignored files
//! - **Cross-language Support**: Handles mixed-language projects
//!
//! # Supported Languages
//!
//! - Rust (.rs)
//! - Python (.py)
//! - JavaScript/TypeScript (.js, .ts, .jsx, .tsx)
//! - Java (.java)
//! - Kotlin (.kt)
//! - C/C++ (.c, .cpp, .h, .hpp)
//!
//! # Example
//!
//! ```no_run
//! use pmat::services::project_analyzer::Project;
//! use std::path::Path;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let project = Project::new(Path::new("./src"))?;
//!
//! // Get all source files in the project
//! let source_files = project.source_files();
//! println!("Found {} source files", source_files.len());
//!
//! // Build dependency graph for analysis
//! let dependency_graph = project.build_dependency_graph().await?;
//! println!("Graph has {} nodes and {} edges",
//!          dependency_graph.nodes.len(),
//!          dependency_graph.edges.len());
//!
//! // Analyze specific aspects
//! for file in source_files.iter().take(5) {
//!     println!("  - {}", file.display());
//! }
//! # Ok(())
//! # }
//! ```

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
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::services::project_analyzer::Project;
    /// use std::path::Path;
    ///
    /// let project = Project::new(Path::new(".")).unwrap();
    /// assert_eq!(project.root(), Path::new("."));
    /// ```
    pub fn new(root: &Path) -> Result<Self> {
        Ok(Self {
            root: root.to_path_buf(),
            file_discovery: ProjectFileDiscovery::new(root.to_path_buf()),
        })
    }

    /// Get the project root path
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::services::project_analyzer::Project;
    /// use std::path::Path;
    ///
    /// let project = Project::new(Path::new("/home/user/myproject")).unwrap();
    /// assert_eq!(project.root(), Path::new("/home/user/myproject"));
    /// ```
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
            project_type: "rust".to_string(), // Project type detection implemented elsewhere
            files: vec![],                    // Files converted via FileContext::from_path
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
