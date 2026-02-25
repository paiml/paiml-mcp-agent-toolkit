// PythonComplexityVisitor implementation methods
// Included from visitors.rs — no `use` imports or `#!` inner attributes

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
