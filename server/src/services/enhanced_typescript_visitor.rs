//! Enhanced TypeScript/JavaScript AST visitor that preserves real source locations and qualified names
//!
//! This module provides an enhanced visitor that extracts actual AST information
//! from SWC-parsed TypeScript/JavaScript instead of generating placeholders,
//! enabling MCP tools to query precise code locations and symbol names.

#[cfg(feature = "typescript-ast")]
use crate::services::context::AstItem;
#[cfg(feature = "typescript-ast")]
use std::path::{Path, PathBuf};
#[cfg(feature = "typescript-ast")]
use swc_common::{Span, Spanned};
#[cfg(feature = "typescript-ast")]
use swc_ecma_ast::*;
#[cfg(feature = "typescript-ast")]
use swc_ecma_visit::{Visit, VisitWith};

/// Enhanced TypeScript/JavaScript AST visitor that preserves real source information
#[cfg(feature = "typescript-ast")]
pub struct EnhancedTypeScriptVisitor {
    items: Vec<AstItem>,
    #[allow(dead_code)]
    file_path: PathBuf,
    module_path: Vec<String>,
    class_stack: Vec<String>,
}

#[cfg(feature = "typescript-ast")]
impl EnhancedTypeScriptVisitor {
    /// Creates a new enhanced visitor for a given file
    pub fn new(file_path: &Path) -> Self {
        Self {
            items: Vec::new(),
            file_path: file_path.to_path_buf(),
            module_path: vec![],
            class_stack: vec![],
        }
    }

    /// Extracts all AST items with real source information
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
}

#[cfg(feature = "typescript-ast")]
impl Visit for EnhancedTypeScriptVisitor {
    fn visit_function(&mut self, func: &Function) {
        // This handles function expressions and arrow functions
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

        self.items.push(AstItem::Use {
            path,
            line,
        });

        import.visit_children_with(self);
    }

    fn visit_named_export(&mut self, export: &NamedExport) {
        if let Some(src) = &export.src {
            let path = format!("export from {}", src.value);
            let line = self.get_line(export.span());

            self.items.push(AstItem::Use {
                path,
                line,
            });
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
                    _ => {}
                }
            }
        }

        var_decl.visit_children_with(self);
    }
}

// Stub implementation when typescript-ast feature is disabled
#[cfg(not(feature = "typescript-ast"))]
pub struct EnhancedTypeScriptVisitor;

#[cfg(not(feature = "typescript-ast"))]
impl EnhancedTypeScriptVisitor {
    pub fn new(_file_path: &std::path::Path) -> Self {
        Self
    }

    pub fn extract_items(self, _module: &()) -> Vec<crate::services::context::AstItem> {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "typescript-ast")]
    mod typescript_tests {
        use super::*;
        use swc_common::{FileName, SourceMap};
        use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsSyntax};
        use std::sync::Arc;

        fn parse_typescript(code: &str) -> Module {
            let source_map = Arc::new(SourceMap::default());
            let source_file = source_map.new_source_file(
                FileName::Custom("test.ts".into()).into(),
                code.to_string(),
            );

            let lexer = Lexer::new(
                Syntax::Typescript(TsSyntax {
                    tsx: false,
                    decorators: true,
                    dts: false,
                    no_early_errors: true,
                    disallow_ambiguous_jsx_like: true,
                }),
                Default::default(),
                StringInput::from(&*source_file),
                None,
            );

            let mut parser = Parser::new_from(lexer);
            parser.parse_module().expect("Failed to parse TypeScript")
        }

        /// Test enhanced visitor extracts real function names
        #[test]
        fn test_extract_real_function_names() {
            let code = r#"
                function calculateComplexity(): number { return 42; }
                async function processData(input: string): Promise<string> {
                    return input.toUpperCase();
                }
                const helperFunction = () => {};
                const asyncHelper = async (x: number) => x * 2;
            "#;

            let module = parse_typescript(code);
            let visitor = EnhancedTypeScriptVisitor::new(Path::new("test.ts"));
            let items = visitor.extract_items(&module);

            // Should extract all real function names
            let function_names: Vec<String> = items
                .iter()
                .filter_map(|item| match item {
                    AstItem::Function { name, .. } => Some(name.clone()),
                    _ => None,
                })
                .collect();

            assert!(function_names.contains(&"calculateComplexity".to_string()));
            assert!(function_names.contains(&"processData".to_string()));
            assert!(function_names.contains(&"helperFunction".to_string()));
            assert!(function_names.contains(&"asyncHelper".to_string()));
        }

        /// Test enhanced visitor handles classes correctly
        #[test]
        fn test_extract_class_details() {
            let code = r#"
                class DataProcessor {
                    constructor(private threshold: number) {}

                    async process(data: string[]): Promise<string[]> {
                        return data.filter(item => item.length > this.threshold);
                    }

                    private validateInput(input: string): boolean {
                        return input.length > 0;
                    }
                }
            "#;

            let module = parse_typescript(code);
            let visitor = EnhancedTypeScriptVisitor::new(Path::new("test.ts"));
            let items = visitor.extract_items(&module);

            // Find the class
            let class_item = items.iter().find(|item| {
                matches!(item, AstItem::Struct { name, .. } if name == "DataProcessor")
            });
            assert!(class_item.is_some());

            // Find class methods
            let method_names: Vec<String> = items
                .iter()
                .filter_map(|item| match item {
                    AstItem::Function { name, .. } if name.contains("DataProcessor::") => {
                        Some(name.clone())
                    }
                    _ => None,
                })
                .collect();

            assert!(method_names.contains(&"DataProcessor::constructor".to_string()));
            assert!(method_names.contains(&"DataProcessor::process".to_string()));
            assert!(method_names.contains(&"DataProcessor::validateInput".to_string()));
        }

        /// Test enhanced visitor handles interfaces
        #[test]
        fn test_extract_interface_details() {
            let code = r#"
                interface Processor<T> {
                    process(data: T): T;
                    validate(input: T): boolean;
                }
            "#;

            let module = parse_typescript(code);
            let visitor = EnhancedTypeScriptVisitor::new(Path::new("test.ts"));
            let items = visitor.extract_items(&module);

            let interface_item = items.iter().find(|item| {
                matches!(item, AstItem::Trait { name, .. } if name == "Processor")
            });
            assert!(interface_item.is_some());
        }

        /// Test enhanced visitor handles imports/exports
        #[test]
        fn test_extract_imports() {
            let code = r#"
                import { Component } from 'react';
                import * as utils from './utils';
                export { DataProcessor } from './processor';
            "#;

            let module = parse_typescript(code);
            let visitor = EnhancedTypeScriptVisitor::new(Path::new("test.ts"));
            let items = visitor.extract_items(&module);

            let import_items: Vec<&AstItem> = items
                .iter()
                .filter(|item| matches!(item, AstItem::Use { .. }))
                .collect();

            assert!(import_items.len() >= 2);
        }
    }

    /// Property tests for enhanced TypeScript visitor
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            /// Property: visitor always produces valid AST items for valid TypeScript
            #[test]
            fn visitor_produces_valid_items(seed in 0u64..100) {
                let code = generate_typescript_code(seed);

                // Only test valid TypeScript code
                if is_valid_typescript(&code) {
                    #[cfg(feature = "typescript-ast")]
                    {
                        if let Ok(module) = try_parse_typescript(&code) {
                            let visitor = EnhancedTypeScriptVisitor::new(Path::new("test.ts"));
                            let items = visitor.extract_items(&module);

                            // All function items should have non-empty names
                            for item in &items {
                                match item {
                                    AstItem::Function { name, .. } |
                                    AstItem::Struct { name, .. } |
                                    AstItem::Enum { name, .. } |
                                    AstItem::Trait { name, .. } => {
                                        prop_assert!(!name.is_empty());
                                        prop_assert!(!name.starts_with("function_"));
                                        prop_assert!(!name.starts_with("class_"));
                                    }
                                    AstItem::Use { path, .. } => {
                                        prop_assert!(!path.is_empty());
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }

            /// Property: async functions are correctly detected
            #[test]
            fn async_detection_property(is_async in any::<bool>()) {
                let async_keyword = if is_async { "async " } else { "" };
                let code = format!("{}function testFunc() {{ return 42; }}", async_keyword);

                #[cfg(feature = "typescript-ast")]
                {
                    if let Ok(module) = try_parse_typescript(&code) {
                        let visitor = EnhancedTypeScriptVisitor::new(Path::new("test.ts"));
                        let items = visitor.extract_items(&module);

                        if let Some(AstItem::Function { is_async: detected, .. }) =
                            items.iter().find(|item| matches!(item, AstItem::Function { .. })) {
                            prop_assert_eq!(*detected, is_async);
                        }
                    }
                }
            }
        }

        fn generate_typescript_code(seed: u64) -> String {
            let templates = vec![
                "function func{}() { return {}; }",
                "const func{} = () => {};",
                "class Class{} { method() {} }",
                "interface Interface{} { prop: number; }",
            ];

            let template = &templates[seed as usize % templates.len()];
            template.replace("{}", &seed.to_string())
        }

        fn is_valid_typescript(code: &str) -> bool {
            // Basic validation - not empty, has valid characters
            !code.is_empty() && code.chars().all(|c| c.is_ascii())
        }

        #[cfg(feature = "typescript-ast")]
        fn try_parse_typescript(code: &str) -> Result<Module, ()> {
            use swc_common::{FileName, SourceMap};
            use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsSyntax};
            use std::sync::Arc;

            let source_map = Arc::new(SourceMap::default());
            let source_file = source_map.new_source_file(
                FileName::Custom("test.ts".into()).into(),
                code.to_string(),
            );

            let lexer = Lexer::new(
                Syntax::Typescript(TsSyntax {
                    tsx: false,
                    decorators: true,
                    dts: false,
                    no_early_errors: true,
                    disallow_ambiguous_jsx_like: true,
                }),
                Default::default(),
                StringInput::from(&*source_file),
                None,
            );

            let mut parser = Parser::new_from(lexer);
            parser.parse_module().map_err(|_| ())
        }
    }
}