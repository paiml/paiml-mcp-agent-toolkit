//! AI-ready context generation for code repositories.
//!
//! This module provides context extraction and generation capabilities that create
//! structured representations of codebases suitable for AI/LLM consumption. It
//! analyzes project structure, extracts key code elements, and generates summaries.
//!
//! # Features
//!
//! - Multi-language support (Rust, TypeScript, Python, C/C++, Kotlin)
//! - AST-based analysis for accurate extraction
//! - Complexity metrics integration
//! - Caching for performance optimization
//! - Gitignore-aware file traversal
//!
//! # Example
//!
//! ```no_run
//! use pmat::services::context::analyze_project;
//! use std::path::Path;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Generate context for a Rust project
//! let context = analyze_project(Path::new("src/"), "rust").await?;
//!
//! println!("Project type: {}", context.project_type);
//! println!("Total files: {}", context.summary.total_files);
//! println!("Total functions: {}", context.summary.total_functions);
//!
//! // Access file-level context
//! for file in &context.files {
//!     println!("File: {} ({} items)", file.path, file.items.len());
//! }
//! # Ok(())
//! # }
//! ```

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
    /// O(1) graph for symbol lookups and PageRank-based importance
    #[serde(skip)]
    pub graph: Option<crate::services::context_graph::ProjectContextGraph>,
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
#[derive(PartialEq)]
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
    /// Import statement for language-specific module imports
    ///
    /// Supports various import patterns across different languages including
    /// Python, JavaScript, TypeScript, and other languages with module systems.
    ///
    /// # Examples
    ///
    /// ## Python Import Patterns
    ///
    /// ```
    /// use pmat::services::context::AstItem;
    ///
    /// // Simple import: import os
    /// let import1 = AstItem::Import {
    ///     module: "os".to_string(),
    ///     items: vec![],
    ///     alias: None,
    ///     line: 1,
    /// };
    /// assert_eq!(import1.display_name(), "os");
    ///
    /// // Import with alias: import numpy as np
    /// let import2 = AstItem::Import {
    ///     module: "numpy".to_string(),
    ///     items: vec![],
    ///     alias: Some("np".to_string()),
    ///     line: 2,
    /// };
    /// assert_eq!(import2.display_name(), "numpy");
    ///
    /// // From import: from typing import List, Dict
    /// let import3 = AstItem::Import {
    ///     module: "typing".to_string(),
    ///     items: vec!["List".to_string(), "Dict".to_string()],
    ///     alias: None,
    ///     line: 3,
    /// };
    /// assert_eq!(import3.display_name(), "typing");
    ///
    /// // Submodule import: import os.path
    /// let import4 = AstItem::Import {
    ///     module: "os.path".to_string(),
    ///     items: vec![],
    ///     alias: None,
    ///     line: 4,
    /// };
    /// assert_eq!(import4.display_name(), "os.path");
    ///
    /// // Wildcard import: from math import *
    /// let import5 = AstItem::Import {
    ///     module: "math".to_string(),
    ///     items: vec!["*".to_string()],
    ///     alias: None,
    ///     line: 5,
    /// };
    /// assert_eq!(import5.display_name(), "math");
    /// ```
    ///
    /// ## JavaScript/TypeScript Import Patterns
    ///
    /// ```
    /// use pmat::services::context::AstItem;
    ///
    /// // ES6 default import: import React from 'react'
    /// let import1 = AstItem::Import {
    ///     module: "react".to_string(),
    ///     items: vec![],
    ///     alias: None,
    ///     line: 1,
    /// };
    /// assert_eq!(import1.display_name(), "react");
    ///
    /// // Named imports: import { useState, useEffect } from 'react'
    /// let import2 = AstItem::Import {
    ///     module: "react".to_string(),
    ///     items: vec!["useState".to_string(), "useEffect".to_string()],
    ///     alias: None,
    ///     line: 2,
    /// };
    /// assert_eq!(import2.display_name(), "react");
    ///
    /// // Scoped package: import { Button } from '@mui/material'
    /// let import3 = AstItem::Import {
    ///     module: "@mui/material".to_string(),
    ///     items: vec!["Button".to_string()],
    ///     alias: None,
    ///     line: 3,
    /// };
    /// assert_eq!(import3.display_name(), "@mui/material");
    ///
    /// // Relative import: import { utils } from './utils'
    /// let import4 = AstItem::Import {
    ///     module: "./utils".to_string(),
    ///     items: vec!["utils".to_string()],
    ///     alias: None,
    ///     line: 4,
    /// };
    /// assert_eq!(import4.display_name(), "./utils");
    ///
    /// // Parent directory import: import Component from '../components/Button'
    /// let import5 = AstItem::Import {
    ///     module: "../components/Button".to_string(),
    ///     items: vec![],
    ///     alias: None,
    ///     line: 5,
    /// };
    /// assert_eq!(import5.display_name(), "../components/Button");
    /// ```
    ///
    /// ## Edge Cases and Special Patterns
    ///
    /// ```
    /// use pmat::services::context::AstItem;
    ///
    /// // Empty module (edge case)
    /// let import1 = AstItem::Import {
    ///     module: "".to_string(),
    ///     items: vec![],
    ///     alias: None,
    ///     line: 1,
    /// };
    /// assert_eq!(import1.display_name(), "");
    ///
    /// // Complex nested module path
    /// let import2 = AstItem::Import {
    ///     module: "matplotlib.pyplot.subplots".to_string(),
    ///     items: vec![],
    ///     alias: Some("plt".to_string()),
    ///     line: 2,
    /// };
    /// assert_eq!(import2.display_name(), "matplotlib.pyplot.subplots");
    ///
    /// // Multiple aliases don't affect display_name
    /// let import3 = AstItem::Import {
    ///     module: "pandas".to_string(),
    ///     items: vec!["DataFrame".to_string(), "Series".to_string()],
    ///     alias: Some("pd".to_string()),
    ///     line: 3,
    /// };
    /// assert_eq!(import3.display_name(), "pandas");
    /// ```
    Import {
        /// What is being imported (module, package, or specific items)
        module: String,
        /// Specific items imported from the module (if any)
        items: Vec<String>,
        /// Alias for the import (if any)
        alias: Option<String>,
        /// Line number
        line: usize,
    },
}

impl AstItem {
    #[must_use]
    pub fn display_name(&self) -> &str {
        match self {
            AstItem::Function { name, .. } => name,
            AstItem::Struct { name, .. } => name,
            AstItem::Enum { name, .. } => name,
            AstItem::Trait { name, .. } => name,
            AstItem::Impl { type_name, .. } => type_name,
            AstItem::Module { name, .. } => name,
            AstItem::Use { path, .. } => path,
            AstItem::Import { module, .. } => module,
        }
    }
}

// RustVisitor implementation split for file health compliance (CB-040)
include!("context_impl/visitor.rs");

// Build functions split for file health compliance (CB-040)
include!("context_impl/build.rs");

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
    // This enables multi-language project analysis for ALL 20+ supported languages
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

        // NOTE: Languages below need analyze_*_file() implementations
        // See server/src/services/languages/go.rs:analyze_go_file() as reference

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
            // Convert Vec<AstItem> to FileContext
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
            // Convert Vec<AstItem> to FileContext
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

        // Ruby files (tree-sitter) - TODO: implement analyze_ruby_file()
        // #[cfg(feature = "ruby-ast")]
        // "rb" => {
        //     use crate::services::languages::ruby;
        //     ruby::analyze_ruby_file(path).await.ok()
        // }

        // Ruby files (ruchy parser - alternative) - TODO: implement analyze_ruby_file()
        // #[cfg(all(feature = "ruchy-ast", not(feature = "ruby-ast")))]
        // "rb" => {
        //     use crate::services::languages::ruchy;
        //     ruchy::analyze_ruby_file(path).await.ok()
        // }

        // Swift files
        #[cfg(feature = "swift-ast")]
        "swift" => {
            use crate::services::deep_context;
            // Convert Vec<AstItem> to FileContext
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

        // Erlang files - TODO: implement analyze_erlang_file()
        // #[cfg(feature = "erlang-ast")]
        // "erl" | "hrl" => {
        //     use crate::services::languages::erlang;
        //     erlang::analyze_erlang_file(path).await.ok()
        // }

        // Elixir files - TODO: implement analyze_elixir_file()
        // #[cfg(feature = "elixir-ast")]
        // "ex" | "exs" => {
        //     use crate::services::languages::elixir;
        //     elixir::analyze_elixir_file(path).await.ok()
        // }

        // Haskell files - TODO: implement analyze_haskell_file()
        // #[cfg(feature = "haskell-ast")]
        // "hs" | "lhs" => {
        //     use crate::services::languages::haskell;
        //     haskell::analyze_haskell_file(path).await.ok()
        // }

        // OCaml files - TODO: implement analyze_ocaml_file()
        // #[cfg(feature = "ocaml-ast")]
        // "ml" | "mli" => {
        //     use crate::services::languages::ocaml;
        //     ocaml::analyze_ocaml_file(path).await.ok()
        // }

        // Shell script files - TODO: implement analyze_shell_file()
        // #[cfg(feature = "shell-ast")]
        // "sh" | "bash" | "zsh" => {
        //     use crate::services::languages::shell;
        //     shell::analyze_shell_file(path).await.ok()
        // }

        // WebAssembly files - TODO: implement analyze_wasm_file()
        // #[cfg(feature = "wasm-ast")]
        // "wat" | "wasm" => {
        //     use crate::services::languages::wasm;
        //     wasm::analyze_wasm_file(path).await.ok()
        // }

        // Unsupported extension
        _ => None,
    }
}

// Formatting functions split for file health compliance (CB-040)
include!("context_impl/formatting.rs");

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod context_tests {
    use super::*;

    // ============================================================================
    // ProjectSummary Tests
    // ============================================================================

    #[test]
    fn test_project_summary_creation() {
        let summary = ProjectSummary {
            total_files: 10,
            total_functions: 50,
            total_structs: 5,
            total_enums: 3,
            total_traits: 2,
            total_impls: 8,
            dependencies: vec!["serde".to_string(), "tokio".to_string()],
        };

        assert_eq!(summary.total_files, 10);
        assert_eq!(summary.total_functions, 50);
        assert_eq!(summary.total_structs, 5);
        assert_eq!(summary.total_enums, 3);
        assert_eq!(summary.total_traits, 2);
        assert_eq!(summary.total_impls, 8);
        assert_eq!(summary.dependencies.len(), 2);
    }

    #[test]
    fn test_project_summary_serialization() {
        let summary = ProjectSummary {
            total_files: 5,
            total_functions: 20,
            total_structs: 3,
            total_enums: 1,
            total_traits: 0,
            total_impls: 4,
            dependencies: vec!["anyhow".to_string()],
        };

        let json = serde_json::to_string(&summary).unwrap();
        let deserialized: ProjectSummary = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.total_files, summary.total_files);
        assert_eq!(deserialized.total_functions, summary.total_functions);
        assert_eq!(deserialized.dependencies, summary.dependencies);
    }

    #[test]
    fn test_project_summary_clone() {
        let summary = ProjectSummary {
            total_files: 1,
            total_functions: 2,
            total_structs: 3,
            total_enums: 4,
            total_traits: 5,
            total_impls: 6,
            dependencies: vec!["dep1".to_string()],
        };

        let cloned = summary.clone();
        assert_eq!(cloned.total_files, summary.total_files);
        assert_eq!(cloned.dependencies, summary.dependencies);
    }

    // ============================================================================
    // FileContext Tests
    // ============================================================================

    #[test]
    fn test_file_context_creation() {
        let file_ctx = FileContext {
            path: "src/main.rs".to_string(),
            language: "rust".to_string(),
            items: vec![],
            complexity_metrics: None,
        };

        assert_eq!(file_ctx.path, "src/main.rs");
        assert_eq!(file_ctx.language, "rust");
        assert!(file_ctx.items.is_empty());
        assert!(file_ctx.complexity_metrics.is_none());
    }

    #[test]
    fn test_file_context_with_items() {
        let file_ctx = FileContext {
            path: "src/lib.rs".to_string(),
            language: "rust".to_string(),
            items: vec![
                AstItem::Function {
                    name: "main".to_string(),
                    visibility: "pub".to_string(),
                    is_async: false,
                    line: 1,
                },
                AstItem::Struct {
                    name: "Config".to_string(),
                    visibility: "pub".to_string(),
                    fields_count: 3,
                    derives: vec!["Debug".to_string()],
                    line: 10,
                },
            ],
            complexity_metrics: None,
        };

        assert_eq!(file_ctx.items.len(), 2);
    }

    #[test]
    fn test_file_context_serialization() {
        let file_ctx = FileContext {
            path: "test.rs".to_string(),
            language: "rust".to_string(),
            items: vec![AstItem::Enum {
                name: "Status".to_string(),
                visibility: "pub".to_string(),
                variants_count: 3,
                line: 5,
            }],
            complexity_metrics: None,
        };

        let json = serde_json::to_string(&file_ctx).unwrap();
        assert!(json.contains("test.rs"));
        assert!(json.contains("Status"));
    }

    // ============================================================================
    // AstItem Tests
    // ============================================================================

    #[test]
    fn test_ast_item_function() {
        let item = AstItem::Function {
            name: "process_data".to_string(),
            visibility: "pub".to_string(),
            is_async: true,
            line: 42,
        };

        assert_eq!(item.display_name(), "process_data");

        if let AstItem::Function {
            name,
            visibility,
            is_async,
            line,
        } = item
        {
            assert_eq!(name, "process_data");
            assert_eq!(visibility, "pub");
            assert!(is_async);
            assert_eq!(line, 42);
        } else {
            panic!("Expected Function variant");
        }
    }

    #[test]
    fn test_ast_item_struct() {
        let item = AstItem::Struct {
            name: "User".to_string(),
            visibility: "pub".to_string(),
            fields_count: 5,
            derives: vec!["Clone".to_string(), "Debug".to_string()],
            line: 10,
        };

        assert_eq!(item.display_name(), "User");

        if let AstItem::Struct {
            fields_count,
            derives,
            ..
        } = &item
        {
            assert_eq!(*fields_count, 5);
            assert_eq!(derives.len(), 2);
        }
    }

    #[test]
    fn test_ast_item_enum() {
        let item = AstItem::Enum {
            name: "Color".to_string(),
            visibility: "pub(crate)".to_string(),
            variants_count: 4,
            line: 20,
        };

        assert_eq!(item.display_name(), "Color");

        if let AstItem::Enum { variants_count, .. } = &item {
            assert_eq!(*variants_count, 4);
        }
    }

    #[test]
    fn test_ast_item_trait() {
        let item = AstItem::Trait {
            name: "Drawable".to_string(),
            visibility: "pub".to_string(),
            line: 30,
        };

        assert_eq!(item.display_name(), "Drawable");
    }

    #[test]
    fn test_ast_item_impl() {
        let item = AstItem::Impl {
            type_name: "User".to_string(),
            trait_name: Some("Clone".to_string()),
            line: 50,
        };

        assert_eq!(item.display_name(), "User");

        if let AstItem::Impl { trait_name, .. } = &item {
            assert_eq!(trait_name.as_ref().unwrap(), "Clone");
        }
    }

    #[test]
    fn test_ast_item_impl_no_trait() {
        let item = AstItem::Impl {
            type_name: "Config".to_string(),
            trait_name: None,
            line: 60,
        };

        assert_eq!(item.display_name(), "Config");

        if let AstItem::Impl { trait_name, .. } = &item {
            assert!(trait_name.is_none());
        }
    }

    #[test]
    fn test_ast_item_module() {
        let item = AstItem::Module {
            name: "utils".to_string(),
            visibility: "pub".to_string(),
            line: 1,
        };

        assert_eq!(item.display_name(), "utils");
    }

    #[test]
    fn test_ast_item_use() {
        let item = AstItem::Use {
            path: "std::collections::HashMap".to_string(),
            line: 3,
        };

        assert_eq!(item.display_name(), "std::collections::HashMap");
    }

    #[test]
    fn test_ast_item_import() {
        let item = AstItem::Import {
            module: "react".to_string(),
            items: vec!["useState".to_string(), "useEffect".to_string()],
            alias: None,
            line: 1,
        };

        assert_eq!(item.display_name(), "react");

        if let AstItem::Import { items, alias, .. } = &item {
            assert_eq!(items.len(), 2);
            assert!(alias.is_none());
        }
    }

    #[test]
    fn test_ast_item_import_with_alias() {
        let item = AstItem::Import {
            module: "numpy".to_string(),
            items: vec![],
            alias: Some("np".to_string()),
            line: 2,
        };

        assert_eq!(item.display_name(), "numpy");

        if let AstItem::Import { alias, .. } = &item {
            assert_eq!(alias.as_ref().unwrap(), "np");
        }
    }

    #[test]
    fn test_ast_item_equality() {
        let item1 = AstItem::Function {
            name: "foo".to_string(),
            visibility: "pub".to_string(),
            is_async: false,
            line: 1,
        };
        let item2 = AstItem::Function {
            name: "foo".to_string(),
            visibility: "pub".to_string(),
            is_async: false,
            line: 1,
        };
        let item3 = AstItem::Function {
            name: "bar".to_string(),
            visibility: "pub".to_string(),
            is_async: false,
            line: 1,
        };

        assert_eq!(item1, item2);
        assert_ne!(item1, item3);
    }

    #[test]
    fn test_ast_item_serialization() {
        let item = AstItem::Struct {
            name: "Test".to_string(),
            visibility: "pub".to_string(),
            fields_count: 2,
            derives: vec!["Debug".to_string()],
            line: 5,
        };

        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("Test"));
        assert!(json.contains("Struct"));

        let deserialized: AstItem = serde_json::from_str(&json).unwrap();
        assert_eq!(item, deserialized);
    }

    // ============================================================================
    // ProjectContext Tests
    // ============================================================================

    #[test]
    fn test_project_context_creation() {
        let ctx = ProjectContext {
            project_type: "rust".to_string(),
            files: vec![],
            summary: ProjectSummary {
                total_files: 0,
                total_functions: 0,
                total_structs: 0,
                total_enums: 0,
                total_traits: 0,
                total_impls: 0,
                dependencies: vec![],
            },
            graph: None,
        };

        assert_eq!(ctx.project_type, "rust");
        assert!(ctx.files.is_empty());
        assert!(ctx.graph.is_none());
    }

    #[test]
    fn test_project_context_with_files() {
        let ctx = ProjectContext {
            project_type: "rust".to_string(),
            files: vec![
                FileContext {
                    path: "src/main.rs".to_string(),
                    language: "rust".to_string(),
                    items: vec![AstItem::Function {
                        name: "main".to_string(),
                        visibility: "pub".to_string(),
                        is_async: false,
                        line: 1,
                    }],
                    complexity_metrics: None,
                },
                FileContext {
                    path: "src/lib.rs".to_string(),
                    language: "rust".to_string(),
                    items: vec![],
                    complexity_metrics: None,
                },
            ],
            summary: ProjectSummary {
                total_files: 2,
                total_functions: 1,
                total_structs: 0,
                total_enums: 0,
                total_traits: 0,
                total_impls: 0,
                dependencies: vec![],
            },
            graph: None,
        };

        assert_eq!(ctx.files.len(), 2);
        assert_eq!(ctx.summary.total_files, 2);
        assert_eq!(ctx.summary.total_functions, 1);
    }

    #[test]
    fn test_project_context_serialization() {
        let ctx = ProjectContext {
            project_type: "python".to_string(),
            files: vec![FileContext {
                path: "main.py".to_string(),
                language: "python".to_string(),
                items: vec![],
                complexity_metrics: None,
            }],
            summary: ProjectSummary {
                total_files: 1,
                total_functions: 0,
                total_structs: 0,
                total_enums: 0,
                total_traits: 0,
                total_impls: 0,
                dependencies: vec!["requests".to_string()],
            },
            graph: None,
        };

        let json = serde_json::to_string(&ctx).unwrap();
        assert!(json.contains("python"));
        assert!(json.contains("main.py"));
        assert!(json.contains("requests"));

        let deserialized: ProjectContext = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.project_type, ctx.project_type);
        assert_eq!(deserialized.files.len(), ctx.files.len());
        // graph is skipped in serde
        assert!(deserialized.graph.is_none());
    }

    #[test]
    fn test_project_context_clone() {
        let ctx = ProjectContext {
            project_type: "typescript".to_string(),
            files: vec![],
            summary: ProjectSummary {
                total_files: 0,
                total_functions: 0,
                total_structs: 0,
                total_enums: 0,
                total_traits: 0,
                total_impls: 0,
                dependencies: vec![],
            },
            graph: None,
        };

        let cloned = ctx.clone();
        assert_eq!(cloned.project_type, ctx.project_type);
    }

}
