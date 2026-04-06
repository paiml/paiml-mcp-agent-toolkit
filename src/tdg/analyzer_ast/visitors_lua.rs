// LuaComplexityVisitor implementation methods
// Included from visitors.rs — no `use` imports or `#!` inner attributes

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
