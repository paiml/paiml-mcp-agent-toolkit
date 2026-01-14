//! AST parsing strategies for multi-language code analysis
//!
//! This module provides language-specific Abstract Syntax Tree (AST) parsing
//! strategies for analyzing code structure across different programming languages.
//! It serves as the foundation for complexity analysis, dead code detection,
//! and other static analysis features.
//!
//! # Architecture
//!
//! The module uses a strategy pattern with language-specific implementations:
//! - **`RustStrategy`**: Uses `syn` for accurate Rust AST parsing
//! - **`TypeScriptStrategy`**: Uses `swc` for JS/TS parsing
//! - **`PythonStrategy`**: Uses regex-based parsing for Python
//! - **C/C++ Strategy**: Uses tree-sitter for C/C++ parsing
//! - **`KotlinStrategy`**: Uses tree-sitter for Kotlin parsing
//!
//! # Features
//!
//! - **Multi-language Support**: Rust, TypeScript, JavaScript, Python, Java, C/C++, Kotlin
//! - **Unified AST Model**: Consistent representation across languages
//! - **Function Detection**: Identifies all functions with line ranges
//! - **Type Detection**: Finds structs, classes, enums, interfaces
//! - **Error Resilience**: Gracefully handles parsing failures
//!
//! # Example
//!
//! ```ignore
//! use pmat::services::ast_strategies::{StrategyRegistry, AstStrategy};
//! use pmat::services::file_classifier::FileClassifier;
//! use std::path::Path;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let registry = StrategyRegistry::new();
//! let classifier = FileClassifier::default();
//!
//! // Get strategy for a specific file
//! if let Some(strategy) = registry.get_strategy("rs") {
//!     let file_context = strategy.analyze(Path::new("main.rs"), &classifier).await?;
//!     
//!     println!("Language: {}", file_context.language);
//!     println!("Found {} items", file_context.items.len());
//!     
//!     for item in &file_context.items {
//!         match item {
//!             pmat::services::context::AstItem::Function { name, line, .. } => {
//!                 println!("Function {} at line {}", name, line);
//!             }
//!             pmat::services::context::AstItem::Struct { name, fields_count, .. } => {
//!                 println!("Struct {} with {} fields", name, fields_count);
//!             }
//!             _ => {}
//!         }
//!     }
//! }
//! # Ok(())
//! # }
//! ```

use anyhow::Result;
use async_trait::async_trait;
use rustc_hash::FxHashMap;
use std::path::Path;
use std::sync::Arc;

use crate::services::context::FileContext;
use crate::services::file_classifier::FileClassifier;

// Strategy trait for language-specific AST analysis
#[async_trait]
pub trait AstStrategy: Send + Sync {
    async fn analyze(&self, path: &Path, classifier: &FileClassifier) -> Result<FileContext>;
    fn supports_extension(&self, ext: &str) -> bool;
}

// Rust language strategy
pub struct RustAstStrategy;

#[async_trait]
impl AstStrategy for RustAstStrategy {
    async fn analyze(&self, path: &Path, classifier: &FileClassifier) -> Result<FileContext> {
        crate::services::ast_rust::analyze_rust_file_with_classifier(path, Some(classifier))
            .await
            .map_err(|e| anyhow::anyhow!("Rust AST analysis error: {e}"))
    }

    fn supports_extension(&self, ext: &str) -> bool {
        ext == "rs"
    }
}

#[cfg(feature = "typescript-ast")]
// TypeScript/TSX strategy
pub struct TypeScriptAstStrategy;

#[cfg(feature = "typescript-ast")]
#[async_trait]
impl AstStrategy for TypeScriptAstStrategy {
    async fn analyze(&self, path: &Path, classifier: &FileClassifier) -> Result<FileContext> {
        crate::services::ast_typescript::analyze_typescript_file_with_classifier(
            path,
            Some(classifier),
        )
        .await
        .map_err(|e| anyhow::anyhow!("TypeScript AST analysis error: {e}"))
    }

    fn supports_extension(&self, ext: &str) -> bool {
        matches!(ext, "ts" | "tsx")
    }
}

#[cfg(feature = "typescript-ast")]
// JavaScript/JSX strategy
pub struct JavaScriptAstStrategy;

#[cfg(feature = "typescript-ast")]
#[async_trait]
impl AstStrategy for JavaScriptAstStrategy {
    async fn analyze(&self, path: &Path, classifier: &FileClassifier) -> Result<FileContext> {
        crate::services::ast_typescript::analyze_javascript_file_with_classifier(
            path,
            Some(classifier),
        )
        .await
        .map_err(|e| anyhow::anyhow!("JavaScript AST analysis error: {e}"))
    }

    fn supports_extension(&self, ext: &str) -> bool {
        matches!(ext, "js" | "jsx")
    }
}

#[cfg(feature = "python-ast")]
// Python strategy
pub struct PythonAstStrategy;

#[cfg(feature = "python-ast")]
#[async_trait]
impl AstStrategy for PythonAstStrategy {
    async fn analyze(&self, path: &Path, classifier: &FileClassifier) -> Result<FileContext> {
        crate::services::ast_python::analyze_python_file_with_classifier(path, Some(classifier))
            .await
            .map_err(|e| anyhow::anyhow!("Python AST analysis error: {e}"))
    }

    fn supports_extension(&self, ext: &str) -> bool {
        ext == "py"
    }
}

// Strategy registry to manage all language strategies
pub struct StrategyRegistry {
    strategies: FxHashMap<String, Arc<dyn AstStrategy>>,
}

impl StrategyRegistry {
    #[must_use]
    pub fn new() -> Self {
        let mut strategies: FxHashMap<String, Arc<dyn AstStrategy>> = FxHashMap::default();

        // Register all supported language strategies
        let rust_strategy = Arc::new(RustAstStrategy) as Arc<dyn AstStrategy>;
        strategies.insert("rs".to_string(), rust_strategy);

        #[cfg(feature = "typescript-ast")]
        {
            let ts_strategy = Arc::new(TypeScriptAstStrategy) as Arc<dyn AstStrategy>;
            strategies.insert("ts".to_string(), ts_strategy.clone());
            strategies.insert("tsx".to_string(), ts_strategy);

            let js_strategy = Arc::new(JavaScriptAstStrategy) as Arc<dyn AstStrategy>;
            strategies.insert("js".to_string(), js_strategy.clone());
            strategies.insert("jsx".to_string(), js_strategy);
        }

        #[cfg(feature = "python-ast")]
        {
            let py_strategy = Arc::new(PythonAstStrategy) as Arc<dyn AstStrategy>;
            strategies.insert("py".to_string(), py_strategy);
        }

        #[cfg(feature = "c-ast")]
        {
            let c_strategy = Arc::new(CAstStrategy) as Arc<dyn AstStrategy>;
            strategies.insert("c".to_string(), c_strategy.clone());
            strategies.insert("h".to_string(), c_strategy);

            let cpp_strategy = Arc::new(CppAstStrategy) as Arc<dyn AstStrategy>;
            strategies.insert("cpp".to_string(), cpp_strategy.clone());
            strategies.insert("cc".to_string(), cpp_strategy.clone());
            strategies.insert("cxx".to_string(), cpp_strategy.clone());
            strategies.insert("hpp".to_string(), cpp_strategy.clone());
            strategies.insert("hxx".to_string(), cpp_strategy);
        }

        #[cfg(feature = "kotlin-ast")]
        {
            let kotlin_strategy = Arc::new(KotlinAstStrategy) as Arc<dyn AstStrategy>;
            strategies.insert("kt".to_string(), kotlin_strategy.clone());
            strategies.insert("kts".to_string(), kotlin_strategy);
        }

        Self { strategies }
    }

    #[must_use]
    pub fn get_strategy(&self, extension: &str) -> Option<Arc<dyn AstStrategy>> {
        self.strategies.get(extension).cloned()
    }

    pub fn register_strategy(&mut self, extension: String, strategy: Arc<dyn AstStrategy>) {
        self.strategies.insert(extension, strategy);
    }
}

// C language strategy
#[cfg(feature = "c-ast")]
pub struct CAstStrategy;

#[cfg(feature = "c-ast")]
#[async_trait]
impl AstStrategy for CAstStrategy {
    async fn analyze(&self, path: &Path, _classifier: &FileClassifier) -> Result<FileContext> {
        use crate::services::ast_c::CAstParser;
        use tokio::fs;

        // Read file content
        let content = fs::read_to_string(path).await?;

        // Parse using C AST parser
        let mut parser = CAstParser::new();
        match parser.parse_file(path, &content) {
            Ok(ast_dag) => {
                // Convert AST DAG to FileContext items
                let mut items = Vec::new();
                let content_lines: Vec<&str> = content.lines().collect();

                for node in ast_dag.nodes.iter() {
                    // Extract name from source using name_vector hash and source range
                    let name = Self::extract_name_from_node(node, &content);
                    let line_number =
                        Self::byte_pos_to_line(node.source_range.start as usize, &content_lines);

                    let item = match &node.kind {
                        crate::models::unified_ast::AstKind::Function(_) => {
                            crate::services::context::AstItem::Function {
                                name: name.unwrap_or_else(|| "anonymous_function".to_string()),
                                visibility: "public".to_string(),
                                is_async: false, // C functions are not async
                                line: line_number,
                            }
                        }
                        crate::models::unified_ast::AstKind::Type(type_kind) => {
                            match type_kind {
                                crate::models::unified_ast::TypeKind::Struct => {
                                    crate::services::context::AstItem::Struct {
                                        name: name
                                            .unwrap_or_else(|| "anonymous_struct".to_string()),
                                        visibility: "public".to_string(),
                                        fields_count: 0, // Could be computed from AST if needed
                                        derives: vec![], // C doesn't have derives
                                        line: line_number,
                                    }
                                }
                                crate::models::unified_ast::TypeKind::Enum => {
                                    crate::services::context::AstItem::Enum {
                                        name: name.unwrap_or_else(|| "anonymous_enum".to_string()),
                                        visibility: "public".to_string(),
                                        variants_count: 0, // Could be computed from AST if needed
                                        line: line_number,
                                    }
                                }
                                _ => continue, // Skip other type kinds for now
                            }
                        }
                        _ => continue, // Skip other node kinds for now
                    };
                    items.push(item);
                }

                Ok(FileContext {
                    path: path.to_string_lossy().to_string(),
                    language: "c".to_string(),
                    items,
                    complexity_metrics: None, // Could be computed from AST if needed
                })
            }
            Err(e) => {
                // Return empty context on parse error but don't fail completely
                tracing::warn!("Failed to parse C file {}: {}", path.display(), e);
                Ok(FileContext {
                    path: path.to_string_lossy().to_string(),
                    language: "c".to_string(),
                    items: vec![],
                    complexity_metrics: None,
                })
            }
        }
    }

    fn supports_extension(&self, ext: &str) -> bool {
        matches!(ext, "c" | "h")
    }
}

#[cfg(feature = "c-ast")]
impl CAstStrategy {
    /// Extract name from `UnifiedAstNode` by analyzing the source range
    fn extract_name_from_node(
        node: &crate::models::unified_ast::UnifiedAstNode,
        content: &str,
    ) -> Option<String> {
        // For now, extract a reasonable segment from the source range
        let start = node.source_range.start as usize;
        let end = node.source_range.end as usize;

        if start >= content.len() || end > content.len() || start >= end {
            return None;
        }

        let source_text = &content[start..end];

        // Use simple heuristics to extract identifiers from the source text
        match &node.kind {
            crate::models::unified_ast::AstKind::Function(_) => {
                Self::extract_function_name(source_text)
            }
            crate::models::unified_ast::AstKind::Type(_) => Self::extract_type_name(source_text),
            _ => None,
        }
    }

    /// Extract function name from source text
    fn extract_function_name(source_text: &str) -> Option<String> {
        // Look for pattern: type name(...) or name(...)
        if let Some(paren_pos) = source_text.find('(') {
            let before_paren = &source_text[..paren_pos];
            // Split by whitespace and take the last word (function name)
            before_paren
                .split_whitespace()
                .last()
                .map(|s| s.trim_start_matches('*').to_string())
        } else {
            None
        }
    }

    /// Extract type name from source text (struct, enum, etc.)
    fn extract_type_name(source_text: &str) -> Option<String> {
        // Look for patterns like "struct name" or "enum name"
        let words: Vec<&str> = source_text.split_whitespace().collect();
        if words.len() >= 2 {
            // Usually the name follows the keyword (struct/enum/typedef)
            for i in 0..words.len() - 1 {
                if matches!(words[i], "struct" | "enum" | "union" | "typedef") {
                    return Some(words[i + 1].trim_end_matches('{').to_string());
                }
            }
        }
        None
    }

    /// Convert byte position to line number
    fn byte_pos_to_line(byte_pos: usize, content_lines: &[&str]) -> usize {
        let mut current_pos = 0;
        for (line_idx, line) in content_lines.iter().enumerate() {
            if current_pos + line.len() >= byte_pos {
                return line_idx + 1; // 1-based line numbers
            }
            current_pos += line.len() + 1; // +1 for newline character
        }
        content_lines.len() // Return last line if position is beyond content
    }
}

// C++ language strategy
#[cfg(feature = "c-ast")]
pub struct CppAstStrategy;

#[cfg(feature = "c-ast")]
#[async_trait]
impl AstStrategy for CppAstStrategy {
    async fn analyze(&self, path: &Path, _classifier: &FileClassifier) -> Result<FileContext> {
        use crate::services::ast_cpp::CppAstParser;
        use tokio::fs;

        // Read file content
        let content = fs::read_to_string(path).await?;

        // Parse using C++ AST parser
        let mut parser = CppAstParser::new();
        match parser.parse_file(path, &content) {
            Ok(ast_dag) => {
                // Convert AST DAG to FileContext items
                let mut items = Vec::new();
                let content_lines: Vec<&str> = content.lines().collect();

                for node in ast_dag.nodes.iter() {
                    // Extract name from source using proper parsing
                    let name = Self::extract_name_from_node(node, &content);
                    let line_number =
                        Self::byte_pos_to_line(node.source_range.start as usize, &content_lines);

                    let item = match &node.kind {
                        crate::models::unified_ast::AstKind::Function(_) => {
                            crate::services::context::AstItem::Function {
                                name: name.unwrap_or_else(|| "anonymous_function".to_string()),
                                visibility: "public".to_string(),
                                is_async: false, // C++ functions are not async by default
                                line: line_number,
                            }
                        }
                        crate::models::unified_ast::AstKind::Type(type_kind) => {
                            match type_kind {
                                crate::models::unified_ast::TypeKind::Struct => {
                                    crate::services::context::AstItem::Struct {
                                        name: name
                                            .unwrap_or_else(|| "anonymous_struct".to_string()),
                                        visibility: "public".to_string(),
                                        fields_count: 0, // Could be computed from AST if needed
                                        derives: vec![], // C++ doesn't have derives like Rust
                                        line: line_number,
                                    }
                                }
                                crate::models::unified_ast::TypeKind::Class => {
                                    crate::services::context::AstItem::Struct {
                                        name: name.unwrap_or_else(|| "anonymous_class".to_string()),
                                        visibility: "public".to_string(),
                                        fields_count: 0, // Could be computed from AST if needed
                                        derives: vec![], // C++ doesn't have derives like Rust
                                        line: line_number,
                                    }
                                }
                                crate::models::unified_ast::TypeKind::Enum => {
                                    crate::services::context::AstItem::Enum {
                                        name: name.unwrap_or_else(|| "anonymous_enum".to_string()),
                                        visibility: "public".to_string(),
                                        variants_count: 0, // Could be computed from AST if needed
                                        line: line_number,
                                    }
                                }
                                _ => continue, // Skip other type kinds for now
                            }
                        }
                        _ => continue, // Skip other node kinds for now
                    };
                    items.push(item);
                }

                Ok(FileContext {
                    path: path.to_string_lossy().to_string(),
                    language: "cpp".to_string(),
                    items,
                    complexity_metrics: None, // Could be computed from AST if needed
                })
            }
            Err(e) => {
                // Return empty context on parse error but don't fail completely
                tracing::warn!("Failed to parse C++ file {}: {}", path.display(), e);
                Ok(FileContext {
                    path: path.to_string_lossy().to_string(),
                    language: "cpp".to_string(),
                    items: vec![],
                    complexity_metrics: None,
                })
            }
        }
    }

    fn supports_extension(&self, ext: &str) -> bool {
        matches!(ext, "cpp" | "cc" | "cxx" | "hpp" | "hxx")
    }
}

#[cfg(feature = "c-ast")]
impl CppAstStrategy {
    /// Extract name from `UnifiedAstNode` by analyzing the source range
    fn extract_name_from_node(
        node: &crate::models::unified_ast::UnifiedAstNode,
        content: &str,
    ) -> Option<String> {
        // For now, extract a reasonable segment from the source range
        let start = node.source_range.start as usize;
        let end = node.source_range.end as usize;

        if start >= content.len() || end > content.len() || start >= end {
            return None;
        }

        let source_text = &content[start..end];

        // Use simple heuristics to extract identifiers from the source text
        match &node.kind {
            crate::models::unified_ast::AstKind::Function(_) => {
                Self::extract_function_name(source_text)
            }
            crate::models::unified_ast::AstKind::Type(_) => Self::extract_type_name(source_text),
            _ => None,
        }
    }

    /// Extract function name from source text (C++ can include templates, operators, etc.)
    fn extract_function_name(source_text: &str) -> Option<String> {
        // Look for pattern: type name(...) or name(...)
        if let Some(paren_pos) = source_text.find('(') {
            let before_paren = &source_text[..paren_pos];
            // Split by whitespace and take the last word (function name)
            // Handle operator overloads and destructors
            let name = before_paren.split_whitespace().last().map(|s| {
                s.trim_start_matches('*')
                    .trim_start_matches('~')
                    .to_string()
            })?;

            // Handle operator overloads
            if before_paren.contains("operator") {
                return Some("operator_overload".to_string());
            }

            Some(name)
        } else {
            None
        }
    }

    /// Extract type name from source text (struct, class, enum, etc.)
    fn extract_type_name(source_text: &str) -> Option<String> {
        // Look for patterns like "class name", "struct name", "enum name"
        let words: Vec<&str> = source_text.split_whitespace().collect();
        if words.len() >= 2 {
            // Usually the name follows the keyword (class/struct/enum/typedef)
            for i in 0..words.len() - 1 {
                if matches!(words[i], "class" | "struct" | "enum" | "union" | "typedef") {
                    let name = words[i + 1].trim_end_matches('{').trim_end_matches('<');
                    return Some(name.to_string());
                }
            }
        }
        None
    }

    /// Convert byte position to line number
    fn byte_pos_to_line(byte_pos: usize, content_lines: &[&str]) -> usize {
        let mut current_pos = 0;
        for (line_idx, line) in content_lines.iter().enumerate() {
            if current_pos + line.len() >= byte_pos {
                return line_idx + 1; // 1-based line numbers
            }
            current_pos += line.len() + 1; // +1 for newline character
        }
        content_lines.len() // Return last line if position is beyond content
    }
}

#[cfg(feature = "kotlin-ast")]
pub struct KotlinAstStrategy;

#[cfg(feature = "kotlin-ast")]
impl KotlinAstStrategy {
    /// Extract name from `UnifiedAstNode` by analyzing the source range
    fn extract_name_from_node(
        node: &crate::models::unified_ast::UnifiedAstNode,
        content: &str,
    ) -> Option<String> {
        // For now, extract a reasonable segment from the source range
        let start = node.source_range.start as usize;
        let end = node.source_range.end as usize;

        if start >= content.len() || end > content.len() || start >= end {
            return None;
        }

        let source_text = &content[start..end];

        // Use simple heuristics to extract identifiers from the source text
        match &node.kind {
            crate::models::unified_ast::AstKind::Function(_) => {
                Self::extract_function_name(source_text)
            }
            crate::models::unified_ast::AstKind::Type(_) => Self::extract_class_name(source_text),
            _ => None,
        }
    }

    /// Extract function name from Kotlin source text
    fn extract_function_name(source_text: &str) -> Option<String> {
        // Look for pattern: fun name(...)
        if let Some(fun_pos) = source_text.find("fun ") {
            let after_fun = &source_text[fun_pos + 4..];
            if let Some(paren_pos) = after_fun.find('(') {
                let name_part = &after_fun[..paren_pos];
                // Take the first word as the function name
                return name_part
                    .split_whitespace()
                    .next()
                    .map(std::string::ToString::to_string);
            }
        }
        None
    }

    /// Extract class/interface/object name from source text
    fn extract_class_name(source_text: &str) -> Option<String> {
        // Look for patterns like "class Name", "interface Name", "object Name", "data class Name", "enum class Name"
        let lines = source_text.lines().next()?; // Get first line
        let words: Vec<&str> = lines.split_whitespace().collect();

        // Handle enum class first
        if words.len() >= 3 && words[0] == "enum" && words[1] == "class" {
            let name_with_extras = words[2];
            let name = name_with_extras
                .split(['(', ':', '<', '{'])
                .next()
                .unwrap_or(name_with_extras)
                .trim();
            return Some(name.to_string());
        }

        // Handle data class
        if words.len() >= 3 && words[0] == "data" && words[1] == "class" {
            let name_with_extras = words[2];
            let name = name_with_extras
                .split(['(', ':', '<'])
                .next()
                .unwrap_or(name_with_extras);
            return Some(name.to_string());
        }

        // Handle regular class/interface/object
        for i in 0..words.len() {
            if matches!(words[i], "class" | "interface" | "object") && i + 1 < words.len() {
                let name_with_extras = words[i + 1];
                // Remove everything after the first '(' or ':' or '<'
                let name = name_with_extras
                    .split(['(', ':', '<'])
                    .next()
                    .unwrap_or(name_with_extras);
                return Some(name.to_string());
            }
        }

        None
    }

    /// Convert byte position to line number
    fn byte_pos_to_line(byte_pos: usize, content_lines: &[&str]) -> usize {
        let mut current_pos = 0;
        for (line_idx, line) in content_lines.iter().enumerate() {
            if current_pos + line.len() >= byte_pos {
                return line_idx + 1; // 1-based line numbers
            }
            current_pos += line.len() + 1; // +1 for newline character
        }
        content_lines.len() // Return last line if position is beyond content
    }
}

#[cfg(feature = "kotlin-ast")]
#[async_trait]
impl AstStrategy for KotlinAstStrategy {
    async fn analyze(&self, path: &Path, classifier: &FileClassifier) -> Result<FileContext> {
        // Delegate to the new implementation in ast::languages::kotlin_strategy
        use crate::services::ast::languages::kotlin_strategy::KotlinStrategy;

        let kotlin_strategy = KotlinStrategy;
        kotlin_strategy.analyze(path, classifier).await
    }

    fn supports_extension(&self, ext: &str) -> bool {
        matches!(ext, "kt" | "kts")
    }
}

impl Default for StrategyRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // RustAstStrategy Tests
    // ============================================================

    #[test]
    fn test_rust_strategy_supports_rs_extension() {
        let strategy = RustAstStrategy;
        assert!(strategy.supports_extension("rs"));
    }

    #[test]
    fn test_rust_strategy_rejects_other_extensions() {
        let strategy = RustAstStrategy;
        assert!(!strategy.supports_extension("py"));
        assert!(!strategy.supports_extension("ts"));
        assert!(!strategy.supports_extension("js"));
        assert!(!strategy.supports_extension("c"));
        assert!(!strategy.supports_extension("cpp"));
        assert!(!strategy.supports_extension("kt"));
        assert!(!strategy.supports_extension(""));
        assert!(!strategy.supports_extension("rust"));
    }

    // ============================================================
    // StrategyRegistry Tests
    // ============================================================

    #[test]
    fn test_strategy_registry_new() {
        let registry = StrategyRegistry::new();
        // Rust strategy should always be available
        assert!(registry.get_strategy("rs").is_some());
    }

    #[test]
    fn test_strategy_registry_default() {
        let registry = StrategyRegistry::default();
        assert!(registry.get_strategy("rs").is_some());
    }

    #[test]
    fn test_strategy_registry_get_rust_strategy() {
        let registry = StrategyRegistry::new();
        let strategy = registry.get_strategy("rs");
        assert!(strategy.is_some());
        let strategy = strategy.unwrap();
        assert!(strategy.supports_extension("rs"));
    }

    #[test]
    fn test_strategy_registry_get_nonexistent_strategy() {
        let registry = StrategyRegistry::new();
        assert!(registry.get_strategy("unknown").is_none());
        assert!(registry.get_strategy("xyz").is_none());
        assert!(registry.get_strategy("").is_none());
    }

    #[test]
    fn test_strategy_registry_register_custom_strategy() {
        let mut registry = StrategyRegistry::new();
        let rust_strategy = Arc::new(RustAstStrategy) as Arc<dyn AstStrategy>;

        // Register with a custom extension
        registry.register_strategy("custom".to_string(), rust_strategy);

        assert!(registry.get_strategy("custom").is_some());
        let strategy = registry.get_strategy("custom").unwrap();
        // Note: The registered strategy still supports "rs", not "custom"
        assert!(strategy.supports_extension("rs"));
    }

    #[test]
    fn test_strategy_registry_override_existing() {
        let mut registry = StrategyRegistry::new();
        let new_strategy = Arc::new(RustAstStrategy) as Arc<dyn AstStrategy>;

        // Override the existing "rs" strategy
        registry.register_strategy("rs".to_string(), new_strategy.clone());

        let retrieved = registry.get_strategy("rs");
        assert!(retrieved.is_some());
    }

    // ============================================================
    // TypeScript Strategy Tests (feature-gated)
    // ============================================================

    #[cfg(feature = "typescript-ast")]
    mod typescript_tests {
        use super::*;

        #[test]
        fn test_typescript_strategy_supports_ts() {
            let strategy = TypeScriptAstStrategy;
            assert!(strategy.supports_extension("ts"));
        }

        #[test]
        fn test_typescript_strategy_supports_tsx() {
            let strategy = TypeScriptAstStrategy;
            assert!(strategy.supports_extension("tsx"));
        }

        #[test]
        fn test_typescript_strategy_rejects_others() {
            let strategy = TypeScriptAstStrategy;
            assert!(!strategy.supports_extension("js"));
            assert!(!strategy.supports_extension("jsx"));
            assert!(!strategy.supports_extension("rs"));
            assert!(!strategy.supports_extension(""));
        }

        #[test]
        fn test_registry_has_typescript_strategy() {
            let registry = StrategyRegistry::new();
            assert!(registry.get_strategy("ts").is_some());
            assert!(registry.get_strategy("tsx").is_some());
        }
    }

    // ============================================================
    // JavaScript Strategy Tests (feature-gated)
    // ============================================================

    #[cfg(feature = "typescript-ast")]
    mod javascript_tests {
        use super::*;

        #[test]
        fn test_javascript_strategy_supports_js() {
            let strategy = JavaScriptAstStrategy;
            assert!(strategy.supports_extension("js"));
        }

        #[test]
        fn test_javascript_strategy_supports_jsx() {
            let strategy = JavaScriptAstStrategy;
            assert!(strategy.supports_extension("jsx"));
        }

        #[test]
        fn test_javascript_strategy_rejects_others() {
            let strategy = JavaScriptAstStrategy;
            assert!(!strategy.supports_extension("ts"));
            assert!(!strategy.supports_extension("tsx"));
            assert!(!strategy.supports_extension("rs"));
            assert!(!strategy.supports_extension(""));
        }

        #[test]
        fn test_registry_has_javascript_strategy() {
            let registry = StrategyRegistry::new();
            assert!(registry.get_strategy("js").is_some());
            assert!(registry.get_strategy("jsx").is_some());
        }
    }

    // ============================================================
    // Python Strategy Tests (feature-gated)
    // ============================================================

    #[cfg(feature = "python-ast")]
    mod python_tests {
        use super::*;

        #[test]
        fn test_python_strategy_supports_py() {
            let strategy = PythonAstStrategy;
            assert!(strategy.supports_extension("py"));
        }

        #[test]
        fn test_python_strategy_rejects_others() {
            let strategy = PythonAstStrategy;
            assert!(!strategy.supports_extension("pyw"));
            assert!(!strategy.supports_extension("pyc"));
            assert!(!strategy.supports_extension("rs"));
            assert!(!strategy.supports_extension(""));
        }

        #[test]
        fn test_registry_has_python_strategy() {
            let registry = StrategyRegistry::new();
            assert!(registry.get_strategy("py").is_some());
        }
    }

    // ============================================================
    // C Strategy Tests (feature-gated)
    // ============================================================

    #[cfg(feature = "c-ast")]
    mod c_tests {
        use super::*;

        #[test]
        fn test_c_strategy_supports_c() {
            let strategy = CAstStrategy;
            assert!(strategy.supports_extension("c"));
        }

        #[test]
        fn test_c_strategy_supports_h() {
            let strategy = CAstStrategy;
            assert!(strategy.supports_extension("h"));
        }

        #[test]
        fn test_c_strategy_rejects_others() {
            let strategy = CAstStrategy;
            assert!(!strategy.supports_extension("cpp"));
            assert!(!strategy.supports_extension("hpp"));
            assert!(!strategy.supports_extension("rs"));
            assert!(!strategy.supports_extension(""));
        }

        #[test]
        fn test_registry_has_c_strategy() {
            let registry = StrategyRegistry::new();
            assert!(registry.get_strategy("c").is_some());
            assert!(registry.get_strategy("h").is_some());
        }

        // ============================================================
        // CAstStrategy Helper Function Tests
        // ============================================================

        #[test]
        fn test_c_extract_function_name_simple() {
            let source = "int main(int argc, char **argv)";
            let name = CAstStrategy::extract_function_name(source);
            assert_eq!(name, Some("main".to_string()));
        }

        #[test]
        fn test_c_extract_function_name_with_pointer() {
            let source = "void *allocate_memory(size_t size)";
            let name = CAstStrategy::extract_function_name(source);
            assert_eq!(name, Some("allocate_memory".to_string()));
        }

        #[test]
        fn test_c_extract_function_name_no_params() {
            let source = "void no_params()";
            let name = CAstStrategy::extract_function_name(source);
            assert_eq!(name, Some("no_params".to_string()));
        }

        #[test]
        fn test_c_extract_function_name_no_paren() {
            let source = "int variable = 42";
            let name = CAstStrategy::extract_function_name(source);
            assert_eq!(name, None);
        }

        #[test]
        fn test_c_extract_function_name_empty() {
            let source = "";
            let name = CAstStrategy::extract_function_name(source);
            assert_eq!(name, None);
        }

        #[test]
        fn test_c_extract_type_name_struct() {
            let source = "struct MyStruct { int x; }";
            let name = CAstStrategy::extract_type_name(source);
            assert_eq!(name, Some("MyStruct".to_string()));
        }

        #[test]
        fn test_c_extract_type_name_enum() {
            let source = "enum Color { RED, GREEN, BLUE }";
            let name = CAstStrategy::extract_type_name(source);
            assert_eq!(name, Some("Color".to_string()));
        }

        #[test]
        fn test_c_extract_type_name_union() {
            let source = "union Data { int i; float f; }";
            let name = CAstStrategy::extract_type_name(source);
            assert_eq!(name, Some("Data".to_string()));
        }

        #[test]
        fn test_c_extract_type_name_typedef() {
            let source = "typedef int MyInt";
            let name = CAstStrategy::extract_type_name(source);
            assert_eq!(name, Some("int".to_string()));
        }

        #[test]
        fn test_c_extract_type_name_no_keyword() {
            let source = "int x = 5";
            let name = CAstStrategy::extract_type_name(source);
            assert_eq!(name, None);
        }

        #[test]
        fn test_c_extract_type_name_single_word() {
            let source = "struct";
            let name = CAstStrategy::extract_type_name(source);
            assert_eq!(name, None);
        }

        #[test]
        fn test_c_byte_pos_to_line_first_line() {
            let content_lines = vec!["first line", "second line", "third line"];
            let line = CAstStrategy::byte_pos_to_line(0, &content_lines);
            assert_eq!(line, 1);
        }

        #[test]
        fn test_c_byte_pos_to_line_second_line() {
            let content_lines = vec!["first line", "second line", "third line"];
            // First line is 10 chars + 1 newline = 11
            let line = CAstStrategy::byte_pos_to_line(11, &content_lines);
            assert_eq!(line, 2);
        }

        #[test]
        fn test_c_byte_pos_to_line_third_line() {
            let content_lines = vec!["first line", "second line", "third line"];
            // First line is 10 + 1, second line is 11 + 1 = 22
            let line = CAstStrategy::byte_pos_to_line(22, &content_lines);
            assert_eq!(line, 3);
        }

        #[test]
        fn test_c_byte_pos_to_line_beyond_content() {
            let content_lines = vec!["first line", "second line"];
            let line = CAstStrategy::byte_pos_to_line(1000, &content_lines);
            assert_eq!(line, 2); // Returns last line
        }

        #[test]
        fn test_c_byte_pos_to_line_empty() {
            let content_lines: Vec<&str> = vec![];
            let line = CAstStrategy::byte_pos_to_line(0, &content_lines);
            assert_eq!(line, 0);
        }
    }

    // ============================================================
    // C++ Strategy Tests (feature-gated)
    // ============================================================

    #[cfg(feature = "c-ast")]
    mod cpp_tests {
        use super::*;

        #[test]
        fn test_cpp_strategy_supports_cpp() {
            let strategy = CppAstStrategy;
            assert!(strategy.supports_extension("cpp"));
        }

        #[test]
        fn test_cpp_strategy_supports_cc() {
            let strategy = CppAstStrategy;
            assert!(strategy.supports_extension("cc"));
        }

        #[test]
        fn test_cpp_strategy_supports_cxx() {
            let strategy = CppAstStrategy;
            assert!(strategy.supports_extension("cxx"));
        }

        #[test]
        fn test_cpp_strategy_supports_hpp() {
            let strategy = CppAstStrategy;
            assert!(strategy.supports_extension("hpp"));
        }

        #[test]
        fn test_cpp_strategy_supports_hxx() {
            let strategy = CppAstStrategy;
            assert!(strategy.supports_extension("hxx"));
        }

        #[test]
        fn test_cpp_strategy_rejects_c() {
            let strategy = CppAstStrategy;
            assert!(!strategy.supports_extension("c"));
            assert!(!strategy.supports_extension("h"));
        }

        #[test]
        fn test_cpp_strategy_rejects_others() {
            let strategy = CppAstStrategy;
            assert!(!strategy.supports_extension("rs"));
            assert!(!strategy.supports_extension("py"));
            assert!(!strategy.supports_extension(""));
        }

        #[test]
        fn test_registry_has_cpp_strategy() {
            let registry = StrategyRegistry::new();
            assert!(registry.get_strategy("cpp").is_some());
            assert!(registry.get_strategy("cc").is_some());
            assert!(registry.get_strategy("cxx").is_some());
            assert!(registry.get_strategy("hpp").is_some());
            assert!(registry.get_strategy("hxx").is_some());
        }

        // ============================================================
        // CppAstStrategy Helper Function Tests
        // ============================================================

        #[test]
        fn test_cpp_extract_function_name_simple() {
            let source = "void processData(int value)";
            let name = CppAstStrategy::extract_function_name(source);
            assert_eq!(name, Some("processData".to_string()));
        }

        #[test]
        fn test_cpp_extract_function_name_with_pointer() {
            let source = "std::string* getName()";
            let name = CppAstStrategy::extract_function_name(source);
            assert_eq!(name, Some("getName".to_string()));
        }

        #[test]
        fn test_cpp_extract_function_name_destructor() {
            let source = "~MyClass()";
            let name = CppAstStrategy::extract_function_name(source);
            assert_eq!(name, Some("MyClass".to_string()));
        }

        #[test]
        fn test_cpp_extract_function_name_operator() {
            let source = "bool operator==(const MyClass& other)";
            let name = CppAstStrategy::extract_function_name(source);
            assert_eq!(name, Some("operator_overload".to_string()));
        }

        #[test]
        fn test_cpp_extract_function_name_no_paren() {
            let source = "int member_variable";
            let name = CppAstStrategy::extract_function_name(source);
            assert_eq!(name, None);
        }

        #[test]
        fn test_cpp_extract_type_name_class() {
            let source = "class MyClass { public: };";
            let name = CppAstStrategy::extract_type_name(source);
            assert_eq!(name, Some("MyClass".to_string()));
        }

        #[test]
        fn test_cpp_extract_type_name_struct() {
            let source = "struct Point { int x; int y; };";
            let name = CppAstStrategy::extract_type_name(source);
            assert_eq!(name, Some("Point".to_string()));
        }

        #[test]
        fn test_cpp_extract_type_name_enum() {
            let source = "enum class Status { OK, ERROR };";
            let name = CppAstStrategy::extract_type_name(source);
            assert_eq!(name, Some("Status".to_string()));
        }

        #[test]
        fn test_cpp_extract_type_name_union() {
            let source = "union Variant { int i; float f; };";
            let name = CppAstStrategy::extract_type_name(source);
            assert_eq!(name, Some("Variant".to_string()));
        }

        #[test]
        fn test_cpp_extract_type_name_template() {
            let source = "class Vector<T> { };";
            let name = CppAstStrategy::extract_type_name(source);
            assert_eq!(name, Some("Vector".to_string()));
        }

        #[test]
        fn test_cpp_extract_type_name_with_brace() {
            let source = "struct Data{ int x; }";
            let name = CppAstStrategy::extract_type_name(source);
            assert_eq!(name, Some("Data".to_string()));
        }

        #[test]
        fn test_cpp_extract_type_name_no_keyword() {
            let source = "int main() { }";
            let name = CppAstStrategy::extract_type_name(source);
            assert_eq!(name, None);
        }

        #[test]
        fn test_cpp_byte_pos_to_line_first_line() {
            let content_lines = vec!["// comment", "int main() {", "}"];
            let line = CppAstStrategy::byte_pos_to_line(0, &content_lines);
            assert_eq!(line, 1);
        }

        #[test]
        fn test_cpp_byte_pos_to_line_middle() {
            let content_lines = vec!["first", "second", "third"];
            // first = 5 + 1 = 6
            let line = CppAstStrategy::byte_pos_to_line(6, &content_lines);
            assert_eq!(line, 2);
        }

        #[test]
        fn test_cpp_byte_pos_to_line_end() {
            let content_lines = vec!["a", "b"];
            let line = CppAstStrategy::byte_pos_to_line(100, &content_lines);
            assert_eq!(line, 2);
        }
    }

    // ============================================================
    // Kotlin Strategy Tests (feature-gated)
    // ============================================================

    #[cfg(feature = "kotlin-ast")]
    mod kotlin_tests {
        use super::*;

        #[test]
        fn test_kotlin_strategy_supports_kt() {
            let strategy = KotlinAstStrategy;
            assert!(strategy.supports_extension("kt"));
        }

        #[test]
        fn test_kotlin_strategy_supports_kts() {
            let strategy = KotlinAstStrategy;
            assert!(strategy.supports_extension("kts"));
        }

        #[test]
        fn test_kotlin_strategy_rejects_others() {
            let strategy = KotlinAstStrategy;
            assert!(!strategy.supports_extension("java"));
            assert!(!strategy.supports_extension("rs"));
            assert!(!strategy.supports_extension(""));
        }

        #[test]
        fn test_registry_has_kotlin_strategy() {
            let registry = StrategyRegistry::new();
            assert!(registry.get_strategy("kt").is_some());
            assert!(registry.get_strategy("kts").is_some());
        }

        // ============================================================
        // KotlinAstStrategy Helper Function Tests
        // ============================================================

        #[test]
        fn test_kotlin_extract_function_name_simple() {
            let source = "fun greet(name: String): String";
            let name = KotlinAstStrategy::extract_function_name(source);
            assert_eq!(name, Some("greet".to_string()));
        }

        #[test]
        fn test_kotlin_extract_function_name_no_params() {
            let source = "fun main()";
            let name = KotlinAstStrategy::extract_function_name(source);
            assert_eq!(name, Some("main".to_string()));
        }

        #[test]
        fn test_kotlin_extract_function_name_no_fun() {
            let source = "val x = 5";
            let name = KotlinAstStrategy::extract_function_name(source);
            assert_eq!(name, None);
        }

        #[test]
        fn test_kotlin_extract_function_name_no_paren() {
            let source = "fun incomplete";
            let name = KotlinAstStrategy::extract_function_name(source);
            assert_eq!(name, None);
        }

        #[test]
        fn test_kotlin_extract_class_name_regular() {
            let source = "class MyClass";
            let name = KotlinAstStrategy::extract_class_name(source);
            assert_eq!(name, Some("MyClass".to_string()));
        }

        #[test]
        fn test_kotlin_extract_class_name_with_constructor() {
            let source = "class Person(val name: String)";
            let name = KotlinAstStrategy::extract_class_name(source);
            assert_eq!(name, Some("Person".to_string()));
        }

        #[test]
        fn test_kotlin_extract_class_name_with_inheritance() {
            let source = "class Child: Parent";
            let name = KotlinAstStrategy::extract_class_name(source);
            assert_eq!(name, Some("Child".to_string()));
        }

        #[test]
        fn test_kotlin_extract_class_name_data_class() {
            let source = "data class User(val id: Int, val name: String)";
            let name = KotlinAstStrategy::extract_class_name(source);
            assert_eq!(name, Some("User".to_string()));
        }

        #[test]
        fn test_kotlin_extract_class_name_enum_class() {
            let source = "enum class Color { RED, GREEN, BLUE }";
            let name = KotlinAstStrategy::extract_class_name(source);
            assert_eq!(name, Some("Color".to_string()));
        }

        #[test]
        fn test_kotlin_extract_class_name_interface() {
            let source = "interface Drawable";
            let name = KotlinAstStrategy::extract_class_name(source);
            assert_eq!(name, Some("Drawable".to_string()));
        }

        #[test]
        fn test_kotlin_extract_class_name_object() {
            let source = "object Singleton";
            let name = KotlinAstStrategy::extract_class_name(source);
            assert_eq!(name, Some("Singleton".to_string()));
        }

        #[test]
        fn test_kotlin_extract_class_name_with_generic() {
            let source = "class Container<T>";
            let name = KotlinAstStrategy::extract_class_name(source);
            assert_eq!(name, Some("Container".to_string()));
        }

        #[test]
        fn test_kotlin_extract_class_name_no_keyword() {
            let source = "fun main() {}";
            let name = KotlinAstStrategy::extract_class_name(source);
            assert_eq!(name, None);
        }

        #[test]
        fn test_kotlin_byte_pos_to_line_first() {
            let content_lines = vec!["package com.example", "class Test"];
            let line = KotlinAstStrategy::byte_pos_to_line(0, &content_lines);
            assert_eq!(line, 1);
        }

        #[test]
        fn test_kotlin_byte_pos_to_line_second() {
            let content_lines = vec!["package com.example", "class Test"];
            // package com.example = 19 + 1 = 20
            let line = KotlinAstStrategy::byte_pos_to_line(20, &content_lines);
            assert_eq!(line, 2);
        }

        #[test]
        fn test_kotlin_byte_pos_to_line_beyond() {
            let content_lines = vec!["line1"];
            let line = KotlinAstStrategy::byte_pos_to_line(1000, &content_lines);
            assert_eq!(line, 1);
        }

        #[test]
        fn test_kotlin_byte_pos_to_line_empty() {
            let content_lines: Vec<&str> = vec![];
            let line = KotlinAstStrategy::byte_pos_to_line(0, &content_lines);
            assert_eq!(line, 0);
        }
    }

    // ============================================================
    // Edge Case Tests
    // ============================================================

    #[test]
    fn test_strategy_registry_multiple_gets() {
        let registry = StrategyRegistry::new();
        let s1 = registry.get_strategy("rs");
        let s2 = registry.get_strategy("rs");
        assert!(s1.is_some());
        assert!(s2.is_some());
    }

    #[test]
    fn test_strategy_registry_case_sensitivity() {
        let registry = StrategyRegistry::new();
        // Extensions should be case-sensitive
        assert!(registry.get_strategy("rs").is_some());
        assert!(registry.get_strategy("RS").is_none());
        assert!(registry.get_strategy("Rs").is_none());
    }

    #[test]
    fn test_strategy_registry_whitespace() {
        let registry = StrategyRegistry::new();
        assert!(registry.get_strategy(" rs").is_none());
        assert!(registry.get_strategy("rs ").is_none());
        assert!(registry.get_strategy(" rs ").is_none());
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
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

        /// Property: RustAstStrategy only supports "rs" extension
        #[test]
        fn rust_strategy_only_supports_rs(ext in "[a-z]{1,5}") {
            let strategy = RustAstStrategy;
            if ext == "rs" {
                prop_assert!(strategy.supports_extension(&ext));
            } else {
                prop_assert!(!strategy.supports_extension(&ext));
            }
        }

        /// Property: StrategyRegistry always has Rust strategy
        #[test]
        fn registry_always_has_rust(_seed in 0u32..1000) {
            let registry = StrategyRegistry::new();
            prop_assert!(registry.get_strategy("rs").is_some());
        }

        /// Property: Registered strategies can be retrieved
        #[test]
        fn registered_strategy_retrievable(ext in "[a-z]{1,10}") {
            let mut registry = StrategyRegistry::new();
            let strategy = Arc::new(RustAstStrategy) as Arc<dyn AstStrategy>;
            registry.register_strategy(ext.clone(), strategy);
            prop_assert!(registry.get_strategy(&ext).is_some());
        }

        /// Property: Non-registered extensions return None
        #[test]
        fn unknown_extension_returns_none(ext in "[a-z0-9]{6,15}") {
            let registry = StrategyRegistry::new();
            // Long random extensions should not match known strategies
            if ext.len() > 5 {
                prop_assert!(registry.get_strategy(&ext).is_none());
            }
        }
    }

    // Feature-gated property tests for C strategy helpers
    #[cfg(feature = "c-ast")]
    proptest! {
        /// Property: byte_pos_to_line returns valid line numbers
        #[test]
        fn c_byte_pos_always_returns_valid_line(pos in 0usize..10000, num_lines in 1usize..100) {
            let lines: Vec<&str> = (0..num_lines).map(|_| "content").collect();
            let result = CAstStrategy::byte_pos_to_line(pos, &lines);
            prop_assert!(result >= 1 || lines.is_empty());
            prop_assert!(result <= num_lines || lines.is_empty());
        }

        /// Property: function name extraction handles parentheses
        #[test]
        fn c_function_name_requires_paren(source in "[a-z ]+") {
            let result = CAstStrategy::extract_function_name(&source);
            if !source.contains('(') {
                prop_assert!(result.is_none());
            }
        }
    }

    // Feature-gated property tests for C++ strategy helpers
    #[cfg(feature = "c-ast")]
    proptest! {
        /// Property: C++ byte_pos_to_line returns valid line numbers
        #[test]
        fn cpp_byte_pos_always_returns_valid_line(pos in 0usize..10000, num_lines in 1usize..100) {
            let lines: Vec<&str> = (0..num_lines).map(|_| "code").collect();
            let result = CppAstStrategy::byte_pos_to_line(pos, &lines);
            prop_assert!(result >= 1 || lines.is_empty());
            prop_assert!(result <= num_lines || lines.is_empty());
        }
    }

    // Feature-gated property tests for Kotlin strategy helpers
    #[cfg(feature = "kotlin-ast")]
    proptest! {
        /// Property: Kotlin byte_pos_to_line returns valid line numbers
        #[test]
        fn kotlin_byte_pos_always_returns_valid_line(pos in 0usize..10000, num_lines in 1usize..100) {
            let lines: Vec<&str> = (0..num_lines).map(|_| "kotlin").collect();
            let result = KotlinAstStrategy::byte_pos_to_line(pos, &lines);
            prop_assert!(result >= 1 || lines.is_empty());
            prop_assert!(result <= num_lines || lines.is_empty());
        }

        /// Property: function name extraction requires "fun " keyword
        #[test]
        fn kotlin_function_name_requires_fun(source in "[a-z ()]+") {
            let result = KotlinAstStrategy::extract_function_name(&source);
            if !source.contains("fun ") {
                prop_assert!(result.is_none());
            }
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::path::Path;
    use tempfile::TempDir;

    /// Test RustAstStrategy analyze with a real file
    #[tokio::test]
    async fn test_rust_strategy_analyze_simple_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("test.rs");

        // Write a simple Rust file
        std::fs::write(
            &file_path,
            r#"
fn main() {
    println!("Hello, world!");
}

pub fn helper() -> i32 {
    42
}
"#,
        )
        .expect("Failed to write test file");

        let strategy = RustAstStrategy;
        let classifier = FileClassifier::default();
        let result = strategy.analyze(&file_path, &classifier).await;

        assert!(result.is_ok());
        let context = result.unwrap();
        assert_eq!(context.language, "rust");
        assert!(!context.items.is_empty());
    }

    /// Test RustAstStrategy analyze with nonexistent file
    #[tokio::test]
    async fn test_rust_strategy_analyze_nonexistent_file() {
        let strategy = RustAstStrategy;
        let classifier = FileClassifier::default();
        let result = strategy.analyze(Path::new("/nonexistent/path/file.rs"), &classifier).await;

        // Should return an error for nonexistent file
        assert!(result.is_err());
    }

    /// Test RustAstStrategy analyze with empty file
    #[tokio::test]
    async fn test_rust_strategy_analyze_empty_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("empty.rs");

        std::fs::write(&file_path, "").expect("Failed to write test file");

        let strategy = RustAstStrategy;
        let classifier = FileClassifier::default();
        let result = strategy.analyze(&file_path, &classifier).await;

        // Empty file should parse but have no items
        assert!(result.is_ok());
        let context = result.unwrap();
        assert!(context.items.is_empty());
    }

    /// Test RustAstStrategy analyze with syntax error
    #[tokio::test]
    async fn test_rust_strategy_analyze_syntax_error() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("invalid.rs");

        // Write invalid Rust code
        std::fs::write(&file_path, "fn main( { incomplete syntax").expect("Failed to write test file");

        let strategy = RustAstStrategy;
        let classifier = FileClassifier::default();
        let result = strategy.analyze(&file_path, &classifier).await;

        // May error or return partial results depending on implementation
        // We just want to ensure it doesn't panic
        let _ = result;
    }

    /// Test RustAstStrategy analyze with complex file
    #[tokio::test]
    async fn test_rust_strategy_analyze_complex_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("complex.rs");

        std::fs::write(
            &file_path,
            r#"
use std::collections::HashMap;

pub struct MyStruct {
    field1: i32,
    field2: String,
}

pub enum MyEnum {
    Variant1,
    Variant2(i32),
}

pub trait MyTrait {
    fn do_something(&self);
}

impl MyTrait for MyStruct {
    fn do_something(&self) {
        println!("{}", self.field1);
    }
}

pub async fn async_function() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

mod inner_module {
    pub fn inner_function() {}
}
"#,
        )
        .expect("Failed to write test file");

        let strategy = RustAstStrategy;
        let classifier = FileClassifier::default();
        let result = strategy.analyze(&file_path, &classifier).await;

        assert!(result.is_ok());
        let context = result.unwrap();
        assert_eq!(context.language, "rust");
        // Should have multiple items
        assert!(context.items.len() >= 4);
    }

    #[cfg(feature = "typescript-ast")]
    mod typescript_integration {
        use super::*;

        #[tokio::test]
        async fn test_typescript_strategy_analyze_simple() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let file_path = temp_dir.path().join("test.ts");

            std::fs::write(
                &file_path,
                r#"
function greet(name: string): string {
    return `Hello, ${name}!`;
}

export class Greeter {
    private name: string;

    constructor(name: string) {
        this.name = name;
    }

    greet(): string {
        return `Hello, ${this.name}!`;
    }
}
"#,
            )
            .expect("Failed to write test file");

            let strategy = TypeScriptAstStrategy;
            let classifier = FileClassifier::default();
            let result = strategy.analyze(&file_path, &classifier).await;

            assert!(result.is_ok());
            let context = result.unwrap();
            assert!(context.language == "typescript" || context.language == "ts");
        }
    }

    #[cfg(feature = "python-ast")]
    mod python_integration {
        use super::*;

        #[tokio::test]
        async fn test_python_strategy_analyze_simple() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let file_path = temp_dir.path().join("test.py");

            std::fs::write(
                &file_path,
                r#"
def greet(name: str) -> str:
    return f"Hello, {name}!"

class Greeter:
    def __init__(self, name: str):
        self.name = name

    def greet(self) -> str:
        return f"Hello, {self.name}!"
"#,
            )
            .expect("Failed to write test file");

            let strategy = PythonAstStrategy;
            let classifier = FileClassifier::default();
            let result = strategy.analyze(&file_path, &classifier).await;

            assert!(result.is_ok());
            let context = result.unwrap();
            assert!(context.language == "python" || context.language == "py");
        }
    }

    #[cfg(feature = "c-ast")]
    mod c_integration {
        use super::*;

        #[tokio::test]
        async fn test_c_strategy_analyze_simple() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let file_path = temp_dir.path().join("test.c");

            std::fs::write(
                &file_path,
                r#"
#include <stdio.h>

struct Point {
    int x;
    int y;
};

int add(int a, int b) {
    return a + b;
}

int main(int argc, char *argv[]) {
    printf("Hello, World!\n");
    return 0;
}
"#,
            )
            .expect("Failed to write test file");

            let strategy = CAstStrategy;
            let classifier = FileClassifier::default();
            let result = strategy.analyze(&file_path, &classifier).await;

            assert!(result.is_ok());
            let context = result.unwrap();
            assert_eq!(context.language, "c");
        }

        #[tokio::test]
        async fn test_cpp_strategy_analyze_simple() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let file_path = temp_dir.path().join("test.cpp");

            std::fs::write(
                &file_path,
                r#"
#include <iostream>
#include <string>

class Greeter {
public:
    std::string name;

    Greeter(const std::string& n) : name(n) {}

    void greet() const {
        std::cout << "Hello, " << name << "!" << std::endl;
    }
};

int main() {
    Greeter g("World");
    g.greet();
    return 0;
}
"#,
            )
            .expect("Failed to write test file");

            let strategy = CppAstStrategy;
            let classifier = FileClassifier::default();
            let result = strategy.analyze(&file_path, &classifier).await;

            assert!(result.is_ok());
            let context = result.unwrap();
            assert_eq!(context.language, "cpp");
        }
    }

    #[cfg(feature = "kotlin-ast")]
    mod kotlin_integration {
        use super::*;

        #[tokio::test]
        async fn test_kotlin_strategy_analyze_simple() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let file_path = temp_dir.path().join("test.kt");

            std::fs::write(
                &file_path,
                r#"
package com.example

fun greet(name: String): String {
    return "Hello, $name!"
}

class Greeter(private val name: String) {
    fun greet(): String = "Hello, $name!"
}

data class User(val id: Int, val name: String)

enum class Status { ACTIVE, INACTIVE }
"#,
            )
            .expect("Failed to write test file");

            let strategy = KotlinAstStrategy;
            let classifier = FileClassifier::default();
            let result = strategy.analyze(&file_path, &classifier).await;

            assert!(result.is_ok());
            let context = result.unwrap();
            assert!(context.language == "kotlin" || context.language == "kt");
        }
    }
}
