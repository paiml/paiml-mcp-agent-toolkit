//! Platform-specific and specialized analysis route handlers
//!
//! Handles: GraphMetrics, NameSimilarity, ProofAnnotations, IncrementalCoverage,
//! SymbolTable, BigO, AssemblyScript, WebAssembly, Wasm, DeepWasm, Mutation,
//! Makefile, Models (MLOps)

use crate::cli::{self, AnalyzeCommands};
use anyhow::Result;

// Route handlers for graph, name similarity, proof, coverage, symbol table,
// big-o, assemblyscript, webassembly, wasm, deep_wasm, mutation, makefile
include!("platform_routes_routing.rs");

// Model analysis route handler and MLOps inventory helpers
include!("platform_routes_models.rs");
