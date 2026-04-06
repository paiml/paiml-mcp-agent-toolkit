// Visit trait implementation for EnhancedTypeScriptVisitor: AST node traversal handlers

#[cfg(feature = "typescript-ast")]
impl Visit for EnhancedTypeScriptVisitor {
    fn visit_function(&mut self, func: &Function) {
        // This handles function expressions and arrow functions
        // Skip functions that are part of classes (already handled by visit_class_method)
        if !self.class_stack.is_empty() {
            // We're inside a class - skip this as class methods are handled elsewhere
            func.visit_children_with(self);
            return;
        }

        let name = self.get_qualified_name("anonymous");
        let is_async = self.is_async_function(func);
        let line = self.get_line(func.span);

        self.items.push(AstItem::Function {
            name,
            visibility: "public".to_string(),
            is_async,
            line,
        });

        func.visit_children_with(self);
    }

    fn visit_fn_decl(&mut self, func_decl: &FnDecl) {
        let name = self.get_qualified_name(func_decl.ident.sym.as_ref());
        let is_async = func_decl.function.is_async;
        let line = self.get_line(func_decl.span());

        self.items.push(AstItem::Function {
            name,
            visibility: "public".to_string(),
            is_async,
            line,
        });

        func_decl.visit_children_with(self);
    }

    fn visit_class_decl(&mut self, class_decl: &ClassDecl) {
        let class_name = class_decl.ident.sym.to_string();
        let qualified_name = self.get_qualified_name(&class_name);
        let line = self.get_line(class_decl.span());

        // Count methods and properties in the class
        let mut method_count = 0;
        for member in &class_decl.class.body {
            match member {
                ClassMember::Method(_) | ClassMember::Constructor(_) => method_count += 1,
                _ => {}
            }
        }

        self.items.push(AstItem::Struct {
            name: qualified_name,
            visibility: "public".to_string(),
            fields_count: method_count,
            derives: vec![], // TypeScript doesn't have derives like Rust
            line,
        });

        // Track class context for nested members
        self.class_stack.push(class_name);
        class_decl.visit_children_with(self);
        self.class_stack.pop();
    }

    fn visit_method_prop(&mut self, method: &MethodProp) {
        let method_name = match &method.key {
            PropName::Ident(ident) => ident.sym.to_string(),
            PropName::Str(s) => s.value.to_string(),
            PropName::Num(n) => n.value.to_string(),
            PropName::Computed(_) => "computed".to_string(),
            PropName::BigInt(b) => b.value.to_string(),
        };

        let qualified_name = self.get_qualified_name(&method_name);
        let is_async = method.function.is_async;
        let line = self.get_line(method.span());

        self.items.push(AstItem::Function {
            name: qualified_name,
            visibility: "public".to_string(),
            is_async,
            line,
        });

        method.visit_children_with(self);
    }

    fn visit_constructor(&mut self, constructor: &Constructor) {
        let qualified_name = self.get_qualified_name("constructor");
        let line = self.get_line(constructor.span());

        self.items.push(AstItem::Function {
            name: qualified_name,
            visibility: "public".to_string(),
            is_async: false, // Constructors can't be async
            line,
        });

        constructor.visit_children_with(self);
    }

    fn visit_class_method(&mut self, method: &ClassMethod) {
        let method_name = match &method.key {
            PropName::Ident(ident) => ident.sym.to_string(),
            PropName::Str(s) => s.value.to_string(),
            PropName::Num(n) => n.value.to_string(),
            PropName::Computed(_) => "computed".to_string(),
            PropName::BigInt(b) => b.value.to_string(),
        };

        let qualified_name = self.get_qualified_name(&method_name);
        let is_async = method.function.is_async;
        let line = self.get_line(method.span());

        self.items.push(AstItem::Function {
            name: qualified_name,
            visibility: if method.is_static { "static" } else { "public" }.to_string(),
            is_async,
            line,
        });

        method.visit_children_with(self);
    }

    fn visit_ts_interface_decl(&mut self, interface: &TsInterfaceDecl) {
        let name = self.get_qualified_name(interface.id.sym.as_ref());
        let line = self.get_line(interface.span());
        let _members_count = interface.body.body.len();

        self.items.push(AstItem::Trait {
            name,
            visibility: "public".to_string(),
            line,
        });

        interface.visit_children_with(self);
    }

    fn visit_ts_enum_decl(&mut self, enum_decl: &TsEnumDecl) {
        let name = self.get_qualified_name(enum_decl.id.sym.as_ref());
        let line = self.get_line(enum_decl.span());
        let variants_count = enum_decl.members.len();

        self.items.push(AstItem::Enum {
            name,
            visibility: "public".to_string(),
            variants_count,
            line,
        });

        enum_decl.visit_children_with(self);
    }

    fn visit_import_decl(&mut self, import: &ImportDecl) {
        let path = import.src.value.to_string();
        let line = self.get_line(import.span());

        self.items.push(AstItem::Use { path, line });

        import.visit_children_with(self);
    }

    fn visit_named_export(&mut self, export: &NamedExport) {
        if let Some(src) = &export.src {
            let path = format!("export from {}", src.value);
            let line = self.get_line(export.span());

            self.items.push(AstItem::Use { path, line });
        }

        export.visit_children_with(self);
    }

    fn visit_module_decl(&mut self, module: &ModuleDecl) {
        match module {
            ModuleDecl::Import(import) => self.visit_import_decl(import),
            ModuleDecl::ExportNamed(export) => self.visit_named_export(export),
            ModuleDecl::ExportAll(export) => {
                let path = format!("export * from {}", export.src.value);
                let line = self.get_line(export.span());
                self.items.push(AstItem::Use { path, line });
            }
            ModuleDecl::ExportDefaultDecl(export_default) => {
                self.visit_export_default_decl(export_default);
            }
            ModuleDecl::ExportDecl(export_decl) => {
                self.visit_export_decl(export_decl);
            }
            _ => {
                module.visit_children_with(self);
            }
        }
    }

    fn visit_var_decl(&mut self, var_decl: &VarDecl) {
        // Handle variable declarations that might be functions
        for declarator in &var_decl.decls {
            if let Some(init) = &declarator.init {
                match init.as_ref() {
                    Expr::Fn(fn_expr) => {
                        if let Pat::Ident(ident) = &declarator.name {
                            let name = self.get_qualified_name(ident.id.sym.as_ref());
                            let is_async = fn_expr.function.is_async;
                            let line = self.get_line(var_decl.span());

                            self.items.push(AstItem::Function {
                                name,
                                visibility: "public".to_string(),
                                is_async,
                                line,
                            });
                        }
                    }
                    Expr::Arrow(arrow) => {
                        if let Pat::Ident(ident) = &declarator.name {
                            let name = self.get_qualified_name(ident.id.sym.as_ref());
                            let is_async = arrow.is_async;
                            let line = self.get_line(var_decl.span());

                            self.items.push(AstItem::Function {
                                name,
                                visibility: "public".to_string(),
                                is_async,
                                line,
                            });
                        }
                    }
                    Expr::Object(obj_lit) => {
                        // Handle object literal with methods (like { get: async (endpoint) => {...} })
                        if let Pat::Ident(ident) = &declarator.name {
                            let object_name = ident.id.sym.as_ref();
                            self.extract_object_methods(obj_lit, object_name);
                        }
                    }
                    _ => {}
                }
            }
        }

        var_decl.visit_children_with(self);
    }

    fn visit_return_stmt(&mut self, return_stmt: &ReturnStmt) {
        if let Some(arg) = &return_stmt.arg {
            self.extract_function_from_expr(arg);
        }
        return_stmt.visit_children_with(self);
    }

    fn visit_stmt(&mut self, stmt: &Stmt) {
        stmt.visit_children_with(self);
    }

    fn visit_export_default_decl(&mut self, export_default: &ExportDefaultDecl) {
        match &export_default.decl {
            DefaultDecl::Class(class_expr) => {
                if let Some(ident) = &class_expr.ident {
                    let class_name = ident.sym.to_string();
                    let qualified_name = self.get_qualified_name(&class_name);
                    let line = self.get_line(class_expr.class.span);

                    // Count methods and properties in the class
                    let mut method_count = 0;
                    for member in &class_expr.class.body {
                        match member {
                            ClassMember::Method(_) | ClassMember::Constructor(_) => {
                                method_count += 1
                            }
                            _ => {}
                        }
                    }

                    self.items.push(AstItem::Struct {
                        name: qualified_name,
                        visibility: "public".to_string(),
                        fields_count: method_count,
                        derives: vec![],
                        line,
                    });

                    // Track class context for nested members
                    self.class_stack.push(class_name);
                    class_expr.class.visit_children_with(self);
                    self.class_stack.pop();
                }
            }
            DefaultDecl::Fn(fn_expr) => {
                if let Some(ident) = &fn_expr.ident {
                    let name = self.get_qualified_name(ident.sym.as_ref());
                    let is_async = fn_expr.function.is_async;
                    let line = self.get_line(fn_expr.span());

                    self.items.push(AstItem::Function {
                        name,
                        visibility: "public".to_string(),
                        is_async,
                        line,
                    });
                }
                fn_expr.visit_children_with(self);
            }
            _ => {
                export_default.visit_children_with(self);
            }
        }
    }

    fn visit_export_decl(&mut self, export_decl: &ExportDecl) {
        // This handles `export function`, `export class`, etc.
        export_decl.visit_children_with(self);
    }
}
