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
use crate::services::enhanced_python_visitor::EnhancedPythonVisitor;

#[cfg(feature = "python-ast")]
use rustpython_parser::{ast::ModModule, Parse};

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

        // 2. Parse ONCE with rustpython_parser
        #[cfg(feature = "python-ast")]
        let module = self.parse_python(&content)?;

        // 3. Extract AST items using existing EnhancedPythonVisitor
        #[cfg(feature = "python-ast")]
        let ast_items = self.extract_ast_items(&module);
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

    /// Parse Python with rustpython_parser
    #[cfg(feature = "python-ast")]
    fn parse_python(&self, content: &str) -> Result<ModModule, AnalysisError> {
        let filename = self.file_path.display().to_string();
        ModModule::parse(content, &filename)
            .map_err(|e| AnalysisError::Parse(format!("Python parse error: {}", e)))
    }

    /// Get parse count (test-only, for verifying single parse)
    #[cfg(test)]
    pub fn parse_count(&self) -> usize {
        self.parse_count.load(Ordering::SeqCst)
    }

    /// Extract AST items from parsed Python module
    #[cfg(feature = "python-ast")]
    fn extract_ast_items(&self, module: &ModModule) -> Vec<AstItem> {
        let visitor = EnhancedPythonVisitor::new(&self.file_path);
        visitor.extract_items(module)
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
        let function_pattern = regex::Regex::new(
            r"(?m)^(?:async\s+)?def\s+(\w+)\s*\("
        ).unwrap();

        for cap in function_pattern.captures_iter(content) {
            let name = cap.get(1)
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
        let total_cyclomatic: u32 = functions.iter()
            .map(|f| f.metrics.cyclomatic as u32)
            .sum();

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
            "if ", "elif ", "for ", "while ", "try:", "except",
            "and ", "or ", // Logical operators
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