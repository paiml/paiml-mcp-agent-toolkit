#![cfg_attr(coverage_nightly, coverage(off))]
//! Project and file analysis functions.
//!
//! This module provides the core analysis functions for scanning and analyzing
//! source files across multiple languages, with caching support.

use super::rust_visitor::RustVisitor;
use super::types::{AstItem, FileContext, ProjectContext, ProjectSummary};
use crate::models::error::TemplateError;
#[cfg(feature = "python-ast")]
use crate::services::ast_python;
#[cfg(feature = "typescript-ast")]
use crate::services::ast_typescript;
use crate::services::cache::{
    manager::SessionCacheManager, persistent_manager::PersistentCacheManager,
};
use futures::future::join_all;
use ignore::gitignore::GitignoreBuilder;
use std::path::Path;
use std::sync::Arc;
use syn::visit::Visit;
use walkdir::WalkDir;

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
                    .map_err(|e| anyhow::anyhow!("Failed to read file: {e}"))?;

                let syntax = syn::parse_file(&content)
                    .map_err(|e| anyhow::anyhow!("Failed to parse Rust file: {e}"))?;

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
                    .map_err(|e| anyhow::anyhow!("Failed to read file: {e}"))?;

                let syntax = syn::parse_file(&content)
                    .map_err(|e| anyhow::anyhow!("Failed to parse Rust file: {e}"))?;

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

/// Optimized project analysis for dead code detection - focuses only on source files
pub async fn analyze_project_for_dead_code(
    root_path: &Path,
    toolchain: &str,
) -> Result<ProjectContext, TemplateError> {
    let gitignore = build_gitignore(root_path)?;
    let files = scan_rust_files_only(root_path, toolchain, None, &gitignore).await;
    let summary = build_project_summary(&files, root_path, toolchain).await;

    Ok(ProjectContext {
        project_type: toolchain.to_string(),
        files,
        summary,
        graph: None,
    })
}

pub async fn analyze_project_with_cache(
    root_path: &Path,
    toolchain: &str,
    cache_manager: Option<Arc<SessionCacheManager>>,
) -> Result<ProjectContext, TemplateError> {
    let gitignore = build_gitignore(root_path)?;
    let files = scan_and_analyze_files(root_path, toolchain, cache_manager, &gitignore).await;
    let summary = build_project_summary(&files, root_path, toolchain).await;

    // Build O(1) graph for symbol lookups and PageRank
    let graph = build_context_graph(&files).ok();

    Ok(ProjectContext {
        project_type: toolchain.to_string(),
        files,
        summary,
        graph,
    })
}

pub(crate) fn build_gitignore(root_path: &Path) -> Result<ignore::gitignore::Gitignore, TemplateError> {
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

/// Optimized file scanner for dead code - only scans Rust source files
async fn scan_rust_files_only(
    root_path: &Path,
    toolchain: &str,
    cache_manager: Option<Arc<SessionCacheManager>>,
    gitignore: &ignore::gitignore::Gitignore,
) -> Vec<FileContext> {
    const MAX_DEPTH: usize = 5; // Shallower search for performance
    const MAX_FILES: usize = 100; // Even lower limit for fast dead code analysis
    const BATCH_SIZE: usize = 20; // Smaller batches for responsiveness

    // Only scan Rust source files, skip tests and examples
    let paths: Vec<_> = WalkDir::new(root_path)
        .follow_links(false)
        .max_depth(MAX_DEPTH)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| {
            let path = entry.path();
            if path.is_dir() || gitignore.matched(path, false).is_ignore() {
                return false;
            }
            // Only Rust files
            if !path.extension().is_some_and(|ext| ext == "rs") {
                return false;
            }
            // Skip test and example files for dead code analysis
            let path_str = path.to_string_lossy();
            !path_str.contains("/tests/")
                && !path_str.contains("/test/")
                && !path_str.contains("/examples/")
                && !path_str.contains("/benches/")
                && !path_str.contains("_test.rs")
                && !path_str.ends_with("/build.rs")
        })
        .take(MAX_FILES)
        .map(|entry| entry.path().to_path_buf())
        .collect();

    eprintln!(
        "🎯 Dead code analysis: scanning {} Rust source files (max {})",
        paths.len(),
        MAX_FILES
    );

    let mut all_results = Vec::new();
    for chunk in paths.chunks(BATCH_SIZE) {
        let batch_tasks: Vec<_> = chunk
            .iter()
            .map(|path| {
                let path = path.clone();
                let toolchain = toolchain.to_string();
                let cache_manager = cache_manager.clone();
                tokio::spawn(async move {
                    let timeout_duration = tokio::time::Duration::from_secs(2);
                    tokio::time::timeout(timeout_duration, async move {
                        analyze_file_by_toolchain(&path, &toolchain, cache_manager).await
                    })
                    .await
                    .ok()
                    .flatten()
                })
            })
            .collect();

        let batch_results = join_all(batch_tasks).await;
        all_results.extend(
            batch_results
                .into_iter()
                .filter_map(std::result::Result::ok)
                .flatten(),
        );
    }

    all_results
}

async fn scan_and_analyze_files(
    root_path: &Path,
    toolchain: &str,
    cache_manager: Option<Arc<SessionCacheManager>>,
    gitignore: &ignore::gitignore::Gitignore,
) -> Vec<FileContext> {
    // FIXED: Add depth limit and file count limit to prevent hanging
    const MAX_DEPTH: usize = 10; // Prevent infinite recursion
    const MAX_FILES: usize = 10000; // Prevent resource exhaustion
    const BATCH_SIZE: usize = 100; // Process files in batches to avoid overwhelming the system

    // First, collect all file paths to analyze with limits
    let mut file_count = 0;
    let paths: Vec<_> = WalkDir::new(root_path)
        .follow_links(false)
        .max_depth(MAX_DEPTH) // TDD Fix: Limit directory traversal depth
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| {
            let path = entry.path();
            !path.is_dir() && !gitignore.matched(path, false).is_ignore()
        })
        .take(MAX_FILES) // TDD Fix: Limit total files analyzed
        .map(|entry| {
            file_count += 1;
            if file_count % 1000 == 0 {
                eprintln!("📁 Scanning files... ({file_count} so far)");
            }
            entry.path().to_path_buf()
        })
        .collect();

    if file_count > MAX_FILES / 2 {
        eprintln!(
            "⚠️ Large project detected: {file_count} files. Limited to {MAX_FILES} for performance."
        );
    }

    // Process files in controlled batches instead of all at once
    let mut all_results = Vec::new();

    for chunk in paths.chunks(BATCH_SIZE) {
        let batch_tasks: Vec<_> = chunk
            .iter()
            .map(|path| {
                let path = path.clone();
                let toolchain = toolchain.to_string();
                let cache_manager = cache_manager.clone();
                tokio::spawn(async move {
                    // Add timeout for individual file analysis
                    let timeout_duration = tokio::time::Duration::from_secs(5);
                    tokio::time::timeout(timeout_duration, async move {
                        analyze_file_by_toolchain(&path, &toolchain, cache_manager).await
                    })
                    .await
                    .ok()
                    .flatten()
                })
            })
            .collect();

        // Wait for this batch to complete before starting the next
        let batch_results = join_all(batch_tasks).await;
        all_results.extend(
            batch_results
                .into_iter()
                .filter_map(std::result::Result::ok)
                .flatten(),
        );
    }

    all_results
}

async fn analyze_file_by_toolchain(
    path: &Path,
    _toolchain: &str,
    cache_manager: Option<Arc<SessionCacheManager>>,
) -> Option<FileContext> {
    // FIXED: Analyze files by extension, not by toolchain
    // This enables multi-language project analysis for ALL supported languages
    let ext = path.extension().and_then(|s| s.to_str())?;

    match ext {
        // Rust files
        "rs" => analyze_rust_file_with_cache(path, cache_manager).await.ok(),

        // TypeScript/JavaScript files
        #[cfg(feature = "typescript-ast")]
        "ts" | "tsx" => ast_typescript::analyze_typescript_file(path).await.ok(),
        #[cfg(feature = "typescript-ast")]
        "js" | "jsx" | "mjs" | "cjs" => ast_typescript::analyze_javascript_file(path).await.ok(),

        // Python files
        #[cfg(feature = "python-ast")]
        "py" | "pyi" => ast_python::analyze_python_file(path).await.ok(),

        // Go files
        #[cfg(feature = "go-ast")]
        "go" => {
            use crate::services::languages::go;
            go::analyze_go_file(path).await.ok()
        }

        // C files
        #[cfg(feature = "c-ast")]
        "c" | "h" => {
            use crate::services::ast::languages::c;
            c::analyze_c_file(path).await.ok()
        }

        // C++ files
        #[cfg(feature = "cpp-ast")]
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" | "hh" => {
            use crate::services::ast::languages::cpp;
            cpp::analyze_cpp_file(path).await.ok()
        }

        // Java files
        #[cfg(feature = "java-ast")]
        "java" => {
            use crate::services::deep_context;
            match deep_context::analyze_java_file(path).await {
                Ok(items) => Some(FileContext {
                    path: path.display().to_string(),
                    language: "java".to_string(),
                    items,
                    complexity_metrics: None,
                }),
                Err(_) => None,
            }
        }

        // C# files
        #[cfg(feature = "csharp-ast")]
        "cs" => {
            use crate::services::deep_context;
            match deep_context::analyze_csharp_file(path).await {
                Ok(items) => Some(FileContext {
                    path: path.display().to_string(),
                    language: "csharp".to_string(),
                    items,
                    complexity_metrics: None,
                }),
                Err(_) => None,
            }
        }

        // Kotlin files
        #[cfg(feature = "kotlin-ast")]
        "kt" | "kts" => {
            use crate::services::languages::kotlin;
            kotlin::analyze_kotlin_file(path).await.ok()
        }

        // Swift files
        #[cfg(feature = "swift-ast")]
        "swift" => {
            use crate::services::deep_context;
            match deep_context::analyze_swift_file(path).await {
                Ok(items) => Some(FileContext {
                    path: path.display().to_string(),
                    language: "swift".to_string(),
                    items,
                    complexity_metrics: None,
                }),
                Err(_) => None,
            }
        }

        // Unsupported extension
        _ => None,
    }
}

#[allow(dead_code)]
async fn analyze_deno_file(path: &Path) -> Option<FileContext> {
    let ext = path.extension().and_then(|s| s.to_str());
    match ext {
        #[cfg(feature = "typescript-ast")]
        Some("ts" | "tsx") => ast_typescript::analyze_typescript_file(path).await.ok(),
        #[cfg(feature = "typescript-ast")]
        Some("js" | "jsx") => ast_typescript::analyze_javascript_file(path).await.ok(),
        _ => None,
    }
}

pub(crate) async fn build_project_summary(
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

/// Build O(1) context graph for symbol lookups and PageRank
///
/// Extracts all functions/structs/etc from files and builds a trueno-graph CSR
/// for O(1) symbol lookups and PageRank-based importance scoring.
fn build_context_graph(
    files: &[FileContext],
) -> Result<crate::services::context_graph::ProjectContextGraph, TemplateError> {
    use crate::services::context_graph::ProjectContextGraph;

    let mut graph = ProjectContextGraph::new();

    // Phase 1: Add all symbols as nodes
    for file in files {
        for item in &file.items {
            let symbol_name = item.display_name();

            // Skip if already added (duplicates from multiple files)
            if graph.get_item(symbol_name).is_some() {
                continue;
            }

            // Add node to graph
            if let Err(e) = graph.add_item(symbol_name.to_string(), item.clone()) {
                eprintln!("Warning: Failed to add item to graph: {}", e);
            }
        }
    }

    // Phase 2: Edge extraction deferred; graph provides O(1) node lookups

    // Phase 3: Run PageRank to identify "hot" symbols
    if let Err(e) = graph.update_hotness() {
        eprintln!("Warning: Failed to compute PageRank: {}", e);
    }

    Ok(graph)
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
        graph: None,
    })
}

async fn scan_and_analyze_files_persistent(
    root_path: &Path,
    toolchain: &str,
    cache_manager: Option<Arc<PersistentCacheManager>>,
    gitignore: &ignore::gitignore::Gitignore,
) -> Vec<FileContext> {
    // FIXED: Add same depth and file limits as non-persistent version
    const MAX_DEPTH: usize = 10; // Prevent infinite recursion
    const MAX_FILES: usize = 10000; // Prevent resource exhaustion

    let mut files = Vec::new();
    let mut file_count = 0;

    for entry in WalkDir::new(root_path)
        .follow_links(false)
        .max_depth(MAX_DEPTH) // TDD Fix: Limit directory traversal depth
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        let path = entry.path();

        // Skip if gitignored
        if gitignore.matched(path, path.is_dir()).is_ignore() {
            continue;
        }

        // TDD Fix: Limit total files analyzed
        file_count += 1;
        if file_count > MAX_FILES {
            eprintln!("⚠️ Reached file limit of {MAX_FILES}. Stopping analysis.");
            break;
        }

        if file_count % 1000 == 0 {
            eprintln!("📁 Scanning files... ({file_count} so far)");
        }

        // Add timeout for individual file analysis
        let timeout_duration = tokio::time::Duration::from_secs(5);
        let result = tokio::time::timeout(timeout_duration, async {
            analyze_file_by_toolchain_persistent(path, toolchain, cache_manager.clone()).await
        })
        .await;

        if let Ok(Some(file_context)) = result {
            files.push(file_context);
        }
    }

    files
}

async fn analyze_file_by_toolchain_persistent(
    path: &Path,
    _toolchain: &str,
    cache_manager: Option<Arc<PersistentCacheManager>>,
) -> Option<FileContext> {
    // FIXED: Analyze files by extension, not by toolchain
    let ext = path.extension().and_then(|s| s.to_str())?;

    match ext {
        // Rust files
        "rs" => analyze_rust_file_with_persistent_cache(path, cache_manager)
            .await
            .ok(),

        // TypeScript/JavaScript files
        #[cfg(feature = "typescript-ast")]
        "ts" | "tsx" => ast_typescript::analyze_typescript_file(path).await.ok(),
        #[cfg(feature = "typescript-ast")]
        "js" | "jsx" | "mjs" | "cjs" => ast_typescript::analyze_javascript_file(path).await.ok(),

        // Python files
        #[cfg(feature = "python-ast")]
        "py" | "pyi" => ast_python::analyze_python_file(path).await.ok(),

        // Go files
        #[cfg(feature = "go-ast")]
        "go" => {
            use crate::services::languages::go;
            go::analyze_go_file(path).await.ok()
        }

        // C files
        #[cfg(feature = "c-ast")]
        "c" | "h" => {
            use crate::services::ast::languages::c;
            c::analyze_c_file(path).await.ok()
        }

        // C++ files
        #[cfg(feature = "cpp-ast")]
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" | "hh" => {
            use crate::services::ast::languages::cpp;
            cpp::analyze_cpp_file(path).await.ok()
        }

        // Java files
        #[cfg(feature = "java-ast")]
        "java" => {
            use crate::services::deep_context;
            match deep_context::analyze_java_file(path).await {
                Ok(items) => Some(FileContext {
                    path: path.display().to_string(),
                    language: "java".to_string(),
                    items,
                    complexity_metrics: None,
                }),
                Err(_) => None,
            }
        }

        // C# files
        #[cfg(feature = "csharp-ast")]
        "cs" => {
            use crate::services::deep_context;
            match deep_context::analyze_csharp_file(path).await {
                Ok(items) => Some(FileContext {
                    path: path.display().to_string(),
                    language: "csharp".to_string(),
                    items,
                    complexity_metrics: None,
                }),
                Err(_) => None,
            }
        }

        // Kotlin files
        #[cfg(feature = "kotlin-ast")]
        "kt" | "kts" => {
            use crate::services::languages::kotlin;
            kotlin::analyze_kotlin_file(path).await.ok()
        }

        // Swift files
        #[cfg(feature = "swift-ast")]
        "swift" => {
            use crate::services::deep_context;
            match deep_context::analyze_swift_file(path).await {
                Ok(items) => Some(FileContext {
                    path: path.display().to_string(),
                    language: "swift".to_string(),
                    items,
                    complexity_metrics: None,
                }),
                Err(_) => None,
            }
        }

        // Unsupported extension
        _ => None,
    }
}
