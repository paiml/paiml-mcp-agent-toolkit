
#[cfg(feature = "rust-ast")]
impl RustComplexityVisitor {
    fn new() -> Self {
        Self {
            cyclomatic_complexity: 1,
            cognitive_complexity: 0,
            max_nesting_depth: 0,
            max_method_length: 0,
            max_params: 0,
            generic_count: 0,
            abstraction_levels: 0,
            import_count: 0,
            external_calls: 0,
            interface_implementations: 0,
            documented_items: 0,
            total_public_items: 0,
            comment_lines: 0,
            total_lines: 0,
            current_depth: 0,
        }
    }
}

#[cfg(feature = "rust-ast")]
impl<'ast> syn::visit::Visit<'ast> for RustComplexityVisitor {
    #[allow(clippy::cast_possible_truncation)]
    fn visit_expr_if(&mut self, node: &'ast syn::ExprIf) {
        self.cyclomatic_complexity += 1;
        self.cognitive_complexity += 1 + self.current_depth as u32;
        self.current_depth += 1;
        self.max_nesting_depth = self.max_nesting_depth.max(self.current_depth);
        syn::visit::visit_expr_if(self, node);
        self.current_depth -= 1;
    }

    #[allow(clippy::cast_possible_truncation)]
    fn visit_expr_while(&mut self, node: &'ast syn::ExprWhile) {
        self.cyclomatic_complexity += 1;
        self.cognitive_complexity += 1 + self.current_depth as u32;
        self.current_depth += 1;
        self.max_nesting_depth = self.max_nesting_depth.max(self.current_depth);
        syn::visit::visit_expr_while(self, node);
        self.current_depth -= 1;
    }

    #[allow(clippy::cast_possible_truncation)]
    fn visit_expr_for_loop(&mut self, node: &'ast syn::ExprForLoop) {
        self.cyclomatic_complexity += 1;
        self.cognitive_complexity += 1 + self.current_depth as u32;
        self.current_depth += 1;
        self.max_nesting_depth = self.max_nesting_depth.max(self.current_depth);
        syn::visit::visit_expr_for_loop(self, node);
        self.current_depth -= 1;
    }

    #[allow(clippy::cast_possible_truncation)]
    fn visit_expr_match(&mut self, node: &'ast syn::ExprMatch) {
        self.cyclomatic_complexity += node.arms.len() as u32;
        self.cognitive_complexity += 1 + self.current_depth as u32;
        self.current_depth += 1;
        self.max_nesting_depth = self.max_nesting_depth.max(self.current_depth);
        syn::visit::visit_expr_match(self, node);
        self.current_depth -= 1;
    }

    #[allow(clippy::cast_possible_truncation)]
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        // Count parameters
        let param_count = node.sig.inputs.len();
        self.max_params = self.max_params.max(param_count);

        // Count generics
        self.generic_count += node.sig.generics.params.len() as u32;

        // Check if documented
        if !node.attrs.is_empty() {
            for attr in &node.attrs {
                if attr.path().is_ident("doc") {
                    self.documented_items += 1;
                    break;
                }
            }
        }

        // Count as public item if applicable
        if matches!(node.vis, syn::Visibility::Public(_)) {
            self.total_public_items += 1;
        }

        syn::visit::visit_item_fn(self, node);
    }

    fn visit_use_tree(&mut self, _node: &'ast syn::UseTree) {
        self.import_count += 1;
    }

    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        if node.trait_.is_some() {
            self.interface_implementations += 1;
        }
        syn::visit::visit_item_impl(self, node);
    }
}

#[cfg(feature = "python-ast")]
struct PythonComplexityVisitor<'a> {
    source: &'a str,
    cyclomatic_complexity: u32,
    cognitive_complexity: u32,
    max_nesting_depth: usize,
    max_method_length: usize,
    max_params: usize,
    decorator_count: u32,
    metaclass_count: u32,
    import_count: u32,
    external_calls: u32,
    documented_functions: u32,
    total_functions: u32,
    docstring_lines: u32,
    total_lines: u32,
    current_nesting_depth: usize,
}

#[cfg(feature = "python-ast")]
impl<'a> PythonComplexityVisitor<'a> {
    #[allow(clippy::cast_possible_truncation)]
    fn new(source: &'a str) -> Self {
        Self {
            source,
            cyclomatic_complexity: 1,
            cognitive_complexity: 0,
            max_nesting_depth: 0,
            max_method_length: 0,
            max_params: 0,
            decorator_count: 0,
            metaclass_count: 0,
            import_count: 0,
            external_calls: 0,
            documented_functions: 0,
            total_functions: 0,
            docstring_lines: 0,
            total_lines: source.lines().count() as u32,
            current_nesting_depth: 0,
        }
    }

    fn analyze_tree(&mut self, tree: &tree_sitter::Tree) {
        let root = tree.root_node();
        self.visit_node(&root);
    }

    #[allow(clippy::cast_possible_truncation)]
    fn visit_node(&mut self, node: &tree_sitter::Node) {
        match node.kind() {
            "function_definition" => {
                self.total_functions += 1;
                self.current_nesting_depth += 1;
                if self.current_nesting_depth > self.max_nesting_depth {
                    self.max_nesting_depth = self.current_nesting_depth;
                }

                // Count parameters
                if let Some(params) = node.child_by_field_name("parameters") {
                    let param_count = params.child_count();
                    if param_count > self.max_params {
                        self.max_params = param_count;
                    }
                }

                // Check for docstring
                if let Some(body) = node.child_by_field_name("body") {
                    if let Some(first_child) = body.child(0) {
                        if first_child.kind() == "expression_statement" {
                            if let Some(string_node) = first_child.child(0) {
                                if string_node.kind() == "string" {
                                    self.documented_functions += 1;
                                    // Count docstring lines
                                    let docstring_text = &self.source[string_node.byte_range()];
                                    self.docstring_lines += docstring_text.lines().count() as u32;
                                }
                            }
                        }
                    }
                }

                // Count decorators
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "decorator" {
                        self.decorator_count += 1;
                    }
                }

                // Visit function body
                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child);
                }

                self.current_nesting_depth -= 1;
            }
            "class_definition" => {
                self.current_nesting_depth += 1;
                if self.current_nesting_depth > self.max_nesting_depth {
                    self.max_nesting_depth = self.current_nesting_depth;
                }

                // Check for metaclass
                if let Some(arg_list) = node.child_by_field_name("superclasses") {
                    let arg_text = &self.source[arg_list.byte_range()];
                    if arg_text.contains("metaclass") {
                        self.metaclass_count += 1;
                    }
                }

                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child);
                }

                self.current_nesting_depth -= 1;
            }
            "import_statement" | "import_from_statement" => {
                self.import_count += 1;
            }
            "if_statement" | "while_statement" | "for_statement" | "match_statement" => {
                self.cyclomatic_complexity += 1;
                self.cognitive_complexity += 1 + self.current_nesting_depth as u32;

                self.current_nesting_depth += 1;
                if self.current_nesting_depth > self.max_nesting_depth {
                    self.max_nesting_depth = self.current_nesting_depth;
                }

                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child);
                }

                self.current_nesting_depth -= 1;
            }
            "elif_clause" | "else_clause" | "except_clause" => {
                self.cyclomatic_complexity += 1;
                self.cognitive_complexity += 1 + self.current_nesting_depth as u32;

                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child);
                }
            }
            "try_statement" => {
                self.cyclomatic_complexity += 1;

                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child);
                }
            }
            "boolean_operator" | "comparison_operator" => {
                // Logical operators add to complexity
                self.cyclomatic_complexity += 1;

                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child);
                }
            }
            "call" => {
                // Count external calls (simplified - counts all calls)
                self.external_calls += 1;

                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child);
                }
            }
            _ => {
                // Visit children for other node types
                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child);
                }
            }
        }
    }
}

#[cfg(feature = "typescript-ast")]
struct JavaScriptComplexityVisitor {
    cyclomatic_complexity: u32,
    cognitive_complexity: u32,
    max_nesting_depth: usize,
    max_function_length: usize,
    max_params: usize,
    async_count: u32,
    callback_depth: u32,
    import_count: u32,
    external_calls: u32,
    class_count: u32,
    jsdoc_count: u32,
    function_count: u32,
    comment_lines: u32,
    total_lines: u32,
}

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
        // In swc 15.x, visit methods automatically recurse - no need to call explicitly
    }

    fn visit_while_stmt(&mut self, _node: &swc_ecma_ast::WhileStmt) {
        self.cyclomatic_complexity += 1;
        self.cognitive_complexity += 1;
        // In swc 15.x, visit methods automatically recurse - no need to call explicitly
    }

    fn visit_for_stmt(&mut self, _node: &swc_ecma_ast::ForStmt) {
        self.cyclomatic_complexity += 1;
        self.cognitive_complexity += 1;
        // In swc 15.x, visit methods automatically recurse - no need to call explicitly
    }

    #[allow(clippy::cast_possible_truncation)]
    fn visit_switch_stmt(&mut self, node: &swc_ecma_ast::SwitchStmt) {
        self.cyclomatic_complexity += node.cases.len() as u32;
        self.cognitive_complexity += 1;
        // In swc 15.x, visit methods automatically recurse - no need to call explicitly
    }

    fn visit_function(&mut self, node: &swc_ecma_ast::Function) {
        self.function_count += 1;
        self.max_params = self.max_params.max(node.params.len());

        if node.is_async {
            self.async_count += 1;
        }

        // In swc 15.x, visit methods automatically recurse - no need to call explicitly
    }

    fn visit_import_decl(&mut self, _node: &swc_ecma_ast::ImportDecl) {
        self.import_count += 1;
    }

    fn visit_class_decl(&mut self, _node: &swc_ecma_ast::ClassDecl) {
        self.class_count += 1;
    }
}

