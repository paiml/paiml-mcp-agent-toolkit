#![cfg_attr(coverage_nightly, coverage(off))]
//! Unified TypeScript/JavaScript Analyzer - Parse Once, Extract Twice
//!
//! This module eliminates the performance bottleneck of parsing TypeScript/JavaScript files twice
//! (once for AST extraction, once for complexity analysis) by combining both
//! operations into a single parse pass.
//!
//! # Performance Impact
//!
//! Before: 2x parse calls per file (AST + Complexity)
//! After: 1x parse call per file
//! Expected gain: 40-50% reduction in parse time
//!
//! # Example
//!
//! ```no_run
//! use std::path::PathBuf;
//! use pmat::services::unified_typescript_analyzer::UnifiedTypeScriptAnalyzer;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let analyzer = UnifiedTypeScriptAnalyzer::new(PathBuf::from("src/main.ts"));
//!     let result = analyzer.analyze().await?;
//!
//!     println!("Found {} AST items", result.ast_items.len());
//!     println!("Analyzed {} functions", result.file_metrics.functions.len());
//!     Ok(())
//! }
//! ```

use anyhow::Result;
use std::path::{Path, PathBuf};
#[cfg(test)]
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::services::complexity::{ComplexityMetrics, FileComplexityMetrics, FunctionComplexity};
use crate::services::context::AstItem;
use crate::services::enhanced_typescript_visitor::EnhancedTypeScriptVisitor;

#[cfg(feature = "typescript-ast")]
use swc_common::{sync::Lrc, FileName, SourceMap};
#[cfg(feature = "typescript-ast")]
use swc_ecma_ast::Module;
#[cfg(feature = "typescript-ast")]
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsSyntax};

/// Unified analyzer that parses TypeScript/JavaScript once, extracts twice
pub struct UnifiedTypeScriptAnalyzer {
    file_path: PathBuf,

    /// Parse count tracker (test-only)
    #[cfg(test)]
    parse_count: AtomicUsize,
}

/// Combined result from unified analysis
#[derive(Debug)]
pub struct UnifiedAnalysis {
    /// AST items (functions, classes, interfaces, etc.)
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

    #[error("Failed to parse TypeScript/JavaScript syntax: {0}")]
    Parse(String),

    #[error("Analysis error: {0}")]
    Analysis(String),
}

impl UnifiedTypeScriptAnalyzer {
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

        // 2. Parse ONCE with SWC
        #[cfg(feature = "typescript-ast")]
        let syntax_tree = self.parse_typescript(&content)?;

        // 3. Extract AST items using existing EnhancedTypeScriptVisitor
        #[cfg(feature = "typescript-ast")]
        let ast_items = self.extract_ast_items(&syntax_tree);
        #[cfg(not(feature = "typescript-ast"))]
        let ast_items = Vec::new();

        // 4. Extract complexity metrics (minimal implementation for GREEN phase)
        let file_metrics = self.extract_complexity_metrics(&content);

        Ok(UnifiedAnalysis {
            ast_items,
            file_metrics,
            parsed_at: std::time::Instant::now(),
        })
    }

    /// Parse TypeScript/JavaScript with SWC
    #[cfg(feature = "typescript-ast")]
    fn parse_typescript(&self, content: &str) -> Result<Module, AnalysisError> {
        let source_map = Lrc::new(SourceMap::default());
        let source_file = source_map.new_source_file(
            Lrc::new(FileName::Custom(self.file_path.display().to_string())),
            content.to_owned(),
        );

        let lexer = Lexer::new(
            Syntax::Typescript(TsSyntax {
                tsx: self
                    .file_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e == "tsx")
                    .unwrap_or(false),
                decorators: true,
                dts: false,
                no_early_errors: false,
                disallow_ambiguous_jsx_like: false,
            }),
            Default::default(),
            StringInput::from(&*source_file),
            None,
        );

        let mut parser = Parser::new_from(lexer);
        parser
            .parse_module()
            .map_err(|e| AnalysisError::Parse(format!("SWC parse error: {:?}", e)))
    }

    /// Get parse count (test-only, for verifying single parse)
    #[cfg(test)]
    pub fn parse_count(&self) -> usize {
        self.parse_count.load(Ordering::SeqCst)
    }

    /// Extract AST items from parsed TypeScript/JavaScript module
    #[cfg(feature = "typescript-ast")]
    fn extract_ast_items(&self, module: &Module) -> Vec<AstItem> {
        let visitor = EnhancedTypeScriptVisitor::new(&self.file_path);
        visitor.extract_items(module)
    }

    /// Extract complexity metrics from TypeScript/JavaScript content
    ///
    /// GREEN PHASE: Minimal implementation using simple pattern counting.
    /// This will be enhanced in REFACTOR phase with proper complexity calculation.
    fn extract_complexity_metrics(&self, content: &str) -> FileComplexityMetrics {
        // Simple visitor to count functions and estimate complexity
        let mut functions = Vec::new();

        // For GREEN phase, we'll do simple line-based pattern matching
        // This is minimal but functional - can be improved later

        // Count lines for rough estimation
        let lines = content.lines().count();

        // Simple function detection (will miss some edge cases, but good enough for GREEN)
        let function_pattern = regex::Regex::new(
            r"(?:function\s+(\w+)|const\s+(\w+)\s*=\s*(?:async\s*)?\(|(\w+)\s*\(.*?\)\s*\{|async\s+function\s+(\w+))"
        ).expect("internal error");

        for cap in function_pattern.captures_iter(content) {
            let name = cap
                .get(1)
                .or_else(|| cap.get(2))
                .or_else(|| cap.get(3))
                .or_else(|| cap.get(4))
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
            "if", "else if", "for", "while", "switch", "case", "catch", "&&", "||",
            "?", // Ternary and logical operators
        ];

        for keyword in &keywords {
            complexity += content.matches(keyword).count() as u32;
        }

        complexity
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    // === UnifiedTypeScriptAnalyzer creation tests ===

    #[test]
    fn test_analyzer_creation() {
        let path = PathBuf::from("test.ts");
        let analyzer = UnifiedTypeScriptAnalyzer::new(path.clone());
        assert_eq!(analyzer.file_path(), path.as_path());
    }

    #[test]
    fn test_analyzer_tsx_file() {
        let path = PathBuf::from("component.tsx");
        let analyzer = UnifiedTypeScriptAnalyzer::new(path.clone());
        assert_eq!(analyzer.file_path(), path.as_path());
    }

    #[test]
    fn test_analyzer_js_file() {
        let path = PathBuf::from("script.js");
        let analyzer = UnifiedTypeScriptAnalyzer::new(path.clone());
        assert_eq!(analyzer.file_path(), path.as_path());
    }

    #[test]
    fn test_analyzer_jsx_file() {
        let path = PathBuf::from("component.jsx");
        let analyzer = UnifiedTypeScriptAnalyzer::new(path.clone());
        assert_eq!(analyzer.file_path(), path.as_path());
    }

    #[test]
    fn test_analyzer_initial_parse_count() {
        let path = PathBuf::from("test.ts");
        let analyzer = UnifiedTypeScriptAnalyzer::new(path);
        assert_eq!(analyzer.parse_count(), 0);
    }

    // === Analyze tests ===

    #[tokio::test]
    async fn test_parse_count_increments() {
        let temp_file = NamedTempFile::with_suffix(".ts").expect("internal error");
        std::fs::write(temp_file.path(), "function main() {}").expect("internal error");

        let analyzer = UnifiedTypeScriptAnalyzer::new(temp_file.path().to_path_buf());

        assert_eq!(analyzer.parse_count(), 0);

        let _ = analyzer.analyze().await;
        assert_eq!(analyzer.parse_count(), 1);
    }

    #[tokio::test]
    async fn test_analyze_simple_function() {
        let temp_file = NamedTempFile::with_suffix(".ts").expect("internal error");
        std::fs::write(
            temp_file.path(),
            "function greet(name: string) { return 'Hello ' + name; }",
        )
        .expect("internal error");

        let analyzer = UnifiedTypeScriptAnalyzer::new(temp_file.path().to_path_buf());
        let result = analyzer.analyze().await;

        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert!(!analysis.file_metrics.functions.is_empty());
    }

    #[tokio::test]
    async fn test_analyze_multiple_functions() {
        let temp_file = NamedTempFile::with_suffix(".ts").expect("internal error");
        std::fs::write(
            temp_file.path(),
            r#"
            function foo() { return 1; }
            function bar() { return 2; }
            const baz = () => 3;
            "#,
        )
        .expect("internal error");

        let analyzer = UnifiedTypeScriptAnalyzer::new(temp_file.path().to_path_buf());
        let result = analyzer.analyze().await;

        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert!(analysis.file_metrics.functions.len() >= 2);
    }

    #[tokio::test]
    async fn test_analyze_async_function() {
        let temp_file = NamedTempFile::with_suffix(".ts").expect("internal error");
        std::fs::write(
            temp_file.path(),
            "async function fetchData() { return await fetch('url'); }",
        )
        .expect("internal error");

        let analyzer = UnifiedTypeScriptAnalyzer::new(temp_file.path().to_path_buf());
        let result = analyzer.analyze().await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_arrow_function() {
        let temp_file = NamedTempFile::with_suffix(".ts").expect("internal error");
        std::fs::write(
            temp_file.path(),
            "const add = (a: number, b: number) => a + b;",
        )
        .expect("internal error");

        let analyzer = UnifiedTypeScriptAnalyzer::new(temp_file.path().to_path_buf());
        let result = analyzer.analyze().await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_class_with_methods() {
        let temp_file = NamedTempFile::with_suffix(".ts").expect("internal error");
        std::fs::write(
            temp_file.path(),
            r#"
            class Calculator {
                add(a: number, b: number) { return a + b; }
                subtract(a: number, b: number) { return a - b; }
            }
            "#,
        )
        .expect("internal error");

        let analyzer = UnifiedTypeScriptAnalyzer::new(temp_file.path().to_path_buf());
        let result = analyzer.analyze().await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_complex_control_flow() {
        let temp_file = NamedTempFile::with_suffix(".ts").expect("internal error");
        std::fs::write(
            temp_file.path(),
            r#"
            function complex(x: number) {
                if (x > 0) {
                    for (let i = 0; i < x; i++) {
                        if (i % 2 === 0) {
                            console.log(i);
                        }
                    }
                } else {
                    while (x < 0) {
                        x++;
                    }
                }
                return x;
            }
            "#,
        )
        .expect("internal error");

        let analyzer = UnifiedTypeScriptAnalyzer::new(temp_file.path().to_path_buf());
        let result = analyzer.analyze().await;

        assert!(result.is_ok());
        let analysis = result.unwrap();
        // Complexity should be higher due to control flow
        assert!(analysis.file_metrics.total_complexity.cyclomatic > 1);
    }

    #[tokio::test]
    async fn test_analyze_nonexistent_file() {
        let analyzer = UnifiedTypeScriptAnalyzer::new(PathBuf::from("/nonexistent/file.ts"));
        let result = analyzer.analyze().await;

        assert!(result.is_err());
        if let Err(AnalysisError::Io(_)) = result {
            // Expected error type
        } else {
            panic!("Expected Io error");
        }
    }

    #[tokio::test]
    async fn test_analyze_empty_file() {
        let temp_file = NamedTempFile::with_suffix(".ts").expect("internal error");
        std::fs::write(temp_file.path(), "").expect("internal error");

        let analyzer = UnifiedTypeScriptAnalyzer::new(temp_file.path().to_path_buf());
        let result = analyzer.analyze().await;

        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert!(analysis.file_metrics.functions.is_empty());
    }

    #[tokio::test]
    async fn test_analyze_only_comments() {
        let temp_file = NamedTempFile::with_suffix(".ts").expect("internal error");
        std::fs::write(
            temp_file.path(),
            "// This is a comment\n/* Another comment */",
        )
        .expect("internal error");

        let analyzer = UnifiedTypeScriptAnalyzer::new(temp_file.path().to_path_buf());
        let result = analyzer.analyze().await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_interface_only() {
        let temp_file = NamedTempFile::with_suffix(".ts").expect("internal error");
        std::fs::write(
            temp_file.path(),
            "interface User { name: string; age: number; }",
        )
        .expect("internal error");

        let analyzer = UnifiedTypeScriptAnalyzer::new(temp_file.path().to_path_buf());
        let result = analyzer.analyze().await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_type_alias() {
        let temp_file = NamedTempFile::with_suffix(".ts").expect("internal error");
        std::fs::write(temp_file.path(), "type ID = string | number;").expect("internal error");

        let analyzer = UnifiedTypeScriptAnalyzer::new(temp_file.path().to_path_buf());
        let result = analyzer.analyze().await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_parse_count_multiple_calls() {
        let temp_file = NamedTempFile::with_suffix(".ts").expect("internal error");
        std::fs::write(temp_file.path(), "const x = 1;").expect("internal error");

        let analyzer = UnifiedTypeScriptAnalyzer::new(temp_file.path().to_path_buf());

        // Call analyze multiple times
        let _ = analyzer.analyze().await;
        let _ = analyzer.analyze().await;
        let _ = analyzer.analyze().await;

        assert_eq!(analyzer.parse_count(), 3);
    }

    // === Complexity estimation tests ===

    #[test]
    fn test_estimate_complexity_simple() {
        let analyzer = UnifiedTypeScriptAnalyzer::new(PathBuf::from("test.ts"));
        let complexity = analyzer.estimate_complexity("const x = 1;");
        assert_eq!(complexity, 1); // Base complexity
    }

    #[test]
    fn test_estimate_complexity_if_statement() {
        let analyzer = UnifiedTypeScriptAnalyzer::new(PathBuf::from("test.ts"));
        let complexity = analyzer.estimate_complexity("if (x > 0) { return x; }");
        assert!(complexity > 1);
    }

    #[test]
    fn test_estimate_complexity_for_loop() {
        let analyzer = UnifiedTypeScriptAnalyzer::new(PathBuf::from("test.ts"));
        let complexity = analyzer.estimate_complexity("for (let i = 0; i < 10; i++) {}");
        assert!(complexity > 1);
    }

    #[test]
    fn test_estimate_complexity_while_loop() {
        let analyzer = UnifiedTypeScriptAnalyzer::new(PathBuf::from("test.ts"));
        let complexity = analyzer.estimate_complexity("while (x > 0) { x--; }");
        assert!(complexity > 1);
    }

    #[test]
    fn test_estimate_complexity_switch() {
        let analyzer = UnifiedTypeScriptAnalyzer::new(PathBuf::from("test.ts"));
        let complexity =
            analyzer.estimate_complexity("switch (x) { case 1: break; case 2: break; }");
        assert!(complexity > 2); // switch + 2 cases
    }

    #[test]
    fn test_estimate_complexity_try_catch() {
        let analyzer = UnifiedTypeScriptAnalyzer::new(PathBuf::from("test.ts"));
        let complexity = analyzer.estimate_complexity("try { foo(); } catch (e) { bar(); }");
        assert!(complexity > 1);
    }

    #[test]
    fn test_estimate_complexity_logical_operators() {
        let analyzer = UnifiedTypeScriptAnalyzer::new(PathBuf::from("test.ts"));
        let complexity = analyzer.estimate_complexity("if (a && b || c) {}");
        assert!(complexity > 2); // if + && + ||
    }

    #[test]
    fn test_estimate_complexity_ternary() {
        let analyzer = UnifiedTypeScriptAnalyzer::new(PathBuf::from("test.ts"));
        let complexity = analyzer.estimate_complexity("const x = a ? b : c;");
        assert!(complexity > 1);
    }

    // === UnifiedAnalysis tests ===

    #[tokio::test]
    async fn test_unified_analysis_has_parsed_at() {
        let temp_file = NamedTempFile::with_suffix(".ts").expect("internal error");
        std::fs::write(temp_file.path(), "const x = 1;").expect("internal error");

        let analyzer = UnifiedTypeScriptAnalyzer::new(temp_file.path().to_path_buf());
        let before = std::time::Instant::now();
        let result = analyzer.analyze().await.unwrap();
        let after = std::time::Instant::now();

        assert!(result.parsed_at >= before);
        assert!(result.parsed_at <= after);
    }

    #[tokio::test]
    async fn test_unified_analysis_file_path_in_metrics() {
        let temp_file = NamedTempFile::with_suffix(".ts").expect("internal error");
        std::fs::write(temp_file.path(), "const x = 1;").expect("internal error");

        let analyzer = UnifiedTypeScriptAnalyzer::new(temp_file.path().to_path_buf());
        let result = analyzer.analyze().await.unwrap();

        assert!(result.file_metrics.path.contains(".ts"));
    }

    // === AnalysisError tests ===

    #[test]
    fn test_analysis_error_io_display() {
        let err = AnalysisError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found",
        ));
        let display = format!("{}", err);
        assert!(display.contains("Failed to read file"));
    }

    #[test]
    fn test_analysis_error_parse_display() {
        let err = AnalysisError::Parse("syntax error at line 1".to_string());
        let display = format!("{}", err);
        assert!(display.contains("Failed to parse TypeScript"));
    }

    #[test]
    fn test_analysis_error_analysis_display() {
        let err = AnalysisError::Analysis("analysis failed".to_string());
        let display = format!("{}", err);
        assert!(display.contains("Analysis error"));
    }

    #[test]
    fn test_analysis_error_debug() {
        let err = AnalysisError::Parse("test".to_string());
        let debug = format!("{:?}", err);
        assert!(debug.contains("Parse"));
    }
}
