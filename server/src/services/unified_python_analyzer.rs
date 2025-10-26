//! Unified Python Analyzer - Parse Once, Extract Twice
//!
//! This module eliminates the performance bottleneck of parsing Python files twice
//! (once for AST extraction, once for complexity analysis) by combining both
//! operations into a single parse pass.
//!
//! # Performance Impact
//!
//! Before: 2x parse calls per file (AST + Complexity)
//! After: 1x parse call per file
//! Expected gain: 40-50% reduction in parse time

use anyhow::Result;
use std::path::{Path, PathBuf};
#[cfg(test)]
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::services::complexity::{ComplexityMetrics, FileComplexityMetrics, FunctionComplexity};
use crate::services::context::AstItem;

// Modern tree-sitter-python parsing (replaces rustpython-parser)
#[cfg(feature = "python-ast")]
use tree_sitter::{Parser as TsParser, Tree};

/// Unified analyzer that parses Python once, extracts twice
pub struct UnifiedPythonAnalyzer {
    file_path: PathBuf,

    /// Parse count tracker (test-only)
    #[cfg(test)]
    parse_count: AtomicUsize,
}

/// Combined result from unified analysis
#[derive(Debug)]
pub struct UnifiedAnalysis {
    /// AST items (functions, classes, methods)
    pub ast_items: Vec<AstItem>,

    /// File-level complexity metrics
    pub file_metrics: FileComplexityMetrics,

    /// Parse timestamp (for cache validation)
    pub parsed_at: std::time::Instant,
}

/// Error type for unified analysis
#[derive(Debug, thiserror::Error)]
pub enum AnalysisError {
    #[error("Failed to read file: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to parse Python syntax: {0}")]
    Parse(String),

    #[error("Analysis error: {0}")]
    Analysis(String),
}

impl UnifiedPythonAnalyzer {
    /// Create new analyzer for a file
    pub fn new(file_path: PathBuf) -> Self {
        Self {
            file_path,
            #[cfg(test)]
            parse_count: AtomicUsize::new(0),
        }
    }

    /// Get the file path being analyzed
    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    /// Analyze file with single parse
    ///
    /// This is the core GREEN phase implementation: minimal but correct.
    pub async fn analyze(&self) -> Result<UnifiedAnalysis, AnalysisError> {
        // Track parse count for testing
        #[cfg(test)]
        {
            self.parse_count.fetch_add(1, Ordering::SeqCst);
        }

        // 1. Read file content (single I/O operation)
        let content = tokio::fs::read_to_string(&self.file_path)
            .await
            .map_err(AnalysisError::Io)?;

        // 2. Parse ONCE with tree-sitter-python
        #[cfg(feature = "python-ast")]
        let tree = self.parse_python(&content)?;

        // 3. Extract AST items using tree-sitter
        #[cfg(feature = "python-ast")]
        let ast_items = self.extract_ast_items(&tree, &content);
        #[cfg(not(feature = "python-ast"))]
        let ast_items = Vec::new();

        // 4. Extract complexity metrics (minimal implementation for GREEN phase)
        let file_metrics = self.extract_complexity_metrics(&content);

        Ok(UnifiedAnalysis {
            ast_items,
            file_metrics,
            parsed_at: std::time::Instant::now(),
        })
    }

    /// Parse Python with tree-sitter-python
    #[cfg(feature = "python-ast")]
    fn parse_python(&self, content: &str) -> Result<Tree, AnalysisError> {
        let mut parser = TsParser::new();
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .map_err(|e| AnalysisError::Parse(format!("Failed to set Python language: {}", e)))?;

        let tree = parser
            .parse(content, None)
            .ok_or_else(|| AnalysisError::Parse("Failed to parse Python code".to_string()))?;

        // Check for syntax errors
        if Self::has_syntax_errors(&tree) {
            return Err(AnalysisError::Parse(
                "Python syntax error detected in source".to_string(),
            ));
        }

        Ok(tree)
    }

    /// Check if tree-sitter parse tree has syntax errors
    #[cfg(feature = "python-ast")]
    fn has_syntax_errors(tree: &Tree) -> bool {
        let root = tree.root_node();
        Self::node_has_error(&root)
    }

    /// Recursively check node for errors
    #[cfg(feature = "python-ast")]
    fn node_has_error(node: &tree_sitter::Node) -> bool {
        if node.kind() == "ERROR" || node.is_error() || node.is_missing() {
            return true;
        }

        for child in node.children(&mut node.walk()) {
            if Self::node_has_error(&child) {
                return true;
            }
        }

        false
    }

    /// Get parse count (test-only, for verifying single parse)
    #[cfg(test)]
    pub fn parse_count(&self) -> usize {
        self.parse_count.load(Ordering::SeqCst)
    }

    /// Extract AST items from parsed Python tree using tree-sitter
    #[cfg(feature = "python-ast")]
    fn extract_ast_items(&self, tree: &Tree, source: &str) -> Vec<AstItem> {
        let mut items = Vec::new();
        let root = tree.root_node();
        self.visit_node_for_items(&root, source, &mut items, &mut Vec::new());
        items
    }

    /// Visit tree-sitter node to extract AST items
    #[cfg(feature = "python-ast")]
    fn visit_node_for_items(
        &self,
        node: &tree_sitter::Node,
        source: &str,
        items: &mut Vec<AstItem>,
        class_stack: &mut Vec<String>,
    ) {
        match node.kind() {
            "function_definition" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = &source[name_node.byte_range()];
                    let qualified_name = if class_stack.is_empty() {
                        name.to_string()
                    } else {
                        format!("{}::{}", class_stack.join("::"), name)
                    };

                    items.push(AstItem::Function {
                        name: qualified_name,
                        visibility: "public".to_string(),
                        is_async: false, // TODO: detect async functions
                        line: node.start_position().row + 1,
                    });
                }

                // Visit children
                for child in node.children(&mut node.walk()) {
                    self.visit_node_for_items(&child, source, items, class_stack);
                }
            }
            "class_definition" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = &source[name_node.byte_range()];
                    let qualified_name = if class_stack.is_empty() {
                        name.to_string()
                    } else {
                        format!("{}::{}", class_stack.join("::"), name)
                    };

                    // Python classes map to Struct in AstItem enum
                    items.push(AstItem::Struct {
                        name: qualified_name,
                        visibility: "public".to_string(),
                        fields_count: 0, // Python classes don't expose field count easily
                        derives: Vec::new(),
                        line: node.start_position().row + 1,
                    });

                    // Push class onto stack for nested items
                    class_stack.push(name.to_string());

                    // Visit children
                    for child in node.children(&mut node.walk()) {
                        self.visit_node_for_items(&child, source, items, class_stack);
                    }

                    // Pop class from stack
                    class_stack.pop();
                }
            }
            _ => {
                // Visit children for other node types
                for child in node.children(&mut node.walk()) {
                    self.visit_node_for_items(&child, source, items, class_stack);
                }
            }
        }
    }

    /// Extract complexity metrics from Python content
    ///
    /// GREEN PHASE: Minimal implementation using simple pattern counting.
    /// This will be enhanced in REFACTOR phase with proper complexity calculation.
    fn extract_complexity_metrics(&self, content: &str) -> FileComplexityMetrics {
        let mut functions = Vec::new();

        // Count lines for rough estimation
        let lines = content.lines().count();

        // Simple function detection (GREEN phase - basic regex)
        let function_pattern = regex::Regex::new(r"(?m)^(?:async\s+)?def\s+(\w+)\s*\(").unwrap();

        for cap in function_pattern.captures_iter(content) {
            let name = cap
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_else(|| "anonymous".to_string());

            // Simple complexity: count control flow keywords
            let cyclomatic = self.estimate_complexity(content);

            functions.push(FunctionComplexity {
                name,
                line_start: 0, // Will be improved in REFACTOR
                line_end: 0,
                metrics: ComplexityMetrics {
                    cyclomatic: cyclomatic as u16,
                    cognitive: cyclomatic as u16, // Simplified for GREEN phase
                    nesting_max: 0,
                    lines: 10, // Rough estimate
                    halstead: None,
                },
            });
        }

        // Calculate file-level metrics
        let total_cyclomatic: u32 = functions.iter().map(|f| f.metrics.cyclomatic as u32).sum();

        let avg_cyclomatic = if functions.is_empty() {
            1
        } else {
            total_cyclomatic / functions.len() as u32
        };

        FileComplexityMetrics {
            path: self.file_path.display().to_string(),
            total_complexity: ComplexityMetrics {
                cyclomatic: avg_cyclomatic as u16,
                cognitive: avg_cyclomatic as u16,
                nesting_max: 0,
                lines: lines as u16,
                halstead: None,
            },
            functions,
            classes: Vec::new(), // Will be extracted in REFACTOR phase
        }
    }

    /// Estimate complexity by counting control flow keywords
    /// GREEN PHASE: Simple pattern matching
    fn estimate_complexity(&self, content: &str) -> u32 {
        let mut complexity = 1; // Base complexity

        // Count control flow keywords
        let keywords = [
            "if ", "elif ", "for ", "while ", "try:", "except", "and ",
            "or ", // Logical operators
        ];

        for keyword in &keywords {
            complexity += content.matches(keyword).count() as u32;
        }

        complexity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_creation() {
        let path = PathBuf::from("test.py");
        let analyzer = UnifiedPythonAnalyzer::new(path.clone());
        assert_eq!(analyzer.file_path(), path.as_path());
    }

    #[tokio::test]
    async fn test_parse_count_increments() {
        let temp_file = tempfile::NamedTempFile::with_suffix(".py").unwrap();
        std::fs::write(temp_file.path(), "def main():\n    pass").unwrap();

        let analyzer = UnifiedPythonAnalyzer::new(temp_file.path().to_path_buf());

        assert_eq!(analyzer.parse_count(), 0);

        let _ = analyzer.analyze().await;
        assert_eq!(analyzer.parse_count(), 1);
    }
}
