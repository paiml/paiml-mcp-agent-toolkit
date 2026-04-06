// Tree-sitter parsing and DAG conversion methods for PythonStrategy

impl PythonStrategy {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
}
