use std::collections::HashMap;
use syn::{self, visit::Visit};

pub struct ComplexityAnalyzer {
    current_complexity: u32,
    cognitive_complexity: u32,
    nesting_depth: u32,
    max_nesting: u32,
}

impl ComplexityAnalyzer {
    pub fn new() -> Self {
        Self {
            current_complexity: 1, // Base complexity
            cognitive_complexity: 0,
            nesting_depth: 0,
            max_nesting: 0,
        }
    }

    pub fn calculate_cyclomatic(&self, ast: &syn::File) -> u32 {
        let mut visitor = ComplexityVisitor {
            complexity: 1,
            nesting_depth: 0,
        };
        visitor.visit_file(ast);
        visitor.complexity
    }

    pub fn calculate_cognitive(&self, ast: &syn::File) -> u32 {
        let mut visitor = CognitiveComplexityVisitor {
            complexity: 0,
            nesting_depth: 0,
        };
        visitor.visit_file(ast);
        visitor.complexity
    }

    pub fn analyze_string(&self, code: &str) -> Result<ComplexityMetrics, syn::Error> {
        let ast = syn::parse_file(code)?;
        Ok(ComplexityMetrics {
            cyclomatic: self.calculate_cyclomatic(&ast),
            cognitive: self.calculate_cognitive(&ast),
        })
    }

    pub fn calculate_shannon_entropy(&self, code: &str) -> f64 {
        let mut char_counts = HashMap::new();
        let total = code.len() as f64;

        for ch in code.chars() {
            *char_counts.entry(ch).or_insert(0) += 1;
        }

        let mut entropy = 0.0;
        for count in char_counts.values() {
            let probability = *count as f64 / total;
            if probability > 0.0 {
                entropy -= probability * probability.log2();
            }
        }

        entropy
    }
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct ComplexityMetrics {
    pub cyclomatic: u32,
    pub cognitive: u32,
}

struct ComplexityVisitor {
    complexity: u32,
    nesting_depth: u32,
}

impl<'ast> Visit<'ast> for ComplexityVisitor {
    fn visit_expr_if(&mut self, node: &'ast syn::ExprIf) {
        self.complexity += 1; // Each if adds a path
        self.nesting_depth += 1;
        syn::visit::visit_expr_if(self, node);
        self.nesting_depth -= 1;
    }

    fn visit_expr_match(&mut self, node: &'ast syn::ExprMatch) {
        // Each arm except the first adds a path
        if node.arms.len() > 1 {
            self.complexity += (node.arms.len() - 1) as u32;
        }
        self.nesting_depth += 1;
        syn::visit::visit_expr_match(self, node);
        self.nesting_depth -= 1;
    }

    fn visit_expr_for_loop(&mut self, node: &'ast syn::ExprForLoop) {
        self.complexity += 1;
        self.nesting_depth += 1;
        syn::visit::visit_expr_for_loop(self, node);
        self.nesting_depth -= 1;
    }

    fn visit_expr_while(&mut self, node: &'ast syn::ExprWhile) {
        self.complexity += 1;
        self.nesting_depth += 1;
        syn::visit::visit_expr_while(self, node);
        self.nesting_depth -= 1;
    }

    fn visit_expr_loop(&mut self, node: &'ast syn::ExprLoop) {
        self.complexity += 1;
        self.nesting_depth += 1;
        syn::visit::visit_expr_loop(self, node);
        self.nesting_depth -= 1;
    }

    fn visit_expr_binary(&mut self, node: &'ast syn::ExprBinary) {
        use syn::BinOp;
        match node.op {
            BinOp::And(_) | BinOp::Or(_) => {
                self.complexity += 1;
            }
            _ => {}
        }
        syn::visit::visit_expr_binary(self, node);
    }
}

struct CognitiveComplexityVisitor {
    complexity: u32,
    nesting_depth: u32,
}

impl<'ast> Visit<'ast> for CognitiveComplexityVisitor {
    fn visit_expr_if(&mut self, node: &'ast syn::ExprIf) {
        self.complexity += 1 + self.nesting_depth; // Nesting adds cognitive load
        self.nesting_depth += 1;
        syn::visit::visit_expr_if(self, node);
        self.nesting_depth -= 1;
    }

    fn visit_expr_match(&mut self, node: &'ast syn::ExprMatch) {
        self.complexity += 1 + self.nesting_depth;
        self.nesting_depth += 1;
        syn::visit::visit_expr_match(self, node);
        self.nesting_depth -= 1;
    }

    fn visit_expr_for_loop(&mut self, node: &'ast syn::ExprForLoop) {
        self.complexity += 1 + self.nesting_depth;
        self.nesting_depth += 1;
        syn::visit::visit_expr_for_loop(self, node);
        self.nesting_depth -= 1;
    }

    fn visit_expr_while(&mut self, node: &'ast syn::ExprWhile) {
        self.complexity += 1 + self.nesting_depth;
        self.nesting_depth += 1;
        syn::visit::visit_expr_while(self, node);
        self.nesting_depth -= 1;
    }

    fn visit_expr_break(&mut self, _node: &'ast syn::ExprBreak) {
        self.complexity += 1; // Breaks add cognitive complexity
    }

    fn visit_expr_continue(&mut self, _node: &'ast syn::ExprContinue) {
        self.complexity += 1; // Continues add cognitive complexity
    }
}
