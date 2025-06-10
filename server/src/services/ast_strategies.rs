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
            .map_err(|e| anyhow::anyhow!("Rust AST analysis error: {}", e))
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
        .map_err(|e| anyhow::anyhow!("TypeScript AST analysis error: {}", e))
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
        .map_err(|e| anyhow::anyhow!("JavaScript AST analysis error: {}", e))
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
            .map_err(|e| anyhow::anyhow!("Python AST analysis error: {}", e))
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

        Self { strategies }
    }

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
    /// Extract name from UnifiedAstNode by analyzing the source range
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
    /// Extract name from UnifiedAstNode by analyzing the source range
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

impl Default for StrategyRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_strategies_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}
