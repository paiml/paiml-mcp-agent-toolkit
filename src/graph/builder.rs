// Dependency graph builder with language-specific parsers
// Complexity: All functions <= 10
// SATD: Zero tolerance

#![cfg_attr(coverage_nightly, coverage(off))]
use super::symbol_table::{SymbolEntry, SymbolTable};
use super::*;
use anyhow::Result;
use rustc_hash::FxHashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub struct DependencyGraphBuilder {
    graph: DependencyGraph,
    symbol_table: SymbolTable,
    node_map: FxHashMap<PathBuf, NodeId>,
    /// Track processed files for incremental updates
    processed_hashes: FxHashMap<PathBuf, u64>,
}

impl Default for DependencyGraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyGraphBuilder {
    /// Create new builder
    /// Complexity: 1
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        DependencyGraphBuilder {
            graph: DependencyGraph::new(),
            symbol_table: SymbolTable::new(),
            node_map: FxHashMap::default(),
            processed_hashes: FxHashMap::default(),
        }
    }

    /// Build from workspace path
    /// Complexity: 8 (file collection + analysis loop)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn from_workspace(path: &Path) -> Result<Self> {
        let mut builder = Self::new();

        // Collect source files
        let files = builder.collect_source_files(path)?;

        // Phase 1: Build symbol table from all files
        for file_path in &files {
            builder.build_file_symbols(file_path)?;
        }

        // Phase 2: Analyze dependencies
        for file_path in &files {
            let node_id = builder.analyze_file(file_path)?;
            builder.resolve_file_dependencies(node_id, file_path)?;
        }

        Ok(builder)
    }

    /// Build the final graph
    /// Complexity: 1
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn build(self) -> Result<DependencyGraph> {
        Ok(self.graph)
    }

    /// Get symbol table
    /// Complexity: 1
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn symbol_table(&self) -> &SymbolTable {
        &self.symbol_table
    }
}

// File collection: collect_source_files, collect_files_recursive, is_source_file
include!("builder_file_collection.rs");

// Symbol parsing: build_file_symbols, parse_rust/python/typescript_symbols
include!("builder_symbol_parsing.rs");

// Import parsing: resolve_file_dependencies, parse_rust/python/typescript_imports
include!("builder_import_parsing.rs");

// Analysis and helpers: analyze_file, extract_* helpers, path_to_module,
// calculate_hash, estimate_complexity, resolve_import_to_node
include!("builder_analysis.rs");

// Tests
include!("builder_tests.rs");
