// Included from accurate_complexity_analyzer.rs — NO use imports or #! attributes

/// AST visitor for calculating complexity metrics
struct ComplexityVisitor {
    cyclomatic: u32,
    cognitive: u32,
    nesting_level: u32,
    function_name: Option<String>,
}

impl ComplexityVisitor {
    fn new() -> Self {
        Self {
            cyclomatic: 1, // Base complexity
            cognitive: 0,
            nesting_level: 0,
            function_name: None,
        }
    }

    fn with_function_name(mut self, name: String) -> Self {
        self.function_name = Some(name);
        self
    }

    fn add_cyclomatic(&mut self, amount: u32) {
        self.cyclomatic += amount;
    }

    fn add_cognitive(&mut self, base: u32) {
        // Add base cognitive complexity plus nesting penalty
        // According to SonarSource spec, nesting adds extra cognitive load
        self.cognitive += base + self.nesting_level;
    }
}

impl ComplexityVisitor {
    fn visit_if_expr(&mut self, if_expr: &syn::ExprIf) {
        self.add_cyclomatic(1);
        self.add_cognitive(1);

        self.visit_expr(&if_expr.cond);
        self.nesting_level += 1;
        for stmt in &if_expr.then_branch.stmts {
            self.visit_stmt(stmt);
        }
        self.nesting_level -= 1;

        if let Some((_, else_expr)) = &if_expr.else_branch {
            self.visit_else_branch(else_expr);
        }
    }

    fn visit_else_branch(&mut self, else_expr: &Expr) {
        match else_expr {
            Expr::If(_) => self.visit_expr(else_expr),
            _ => {
                self.nesting_level += 1;
                self.visit_expr(else_expr);
                self.nesting_level -= 1;
            }
        }
    }

    fn visit_match_expr(&mut self, match_expr: &syn::ExprMatch) {
        self.add_cyclomatic(1);
        self.add_cognitive(1);
        self.visit_expr(&match_expr.expr);

        for arm in &match_expr.arms {
            if let Some((_, guard)) = &arm.guard {
                self.add_cyclomatic(1);
                self.add_cognitive(1);
                self.visit_expr(guard);
            }
        }

        self.nesting_level += 1;
        for arm in &match_expr.arms {
            self.visit_expr(&arm.body);
        }
        self.nesting_level -= 1;
    }

    fn visit_loop_body_stmts(&mut self, stmts: &[Stmt]) {
        self.add_cyclomatic(1);
        self.add_cognitive(1);
        self.nesting_level += 1;
        for stmt in stmts {
            self.visit_stmt(stmt);
        }
        self.nesting_level -= 1;
    }

    fn check_recursive_call(&mut self, call: &syn::ExprCall) {
        let is_recursive = matches!(
            call.func.as_ref(),
            Expr::Path(path) if path.path.segments.last()
                .map(|seg| self.function_name.as_ref().is_some_and(|name| seg.ident == name))
                .unwrap_or(false)
        );
        if is_recursive {
            self.add_cognitive(1);
        }
    }
}

impl<'ast> Visit<'ast> for ComplexityVisitor {
    fn visit_expr(&mut self, expr: &'ast Expr) {
        match expr {
            Expr::If(if_expr) => self.visit_if_expr(if_expr),
            Expr::Match(match_expr) => self.visit_match_expr(match_expr),
            Expr::While(w) => {
                self.visit_expr(&w.cond);
                self.visit_loop_body_stmts(&w.body.stmts);
            }
            Expr::ForLoop(f) => {
                self.visit_expr(&f.expr);
                self.visit_loop_body_stmts(&f.body.stmts);
            }
            Expr::Loop(l) => self.visit_loop_body_stmts(&l.body.stmts),
            Expr::Binary(bin) => {
                use syn::BinOp;
                if matches!(bin.op, BinOp::And(_) | BinOp::Or(_)) {
                    self.add_cyclomatic(1);
                    self.add_cognitive(1);
                }
                syn::visit::visit_expr(self, expr);
            }
            Expr::Try(_) => {
                self.add_cyclomatic(1);
                self.add_cognitive(1);
                syn::visit::visit_expr(self, expr);
            }
            Expr::Break(_) | Expr::Continue(_) => {
                self.add_cognitive(1);
                syn::visit::visit_expr(self, expr);
            }
            Expr::Return(_) => {
                if self.nesting_level > 0 {
                    self.add_cognitive(1);
                }
                syn::visit::visit_expr(self, expr);
            }
            Expr::Call(call) => {
                self.check_recursive_call(call);
                syn::visit::visit_expr(self, expr);
            }
            _ => syn::visit::visit_expr(self, expr),
        }
    }

    fn visit_stmt(&mut self, stmt: &'ast Stmt) {
        syn::visit::visit_stmt(self, stmt);
    }
}
