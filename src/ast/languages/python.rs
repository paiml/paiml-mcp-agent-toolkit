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
use crate::ast::core::{AstDag, AstKind, Language, NodeFlags, UnifiedAstNode};

#[cfg(feature = "python-ast")]
use crate::ast::core::{ClassKind, FunctionKind, ImportKind, StmtKind};

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
    #[allow(dead_code)]
    fn parse_with_tree_sitter(&self, _content: &str) -> Result<()> {
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

    #[cfg(feature = "python-ast")]
    async fn parse_file(&self, _path: &Path, content: &str) -> Result<AstDag> {
        // Use tree-sitter-python (modern approach)
        let tree = self.parse_with_tree_sitter(content)?;
        Ok(self.convert_tree_to_dag(&tree, content))
    }

    #[cfg(not(feature = "python-ast"))]
    async fn parse_file(&self, _path: &Path, _content: &str) -> Result<AstDag> {
        Err(anyhow::anyhow!(
            "Python AST parsing not available - compile with 'python-ast' feature"
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

// DEPRECATED: Old rustpython-parser visitor (commented out during migration to tree-sitter)
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
                let mut n =
                    UnifiedAstNode::new(AstKind::Import(ImportKind::Module), Language::Python);
                n.flags.set(NodeFlags::IMPORT);
                self.dag.add_node(n);

                // Still visit children for completeness
                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child, parent);
                }
            }
            "if_statement" | "while_statement" | "for_statement" | "match_statement"
            | "try_statement" => {
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
                let mut n =
                    UnifiedAstNode::new(AstKind::Statement(StmtKind::Block), Language::Python);
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
            "list_comprehension"
            | "set_comprehension"
            | "dictionary_comprehension"
            | "generator_expression" => {
                // Comprehensions add cognitive complexity
                let mut n =
                    UnifiedAstNode::new(AstKind::Statement(StmtKind::For), Language::Python);
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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;
    use std::path::PathBuf;

    // Test PythonStrategy construction and defaults
    #[test]
    fn test_python_strategy_new() {
        let strategy = PythonStrategy::new();
        assert_eq!(strategy.language(), Language::Python);
    }

    #[test]
    fn test_python_strategy_default() {
        let strategy = PythonStrategy::default();
        assert_eq!(strategy.language(), Language::Python);
    }

    // Test can_parse for various file extensions
    #[test]
    fn test_can_parse_py_file() {
        let strategy = PythonStrategy::new();
        assert!(strategy.can_parse(Path::new("test.py")));
        assert!(strategy.can_parse(Path::new("/path/to/module.py")));
        assert!(strategy.can_parse(Path::new("script.py")));
    }

    #[test]
    fn test_can_parse_pyi_file() {
        let strategy = PythonStrategy::new();
        assert!(strategy.can_parse(Path::new("test.pyi")));
        assert!(strategy.can_parse(Path::new("stub.pyi")));
    }

    #[test]
    fn test_can_parse_non_python_files() {
        let strategy = PythonStrategy::new();
        assert!(!strategy.can_parse(Path::new("test.rs")));
        assert!(!strategy.can_parse(Path::new("test.ts")));
        assert!(!strategy.can_parse(Path::new("test.js")));
        assert!(!strategy.can_parse(Path::new("test.c")));
        assert!(!strategy.can_parse(Path::new("test")));
        assert!(!strategy.can_parse(Path::new("")));
        assert!(!strategy.can_parse(Path::new("test.pyc")));
    }

    #[test]
    fn test_can_parse_no_extension() {
        let strategy = PythonStrategy::new();
        assert!(!strategy.can_parse(Path::new("Makefile")));
        assert!(!strategy.can_parse(Path::new("README")));
    }

    // Test extract_imports
    #[test]
    fn test_extract_imports_empty() {
        let strategy = PythonStrategy::new();
        let dag = AstDag::new();
        let imports = strategy.extract_imports(&dag);
        assert!(imports.is_empty());
    }

    #[test]
    fn test_extract_imports_with_nodes() {
        let strategy = PythonStrategy::new();
        let mut dag = AstDag::new();

        // Add a node with Import kind
        let node = UnifiedAstNode::new(
            AstKind::Import(crate::ast::core::ImportKind::Module),
            Language::Python,
        );
        dag.add_node(node);

        let imports = strategy.extract_imports(&dag);
        assert_eq!(imports.len(), 1);
        assert!(imports[0].starts_with("import_"));
    }

    // Test extract_functions
    #[test]
    fn test_extract_functions_empty_dag() {
        let strategy = PythonStrategy::new();
        let dag = AstDag::new();
        let functions = strategy.extract_functions(&dag);
        assert!(functions.is_empty());
    }

    #[test]
    fn test_extract_functions_with_function_nodes() {
        let strategy = PythonStrategy::new();
        let mut dag = AstDag::new();

        // Add function nodes
        let node1 = UnifiedAstNode::new(
            AstKind::Function(crate::ast::core::FunctionKind::Regular),
            Language::Python,
        );
        dag.add_node(node1);

        let node2 = UnifiedAstNode::new(
            AstKind::Function(crate::ast::core::FunctionKind::Lambda),
            Language::Python,
        );
        dag.add_node(node2);

        let functions = strategy.extract_functions(&dag);
        assert_eq!(functions.len(), 2);
    }

    // Test extract_types
    #[test]
    fn test_extract_types_empty_dag() {
        let strategy = PythonStrategy::new();
        let dag = AstDag::new();
        let types = strategy.extract_types(&dag);
        assert!(types.is_empty());
    }

    #[test]
    fn test_extract_types_with_class_nodes() {
        let strategy = PythonStrategy::new();
        let mut dag = AstDag::new();

        // Add class node
        let node = UnifiedAstNode::new(
            AstKind::Class(crate::ast::core::ClassKind::Regular),
            Language::Python,
        );
        dag.add_node(node);

        let types = strategy.extract_types(&dag);
        assert_eq!(types.len(), 1);
    }

    // Test calculate_complexity
    #[test]
    fn test_calculate_complexity_empty_dag() {
        let strategy = PythonStrategy::new();
        let dag = AstDag::new();
        let (cyclomatic, cognitive) = strategy.calculate_complexity(&dag);
        assert_eq!(cyclomatic, 1); // Base complexity
        assert_eq!(cognitive, 0);
    }

    #[test]
    fn test_calculate_complexity_with_control_flow() {
        let strategy = PythonStrategy::new();
        let mut dag = AstDag::new();

        // Add nodes with CONTROL_FLOW flag
        let mut node1 = UnifiedAstNode::new(
            AstKind::Statement(crate::ast::core::StmtKind::If),
            Language::Python,
        );
        node1.flags.set(NodeFlags::CONTROL_FLOW);
        dag.add_node(node1);

        let mut node2 = UnifiedAstNode::new(
            AstKind::Statement(crate::ast::core::StmtKind::For),
            Language::Python,
        );
        node2.flags.set(NodeFlags::CONTROL_FLOW);
        dag.add_node(node2);

        let mut node3 = UnifiedAstNode::new(
            AstKind::Statement(crate::ast::core::StmtKind::While),
            Language::Python,
        );
        node3.flags.set(NodeFlags::CONTROL_FLOW);
        dag.add_node(node3);

        let (cyclomatic, cognitive) = strategy.calculate_complexity(&dag);
        assert_eq!(cyclomatic, 4); // 1 base + 3 control flow
        assert_eq!(cognitive, 3);
    }

    // Tests requiring python-ast feature
    #[cfg(feature = "python-ast")]
    mod python_ast_tests {
        use super::*;

        #[test]
        fn test_parse_with_tree_sitter_simple_function() {
            let strategy = PythonStrategy::new();
            let code = "def hello():\n    pass";
            let result = strategy.parse_with_tree_sitter(code);
            assert!(result.is_ok());
        }

        #[test]
        fn test_parse_with_tree_sitter_class() {
            let strategy = PythonStrategy::new();
            let code = "class MyClass:\n    def __init__(self):\n        pass";
            let result = strategy.parse_with_tree_sitter(code);
            assert!(result.is_ok());
        }

        #[test]
        fn test_parse_with_tree_sitter_imports() {
            let strategy = PythonStrategy::new();
            let code = "import os\nfrom collections import defaultdict";
            let result = strategy.parse_with_tree_sitter(code);
            assert!(result.is_ok());
        }

        #[test]
        fn test_parse_with_tree_sitter_control_flow() {
            let strategy = PythonStrategy::new();
            let code = r#"
def foo(x):
    if x > 0:
        return x
    elif x < 0:
        return -x
    else:
        return 0
"#;
            let result = strategy.parse_with_tree_sitter(code);
            assert!(result.is_ok());
        }

        #[test]
        fn test_parse_with_tree_sitter_syntax_error() {
            let strategy = PythonStrategy::new();
            let code = "def foo(\n    pass"; // Invalid syntax
            let result = strategy.parse_with_tree_sitter(code);
            assert!(result.is_err());
        }

        #[test]
        fn test_convert_tree_to_dag_function() {
            let strategy = PythonStrategy::new();
            let code = "def hello():\n    pass";
            let tree = strategy.parse_with_tree_sitter(code).unwrap();
            let dag = strategy.convert_tree_to_dag(&tree, code);

            let has_function = dag
                .nodes
                .iter()
                .any(|node| matches!(node.kind, AstKind::Function(_)));
            assert!(has_function, "Should have a function node");
        }

        #[test]
        fn test_convert_tree_to_dag_class() {
            let strategy = PythonStrategy::new();
            let code = "class MyClass:\n    pass";
            let tree = strategy.parse_with_tree_sitter(code).unwrap();
            let dag = strategy.convert_tree_to_dag(&tree, code);

            let has_class = dag
                .nodes
                .iter()
                .any(|node| matches!(node.kind, AstKind::Class(_)));
            assert!(has_class, "Should have a class node");
        }

        #[test]
        fn test_convert_tree_to_dag_import() {
            let strategy = PythonStrategy::new();
            let code = "import os";
            let tree = strategy.parse_with_tree_sitter(code).unwrap();
            let dag = strategy.convert_tree_to_dag(&tree, code);

            let has_import = dag
                .nodes
                .iter()
                .any(|node| matches!(node.kind, AstKind::Import(_)));
            assert!(has_import, "Should have an import node");
        }

        #[test]
        fn test_convert_tree_to_dag_import_from() {
            let strategy = PythonStrategy::new();
            let code = "from os import path";
            let tree = strategy.parse_with_tree_sitter(code).unwrap();
            let dag = strategy.convert_tree_to_dag(&tree, code);

            let has_import = dag
                .nodes
                .iter()
                .any(|node| matches!(node.kind, AstKind::Import(_)));
            assert!(has_import, "Should have an import node");
        }

        #[test]
        fn test_convert_tree_to_dag_control_flow() {
            let strategy = PythonStrategy::new();
            let code = "if True:\n    pass\nelse:\n    pass";
            let tree = strategy.parse_with_tree_sitter(code).unwrap();
            let dag = strategy.convert_tree_to_dag(&tree, code);

            let has_control_flow = dag
                .nodes
                .iter()
                .any(|node| node.flags.has(NodeFlags::CONTROL_FLOW));
            assert!(has_control_flow, "Should have control flow nodes");
        }

        #[test]
        fn test_convert_tree_to_dag_lambda() {
            let strategy = PythonStrategy::new();
            let code = "f = lambda x: x * 2";
            let tree = strategy.parse_with_tree_sitter(code).unwrap();
            let dag = strategy.convert_tree_to_dag(&tree, code);

            let has_lambda = dag
                .nodes
                .iter()
                .any(|node| matches!(node.kind, AstKind::Function(FunctionKind::Lambda)));
            assert!(has_lambda, "Should have a lambda node");
        }

        #[test]
        fn test_convert_tree_to_dag_comprehension() {
            let strategy = PythonStrategy::new();
            let code = "result = [x * 2 for x in range(10)]";
            let tree = strategy.parse_with_tree_sitter(code).unwrap();
            let dag = strategy.convert_tree_to_dag(&tree, code);

            // Comprehensions add to control flow
            let has_control_flow = dag
                .nodes
                .iter()
                .any(|node| node.flags.has(NodeFlags::CONTROL_FLOW));
            assert!(
                has_control_flow,
                "Should have control flow from comprehension"
            );
        }

        #[test]
        fn test_convert_tree_to_dag_try_except() {
            let strategy = PythonStrategy::new();
            let code = r#"
try:
    pass
except Exception:
    pass
"#;
            let tree = strategy.parse_with_tree_sitter(code).unwrap();
            let dag = strategy.convert_tree_to_dag(&tree, code);

            let control_flow_count = dag
                .nodes
                .iter()
                .filter(|node| node.flags.has(NodeFlags::CONTROL_FLOW))
                .count();
            assert!(
                control_flow_count >= 2,
                "Should have control flow from try/except"
            );
        }

        #[test]
        fn test_convert_tree_to_dag_with_statement() {
            let strategy = PythonStrategy::new();
            let code = "with open('file.txt') as f:\n    pass";
            let tree = strategy.parse_with_tree_sitter(code).unwrap();
            let dag = strategy.convert_tree_to_dag(&tree, code);

            // with statement adds a block node
            assert!(!dag.nodes.is_empty());
        }

        #[test]
        fn test_convert_tree_to_dag_conditional_expression() {
            let strategy = PythonStrategy::new();
            let code = "x = 1 if True else 0";
            let tree = strategy.parse_with_tree_sitter(code).unwrap();
            let dag = strategy.convert_tree_to_dag(&tree, code);

            let has_control_flow = dag
                .nodes
                .iter()
                .any(|node| node.flags.has(NodeFlags::CONTROL_FLOW));
            assert!(
                has_control_flow,
                "Conditional expression should add control flow"
            );
        }

        #[tokio::test]
        async fn test_parse_file_success() {
            let strategy = PythonStrategy::new();
            let path = PathBuf::from("test.py");
            let code = "def hello():\n    print('Hello!')";
            let result = strategy.parse_file(&path, code).await;
            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_parse_file_complex() {
            let strategy = PythonStrategy::new();
            let path = PathBuf::from("test.py");
            let code = r#"
import os
from typing import List

class Calculator:
    def __init__(self):
        self.result = 0

    def add(self, x: int, y: int) -> int:
        if x < 0:
            return y
        elif y < 0:
            return x
        else:
            return x + y

    @staticmethod
    def multiply(x, y):
        return x * y

def main():
    calc = Calculator()
    numbers = [i * 2 for i in range(10) if i % 2 == 0]
    total = sum(numbers)
    print(f"Total: {total}")
"#;
            let result = strategy.parse_file(&path, code).await;
            assert!(result.is_ok());

            let dag = result.unwrap();
            let functions = strategy.extract_functions(&dag);
            let types = strategy.extract_types(&dag);

            // Should have functions: __init__, add, multiply, main
            assert!(functions.len() >= 4);
            // Should have class: Calculator
            assert!(types.len() >= 1);
        }

        #[tokio::test]
        async fn test_parse_file_error() {
            let strategy = PythonStrategy::new();
            let path = PathBuf::from("test.py");
            let code = "def foo(\n    pass"; // Invalid syntax
            let result = strategy.parse_file(&path, code).await;
            assert!(result.is_err());
        }

        #[test]
        fn test_has_syntax_errors_valid() {
            let strategy = PythonStrategy::new();
            let code = "def hello():\n    pass";
            let tree = strategy.parse_with_tree_sitter(code).unwrap();
            assert!(!PythonStrategy::has_syntax_errors(&tree));
        }

        #[test]
        fn test_visitor_add_node() {
            let mut dag = AstDag::new();
            let content = "def foo(): pass";
            let mut visitor = PythonTreeSitterVisitor::new(&mut dag, content);

            let key = visitor.add_node(AstKind::Function(FunctionKind::Regular));
            assert_eq!(key, 0);
            assert_eq!(dag.nodes.len(), 1);
        }

        #[test]
        fn test_visitor_add_node_with_parent() {
            let mut dag = AstDag::new();
            let content = "class Foo:\n    def bar(): pass";
            let mut visitor = PythonTreeSitterVisitor::new(&mut dag, content);

            // Add parent node
            let parent_key = visitor.add_node(AstKind::Class(ClassKind::Regular));
            visitor.current_parent = Some(parent_key);

            // Add child node
            let child_key = visitor.add_node(AstKind::Function(FunctionKind::Method));

            // Verify parent is set
            let child_node = dag.nodes.get(child_key).unwrap();
            assert_eq!(child_node.parent, parent_key);
        }
    }

    // Tests when python-ast feature is not enabled
    #[cfg(not(feature = "python-ast"))]
    mod non_python_ast_tests {
        use super::*;

        #[tokio::test]
        async fn test_parse_file_without_feature() {
            let strategy = PythonStrategy::new();
            let path = PathBuf::from("test.py");
            let code = "def hello():\n    pass";
            let result = strategy.parse_file(&path, code).await;
            assert!(result.is_err());
            assert!(result.err().unwrap().to_string().contains("python-ast"));
        }

        #[test]
        fn test_parse_with_tree_sitter_without_feature() {
            let strategy = PythonStrategy::new();
            let code = "def hello():\n    pass";
            let result = strategy.parse_with_tree_sitter(code);
            assert!(result.is_err());
        }
    }
}
