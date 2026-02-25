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

// Rust-specific file analysis (analyze_rust_file, analyze_rust_file_with_cache,
// analyze_rust_file_with_persistent_cache)
include!("analysis_rust_files.rs");

// Project summary, context graph, item counting, and gitignore configuration
include!("analysis_summary.rs");

// Dependency reading for Rust, Deno, and Python projects
include!("analysis_dependencies.rs");

// File scanning (scan_rust_files_only, scan_and_analyze_files,
// scan_and_analyze_files_persistent)
include!("analysis_scanning.rs");

// Toolchain dispatch and project analysis entry points (analyze_project,
// analyze_project_with_cache, analyze_file_by_toolchain, etc.)
include!("analysis_toolchain.rs");
