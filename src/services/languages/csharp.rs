#![cfg_attr(coverage_nightly, coverage(off))]
//! C# Language Support for PMAT
//!
//! This module provides C#-specific analysis capabilities using tree-sitter-c-sharp parser
//! for AST extraction and complexity analysis aligned with C# best practices.
//!
//! ## Module Structure
//!
//! - `csharp_visitor.rs` - CSharpAstVisitor impl (AST extraction methods)
//! - `csharp_complexity.rs` - CSharpComplexityAnalyzer struct and impl
//! - `csharp_tests.rs` - Unit tests
//! - `csharp_property_tests.rs` - Property-based tests (proptest)

#[cfg(feature = "csharp-ast")]
use crate::services::context::AstItem;
#[cfg(feature = "csharp-ast")]
use std::path::{Path, PathBuf};

/// C# AST visitor that extracts C#-specific AST information
#[cfg(feature = "csharp-ast")]
pub struct CSharpAstVisitor {
    items: Vec<AstItem>,
    _file_path: PathBuf,
    namespace_name: String,
    class_count: usize,
}

// CSharpAstVisitor implementation (AST extraction methods)
include!("csharp_visitor.rs");

// CSharpComplexityAnalyzer struct and implementation
include!("csharp_complexity.rs");

// Unit tests
include!("csharp_tests.rs");

// Property-based tests
include!("csharp_property_tests.rs");
