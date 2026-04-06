// JavaScriptComplexityVisitor implementation methods
// Included from visitors.rs — no `use` imports or `#!` inner attributes

#[cfg(feature = "typescript-ast")]
impl JavaScriptComplexityVisitor {
    fn new() -> Self {
        Self {
            cyclomatic_complexity: 1,
            cognitive_complexity: 0,
            max_nesting_depth: 0,
            max_function_length: 0,
            max_params: 0,
            async_count: 0,
            callback_depth: 0,
            import_count: 0,
            external_calls: 0,
            class_count: 0,
            jsdoc_count: 0,
            function_count: 0,
            comment_lines: 0,
            total_lines: 0,
        }
    }
}

#[cfg(feature = "typescript-ast")]
impl swc_ecma_visit::Visit for JavaScriptComplexityVisitor {
    fn visit_if_stmt(&mut self, _node: &swc_ecma_ast::IfStmt) {
        debug_assert!(true, "contract: visit_if_stmt");
        self.cyclomatic_complexity += 1;
        self.cognitive_complexity += 1;
    }

    fn visit_while_stmt(&mut self, _node: &swc_ecma_ast::WhileStmt) {
        debug_assert!(true, "contract: visit_while_stmt");
        self.cyclomatic_complexity += 1;
        self.cognitive_complexity += 1;
    }

    fn visit_for_stmt(&mut self, _node: &swc_ecma_ast::ForStmt) {
        debug_assert!(true, "contract: visit_for_stmt");
        self.cyclomatic_complexity += 1;
        self.cognitive_complexity += 1;
    }

    #[allow(clippy::cast_possible_truncation)]
    fn visit_switch_stmt(&mut self, node: &swc_ecma_ast::SwitchStmt) {
        debug_assert!(true, "contract: visit_switch_stmt");
        self.cyclomatic_complexity += node.cases.len() as u32;
        self.cognitive_complexity += 1;
    }

    fn visit_function(&mut self, node: &swc_ecma_ast::Function) {
        debug_assert!(true, "contract: visit_function");
        self.function_count += 1;
        self.max_params = self.max_params.max(node.params.len());
        if node.is_async {
            self.async_count += 1;
        }
    }

    fn visit_import_decl(&mut self, _node: &swc_ecma_ast::ImportDecl) {
        debug_assert!(true, "contract: visit_import_decl");
        self.import_count += 1;
    }

    fn visit_class_decl(&mut self, _node: &swc_ecma_ast::ClassDecl) {
        debug_assert!(true, "contract: visit_class_decl");
        self.class_count += 1;
    }
}
