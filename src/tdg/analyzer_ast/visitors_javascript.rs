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
        self.cyclomatic_complexity += 1;
        self.cognitive_complexity += 1;
    }

    fn visit_while_stmt(&mut self, _node: &swc_ecma_ast::WhileStmt) {
        self.cyclomatic_complexity += 1;
        self.cognitive_complexity += 1;
    }

    fn visit_for_stmt(&mut self, _node: &swc_ecma_ast::ForStmt) {
        self.cyclomatic_complexity += 1;
        self.cognitive_complexity += 1;
    }

    #[allow(clippy::cast_possible_truncation)]
    fn visit_switch_stmt(&mut self, node: &swc_ecma_ast::SwitchStmt) {
        self.cyclomatic_complexity += node.cases.len() as u32;
        self.cognitive_complexity += 1;
    }

    fn visit_function(&mut self, node: &swc_ecma_ast::Function) {
        self.function_count += 1;
        self.max_params = self.max_params.max(node.params.len());
        if node.is_async {
            self.async_count += 1;
        }
    }

    fn visit_import_decl(&mut self, _node: &swc_ecma_ast::ImportDecl) {
        self.import_count += 1;
    }

    fn visit_class_decl(&mut self, _node: &swc_ecma_ast::ClassDecl) {
        self.class_count += 1;
    }
}
