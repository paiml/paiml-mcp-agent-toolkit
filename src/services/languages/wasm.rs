#![cfg_attr(coverage_nightly, coverage(off))]
//! WebAssembly Analysis Support for PMAT
//!
//! This module provides WASM-specific analysis capabilities using wasmparser
//! for bytecode analysis and complexity metrics extraction from WASM modules.
//!
//! ## Module Structure
//! - `wasm_analysis.rs`: WasmModuleAnalyzer and WasmStackAnalyzer implementations
//! - `wasm_validation.rs`: WasmValidator implementation
//! - `wasm_tests.rs`: Unit tests and property-based tests

#[cfg(feature = "wasm-ast")]
use crate::services::context::AstItem;
use std::path::{Path, PathBuf};
use wasmparser::{Parser, Payload};

/// WASM module analyzer that extracts WASM-specific information
pub struct WasmModuleAnalyzer {
    items: Vec<AstItem>,
    _file_path: PathBuf,
    module_name: String,
    function_count: usize,
    _import_count: usize,
    _export_count: usize,
}

/// WASM stack depth analyzer for complexity calculation (complexity ≤10)
pub struct WasmStackAnalyzer {
    max_stack_depth: u32,
    current_depth: u32,
    branch_count: u32,
}

impl Default for WasmStackAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// WASM validation and quality checks (complexity ≤10)
pub struct WasmValidator {
    validation_errors: Vec<String>,
    security_warnings: Vec<String>,
}

impl Default for WasmValidator {
    fn default() -> Self {
        Self::new()
    }
}

// --- Submodule includes ---
include!("wasm_analysis.rs");
include!("wasm_validation.rs");
include!("wasm_tests.rs");
