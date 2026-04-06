#![cfg_attr(coverage_nightly, coverage(off))]
//! JavaScript language parsing strategy

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
use super::strategy::TypeScriptStrategy;
use crate::ast::core::{AstDag, Language, UnifiedAstNode};

/// JavaScript language parsing strategy
pub struct JavaScriptStrategy;

impl Default for JavaScriptStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl JavaScriptStrategy {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    fn parse_module(&self, content: &str, filename: &str) -> Result<Module> {
        let source_map = SourceMap::default();
        let source_file = source_map.new_source_file(
            FileName::Custom(filename.to_string()).into(),
            content.to_string(),
        );

        let lexer = Lexer::new(
            Syntax::Es(swc_ecma_parser::EsSyntax {
                jsx: filename.ends_with(".jsx"),
                decorators: true,
                ..Default::default()
            }),
            Default::default(),
            StringInput::from(&*source_file),
            None,
        );

        let mut parser = Parser::new_from(lexer);
        parser
            .parse_module()
            .map_err(|e| anyhow::anyhow!("JavaScript parse error: {e:?}"))
    }
}

#[async_trait]
impl LanguageStrategy for JavaScriptStrategy {
    fn language(&self) -> Language {
        Language::JavaScript
    }

    fn can_parse(&self, path: &Path) -> bool {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| matches!(ext, "js" | "jsx" | "mjs"))
    }

    async fn parse_file(&self, path: &Path, content: &str) -> Result<AstDag> {
        let filename = path.display().to_string();
        let module = self.parse_module(content, &filename)?;
        let ts_strategy = TypeScriptStrategy::new();
        Ok(ts_strategy.convert_to_dag(&module, Language::JavaScript))
    }

    // Delegate other methods to TypeScript strategy since the AST is the same
    fn extract_imports(&self, ast: &AstDag) -> Vec<String> {
        TypeScriptStrategy::new().extract_imports(ast)
    }

    fn extract_functions(&self, ast: &AstDag) -> Vec<UnifiedAstNode> {
        TypeScriptStrategy::new().extract_functions(ast)
    }

    fn extract_types(&self, ast: &AstDag) -> Vec<UnifiedAstNode> {
        TypeScriptStrategy::new().extract_types(ast)
    }

    fn calculate_complexity(&self, ast: &AstDag) -> (u32, u32) {
        TypeScriptStrategy::new().calculate_complexity(ast)
    }
}
