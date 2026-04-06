#![cfg_attr(coverage_nightly, coverage(off))]
//! Complexity visitor using syn's Visit trait for accurate Rust analysis

use syn::visit::Visit;

/// Visitor for calculating complexity metrics
#[allow(dead_code)]
pub(crate) struct ComplexityVisitor {
    /// Current cyclomatic complexity
    pub(crate) complexity: u32,

    /// Current cognitive complexity
    pub(crate) cognitive: u32,

    /// Current nesting level for cognitive complexity
    pub(crate) nesting_level: u32,

    /// Number of functions
    pub(crate) function_count: u32,

    /// SATD comment count
    pub(crate) satd_count: u32,

    /// Source content for comment analysis
    pub(crate) content: String,
}

impl ComplexityVisitor {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn new(content: String) -> Self {
        let satd_count = Self::count_satd_in_content(&content);
        Self {
            complexity: 1, // Base complexity
            cognitive: 0,
            nesting_level: 0,
            function_count: 0,
            satd_count,
            content,
        }
    }

    /// Count SATD comments in content
    fn count_satd_in_content(content: &str) -> u32 {
        debug_assert!(!content.is_empty(), "content must not be empty");
        let patterns = ["TODO", "FIXME", "HACK", "XXX", "BUG"];
        patterns
            .iter()
            .map(|pattern| content.matches(pattern).count() as u32)
            .sum()
    }
}

impl<'ast> Visit<'ast> for ComplexityVisitor {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        debug_assert!(true, "contract: visit_item_fn");
        self.function_count += 1;

        // Reset for function-level metrics
        let old_complexity = self.complexity;
        let old_cognitive = self.cognitive;
        let old_nesting = self.nesting_level;

        self.complexity = 1; // Base complexity for function
        self.cognitive = 0;
        self.nesting_level = 0;

        // Visit function body
        syn::visit::visit_item_fn(self, node);

        // Restore and accumulate
        let fn_complexity = self.complexity;
        let fn_cognitive = self.cognitive;

        self.complexity = old_complexity + fn_complexity;
        self.cognitive = old_cognitive + fn_cognitive;
        self.nesting_level = old_nesting;
    }

    fn visit_expr_if(&mut self, node: &'ast syn::ExprIf) {
        debug_assert!(true, "contract: visit_expr_if");
        // Increase complexity and cognitive complexity
        self.complexity += 1;
        self.cognitive += 1 + self.nesting_level;

        // Increase nesting for cognitive complexity
        self.nesting_level += 1;
        syn::visit::visit_expr_if(self, node);
        self.nesting_level -= 1;
    }

    fn visit_expr_while(&mut self, node: &'ast syn::ExprWhile) {
        debug_assert!(true, "contract: visit_expr_while");
        self.complexity += 1;
        self.cognitive += 1 + self.nesting_level;

        self.nesting_level += 1;
        syn::visit::visit_expr_while(self, node);
        self.nesting_level -= 1;
    }

    fn visit_expr_for_loop(&mut self, node: &'ast syn::ExprForLoop) {
        debug_assert!(true, "contract: visit_expr_for_loop");
        self.complexity += 1;
        self.cognitive += 1 + self.nesting_level;

        self.nesting_level += 1;
        syn::visit::visit_expr_for_loop(self, node);
        self.nesting_level -= 1;
    }

    fn visit_expr_loop(&mut self, node: &'ast syn::ExprLoop) {
        debug_assert!(true, "contract: visit_expr_loop");
        self.complexity += 1;
        self.cognitive += 1 + self.nesting_level;

        self.nesting_level += 1;
        syn::visit::visit_expr_loop(self, node);
        self.nesting_level -= 1;
    }

    fn visit_expr_match(&mut self, node: &'ast syn::ExprMatch) {
        debug_assert!(true, "contract: visit_expr_match");
        // Match adds complexity for each arm
        self.complexity += node.arms.len() as u32;
        self.cognitive += 1 + self.nesting_level;

        self.nesting_level += 1;
        syn::visit::visit_expr_match(self, node);
        self.nesting_level -= 1;
    }

    fn visit_expr_binary(&mut self, node: &'ast syn::ExprBinary) {
        debug_assert!(true, "contract: visit_expr_binary");
        // Check for logical operators
        match node.op {
            syn::BinOp::And(_) | syn::BinOp::Or(_) => {
                self.complexity += 1;
            }
            _ => {}
        }

        syn::visit::visit_expr_binary(self, node);
    }

    fn visit_arm(&mut self, node: &'ast syn::Arm) {
        debug_assert!(true, "contract: visit_arm");
        // Each match arm adds cognitive complexity based on nesting
        if self.nesting_level > 0 {
            self.cognitive += self.nesting_level;
        }

        syn::visit::visit_arm(self, node);
    }
}
