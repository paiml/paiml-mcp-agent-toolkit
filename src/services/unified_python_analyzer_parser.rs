impl UnifiedPythonAnalyzer {
    /// Parse Python with tree-sitter-python
    #[cfg(feature = "python-ast")]
    fn parse_python(&self, content: &str) -> Result<Tree, AnalysisError> {
        let mut parser = TsParser::new();
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .map_err(|e| AnalysisError::Parse(format!("Failed to set Python language: {}", e)))?;

        let tree = parser
            .parse(content, None)
            .ok_or_else(|| AnalysisError::Parse("Failed to parse Python code".to_string()))?;

        // Check for syntax errors
        if Self::has_syntax_errors(&tree) {
            return Err(AnalysisError::Parse(
                "Python syntax error detected in source".to_string(),
            ));
        }

        Ok(tree)
    }

    /// Check if tree-sitter parse tree has syntax errors
    #[cfg(feature = "python-ast")]
    fn has_syntax_errors(tree: &Tree) -> bool {
        let root = tree.root_node();
        Self::node_has_error(&root)
    }

    /// Recursively check node for errors
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
}
