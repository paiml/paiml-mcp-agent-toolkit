//! Python language AST parsing strategy

use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;

// Legacy rustpython-parser support (will be removed in future)
// DEPRECATED: Temporarily disabled while migrating to tree-sitter
// #[cfg(feature = "python-ast")]
// use rustpython_parser::{ast, Parse};

// Modern tree-sitter-python support (preferred)
#[cfg(feature = "python-ast")]
use tree_sitter::{Parser as TsParser, Tree};

use super::LanguageStrategy;
use crate::ast::core::{
    AstDag, AstKind, ClassKind, FunctionKind, ImportKind, Language, NodeFlags, StmtKind,
    UnifiedAstNode,
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
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }

    // Tree-sitter-python parsing (modern, preferred)
    #[cfg(feature = "python-ast")]
    fn parse_with_tree_sitter(&self, content: &str) -> Result<Tree> {
        let mut parser = TsParser::new();
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .map_err(|e| anyhow::anyhow!("Failed to set Python language: {e}"))?;

        let tree = parser
            .parse(content, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse Python code"))?;

        // Check for syntax errors in the tree
        if Self::has_syntax_errors(&tree) {
            return Err(anyhow::anyhow!("Python syntax error detected in source"));
        }

        Ok(tree)
    }

    #[cfg(feature = "python-ast")]
    fn has_syntax_errors(tree: &Tree) -> bool {
        let root = tree.root_node();
        Self::node_has_error(&root)
    }

    #[cfg(feature = "python-ast")]
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

    #[cfg(not(feature = "python-ast"))]
    fn parse_with_tree_sitter(&self, _content: &str) -> Result<Tree> {
        Err(anyhow::anyhow!(
            "Python AST parsing not available - compile with 'python-ast' feature"
        ))
    }

    #[cfg(feature = "python-ast")]
    fn convert_tree_to_dag(&self, tree: &Tree, content: &str) -> AstDag {
        let mut dag = AstDag::new();
        let root = tree.root_node();
        let mut visitor = PythonTreeSitterVisitor::new(&mut dag, content);
        visitor.visit_node(&root, None);
        dag
    }

    // Legacy rustpython-parser conversion (DEPRECATED - commented out during migration)
    // #[cfg(feature = "python-ast")]
    // fn convert_to_dag(&self, module: &ast::ModModule) -> AstDag {
    //     let mut dag = AstDag::new();
    //     let mut visitor = PythonAstVisitor::new(&mut dag);
    //     visitor.visit_module(module);
    //     dag
    // }
}

#[async_trait]
impl LanguageStrategy for PythonStrategy {
    fn language(&self) -> Language {
        Language::Python
    }

    fn can_parse(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext == "py" || ext == "pyi")
    }

    async fn parse_file(&self, _path: &Path, content: &str) -> Result<AstDag> {
        // Use tree-sitter-python (modern approach)
        let tree = self.parse_with_tree_sitter(content)?;
        Ok(self.convert_tree_to_dag(&tree, content))

        // Legacy rustpython-parser approach (commented out for now)
        // #[cfg(all(feature = "python-ast", not(feature = "python-treesitter")))]
        // {
        //     let filename = path.display().to_string();
        //     let module = ast::ModModule::parse(content, &filename)
        //         .map_err(|e| anyhow::anyhow!("Python parse error: {e}"))?;
        //     Ok(self.convert_to_dag(&module))
        // }
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

/// DEPRECATED: Old rustpython-parser visitor (commented out during migration to tree-sitter)
// struct PythonAstVisitor<'a> {
//     dag: &'a mut AstDag,
//     current_parent: Option<u32>,
// }

/*
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
*/

/// Tree-sitter-based Python AST visitor (modern approach)
#[cfg(feature = "python-ast")]
struct PythonTreeSitterVisitor<'a> {
    dag: &'a mut AstDag,
    #[allow(dead_code)]
    content: &'a str,
    current_parent: Option<u32>,
}

#[cfg(feature = "python-ast")]
impl<'a> PythonTreeSitterVisitor<'a> {
    fn new(dag: &'a mut AstDag, content: &'a str) -> Self {
        Self {
            dag,
            content,
            current_parent: None,
        }
    }

    fn add_node(&mut self, kind: AstKind) -> u32 {
        let mut node = UnifiedAstNode::new(kind, Language::Python);

        if let Some(parent) = self.current_parent {
            node.parent = parent;
        }

        self.dag.add_node(node)
    }

    fn visit_node(&mut self, node: &tree_sitter::Node, parent: Option<u32>) {
        let old_parent = self.current_parent;
        self.current_parent = parent;

        match node.kind() {
            "function_definition" => {
                // Regular function or async function
                let key = self.add_node(AstKind::Function(FunctionKind::Regular));

                self.current_parent = Some(key);
                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child, Some(key));
                }
            }
            "class_definition" => {
                let key = self.add_node(AstKind::Class(ClassKind::Regular));

                self.current_parent = Some(key);
                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child, Some(key));
                }
            }
            "import_statement" | "import_from_statement" => {
                let mut n = UnifiedAstNode::new(AstKind::Import(ImportKind::Module), Language::Python);
                n.flags.set(NodeFlags::IMPORT);
                self.dag.add_node(n);

                // Still visit children for completeness
                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child, parent);
                }
            }
            "if_statement" | "while_statement" | "for_statement" | "match_statement" | "try_statement" => {
                let mut n = UnifiedAstNode::new(AstKind::Statement(StmtKind::If), Language::Python);
                n.flags.set(NodeFlags::CONTROL_FLOW);
                self.dag.add_node(n);

                // Visit children to capture nested control flow
                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child, parent);
                }
            }
            "elif_clause" | "else_clause" | "except_clause" => {
                // Additional decision points for cognitive complexity
                let mut n = UnifiedAstNode::new(AstKind::Statement(StmtKind::If), Language::Python);
                n.flags.set(NodeFlags::CONTROL_FLOW);
                self.dag.add_node(n);

                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child, parent);
                }
            }
            "lambda" => {
                // Lambda functions
                let key = self.add_node(AstKind::Function(FunctionKind::Lambda));

                self.current_parent = Some(key);
                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child, Some(key));
                }
            }
            "with_statement" | "assert_statement" => {
                // Statements that add complexity
                let n = UnifiedAstNode::new(AstKind::Statement(StmtKind::Block), Language::Python);
                self.dag.add_node(n);

                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child, parent);
                }
            }
            "boolean_operator" | "comparison_operator" => {
                // Logical operators add to cyclomatic complexity
                let mut n = UnifiedAstNode::new(AstKind::Statement(StmtKind::Block), Language::Python);
                n.flags.set(NodeFlags::CONTROL_FLOW);
                self.dag.add_node(n);

                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child, parent);
                }
            }
            "conditional_expression" => {
                // Ternary expressions (a if b else c)
                let mut n = UnifiedAstNode::new(AstKind::Statement(StmtKind::If), Language::Python);
                n.flags.set(NodeFlags::CONTROL_FLOW);
                self.dag.add_node(n);

                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child, parent);
                }
            }
            "list_comprehension" | "set_comprehension" | "dictionary_comprehension" | "generator_expression" => {
                // Comprehensions add cognitive complexity
                let mut n = UnifiedAstNode::new(AstKind::Statement(StmtKind::For), Language::Python);
                n.flags.set(NodeFlags::CONTROL_FLOW);
                self.dag.add_node(n);

                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child, parent);
                }
            }
            _ => {
                // Visit children for other node types
                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child, parent);
                }
            }
        }

        self.current_parent = old_parent;
    }
}
