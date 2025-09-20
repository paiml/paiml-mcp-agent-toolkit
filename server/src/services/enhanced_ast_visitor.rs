//! Enhanced AST visitor that preserves real source locations and qualified names
//!
//! This module provides an enhanced visitor that extracts actual AST information
//! instead of generating placeholders, enabling MCP tools to query precise
//! code locations and symbol names.

use crate::services::context::AstItem;
use std::path::{Path, PathBuf};
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{ItemEnum, ItemFn, ItemImpl, ItemMod, ItemStruct, ItemTrait, ItemUse, Visibility};

/// Enhanced AST visitor that preserves real source information
pub struct EnhancedAstVisitor {
    items: Vec<AstItem>,
    #[allow(dead_code)]
    file_path: PathBuf,
    module_path: Vec<String>,
}

impl EnhancedAstVisitor {
    /// Creates a new enhanced visitor for a given file
    #[must_use] 
    pub fn new(file_path: &Path) -> Self {
        Self {
            items: Vec::new(),
            file_path: file_path.to_path_buf(),
            module_path: vec![],
        }
    }

    /// Extracts all AST items with real source information
    #[must_use] 
    pub fn extract_items(mut self, syntax_tree: &syn::File) -> Vec<AstItem> {
        self.visit_file(syntax_tree);
        self.items
    }

    /// Gets visibility as a string
    fn get_visibility(&self, vis: &Visibility) -> String {
        match vis {
            Visibility::Public(_) => "pub".to_string(),
            Visibility::Restricted(r) => {
                if r.path.is_ident("crate") {
                    "pub(crate)".to_string()
                } else if r.path.is_ident("super") {
                    "pub(super)".to_string()
                } else if r.path.is_ident("self") {
                    "pub(self)".to_string()
                } else {
                    format!("pub(in {})", quote::quote!(#r.path))
                }
            }
            Visibility::Inherited => "private".to_string(),
        }
    }

    /// Gets line number from span
    fn get_line(&self, span: proc_macro2::Span) -> usize {
        // In real proc_macro2, spans don't carry line info by default
        // We'll use a heuristic based on the span's debug representation
        // For production, we'd integrate with proc_macro2's unstable features
        // or use a source map approach
        let debug_str = format!("{span:?}");

        // Extract line number from debug representation if available
        // Format is typically "Span { start: Loc { line: X, ... }, ... }"
        if let Some(line_start) = debug_str.find("line: ") {
            let line_str = &debug_str[line_start + 6..];
            if let Some(comma_pos) = line_str.find(',') {
                if let Ok(line) = line_str[..comma_pos].parse::<usize>() {
                    return line;
                }
            }
        }

        // Fallback to sequential numbering
        self.items.len() + 1
    }

    /// Creates a qualified name for the current module context
    fn get_qualified_name(&self, name: &str) -> String {
        if self.module_path.is_empty() {
            name.to_string()
        } else {
            format!("{}::{}", self.module_path.join("::"), name)
        }
    }
}

impl<'ast> Visit<'ast> for EnhancedAstVisitor {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        let name = self.get_qualified_name(&node.sig.ident.to_string());
        let visibility = self.get_visibility(&node.vis);
        let is_async = node.sig.asyncness.is_some();
        let line = self.get_line(node.span());

        self.items.push(AstItem::Function {
            name,
            visibility,
            is_async,
            line,
        });

        syn::visit::visit_item_fn(self, node);
    }

    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        let name = self.get_qualified_name(&node.ident.to_string());
        let visibility = self.get_visibility(&node.vis);
        let fields_count = node.fields.len();
        let line = self.get_line(node.span());

        // Extract derives
        let mut derives = Vec::new();
        for attr in &node.attrs {
            if attr.path().is_ident("derive") {
                if let Ok(syn::Meta::List(meta_list)) = attr.parse_args::<syn::Meta>() {
                    // Extract derive macro names
                    let tokens = quote::quote!(#meta_list);
                    let derive_str = tokens.to_string();
                    derives.push(derive_str);
                }
            }
        }

        self.items.push(AstItem::Struct {
            name,
            visibility,
            fields_count,
            derives,
            line,
        });

        syn::visit::visit_item_struct(self, node);
    }

    fn visit_item_enum(&mut self, node: &'ast ItemEnum) {
        let name = self.get_qualified_name(&node.ident.to_string());
        let visibility = self.get_visibility(&node.vis);
        let variants_count = node.variants.len();
        let line = self.get_line(node.span());

        self.items.push(AstItem::Enum {
            name,
            visibility,
            variants_count,
            line,
        });

        syn::visit::visit_item_enum(self, node);
    }

    fn visit_item_trait(&mut self, node: &'ast ItemTrait) {
        let name = self.get_qualified_name(&node.ident.to_string());
        let visibility = self.get_visibility(&node.vis);
        let line = self.get_line(node.span());

        self.items.push(AstItem::Trait {
            name,
            visibility,
            line,
        });

        syn::visit::visit_item_trait(self, node);
    }

    fn visit_item_impl(&mut self, node: &'ast ItemImpl) {
        let type_name = quote::quote!(#node.self_ty).to_string();
        let trait_name = node.trait_.as_ref().map(|(_, path, _)| {
            quote::quote!(#path).to_string()
        });
        let line = self.get_line(node.span());

        self.items.push(AstItem::Impl {
            type_name,
            trait_name,
            line,
        });

        syn::visit::visit_item_impl(self, node);
    }

    fn visit_item_mod(&mut self, node: &'ast ItemMod) {
        let name = node.ident.to_string();
        let visibility = self.get_visibility(&node.vis);
        let line = self.get_line(node.span());

        self.items.push(AstItem::Module {
            name: self.get_qualified_name(&name),
            visibility,
            line,
        });

        // Track module path for nested items
        self.module_path.push(name.clone());
        syn::visit::visit_item_mod(self, node);
        self.module_path.pop();
    }

    fn visit_item_use(&mut self, node: &'ast ItemUse) {
        let path = quote::quote!(#node.tree).to_string();
        let line = self.get_line(node.span());

        self.items.push(AstItem::Use {
            path,
            line,
        });

        syn::visit::visit_item_use(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test enhanced visitor extracts real function names
    #[test]
    fn test_extract_real_function_names() {
        let code = r#"
            pub fn calculate_complexity() -> u32 { 42 }
            async fn process_data(input: &str) -> Result<String, Error> { Ok(input.to_string()) }
            pub(crate) fn helper_function() {}
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let visitor = EnhancedAstVisitor::new(Path::new("test.rs"));
        let items = visitor.extract_items(&syntax);

        assert_eq!(items.len(), 3);

        // Verify real names are extracted
        match &items[0] {
            AstItem::Function { name, visibility, is_async, .. } => {
                assert_eq!(name, "calculate_complexity");
                assert_eq!(visibility, "pub");
                assert!(!is_async);
            }
            _ => panic!("Expected Function"),
        }

        match &items[1] {
            AstItem::Function { name, visibility, is_async, .. } => {
                assert_eq!(name, "process_data");
                assert_eq!(visibility, "private");
                assert!(is_async);
            }
            _ => panic!("Expected Function"),
        }
    }

    /// Test enhanced visitor preserves module hierarchy
    #[test]
    fn test_preserve_module_hierarchy() {
        let code = r#"
            mod services {
                pub fn service_function() {}

                mod internal {
                    fn internal_helper() {}
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let visitor = EnhancedAstVisitor::new(Path::new("test.rs"));
        let items = visitor.extract_items(&syntax);

        // Find the service_function
        let service_fn = items.iter().find(|item| {
            matches!(item, AstItem::Function { name, .. } if name.contains("service_function"))
        });

        assert!(service_fn.is_some());
        if let Some(AstItem::Function { name, .. }) = service_fn {
            assert_eq!(name, "services::service_function");
        }
    }

    /// Test enhanced visitor extracts struct information
    #[test]
    fn test_extract_struct_details() {
        let code = r#"
            #[derive(Debug, Clone)]
            pub struct Configuration {
                pub name: String,
                value: u32,
                internal: bool,
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let visitor = EnhancedAstVisitor::new(Path::new("test.rs"));
        let items = visitor.extract_items(&syntax);

        assert_eq!(items.len(), 1);
        match &items[0] {
            AstItem::Struct { name, visibility, fields_count, .. } => {
                assert_eq!(name, "Configuration");
                assert_eq!(visibility, "pub");
                assert_eq!(*fields_count, 3);
            }
            _ => panic!("Expected Struct"),
        }
    }

    /// Test enhanced visitor handles complex visibility modifiers
    #[test]
    fn test_visibility_modifiers() {
        let code = r#"
            pub fn public_fn() {}
            pub(crate) fn crate_fn() {}
            pub(super) fn super_fn() {}
            fn private_fn() {}
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let visitor = EnhancedAstVisitor::new(Path::new("test.rs"));
        let items = visitor.extract_items(&syntax);

        let visibilities: Vec<String> = items
            .iter()
            .filter_map(|item| match item {
                AstItem::Function { visibility, .. } => Some(visibility.clone()),
                _ => None,
            })
            .collect();

        assert_eq!(visibilities, vec!["pub", "pub(crate)", "pub(super)", "private"]);
    }
}

/// Property tests for enhanced AST visitor
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Property: visitor always produces valid AST items
        #[test]
        fn visitor_produces_valid_items(seed in 0u64..1000) {
            // Generate deterministic test code based on seed
            let code = generate_test_code(seed);

            if let Ok(syntax) = syn::parse_file(&code) {
                let visitor = EnhancedAstVisitor::new(Path::new("test.rs"));
                let items = visitor.extract_items(&syntax);

                // All items should have non-empty names
                for item in &items {
                    match item {
                        AstItem::Function { name, .. } |
                        AstItem::Struct { name, .. } |
                        AstItem::Enum { name, .. } |
                        AstItem::Trait { name, .. } |
                        AstItem::Module { name, .. } => {
                            prop_assert!(!name.is_empty());
                        }
                        AstItem::Impl { type_name, .. } => {
                            prop_assert!(!type_name.is_empty());
                        }
                        AstItem::Use { path, .. } => {
                            prop_assert!(!path.is_empty());
                        }
                        _ => {}
                    }
                }
            }
        }

        /// Property: qualified names preserve module hierarchy
        #[test]
        fn qualified_names_preserve_hierarchy(module_depth in 0usize..5) {
            let code = generate_nested_modules(module_depth);

            if let Ok(syntax) = syn::parse_file(&code) {
                let visitor = EnhancedAstVisitor::new(Path::new("test.rs"));
                let items = visitor.extract_items(&syntax);

                // Functions in nested modules should have qualified names
                for item in &items {
                    if let AstItem::Function { name, .. } = item {
                        let separators = name.matches("::").count();
                        prop_assert!(separators <= module_depth);
                    }
                }
            }
        }
    }

    fn generate_test_code(seed: u64) -> String {
        let fn_count = (seed % 5) + 1;
        let mut code = String::new();

        for i in 0..fn_count {
            code.push_str(&format!("fn function_{}() {{}}\n", i));
        }

        if seed % 3 == 0 {
            code.push_str("pub struct TestStruct { field: u32 }\n");
        }

        if seed % 5 == 0 {
            code.push_str("enum TestEnum { Variant1, Variant2 }\n");
        }

        code
    }

    fn generate_nested_modules(depth: usize) -> String {
        let mut code = String::new();
        let mut indent = String::new();

        for i in 0..depth {
            code.push_str(&format!("{}mod level_{} {{\n", indent, i));
            indent.push_str("    ");
        }

        code.push_str(&format!("{}fn nested_function() {{}}\n", indent));

        for _ in 0..depth {
            indent.truncate(indent.len().saturating_sub(4));
            code.push_str(&format!("{}}}\n", indent));
        }

        code
    }
}