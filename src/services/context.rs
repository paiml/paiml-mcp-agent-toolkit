#![cfg_attr(coverage_nightly, coverage(off))]
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

// Type definitions: ProjectContext, ProjectSummary, FileContext, AstItem
include!("context_impl/types.rs");

// RustVisitor implementation split for file health compliance (CB-040)
include!("context_impl/visitor.rs");

// Build functions split for file health compliance (CB-040)
include!("context_impl/build.rs");

// Dependency reading and item counting utilities
include!("context_impl/dependencies.rs");

// Persistent cache analysis: analyze_project_with_persistent_cache and helpers
include!("context_impl/persistent_analysis.rs");

// Formatting functions split for file health compliance (CB-040)
include!("context_impl/formatting.rs");

// Unit tests
include!("context_impl/tests.rs");
