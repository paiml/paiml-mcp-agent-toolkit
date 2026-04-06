#[async_trait]
impl LanguageStrategy for RustStrategy {
    fn language(&self) -> Language {
        Language::Rust
    }

    fn can_parse(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext == "rs")
    }

    async fn parse_file(&self, _path: &Path, content: &str) -> Result<AstDag> {
        let syn_file = self.parse_syn_file(content)?;
        Ok(self.convert_to_dag(&syn_file))
    }

    fn extract_imports(&self, ast: &AstDag) -> Vec<String> {
        // Iterate through nodes looking for imports
        let mut imports = Vec::new();
        for i in 0..ast.nodes.len() {
            if let Some(node) = ast.nodes.get(i as u32) {
                if node.flags.has(NodeFlags::IMPORT) {
                    // Extract import name from metadata if available
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
                if matches!(node.kind, AstKind::Class(_) | AstKind::Type(_)) {
                    types.push(node.clone());
                }
            }
        }
        types
    }

    fn calculate_complexity(&self, ast: &AstDag) -> (u32, u32) {
        let mut cyclomatic = 1;
        let mut cognitive = 0;

        // Count control flow nodes
        for i in 0..ast.nodes.len() {
            if let Some(node) = ast.nodes.get(i as u32) {
                if node.flags.has(NodeFlags::CONTROL_FLOW) {
                    cyclomatic += 1;
                    // Cognitive complexity increases with nesting depth
                    cognitive += 1;
                }
            }
        }

        (cyclomatic, cognitive)
    }
}
