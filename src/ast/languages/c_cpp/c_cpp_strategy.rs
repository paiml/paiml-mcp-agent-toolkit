//! C and C++ language strategy implementations

#![cfg_attr(coverage_nightly, coverage(off))]

use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;

#[cfg(feature = "c-ast")]
use tree_sitter::Tree;

use crate::ast::core::{AstDag, AstKind, Language, NodeFlags, UnifiedAstNode};
use crate::ast::languages::LanguageStrategy;

#[cfg(any(feature = "c-ast", feature = "cpp-ast"))]
use super::c_cpp_visitor::CTreeSitterVisitor;

/// C language parsing strategy
pub struct CStrategy;

impl Default for CStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl CStrategy {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    #[cfg(feature = "c-ast")]
    fn parse_with_tree_sitter(&self, content: &str) -> Result<Tree> {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_c::LANGUAGE.into())
            .map_err(|e| anyhow::anyhow!("Failed to set C language: {e}"))?;

        parser
            .parse(content, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse C code"))
    }

    #[cfg(not(feature = "c-ast"))]
    #[allow(dead_code)]
    fn parse_with_tree_sitter(&self, _content: &str) -> Result<()> {
        Err(anyhow::anyhow!(
            "C AST parsing not available - compile with 'c-ast' feature"
        ))
    }

    #[cfg(feature = "c-ast")]
    fn convert_to_dag(&self, tree: &Tree, content: &str) -> AstDag {
        let mut dag = AstDag::new();
        let root = tree.root_node();
        let mut visitor = CTreeSitterVisitor::new(&mut dag, content, Language::C);
        visitor.visit_node(&root, None);
        dag
    }
}

#[async_trait]
impl LanguageStrategy for CStrategy {
    fn language(&self) -> Language {
        Language::C
    }

    fn can_parse(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| matches!(ext, "c" | "h"))
    }

    #[cfg(feature = "c-ast")]
    async fn parse_file(&self, _path: &Path, content: &str) -> Result<AstDag> {
        let tree = self.parse_with_tree_sitter(content)?;
        Ok(self.convert_to_dag(&tree, content))
    }

    #[cfg(not(feature = "c-ast"))]
    async fn parse_file(&self, _path: &Path, _content: &str) -> Result<AstDag> {
        Err(anyhow::anyhow!(
            "C AST parsing not available - compile with 'c-ast' feature"
        ))
    }

    fn extract_imports(&self, ast: &AstDag) -> Vec<String> {
        let mut imports = Vec::new();
        for i in 0..ast.nodes.len() {
            if let Some(node) = ast.nodes.get(i as u32) {
                if matches!(node.kind, AstKind::Import(_)) {
                    imports.push(format!("import_{i}"));
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
                if matches!(node.kind, AstKind::Class(_)) {
                    types.push(node.clone());
                }
            }
        }
        types
    }

    fn calculate_complexity(&self, ast: &AstDag) -> (u32, u32) {
        let mut cyclomatic = 1;
        let mut cognitive = 0;

        for i in 0..ast.nodes.len() {
            if let Some(node) = ast.nodes.get(i as u32) {
                if node.flags.has(NodeFlags::CONTROL_FLOW) {
                    cyclomatic += 1;
                    cognitive += 1;
                }
            }
        }

        (cyclomatic, cognitive)
    }
}

/// C++ language parsing strategy
pub struct CppStrategy;

impl Default for CppStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl CppStrategy {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    #[cfg(feature = "cpp-ast")]
    fn parse_with_tree_sitter(&self, content: &str) -> Result<Tree> {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_cpp::LANGUAGE.into())
            .map_err(|e| anyhow::anyhow!("Failed to set C++ language: {e}"))?;

        parser
            .parse(content, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse C++ code"))
    }

    #[cfg(not(feature = "cpp-ast"))]
    #[allow(dead_code)]
    fn parse_with_tree_sitter(&self, _content: &str) -> Result<()> {
        Err(anyhow::anyhow!(
            "C++ AST parsing not available - compile with 'cpp-ast' feature"
        ))
    }

    #[cfg(feature = "cpp-ast")]
    fn convert_to_dag(&self, tree: &Tree, content: &str) -> AstDag {
        let mut dag = AstDag::new();
        let root = tree.root_node();
        let mut visitor = CTreeSitterVisitor::new(&mut dag, content, Language::Cpp);
        visitor.visit_node(&root, None);
        dag
    }
}

#[async_trait]
impl LanguageStrategy for CppStrategy {
    fn language(&self) -> Language {
        Language::Cpp
    }

    fn can_parse(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| {
                matches!(
                    ext,
                    "cpp" | "cc" | "cxx" | "hpp" | "hh" | "hxx" | "cu" | "cuh"
                )
            })
    }

    #[cfg(feature = "cpp-ast")]
    async fn parse_file(&self, _path: &Path, content: &str) -> Result<AstDag> {
        let tree = self.parse_with_tree_sitter(content)?;
        Ok(self.convert_to_dag(&tree, content))
    }

    #[cfg(not(feature = "cpp-ast"))]
    async fn parse_file(&self, _path: &Path, _content: &str) -> Result<AstDag> {
        Err(anyhow::anyhow!(
            "C++ AST parsing not available - compile with 'cpp-ast' feature"
        ))
    }

    // Delegate to C strategy since the AST structure is similar
    fn extract_imports(&self, ast: &AstDag) -> Vec<String> {
        CStrategy::new().extract_imports(ast)
    }

    fn extract_functions(&self, ast: &AstDag) -> Vec<UnifiedAstNode> {
        CStrategy::new().extract_functions(ast)
    }

    fn extract_types(&self, ast: &AstDag) -> Vec<UnifiedAstNode> {
        CStrategy::new().extract_types(ast)
    }

    fn calculate_complexity(&self, ast: &AstDag) -> (u32, u32) {
        CStrategy::new().calculate_complexity(ast)
    }
}
