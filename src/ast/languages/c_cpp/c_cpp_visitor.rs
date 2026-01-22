//! Tree-sitter visitor for C/C++ AST conversion

use crate::ast::core::{
    AstDag, AstKind, ClassKind, FunctionKind, ImportKind, Language, NodeFlags, StmtKind,
    UnifiedAstNode,
};

/// Tree-sitter visitor for C/C++ AST conversion
#[allow(dead_code)]
pub struct CTreeSitterVisitor<'a> {
    dag: &'a mut AstDag,
    #[allow(dead_code)]
    content: &'a str,
    pub(crate) language: Language,
    pub(crate) current_parent: Option<u32>,
}

#[allow(dead_code)]
impl<'a> CTreeSitterVisitor<'a> {
    pub fn new(dag: &'a mut AstDag, content: &'a str, language: Language) -> Self {
        Self {
            dag,
            content,
            language,
            current_parent: None,
        }
    }

    pub fn add_node(&mut self, kind: AstKind) -> u32 {
        let mut node = UnifiedAstNode::new(kind, self.language);

        if let Some(parent) = self.current_parent {
            node.parent = parent;
        }

        self.dag.add_node(node)
    }

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
