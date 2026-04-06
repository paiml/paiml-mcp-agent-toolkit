impl UnifiedRustAnalyzer {
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
                        lines: 10,      // Rough estimate for GREEN phase
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
        let total_cyclomatic: u32 = visitor
            .functions
            .iter()
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
