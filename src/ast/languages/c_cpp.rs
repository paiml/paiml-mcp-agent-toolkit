//! C and C++ language AST parsing strategies

use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;

#[cfg(feature = "c-ast")]
use tree_sitter::{Parser as TsParser, Tree};

use super::LanguageStrategy;
use crate::ast::core::{
    AstDag, AstKind, ClassKind, FunctionKind, ImportKind, Language, NodeFlags, StmtKind,
    UnifiedAstNode,
};

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
        let mut parser = TsParser::new();
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
        let mut parser = TsParser::new();
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
            .is_some_and(|ext| matches!(ext, "cpp" | "cc" | "cxx" | "hpp" | "hh" | "hxx"))
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

/// Tree-sitter visitor for C/C++ AST conversion
#[allow(dead_code)]
struct CTreeSitterVisitor<'a> {
    dag: &'a mut AstDag,
    #[allow(dead_code)]
    content: &'a str,
    language: Language,
    current_parent: Option<u32>,
}

#[allow(dead_code)]
impl<'a> CTreeSitterVisitor<'a> {
    fn new(dag: &'a mut AstDag, content: &'a str, language: Language) -> Self {
        Self {
            dag,
            content,
            language,
            current_parent: None,
        }
    }

    fn add_node(&mut self, kind: AstKind) -> u32 {
        let mut node = UnifiedAstNode::new(kind, self.language);

        if let Some(parent) = self.current_parent {
            node.parent = parent;
        }

        self.dag.add_node(node)
    }

    fn visit_node(&mut self, node: &tree_sitter::Node, parent: Option<u32>) {
        let old_parent = self.current_parent;
        self.current_parent = parent;

        match node.kind() {
            "function_definition" | "function_declarator" => {
                let key = self.add_node(AstKind::Function(FunctionKind::Regular));

                self.current_parent = Some(key);
                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child, Some(key));
                }
            }
            "struct_specifier" => {
                let key = self.add_node(AstKind::Class(ClassKind::Struct));

                self.current_parent = Some(key);
                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child, Some(key));
                }
            }
            "enum_specifier" => {
                let key = self.add_node(AstKind::Class(ClassKind::Enum));

                self.current_parent = Some(key);
                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child, Some(key));
                }
            }
            "class_specifier" => {
                // C++ specific
                let key = self.add_node(AstKind::Class(ClassKind::Regular));

                self.current_parent = Some(key);
                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child, Some(key));
                }
            }
            "preproc_include" => {
                let mut n = UnifiedAstNode::new(AstKind::Import(ImportKind::Module), self.language);
                n.flags.set(NodeFlags::IMPORT);
                self.dag.add_node(n);
            }
            "if_statement" | "while_statement" | "for_statement" | "switch_statement" => {
                let mut n = UnifiedAstNode::new(AstKind::Statement(StmtKind::If), self.language);
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

#[cfg(test)]
mod coverage_tests {
    use super::*;
    use std::path::PathBuf;

    // ==================== CStrategy Tests ====================

    #[test]
    fn test_c_strategy_new() {
        let strategy = CStrategy::new();
        assert_eq!(strategy.language(), Language::C);
    }

    #[test]
    fn test_c_strategy_default() {
        let strategy = CStrategy::default();
        assert_eq!(strategy.language(), Language::C);
    }

    #[test]
    fn test_c_can_parse_c_file() {
        let strategy = CStrategy::new();
        assert!(strategy.can_parse(Path::new("test.c")));
        assert!(strategy.can_parse(Path::new("/path/to/source.c")));
    }

    #[test]
    fn test_c_can_parse_h_file() {
        let strategy = CStrategy::new();
        assert!(strategy.can_parse(Path::new("test.h")));
        assert!(strategy.can_parse(Path::new("header.h")));
    }

    #[test]
    fn test_c_can_parse_non_c_files() {
        let strategy = CStrategy::new();
        assert!(!strategy.can_parse(Path::new("test.cpp")));
        assert!(!strategy.can_parse(Path::new("test.rs")));
        assert!(!strategy.can_parse(Path::new("test.py")));
        assert!(!strategy.can_parse(Path::new("test.hpp")));
        assert!(!strategy.can_parse(Path::new("test")));
        assert!(!strategy.can_parse(Path::new("")));
    }

    #[test]
    fn test_c_extract_imports_empty() {
        let strategy = CStrategy::new();
        let dag = AstDag::new();
        let imports = strategy.extract_imports(&dag);
        assert!(imports.is_empty());
    }

    #[test]
    fn test_c_extract_imports_with_nodes() {
        let strategy = CStrategy::new();
        let mut dag = AstDag::new();

        let node = UnifiedAstNode::new(AstKind::Import(ImportKind::Module), Language::C);
        dag.add_node(node);

        let imports = strategy.extract_imports(&dag);
        assert_eq!(imports.len(), 1);
    }

    #[test]
    fn test_c_extract_functions_empty() {
        let strategy = CStrategy::new();
        let dag = AstDag::new();
        let functions = strategy.extract_functions(&dag);
        assert!(functions.is_empty());
    }

    #[test]
    fn test_c_extract_functions_with_nodes() {
        let strategy = CStrategy::new();
        let mut dag = AstDag::new();

        let node = UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::C);
        dag.add_node(node);

        let functions = strategy.extract_functions(&dag);
        assert_eq!(functions.len(), 1);
    }

    #[test]
    fn test_c_extract_types_empty() {
        let strategy = CStrategy::new();
        let dag = AstDag::new();
        let types = strategy.extract_types(&dag);
        assert!(types.is_empty());
    }

    #[test]
    fn test_c_extract_types_with_nodes() {
        let strategy = CStrategy::new();
        let mut dag = AstDag::new();

        let node = UnifiedAstNode::new(AstKind::Class(ClassKind::Struct), Language::C);
        dag.add_node(node);

        let types = strategy.extract_types(&dag);
        assert_eq!(types.len(), 1);
    }

    #[test]
    fn test_c_calculate_complexity_empty() {
        let strategy = CStrategy::new();
        let dag = AstDag::new();
        let (cyclomatic, cognitive) = strategy.calculate_complexity(&dag);
        assert_eq!(cyclomatic, 1);
        assert_eq!(cognitive, 0);
    }

    #[test]
    fn test_c_calculate_complexity_with_control_flow() {
        let strategy = CStrategy::new();
        let mut dag = AstDag::new();

        let mut node = UnifiedAstNode::new(AstKind::Statement(StmtKind::If), Language::C);
        node.flags.set(NodeFlags::CONTROL_FLOW);
        dag.add_node(node);

        let (cyclomatic, cognitive) = strategy.calculate_complexity(&dag);
        assert_eq!(cyclomatic, 2);
        assert_eq!(cognitive, 1);
    }

    // ==================== CppStrategy Tests ====================

    #[test]
    fn test_cpp_strategy_new() {
        let strategy = CppStrategy::new();
        assert_eq!(strategy.language(), Language::Cpp);
    }

    #[test]
    fn test_cpp_strategy_default() {
        let strategy = CppStrategy::default();
        assert_eq!(strategy.language(), Language::Cpp);
    }

    #[test]
    fn test_cpp_can_parse_cpp_file() {
        let strategy = CppStrategy::new();
        assert!(strategy.can_parse(Path::new("test.cpp")));
        assert!(strategy.can_parse(Path::new("test.cc")));
        assert!(strategy.can_parse(Path::new("test.cxx")));
    }

    #[test]
    fn test_cpp_can_parse_hpp_file() {
        let strategy = CppStrategy::new();
        assert!(strategy.can_parse(Path::new("test.hpp")));
        assert!(strategy.can_parse(Path::new("test.hh")));
        assert!(strategy.can_parse(Path::new("test.hxx")));
    }

    #[test]
    fn test_cpp_can_parse_non_cpp_files() {
        let strategy = CppStrategy::new();
        assert!(!strategy.can_parse(Path::new("test.c")));
        assert!(!strategy.can_parse(Path::new("test.h")));
        assert!(!strategy.can_parse(Path::new("test.rs")));
        assert!(!strategy.can_parse(Path::new("test.py")));
        assert!(!strategy.can_parse(Path::new("test")));
    }

    #[test]
    fn test_cpp_extract_imports() {
        let strategy = CppStrategy::new();
        let mut dag = AstDag::new();

        let node = UnifiedAstNode::new(AstKind::Import(ImportKind::Module), Language::Cpp);
        dag.add_node(node);

        let imports = strategy.extract_imports(&dag);
        assert_eq!(imports.len(), 1);
    }

    #[test]
    fn test_cpp_extract_functions() {
        let strategy = CppStrategy::new();
        let mut dag = AstDag::new();

        let node = UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Cpp);
        dag.add_node(node);

        let functions = strategy.extract_functions(&dag);
        assert_eq!(functions.len(), 1);
    }

    #[test]
    fn test_cpp_extract_types() {
        let strategy = CppStrategy::new();
        let mut dag = AstDag::new();

        let node = UnifiedAstNode::new(AstKind::Class(ClassKind::Regular), Language::Cpp);
        dag.add_node(node);

        let types = strategy.extract_types(&dag);
        assert_eq!(types.len(), 1);
    }

    #[test]
    fn test_cpp_calculate_complexity() {
        let strategy = CppStrategy::new();
        let mut dag = AstDag::new();

        let mut node = UnifiedAstNode::new(AstKind::Statement(StmtKind::If), Language::Cpp);
        node.flags.set(NodeFlags::CONTROL_FLOW);
        dag.add_node(node);

        let (cyclomatic, cognitive) = strategy.calculate_complexity(&dag);
        assert_eq!(cyclomatic, 2);
        assert_eq!(cognitive, 1);
    }

    // ==================== Feature-gated tests ====================

    #[cfg(feature = "c-ast")]
    mod c_ast_tests {
        use super::*;

        #[test]
        fn test_c_parse_with_tree_sitter_simple_function() {
            let strategy = CStrategy::new();
            let code = "int main() { return 0; }";
            let result = strategy.parse_with_tree_sitter(code);
            assert!(result.is_ok());
        }

        #[test]
        fn test_c_parse_with_tree_sitter_struct() {
            let strategy = CStrategy::new();
            let code = "struct Point { int x; int y; };";
            let result = strategy.parse_with_tree_sitter(code);
            assert!(result.is_ok());
        }

        #[test]
        fn test_c_parse_with_tree_sitter_enum() {
            let strategy = CStrategy::new();
            let code = "enum Color { RED, GREEN, BLUE };";
            let result = strategy.parse_with_tree_sitter(code);
            assert!(result.is_ok());
        }

        #[test]
        fn test_c_parse_with_tree_sitter_include() {
            let strategy = CStrategy::new();
            let code = "#include <stdio.h>\nint main() { return 0; }";
            let result = strategy.parse_with_tree_sitter(code);
            assert!(result.is_ok());
        }

        #[test]
        fn test_c_convert_to_dag_function() {
            let strategy = CStrategy::new();
            let code = "int add(int a, int b) { return a + b; }";
            let tree = strategy.parse_with_tree_sitter(code).unwrap();
            let dag = strategy.convert_to_dag(&tree, code);

            let has_function = dag
                .nodes
                .iter()
                .any(|node| matches!(node.kind, AstKind::Function(_)));
            assert!(has_function, "Should have a function node");
        }

        #[test]
        fn test_c_convert_to_dag_struct() {
            let strategy = CStrategy::new();
            let code = "struct Person { char* name; int age; };";
            let tree = strategy.parse_with_tree_sitter(code).unwrap();
            let dag = strategy.convert_to_dag(&tree, code);

            let has_struct = dag
                .nodes
                .iter()
                .any(|node| matches!(node.kind, AstKind::Class(ClassKind::Struct)));
            assert!(has_struct, "Should have a struct node");
        }

        #[test]
        fn test_c_convert_to_dag_enum() {
            let strategy = CStrategy::new();
            let code = "enum Status { OK, ERROR, PENDING };";
            let tree = strategy.parse_with_tree_sitter(code).unwrap();
            let dag = strategy.convert_to_dag(&tree, code);

            let has_enum = dag
                .nodes
                .iter()
                .any(|node| matches!(node.kind, AstKind::Class(ClassKind::Enum)));
            assert!(has_enum, "Should have an enum node");
        }

        #[test]
        fn test_c_convert_to_dag_control_flow() {
            let strategy = CStrategy::new();
            let code = r#"
                int foo(int x) {
                    if (x > 0) {
                        return x;
                    }
                    while (x < 10) {
                        x++;
                    }
                    return x;
                }
            "#;
            let tree = strategy.parse_with_tree_sitter(code).unwrap();
            let dag = strategy.convert_to_dag(&tree, code);

            let control_flow_count = dag
                .nodes
                .iter()
                .filter(|node| node.flags.has(NodeFlags::CONTROL_FLOW))
                .count();
            assert!(control_flow_count >= 2, "Should have control flow nodes");
        }

        #[test]
        fn test_c_convert_to_dag_include() {
            let strategy = CStrategy::new();
            let code = "#include <stdio.h>";
            let tree = strategy.parse_with_tree_sitter(code).unwrap();
            let dag = strategy.convert_to_dag(&tree, code);

            let has_import = dag
                .nodes
                .iter()
                .any(|node| matches!(node.kind, AstKind::Import(_)));
            assert!(has_import, "Should have an import node");
        }

        #[tokio::test]
        async fn test_c_parse_file_success() {
            let strategy = CStrategy::new();
            let path = PathBuf::from("test.c");
            let code = "int main() { return 0; }";
            let result = strategy.parse_file(&path, code).await;
            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_c_parse_file_complex() {
            let strategy = CStrategy::new();
            let path = PathBuf::from("test.c");
            let code = r#"
                #include <stdio.h>
                #include <stdlib.h>

                struct Node {
                    int value;
                    struct Node* next;
                };

                enum ErrorCode {
                    SUCCESS = 0,
                    FAILURE = -1
                };

                int calculate(int x, int y) {
                    if (x > y) {
                        return x - y;
                    } else if (x < y) {
                        return y - x;
                    }
                    return 0;
                }

                int main() {
                    for (int i = 0; i < 10; i++) {
                        printf("%d\n", i);
                    }
                    return 0;
                }
            "#;
            let result = strategy.parse_file(&path, code).await;
            assert!(result.is_ok());

            let dag = result.unwrap();
            let functions = strategy.extract_functions(&dag);
            let types = strategy.extract_types(&dag);

            assert!(functions.len() >= 2);
            assert!(types.len() >= 2);
        }
    }

    #[cfg(feature = "cpp-ast")]
    mod cpp_ast_tests {
        use super::*;

        #[test]
        fn test_cpp_parse_with_tree_sitter_simple_function() {
            let strategy = CppStrategy::new();
            let code = "int main() { return 0; }";
            let result = strategy.parse_with_tree_sitter(code);
            assert!(result.is_ok());
        }

        #[test]
        fn test_cpp_parse_with_tree_sitter_class() {
            let strategy = CppStrategy::new();
            let code = "class MyClass { public: void doSomething(); };";
            let result = strategy.parse_with_tree_sitter(code);
            assert!(result.is_ok());
        }

        #[test]
        fn test_cpp_parse_with_tree_sitter_struct() {
            let strategy = CppStrategy::new();
            let code = "struct Point { int x; int y; };";
            let result = strategy.parse_with_tree_sitter(code);
            assert!(result.is_ok());
        }

        #[test]
        fn test_cpp_convert_to_dag_class() {
            let strategy = CppStrategy::new();
            let code = r#"
                class Rectangle {
                public:
                    Rectangle(int w, int h) : width(w), height(h) {}
                    int area() { return width * height; }
                private:
                    int width;
                    int height;
                };
            "#;
            let tree = strategy.parse_with_tree_sitter(code).unwrap();
            let dag = strategy.convert_to_dag(&tree, code);

            let has_class = dag
                .nodes
                .iter()
                .any(|node| matches!(node.kind, AstKind::Class(ClassKind::Regular)));
            assert!(has_class, "Should have a class node");
        }

        #[test]
        fn test_cpp_convert_to_dag_function() {
            let strategy = CppStrategy::new();
            let code = "int add(int a, int b) { return a + b; }";
            let tree = strategy.parse_with_tree_sitter(code).unwrap();
            let dag = strategy.convert_to_dag(&tree, code);

            let has_function = dag
                .nodes
                .iter()
                .any(|node| matches!(node.kind, AstKind::Function(_)));
            assert!(has_function, "Should have a function node");
        }

        #[test]
        fn test_cpp_convert_to_dag_control_flow() {
            let strategy = CppStrategy::new();
            let code = r#"
                int foo(int x) {
                    if (x > 0) {
                        return x;
                    }
                    switch (x) {
                        case 0: return 0;
                        default: return -1;
                    }
                }
            "#;
            let tree = strategy.parse_with_tree_sitter(code).unwrap();
            let dag = strategy.convert_to_dag(&tree, code);

            let control_flow_count = dag
                .nodes
                .iter()
                .filter(|node| node.flags.has(NodeFlags::CONTROL_FLOW))
                .count();
            assert!(control_flow_count >= 2, "Should have control flow nodes");
        }

        #[tokio::test]
        async fn test_cpp_parse_file_success() {
            let strategy = CppStrategy::new();
            let path = PathBuf::from("test.cpp");
            let code = "int main() { return 0; }";
            let result = strategy.parse_file(&path, code).await;
            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_cpp_parse_file_complex() {
            let strategy = CppStrategy::new();
            let path = PathBuf::from("test.cpp");
            let code = r#"
                #include <iostream>
                #include <vector>

                class Stack {
                public:
                    void push(int value) {
                        data.push_back(value);
                    }

                    int pop() {
                        if (data.empty()) {
                            return -1;
                        }
                        int val = data.back();
                        data.pop_back();
                        return val;
                    }

                    bool isEmpty() const {
                        return data.empty();
                    }

                private:
                    std::vector<int> data;
                };

                enum class Status {
                    OK,
                    ERROR,
                    PENDING
                };

                int main() {
                    Stack s;
                    for (int i = 0; i < 10; ++i) {
                        s.push(i);
                    }
                    while (!s.isEmpty()) {
                        std::cout << s.pop() << std::endl;
                    }
                    return 0;
                }
            "#;
            let result = strategy.parse_file(&path, code).await;
            assert!(result.is_ok());

            let dag = result.unwrap();
            let functions = strategy.extract_functions(&dag);
            let types = strategy.extract_types(&dag);

            assert!(functions.len() >= 3);
            assert!(types.len() >= 1);
        }
    }

    // Test visitor directly
    #[cfg(any(feature = "c-ast", feature = "cpp-ast"))]
    mod visitor_tests {
        use super::*;

        #[test]
        fn test_visitor_new() {
            let mut dag = AstDag::new();
            let content = "int main() {}";
            let visitor = CTreeSitterVisitor::new(&mut dag, content, Language::C);
            assert!(visitor.current_parent.is_none());
        }

        #[test]
        fn test_visitor_add_node() {
            let mut dag = AstDag::new();
            let content = "int main() {}";
            let mut visitor = CTreeSitterVisitor::new(&mut dag, content, Language::C);

            let key = visitor.add_node(AstKind::Function(FunctionKind::Regular));
            assert_eq!(key, 0);
            assert_eq!(dag.nodes.len(), 1);

            let node = dag.nodes.get(key).unwrap();
            assert_eq!(node.lang, Language::C);
        }

        #[test]
        fn test_visitor_add_node_with_parent() {
            let mut dag = AstDag::new();
            let content = "struct Foo { void bar(); };";
            let mut visitor = CTreeSitterVisitor::new(&mut dag, content, Language::Cpp);

            // Add parent node
            let parent_key = visitor.add_node(AstKind::Class(ClassKind::Struct));
            visitor.current_parent = Some(parent_key);

            // Add child node
            let child_key = visitor.add_node(AstKind::Function(FunctionKind::Method));

            // Verify parent is set
            let child_node = dag.nodes.get(child_key).unwrap();
            assert_eq!(child_node.parent, parent_key);
        }
    }

    // Tests when c-ast feature is not enabled
    #[cfg(not(feature = "c-ast"))]
    mod non_c_ast_tests {
        use super::*;

        #[tokio::test]
        async fn test_c_parse_file_without_feature() {
            let strategy = CStrategy::new();
            let path = PathBuf::from("test.c");
            let code = "int main() { return 0; }";
            let result = strategy.parse_file(&path, code).await;
            assert!(result.is_err());
            assert!(result.err().unwrap().to_string().contains("c-ast"));
        }
    }

    // Tests when cpp-ast feature is not enabled
    #[cfg(not(feature = "cpp-ast"))]
    mod non_cpp_ast_tests {
        use super::*;

        #[tokio::test]
        async fn test_cpp_parse_file_without_feature() {
            let strategy = CppStrategy::new();
            let path = PathBuf::from("test.cpp");
            let code = "int main() { return 0; }";
            let result = strategy.parse_file(&path, code).await;
            assert!(result.is_err());
            assert!(result.err().unwrap().to_string().contains("cpp-ast"));
        }
    }

    // ==================== Additional Coverage Tests ====================

    #[test]
    fn test_c_extract_imports_multiple() {
        let strategy = CStrategy::new();
        let mut dag = AstDag::new();

        // Add multiple import nodes
        for _ in 0..5 {
            let node = UnifiedAstNode::new(AstKind::Import(ImportKind::Module), Language::C);
            dag.add_node(node);
        }

        let imports = strategy.extract_imports(&dag);
        assert_eq!(imports.len(), 5);
        assert!(imports[0].starts_with("import_"));
    }

    #[test]
    fn test_c_extract_functions_multiple() {
        let strategy = CStrategy::new();
        let mut dag = AstDag::new();

        // Add multiple function nodes
        dag.add_node(UnifiedAstNode::new(
            AstKind::Function(FunctionKind::Regular),
            Language::C,
        ));
        dag.add_node(UnifiedAstNode::new(
            AstKind::Function(FunctionKind::Method),
            Language::C,
        ));
        dag.add_node(UnifiedAstNode::new(
            AstKind::Function(FunctionKind::Lambda),
            Language::C,
        ));

        let functions = strategy.extract_functions(&dag);
        assert_eq!(functions.len(), 3);
    }

    #[test]
    fn test_c_extract_types_multiple() {
        let strategy = CStrategy::new();
        let mut dag = AstDag::new();

        // Add multiple type nodes
        dag.add_node(UnifiedAstNode::new(
            AstKind::Class(ClassKind::Struct),
            Language::C,
        ));
        dag.add_node(UnifiedAstNode::new(
            AstKind::Class(ClassKind::Enum),
            Language::C,
        ));
        dag.add_node(UnifiedAstNode::new(
            AstKind::Class(ClassKind::Regular),
            Language::C,
        ));

        let types = strategy.extract_types(&dag);
        assert_eq!(types.len(), 3);
    }

    #[test]
    fn test_c_calculate_complexity_multiple_control_flow() {
        let strategy = CStrategy::new();
        let mut dag = AstDag::new();

        // Add multiple control flow nodes
        for _ in 0..5 {
            let mut node = UnifiedAstNode::new(AstKind::Statement(StmtKind::If), Language::C);
            node.flags.set(NodeFlags::CONTROL_FLOW);
            dag.add_node(node);
        }

        let (cyclomatic, cognitive) = strategy.calculate_complexity(&dag);
        assert_eq!(cyclomatic, 6); // 1 + 5
        assert_eq!(cognitive, 5);
    }

    #[test]
    fn test_c_extract_imports_mixed_nodes() {
        let strategy = CStrategy::new();
        let mut dag = AstDag::new();

        // Add mixed node types
        dag.add_node(UnifiedAstNode::new(
            AstKind::Import(ImportKind::Module),
            Language::C,
        ));
        dag.add_node(UnifiedAstNode::new(
            AstKind::Function(FunctionKind::Regular),
            Language::C,
        ));
        dag.add_node(UnifiedAstNode::new(
            AstKind::Import(ImportKind::Named),
            Language::C,
        ));
        dag.add_node(UnifiedAstNode::new(
            AstKind::Class(ClassKind::Struct),
            Language::C,
        ));

        let imports = strategy.extract_imports(&dag);
        assert_eq!(imports.len(), 2);
    }

    #[test]
    fn test_c_extract_functions_no_functions() {
        let strategy = CStrategy::new();
        let mut dag = AstDag::new();

        // Add non-function nodes only
        dag.add_node(UnifiedAstNode::new(
            AstKind::Import(ImportKind::Module),
            Language::C,
        ));
        dag.add_node(UnifiedAstNode::new(
            AstKind::Class(ClassKind::Struct),
            Language::C,
        ));
        dag.add_node(UnifiedAstNode::new(
            AstKind::Statement(StmtKind::If),
            Language::C,
        ));

        let functions = strategy.extract_functions(&dag);
        assert!(functions.is_empty());
    }

    #[test]
    fn test_c_extract_types_no_types() {
        let strategy = CStrategy::new();
        let mut dag = AstDag::new();

        // Add non-type nodes only
        dag.add_node(UnifiedAstNode::new(
            AstKind::Import(ImportKind::Module),
            Language::C,
        ));
        dag.add_node(UnifiedAstNode::new(
            AstKind::Function(FunctionKind::Regular),
            Language::C,
        ));

        let types = strategy.extract_types(&dag);
        assert!(types.is_empty());
    }

    #[test]
    fn test_c_calculate_complexity_no_control_flow() {
        let strategy = CStrategy::new();
        let mut dag = AstDag::new();

        // Add nodes without CONTROL_FLOW flag
        dag.add_node(UnifiedAstNode::new(
            AstKind::Function(FunctionKind::Regular),
            Language::C,
        ));
        dag.add_node(UnifiedAstNode::new(
            AstKind::Class(ClassKind::Struct),
            Language::C,
        ));

        let (cyclomatic, cognitive) = strategy.calculate_complexity(&dag);
        assert_eq!(cyclomatic, 1); // Base complexity
        assert_eq!(cognitive, 0);
    }

    #[test]
    fn test_cpp_extract_imports_empty() {
        let strategy = CppStrategy::new();
        let dag = AstDag::new();
        let imports = strategy.extract_imports(&dag);
        assert!(imports.is_empty());
    }

    #[test]
    fn test_cpp_extract_functions_empty() {
        let strategy = CppStrategy::new();
        let dag = AstDag::new();
        let functions = strategy.extract_functions(&dag);
        assert!(functions.is_empty());
    }

    #[test]
    fn test_cpp_extract_types_empty() {
        let strategy = CppStrategy::new();
        let dag = AstDag::new();
        let types = strategy.extract_types(&dag);
        assert!(types.is_empty());
    }

    #[test]
    fn test_cpp_calculate_complexity_empty() {
        let strategy = CppStrategy::new();
        let dag = AstDag::new();
        let (cyclomatic, cognitive) = strategy.calculate_complexity(&dag);
        assert_eq!(cyclomatic, 1);
        assert_eq!(cognitive, 0);
    }

    #[test]
    fn test_c_can_parse_edge_cases() {
        let strategy = CStrategy::new();
        // Test with dots in directory names
        assert!(strategy.can_parse(Path::new("/path.to.something/file.c")));
        assert!(strategy.can_parse(Path::new("./relative/path.h")));
        // Test with multiple extensions (should only look at last)
        assert!(!strategy.can_parse(Path::new("file.c.bak")));
        assert!(!strategy.can_parse(Path::new("file.h.old")));
    }

    #[test]
    fn test_cpp_can_parse_edge_cases() {
        let strategy = CppStrategy::new();
        // Test with dots in directory names
        assert!(strategy.can_parse(Path::new("/path.to.something/file.cpp")));
        assert!(strategy.can_parse(Path::new("./relative/path.hpp")));
        // Test with multiple extensions
        assert!(!strategy.can_parse(Path::new("file.cpp.bak")));
        assert!(!strategy.can_parse(Path::new("file.hpp.old")));
    }

    // ==================== Feature-gated additional tests ====================

    #[cfg(feature = "c-ast")]
    mod c_ast_additional_tests {
        use super::*;

        #[test]
        fn test_c_parse_empty_code() {
            let strategy = CStrategy::new();
            let code = "";
            let result = strategy.parse_with_tree_sitter(code);
            assert!(result.is_ok());
        }

        #[test]
        fn test_c_parse_whitespace_only() {
            let strategy = CStrategy::new();
            let code = "   \n\t  \n  ";
            let result = strategy.parse_with_tree_sitter(code);
            assert!(result.is_ok());
        }

        #[test]
        fn test_c_parse_comment_only() {
            let strategy = CStrategy::new();
            let code = "// This is a comment\n/* Multi-line\ncomment */";
            let result = strategy.parse_with_tree_sitter(code);
            assert!(result.is_ok());
        }

        #[test]
        fn test_c_convert_to_dag_empty() {
            let strategy = CStrategy::new();
            let code = "";
            let tree = strategy.parse_with_tree_sitter(code).unwrap();
            let dag = strategy.convert_to_dag(&tree, code);
            assert!(dag.nodes.is_empty());
        }

        #[test]
        fn test_c_convert_to_dag_multiple_includes() {
            let strategy = CStrategy::new();
            let code = r#"
                #include <stdio.h>
                #include <stdlib.h>
                #include <string.h>
                #include "myheader.h"
            "#;
            let tree = strategy.parse_with_tree_sitter(code).unwrap();
            let dag = strategy.convert_to_dag(&tree, code);

            let import_count = dag
                .nodes
                .iter()
                .filter(|node| matches!(node.kind, AstKind::Import(_)))
                .count();
            assert_eq!(import_count, 4);
        }

        #[test]
        fn test_c_convert_to_dag_for_loop() {
            let strategy = CStrategy::new();
            let code = r#"
                void loop_test() {
                    for (int i = 0; i < 10; i++) {
                        // body
                    }
                }
            "#;
            let tree = strategy.parse_with_tree_sitter(code).unwrap();
            let dag = strategy.convert_to_dag(&tree, code);

            let control_flow_count = dag
                .nodes
                .iter()
                .filter(|node| node.flags.has(NodeFlags::CONTROL_FLOW))
                .count();
            assert!(control_flow_count >= 1, "Should detect for loop as control flow");
        }

        #[test]
        fn test_c_convert_to_dag_switch_statement() {
            let strategy = CStrategy::new();
            let code = r#"
                int switch_test(int x) {
                    switch (x) {
                        case 0: return 0;
                        case 1: return 1;
                        default: return -1;
                    }
                }
            "#;
            let tree = strategy.parse_with_tree_sitter(code).unwrap();
            let dag = strategy.convert_to_dag(&tree, code);

            let control_flow_count = dag
                .nodes
                .iter()
                .filter(|node| node.flags.has(NodeFlags::CONTROL_FLOW))
                .count();
            assert!(
                control_flow_count >= 1,
                "Should detect switch as control flow"
            );
        }

        #[test]
        fn test_c_convert_to_dag_nested_control_flow() {
            let strategy = CStrategy::new();
            let code = r#"
                void nested_test(int x) {
                    if (x > 0) {
                        while (x > 0) {
                            for (int i = 0; i < x; i++) {
                                if (i % 2 == 0) {
                                    // even
                                }
                            }
                            x--;
                        }
                    }
                }
            "#;
            let tree = strategy.parse_with_tree_sitter(code).unwrap();
            let dag = strategy.convert_to_dag(&tree, code);

            let control_flow_count = dag
                .nodes
                .iter()
                .filter(|node| node.flags.has(NodeFlags::CONTROL_FLOW))
                .count();
            assert!(
                control_flow_count >= 4,
                "Should detect nested control flow structures"
            );
        }

        #[test]
        fn test_c_convert_to_dag_multiple_functions() {
            let strategy = CStrategy::new();
            let code = r#"
                int func1() { return 1; }
                int func2() { return 2; }
                int func3() { return 3; }
                int main() { return 0; }
            "#;
            let tree = strategy.parse_with_tree_sitter(code).unwrap();
            let dag = strategy.convert_to_dag(&tree, code);

            let function_count = dag
                .nodes
                .iter()
                .filter(|node| matches!(node.kind, AstKind::Function(_)))
                .count();
            assert!(function_count >= 4, "Should detect all 4 functions");
        }

        #[test]
        fn test_c_convert_to_dag_multiple_structs() {
            let strategy = CStrategy::new();
            let code = r#"
                struct Point { int x; int y; };
                struct Line { struct Point start; struct Point end; };
                struct Rectangle { struct Point topLeft; int width; int height; };
            "#;
            let tree = strategy.parse_with_tree_sitter(code).unwrap();
            let dag = strategy.convert_to_dag(&tree, code);

            let struct_count = dag
                .nodes
                .iter()
                .filter(|node| matches!(node.kind, AstKind::Class(ClassKind::Struct)))
                .count();
            assert!(struct_count >= 3, "Should detect all 3 structs");
        }

        #[test]
        fn test_c_convert_to_dag_mixed_types() {
            let strategy = CStrategy::new();
            let code = r#"
                struct Point { int x; int y; };
                enum Color { RED, GREEN, BLUE };
                typedef struct { float r; float g; float b; } RGB;
            "#;
            let tree = strategy.parse_with_tree_sitter(code).unwrap();
            let dag = strategy.convert_to_dag(&tree, code);

            let struct_count = dag
                .nodes
                .iter()
                .filter(|node| matches!(node.kind, AstKind::Class(ClassKind::Struct)))
                .count();
            let enum_count = dag
                .nodes
                .iter()
                .filter(|node| matches!(node.kind, AstKind::Class(ClassKind::Enum)))
                .count();

            assert!(struct_count >= 1, "Should detect struct");
            assert!(enum_count >= 1, "Should detect enum");
        }

        #[tokio::test]
        async fn test_c_parse_file_empty() {
            let strategy = CStrategy::new();
            let path = PathBuf::from("empty.c");
            let code = "";
            let result = strategy.parse_file(&path, code).await;
            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_c_parse_file_minimal() {
            let strategy = CStrategy::new();
            let path = PathBuf::from("minimal.c");
            let code = "void f(){}";
            let result = strategy.parse_file(&path, code).await;
            assert!(result.is_ok());

            let dag = result.unwrap();
            let functions = strategy.extract_functions(&dag);
            assert!(!functions.is_empty());
        }

        #[test]
        fn test_c_complexity_integration() {
            let strategy = CStrategy::new();
            let code = r#"
                int complex_function(int x, int y) {
                    if (x > 0) {
                        if (y > 0) {
                            return x + y;
                        }
                    }
                    while (x < y) {
                        for (int i = 0; i < 10; i++) {
                            x++;
                        }
                    }
                    switch (x) {
                        case 0: return 0;
                        case 1: return 1;
                        default: return -1;
                    }
                }
            "#;
            let tree = strategy.parse_with_tree_sitter(code).unwrap();
            let dag = strategy.convert_to_dag(&tree, code);

            let (cyclomatic, cognitive) = strategy.calculate_complexity(&dag);
            assert!(cyclomatic > 1, "Complex function should have high cyclomatic complexity");
            assert!(cognitive > 0, "Complex function should have high cognitive complexity");
        }
    }

    #[cfg(feature = "cpp-ast")]
    mod cpp_ast_additional_tests {
        use super::*;

        #[test]
        fn test_cpp_parse_empty_code() {
            let strategy = CppStrategy::new();
            let code = "";
            let result = strategy.parse_with_tree_sitter(code);
            assert!(result.is_ok());
        }

        #[test]
        fn test_cpp_parse_whitespace_only() {
            let strategy = CppStrategy::new();
            let code = "   \n\t  \n  ";
            let result = strategy.parse_with_tree_sitter(code);
            assert!(result.is_ok());
        }

        #[test]
        fn test_cpp_parse_namespace() {
            let strategy = CppStrategy::new();
            let code = r#"
                namespace MyNamespace {
                    class MyClass {
                    public:
                        void doSomething();
                    };
                }
            "#;
            let result = strategy.parse_with_tree_sitter(code);
            assert!(result.is_ok());
        }

        #[test]
        fn test_cpp_convert_to_dag_empty() {
            let strategy = CppStrategy::new();
            let code = "";
            let tree = strategy.parse_with_tree_sitter(code).unwrap();
            let dag = strategy.convert_to_dag(&tree, code);
            assert!(dag.nodes.is_empty());
        }

        #[test]
        fn test_cpp_convert_to_dag_multiple_classes() {
            let strategy = CppStrategy::new();
            let code = r#"
                class Shape { public: virtual void draw() = 0; };
                class Circle : public Shape { public: void draw() override {} };
                class Rectangle : public Shape { public: void draw() override {} };
            "#;
            let tree = strategy.parse_with_tree_sitter(code).unwrap();
            let dag = strategy.convert_to_dag(&tree, code);

            let class_count = dag
                .nodes
                .iter()
                .filter(|node| matches!(node.kind, AstKind::Class(ClassKind::Regular)))
                .count();
            assert!(class_count >= 3, "Should detect all 3 classes");
        }

        #[test]
        fn test_cpp_convert_to_dag_template() {
            let strategy = CppStrategy::new();
            let code = r#"
                template<typename T>
                class Container {
                public:
                    void add(T item);
                    T get(int index);
                };
            "#;
            let tree = strategy.parse_with_tree_sitter(code).unwrap();
            let dag = strategy.convert_to_dag(&tree, code);

            let has_class = dag
                .nodes
                .iter()
                .any(|node| matches!(node.kind, AstKind::Class(_)));
            assert!(has_class, "Should detect template class");
        }

        #[test]
        fn test_cpp_convert_to_dag_enum_class() {
            let strategy = CppStrategy::new();
            let code = r#"
                enum class Color { Red, Green, Blue };
                enum class Direction : int { North = 0, South = 1, East = 2, West = 3 };
            "#;
            let tree = strategy.parse_with_tree_sitter(code).unwrap();
            let dag = strategy.convert_to_dag(&tree, code);

            let enum_count = dag
                .nodes
                .iter()
                .filter(|node| matches!(node.kind, AstKind::Class(ClassKind::Enum)))
                .count();
            assert!(enum_count >= 2, "Should detect enum classes");
        }

        #[test]
        fn test_cpp_convert_to_dag_lambda() {
            let strategy = CppStrategy::new();
            let code = r#"
                void lambda_test() {
                    auto lambda = [](int x) { return x * 2; };
                    auto capture = [&](int y) { return y + 1; };
                }
            "#;
            let tree = strategy.parse_with_tree_sitter(code).unwrap();
            let dag = strategy.convert_to_dag(&tree, code);

            let has_function = dag
                .nodes
                .iter()
                .any(|node| matches!(node.kind, AstKind::Function(_)));
            assert!(has_function, "Should detect function with lambdas");
        }

        #[test]
        fn test_cpp_convert_to_dag_nested_classes() {
            let strategy = CppStrategy::new();
            let code = r#"
                class Outer {
                public:
                    class Inner {
                    public:
                        void innerMethod();
                    };
                    void outerMethod();
                };
            "#;
            let tree = strategy.parse_with_tree_sitter(code).unwrap();
            let dag = strategy.convert_to_dag(&tree, code);

            let class_count = dag
                .nodes
                .iter()
                .filter(|node| matches!(node.kind, AstKind::Class(ClassKind::Regular)))
                .count();
            assert!(class_count >= 2, "Should detect nested classes");
        }

        #[tokio::test]
        async fn test_cpp_parse_file_empty() {
            let strategy = CppStrategy::new();
            let path = PathBuf::from("empty.cpp");
            let code = "";
            let result = strategy.parse_file(&path, code).await;
            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_cpp_parse_file_minimal() {
            let strategy = CppStrategy::new();
            let path = PathBuf::from("minimal.cpp");
            let code = "void f(){}";
            let result = strategy.parse_file(&path, code).await;
            assert!(result.is_ok());

            let dag = result.unwrap();
            let functions = strategy.extract_functions(&dag);
            assert!(!functions.is_empty());
        }

        #[test]
        fn test_cpp_convert_to_dag_with_inheritance() {
            let strategy = CppStrategy::new();
            let code = r#"
                class Base {
                public:
                    virtual void method() {}
                };
                class Derived : public Base {
                public:
                    void method() override {}
                };
            "#;
            let tree = strategy.parse_with_tree_sitter(code).unwrap();
            let dag = strategy.convert_to_dag(&tree, code);

            let class_count = dag
                .nodes
                .iter()
                .filter(|node| matches!(node.kind, AstKind::Class(ClassKind::Regular)))
                .count();
            assert!(class_count >= 2, "Should detect both base and derived classes");
        }

        #[test]
        fn test_cpp_complexity_integration() {
            let strategy = CppStrategy::new();
            let code = r#"
                class Calculator {
                public:
                    int compute(int x, int y) {
                        if (x > 0) {
                            if (y > 0) {
                                return x + y;
                            }
                        }
                        while (x < y) {
                            for (int i = 0; i < 10; i++) {
                                x++;
                            }
                        }
                        return 0;
                    }
                };
            "#;
            let tree = strategy.parse_with_tree_sitter(code).unwrap();
            let dag = strategy.convert_to_dag(&tree, code);

            let (cyclomatic, cognitive) = strategy.calculate_complexity(&dag);
            assert!(
                cyclomatic > 1,
                "Complex method should have high cyclomatic complexity"
            );
            assert!(
                cognitive > 0,
                "Complex method should have high cognitive complexity"
            );
        }

        #[test]
        fn test_cpp_convert_to_dag_multiple_includes() {
            let strategy = CppStrategy::new();
            let code = r#"
                #include <iostream>
                #include <vector>
                #include <string>
                #include <memory>
                #include "myheader.hpp"
            "#;
            let tree = strategy.parse_with_tree_sitter(code).unwrap();
            let dag = strategy.convert_to_dag(&tree, code);

            let import_count = dag
                .nodes
                .iter()
                .filter(|node| matches!(node.kind, AstKind::Import(_)))
                .count();
            assert_eq!(import_count, 5);
        }
    }

    // ==================== Visitor Coverage Tests ====================

    #[cfg(any(feature = "c-ast", feature = "cpp-ast"))]
    mod visitor_coverage_tests {
        use super::*;

        #[test]
        fn test_visitor_with_cpp_language() {
            let mut dag = AstDag::new();
            let content = "class Foo {};";
            let visitor = CTreeSitterVisitor::new(&mut dag, content, Language::Cpp);
            assert_eq!(visitor.language, Language::Cpp);
            assert!(visitor.current_parent.is_none());
        }

        #[test]
        fn test_visitor_multiple_nodes() {
            let mut dag = AstDag::new();
            let content = "struct Foo {}; int bar();";
            let mut visitor = CTreeSitterVisitor::new(&mut dag, content, Language::C);

            // Add multiple nodes
            let _ = visitor.add_node(AstKind::Class(ClassKind::Struct));
            let _ = visitor.add_node(AstKind::Function(FunctionKind::Regular));
            let _ = visitor.add_node(AstKind::Import(ImportKind::Module));

            assert_eq!(dag.nodes.len(), 3);
        }

        #[test]
        fn test_visitor_parent_chain() {
            let mut dag = AstDag::new();
            let content = "struct Outer { struct Inner {}; };";
            let mut visitor = CTreeSitterVisitor::new(&mut dag, content, Language::C);

            // Create parent-child relationship
            let outer_key = visitor.add_node(AstKind::Class(ClassKind::Struct));
            visitor.current_parent = Some(outer_key);
            let inner_key = visitor.add_node(AstKind::Class(ClassKind::Struct));

            assert_eq!(dag.nodes.get(inner_key).unwrap().parent, outer_key);
        }

        #[test]
        fn test_visitor_no_parent() {
            let mut dag = AstDag::new();
            let content = "int main() {}";
            let mut visitor = CTreeSitterVisitor::new(&mut dag, content, Language::C);

            let key = visitor.add_node(AstKind::Function(FunctionKind::Regular));
            // Parent should be 0 (default) when no parent is set
            assert_eq!(dag.nodes.get(key).unwrap().parent, 0);
        }
    }

    // ==================== Edge Case Tests ====================

    #[test]
    fn test_extract_imports_large_dag() {
        let strategy = CStrategy::new();
        let mut dag = AstDag::new();

        // Add many nodes of various types
        for i in 0..100 {
            match i % 4 {
                0 => {
                    dag.add_node(UnifiedAstNode::new(
                        AstKind::Import(ImportKind::Module),
                        Language::C,
                    ));
                }
                1 => {
                    dag.add_node(UnifiedAstNode::new(
                        AstKind::Function(FunctionKind::Regular),
                        Language::C,
                    ));
                }
                2 => {
                    dag.add_node(UnifiedAstNode::new(
                        AstKind::Class(ClassKind::Struct),
                        Language::C,
                    ));
                }
                _ => {
                    let mut node =
                        UnifiedAstNode::new(AstKind::Statement(StmtKind::If), Language::C);
                    node.flags.set(NodeFlags::CONTROL_FLOW);
                    dag.add_node(node);
                }
            }
        }

        let imports = strategy.extract_imports(&dag);
        assert_eq!(imports.len(), 25);

        let functions = strategy.extract_functions(&dag);
        assert_eq!(functions.len(), 25);

        let types = strategy.extract_types(&dag);
        assert_eq!(types.len(), 25);

        let (cyclomatic, cognitive) = strategy.calculate_complexity(&dag);
        assert_eq!(cyclomatic, 26); // 1 + 25
        assert_eq!(cognitive, 25);
    }

    #[test]
    fn test_c_language_identity() {
        let strategy = CStrategy::new();
        assert_eq!(strategy.language(), Language::C);

        let strategy2 = CStrategy::default();
        assert_eq!(strategy2.language(), Language::C);
    }

    #[test]
    fn test_cpp_language_identity() {
        let strategy = CppStrategy::new();
        assert_eq!(strategy.language(), Language::Cpp);

        let strategy2 = CppStrategy::default();
        assert_eq!(strategy2.language(), Language::Cpp);
    }
}
