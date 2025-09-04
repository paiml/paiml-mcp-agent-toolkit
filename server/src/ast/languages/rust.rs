//! Rust language AST parsing strategy

use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;
use syn::{visit::Visit, File as SynFile, ItemEnum, ItemFn, ItemImpl, ItemStruct, ItemTrait};

use super::LanguageStrategy;
use crate::ast::core::{
    AstDag, AstKind, ClassKind, FunctionKind, Language, NodeFlags, UnifiedAstNode,
};

/// Rust language parsing strategy
pub struct RustStrategy {
    // Configuration options can be added here
}

impl Default for RustStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl RustStrategy {
    pub fn new() -> Self {
        Self {}
    }

    fn parse_syn_file(&self, content: &str) -> Result<SynFile> {
        syn::parse_file(content).map_err(|e| anyhow::anyhow!("Rust parse error: {}", e))
    }

    fn convert_to_dag(&self, syn_file: &SynFile) -> AstDag {
        let mut dag = AstDag::new();
        let mut visitor = RustAstVisitor::new(&mut dag);
        visitor.visit_file(syn_file);
        dag
    }
}

#[async_trait]
impl LanguageStrategy for RustStrategy {
    fn language(&self) -> Language {
        Language::Rust
    }

    fn can_parse(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext == "rs")
            .unwrap_or(false)
    }

    async fn parse_file(&self, _path: &Path, content: &str) -> Result<AstDag> {
        let syn_file = self.parse_syn_file(content)?;
        Ok(self.convert_to_dag(&syn_file))
    }

    fn extract_imports(&self, ast: &AstDag) -> Vec<String> {
        // Iterate through nodes looking for imports
        let mut imports = Vec::new();
        for i in 0..ast.nodes.len() {
            if let Some(node) = ast.nodes.get(i as u32) {
                if node.flags.has(NodeFlags::IMPORT) {
                    // Extract import name from metadata if available
                    imports.push(format!("import_{}", i));
                }
            }
        }
        imports
    }

    fn extract_functions(&self, ast: &AstDag) -> Vec<UnifiedAstNode> {
        let mut functions = Vec::new();
        for i in 0..ast.nodes.len() {
            if let Some(node) = ast.nodes.get(i as u32) {
                if matches!(node.kind, AstKind::Function(_)) {
                    functions.push(node.clone());
                }
            }
        }
        functions
    }

    fn extract_types(&self, ast: &AstDag) -> Vec<UnifiedAstNode> {
        let mut types = Vec::new();
        for i in 0..ast.nodes.len() {
            if let Some(node) = ast.nodes.get(i as u32) {
                if matches!(node.kind, AstKind::Class(_) | AstKind::Type(_)) {
                    types.push(node.clone());
                }
            }
        }
        types
    }

    fn calculate_complexity(&self, ast: &AstDag) -> (u32, u32) {
        let mut cyclomatic = 1;
        let mut cognitive = 0;

        // Count control flow nodes
        for i in 0..ast.nodes.len() {
            if let Some(node) = ast.nodes.get(i as u32) {
                if node.flags.has(NodeFlags::CONTROL_FLOW) {
                    cyclomatic += 1;
                    // Cognitive complexity increases with nesting depth
                    cognitive += 1;
                }
            }
        }

        (cyclomatic, cognitive)
    }
}

/// Visitor for converting syn AST to unified AST
struct RustAstVisitor<'a> {
    dag: &'a mut AstDag,
    current_parent: Option<u32>,
}

impl<'a> RustAstVisitor<'a> {
    fn new(dag: &'a mut AstDag) -> Self {
        Self {
            dag,
            current_parent: None,
        }
    }

    #[allow(dead_code)]
    fn add_node(&mut self, kind: AstKind) -> u32 {
        let mut node = UnifiedAstNode::new(kind, Language::Rust);

        // Set parent if we have one
        if let Some(parent) = self.current_parent {
            node.parent = parent;
        }

        self.dag.add_node(node)
    }
}

impl<'a> Visit<'_> for RustAstVisitor<'a> {
    fn visit_item_fn(&mut self, node: &ItemFn) {
        let mut ast_node =
            UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Rust);

        // Set async flag if needed
        if node.sig.asyncness.is_some() {
            ast_node.flags.set(NodeFlags::ASYNC);
        }

        let key = self.dag.add_node(ast_node);

        let old_parent = self.current_parent;
        self.current_parent = Some(key);

        syn::visit::visit_item_fn(self, node);

        self.current_parent = old_parent;
    }

    fn visit_item_struct(&mut self, node: &ItemStruct) {
        let ast_node = UnifiedAstNode::new(AstKind::Class(ClassKind::Struct), Language::Rust);

        let key = self.dag.add_node(ast_node);

        let old_parent = self.current_parent;
        self.current_parent = Some(key);

        syn::visit::visit_item_struct(self, node);

        self.current_parent = old_parent;
    }

    fn visit_item_enum(&mut self, node: &ItemEnum) {
        let ast_node = UnifiedAstNode::new(AstKind::Class(ClassKind::Enum), Language::Rust);

        let key = self.dag.add_node(ast_node);

        let old_parent = self.current_parent;
        self.current_parent = Some(key);

        syn::visit::visit_item_enum(self, node);

        self.current_parent = old_parent;
    }

    fn visit_item_trait(&mut self, node: &ItemTrait) {
        let ast_node = UnifiedAstNode::new(AstKind::Class(ClassKind::Trait), Language::Rust);

        let key = self.dag.add_node(ast_node);

        let old_parent = self.current_parent;
        self.current_parent = Some(key);

        syn::visit::visit_item_trait(self, node);

        self.current_parent = old_parent;
    }

    fn visit_item_impl(&mut self, node: &ItemImpl) {
        // For impl blocks, we create a special kind
        let ast_node = UnifiedAstNode::new(AstKind::Class(ClassKind::Regular), Language::Rust);

        let key = self.dag.add_node(ast_node);

        let old_parent = self.current_parent;
        self.current_parent = Some(key);

        syn::visit::visit_item_impl(self, node);

        self.current_parent = old_parent;
    }
}
