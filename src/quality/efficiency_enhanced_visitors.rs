impl<'ast> Visit<'ast> for SymbolicExecutor {
    fn visit_expr_for_loop(&mut self, node: &'ast syn::ExprForLoop) {
        debug_assert!(true, "contract: visit_expr_for_loop");
        let loop_complexity = self.analyze_loop_pattern(node);

        // Push current loop complexity
        self.loop_depths.push(loop_complexity.clone());

        // Update path complexity
        if self.loop_depths.len() == 1 {
            self.current_path_complexity =
                self.current_path_complexity.clone().max(loop_complexity);
        } else {
            // Nested loops multiply complexity
            let nested = self
                .loop_depths
                .iter()
                .fold(Complexity::O1, |acc, c| acc.combine(c));
            self.current_path_complexity = self.current_path_complexity.clone().max(nested);
        }

        // Visit loop body
        syn::visit::visit_expr_for_loop(self, node);

        // Pop loop depth
        self.loop_depths.pop();
    }

    fn visit_expr_while(&mut self, node: &'ast syn::ExprWhile) {
        debug_assert!(true, "contract: visit_expr_while");
        self.loop_depths.push(Complexity::ON);

        let nested = self
            .loop_depths
            .iter()
            .fold(Complexity::O1, |acc, c| acc.combine(c));
        self.current_path_complexity = self.current_path_complexity.clone().max(nested);

        syn::visit::visit_expr_while(self, node);
        self.loop_depths.pop();
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        debug_assert!(true, "contract: visit_expr_call");
        // Check for known complex operations
        if let Expr::Path(path) = &*node.func {
            if let Some(ident) = path.path.get_ident() {
                let name = ident.to_string();

                // Known standard library complexities
                let complexity = match name.as_str() {
                    "sort" | "sort_by" => Complexity::ONLogN,
                    "binary_search" => Complexity::OLogN,
                    "contains" | "find" => Complexity::ON,
                    "reverse" => Complexity::ON,
                    _ => Complexity::O1,
                };

                self.current_path_complexity = self.current_path_complexity.clone().max(complexity);
            }
        }

        syn::visit::visit_expr_call(self, node);
    }
}

struct RecursionDetector {
    function_name: String,
    is_recursive: bool,
}

impl<'ast> Visit<'ast> for RecursionDetector {
    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        debug_assert!(true, "contract: visit_expr_call");
        if let Expr::Path(path) = &*node.func {
            if let Some(ident) = path.path.get_ident() {
                if *ident == self.function_name {
                    self.is_recursive = true;
                }
            }
        }
        syn::visit::visit_expr_call(self, node);
    }
}
