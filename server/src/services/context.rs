use crate::models::error::TemplateError;
#[cfg(feature = "python-ast")]
use crate::services::ast_python;
#[cfg(feature = "typescript-ast")]
use crate::services::ast_typescript;
use crate::services::cache::{
    manager::SessionCacheManager, persistent_manager::PersistentCacheManager,
};
use crate::services::deep_context::DeepContext;
use futures::future::join_all;
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

impl AstItem {
    pub fn display_name(&self) -> &str {
        match self {
            AstItem::Function { name, .. } => name,
            AstItem::Struct { name, .. } => name,
            AstItem::Enum { name, .. } => name,
            AstItem::Trait { name, .. } => name,
            AstItem::Impl { type_name, .. } => type_name,
            AstItem::Module { name, .. } => name,
            AstItem::Use { path, .. } => path,
        }
    }
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
    // First, collect all file paths to analyze
    let paths: Vec<_> = WalkDir::new(root_path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|entry| {
            let path = entry.path();
            !path.is_dir() && !gitignore.matched(path, false).is_ignore()
        })
        .map(|entry| entry.path().to_path_buf())
        .collect();

    // Process files in parallel
    let tasks: Vec<_> = paths
        .into_iter()
        .map(|path| {
            let toolchain = toolchain.to_string();
            let cache_manager = cache_manager.clone();
            tokio::spawn(async move {
                analyze_file_by_toolchain(&path, &toolchain, cache_manager).await
            })
        })
        .collect();

    // Wait for all tasks to complete and collect results
    let results = join_all(tasks).await;
    results
        .into_iter()
        .filter_map(|result| result.ok())
        .flatten()
        .collect()
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
            #[cfg(feature = "python-ast")]
            if path.extension().and_then(|s| s.to_str()) == Some("py") {
                ast_python::analyze_python_file(path).await.ok()
            } else {
                None
            }
            #[cfg(not(feature = "python-ast"))]
            None
        }
        "kotlin" => {
            #[cfg(feature = "kotlin-ast")]
            {
                use crate::services::ast_strategies::{AstStrategy, KotlinAstStrategy};
                use crate::services::file_classifier::FileClassifier;
                let ext = path.extension().and_then(|s| s.to_str());
                if matches!(ext, Some("kt") | Some("kts")) {
                    let classifier = FileClassifier::new();
                    let strategy = KotlinAstStrategy;
                    strategy.analyze(path, &classifier).await.ok()
                } else {
                    None
                }
            }
            #[cfg(not(feature = "kotlin-ast"))]
            None
        }
        _ => None,
    }
}

async fn analyze_deno_file(path: &Path) -> Option<FileContext> {
    let ext = path.extension().and_then(|s| s.to_str());
    match ext {
        #[cfg(feature = "typescript-ast")]
        Some("ts") | Some("tsx") => ast_typescript::analyze_typescript_file(path).await.ok(),
        #[cfg(feature = "typescript-ast")]
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
            #[cfg(feature = "python-ast")]
            if path.extension().and_then(|s| s.to_str()) == Some("py") {
                ast_python::analyze_python_file(path).await.ok()
            } else {
                None
            }
            #[cfg(not(feature = "python-ast"))]
            None
        }
        "kotlin" => {
            #[cfg(feature = "kotlin-ast")]
            {
                use crate::services::ast_strategies::{AstStrategy, KotlinAstStrategy};
                use crate::services::file_classifier::FileClassifier;
                let ext = path.extension().and_then(|s| s.to_str());
                if matches!(ext, Some("kt") | Some("kts")) {
                    let classifier = FileClassifier::new();
                    let strategy = KotlinAstStrategy;
                    strategy.analyze(path, &classifier).await.ok()
                } else {
                    None
                }
            }
            #[cfg(not(feature = "kotlin-ast"))]
            None
        }
        _ => None,
    }
}

pub fn format_context_as_markdown(context: &ProjectContext) -> String {
    let mut output = String::new();

    format_header(&mut output, context);
    format_summary(&mut output, &context.summary);
    format_dependencies(&mut output, &context.summary.dependencies);
    format_files(&mut output, &context.files);
    format_footer(&mut output);

    output
}

fn format_header(output: &mut String, context: &ProjectContext) {
    output.push_str(&format!(
        "# Project Context: {} Project\n\n",
        context.project_type
    ));
    output.push_str(&format!(
        "Generated: {}\n\n",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    ));
}

fn format_summary(output: &mut String, summary: &ProjectSummary) {
    output.push_str("## Summary\n\n");
    output.push_str(&format!("- Files analyzed: {}\n", summary.total_files));
    output.push_str(&format!("- Functions: {}\n", summary.total_functions));
    output.push_str(&format!("- Structs: {}\n", summary.total_structs));
    output.push_str(&format!("- Enums: {}\n", summary.total_enums));
    output.push_str(&format!("- Traits: {}\n", summary.total_traits));
    output.push_str(&format!("- Implementations: {}\n", summary.total_impls));
}

fn format_dependencies(output: &mut String, dependencies: &[String]) {
    if !dependencies.is_empty() {
        output.push_str("\n## Dependencies\n\n");
        for dep in dependencies {
            output.push_str(&format!("- {dep}\n"));
        }
    }
}

fn format_files(output: &mut String, files: &[FileContext]) {
    output.push_str("\n## Files\n\n");

    for file in files {
        output.push_str(&format!("### {}\n\n", file.path));

        let grouped_items = group_items_by_type(&file.items);
        format_item_groups(output, &grouped_items);
    }
}

struct GroupedItems<'a> {
    functions: Vec<&'a AstItem>,
    structs: Vec<&'a AstItem>,
    enums: Vec<&'a AstItem>,
    traits: Vec<&'a AstItem>,
    impls: Vec<&'a AstItem>,
    modules: Vec<&'a AstItem>,
}

fn group_items_by_type(items: &[AstItem]) -> GroupedItems {
    let mut grouped = GroupedItems {
        functions: Vec::new(),
        structs: Vec::new(),
        enums: Vec::new(),
        traits: Vec::new(),
        impls: Vec::new(),
        modules: Vec::new(),
    };

    for item in items {
        match item {
            AstItem::Function { .. } => grouped.functions.push(item),
            AstItem::Struct { .. } => grouped.structs.push(item),
            AstItem::Enum { .. } => grouped.enums.push(item),
            AstItem::Trait { .. } => grouped.traits.push(item),
            AstItem::Impl { .. } => grouped.impls.push(item),
            AstItem::Module { .. } => grouped.modules.push(item),
            _ => {}
        }
    }

    grouped
}

fn format_item_groups(output: &mut String, groups: &GroupedItems) {
    format_item_group(output, "Modules", &groups.modules, format_module_item);
    format_item_group(output, "Structs", &groups.structs, format_struct_item);
    format_item_group(output, "Enums", &groups.enums, format_enum_item);
    format_item_group(output, "Traits", &groups.traits, format_trait_item);
    format_item_group(output, "Functions", &groups.functions, format_function_item);
    format_item_group(output, "Implementations", &groups.impls, format_impl_item);
}

fn format_item_group<F>(output: &mut String, title: &str, items: &[&AstItem], formatter: F)
where
    F: Fn(&AstItem) -> String,
{
    if !items.is_empty() {
        output.push_str(&format!("**{title}:**\n"));
        for item in items {
            output.push_str(&format!("{}\n", formatter(item)));
        }
        output.push('\n');
    }
}

fn format_module_item(item: &AstItem) -> String {
    if let AstItem::Module {
        name,
        visibility,
        line,
    } = item
    {
        format!("- `{visibility} mod {name}` (line {line})")
    } else {
        String::new()
    }
}

fn format_struct_item(item: &AstItem) -> String {
    if let AstItem::Struct {
        name,
        visibility,
        fields_count,
        derives,
        line,
    } = item
    {
        let mut result = format!("- `{visibility} struct {name}` ({fields_count} fields)");
        if !derives.is_empty() {
            result.push_str(&format!(" [derives: {}]", derives.join(", ")));
        }
        result.push_str(&format!(" (line {line})"));
        result
    } else {
        String::new()
    }
}

fn format_enum_item(item: &AstItem) -> String {
    if let AstItem::Enum {
        name,
        visibility,
        variants_count,
        line,
    } = item
    {
        format!("- `{visibility} enum {name}` ({variants_count} variants) (line {line})")
    } else {
        String::new()
    }
}

fn format_trait_item(item: &AstItem) -> String {
    if let AstItem::Trait {
        name,
        visibility,
        line,
    } = item
    {
        format!("- `{visibility} trait {name}` (line {line})")
    } else {
        String::new()
    }
}

fn format_function_item(item: &AstItem) -> String {
    if let AstItem::Function {
        name,
        visibility,
        is_async,
        line,
    } = item
    {
        format!(
            "- `{} {}fn {}` (line {})",
            visibility,
            if *is_async { "async " } else { "" },
            name,
            line
        )
    } else {
        String::new()
    }
}

fn format_impl_item(item: &AstItem) -> String {
    if let AstItem::Impl {
        type_name,
        trait_name,
        line,
    } = item
    {
        match trait_name {
            Some(trait_name) => {
                format!("- `impl {trait_name} for {type_name}` (line {line})")
            }
            None => format!("- `impl {type_name}` (line {line})"),
        }
    } else {
        String::new()
    }
}

fn format_footer(output: &mut String) {
    output.push_str("---\n");
    output.push_str("Generated by paiml-mcp-agent-toolkit\n");
}

/// Format a comprehensive DeepContext as markdown with quality metrics
pub fn format_deep_context_as_markdown(context: &DeepContext) -> String {
    let mut output = String::new();

    // Header
    output.push_str(&format!(
        "# Deep Project Context\n\nGenerated: {}\nTool Version: {}\n\n",
        context
            .metadata
            .generated_at
            .format("%Y-%m-%d %H:%M:%S UTC"),
        context.metadata.tool_version
    ));

    // Quality Scorecard
    format_quality_scorecard(&mut output, &context.quality_scorecard);

    // Project Summary
    format_project_summary(&mut output, context);

    // Analysis Results
    format_analysis_results(&mut output, &context.analyses);

    // AST Summary
    format_ast_summary(&mut output, &context.analyses.ast_contexts);

    output
}

fn format_quality_scorecard(
    output: &mut String,
    scorecard: &crate::services::deep_context::QualityScorecard,
) {
    output.push_str("## Quality Scorecard\n\n");
    output.push_str(&format!(
        "- **Overall Health**: {:.1}%\n",
        scorecard.overall_health
    ));
    output.push_str(&format!(
        "- **Complexity Score**: {:.1}%\n",
        scorecard.complexity_score
    ));
    output.push_str(&format!(
        "- **Maintainability Index**: {:.1}%\n",
        scorecard.maintainability_index
    ));
    output.push_str(&format!(
        "- **Modularity Score**: {:.1}%\n",
        scorecard.modularity_score
    ));
    if let Some(coverage) = scorecard.test_coverage {
        output.push_str(&format!("- **Test Coverage**: {coverage:.1}%\n"));
    }
    output.push_str(&format!(
        "- **Refactoring Estimate**: {:.1} hours\n\n",
        scorecard.technical_debt_hours
    ));
}

fn format_project_summary(output: &mut String, context: &DeepContext) {
    output.push_str("## Project Summary\n\n");
    output.push_str(&format!(
        "- **Total Files**: {}\n",
        context.file_tree.total_files
    ));
    output.push_str(&format!(
        "- **Total Size**: {} bytes\n",
        context.file_tree.total_size_bytes
    ));
    output.push_str(&format!(
        "- **AST Contexts**: {}\n",
        context.analyses.ast_contexts.len()
    ));

    // Count various AST items
    let (functions, structs, enums, traits, impls) =
        count_ast_items(&context.analyses.ast_contexts);
    output.push_str(&format!("- **Functions**: {functions}\n"));
    output.push_str(&format!("- **Structs**: {structs}\n"));
    output.push_str(&format!("- **Enums**: {enums}\n"));
    output.push_str(&format!("- **Traits**: {traits}\n"));
    output.push_str(&format!("- **Implementations**: {impls}\n\n"));
}

fn format_analysis_results(
    output: &mut String,
    analyses: &crate::services::deep_context::AnalysisResults,
) {
    output.push_str("## Analysis Results\n\n");

    // Complexity Analysis
    if let Some(ref complexity) = analyses.complexity_report {
        output.push_str("### Complexity Metrics\n\n");
        output.push_str(&format!(
            "- **Total Files Analyzed**: {}\n",
            complexity.files.len()
        ));
        output.push_str(&format!(
            "- **Median Cyclomatic Complexity**: {:.1}\n",
            complexity.summary.median_cyclomatic
        ));
        output.push_str(&format!(
            "- **Max Cyclomatic Complexity**: {}\n",
            complexity.summary.max_cyclomatic
        ));
        output.push_str(&format!(
            "- **Median Cognitive Complexity**: {:.1}\n",
            complexity.summary.median_cognitive
        ));
        output.push_str(&format!(
            "- **Max Cognitive Complexity**: {}\n",
            complexity.summary.max_cognitive
        ));
        output.push_str(&format!(
            "- **Refactoring Hours**: {:.1}\n\n",
            complexity.summary.technical_debt_hours
        ));
    }

    // Churn Analysis
    if let Some(ref churn) = analyses.churn_analysis {
        output.push_str("### Code Churn Analysis\n\n");
        output.push_str(&format!(
            "- **Analysis Period**: {} days\n",
            churn.period_days
        ));
        output.push_str(&format!(
            "- **Total Files Changed**: {}\n",
            churn.summary.total_files_changed
        ));
        output.push_str(&format!(
            "- **Total Commits**: {}\n",
            churn.summary.total_commits
        ));
        output.push_str(&format!(
            "- **Hotspot Files**: {}\n",
            churn.summary.hotspot_files.len()
        ));
        if !churn.summary.hotspot_files.is_empty() {
            output.push_str("- **Top Hotspots**:\n");
            for (i, hotspot) in churn.summary.hotspot_files.iter().take(5).enumerate() {
                output.push_str(&format!("  {}. {}\n", i + 1, hotspot.display()));
            }
        }
        output.push('\n');
    }

    // Dependency Graph
    if let Some(ref dag) = analyses.dependency_graph {
        output.push_str("### Dependency Graph Statistics\n\n");
        output.push_str(&format!("- **Total Nodes**: {}\n", dag.nodes.len()));
        output.push_str(&format!("- **Total Edges**: {}\n", dag.edges.len()));
        output.push_str("- **Graph Analysis**: Dependency relationships analyzed\n\n");
    }

    // Dead Code Analysis
    if let Some(ref dead_code) = analyses.dead_code_results {
        output.push_str("### Dead Code Analysis\n\n");
        output.push_str(&format!(
            "- **Total Files Analyzed**: {}\n",
            dead_code.summary.total_files_analyzed
        ));
        output.push_str(&format!(
            "- **Dead Functions Found**: {}\n",
            dead_code.summary.dead_functions
        ));
        output.push_str(&format!(
            "- **Dead Classes Found**: {}\n",
            dead_code.summary.dead_classes
        ));
        output.push_str(&format!(
            "- **Dead Lines**: {}\n\n",
            dead_code.summary.total_dead_lines
        ));
    }

    // SATD Analysis
    if let Some(ref satd) = analyses.satd_results {
        output.push_str("### Self-Admitted Debt Analysis\n\n");
        output.push_str(&format!("- **Total SATD Items**: {}\n", satd.items.len()));
        output.push_str("- **Categories**: Various debt types detected\n\n");
    }
}

fn format_ast_summary(
    output: &mut String,
    ast_contexts: &[crate::services::deep_context::EnhancedFileContext],
) {
    if ast_contexts.is_empty() {
        return;
    }

    output.push_str("## AST Analysis\n\n");

    for enhanced_context in ast_contexts.iter().take(20) {
        // Limit to top 20 files
        let file_context = &enhanced_context.base;
        output.push_str(&format!("### {}\n\n", file_context.path));

        let grouped_items = group_items_by_type(&file_context.items);
        format_item_groups(output, &grouped_items);
    }
}

fn count_ast_items(
    ast_contexts: &[crate::services::deep_context::EnhancedFileContext],
) -> (usize, usize, usize, usize, usize) {
    let mut functions = 0;
    let mut structs = 0;
    let mut enums = 0;
    let mut traits = 0;
    let mut impls = 0;

    for enhanced_context in ast_contexts {
        for item in &enhanced_context.base.items {
            match item {
                AstItem::Function { .. } => functions += 1,
                AstItem::Struct { .. } => structs += 1,
                AstItem::Enum { .. } => enums += 1,
                AstItem::Trait { .. } => traits += 1,
                AstItem::Impl { .. } => impls += 1,
                _ => {}
            }
        }
    }

    (functions, structs, enums, traits, impls)
}

#[cfg(test)]
mod tests {
    // use super::*; // Unused in simple tests

    #[test]
    fn test_context_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}
