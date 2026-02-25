// GoComplexityVisitor implementation methods
// Included from visitors.rs — no `use` imports or `#!` inner attributes

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
