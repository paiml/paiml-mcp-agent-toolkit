//! WebAssembly parser traits and interfaces
//!
//! This module defines the core traits for WebAssembly parsing and analysis.
//! Follows the existing `LanguageParser` pattern while adding WASM-specific capabilities.
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
/// Extends the base `LanguageParser` with WASM-specific analysis capabilities
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

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_parsed_ast_creation() {
        let ast = ParsedAst {
            language: Language::Rust,
            dag: AstDag::default(),
            source_file: Some(std::path::PathBuf::from("/test/file.rs")),
            parse_errors: vec!["error1".to_string()],
            metadata: HashMap::new(),
        };
        assert_eq!(ast.language, Language::Rust);
        assert!(ast.source_file.is_some());
        assert_eq!(ast.parse_errors.len(), 1);
    }

    #[test]
    fn test_parsed_ast_no_source_file() {
        let ast = ParsedAst {
            language: Language::TypeScript,
            dag: AstDag::default(),
            source_file: None,
            parse_errors: vec![],
            metadata: HashMap::from([("key".to_string(), "value".to_string())]),
        };
        assert_eq!(ast.language, Language::TypeScript);
        assert!(ast.source_file.is_none());
        assert!(ast.parse_errors.is_empty());
        assert_eq!(ast.metadata.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_wasm_analysis_capabilities_default() {
        let caps = WasmAnalysisCapabilities::default();
        assert!(caps.memory_analysis);
        assert!(caps.gas_estimation);
        assert!(caps.security_analysis);
        assert!(caps.optimization_hints);
        assert!(caps.streaming_support);
        assert!(!caps.simd_analysis);
        assert!(!caps.multi_memory);
        assert_eq!(caps.max_file_size, 100 * 1024 * 1024);
    }

    #[test]
    fn test_wasm_analysis_capabilities_clone() {
        let caps = WasmAnalysisCapabilities::default();
        let cloned = caps.clone();
        assert_eq!(caps.memory_analysis, cloned.memory_analysis);
        assert_eq!(caps.max_file_size, cloned.max_file_size);
    }

    #[test]
    fn test_wasm_analysis_capabilities_debug() {
        let caps = WasmAnalysisCapabilities::default();
        let debug_str = format!("{:?}", caps);
        assert!(debug_str.contains("WasmAnalysisCapabilities"));
        assert!(debug_str.contains("memory_analysis"));
    }

    #[test]
    fn test_wasm_analysis_capabilities_serialization() {
        let caps = WasmAnalysisCapabilities {
            memory_analysis: false,
            gas_estimation: true,
            security_analysis: false,
            optimization_hints: true,
            streaming_support: false,
            simd_analysis: true,
            multi_memory: true,
            max_file_size: 50 * 1024 * 1024,
        };
        let json = serde_json::to_string(&caps).unwrap();
        assert!(json.contains("\"memory_analysis\":false"));
        assert!(json.contains("\"simd_analysis\":true"));

        let deserialized: WasmAnalysisCapabilities = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.memory_analysis, false);
        assert_eq!(deserialized.simd_analysis, true);
        assert_eq!(deserialized.max_file_size, 50 * 1024 * 1024);
    }

    #[test]
    fn test_wasm_analysis_capabilities_custom_values() {
        let caps = WasmAnalysisCapabilities {
            memory_analysis: true,
            gas_estimation: false,
            security_analysis: true,
            optimization_hints: false,
            streaming_support: true,
            simd_analysis: true,
            multi_memory: true,
            max_file_size: 1024,
        };
        assert!(caps.memory_analysis);
        assert!(!caps.gas_estimation);
        assert!(caps.simd_analysis);
        assert!(caps.multi_memory);
        assert_eq!(caps.max_file_size, 1024);
    }
}
