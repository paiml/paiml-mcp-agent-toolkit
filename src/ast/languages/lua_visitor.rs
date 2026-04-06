/// Tree-sitter-based Lua AST visitor
#[cfg(feature = "lua-ast")]
struct LuaTreeSitterVisitor<'a> {
    dag: &'a mut AstDag,
    content: &'a str,
    current_parent: Option<u32>,
}

#[cfg(feature = "lua-ast")]
impl<'a> LuaTreeSitterVisitor<'a> {
    fn new(dag: &'a mut AstDag, content: &'a str) -> Self {
        Self {
            dag,
            content,
            current_parent: None,
        }
    }

    fn add_node(&mut self, kind: AstKind) -> u32 {
        let mut node = UnifiedAstNode::new(kind, Language::Lua);

        if let Some(parent) = self.current_parent {
            node.parent = parent;
        }

        self.dag.add_node(node)
    }

    /// Check if a function_call node is a `require(...)` call
    fn is_require_call(&self, node: &tree_sitter::Node) -> bool {
        // In tree-sitter-lua 0.2.0, function_call has a child "name" or the
        // first named child is an identifier
        for child in node.children(&mut node.walk()) {
            if child.kind() == "identifier" {
                let text = child.utf8_text(self.content.as_bytes()).unwrap_or_default();
                if text == "require" {
                    return true;
                }
            }
            // Also check dot_index_expression for nested calls like foo.require
            // but the simple case is just `require("module")`
        }
        false
    }

    fn visit_node(&mut self, node: &tree_sitter::Node, parent: Option<u32>) {
        let old_parent = self.current_parent;
        self.current_parent = parent;

        match node.kind() {
            "function_definition" | "function_declaration" => {
                let key = self.add_node(AstKind::Function(FunctionKind::Regular));

                self.current_parent = Some(key);
                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child, Some(key));
                }
            }
            "variable_declaration" => {
                let _key = self.add_node(AstKind::Variable(VarKind::Let));

                // Visit children to capture nested function definitions
                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child, parent);
                }
            }
            "if_statement" => {
                let mut n = UnifiedAstNode::new(AstKind::Statement(StmtKind::If), Language::Lua);
                n.flags.set(NodeFlags::CONTROL_FLOW);
                self.dag.add_node(n);

                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child, parent);
                }
            }
            "for_statement" => {
                let mut n = UnifiedAstNode::new(AstKind::Statement(StmtKind::For), Language::Lua);
                n.flags.set(NodeFlags::CONTROL_FLOW);
                self.dag.add_node(n);

                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child, parent);
                }
            }
            "while_statement" => {
                let mut n = UnifiedAstNode::new(AstKind::Statement(StmtKind::While), Language::Lua);
                n.flags.set(NodeFlags::CONTROL_FLOW);
                self.dag.add_node(n);

                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child, parent);
                }
            }
            "repeat_statement" => {
                let mut n =
                    UnifiedAstNode::new(AstKind::Statement(StmtKind::DoWhile), Language::Lua);
                n.flags.set(NodeFlags::CONTROL_FLOW);
                self.dag.add_node(n);

                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child, parent);
                }
            }
            "do_statement" => {
                let _key = self.add_node(AstKind::Statement(StmtKind::Block));

                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child, parent);
                }
            }
            "return_statement" => {
                let _key = self.add_node(AstKind::Statement(StmtKind::Return));

                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child, parent);
                }
            }
            "table_constructor" => {
                let _key = self.add_node(AstKind::Class(ClassKind::Regular));

                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child, parent);
                }
            }
            "function_call" => {
                if self.is_require_call(node) {
                    let mut n =
                        UnifiedAstNode::new(AstKind::Import(ImportKind::Module), Language::Lua);
                    n.flags.set(NodeFlags::IMPORT);
                    self.dag.add_node(n);
                }

                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child, parent);
                }
            }
            "elseif_statement" => {
                let mut n = UnifiedAstNode::new(AstKind::Statement(StmtKind::If), Language::Lua);
                n.flags.set(NodeFlags::CONTROL_FLOW);
                self.dag.add_node(n);

                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child, parent);
                }
            }
            "binary_expression" => {
                // Check for `and` / `or` operators which add to cyclomatic complexity
                for child in node.children(&mut node.walk()) {
                    if child.kind() == "and" || child.kind() == "or" {
                        let mut n =
                            UnifiedAstNode::new(AstKind::Statement(StmtKind::Block), Language::Lua);
                        n.flags.set(NodeFlags::CONTROL_FLOW);
                        self.dag.add_node(n);
                    }
                    self.visit_node(&child, parent);
                }
            }
            _ => {
                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child, parent);
                }
            }
        }

        self.current_parent = old_parent;
    }
}
