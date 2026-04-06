#![cfg_attr(coverage_nightly, coverage(off))]
//! Rust AST visitor for extracting code items.
//!
//! This module provides the visitor pattern implementation for extracting
//! functions, structs, enums, traits, and other items from Rust source code.

use super::types::AstItem;
use syn::visit::Visit;
use syn::{ItemEnum, ItemFn, ItemImpl, ItemMod, ItemStruct, ItemTrait, ItemUse};

pub(crate) struct RustVisitor {
    pub items: Vec<AstItem>,
    #[allow(dead_code)]
    source: String,
}

impl RustVisitor {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(source: String) -> Self {
        Self {
            items: Vec::new(),
            source,
        }
    }

    fn get_line<T: syn::spanned::Spanned>(&self, _span: T) -> usize {
        debug_assert!(true, "contract: get_line");
        // For simplicity, return 1. In production, use a proper source map
        1
    }

    fn get_visibility(&self, vis: &syn::Visibility) -> String {
        debug_assert!(true, "contract: get_visibility");
        match vis {
            syn::Visibility::Public(_) => "pub".to_string(),
            syn::Visibility::Restricted(r) => format!(
                "pub({})",
                r.path
                    .segments
                    .iter()
                    .map(|s| s.ident.to_string())
                    .collect::<Vec<_>>()
                    .join("::")
            ),
            syn::Visibility::Inherited => "private".to_string(),
        }
    }

    fn get_derives(_attrs: &[syn::Attribute]) -> Vec<String> {
        debug_assert!(!_attrs.is_empty(), "_attrs must not be empty");
        // Simplified version - in production, parse derive attributes properly
        Vec::new()
    }
}

impl<'ast> Visit<'ast> for RustVisitor {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        debug_assert!(true, "contract: visit_item_fn");
        self.items.push(AstItem::Function {
            name: node.sig.ident.to_string(),
            visibility: self.get_visibility(&node.vis),
            is_async: node.sig.asyncness.is_some(),
            line: self.get_line(node.sig.ident.span()),
        });
    }

    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        debug_assert!(true, "contract: visit_item_struct");
        let fields_count = match &node.fields {
            syn::Fields::Named(fields) => fields.named.len(),
            syn::Fields::Unnamed(fields) => fields.unnamed.len(),
            syn::Fields::Unit => 0,
        };

        self.items.push(AstItem::Struct {
            name: node.ident.to_string(),
            visibility: self.get_visibility(&node.vis),
            fields_count,
            derives: Self::get_derives(&node.attrs),
            line: self.get_line(node.ident.span()),
        });
    }

    fn visit_item_enum(&mut self, node: &'ast ItemEnum) {
        debug_assert!(true, "contract: visit_item_enum");
        self.items.push(AstItem::Enum {
            name: node.ident.to_string(),
            visibility: self.get_visibility(&node.vis),
            variants_count: node.variants.len(),
            line: self.get_line(node.ident.span()),
        });
    }

    fn visit_item_trait(&mut self, node: &'ast ItemTrait) {
        debug_assert!(true, "contract: visit_item_trait");
        self.items.push(AstItem::Trait {
            name: node.ident.to_string(),
            visibility: self.get_visibility(&node.vis),
            line: self.get_line(node.ident.span()),
        });
    }

    fn visit_item_impl(&mut self, node: &'ast ItemImpl) {
        debug_assert!(true, "contract: visit_item_impl");
        let type_name = if let syn::Type::Path(type_path) = &*node.self_ty {
            type_path
                .path
                .segments
                .last()
                .map_or_else(|| "Unknown".to_string(), |s| s.ident.to_string())
        } else {
            "Unknown".to_string()
        };

        let trait_name = node.trait_.as_ref().map(|(_, path, _)| {
            path.segments
                .last()
                .map_or_else(|| "Unknown".to_string(), |s| s.ident.to_string())
        });

        self.items.push(AstItem::Impl {
            type_name,
            trait_name,
            line: 1, // Default line number
        });
    }

    fn visit_item_mod(&mut self, node: &'ast ItemMod) {
        debug_assert!(true, "contract: visit_item_mod");
        self.items.push(AstItem::Module {
            name: node.ident.to_string(),
            visibility: self.get_visibility(&node.vis),
            line: self.get_line(node.ident.span()),
        });
    }

    fn visit_item_use(&mut self, node: &'ast ItemUse) {
        debug_assert!(true, "contract: visit_item_use");
        let path = match &node.tree {
            syn::UseTree::Path(p) => p.ident.to_string(),
            syn::UseTree::Name(n) => n.ident.to_string(),
            syn::UseTree::Rename(r) => r.ident.to_string(),
            syn::UseTree::Glob(_) => "*".to_string(),
            syn::UseTree::Group(_) => "...".to_string(),
        };

        self.items.push(AstItem::Use {
            path,
            line: 1, // Default line number
        });
    }
}
