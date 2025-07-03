//! WebAssembly parser traits and interfaces
//!
//! This module defines the core traits for WebAssembly parsing and analysis.
//! Follows the existing LanguageParser pattern while adding WASM-specific capabilities.
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::Path;

use super::types::{MemoryAnalysis, WasmComplexity, WasmMetrics};
use crate::models::unified_ast::{AstDag, Language};

/// `Result` of parsing a file
pub struct ParsedAst {
    pub language: Language,
    pub dag: AstDag,
    pub source_file: Option<std::path::PathBuf>,
    pub parse_errors: Vec<String>,
    pub metadata: std::collections::HashMap<String, String>,
}

/// Base language parser `trait`
#[async_trait]
pub trait LanguageParser: Send + Sync {
    /// Parse content into an `AST`
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails
    fn parse_content(&self, content: &str, path: Option<&Path>) -> Result<ParsedAst>;

    /// Get the language this parser supports
    fn language(&self) -> Language;
}

/// Core `trait` for WebAssembly-aware parsers
/// Extends the base LanguageParser with WASM-specific analysis capabilities
#[async_trait]
pub trait WasmAwareParser: LanguageParser {
    /// Extract WebAssembly-specific metrics from the `AST`
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails
    fn extract_wasm_metrics(&self, ast: &AstDag) -> Result<WasmMetrics>;

    /// Analyze memory usage patterns for optimization opportunities
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails
    fn analyze_memory_patterns(&self, ast: &AstDag) -> Result<MemoryAnalysis>;

    /// Calculate WebAssembly computational complexity including gas estimation
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails
    fn calculate_wasm_complexity(&self, ast: &AstDag) -> Result<WasmComplexity>;

    /// Get parser capabilities for feature detection
    fn capabilities(&self) -> WasmAnalysisCapabilities {
        WasmAnalysisCapabilities::default()
    }
}

/// Capabilities supported by a WebAssembly parser
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmAnalysisCapabilities {
    /// Can analyze memory allocation patterns
    pub memory_analysis: bool,

    /// Can estimate gas costs for blockchain deployment
    pub gas_estimation: bool,

    /// Can detect security vulnerabilities
    pub security_analysis: bool,

    /// Can suggest optimization opportunities
    pub optimization_hints: bool,

    /// Supports streaming analysis for large files
    pub streaming_support: bool,

    /// Can analyze SIMD instructions
    pub simd_analysis: bool,

    /// Can analyze multi-memory proposals
    pub multi_memory: bool,

    /// Maximum file size supported (in bytes)
    pub max_file_size: usize,
}

impl Default for WasmAnalysisCapabilities {
    fn default() -> Self {
        Self {
            memory_analysis: true,
            gas_estimation: true,
            security_analysis: true,
            optimization_hints: true,
            streaming_support: true,
            simd_analysis: false,
            multi_memory: false,
            max_file_size: 100 * 1_024 * 1_024, // 100MB default
        }
    }
}
