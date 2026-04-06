#![cfg_attr(coverage_nightly, coverage(off))]
//! Symbol table for location resolution and qualified name mapping
//!
//! This module provides efficient symbol resolution for proof annotation location mapping.

use crate::models::unified_ast::{BytePos, Location, QualifiedName, RelativeLocation, Span};
use dashmap::DashMap;
use std::path::Path;
use std::sync::Arc;
use tracing::debug;

/// Symbol table for mapping qualified names to locations
#[derive(Debug)]
pub struct SymbolTable {
    /// Maps qualified names to canonical locations
    symbols: DashMap<QualifiedName, Location>,
    /// Reverse index for span-to-symbol lookup (simplified approach)
    /// Maps file path to sorted list of (`start_pos`, `qualified_name`) for binary search
    span_index: DashMap<std::path::PathBuf, Vec<(BytePos, QualifiedName)>>,
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for constructing symbol tables from AST analysis
pub struct SymbolTableBuilder {
    table: Arc<SymbolTable>,
}

impl SymbolTableBuilder {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            table: Arc::new(SymbolTable::new()),
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn add_symbol(&self, qualified_name: QualifiedName, location: Location) {
        self.table.insert(qualified_name, location);
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn build(self) -> Arc<SymbolTable> {
        self.table
    }
}

impl Default for SymbolTableBuilder {
    fn default() -> Self {
        Self::new()
    }
}

include!("symbol_table_methods.rs");
include!("symbol_table_tests.rs");
