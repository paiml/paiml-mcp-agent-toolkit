//! Python language AST parsing strategy

use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;

#[cfg(feature = "python-ast")]
use rustpython_parser::{ast, Parse};

use super::LanguageStrategy;
use crate::ast::core::{
    AstDag, AstKind, ClassKind, FunctionKind, Language, NodeFlags, UnifiedAstNode,
};

/// Python language parsing strategy
pub struct PythonStrategy {
    // Configuration options can be added here
}

impl Default for PythonStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl PythonStrategy {
    pub fn new() -> Self {
        Self {}
    }

    fn convert_to_dag(&self, module: &ast::ModModule) -> AstDag {
        let mut dag = AstDag::new();
        let mut visitor = PythonAstVisitor::new(&mut dag);
        visitor.visit_module(module);
        dag
    }
}

#[async_trait]
impl LanguageStrategy for PythonStrategy {
    fn language(&self) -> Language {
        Language::Python
    }

    fn can_parse(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext == "py" || ext == "pyi")
            .unwrap_or(false)
    }

    async fn parse_file(&self, path: &Path, content: &str) -> Result<AstDag> {
        let filename = path.display().to_string();
        let module = ast::ModModule::parse(content, &filename)
            .map_err(|e| anyhow::anyhow!("Python parse error: {}", e))?;

        Ok(self.convert_to_dag(&module))
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

/// Visitor for converting Python AST to unified AST
struct PythonAstVisitor<'a> {
    dag: &'a mut AstDag,
    current_parent: Option<u32>,
}

impl<'a> PythonAstVisitor<'a> {
    fn new(dag: &'a mut AstDag) -> Self {
        Self {
            dag,
            current_parent: None,
        }
    }

    #[allow(dead_code)]
    fn add_node(&mut self, kind: AstKind) -> u32 {
        let mut node = UnifiedAstNode::new(kind, Language::Python);

        if let Some(parent) = self.current_parent {
            node.parent = parent;
        }

        self.dag.add_node(node)
    }

    fn visit_module(&mut self, module: &ast::ModModule) {
        for stmt in &module.body {
            self.visit_stmt(stmt);
        }
    }

    fn visit_stmt(&mut self, stmt: &ast::Stmt) {
        match stmt {
            ast::Stmt::FunctionDef(f) => {
                let mut node =
                    UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Python);

                // Check for async decorator
                for decorator in &f.decorator_list {
                    if let ast::Expr::Name(name) = &decorator {
                        if name.id.as_str() == "async" {
                            node.flags.set(NodeFlags::ASYNC);
                        }
                    }
                }

                let key = self.dag.add_node(node);

                let old_parent = self.current_parent;
                self.current_parent = Some(key);

                for stmt in &f.body {
                    self.visit_stmt(stmt);
                }

                self.current_parent = old_parent;
            }
            ast::Stmt::AsyncFunctionDef(f) => {
                let mut node =
                    UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Python);
                node.flags.set(NodeFlags::ASYNC);

                let key = self.dag.add_node(node);

                let old_parent = self.current_parent;
                self.current_parent = Some(key);

                for stmt in &f.body {
                    self.visit_stmt(stmt);
                }

                self.current_parent = old_parent;
            }
            ast::Stmt::ClassDef(c) => {
                let node =
                    UnifiedAstNode::new(AstKind::Class(ClassKind::Regular), Language::Python);

                let key = self.dag.add_node(node);

                let old_parent = self.current_parent;
                self.current_parent = Some(key);

                for stmt in &c.body {
                    self.visit_stmt(stmt);
                }

                self.current_parent = old_parent;
            }
            ast::Stmt::Import(_) | ast::Stmt::ImportFrom(_) => {
                let mut node = UnifiedAstNode::new(
                    AstKind::Import(crate::ast::core::ImportKind::Module),
                    Language::Python,
                );
                node.flags.set(NodeFlags::IMPORT);
                self.dag.add_node(node);
            }
            ast::Stmt::If(_) | ast::Stmt::While(_) | ast::Stmt::For(_) => {
                let mut node = UnifiedAstNode::new(
                    AstKind::Statement(crate::ast::core::StmtKind::If),
                    Language::Python,
                );
                node.flags.set(NodeFlags::CONTROL_FLOW);
                self.dag.add_node(node);
            }
            _ => {
                // Handle other statement types as needed
            }
        }
    }
}
