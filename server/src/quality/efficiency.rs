use syn::{self, visit::Visit};

pub struct EfficiencyAnalyzer {
    _max_loop_depth: u32,
    _recursive_calls: u32,
}

impl Default for EfficiencyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl EfficiencyAnalyzer {
    pub fn new() -> Self {
        Self {
            _max_loop_depth: 0,
            _recursive_calls: 0,
        }
    }

    pub fn analyze(&self, ast: &syn::File) -> String {
        let mut visitor = EfficiencyVisitor {
            current_loop_depth: 0,
            max_loop_depth: 0,
            has_recursion: false,
        };
        visitor.visit_file(ast);
        visitor.compute_complexity()
    }

    pub fn analyze_string(&self, code: &str) -> Result<EfficiencyResult, syn::Error> {
        let ast = syn::parse_file(code)?;
        Ok(EfficiencyResult {
            time_complexity: self.analyze(&ast),
            space_complexity: self.analyze_space(&ast),
        })
    }

    fn analyze_space(&self, ast: &syn::File) -> String {
        let mut visitor = SpaceComplexityVisitor {
            allocations: 0,
            recursive_depth: 0,
        };
        visitor.visit_file(ast);
        visitor.compute_space_complexity()
    }
}

pub struct EfficiencyResult {
    pub time_complexity: String,
    pub space_complexity: String,
}

struct EfficiencyVisitor {
    current_loop_depth: u32,
    max_loop_depth: u32,
    has_recursion: bool,
}

impl EfficiencyVisitor {
    fn compute_complexity(&self) -> String {
        match self.max_loop_depth {
            0 => "O(1)".to_string(),
            1 => {
                if self.has_recursion {
                    "O(n log n)".to_string() // Assume divide-and-conquer
                } else {
                    "O(n)".to_string()
                }
            }
            2 => "O(n^2)".to_string(),
            3 => "O(n^3)".to_string(),
            _ => format!("O(n^{})", self.max_loop_depth),
        }
    }
}

impl<'ast> Visit<'ast> for EfficiencyVisitor {
    fn visit_expr_for_loop(&mut self, node: &'ast syn::ExprForLoop) {
        self.current_loop_depth += 1;
        if self.current_loop_depth > self.max_loop_depth {
            self.max_loop_depth = self.current_loop_depth;
        }
        syn::visit::visit_expr_for_loop(self, node);
        self.current_loop_depth -= 1;
    }

    fn visit_expr_while(&mut self, node: &'ast syn::ExprWhile) {
        self.current_loop_depth += 1;
        if self.current_loop_depth > self.max_loop_depth {
            self.max_loop_depth = self.current_loop_depth;
        }
        syn::visit::visit_expr_while(self, node);
        self.current_loop_depth -= 1;
    }

    fn visit_expr_loop(&mut self, node: &'ast syn::ExprLoop) {
        self.current_loop_depth += 1;
        if self.current_loop_depth > self.max_loop_depth {
            self.max_loop_depth = self.current_loop_depth;
        }
        syn::visit::visit_expr_loop(self, node);
        self.current_loop_depth -= 1;
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        // Simple recursion detection (would need more sophisticated analysis in production)
        if let syn::Expr::Path(_path) = &*node.func {
            // Check if it's potentially a recursive call
            // This is simplified - real implementation would track function names
            self.has_recursion = self.current_loop_depth == 0;
        }
        syn::visit::visit_expr_call(self, node);
    }
}

struct SpaceComplexityVisitor {
    allocations: u32,
    recursive_depth: u32,
}

impl SpaceComplexityVisitor {
    fn compute_space_complexity(&self) -> String {
        if self.recursive_depth > 0 {
            "O(n)".to_string() // Recursive calls use stack space
        } else if self.allocations > 0 {
            "O(n)".to_string() // Assuming dynamic allocations scale with input
        } else {
            "O(1)".to_string() // Constant space
        }
    }
}

impl<'ast> Visit<'ast> for SpaceComplexityVisitor {
    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        // Check for allocation functions
        if let syn::Expr::Path(path) = &*node.func {
            if let Some(ident) = path.path.get_ident() {
                let name = ident.to_string();
                if name.contains("vec") || name.contains("Vec") || name.contains("alloc") {
                    self.allocations += 1;
                }
            }
        }
        syn::visit::visit_expr_call(self, node);
    }

    fn visit_local(&mut self, node: &'ast syn::Local) {
        // Check for vector/array declarations
        if let syn::Pat::Type(_pat_type) = &node.pat {
            // Simplified check for dynamic allocations
            self.allocations += 1;
        }
        syn::visit::visit_local(self, node);
    }
}
