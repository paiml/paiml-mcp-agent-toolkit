//! Unified AST Parser Trait for Language Consolidation
//!
//! This module defines the UnifiedAstParser trait that all language-specific
//! parsers must implement for architectural consolidation.

use crate::models::error::PmatError;
use crate::models::unified_ast::{Language, UnifiedAstNode};
use crate::services::{
    complexity::FileComplexityMetrics,
    context::FileContext,
    file_classifier::{FileClassifier, ParseDecision},
};
use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;

/// Result of parsing with structured error information
#[derive(Debug, Clone)]
pub struct ParseResult {
    /// File context with AST items
    pub context: FileContext,
    /// Complexity metrics
    pub complexity: FileComplexityMetrics,
    /// Unified AST nodes for vectorized analysis
    pub unified_nodes: Vec<UnifiedAstNode>,
    /// Parse warnings (non-fatal issues)
    pub warnings: Vec<String>,
}

/// Parser capabilities and metadata
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParserCapabilities {
    /// Language this parser supports
    pub language: Language,
    /// File extensions supported
    pub extensions: Vec<&'static str>,
    /// Whether parser supports complexity analysis
    pub supports_complexity: bool,
    /// Whether parser supports dependency analysis
    pub supports_dependencies: bool,
    /// Whether parser supports semantic analysis
    pub supports_semantics: bool,
    /// SIMD-optimized for vectorized operations
    pub simd_optimized: bool,
}

/// Configuration for parser behavior
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// Whether to extract complexity metrics
    pub extract_complexity: bool,
    /// Whether to extract dependency information
    pub extract_dependencies: bool,
    /// Whether to compute semantic hashes
    pub compute_semantic_hashes: bool,
    /// Maximum file size to parse (bytes)
    pub max_file_size: usize,
    /// Parser timeout in milliseconds
    pub timeout_ms: u64,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            extract_complexity: true,
            extract_dependencies: true,
            compute_semantic_hashes: true,
            max_file_size: 10 * 1024 * 1024, // 10MB
            timeout_ms: 5000,                // 5 seconds
        }
    }
}

/// Unified trait that all AST parsers must implement
#[async_trait]
pub trait UnifiedAstParser: Send + Sync {
    /// Parse a file and return structured results
    async fn parse_file(
        &self,
        path: &Path,
        config: &ParserConfig,
        classifier: Option<&FileClassifier>,
    ) -> Result<ParseResult, PmatError>;

    /// Parse content directly (for testing/streaming)
    async fn parse_content(
        &self,
        content: &str,
        file_path: &Path,
        config: &ParserConfig,
    ) -> Result<ParseResult, PmatError>;

    /// Get parser capabilities
    fn capabilities(&self) -> ParserCapabilities;

    /// Check if this parser can handle the given file
    fn can_parse(&self, path: &Path) -> bool {
        let caps = self.capabilities();
        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                return caps.extensions.contains(&ext_str);
            }
        }
        false
    }

    /// Get parser name for logging/debugging
    fn name(&self) -> &'static str;

    /// Validate file before parsing (pre-flight check)
    fn validate_file(
        &self,
        path: &Path,
        config: &ParserConfig,
        classifier: Option<&FileClassifier>,
    ) -> Result<(), PmatError> {
        // Check file size
        if let Ok(metadata) = std::fs::metadata(path) {
            if metadata.len() > config.max_file_size as u64 {
                return Err(PmatError::ValidationError {
                    field: "file_size".to_string(),
                    reason: format!(
                        "File too large: {} bytes > {} bytes",
                        metadata.len(),
                        config.max_file_size
                    ),
                });
            }
        }

        // Check with classifier if provided
        if let Some(classifier) = classifier {
            if let Ok(content) = std::fs::read(path) {
                match classifier.should_parse(path, &content) {
                    ParseDecision::Skip(reason) => {
                        return Err(PmatError::ValidationError {
                            field: "file_content".to_string(),
                            reason: format!("Classifier rejected file: {reason:?}"),
                        });
                    }
                    ParseDecision::Parse => {}
                }
            }
        }

        Ok(())
    }
}

/// Registry for managing multiple AST parsers
#[derive(Default)]
pub struct AstParserRegistry {
    parsers: Vec<Arc<dyn UnifiedAstParser>>,
}

impl AstParserRegistry {
    /// Create a new registry
    pub fn new() -> Self {
        Self {
            parsers: Vec::with_capacity(256),
        }
    }

    /// Register a parser
    pub fn register(&mut self, parser: Arc<dyn UnifiedAstParser>) {
        self.parsers.push(parser);
    }

    /// Find parser for the given file
    pub fn find_parser(&self, path: &Path) -> Option<Arc<dyn UnifiedAstParser>> {
        self.parsers
            .iter()
            .find(|parser| parser.can_parse(path))
            .cloned()
    }

    /// Get all registered parsers
    #[inline]
    pub fn parsers(&self) -> &[Arc<dyn UnifiedAstParser>] {
        &self.parsers
    }

    /// Get parser by language
    pub fn get_parser(&self, language: Language) -> Option<Arc<dyn UnifiedAstParser>> {
        self.parsers
            .iter()
            .find(|parser| parser.capabilities().language == language)
            .cloned()
    }

    /// Parse file using appropriate parser
    pub async fn parse_file(
        &self,
        path: &Path,
        config: &ParserConfig,
        classifier: Option<&FileClassifier>,
    ) -> Result<ParseResult, PmatError> {
        let parser = self
            .find_parser(path)
            .ok_or_else(|| PmatError::ValidationError {
                field: "file_extension".to_string(),
                reason: format!("No parser found for file: {}", path.display()),
            })?;

        parser.parse_file(path, config, classifier).await
    }

    /// Get supported languages
    pub fn supported_languages(&self) -> Vec<Language> {
        self.parsers
            .iter()
            .map(|parser| parser.capabilities().language)
            .collect()
    }

    /// Get supported file extensions
    pub fn supported_extensions(&self) -> Vec<&'static str> {
        self.parsers
            .iter()
            .flat_map(|parser| parser.capabilities().extensions)
            .collect()
    }
}

/// Default registry with all standard parsers
pub fn create_default_registry() -> AstParserRegistry {
    let mut registry = AstParserRegistry::new();

    // Register Rust parser
    registry.register(Arc::new(super::ast_rust_unified::RustAstParser::new()));

    // Register other language parsers (will be implemented for each language)
    // registry.register(Arc::new(TypeScriptAstParser::new()));
    // registry.register(Arc::new(PythonAstParser::new()));
    // registry.register(Arc::new(CAstParser::new()));
    // registry.register(Arc::new(CppAstParser::new()));

    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_config_default() {
        let config = ParserConfig::default();
        assert!(config.extract_complexity);
        assert!(config.extract_dependencies);
        assert!(config.compute_semantic_hashes);
        assert_eq!(config.max_file_size, 10 * 1024 * 1024);
        assert_eq!(config.timeout_ms, 5000);
    }

    #[test]
    fn test_registry_creation() {
        let registry = AstParserRegistry::new();
        assert_eq!(registry.parsers().len(), 0);
    }

    #[test]
    fn test_default_registry() {
        let registry = create_default_registry();
        assert_eq!(registry.parsers().len(), 1); // Rust parser

        // Test that Rust parser can be found
        let rust_parser = registry.find_parser(std::path::Path::new("test.rs"));
        assert!(rust_parser.is_some());

        // Test supported languages
        let languages = registry.supported_languages();
        assert!(languages.contains(&Language::Rust));

        // Test supported extensions
        let extensions = registry.supported_extensions();
        assert!(extensions.contains(&"rs"));
    }
}
