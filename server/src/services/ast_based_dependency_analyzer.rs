use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use syn::visit::{self, Visit};

use crate::models::unified_ast::Language;

/// AST-based external dependency detection
/// Replaces fragile regex-based approach with proper AST analysis
pub struct AstBasedDependencyAnalyzer {
    builtin_modules: Arc<BuiltinModuleRegistry>,
    workspace_resolver: Arc<WorkspaceResolver>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyAnalysis {
    pub external: Vec<Dependency>,
    pub internal: Vec<Dependency>,
    pub boundary_violations: Vec<BoundaryViolation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version: Option<String>,
    pub is_external: bool,
    pub import_type: ImportType,
    pub location: Location,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImportType {
    Use,           // Rust use
    Import,        // Python/JS import
    FromImport,    // Python from x import y
    Require,       // Node.js require
    DynamicImport, // JS dynamic import()
    TypeOnly,      // TypeScript type-only import
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub file: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryViolation {
    pub from_module: String,
    pub to_module: String,
    pub violation_type: ViolationType,
    pub location: Location,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationType {
    LayerViolation,      // e.g., UI importing from DB layer
    VisibilityViolation, // Private module access
    CyclicDependency,    // Circular imports
}

pub struct BuiltinModuleRegistry {
    rust_builtins: HashSet<String>,
    python_builtins: HashSet<String>,
    node_builtins: HashSet<String>,
}

pub struct WorkspaceResolver {
    workspace_members: HashSet<String>,
    #[allow(dead_code)]
    module_boundaries: HashMap<String, ModuleBoundary>,
}

#[derive(Debug, Clone)]
pub struct ModuleBoundary {
    pub allowed_imports: HashSet<String>,
    pub layer: ArchitectureLayer,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArchitectureLayer {
    Presentation,
    Application,
    Domain,
    Infrastructure,
}

struct RustDependencyVisitor {
    dependencies: Vec<Dependency>,
    internal_deps: Vec<Dependency>,
    #[allow(dead_code)]
    current_scope: Scope,
    workspace_members: HashSet<String>,
    file_path: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum Scope {
    Module,
    Function,
    Impl,
}

impl AstBasedDependencyAnalyzer {
    pub fn new() -> Self {
        Self {
            builtin_modules: Arc::new(BuiltinModuleRegistry::new()),
            workspace_resolver: Arc::new(WorkspaceResolver::new()),
        }
    }

    pub async fn analyze_file(&self, file_path: &Path) -> Result<DependencyAnalysis> {
        let content = tokio::fs::read_to_string(file_path).await?;

        // Determine language from file extension
        let language = match file_path.extension().and_then(|s| s.to_str()) {
            Some("rs") => Language::Rust,
            Some("ts") | Some("tsx") => Language::TypeScript,
            Some("js") | Some("jsx") => Language::JavaScript,
            Some("py") => Language::Python,
            Some("c") | Some("h") => Language::C,
            Some("cpp") | Some("cc") | Some("cxx") | Some("hpp") | Some("hxx") => Language::Cpp,
            _ => {
                return Ok(DependencyAnalysis {
                    external: vec![],
                    internal: vec![],
                    boundary_violations: vec![],
                })
            }
        };

        match language {
            Language::Rust => self.analyze_rust_content(&content, file_path),
            Language::TypeScript | Language::JavaScript => {
                self.analyze_typescript_content(&content, file_path).await
            }
            Language::Python => self.analyze_python_content(&content, file_path).await,
            Language::C | Language::Cpp | Language::Kotlin => {
                // C/C++/Kotlin dependency analysis would go here
                // For now, return empty analysis
                Ok(DependencyAnalysis {
                    external: Vec::new(),
                    internal: Vec::new(),
                    boundary_violations: Vec::new(),
                })
            }
            Language::Cython => {
                // Cython dependency analysis would go here
                // For now, return empty analysis
                Ok(DependencyAnalysis {
                    external: Vec::new(),
                    internal: Vec::new(),
                    boundary_violations: Vec::new(),
                })
            }
            Language::Markdown
            | Language::Makefile
            | Language::Toml
            | Language::Yaml
            | Language::Json
            | Language::Shell
            | Language::AssemblyScript
            | Language::WebAssembly => {
                // No dependency analysis for these file types yet
                Ok(DependencyAnalysis {
                    external: Vec::new(),
                    internal: Vec::new(),
                    boundary_violations: Vec::new(),
                })
            }
        }
    }

    fn analyze_rust_content(&self, content: &str, file_path: &Path) -> Result<DependencyAnalysis> {
        let ast = syn::parse_file(content)?;

        let mut visitor = RustDependencyVisitor {
            dependencies: Vec::new(),
            internal_deps: Vec::new(),
            current_scope: Scope::Module,
            workspace_members: self.workspace_resolver.get_members(),
            file_path: file_path.to_string_lossy().to_string(),
        };

        visitor.visit_file(&ast);

        let boundary_violations = self.check_visibility_violations(&visitor);

        Ok(DependencyAnalysis {
            external: visitor
                .dependencies
                .into_iter()
                .filter(|dep| dep.is_external)
                .collect(),
            internal: visitor.internal_deps,
            boundary_violations,
        })
    }

    async fn analyze_typescript_content(
        &self,
        content: &str,
        file_path: &Path,
    ) -> Result<DependencyAnalysis> {
        // Simplified TypeScript analysis using regex for now
        // In production, would use swc or tree-sitter
        let mut dependencies = Vec::new();
        let mut line_number = 1;

        // Pre-compile regexes
        let import_regex = regex::Regex::new(r#"import\s+.*\s+from\s+['"]([^'"]+)['"]"#)?;
        let dynamic_import_regex = regex::Regex::new(r#"import\(['"]([^'"]+)['"]\)"#)?;
        let type_import_regex =
            regex::Regex::new(r#"import\s+type\s+.*\s+from\s+['"]([^'"]+)['"]"#)?;

        for line in content.lines() {
            // Static imports
            if let Some(caps) = import_regex.captures(line) {
                let module = caps[1].to_string();
                dependencies.push(Dependency {
                    name: module.clone(),
                    version: None,
                    is_external: self.is_external_module(&module),
                    import_type: ImportType::Import,
                    location: Location {
                        file: file_path.to_string_lossy().to_string(),
                        line: line_number,
                        column: 0,
                    },
                });
            }

            // Dynamic imports
            if line.contains("import(") {
                if let Some(caps) = dynamic_import_regex.captures(line) {
                    let module = caps[1].to_string();
                    dependencies.push(Dependency {
                        name: module.clone(),
                        version: None,
                        is_external: self.is_external_module(&module),
                        import_type: ImportType::DynamicImport,
                        location: Location {
                            file: file_path.to_string_lossy().to_string(),
                            line: line_number,
                            column: 0,
                        },
                    });
                }
            }

            // Type-only imports
            if let Some(caps) = type_import_regex.captures(line) {
                let module = caps[1].to_string();
                dependencies.push(Dependency {
                    name: module.clone(),
                    version: None,
                    is_external: self.is_external_module(&module),
                    import_type: ImportType::TypeOnly,
                    location: Location {
                        file: file_path.to_string_lossy().to_string(),
                        line: line_number,
                        column: 0,
                    },
                });
            }

            line_number += 1;
        }

        Ok(DependencyAnalysis {
            external: dependencies
                .into_iter()
                .filter(|dep| dep.is_external)
                .collect(),
            internal: vec![],
            boundary_violations: vec![],
        })
    }

    async fn analyze_python_content(
        &self,
        content: &str,
        file_path: &Path,
    ) -> Result<DependencyAnalysis> {
        // Simplified Python analysis using regex
        // In production, would use rustpython-parser
        let mut dependencies = Vec::new();
        let mut line_number = 1;

        // Pre-compile regexes
        let import_regex = regex::Regex::new(r"^import\s+(\S+)")?;
        let from_import_regex = regex::Regex::new(r"^from\s+(\S+)\s+import")?;

        for line in content.lines() {
            // Standard imports
            if let Some(caps) = import_regex.captures(line) {
                let module = caps[1].to_string();
                dependencies.push(Dependency {
                    name: module.clone(),
                    version: None,
                    is_external: self.is_external_module(&module),
                    import_type: ImportType::Import,
                    location: Location {
                        file: file_path.to_string_lossy().to_string(),
                        line: line_number,
                        column: 0,
                    },
                });
            }

            // From imports
            if let Some(caps) = from_import_regex.captures(line) {
                let module = caps[1].to_string();
                dependencies.push(Dependency {
                    name: module.clone(),
                    version: None,
                    is_external: self.is_external_module(&module),
                    import_type: ImportType::FromImport,
                    location: Location {
                        file: file_path.to_string_lossy().to_string(),
                        line: line_number,
                        column: 0,
                    },
                });
            }

            line_number += 1;
        }

        Ok(DependencyAnalysis {
            external: dependencies
                .into_iter()
                .filter(|dep| dep.is_external)
                .collect(),
            internal: vec![],
            boundary_violations: vec![],
        })
    }

    fn is_external_module(&self, module: &str) -> bool {
        // Check if it's a relative import
        if module.starts_with('.') || module.starts_with("./") || module.starts_with("../") {
            return false;
        }

        // Check if it's in workspace
        if self.workspace_resolver.is_workspace_member(module) {
            return false;
        }

        // Check if it's a builtin
        if self.builtin_modules.is_builtin(module) {
            return false;
        }

        true
    }

    fn check_visibility_violations(
        &self,
        visitor: &RustDependencyVisitor,
    ) -> Vec<BoundaryViolation> {
        let mut violations = Vec::new();

        // Check for layer violations
        if let Some(from_boundary) = self
            .workspace_resolver
            .get_module_boundary(&visitor.file_path)
        {
            for dep in &visitor.dependencies {
                if let Some(to_boundary) = self.workspace_resolver.get_module_boundary(&dep.name) {
                    if !self.is_allowed_dependency(&from_boundary, &to_boundary) {
                        violations.push(BoundaryViolation {
                            from_module: visitor.file_path.clone(),
                            to_module: dep.name.clone(),
                            violation_type: ViolationType::LayerViolation,
                            location: dep.location.clone(),
                        });
                    }
                }
            }
        }

        violations
    }

    fn is_allowed_dependency(&self, from: &ModuleBoundary, to: &ModuleBoundary) -> bool {
        use ArchitectureLayer::*;

        match (&from.layer, &to.layer) {
            // Presentation can depend on Application and Domain
            (Presentation, Application) | (Presentation, Domain) => true,
            // Application can depend on Domain and Infrastructure
            (Application, Domain) | (Application, Infrastructure) => true,
            // Domain should not depend on outer layers
            (Domain, Domain) => true,
            // Infrastructure can depend on Domain
            (Infrastructure, Domain) => true,
            // Everything else is a violation
            _ => false,
        }
    }
}

impl<'ast> Visit<'ast> for RustDependencyVisitor {
    fn visit_use_tree(&mut self, use_tree: &'ast syn::UseTree) {
        if let Some(dep) = self.extract_dependency(use_tree) {
            let is_external = !self.workspace_members.contains(&dep.crate_name);

            let dependency = Dependency {
                name: dep.crate_name.clone(),
                version: dep.version,
                is_external,
                import_type: ImportType::Use,
                location: Location {
                    file: self.file_path.clone(),
                    line: 0, // Would need span info for accurate line
                    column: 0,
                },
            };

            if is_external {
                self.dependencies.push(dependency);
            } else {
                self.internal_deps.push(dependency);
            }
        }

        visit::visit_use_tree(self, use_tree);
    }
}

impl RustDependencyVisitor {
    fn extract_dependency(&self, use_tree: &syn::UseTree) -> Option<ExtractedDependency> {
        match use_tree {
            syn::UseTree::Path(path) => {
                let crate_name = path.ident.to_string();
                Some(ExtractedDependency {
                    crate_name,
                    version: None,
                })
            }
            _ => None,
        }
    }
}

struct ExtractedDependency {
    crate_name: String,
    version: Option<String>,
}

impl BuiltinModuleRegistry {
    fn new() -> Self {
        let mut rust_builtins = HashSet::new();
        rust_builtins.insert("std".to_string());
        rust_builtins.insert("core".to_string());
        rust_builtins.insert("alloc".to_string());

        let mut python_builtins = HashSet::new();
        python_builtins.insert("os".to_string());
        python_builtins.insert("sys".to_string());
        python_builtins.insert("math".to_string());
        python_builtins.insert("json".to_string());

        let mut node_builtins = HashSet::new();
        node_builtins.insert("fs".to_string());
        node_builtins.insert("path".to_string());
        node_builtins.insert("http".to_string());
        node_builtins.insert("crypto".to_string());

        Self {
            rust_builtins,
            python_builtins,
            node_builtins,
        }
    }

    fn is_builtin(&self, module: &str) -> bool {
        self.rust_builtins.contains(module)
            || self.python_builtins.contains(module)
            || self.node_builtins.contains(module)
    }
}

impl WorkspaceResolver {
    fn new() -> Self {
        // In production, would parse Cargo.toml workspace
        let mut workspace_members = HashSet::new();
        workspace_members.insert("server".to_string());

        Self {
            workspace_members,
            module_boundaries: HashMap::new(),
        }
    }

    fn get_members(&self) -> HashSet<String> {
        self.workspace_members.clone()
    }

    fn is_workspace_member(&self, module: &str) -> bool {
        self.workspace_members.contains(module)
    }

    fn get_module_boundary(&self, _path: &str) -> Option<ModuleBoundary> {
        None // Simplified for now
    }
}

impl Default for AstBasedDependencyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_rust_dependency_analysis() {
        let analyzer = AstBasedDependencyAnalyzer::new();
        let temp_dir = TempDir::new().unwrap();

        let test_file = temp_dir.path().join("test.rs");
        tokio::fs::write(
            &test_file,
            r#"
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::models::User;

fn main() {
    let map = HashMap::new();
}
"#,
        )
        .await
        .unwrap();

        let analysis = analyzer.analyze_file(&test_file).await.unwrap();

        assert!(!analysis.external.is_empty());
        assert!(analysis.external.iter().any(|d| d.name == "serde"));
    }

    #[tokio::test]
    async fn test_typescript_dependency_analysis() {
        let analyzer = AstBasedDependencyAnalyzer::new();
        let temp_dir = TempDir::new().unwrap();

        let test_file = temp_dir.path().join("test.ts");
        tokio::fs::write(
            &test_file,
            r#"
import React from 'react';
import type { User } from './types';
import { api } from '../api';

const loadUser = async () => {
    const module = await import('lodash');
    return module.getUser();
};
"#,
        )
        .await
        .unwrap();

        let analysis = analyzer.analyze_file(&test_file).await.unwrap();

        assert!(analysis.external.iter().any(|d| d.name == "react"));
        assert!(analysis
            .external
            .iter()
            .any(|d| matches!(d.import_type, ImportType::DynamicImport)));
    }
}
