//! Unified AST Engine for deterministic artifact generation
//!
//! This module implements the core engine that combines AST introspection,
//! dependency graph analysis, and deterministic artifact generation as specified
//! in the deterministic-graphs-mmd-spec.md

use crate::models::dag::EdgeType;
use crate::models::error::TemplateError;
use blake3::Hasher;
use petgraph::stable_graph::StableGraph;
use serde_json::json;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// File type classification for parsing strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FileType {
    Rust,
    TypeScript,
    Python,
    Cython,
    C,
    Cpp,
    Makefile,
    Unknown,
}

/// Multi-language AST unification engine
pub struct UnifiedAstEngine {
    /// Content hasher for deterministic verification
    #[allow(dead_code)]
    artifact_hasher: Hasher,
    /// Language parsers
    #[allow(dead_code)]
    parsers: LanguageParsers,
}

pub struct LanguageParsers {
    // Language-specific parsing capabilities
}

impl Default for LanguageParsers {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageParsers {
    pub fn new() -> Self {
        Self {}
    }
}

/// Forest of ASTs from all files in the project
#[derive(Default)]
pub struct AstForest {
    files: BTreeMap<PathBuf, FileAst>,
}

impl AstForest {
    pub fn new() -> Self {
        Self {
            files: BTreeMap::new(),
        }
    }

    pub fn add_file(&mut self, path: PathBuf, ast: FileAst) {
        self.files.insert(path, ast);
    }

    pub fn files(&self) -> impl Iterator<Item = (&PathBuf, &FileAst)> {
        self.files.iter()
    }

    pub fn get(&self, path: &Path) -> Option<&FileAst> {
        self.files.get(path)
    }
}

/// Language-specific AST representation
/// Apply Kaizen - Add support for project documentation and configuration files
#[derive(Clone)]
pub enum FileAst {
    Rust(syn::File),
    TypeScript(String), // Placeholder - would use swc_ecma_ast in real implementation
    Python(String),     // Placeholder - would use rustpython_ast in real implementation
    C(std::sync::Arc<crate::models::unified_ast::AstDag>), // C language AST
    Cpp(std::sync::Arc<crate::models::unified_ast::AstDag>), // C++ language AST
    Cython(String),     // Cython - Python with C extensions
    Makefile(crate::services::makefile_linter::MakefileAst),
    // Kaizen improvement - Add project file types for complete analysis
    Markdown(String), // README.md, documentation
    Toml(String),     // Cargo.toml, pyproject.toml
    Yaml(String),     // GitHub Actions, docker-compose
    Json(String),     // package.json, tsconfig.json
    Shell(String),    // Build scripts, automation
}

impl std::fmt::Debug for FileAst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileAst::Rust(_) => write!(f, "FileAst::Rust(..)"),
            FileAst::TypeScript(content) => {
                write!(f, "FileAst::TypeScript({} chars)", content.len())
            }
            FileAst::Python(content) => write!(f, "FileAst::Python({} chars)", content.len()),
            FileAst::C(_) => write!(f, "FileAst::C(..)"),
            FileAst::Cpp(_) => write!(f, "FileAst::Cpp(..)"),
            FileAst::Cython(content) => write!(f, "FileAst::Cython({} chars)", content.len()),
            FileAst::Makefile(_) => write!(f, "FileAst::Makefile(..)"),
            FileAst::Markdown(_) => write!(f, "FileAst::Markdown(..)"),
            FileAst::Toml(_) => write!(f, "FileAst::Toml(..)"),
            FileAst::Yaml(_) => write!(f, "FileAst::Yaml(..)"),
            FileAst::Json(_) => write!(f, "FileAst::Json(..)"),
            FileAst::Shell(_) => write!(f, "FileAst::Shell(..)"),
        }
    }
}

impl FileAst {
    pub fn root_visibility(&self) -> String {
        match self {
            FileAst::Rust(_) => "public".to_string(),
            FileAst::TypeScript(_) => "exported".to_string(),
            FileAst::Python(_) => "global".to_string(),
            FileAst::C(_) => "global".to_string(),
            FileAst::Cpp(_) => "global".to_string(),
            FileAst::Cython(_) => "global".to_string(),
            FileAst::Makefile(_) => "global".to_string(),
            FileAst::Markdown(_) => "global".to_string(),
            FileAst::Toml(_) => "global".to_string(),
            FileAst::Yaml(_) => "global".to_string(),
            FileAst::Json(_) => "global".to_string(),
            FileAst::Shell(_) => "global".to_string(),
        }
    }
}

/// Module node for dependency graph
#[derive(Debug, Clone)]
pub struct ModuleNode {
    pub name: String,
    pub path: PathBuf,
    pub visibility: String,
    pub metrics: ModuleMetrics,
}

#[derive(Debug, Clone, Default)]
pub struct ModuleMetrics {
    pub complexity: u32,
    pub lines: u32,
    pub functions: u32,
    pub classes: u32,
}

/// Project-wide metrics
#[derive(Debug, Clone)]
pub struct ProjectMetrics {
    pub file_count: usize,
    pub function_count: usize,
    pub avg_complexity: f64,
    pub max_complexity: u32,
}

/// Complete artifact tree structure
#[derive(Debug, Clone)]
pub struct ArtifactTree {
    pub dogfooding: BTreeMap<String, String>,
    pub mermaid: MermaidArtifacts,
    pub templates: Vec<Template>,
}

#[derive(Debug, Clone)]
pub struct MermaidArtifacts {
    pub ast_generated: BTreeMap<String, String>,
    pub non_code: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct Template {
    pub name: String,
    pub content: String,
    pub hash: blake3::Hash,
    pub source_location: PathBuf,
}

impl UnifiedAstEngine {
    pub fn new() -> Self {
        Self {
            artifact_hasher: Hasher::new(),
            parsers: LanguageParsers::new(),
        }
    }

    /// Main entry point for deterministic artifact generation
    pub async fn generate_artifacts(&self, root: &Path) -> Result<ArtifactTree, TemplateError> {
        // Phase 1: Parse all source files into AST forest
        let ast_forest = self.parse_project(root).await?;

        // Phase 2: Extract dependency graph from AST
        let dependency_graph = self.extract_dependencies(&ast_forest)?;

        // Phase 3: Compute project-wide metrics
        let metrics = self.compute_metrics(&ast_forest)?;

        // Phase 4: Generate deterministic artifacts
        let artifacts = ArtifactTree {
            dogfooding: self.generate_dogfooding_artifacts(&ast_forest, &metrics)?,
            mermaid: self.generate_mermaid_artifacts(&dependency_graph, &metrics)?,
            templates: self.extract_embedded_templates(&ast_forest)?,
        };

        // Phase 5: Verify determinism via content hashing
        let hash = self.compute_tree_hash(&artifacts);
        println!("Generated artifact tree with hash: {hash}");

        Ok(artifacts)
    }

    /// Parse all source files in the project
    pub async fn parse_project(&self, root: &Path) -> Result<AstForest, TemplateError> {
        let mut forest = AstForest::new();

        // Find all source files in deterministic order
        let mut source_files = Vec::new();
        self.collect_source_files(root, &mut source_files)?;
        source_files.sort(); // Deterministic ordering

        for file_path in source_files {
            if let Some(ast) = self.parse_file(&file_path).await? {
                forest.add_file(file_path, ast);
            }
        }

        Ok(forest)
    }

    /// Collect all source files recursively
    fn collect_source_files(
        &self,
        dir: &Path,
        files: &mut Vec<PathBuf>,
    ) -> Result<(), TemplateError> {
        if !dir.is_dir() {
            return Ok(());
        }

        let entries = std::fs::read_dir(dir).map_err(TemplateError::Io)?;

        for entry in entries {
            let entry = entry.map_err(TemplateError::Io)?;
            let path = entry.path();

            if path.is_dir() {
                // Skip hidden directories and target/node_modules
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with('.') || name == "target" || name == "node_modules" {
                        continue;
                    }
                }
                self.collect_source_files(&path, files)?;
            } else if self.is_source_file(&path) {
                files.push(path);
            }
        }

        Ok(())
    }

    /// Check if a file is a supported source file
    fn is_source_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            matches!(
                ext,
                "rs" | "ts"
                    | "js"
                    | "tsx"
                    | "jsx"
                    | "py"
                    | "pyx"
                    | "pxd"
                    | "mk"
                    | "make"
                    | "c"
                    | "h"
                    | "cpp"
                    | "cc"
                    | "cxx"
                    | "hpp"
                    | "hxx"
            )
        } else if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            name == "Makefile" || name == "makefile"
        } else {
            false
        }
    }

    /// Parse a single file based on its extension using strategy pattern
    async fn parse_file(&self, path: &Path) -> Result<Option<FileAst>, TemplateError> {
        // Determine file type and delegate to appropriate parser
        let file_type = self.determine_file_type(path);
        self.parse_file_by_type(path, file_type).await
    }

    /// Determine file type from path (reduces complexity)
    fn determine_file_type(&self, path: &Path) -> FileType {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        match ext {
            "rs" => FileType::Rust,
            "ts" | "tsx" | "js" | "jsx" => FileType::TypeScript,
            "py" => FileType::Python,
            "pyx" | "pxd" => FileType::Cython,
            "c" | "h" => FileType::C,
            "cpp" | "cc" | "cxx" | "hpp" | "hxx" => FileType::Cpp,
            "mk" | "make" => FileType::Makefile,
            _ => {
                if filename == "Makefile" || filename == "makefile" {
                    FileType::Makefile
                } else {
                    FileType::Unknown
                }
            }
        }
    }

    /// Parse file based on determined type (strategy pattern)
    async fn parse_file_by_type(
        &self,
        path: &Path,
        file_type: FileType,
    ) -> Result<Option<FileAst>, TemplateError> {
        match file_type {
            FileType::Rust => self.parse_rust_file(path).await,
            FileType::TypeScript => self.parse_typescript_file(path).await,
            FileType::Python => self.parse_python_file(path).await,
            FileType::Cython => self.parse_cython_file(path).await,
            FileType::C => self.parse_c_file(path).await,
            FileType::Cpp => self.parse_cpp_file(path).await,
            FileType::Makefile => self.parse_makefile_file(path).await,
            FileType::Unknown => Ok(None),
        }
    }

    /// Parse Rust file
    async fn parse_rust_file(&self, path: &Path) -> Result<Option<FileAst>, TemplateError> {
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(TemplateError::Io)?;

        let ast = syn::parse_file(&content)
            .map_err(|e| TemplateError::InvalidUtf8(format!("Rust parse error: {e}")))?;

        Ok(Some(FileAst::Rust(ast)))
    }

    /// Parse TypeScript/JavaScript file
    async fn parse_typescript_file(&self, path: &Path) -> Result<Option<FileAst>, TemplateError> {
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(TemplateError::Io)?;
        Ok(Some(FileAst::TypeScript(content)))
    }

    /// Parse Python file
    async fn parse_python_file(&self, path: &Path) -> Result<Option<FileAst>, TemplateError> {
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(TemplateError::Io)?;
        Ok(Some(FileAst::Python(content)))
    }

    /// Parse Cython file
    async fn parse_cython_file(&self, path: &Path) -> Result<Option<FileAst>, TemplateError> {
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(TemplateError::Io)?;
        Ok(Some(FileAst::Cython(content)))
    }

    /// Parse C file
    async fn parse_c_file(&self, path: &Path) -> Result<Option<FileAst>, TemplateError> {
        #[cfg(feature = "c-ast")]
        {
            let content = tokio::fs::read_to_string(path)
                .await
                .map_err(TemplateError::Io)?;

            let mut parser = crate::services::ast_c::CAstParser::new();
            match parser.parse_file(path, &content) {
                Ok(ast_dag) => Ok(Some(FileAst::C(std::sync::Arc::new(ast_dag)))),
                Err(e) => Err(TemplateError::InvalidUtf8(format!("C parse error: {e}"))),
            }
        }
        #[cfg(not(feature = "c-ast"))]
        {
            let _content = tokio::fs::read_to_string(path)
                .await
                .map_err(TemplateError::Io)?;
            Ok(Some(FileAst::C(std::sync::Arc::new(
                crate::models::unified_ast::AstDag::new(),
            ))))
        }
    }

    /// Parse C++ file
    async fn parse_cpp_file(&self, path: &Path) -> Result<Option<FileAst>, TemplateError> {
        #[cfg(feature = "cpp-ast")]
        {
            let content = tokio::fs::read_to_string(path)
                .await
                .map_err(TemplateError::Io)?;

            let mut parser = crate::services::ast_cpp::CppAstParser::new();
            match parser.parse_file(path, &content) {
                Ok(ast_dag) => Ok(Some(FileAst::Cpp(std::sync::Arc::new(ast_dag)))),
                Err(e) => Err(TemplateError::InvalidUtf8(format!("C++ parse error: {e}"))),
            }
        }
        #[cfg(not(feature = "cpp-ast"))]
        {
            let _content = tokio::fs::read_to_string(path)
                .await
                .map_err(TemplateError::Io)?;
            Ok(Some(FileAst::Cpp(std::sync::Arc::new(
                crate::models::unified_ast::AstDag::new(),
            ))))
        }
    }

    /// Parse Makefile
    async fn parse_makefile_file(&self, path: &Path) -> Result<Option<FileAst>, TemplateError> {
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(TemplateError::Io)?;
        self.parse_makefile(&content)
            .map(|ast| Some(FileAst::Makefile(ast)))
    }

    /// Parse a Makefile
    fn parse_makefile(
        &self,
        content: &str,
    ) -> Result<crate::services::makefile_linter::MakefileAst, TemplateError> {
        let mut parser = crate::services::makefile_linter::MakefileParser::new(content);
        parser.parse().map_err(|errors| {
            TemplateError::InvalidUtf8(format!("Makefile parse errors: {errors:?}"))
        })
    }

    /// Extract dependency graph from AST forest using stable algorithms
    pub fn extract_dependencies(
        &self,
        forest: &AstForest,
    ) -> Result<StableGraph<ModuleNode, EdgeType>, TemplateError> {
        let mut graph = StableGraph::new();
        let mut node_indices = BTreeMap::new();

        // Phase 1: Create nodes from module declarations in deterministic order
        for (path, ast) in forest.files() {
            let module_name = self.path_to_module_name(path);
            let node_data = ModuleNode {
                name: module_name.clone(),
                path: path.clone(),
                visibility: ast.root_visibility(),
                metrics: self.compute_node_metrics(ast),
            };
            let idx = graph.add_node(node_data);
            node_indices.insert(module_name, idx);
        }

        // Phase 2: Extract edges from imports/uses in deterministic order
        for (path, ast) in forest.files() {
            let source = self.path_to_module_name(path);
            self.extract_dependencies_for_ast(ast, &source, &node_indices, &mut graph);
        }

        Ok(graph)
    }

    /// Extract dependencies for a single AST (reduces complexity)
    fn extract_dependencies_for_ast(
        &self,
        ast: &FileAst,
        source: &str,
        node_indices: &BTreeMap<String, petgraph::stable_graph::NodeIndex>,
        graph: &mut StableGraph<ModuleNode, EdgeType>,
    ) {
        match ast {
            FileAst::Rust(syn_ast) => {
                self.extract_rust_dependencies(syn_ast, source, node_indices, graph);
            }
            FileAst::TypeScript(_content) => {
                // TODO: Implement TypeScript import resolution
            }
            FileAst::Python(_content) => {
                // TODO: Implement Python import resolution
            }
            FileAst::Cython(_content) => {
                // TODO: Implement Cython import resolution (both Python and C imports)
            }
            FileAst::C(_ast) => {
                // TODO: Implement C #include resolution
            }
            FileAst::Cpp(_ast) => {
                // TODO: Implement C++ #include resolution
            }
            FileAst::Makefile(_ast) => {
                // Makefiles don't have traditional imports
            }
            FileAst::Markdown(_)
            | FileAst::Toml(_)
            | FileAst::Yaml(_)
            | FileAst::Json(_)
            | FileAst::Shell(_) => {
                // These file types don't have traditional imports
            }
        }
    }

    /// Extract dependencies from Rust AST
    fn extract_rust_dependencies(
        &self,
        syn_ast: &syn::File,
        source: &str,
        node_indices: &BTreeMap<String, petgraph::stable_graph::NodeIndex>,
        graph: &mut StableGraph<ModuleNode, EdgeType>,
    ) {
        for item in &syn_ast.items {
            if let syn::Item::Use(use_item) = item {
                let targets = self.resolve_rust_imports(use_item);
                for target in targets {
                    if let Some(&target_idx) = node_indices.get(&target) {
                        if let Some(&source_idx) = node_indices.get(source) {
                            graph.add_edge(source_idx, target_idx, EdgeType::Imports);
                        }
                    }
                }
            }
        }
    }

    /// Convert file path to module name
    fn path_to_module_name(&self, path: &Path) -> String {
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .replace('-', "_")
    }

    /// Compute metrics for a single module/file (delegated approach)
    fn compute_node_metrics(&self, ast: &FileAst) -> ModuleMetrics {
        match ast {
            FileAst::Rust(syn_ast) => self.compute_rust_metrics(syn_ast),
            FileAst::TypeScript(_) | FileAst::Python(_) | FileAst::Cython(_) => {
                ModuleMetrics::default() // TODO: Implement for other languages
            }
            FileAst::C(ast_dag) | FileAst::Cpp(ast_dag) => self.compute_c_cpp_metrics(ast_dag),
            FileAst::Makefile(makefile_ast) => self.compute_makefile_metrics(makefile_ast),
            FileAst::Markdown(_)
            | FileAst::Toml(_)
            | FileAst::Yaml(_)
            | FileAst::Json(_)
            | FileAst::Shell(_) => ModuleMetrics::default(),
        }
    }

    /// Compute metrics for Rust AST
    fn compute_rust_metrics(&self, syn_ast: &syn::File) -> ModuleMetrics {
        let mut metrics = ModuleMetrics::default();

        for item in &syn_ast.items {
            match item {
                syn::Item::Fn(_) => metrics.functions += 1,
                syn::Item::Struct(_) | syn::Item::Enum(_) => metrics.classes += 1,
                _ => {}
            }
        }

        metrics.complexity = (metrics.functions + metrics.classes) * 2; // Simple heuristic
        metrics.lines = syn_ast.items.len() as u32 * 10; // Rough estimate
        metrics
    }

    /// Compute metrics for C/C++ AST
    fn compute_c_cpp_metrics(
        &self,
        ast_dag: &std::sync::Arc<crate::models::unified_ast::AstDag>,
    ) -> ModuleMetrics {
        let mut metrics = ModuleMetrics::default();

        // Count functions and complexity from AST DAG
        for node in ast_dag.nodes.iter() {
            match &node.kind {
                crate::models::unified_ast::AstKind::Function(_) => {
                    metrics.functions += 1;
                    metrics.complexity += node.complexity();
                }
                crate::models::unified_ast::AstKind::Type(type_kind) => {
                    if matches!(
                        type_kind,
                        crate::models::unified_ast::TypeKind::Struct
                            | crate::models::unified_ast::TypeKind::Enum
                    ) {
                        metrics.classes += 1;
                    }
                }
                _ => {}
            }
        }

        metrics.lines = ast_dag.nodes.len() as u32 * 5; // Rough estimate
        metrics
    }

    /// Compute metrics for Makefile
    fn compute_makefile_metrics(
        &self,
        makefile_ast: &crate::services::makefile_linter::MakefileAst,
    ) -> ModuleMetrics {
        let mut metrics = ModuleMetrics::default();
        metrics.functions = makefile_ast
            .nodes
            .iter()
            .filter(|n| n.kind == crate::services::makefile_linter::MakefileNodeKind::Rule)
            .count() as u32;
        metrics.lines = makefile_ast.nodes.len() as u32 * 3; // Rough estimate
        metrics.complexity = metrics.functions; // Simple heuristic
        metrics
    }

    /// Resolve Rust use/import statements
    fn resolve_rust_imports(&self, use_item: &syn::ItemUse) -> Vec<String> {
        let mut imports = Vec::new();

        // Simple implementation - would need more sophisticated path resolution
        if let syn::UseTree::Path(path) = &use_item.tree {
            imports.push(path.ident.to_string());
        }

        imports
    }

    /// Compute project-wide metrics
    pub fn compute_metrics(&self, forest: &AstForest) -> Result<ProjectMetrics, TemplateError> {
        let mut total_functions = 0;
        let mut total_complexity = 0u64;
        let mut max_complexity = 0u32;

        for (_path, ast) in forest.files() {
            let metrics = self.compute_node_metrics(ast);
            total_functions += metrics.functions;
            total_complexity += metrics.complexity as u64;
            max_complexity = max_complexity.max(metrics.complexity);
        }

        let file_count = forest.files.len();
        let avg_complexity = if total_functions > 0 {
            total_complexity as f64 / total_functions as f64
        } else {
            0.0
        };

        Ok(ProjectMetrics {
            file_count,
            function_count: total_functions as usize,
            avg_complexity,
            max_complexity,
        })
    }

    /// Generate dogfooding artifacts (self-analysis)
    fn generate_dogfooding_artifacts(
        &self,
        forest: &AstForest,
        metrics: &ProjectMetrics,
    ) -> Result<BTreeMap<String, String>, TemplateError> {
        let mut artifacts = BTreeMap::new();

        // Generate AST context
        let ast_context = self.generate_ast_context(forest)?;
        artifacts.insert("ast-context-2025-05-31.md".to_string(), ast_context);

        // Generate combined metrics
        let combined_metrics = self.generate_combined_metrics(metrics)?;
        artifacts.insert(
            "combined-metrics-2025-05-31.json".to_string(),
            combined_metrics,
        );

        // Generate complexity analysis
        let complexity_md = self.generate_complexity_analysis(metrics)?;
        artifacts.insert("complexity-2025-05-31.md".to_string(), complexity_md);

        Ok(artifacts)
    }

    /// Generate AST context analysis
    fn generate_ast_context(&self, forest: &AstForest) -> Result<String, TemplateError> {
        let mut context = String::new();

        context.push_str("# AST Context Analysis - 2025-05-31\n\n");
        context.push_str("## Project Structure\n\n");

        // Process files in deterministic order
        let mut sorted_files: Vec<_> = forest.files().collect();
        sorted_files.sort_by_key(|(path, _)| path.to_string_lossy());

        for (path, ast) in sorted_files {
            context.push_str(&format!("### {}\n\n", path.display()));

            let metrics = self.compute_node_metrics(ast);
            context.push_str(&format!("- **Functions**: {}\n", metrics.functions));
            context.push_str(&format!("- **Classes**: {}\n", metrics.classes));
            context.push_str(&format!("- **Complexity**: {}\n\n", metrics.complexity));
        }

        Ok(context)
    }

    /// Generate combined metrics JSON
    fn generate_combined_metrics(&self, metrics: &ProjectMetrics) -> Result<String, TemplateError> {
        let json_data = json!({
            "timestamp": "2025-05-31",
            "ast": {
                "total_files": metrics.file_count,
                "total_functions": metrics.function_count,
                "avg_complexity": metrics.avg_complexity,
                "max_complexity": metrics.max_complexity,
            },
            "generation": "unified_ast_engine",
            "hash": "deterministic_placeholder"
        });

        serde_json::to_string_pretty(&json_data)
            .map_err(|e| TemplateError::InvalidUtf8(e.to_string()))
    }

    /// Generate complexity analysis markdown
    fn generate_complexity_analysis(
        &self,
        metrics: &ProjectMetrics,
    ) -> Result<String, TemplateError> {
        let mut analysis = String::new();

        analysis.push_str("# Complexity Analysis - 2025-05-31\n\n");
        analysis.push_str("## Summary\n\n");
        analysis.push_str(&format!("- **Total Files**: {}\n", metrics.file_count));
        analysis.push_str(&format!(
            "- **Total Functions**: {}\n",
            metrics.function_count
        ));
        analysis.push_str(&format!(
            "- **Average Complexity**: {:.2}\n",
            metrics.avg_complexity
        ));
        analysis.push_str(&format!(
            "- **Maximum Complexity**: {}\n\n",
            metrics.max_complexity
        ));

        Ok(analysis)
    }

    /// Generate Mermaid artifacts (placeholder)
    fn generate_mermaid_artifacts(
        &self,
        _graph: &StableGraph<ModuleNode, EdgeType>,
        _metrics: &ProjectMetrics,
    ) -> Result<MermaidArtifacts, TemplateError> {
        Ok(MermaidArtifacts {
            ast_generated: BTreeMap::new(),
            non_code: BTreeMap::new(),
        })
    }

    /// Extract embedded templates from AST
    fn extract_embedded_templates(
        &self,
        forest: &AstForest,
    ) -> Result<Vec<Template>, TemplateError> {
        let mut templates = Vec::new();

        for (path, ast) in forest.files() {
            if path.file_name().and_then(|n| n.to_str()) == Some("embedded_templates.rs") {
                if let FileAst::Rust(syn_ast) = ast {
                    templates.extend(self.extract_rust_templates(path, syn_ast)?);
                }
            }
        }

        // Sort for deterministic output
        templates.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(templates)
    }

    /// Extract templates from Rust embedded_templates.rs file
    fn extract_rust_templates(
        &self,
        path: &Path,
        ast: &syn::File,
    ) -> Result<Vec<Template>, TemplateError> {
        let mut templates = Vec::new();

        for item in &ast.items {
            if let syn::Item::Const(const_item) = item {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit_str),
                    ..
                }) = &*const_item.expr
                {
                    let name = const_item.ident.to_string();
                    let content = lit_str.value();
                    let hash = blake3::hash(content.as_bytes());

                    templates.push(Template {
                        name,
                        content,
                        hash,
                        source_location: path.to_path_buf(),
                    });
                }
            }
        }

        Ok(templates)
    }

    /// Compute hash of entire artifact tree for determinism verification
    fn compute_tree_hash(&self, artifacts: &ArtifactTree) -> blake3::Hash {
        let mut hasher = Hasher::new();

        // Hash dogfooding artifacts in deterministic order
        for (name, content) in &artifacts.dogfooding {
            hasher.update(name.as_bytes());
            hasher.update(content.as_bytes());
        }

        // Hash mermaid artifacts
        for (name, content) in &artifacts.mermaid.ast_generated {
            hasher.update(name.as_bytes());
            hasher.update(content.as_bytes());
        }

        // Hash templates
        for template in &artifacts.templates {
            hasher.update(template.name.as_bytes());
            hasher.update(template.content.as_bytes());
        }

        hasher.finalize()
    }
}

impl Default for UnifiedAstEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_deterministic_artifact_generation() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create a simple Rust file
        let rust_file = temp_path.join("lib.rs");
        fs::write(
            &rust_file,
            r#"
            pub fn hello() -> String {
                "Hello, World!".to_string()
            }
            
            pub struct Config {
                pub name: String,
            }
        "#,
        )
        .unwrap();

        let engine = UnifiedAstEngine::new();

        // Generate artifacts multiple times
        let artifacts1 = engine.generate_artifacts(temp_path).await.unwrap();
        let artifacts2 = engine.generate_artifacts(temp_path).await.unwrap();

        // Hashes should be identical for determinism
        let hash1 = engine.compute_tree_hash(&artifacts1);
        let hash2 = engine.compute_tree_hash(&artifacts2);

        assert_eq!(hash1, hash2, "Artifact generation must be deterministic");
    }

    #[test]
    fn test_path_to_module_name() {
        let engine = UnifiedAstEngine::new();

        assert_eq!(engine.path_to_module_name(Path::new("foo.rs")), "foo");
        assert_eq!(
            engine.path_to_module_name(Path::new("foo-bar.rs")),
            "foo_bar"
        );
        assert_eq!(engine.path_to_module_name(Path::new("src/lib.rs")), "lib");
    }

    #[test]
    fn test_is_source_file() {
        let engine = UnifiedAstEngine::new();

        assert!(engine.is_source_file(Path::new("foo.rs")));
        assert!(engine.is_source_file(Path::new("foo.ts")));
        assert!(engine.is_source_file(Path::new("foo.py")));
        assert!(engine.is_source_file(Path::new("foo.c")));
        assert!(engine.is_source_file(Path::new("foo.h")));
        assert!(engine.is_source_file(Path::new("foo.cpp")));
        assert!(engine.is_source_file(Path::new("foo.cc")));
        assert!(engine.is_source_file(Path::new("foo.hpp")));
        assert!(!engine.is_source_file(Path::new("foo.txt")));
        assert!(!engine.is_source_file(Path::new("Cargo.toml")));
    }
}
