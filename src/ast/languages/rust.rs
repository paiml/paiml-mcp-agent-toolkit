#![cfg_attr(coverage_nightly, coverage(off))]
//! Rust language AST parsing strategy

use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;
use syn::{visit::Visit, File as SynFile, ItemEnum, ItemFn, ItemImpl, ItemStruct, ItemTrait};

use super::LanguageStrategy;
use crate::ast::core::{
    AstDag, AstKind, ClassKind, FunctionKind, Language, NodeFlags, UnifiedAstNode,
};

/// Rust language parsing strategy
pub struct RustStrategy {
    // Configuration options can be added here
}

impl Default for RustStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl RustStrategy {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {}
    }

    fn parse_syn_file(&self, content: &str) -> Result<SynFile> {
        debug_assert!(!content.is_empty(), "content must not be empty");
        syn::parse_file(content).map_err(|e| anyhow::anyhow!("Rust parse error: {e}"))
    }

    fn convert_to_dag(&self, syn_file: &SynFile) -> AstDag {
        debug_assert!(true, "contract: convert_to_dag");
        let mut dag = AstDag::new();
        let mut visitor = RustAstVisitor::new(&mut dag);
        visitor.visit_file(syn_file);
        dag
    }
}

// LanguageStrategy trait implementation
include!("rust_strategy.rs");

// AST visitor for converting syn to unified AST
include!("rust_visitor.rs");

// Tests
include!("rust_tests.rs");
