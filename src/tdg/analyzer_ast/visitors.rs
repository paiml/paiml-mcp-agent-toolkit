
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

    fn visit_node(&mut self, node: &tree_sitter::Node) {
        match node.kind() {
            "function_definition" => self.visit_function_def(node),
            "class_definition" => self.visit_class_def(node),
            "import_statement" | "import_from_statement" => {
                self.import_count += 1;
            }
            "if_statement" | "while_statement" | "for_statement" | "match_statement" => {
                self.visit_nesting_branch(node);
            }
            "elif_clause" | "else_clause" | "except_clause" => {
                self.visit_flat_branch(node);
            }
            "try_statement" | "boolean_operator" | "comparison_operator" => {
                self.cyclomatic_complexity += 1;
                self.visit_children_recursive(node);
            }
            "call" => {
                self.external_calls += 1;
                self.visit_children_recursive(node);
            }
            _ => self.visit_children_recursive(node),
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn visit_function_def(&mut self, node: &tree_sitter::Node) {
        self.total_functions += 1;
        self.current_nesting_depth += 1;
        self.max_nesting_depth = self.max_nesting_depth.max(self.current_nesting_depth);

        if let Some(params) = node.child_by_field_name("parameters") {
            self.max_params = self.max_params.max(params.child_count());
        }

        self.check_python_docstring(node);

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "decorator" {
                self.decorator_count += 1;
            }
        }

        self.visit_children_recursive(node);
        self.current_nesting_depth -= 1;
    }

    #[allow(clippy::cast_possible_truncation)]
    fn check_python_docstring(&mut self, node: &tree_sitter::Node) {
        let body = match node.child_by_field_name("body") {
            Some(b) => b,
            None => return,
        };
        let first_child = match body.child(0) {
            Some(c) if c.kind() == "expression_statement" => c,
            _ => return,
        };
        let string_node = match first_child.child(0) {
            Some(s) if s.kind() == "string" => s,
            _ => return,
        };
        self.documented_functions += 1;
        let docstring_text = &self.source[string_node.byte_range()];
        self.docstring_lines += docstring_text.lines().count() as u32;
    }

    fn visit_class_def(&mut self, node: &tree_sitter::Node) {
        self.current_nesting_depth += 1;
        self.max_nesting_depth = self.max_nesting_depth.max(self.current_nesting_depth);

        if let Some(arg_list) = node.child_by_field_name("superclasses") {
            let arg_text = &self.source[arg_list.byte_range()];
            if arg_text.contains("metaclass") {
                self.metaclass_count += 1;
            }
        }

        self.visit_children_recursive(node);
        self.current_nesting_depth -= 1;
    }

    fn visit_nesting_branch(&mut self, node: &tree_sitter::Node) {
        self.cyclomatic_complexity += 1;
        self.cognitive_complexity += 1 + self.current_nesting_depth as u32;
        self.current_nesting_depth += 1;
        self.max_nesting_depth = self.max_nesting_depth.max(self.current_nesting_depth);
        self.visit_children_recursive(node);
        self.current_nesting_depth -= 1;
    }

    fn visit_flat_branch(&mut self, node: &tree_sitter::Node) {
        self.cyclomatic_complexity += 1;
        self.cognitive_complexity += 1 + self.current_nesting_depth as u32;
        self.visit_children_recursive(node);
    }

    fn visit_children_recursive(&mut self, node: &tree_sitter::Node) {
        for child in node.children(&mut node.walk()) {
            self.visit_node(&child);
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

#[cfg(feature = "lua-ast")]
struct LuaComplexityVisitor<'a> {
    source: &'a str,
    cyclomatic_complexity: u32,
    cognitive_complexity: u32,
    max_nesting_depth: usize,
    max_method_length: usize,
    max_params: usize,
    import_count: u32,
    external_calls: u32,
    documented_functions: u32,
    total_functions: u32,
    comment_lines: u32,
    total_lines: u32,
    current_nesting_depth: usize,
    /// Count of metatables set (Lua's OOP pattern)
    metatable_count: u32,
}

#[cfg(feature = "lua-ast")]
impl<'a> LuaComplexityVisitor<'a> {
    #[allow(clippy::cast_possible_truncation)]
    fn new(source: &'a str) -> Self {
        Self {
            source,
            cyclomatic_complexity: 1,
            cognitive_complexity: 0,
            max_nesting_depth: 0,
            max_method_length: 0,
            max_params: 0,
            import_count: 0,
            external_calls: 0,
            documented_functions: 0,
            total_functions: 0,
            comment_lines: 0,
            total_lines: source.lines().count() as u32,
            current_nesting_depth: 0,
            metatable_count: 0,
        }
    }

    fn analyze_tree(&mut self, tree: &tree_sitter::Tree) {
        // Count comment lines via simple scan (comments aren't always visited as nodes)
        for line in self.source.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("--") {
                self.comment_lines += 1;
            }
        }
        let root = tree.root_node();
        self.visit_node(&root);
    }

    fn visit_node(&mut self, node: &tree_sitter::Node) {
        match node.kind() {
            "function_declaration" | "function_definition" => self.visit_function_decl(node),
            "if_statement" | "for_statement" | "while_statement" | "repeat_statement" => {
                self.visit_nesting_control_flow(node);
            }
            "elseif_statement" => self.visit_flat_control_flow(node),
            "binary_expression" => self.visit_binary_expr(node),
            "function_call" => self.visit_function_call(node),
            _ => self.visit_children(node),
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn visit_function_decl(&mut self, node: &tree_sitter::Node) {
        self.total_functions += 1;
        self.current_nesting_depth += 1;
        self.max_nesting_depth = self.max_nesting_depth.max(self.current_nesting_depth);

        if let Some(params) = node.child_by_field_name("parameters") {
            self.max_params = self.max_params.max(params.named_child_count());
        }

        let fn_length = node.end_position().row.saturating_sub(node.start_position().row);
        self.max_method_length = self.max_method_length.max(fn_length);

        if node.prev_sibling().is_some_and(|s| s.kind() == "comment") {
            self.documented_functions += 1;
        }

        self.visit_children(node);
        self.current_nesting_depth -= 1;
    }

    #[allow(clippy::cast_possible_truncation)]
    fn visit_nesting_control_flow(&mut self, node: &tree_sitter::Node) {
        self.cyclomatic_complexity += 1;
        self.cognitive_complexity += 1 + self.current_nesting_depth as u32;
        self.current_nesting_depth += 1;
        self.max_nesting_depth = self.max_nesting_depth.max(self.current_nesting_depth);
        self.visit_children(node);
        self.current_nesting_depth -= 1;
    }

    #[allow(clippy::cast_possible_truncation)]
    fn visit_flat_control_flow(&mut self, node: &tree_sitter::Node) {
        self.cyclomatic_complexity += 1;
        self.cognitive_complexity += 1 + self.current_nesting_depth as u32;
        self.visit_children(node);
    }

    fn visit_binary_expr(&mut self, node: &tree_sitter::Node) {
        for child in node.children(&mut node.walk()) {
            if child.kind() == "and" || child.kind() == "or" {
                self.cyclomatic_complexity += 1;
            }
            self.visit_node(&child);
        }
    }

    fn visit_function_call(&mut self, node: &tree_sitter::Node) {
        self.external_calls += 1;
        let call_text = &self.source[node.byte_range()];
        if call_text.starts_with("require") {
            self.import_count += 1;
        }
        if call_text.starts_with("setmetatable") {
            self.metatable_count += 1;
        }
        self.visit_children(node);
    }

    fn visit_children(&mut self, node: &tree_sitter::Node) {
        for child in node.children(&mut node.walk()) {
            self.visit_node(&child);
        }
    }
}

// ===== Go Complexity Visitor =====

#[cfg(feature = "go-ast")]
struct GoComplexityVisitor<'a> {
    source: &'a str,
    cyclomatic_complexity: u32,
    cognitive_complexity: u32,
    max_nesting_depth: usize,
    max_method_length: usize,
    max_params: usize,
    import_count: u32,
    external_calls: u32,
    documented_functions: u32,
    total_functions: u32,
    comment_lines: u32,
    total_lines: u32,
    current_nesting_depth: usize,
    interface_count: u32,
}

#[cfg(feature = "go-ast")]
impl<'a> GoComplexityVisitor<'a> {
    #[allow(clippy::cast_possible_truncation)]
    fn new(source: &'a str) -> Self {
        let comment_lines = source
            .lines()
            .filter(|l| {
                let t = l.trim();
                t.starts_with("//") || t.starts_with("/*") || t.starts_with("*")
            })
            .count() as u32;
        Self {
            source,
            cyclomatic_complexity: 1,
            cognitive_complexity: 0,
            max_nesting_depth: 0,
            max_method_length: 0,
            max_params: 0,
            import_count: 0,
            external_calls: 0,
            documented_functions: 0,
            total_functions: 0,
            comment_lines,
            total_lines: source.lines().count() as u32,
            current_nesting_depth: 0,
            interface_count: 0,
        }
    }

    fn analyze_tree(&mut self, tree: &tree_sitter::Tree) {
        let root = tree.root_node();
        self.visit_node(&root);
    }

    fn visit_node(&mut self, node: &tree_sitter::Node) {
        match node.kind() {
            "function_declaration" | "method_declaration" | "func_literal" => {
                self.visit_function_decl(node);
            }
            "if_statement" | "for_statement" => self.visit_nesting_branch(node),
            "expression_switch_statement" | "type_switch_statement" | "select_statement" => {
                self.visit_nesting_branch(node);
            }
            "expression_case" | "type_case" | "default_case" | "communication_case" => {
                self.visit_flat_branch(node);
            }
            "binary_expression" => self.visit_binary_expr(node),
            "call_expression" => {
                self.external_calls += 1;
                self.visit_children_go(node);
            }
            "import_declaration" => self.visit_import_decl(node),
            "type_declaration" => self.visit_type_decl(node),
            _ => self.visit_children_go(node),
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn visit_function_decl(&mut self, node: &tree_sitter::Node) {
        self.total_functions += 1;
        self.current_nesting_depth += 1;
        self.max_nesting_depth = self.max_nesting_depth.max(self.current_nesting_depth);

        if let Some(params) = node.child_by_field_name("parameters") {
            self.max_params = self.max_params.max(params.named_child_count());
        }

        let fn_length = node.end_position().row.saturating_sub(node.start_position().row);
        self.max_method_length = self.max_method_length.max(fn_length);

        if node.prev_sibling().is_some_and(|s| s.kind() == "comment") {
            self.documented_functions += 1;
        }

        self.visit_children_go(node);
        self.current_nesting_depth -= 1;
    }

    fn visit_nesting_branch(&mut self, node: &tree_sitter::Node) {
        self.cyclomatic_complexity += 1;
        self.cognitive_complexity += 1 + self.current_nesting_depth as u32;
        self.current_nesting_depth += 1;
        self.max_nesting_depth = self.max_nesting_depth.max(self.current_nesting_depth);
        self.visit_children_go(node);
        self.current_nesting_depth -= 1;
    }

    fn visit_flat_branch(&mut self, node: &tree_sitter::Node) {
        self.cyclomatic_complexity += 1;
        self.cognitive_complexity += 1;
        self.visit_children_go(node);
    }

    fn visit_binary_expr(&mut self, node: &tree_sitter::Node) {
        if let Some(op) = node.child_by_field_name("operator") {
            let op_text = &self.source[op.byte_range()];
            if op_text == "&&" || op_text == "||" {
                self.cyclomatic_complexity += 1;
            }
        }
        self.visit_children_go(node);
    }

    fn visit_import_decl(&mut self, node: &tree_sitter::Node) {
        // Count each import spec inside the declaration
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "import_spec" || child.kind() == "import_spec_list" {
                self.import_count += 1;
            }
        }
        if self.import_count == 0 {
            self.import_count = 1; // single-line import
        }
    }

    fn visit_type_decl(&mut self, node: &tree_sitter::Node) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "type_spec" {
                // Check if it's an interface type
                let mut spec_cursor = child.walk();
                for spec_child in child.children(&mut spec_cursor) {
                    if spec_child.kind() == "interface_type" {
                        self.interface_count += 1;
                    }
                }
            }
        }
    }

    fn visit_children_go(&mut self, node: &tree_sitter::Node) {
        for child in node.children(&mut node.walk()) {
            self.visit_node(&child);
        }
    }
}

