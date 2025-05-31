use crate::models::error::TemplateError;
use crate::services::cache::{
    manager::SessionCacheManager, persistent_manager::PersistentCacheManager,
};
use crate::services::{ast_python, ast_typescript};
use ignore::gitignore::GitignoreBuilder;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use syn::visit::Visit;
use syn::{ItemEnum, ItemFn, ItemImpl, ItemMod, ItemStruct, ItemTrait, ItemUse};
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectContext {
    pub project_type: String,
    pub files: Vec<FileContext>,
    pub summary: ProjectSummary,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectSummary {
    pub total_files: usize,
    pub total_functions: usize,
    pub total_structs: usize,
    pub total_enums: usize,
    pub total_traits: usize,
    pub total_impls: usize,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileContext {
    pub path: String,
    pub language: String,
    pub items: Vec<AstItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complexity_metrics: Option<crate::services::complexity::FileComplexityMetrics>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum AstItem {
    Function {
        name: String,
        visibility: String,
        is_async: bool,
        line: usize,
    },
    Struct {
        name: String,
        visibility: String,
        fields_count: usize,
        derives: Vec<String>,
        line: usize,
    },
    Enum {
        name: String,
        visibility: String,
        variants_count: usize,
        line: usize,
    },
    Trait {
        name: String,
        visibility: String,
        line: usize,
    },
    Impl {
        type_name: String,
        trait_name: Option<String>,
        line: usize,
    },
    Module {
        name: String,
        visibility: String,
        line: usize,
    },
    Use {
        path: String,
        line: usize,
    },
}

struct RustVisitor {
    items: Vec<AstItem>,
    #[allow(dead_code)]
    source: String,
}

impl RustVisitor {
    fn new(source: String) -> Self {
        Self {
            items: Vec::new(),
            source,
        }
    }

    fn get_line(&self, _span: proc_macro2::Span) -> usize {
        // For simplicity, return 1. In production, use a proper source map
        1
    }

    fn get_visibility(&self, vis: &syn::Visibility) -> String {
        match vis {
            syn::Visibility::Public(_) => "pub".to_string(),
            syn::Visibility::Restricted(r) => format!(
                "pub({})",
                r.path
                    .segments
                    .iter()
                    .map(|s| s.ident.to_string())
                    .collect::<Vec<_>>()
                    .join("::")
            ),
            syn::Visibility::Inherited => "private".to_string(),
        }
    }

    fn get_derives(_attrs: &[syn::Attribute]) -> Vec<String> {
        // Simplified version - in production, parse derive attributes properly
        Vec::new()
    }
}

impl<'ast> Visit<'ast> for RustVisitor {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        self.items.push(AstItem::Function {
            name: node.sig.ident.to_string(),
            visibility: self.get_visibility(&node.vis),
            is_async: node.sig.asyncness.is_some(),
            line: self.get_line(node.sig.ident.span()),
        });
    }

    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        let fields_count = match &node.fields {
            syn::Fields::Named(fields) => fields.named.len(),
            syn::Fields::Unnamed(fields) => fields.unnamed.len(),
            syn::Fields::Unit => 0,
        };

        self.items.push(AstItem::Struct {
            name: node.ident.to_string(),
            visibility: self.get_visibility(&node.vis),
            fields_count,
            derives: Self::get_derives(&node.attrs),
            line: self.get_line(node.ident.span()),
        });
    }

    fn visit_item_enum(&mut self, node: &'ast ItemEnum) {
        self.items.push(AstItem::Enum {
            name: node.ident.to_string(),
            visibility: self.get_visibility(&node.vis),
            variants_count: node.variants.len(),
            line: self.get_line(node.ident.span()),
        });
    }

    fn visit_item_trait(&mut self, node: &'ast ItemTrait) {
        self.items.push(AstItem::Trait {
            name: node.ident.to_string(),
            visibility: self.get_visibility(&node.vis),
            line: self.get_line(node.ident.span()),
        });
    }

    fn visit_item_impl(&mut self, node: &'ast ItemImpl) {
        let type_name = if let syn::Type::Path(type_path) = &*node.self_ty {
            type_path
                .path
                .segments
                .last()
                .map(|s| s.ident.to_string())
                .unwrap_or_else(|| "Unknown".to_string())
        } else {
            "Unknown".to_string()
        };

        let trait_name = node.trait_.as_ref().map(|(_, path, _)| {
            path.segments
                .last()
                .map(|s| s.ident.to_string())
                .unwrap_or_else(|| "Unknown".to_string())
        });

        self.items.push(AstItem::Impl {
            type_name,
            trait_name,
            line: 1, // Default line number
        });
    }

    fn visit_item_mod(&mut self, node: &'ast ItemMod) {
        self.items.push(AstItem::Module {
            name: node.ident.to_string(),
            visibility: self.get_visibility(&node.vis),
            line: self.get_line(node.ident.span()),
        });
    }

    fn visit_item_use(&mut self, node: &'ast ItemUse) {
        let path = match &node.tree {
            syn::UseTree::Path(p) => p.ident.to_string(),
            syn::UseTree::Name(n) => n.ident.to_string(),
            syn::UseTree::Rename(r) => r.ident.to_string(),
            syn::UseTree::Glob(_) => "*".to_string(),
            syn::UseTree::Group(_) => "...".to_string(),
        };

        self.items.push(AstItem::Use {
            path,
            line: 1, // Default line number
        });
    }
}

pub async fn analyze_rust_file(path: &Path) -> Result<FileContext, TemplateError> {
    analyze_rust_file_with_cache(path, None).await
}

pub async fn analyze_rust_file_with_cache(
    path: &Path,
    cache_manager: Option<Arc<SessionCacheManager>>,
) -> Result<FileContext, TemplateError> {
    if let Some(cache) = cache_manager {
        cache
            .get_or_compute_ast(path, || async {
                // Parse the file
                let content = tokio::fs::read_to_string(path)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to read file: {}", e))?;

                let syntax = syn::parse_file(&content)
                    .map_err(|e| anyhow::anyhow!("Failed to parse Rust file: {}", e))?;

                let mut visitor = RustVisitor::new(content);
                visitor.visit_file(&syntax);

                Ok(FileContext {
                    path: path.display().to_string(),
                    language: "rust".to_string(),
                    items: visitor.items,
                    complexity_metrics: None,
                })
            })
            .await
            .map(|arc| (*arc).clone())
            .map_err(|e| TemplateError::InvalidUtf8(e.to_string()))
    } else {
        // No cache, compute directly
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(TemplateError::Io)?;

        let syntax =
            syn::parse_file(&content).map_err(|e| TemplateError::InvalidUtf8(e.to_string()))?;

        let mut visitor = RustVisitor::new(content);
        visitor.visit_file(&syntax);

        Ok(FileContext {
            path: path.display().to_string(),
            language: "rust".to_string(),
            items: visitor.items,
            complexity_metrics: None,
        })
    }
}

pub async fn analyze_project(
    root_path: &Path,
    toolchain: &str,
) -> Result<ProjectContext, TemplateError> {
    analyze_project_with_cache(root_path, toolchain, None).await
}

// Persistent cache version
pub async fn analyze_rust_file_with_persistent_cache(
    path: &Path,
    cache_manager: Option<Arc<PersistentCacheManager>>,
) -> Result<FileContext, TemplateError> {
    if let Some(cache) = cache_manager {
        cache
            .get_or_compute_ast(path, || async {
                // Parse the file
                let content = tokio::fs::read_to_string(path)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to read file: {}", e))?;

                let syntax = syn::parse_file(&content)
                    .map_err(|e| anyhow::anyhow!("Failed to parse Rust file: {}", e))?;

                let mut visitor = RustVisitor::new(content);
                visitor.visit_file(&syntax);

                Ok(FileContext {
                    path: path.display().to_string(),
                    language: "rust".to_string(),
                    items: visitor.items,
                    complexity_metrics: None,
                })
            })
            .await
            .map(|arc| (*arc).clone())
            .map_err(|e| TemplateError::InvalidUtf8(e.to_string()))
    } else {
        // No cache, compute directly
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(TemplateError::Io)?;

        let syntax =
            syn::parse_file(&content).map_err(|e| TemplateError::InvalidUtf8(e.to_string()))?;

        let mut visitor = RustVisitor::new(content);
        visitor.visit_file(&syntax);

        Ok(FileContext {
            path: path.display().to_string(),
            language: "rust".to_string(),
            items: visitor.items,
            complexity_metrics: None,
        })
    }
}

pub async fn analyze_project_with_cache(
    root_path: &Path,
    toolchain: &str,
    cache_manager: Option<Arc<SessionCacheManager>>,
) -> Result<ProjectContext, TemplateError> {
    let gitignore = build_gitignore(root_path)?;
    let files = scan_and_analyze_files(root_path, toolchain, cache_manager, &gitignore).await;
    let summary = build_project_summary(&files, root_path, toolchain).await;

    Ok(ProjectContext {
        project_type: toolchain.to_string(),
        files,
        summary,
    })
}

fn build_gitignore(root_path: &Path) -> Result<ignore::gitignore::Gitignore, TemplateError> {
    let mut gitignore = GitignoreBuilder::new(root_path);

    // Add default ignores
    let default_ignores = [".git", "target", "node_modules", ".venv", "__pycache__"];
    for pattern in &default_ignores {
        gitignore.add_line(None, pattern).ok();
    }

    if let Ok(gi_path) = root_path.join(".gitignore").canonicalize() {
        gitignore.add(&gi_path);
    }

    gitignore
        .build()
        .map_err(|e| TemplateError::InvalidUtf8(e.to_string()))
}

async fn scan_and_analyze_files(
    root_path: &Path,
    toolchain: &str,
    cache_manager: Option<Arc<SessionCacheManager>>,
    gitignore: &ignore::gitignore::Gitignore,
) -> Vec<FileContext> {
    let mut files = Vec::new();

    for entry in WalkDir::new(root_path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Skip if gitignored
        if gitignore.matched(path, path.is_dir()).is_ignore() {
            continue;
        }

        if let Some(file_context) =
            analyze_file_by_toolchain(path, toolchain, cache_manager.clone()).await
        {
            files.push(file_context);
        }
    }

    files
}

async fn analyze_file_by_toolchain(
    path: &Path,
    toolchain: &str,
    cache_manager: Option<Arc<SessionCacheManager>>,
) -> Option<FileContext> {
    match toolchain {
        "rust" => {
            if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                analyze_rust_file_with_cache(path, cache_manager).await.ok()
            } else {
                None
            }
        }
        "deno" => analyze_deno_file(path).await,
        "python-uv" => {
            if path.extension().and_then(|s| s.to_str()) == Some("py") {
                ast_python::analyze_python_file(path).await.ok()
            } else {
                None
            }
        }
        _ => None,
    }
}

async fn analyze_deno_file(path: &Path) -> Option<FileContext> {
    let ext = path.extension().and_then(|s| s.to_str());
    match ext {
        Some("ts") | Some("tsx") => ast_typescript::analyze_typescript_file(path).await.ok(),
        Some("js") | Some("jsx") => ast_typescript::analyze_javascript_file(path).await.ok(),
        _ => None,
    }
}

async fn build_project_summary(
    files: &[FileContext],
    root_path: &Path,
    toolchain: &str,
) -> ProjectSummary {
    let mut summary = ProjectSummary {
        total_files: files.len(),
        total_functions: 0,
        total_structs: 0,
        total_enums: 0,
        total_traits: 0,
        total_impls: 0,
        dependencies: Vec::new(),
    };

    // Calculate item counts
    calculate_item_counts(&mut summary, files);

    // Read dependencies
    summary.dependencies = read_dependencies(root_path, toolchain).await;

    summary
}

fn calculate_item_counts(summary: &mut ProjectSummary, files: &[FileContext]) {
    for file in files {
        for item in &file.items {
            match item {
                AstItem::Function { .. } => summary.total_functions += 1,
                AstItem::Struct { .. } => summary.total_structs += 1,
                AstItem::Enum { .. } => summary.total_enums += 1,
                AstItem::Trait { .. } => summary.total_traits += 1,
                AstItem::Impl { .. } => summary.total_impls += 1,
                _ => {}
            }
        }
    }
}

async fn read_dependencies(root_path: &Path, toolchain: &str) -> Vec<String> {
    match toolchain {
        "rust" => read_rust_dependencies(root_path).await,
        "deno" => read_deno_dependencies(root_path).await,
        "python-uv" => read_python_dependencies(root_path).await,
        _ => Vec::new(),
    }
}

async fn read_rust_dependencies(root_path: &Path) -> Vec<String> {
    if let Ok(cargo_content) = tokio::fs::read_to_string(root_path.join("Cargo.toml")).await {
        if let Ok(cargo_toml) = cargo_content.parse::<toml::Value>() {
            if let Some(deps) = cargo_toml.get("dependencies").and_then(|d| d.as_table()) {
                return deps.keys().cloned().collect();
            }
        }
    }
    Vec::new()
}

async fn read_deno_dependencies(root_path: &Path) -> Vec<String> {
    let mut dependencies = Vec::new();

    // Check deno.json
    if let Ok(deno_json) = tokio::fs::read_to_string(root_path.join("deno.json")).await {
        if let Ok(deno_config) = serde_json::from_str::<serde_json::Value>(&deno_json) {
            if let Some(imports) = deno_config.get("imports").and_then(|i| i.as_object()) {
                dependencies.extend(imports.keys().cloned());
            }
        }
    }

    // Check package.json
    if let Ok(package_json) = tokio::fs::read_to_string(root_path.join("package.json")).await {
        if let Ok(package) = serde_json::from_str::<serde_json::Value>(&package_json) {
            if let Some(deps) = package.get("dependencies").and_then(|d| d.as_object()) {
                dependencies.extend(deps.keys().cloned());
            }
        }
    }

    dependencies
}

async fn read_python_dependencies(root_path: &Path) -> Vec<String> {
    let mut dependencies = Vec::new();

    // Check pyproject.toml
    if let Ok(pyproject_content) = tokio::fs::read_to_string(root_path.join("pyproject.toml")).await
    {
        if let Ok(pyproject) = pyproject_content.parse::<toml::Value>() {
            if let Some(deps) = pyproject
                .get("project")
                .and_then(|p| p.get("dependencies"))
                .and_then(|d| d.as_array())
            {
                dependencies.extend(
                    deps.iter()
                        .filter_map(|d| d.as_str())
                        .map(|s| s.split_whitespace().next().unwrap_or(s).to_string()),
                );
            }
        }
    }

    // Check requirements.txt
    if let Ok(requirements) = tokio::fs::read_to_string(root_path.join("requirements.txt")).await {
        for line in requirements.lines() {
            let line = line.trim();
            if !line.is_empty() && !line.starts_with('#') {
                let dep_name = line
                    .split(['=', '>', '<', '~'])
                    .next()
                    .unwrap_or(line)
                    .trim();
                dependencies.push(dep_name.to_string());
            }
        }
    }

    dependencies
}

pub async fn analyze_project_with_persistent_cache(
    root_path: &Path,
    toolchain: &str,
    cache_manager: Option<Arc<PersistentCacheManager>>,
) -> Result<ProjectContext, TemplateError> {
    let gitignore = build_gitignore(root_path)?;
    let files =
        scan_and_analyze_files_persistent(root_path, toolchain, cache_manager, &gitignore).await;
    let summary = build_project_summary(&files, root_path, toolchain).await;

    Ok(ProjectContext {
        project_type: toolchain.to_string(),
        files,
        summary,
    })
}

async fn scan_and_analyze_files_persistent(
    root_path: &Path,
    toolchain: &str,
    cache_manager: Option<Arc<PersistentCacheManager>>,
    gitignore: &ignore::gitignore::Gitignore,
) -> Vec<FileContext> {
    let mut files = Vec::new();

    for entry in WalkDir::new(root_path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Skip if gitignored
        if gitignore.matched(path, path.is_dir()).is_ignore() {
            continue;
        }

        if let Some(file_context) =
            analyze_file_by_toolchain_persistent(path, toolchain, cache_manager.clone()).await
        {
            files.push(file_context);
        }
    }

    files
}

async fn analyze_file_by_toolchain_persistent(
    path: &Path,
    toolchain: &str,
    cache_manager: Option<Arc<PersistentCacheManager>>,
) -> Option<FileContext> {
    match toolchain {
        "rust" => {
            if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                analyze_rust_file_with_persistent_cache(path, cache_manager)
                    .await
                    .ok()
            } else {
                None
            }
        }
        "deno" => analyze_deno_file(path).await,
        "python-uv" => {
            if path.extension().and_then(|s| s.to_str()) == Some("py") {
                ast_python::analyze_python_file(path).await.ok()
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn format_context_as_markdown(context: &ProjectContext) -> String {
    let mut output = String::new();

    output.push_str(&format!(
        "# Project Context: {} Project\n\n",
        context.project_type
    ));
    output.push_str(&format!(
        "Generated: {}\n\n",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    ));

    output.push_str("## Summary\n\n");
    output.push_str(&format!(
        "- Files analyzed: {}\n",
        context.summary.total_files
    ));
    output.push_str(&format!(
        "- Functions: {}\n",
        context.summary.total_functions
    ));
    output.push_str(&format!("- Structs: {}\n", context.summary.total_structs));
    output.push_str(&format!("- Enums: {}\n", context.summary.total_enums));
    output.push_str(&format!("- Traits: {}\n", context.summary.total_traits));
    output.push_str(&format!(
        "- Implementations: {}\n",
        context.summary.total_impls
    ));

    if !context.summary.dependencies.is_empty() {
        output.push_str("\n## Dependencies\n\n");
        for dep in &context.summary.dependencies {
            output.push_str(&format!("- {}\n", dep));
        }
    }

    output.push_str("\n## Files\n\n");

    for file in &context.files {
        output.push_str(&format!("### {}\n\n", file.path));

        // Group items by type
        let mut functions = Vec::new();
        let mut structs = Vec::new();
        let mut enums = Vec::new();
        let mut traits = Vec::new();
        let mut impls = Vec::new();
        let mut modules = Vec::new();

        for item in &file.items {
            match item {
                AstItem::Function { .. } => functions.push(item),
                AstItem::Struct { .. } => structs.push(item),
                AstItem::Enum { .. } => enums.push(item),
                AstItem::Trait { .. } => traits.push(item),
                AstItem::Impl { .. } => impls.push(item),
                AstItem::Module { .. } => modules.push(item),
                _ => {}
            }
        }

        if !modules.is_empty() {
            output.push_str("**Modules:**\n");
            for item in modules {
                if let AstItem::Module {
                    name,
                    visibility,
                    line,
                } = item
                {
                    output.push_str(&format!(
                        "- `{} mod {}` (line {})\n",
                        visibility, name, line
                    ));
                }
            }
            output.push('\n');
        }

        if !structs.is_empty() {
            output.push_str("**Structs:**\n");
            for item in structs {
                if let AstItem::Struct {
                    name,
                    visibility,
                    fields_count,
                    derives,
                    line,
                } = item
                {
                    output.push_str(&format!(
                        "- `{} struct {}` ({} fields)",
                        visibility, name, fields_count
                    ));
                    if !derives.is_empty() {
                        output.push_str(&format!(" [derives: {}]", derives.join(", ")));
                    }
                    output.push_str(&format!(" (line {})\n", line));
                }
            }
            output.push('\n');
        }

        if !enums.is_empty() {
            output.push_str("**Enums:**\n");
            for item in enums {
                if let AstItem::Enum {
                    name,
                    visibility,
                    variants_count,
                    line,
                } = item
                {
                    output.push_str(&format!(
                        "- `{} enum {}` ({} variants) (line {})\n",
                        visibility, name, variants_count, line
                    ));
                }
            }
            output.push('\n');
        }

        if !traits.is_empty() {
            output.push_str("**Traits:**\n");
            for item in traits {
                if let AstItem::Trait {
                    name,
                    visibility,
                    line,
                } = item
                {
                    output.push_str(&format!(
                        "- `{} trait {}` (line {})\n",
                        visibility, name, line
                    ));
                }
            }
            output.push('\n');
        }

        if !functions.is_empty() {
            output.push_str("**Functions:**\n");
            for item in functions {
                if let AstItem::Function {
                    name,
                    visibility,
                    is_async,
                    line,
                } = item
                {
                    output.push_str(&format!(
                        "- `{} {}fn {}` (line {})\n",
                        visibility,
                        if *is_async { "async " } else { "" },
                        name,
                        line
                    ));
                }
            }
            output.push('\n');
        }

        if !impls.is_empty() {
            output.push_str("**Implementations:**\n");
            for item in impls {
                if let AstItem::Impl {
                    type_name,
                    trait_name,
                    line,
                } = item
                {
                    if let Some(trait_name) = trait_name {
                        output.push_str(&format!(
                            "- `impl {} for {}` (line {})\n",
                            trait_name, type_name, line
                        ));
                    } else {
                        output.push_str(&format!("- `impl {}` (line {})\n", type_name, line));
                    }
                }
            }
            output.push('\n');
        }
    }

    output.push_str("---\n");
    output.push_str("Generated by paiml-mcp-agent-toolkit\n");

    output
}
