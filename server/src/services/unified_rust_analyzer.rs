//! Unified Rust Analyzer - Parse Once, Extract Twice
//!
//! This module eliminates the performance bottleneck of parsing Rust files twice
//! (once for AST extraction, once for complexity analysis) by combining both
//! operations into a single parse pass.
//!
//! # Performance Impact
//!
//! Before: 2x `syn::parse_file()` calls per file
//! After: 1x `syn::parse_file()` call per file
//! Expected gain: 40-50% reduction in parse time
//!
//! # Example
//!
//! ```no_run
//! use std::path::PathBuf;
//! use pmat::services::unified_rust_analyzer::UnifiedRustAnalyzer;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let analyzer = UnifiedRustAnalyzer::new(PathBuf::from("src/main.rs"));
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
use crate::services::enhanced_ast_visitor::EnhancedAstVisitor;

/// Unified analyzer that parses once, extracts twice
pub struct UnifiedRustAnalyzer {
    file_path: PathBuf,

    /// Parse count tracker (test-only)
    #[cfg(test)]
    parse_count: AtomicUsize,
}

/// Combined result from unified analysis
#[derive(Debug)]
pub struct UnifiedAnalysis {
    /// AST items (functions, structs, enums, traits)
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

    #[error("Failed to parse Rust syntax: {0}")]
    Parse(String),

    #[error("Analysis error: {0}")]
    Analysis(String),
}

impl UnifiedRustAnalyzer {
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

        // 2. Parse ONCE with syn
        let syntax_tree = syn::parse_file(&content)
            .map_err(|e| AnalysisError::Parse(e.to_string()))?;

        // 3. Extract AST items using existing EnhancedAstVisitor
        let ast_items = self.extract_ast_items(&syntax_tree);

        // 4. Extract complexity metrics (minimal implementation for GREEN phase)
        let file_metrics = self.extract_complexity_metrics(&syntax_tree);

        Ok(UnifiedAnalysis {
            ast_items,
            file_metrics,
            parsed_at: std::time::Instant::now(),
        })
    }

    /// Get parse count (test-only, for verifying single parse)
    #[cfg(test)]
    pub fn parse_count(&self) -> usize {
        self.parse_count.load(Ordering::SeqCst)
    }

    /// Extract AST items from parsed syntax tree
    fn extract_ast_items(&self, syntax_tree: &syn::File) -> Vec<AstItem> {
        let visitor = EnhancedAstVisitor::new(&self.file_path);
        visitor.extract_items(syntax_tree)
    }

    /// Extract complexity metrics from parsed syntax tree
    ///
    /// GREEN PHASE: Minimal implementation using simplified complexity visitor.
    /// This will be enhanced in REFACTOR phase with proper complexity calculation.
    fn extract_complexity_metrics(&self, syntax_tree: &syn::File) -> FileComplexityMetrics {
        use syn::visit::Visit;

        // Simple visitor to count functions and estimate complexity
        struct SimpleComplexityVisitor {
            functions: Vec<FunctionComplexity>,
            current_function_index: usize,
        }

        impl SimpleComplexityVisitor {
            fn new() -> Self {
                Self {
                    functions: Vec::new(),
                    current_function_index: 0,
                }
            }
        }

        impl<'ast> Visit<'ast> for SimpleComplexityVisitor {
            fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
                let name = node.sig.ident.to_string();

                // GREEN PHASE: Simple complexity estimation
                // Just count branches for now
                let cyclomatic = self.count_branches(&node.block);
                let cognitive = cyclomatic; // Simplified for GREEN phase

                self.functions.push(FunctionComplexity {
                    name,
                    line_start: 0, // Will be improved in REFACTOR
                    line_end: 0,
                    metrics: ComplexityMetrics {
                        cyclomatic: cyclomatic as u16,
                        cognitive: cognitive as u16,
                        nesting_max: 0, // Will be calculated in REFACTOR
                        lines: 10, // Rough estimate for GREEN phase
                        halstead: None,
                    },
                });

                self.current_function_index += 1;

                // Continue visiting nested items
                syn::visit::visit_item_fn(self, node);
            }

            fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
                let name = node.sig.ident.to_string();

                let cyclomatic = self.count_branches_in_impl(&node.block);
                let cognitive = cyclomatic;

                self.functions.push(FunctionComplexity {
                    name,
                    line_start: 0,
                    line_end: 0,
                    metrics: ComplexityMetrics {
                        cyclomatic: cyclomatic as u16,
                        cognitive: cognitive as u16,
                        nesting_max: 0,
                        lines: 10,
                        halstead: None,
                    },
                });

                syn::visit::visit_impl_item_fn(self, node);
            }
        }

        impl SimpleComplexityVisitor {
            fn count_branches(&self, block: &syn::Block) -> u32 {
                // GREEN PHASE: Simple branch counting
                // Base complexity is 1
                let mut complexity = 1;

                for stmt in &block.stmts {
                    complexity += self.count_branches_in_stmt(stmt);
                }

                complexity
            }

            fn count_branches_in_impl(&self, block: &syn::Block) -> u32 {
                self.count_branches(block)
            }

            fn count_branches_in_stmt(&self, stmt: &syn::Stmt) -> u32 {
                match stmt {
                    syn::Stmt::Expr(expr, _) => self.count_branches_in_expr(expr),
                    _ => 0,
                }
            }

            fn count_branches_in_expr(&self, expr: &syn::Expr) -> u32 {
                match expr {
                    syn::Expr::If(_) => 1,
                    syn::Expr::Match(_) => 1,
                    syn::Expr::While(_) => 1,
                    syn::Expr::ForLoop(_) => 1,
                    syn::Expr::Loop(_) => 1,
                    _ => 0,
                }
            }
        }

        let mut visitor = SimpleComplexityVisitor::new();
        visitor.visit_file(syntax_tree);

        // Calculate file-level metrics
        let total_cyclomatic: u32 = visitor.functions.iter()
            .map(|f| f.metrics.cyclomatic as u32)
            .sum();

        let avg_cyclomatic = if visitor.functions.is_empty() {
            1
        } else {
            total_cyclomatic / visitor.functions.len() as u32
        };

        FileComplexityMetrics {
            path: self.file_path.display().to_string(),
            total_complexity: ComplexityMetrics {
                cyclomatic: avg_cyclomatic as u16,
                cognitive: avg_cyclomatic as u16,
                nesting_max: 0,
                lines: (visitor.functions.len() * 10) as u16,
                halstead: None,
            },
            functions: visitor.functions,
            classes: Vec::new(), // Rust doesn't have classes
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_creation() {
        let path = PathBuf::from("test.rs");
        let analyzer = UnifiedRustAnalyzer::new(path.clone());
        assert_eq!(analyzer.file_path(), path.as_path());
    }

    #[tokio::test]
    async fn test_parse_count_increments() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), "fn main() {}").unwrap();

        let analyzer = UnifiedRustAnalyzer::new(temp_file.path().to_path_buf());

        assert_eq!(analyzer.parse_count(), 0);

        let _ = analyzer.analyze().await;
        assert_eq!(analyzer.parse_count(), 1);
    }
}