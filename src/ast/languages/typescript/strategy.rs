#![cfg_attr(coverage_nightly, coverage(off))]
//! TypeScript language parsing strategy

use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;

#[cfg(feature = "typescript-ast")]
use swc_common::{FileName, SourceMap};
#[cfg(feature = "typescript-ast")]
use swc_ecma_ast::Module;
#[cfg(feature = "typescript-ast")]
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};

use super::super::LanguageStrategy;
use crate::ast::core::{AstDag, AstKind, Language, NodeFlags, UnifiedAstNode};

use super::visitor::TypeScriptAstVisitor;

/// TypeScript language parsing strategy
pub struct TypeScriptStrategy;

impl Default for TypeScriptStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeScriptStrategy {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    fn parse_module(&self, content: &str, filename: &str) -> Result<Module> {
        debug_assert!(!content.is_empty(), "content must not be empty");
        debug_assert!(!filename.is_empty(), "filename must not be empty");
        let source_map = SourceMap::default();
        let source_file = source_map.new_source_file(
            FileName::Custom(filename.to_string()).into(),
            content.to_string(),
        );

        // In swc 24.x, Syntax::Typescript takes a direct config
        let syntax = if filename.ends_with(".tsx") {
            Syntax::Typescript(swc_ecma_parser::TsSyntax {
                tsx: true,
                decorators: true,
                ..Default::default()
            })
        } else if filename.ends_with(".ts") {
            Syntax::Typescript(swc_ecma_parser::TsSyntax {
                tsx: false,
                decorators: true,
                ..Default::default()
            })
        } else {
            // JavaScript
            Syntax::Es(swc_ecma_parser::EsSyntax {
                decorators: true,
                ..Default::default()
            })
        };

        let lexer = Lexer::new(
            syntax,
            Default::default(),
            StringInput::from(&*source_file),
            None,
        );

        let mut parser = Parser::new_from(lexer);
        parser
            .parse_module()
            .map_err(|e| anyhow::anyhow!("TypeScript parse error: {e:?}"))
    }

    pub(crate) fn convert_to_dag(&self, module: &Module, language: Language) -> AstDag {
        let mut dag = AstDag::new();
        let mut visitor = TypeScriptAstVisitor::new(&mut dag, language);
        visitor.visit_module(module);
        dag
    }
}

#[async_trait]
impl LanguageStrategy for TypeScriptStrategy {
    fn language(&self) -> Language {
        Language::TypeScript
    }

    fn can_parse(&self, path: &Path) -> bool {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| matches!(ext, "ts" | "tsx"))
    }

    async fn parse_file(&self, path: &Path, content: &str) -> Result<AstDag> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let filename = path.display().to_string();
        let module = self.parse_module(content, &filename)?;
        Ok(self.convert_to_dag(&module, Language::TypeScript))
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
