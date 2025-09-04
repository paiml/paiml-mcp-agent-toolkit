//! Stub module for backward compatibility during AST migration
//! 
//! This module provides minimal stubs to prevent compilation errors.
//! All functionality has been moved to server/src/ast/

use std::path::Path;
use anyhow::Result;
use std::collections::HashMap;

// Stub types for backward compatibility
pub struct UnifiedAstEngine;

impl Default for UnifiedAstEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl UnifiedAstEngine {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn analyze_project(&self, _path: &Path) -> Result<AstForest> {
        Ok(AstForest::default())
    }
    
    pub async fn parse_project(&self, _path: &Path) -> Result<AstForest> {
        Ok(AstForest::default())
    }
    
    pub fn compute_metrics(&self, _forest: &AstForest) -> Result<ProjectMetrics> {
        Ok(ProjectMetrics::default())
    }
    
    pub fn extract_dependencies(&self, _forest: &AstForest) -> Result<crate::models::dag::DependencyGraph> {
        Ok(crate::models::dag::DependencyGraph::new())
    }
}

#[derive(Default, Debug, Clone)]
pub struct AstForest {
    pub modules: Vec<ModuleNode>,
    pub metrics: ProjectMetrics,
}

impl AstForest {
    pub fn files(&self) -> impl Iterator<Item = &ModuleNode> {
        self.modules.iter()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ModuleNode {
    pub path: String,
    pub name: String,
    pub metrics: ModuleMetrics,
}

#[derive(Default, Debug, Clone)]
pub struct ModuleMetrics {
    pub complexity: u32,
    pub lines: usize,
}

#[derive(Default, Debug, Clone)]
pub struct ProjectMetrics {
    pub total_complexity: u32,
    pub total_lines: usize,
    pub file_count: usize,
    pub function_count: usize,
    pub avg_complexity: f32,
    pub max_complexity: u32,
}


// Additional stub types for deterministic_mermaid_engine
#[derive(Default, Debug, Clone)]
pub struct ArtifactTree {
    pub nodes: Vec<String>,
    pub dogfooding: Vec<(String, String)>, // (name, content) pairs
    pub mermaid: MermaidArtifacts,
    pub templates: Vec<Template>,
}

#[derive(Default, Debug, Clone)]
pub struct MermaidArtifacts {
    pub diagrams: Vec<String>,
    pub ast_generated: HashMap<String, String>,
    pub non_code: HashMap<String, String>,
}

#[derive(Default, Debug, Clone)]
pub struct Template {
    pub name: String,
    pub content: String,
}

// FileAst stub enum for backward compatibility
pub enum FileAst {
    Rust(syn::File),
    TypeScript(String),
    Python(String), 
    C(String),
    Cpp(String),
    Cython(String),
    Kotlin(String),
    Makefile(String),
    Markdown(String),
    Toml(String),
    Yaml(String),
    Json(String),
    Shell(String),
}