// Tree-sitter-based Python AST visitor (modern approach)

#[cfg(feature = "python-ast")]
struct PythonTreeSitterVisitor<'a> {
    dag: &'a mut AstDag,
    
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
