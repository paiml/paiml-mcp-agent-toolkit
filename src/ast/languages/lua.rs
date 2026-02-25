#![cfg_attr(coverage_nightly, coverage(off))]
//! Lua language AST parsing strategy

use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;

#[cfg(feature = "lua-ast")]
use tree_sitter::{Parser as TsParser, Tree};

use super::LanguageStrategy;
use crate::ast::core::{AstDag, AstKind, Language, NodeFlags, UnifiedAstNode};

#[cfg(feature = "lua-ast")]
use crate::ast::core::{ClassKind, FunctionKind, ImportKind, StmtKind, VarKind};

/// Lua language parsing strategy
pub struct LuaStrategy;

impl Default for LuaStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl LuaStrategy {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    #[cfg(feature = "lua-ast")]
    fn parse_with_tree_sitter(&self, content: &str) -> Result<Tree> {
        let mut parser = TsParser::new();
        parser
            .set_language(&tree_sitter_lua::LANGUAGE.into())
            .map_err(|e| anyhow::anyhow!("Failed to set Lua language: {e}"))?;

        let tree = parser
            .parse(content, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse Lua code"))?;

        if Self::has_syntax_errors(&tree) {
            return Err(anyhow::anyhow!("Lua syntax error detected in source"));
        }

        Ok(tree)
    }

    #[cfg(feature = "lua-ast")]
    fn has_syntax_errors(tree: &Tree) -> bool {
        let root = tree.root_node();
        Self::node_has_error(&root)
    }

    #[cfg(feature = "lua-ast")]
    fn node_has_error(node: &tree_sitter::Node) -> bool {
        if node.kind() == "ERROR" || node.is_error() || node.is_missing() {
            return true;
        }

        for child in node.children(&mut node.walk()) {
            if Self::node_has_error(&child) {
                return true;
            }
        }

        false
    }

    #[cfg(not(feature = "lua-ast"))]
    #[allow(dead_code)]
    fn parse_with_tree_sitter(&self, _content: &str) -> Result<()> {
        Err(anyhow::anyhow!(
            "Lua AST parsing not available - compile with 'lua-ast' feature"
        ))
    }

    #[cfg(feature = "lua-ast")]
    fn convert_tree_to_dag(&self, tree: &Tree, content: &str) -> AstDag {
        let mut dag = AstDag::new();
        let root = tree.root_node();
        let mut visitor = LuaTreeSitterVisitor::new(&mut dag, content);
        visitor.visit_node(&root, None);
        dag
    }
}

#[async_trait]
impl LanguageStrategy for LuaStrategy {
    fn language(&self) -> Language {
        Language::Lua
    }

    fn can_parse(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext == "lua")
    }

    #[cfg(feature = "lua-ast")]
    async fn parse_file(&self, _path: &Path, content: &str) -> Result<AstDag> {
        let tree = self.parse_with_tree_sitter(content)?;
        Ok(self.convert_tree_to_dag(&tree, content))
    }

    #[cfg(not(feature = "lua-ast"))]
    async fn parse_file(&self, _path: &Path, _content: &str) -> Result<AstDag> {
        Err(anyhow::anyhow!(
            "Lua AST parsing not available - compile with 'lua-ast' feature"
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

// Tree-sitter visitor for converting Lua parse trees to AstDag
include!("lua_visitor.rs");

// Tests
include!("lua_tests.rs");
