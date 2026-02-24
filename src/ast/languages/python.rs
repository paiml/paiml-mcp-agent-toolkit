#![cfg_attr(coverage_nightly, coverage(off))]
//! Python language AST parsing strategy
//!
//! Split into include files for maintainability:
//! - `python_parsing.rs` - Tree-sitter parsing and DAG conversion
//! - `python_strategy.rs` - LanguageStrategy trait implementation
//! - `python_visitor.rs` - Tree-sitter AST visitor
//! - `python_tests.rs` - All tests

use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;

// Modern tree-sitter-python support (preferred)
#[cfg(feature = "python-ast")]
use tree_sitter::{Parser as TsParser, Tree};

use super::LanguageStrategy;
use crate::ast::core::{AstDag, AstKind, Language, NodeFlags, UnifiedAstNode};

#[cfg(feature = "python-ast")]
use crate::ast::core::{ClassKind, FunctionKind, ImportKind, StmtKind};

/// Python language parsing strategy
pub struct PythonStrategy {
    // Configuration options can be added here
}

impl Default for PythonStrategy {
    fn default() -> Self {
        Self::new()
    }
}

// Tree-sitter parsing and DAG conversion methods
include!("python_parsing.rs");

// LanguageStrategy trait implementation
include!("python_strategy.rs");

// Tree-sitter AST visitor
include!("python_visitor.rs");

// Tests
include!("python_tests.rs");
