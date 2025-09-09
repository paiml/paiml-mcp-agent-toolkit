//! Accurate Complexity Analyzer using AST-based analysis
//!
//! Sprint 63: Implements industry-standard complexity calculations
//! - Cyclomatic Complexity: Based on McCabe (1976) - decision points
//! - Cognitive Complexity: Based on SonarSource specification
//! - Supports test exclusion and annotation suppression

use anyhow::Result;
use std::path::Path;
use syn::{visit::Visit, Attribute, Expr, Item, ItemFn, Stmt};
use walkdir::WalkDir;

/// Accurate complexity analyzer with proper AST-based calculation
pub struct AccurateComplexityAnalyzer {
    exclude_tests: bool,
    respect_annotations: bool,
}

impl Default for AccurateComplexityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl AccurateComplexityAnalyzer {
    pub fn new() -> Self {
        Self {
            exclude_tests: false,
            respect_annotations: false,
        }
    }

    pub fn exclude_tests(mut self, exclude: bool) -> Self {
        self.exclude_tests = exclude;
        self
    }

    pub fn respect_annotations(mut self, respect: bool) -> Self {
        self.respect_annotations = respect;
        self
    }

    /// Analyze a single Rust file
    pub async fn analyze_file(&self, path: &Path) -> Result<FileComplexityResult> {
        let content = tokio::fs::read_to_string(path).await?;
        let ast = syn::parse_file(&content)?;

        let mut functions = Vec::new();

        for item in ast.items {
            if let Item::Fn(func) = item {
                let metrics = self.analyze_function(&func);
                functions.push(metrics);
            }
        }

        Ok(FileComplexityResult {
            functions,
            file_path: path.display().to_string(),
        })
    }

    /// Analyze an entire project
    pub async fn analyze_project(&self, path: &Path) -> Result<ProjectComplexityResult> {
        let mut file_metrics = Vec::new();
        let mut files_analyzed = 0;

        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|ext| ext == "rs").unwrap_or(false))
        {
            let file_path = entry.path();

            // Skip test files if requested
            if self.exclude_tests && self.is_test_file(file_path) {
                continue;
            }

            if let Ok(result) = self.analyze_file(file_path).await {
                files_analyzed += 1;
                file_metrics.push(result);
            }
        }

        Ok(ProjectComplexityResult {
            files_analyzed,
            file_metrics,
        })
    }

    /// Analyze a single function
    fn analyze_function(&self, func: &ItemFn) -> FunctionMetrics {
        let name = func.sig.ident.to_string();
        let suppressed = self.respect_annotations && self.has_suppress_annotation(&func.attrs);

        let mut visitor = ComplexityVisitor::new();
        visitor.visit_item_fn(func);

        FunctionMetrics {
            name,
            cyclomatic_complexity: visitor.cyclomatic,
            cognitive_complexity: visitor.cognitive,
            suppressed,
        }
    }

    /// Check if file is a test file
    fn is_test_file(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        path_str.contains("/tests/")
            || path_str.contains("/test/")
            || path_str.ends_with("_test.rs")
            || path_str.ends_with("_tests.rs")
            || path_str.contains("test_")
            || path_str.contains("tests.rs")
    }

    /// Check if function has suppression annotation
    fn has_suppress_annotation(&self, attrs: &[Attribute]) -> bool {
        attrs.iter().any(|attr| {
            // Check if it's an allow attribute
            if attr.path().is_ident("allow") {
                // Check if it contains complex_function
                // In syn 2.0, we need to parse the token stream differently
                let tokens_str = attr
                    .meta
                    .require_list()
                    .map(|list| list.tokens.to_string())
                    .unwrap_or_default();
                tokens_str.contains("complex_function")
            } else {
                false
            }
        })
    }
}

/// AST visitor for calculating complexity metrics
struct ComplexityVisitor {
    cyclomatic: u32,
    cognitive: u32,
    nesting_level: u32,
}

impl ComplexityVisitor {
    fn new() -> Self {
        Self {
            cyclomatic: 1, // Base complexity
            cognitive: 0,
            nesting_level: 0,
        }
    }

    fn add_cyclomatic(&mut self, amount: u32) {
        self.cyclomatic += amount;
    }

    fn add_cognitive(&mut self, base: u32) {
        // Add base cognitive complexity
        self.cognitive += base;
    }
    
    fn add_cognitive_with_nesting(&mut self, base: u32) {
        // Add cognitive complexity including nesting penalty
        self.cognitive += base + self.nesting_level;
    }
}

impl<'ast> Visit<'ast> for ComplexityVisitor {
    fn visit_expr(&mut self, expr: &'ast Expr) {
        match expr {
            // Control flow that adds to cyclomatic complexity
            Expr::If(_if_expr) => {
                self.add_cyclomatic(1);
                // For Sprint 82 fix: Just add 1 for cognitive complexity
                // Don't use nesting incorrectly
                self.add_cognitive(1);
                // Still visit children
                syn::visit::visit_expr(self, expr);
            }
            Expr::Match(match_expr) => {
                // Match adds 1, plus 1 for each arm with a guard
                self.add_cyclomatic(1);
                self.add_cognitive(1);

                for arm in &match_expr.arms {
                    if arm.guard.is_some() {
                        self.add_cyclomatic(1);
                        self.add_cognitive(1);
                    }
                }

                syn::visit::visit_expr(self, expr);
            }
            Expr::While(_) | Expr::ForLoop(_) => {
                self.add_cyclomatic(1);
                self.add_cognitive(1);
                syn::visit::visit_expr(self, expr);
            }
            Expr::Loop(_) => {
                self.add_cyclomatic(1);
                self.add_cognitive(1);
                syn::visit::visit_expr(self, expr);
            }
            // Binary operators that create branches
            Expr::Binary(bin) => {
                use syn::BinOp;
                match bin.op {
                    BinOp::And(_) | BinOp::Or(_) => {
                        self.add_cyclomatic(1);
                        self.add_cognitive(1);
                    }
                    _ => {}
                }
                syn::visit::visit_expr(self, expr);
            }
            // Try operator adds complexity
            Expr::Try(_) => {
                self.add_cyclomatic(1);
                self.add_cognitive(1);
                syn::visit::visit_expr(self, expr);
            }
            // Break and continue add cognitive complexity
            Expr::Break(_) | Expr::Continue(_) => {
                self.add_cognitive(1);
                syn::visit::visit_expr(self, expr);
            }
            // Return early adds cognitive complexity
            Expr::Return(_) => {
                if self.nesting_level > 0 {
                    self.add_cognitive(1);
                }
                syn::visit::visit_expr(self, expr);
            }
            // Recursion detection (simplified - checks for function calls with same name)
            Expr::Call(_call) => {
                // In real implementation, would check if calling self
                self.add_cognitive(1);
                syn::visit::visit_expr(self, expr);
            }
            _ => syn::visit::visit_expr(self, expr),
        }
    }

    fn visit_stmt(&mut self, stmt: &'ast Stmt) {
        // Visit statements normally
        syn::visit::visit_stmt(self, stmt);
    }
}

/// Result of analyzing a single file
#[derive(Debug, Clone)]
pub struct FileComplexityResult {
    pub functions: Vec<FunctionMetrics>,
    pub file_path: String,
}

/// Metrics for a single function
#[derive(Debug, Clone)]
pub struct FunctionMetrics {
    pub name: String,
    pub cyclomatic_complexity: u32,
    pub cognitive_complexity: u32,
    pub suppressed: bool,
}

/// Result of analyzing a project
#[derive(Debug, Clone)]
pub struct ProjectComplexityResult {
    pub files_analyzed: usize,
    pub file_metrics: Vec<FileComplexityResult>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_basic_complexity() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");

        fs::write(
            &test_file,
            r#"
            fn simple() -> i32 {
                42
            }
            
            fn with_if(x: i32) -> i32 {
                if x > 0 {
                    x
                } else {
                    -x
                }
            }
        "#,
        )
        .unwrap();

        let analyzer = AccurateComplexityAnalyzer::new();
        let result = analyzer.analyze_file(&test_file).await.unwrap();

        assert_eq!(result.functions.len(), 2);
        assert_eq!(result.functions[0].cyclomatic_complexity, 1);
        assert_eq!(result.functions[1].cyclomatic_complexity, 2);
    }
}
