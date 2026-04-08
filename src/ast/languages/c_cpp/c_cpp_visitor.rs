#![allow(unused)]
#![cfg_attr(coverage_nightly, coverage(off))]
//! Tree-sitter visitor for C/C++ AST conversion

use crate::ast::core::{
    AstDag, AstKind, ClassKind, FunctionKind, ImportKind, Language, NodeFlags, StmtKind,
    UnifiedAstNode,
};

/// Tree-sitter visitor for C/C++ AST conversion
pub struct CTreeSitterVisitor<'a> {
    dag: &'a mut AstDag,

    content: &'a str,
    pub(crate) language: Language,
    pub(crate) current_parent: Option<u32>,
}

impl<'a> CTreeSitterVisitor<'a> {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new(dag: &'a mut AstDag, content: &'a str, language: Language) -> Self {
        Self {
            dag,
            content,
            language,
            current_parent: None,
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Add node.
    pub fn add_node(&mut self, kind: AstKind) -> u32 {
        let mut node = UnifiedAstNode::new(kind, self.language);

        if let Some(parent) = self.current_parent {
            node.parent = parent;
        }

        self.dag.add_node(node)
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Visit node.
    pub fn visit_node(&mut self, node: &tree_sitter::Node, parent: Option<u32>) {
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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod c_cpp_visitor_tests {
    use super::*;

    fn create_test_dag() -> AstDag {
        AstDag::default()
    }

    #[test]
    fn test_new_visitor() {
        let mut dag = create_test_dag();
        let content = "int main() {}";
        let visitor = CTreeSitterVisitor::new(&mut dag, content, Language::C);

        assert_eq!(visitor.language, Language::C);
        assert!(visitor.current_parent.is_none());
    }

    #[test]
    fn test_new_visitor_cpp() {
        let mut dag = create_test_dag();
        let content = "class Foo {};";
        let visitor = CTreeSitterVisitor::new(&mut dag, content, Language::Cpp);

        assert_eq!(visitor.language, Language::Cpp);
    }

    #[test]
    fn test_add_node_function() {
        let mut dag = create_test_dag();
        let content = "";
        let mut visitor = CTreeSitterVisitor::new(&mut dag, content, Language::C);

        let key = visitor.add_node(AstKind::Function(FunctionKind::Regular));

        assert_eq!(key, 0); // First node gets key 0
        assert_eq!(dag.nodes.len(), 1);
    }

    #[test]
    fn test_add_node_with_parent() {
        let mut dag = create_test_dag();
        let content = "";
        let mut visitor = CTreeSitterVisitor::new(&mut dag, content, Language::C);

        // Add parent node
        let parent_key = visitor.add_node(AstKind::Class(ClassKind::Struct));
        visitor.current_parent = Some(parent_key);

        // Add child node
        let child_key = visitor.add_node(AstKind::Function(FunctionKind::Regular));

        // Child's parent should be set
        assert_eq!(dag.nodes.len(), 2);
        let child = dag.nodes.get(child_key);
        assert!(child.is_some());
        assert_eq!(child.unwrap().parent, parent_key);
    }

    #[test]
    fn test_add_node_struct() {
        let mut dag = create_test_dag();
        let content = "";
        let mut visitor = CTreeSitterVisitor::new(&mut dag, content, Language::C);

        let key = visitor.add_node(AstKind::Class(ClassKind::Struct));

        let node = dag.nodes.get(key);
        assert!(node.is_some());
        matches!(node.unwrap().kind, AstKind::Class(ClassKind::Struct));
    }

    #[test]
    fn test_add_node_enum() {
        let mut dag = create_test_dag();
        let content = "";
        let mut visitor = CTreeSitterVisitor::new(&mut dag, content, Language::C);

        let key = visitor.add_node(AstKind::Class(ClassKind::Enum));

        let node = dag.nodes.get(key);
        assert!(node.is_some());
        matches!(node.unwrap().kind, AstKind::Class(ClassKind::Enum));
    }

    #[test]
    fn test_add_multiple_nodes() {
        let mut dag = create_test_dag();
        let content = "";
        let mut visitor = CTreeSitterVisitor::new(&mut dag, content, Language::Cpp);

        visitor.add_node(AstKind::Function(FunctionKind::Regular));
        visitor.add_node(AstKind::Class(ClassKind::Regular));
        visitor.add_node(AstKind::Class(ClassKind::Struct));

        assert_eq!(dag.nodes.len(), 3);
    }

    #[test]
    fn test_visitor_language_preservation() {
        let mut dag = create_test_dag();
        let content = "";
        let mut visitor = CTreeSitterVisitor::new(&mut dag, content, Language::Cpp);

        let key = visitor.add_node(AstKind::Function(FunctionKind::Regular));

        let node = dag.nodes.get(key);
        assert!(node.is_some());
        assert_eq!(node.unwrap().lang, Language::Cpp);
    }

    #[test]
    fn test_add_import_node() {
        let mut dag = create_test_dag();
        let content = "";
        let mut visitor = CTreeSitterVisitor::new(&mut dag, content, Language::C);

        let key = visitor.add_node(AstKind::Import(ImportKind::Module));

        let node = dag.nodes.get(key);
        assert!(node.is_some());
        matches!(node.unwrap().kind, AstKind::Import(ImportKind::Module));
    }

    #[test]
    fn test_add_statement_node() {
        let mut dag = create_test_dag();
        let content = "";
        let mut visitor = CTreeSitterVisitor::new(&mut dag, content, Language::C);

        let key = visitor.add_node(AstKind::Statement(StmtKind::If));

        let node = dag.nodes.get(key);
        assert!(node.is_some());
        matches!(node.unwrap().kind, AstKind::Statement(StmtKind::If));
    }
}
