// Tests for Python language AST parsing strategy

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
