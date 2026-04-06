// Import parsing and dependency resolution methods for DependencyGraphBuilder
// Extracted from builder.rs

impl DependencyGraphBuilder {
    /// Resolve dependencies for a file
    /// Complexity: 9 (import parsing + edge creation)
    fn resolve_file_dependencies(&mut self, node_id: NodeId, path: &Path) -> Result<()> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let content = fs::read_to_string(path)?;

        let imports = match path.extension().and_then(|s| s.to_str()) {
            Some("rs") => self.parse_rust_imports(&content)?,
            Some("py") => self.parse_python_imports(&content)?,
            Some("ts") | Some("tsx") | Some("js") | Some("jsx") => {
                self.parse_typescript_imports(&content)?
            }
            _ => vec![],
        };

        // Create edges for each import
        for import in imports {
            if let Some(target_node) = self.resolve_import_to_node(&import) {
                let edge = EdgeData::Import {
                    weight: 1.0,
                    visibility: Visibility::Public,
                };
                self.graph.add_edge(node_id, target_node, edge);
            }
        }

        Ok(())
    }

    /// Parse Rust imports
    /// Complexity: 6
    fn parse_rust_imports(&self, content: &str) -> Result<Vec<String>> {
        let mut imports = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("use ") && trimmed.ends_with(';') {
                let import = trimmed
                    .strip_prefix("use ")
                    .and_then(|s| s.strip_suffix(';'))
                    .unwrap_or("")
                    .trim();
                imports.push(import.to_string());
            }
        }

        Ok(imports)
    }

    /// Parse Python imports
    /// Complexity: 5
    fn parse_python_imports(&self, content: &str) -> Result<Vec<String>> {
        let mut imports = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("import ") || trimmed.starts_with("from ") {
                imports.push(trimmed.to_string());
            }
        }

        Ok(imports)
    }

    /// Parse TypeScript/JavaScript imports
    /// Complexity: 6
    fn parse_typescript_imports(&self, content: &str) -> Result<Vec<String>> {
        let mut imports = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("import ") {
                if let Some(end) = trimmed.rfind(" from ") {
                    let module = trimmed
                        .get(end + 6..)
                        .unwrap_or_default()
                        .trim_matches(|c| c == '\'' || c == '"' || c == ';');
                    imports.push(module.to_string());
                }
            } else if trimmed.starts_with("const ") && trimmed.contains(" = require(") {
                if let Some(start) = trimmed.find("require('") {
                    if let Some(end) = trimmed.get(start + 9..).unwrap_or_default().find('\'') {
                        let module = trimmed.get(start + 9..start + 9 + end).unwrap_or_default();
                        imports.push(module.to_string());
                    }
                }
            }
        }

        Ok(imports)
    }
}
