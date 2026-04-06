// Enhanced TypeScript visitor inherent methods: construction, item extraction, and helper utilities

#[cfg(feature = "typescript-ast")]
impl EnhancedTypeScriptVisitor {
    /// Creates a new enhanced visitor for a given file
    #[must_use]
    pub fn new(file_path: &Path) -> Self {
        debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
        Self {
            items: Vec::new(),
            file_path: file_path.to_path_buf(),
            module_path: vec![],
            class_stack: vec![],
        }
    }

    /// Extracts all AST items with real source information
    #[must_use]
    pub fn extract_items(mut self, module: &Module) -> Vec<AstItem> {
        self.visit_module(module);
        self.items
    }

    /// Gets line number from span (simplified - would need source map for accuracy)
    fn get_line(&self, span: Span) -> usize {
        // In production, this would use a source map
        // For now, use span's start position as approximation
        let start = span.lo.0;
        ((start / 100) + 1) as usize // Rough line approximation
    }

    /// Creates a qualified name for the current context
    fn get_qualified_name(&self, name: &str) -> String {
        let mut parts = Vec::new();

        // Add module path
        parts.extend(self.module_path.iter().cloned());

        // Add class context if we're inside a class
        if let Some(class_name) = self.class_stack.last() {
            parts.push(class_name.clone());
        }

        parts.push(name.to_string());
        parts.join("::")
    }

    /// Checks if function is async
    fn is_async_function(&self, func: &Function) -> bool {
        func.is_async
    }

    /// Extracts functions from expressions (handles named function expressions)
    fn extract_function_from_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Fn(fn_expr) => {
                if let Some(ident) = &fn_expr.ident {
                    let name = self.get_qualified_name(ident.sym.as_ref());
                    let is_async = fn_expr.function.is_async;
                    let line = self.get_line(fn_expr.function.span);

                    self.items.push(AstItem::Function {
                        name,
                        visibility: "public".to_string(),
                        is_async,
                        line,
                    });
                }
            }
            Expr::Arrow(arrow_expr) => {
                // Arrow functions are typically anonymous, but we could handle special cases
                let name = self.get_qualified_name("anonymous");
                let is_async = arrow_expr.is_async;
                let line = self.get_line(arrow_expr.span);

                self.items.push(AstItem::Function {
                    name,
                    visibility: "public".to_string(),
                    is_async,
                    line,
                });
            }
            _ => {}
        }
    }

    /// Extracts methods from object literals (complexity <=10)
    fn extract_object_methods(&mut self, obj_lit: &ObjectLit, object_name: &str) {
        for prop_or_spread in &obj_lit.props {
            if let PropOrSpread::Prop(prop) = prop_or_spread {
                match prop.as_ref() {
                    Prop::KeyValue(KeyValueProp { key, value }) => {
                        let method_name = match key {
                            PropName::Ident(ident) => ident.sym.to_string(),
                            PropName::Str(s) => s.value.to_string(),
                            _ => continue,
                        };

                        let qualified_name = format!("{}::{}", object_name, method_name);
                        let (is_async, line) = match value.as_ref() {
                            Expr::Fn(fn_expr) => (
                                fn_expr.function.is_async,
                                self.get_line(fn_expr.function.span),
                            ),
                            Expr::Arrow(arrow) => (arrow.is_async, self.get_line(arrow.span)),
                            _ => continue,
                        };

                        self.items.push(AstItem::Function {
                            name: qualified_name,
                            visibility: "public".to_string(),
                            is_async,
                            line,
                        });
                    }
                    Prop::Method(method_prop) => {
                        let method_name = match &method_prop.key {
                            PropName::Ident(ident) => ident.sym.to_string(),
                            PropName::Str(s) => s.value.to_string(),
                            _ => continue,
                        };

                        let qualified_name = format!("{}::{}", object_name, method_name);
                        let is_async = method_prop.function.is_async;
                        let line = self.get_line(method_prop.function.span);

                        self.items.push(AstItem::Function {
                            name: qualified_name,
                            visibility: "public".to_string(),
                            is_async,
                            line,
                        });
                    }
                    _ => {}
                }
            }
        }
    }
}
