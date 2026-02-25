#![cfg_attr(coverage_nightly, coverage(off))]
//! WebAssembly and `AssemblyScript` analysis handlers
//!
//! This module contains handlers for WebAssembly binary/text format and
//! `AssemblyScript` source code analysis.
//!
//! ## Submodule layout (include! pattern)
//!
//! - `wasm_handlers_assemblyscript.rs` - AssemblyScript parsing and analysis
//! - `wasm_handlers_execution.rs` - WebAssembly binary/text analysis
//! - `wasm_handlers_collection.rs` - File discovery and collection helpers
//! - `wasm_handlers_formatting.rs` - Output formatting (JSON and Markdown)
//! - `wasm_handlers_tests.rs` - Unit and property tests

use crate::cli::ComplexityOutputFormat;
use crate::models::unified_ast::AstDag;
use crate::services::wasm::{
    AssemblyScriptParser, WasmBinaryAnalyzer, WasmComplexity, WasmComplexityAnalyzer,
    WasmLanguageDetector, WasmMetrics, WasmSecurityValidator, WatParser,
};
use anyhow::Result;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

// --- AssemblyScript analysis (handle_analyze_assemblyscript + helpers) ---
include!("wasm_handlers_assemblyscript.rs");

// --- WebAssembly binary/text analysis (handle_analyze_webassembly + helpers) ---
include!("wasm_handlers_execution.rs");

// --- File discovery and collection helpers ---
include!("wasm_handlers_collection.rs");

// --- Output formatting (JSON and Markdown) ---
include!("wasm_handlers_formatting.rs");

// --- Unit and property tests ---
include!("wasm_handlers_tests.rs");
