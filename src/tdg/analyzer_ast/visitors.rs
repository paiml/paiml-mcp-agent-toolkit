// ===== Rust Complexity Visitor =====

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
        let param_count = node.sig.inputs.len();
        self.max_params = self.max_params.max(param_count);
        self.generic_count += node.sig.generics.params.len() as u32;

        if !node.attrs.is_empty() {
            for attr in &node.attrs {
                if attr.path().is_ident("doc") {
                    self.documented_items += 1;
                    break;
                }
            }
        }

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

// ===== Python Complexity Visitor =====

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

// ===== JavaScript/TypeScript Complexity Visitor =====

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

// ===== Lua Complexity Visitor =====

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

// ===== Include visitor implementation methods =====
include!("visitors_python.rs");
include!("visitors_javascript.rs");
include!("visitors_lua.rs");
include!("visitors_go.rs");
