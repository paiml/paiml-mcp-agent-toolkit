// File analysis, extraction helpers, and utility methods for DependencyGraphBuilder
// Extracted from builder.rs

impl DependencyGraphBuilder {
    /// Analyze single file and create node
    /// Complexity: 10 (parsing + node creation)
    fn analyze_file(&mut self, path: &Path) -> Result<NodeId> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let content = fs::read_to_string(path)?;

        // Calculate hash for incremental updates
        let hash = self.calculate_hash(&content);

        // Check if already processed with same hash
        if let Some(&existing_hash) = self.processed_hashes.get(path) {
            if existing_hash == hash {
                // Skip if unchanged - path must exist in node_map (inserted at line 184)
                return Ok(*self.node_map.get(path).expect(
                    "path exists in node_map (inserted at line 184 when added to processed_hashes)",
                ));
            }
        }

        // Create node data
        let node_data = NodeData {
            path: path.to_path_buf(),
            module: self.path_to_module(path),
            symbols: self
                .symbol_table
                .get_file_symbols(path)
                .iter()
                .map(|e| e.symbol.clone())
                .collect(),
            loc: content.lines().count(),
            complexity: self.estimate_complexity(&content),
            ast_hash: hash,
        };

        // Add or update node
        let node_id = if let Some(&existing_id) = self.node_map.get(path) {
            // existing_id came from node_map, so node must exist in graph (added at line 186)
            *self
                .graph
                .node_weight_mut(existing_id)
                .expect("node exists in graph (node_id came from node_map at line 184)") =
                node_data;
            existing_id
        } else {
            let id = self.graph.add_node(node_data);
            self.node_map.insert(path.to_path_buf(), id);
            id
        };

        self.processed_hashes.insert(path.to_path_buf(), hash);
        Ok(node_id)
    }

    /// Helper functions for parsing
    /// Complexity: 2 each
    fn extract_function_name(line: &str) -> Option<&str> {
        debug_assert!(!line.is_empty(), "line must not be empty");
        line.split_whitespace()
            .find(|&w| w != "pub" && w != "fn")
            .and_then(|s| s.split('(').next())
    }

    fn extract_type_name<'a>(line: &'a str, keyword: &str) -> Option<&'a str> {
        debug_assert!(!keyword.is_empty(), "keyword must not be empty");
        line.split_whitespace()
            .find(|&w| w != "pub" && w != keyword)
            .and_then(|s| s.split('{').next())
            .and_then(|s| s.split('<').next())
    }

    fn extract_python_function_name(line: &str) -> Option<&str> {
        debug_assert!(!line.is_empty(), "line must not be empty");
        line.strip_prefix("def ")
            .and_then(|s| s.split('(').next())
            .map(|s| s.trim())
    }

    fn extract_python_class_name(line: &str) -> Option<&str> {
        debug_assert!(!line.is_empty(), "line must not be empty");
        line.strip_prefix("class ")
            .and_then(|s| s.split('(').next())
            .and_then(|s| s.split(':').next())
            .map(|s| s.trim())
    }

    fn extract_ts_name(line: &str) -> Option<&str> {
        debug_assert!(!line.is_empty(), "line must not be empty");
        line.split_whitespace()
            .find(|&w| w != "export" && w != "const" && w != "function")
            .and_then(|s| s.split('(').next())
            .and_then(|s| s.split('=').next())
            .map(|s| s.trim())
    }

    fn extract_ts_class_name(line: &str) -> Option<&str> {
        line.split_whitespace()
            .find(|&w| w != "export" && w != "class")
            .and_then(|s| s.split('{').next())
            .map(|s| s.trim())
    }

    /// Convert path to module name
    /// Complexity: 4
    fn path_to_module(&self, path: &Path) -> String {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string()
    }

    /// Calculate content hash
    /// Complexity: 2
    fn calculate_hash(&self, content: &str) -> u64 {
        debug_assert!(!content.is_empty(), "content must not be empty");
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        hasher.finish()
    }

    /// Estimate complexity from content
    /// Complexity: 5
    fn estimate_complexity(&self, content: &str) -> f64 {
        debug_assert!(!content.is_empty(), "content must not be empty");
        let mut complexity = 1.0;

        for line in content.lines() {
            let trimmed = line.trim();
            // Count control flow keywords
            if trimmed.starts_with("if ") || trimmed.starts_with("else") {
                complexity += 1.0;
            } else if trimmed.starts_with("for ") || trimmed.starts_with("while ") {
                complexity += 2.0;
            } else if trimmed.starts_with("match ") || trimmed.starts_with("switch ") {
                complexity += 1.5;
            }
        }

        complexity
    }

    /// Resolve import string to node
    /// Complexity: 4
    fn resolve_import_to_node(&self, import: &str) -> Option<NodeId> {
        debug_assert!(!import.is_empty(), "import must not be empty");
        // Try to find matching module in node_map
        for (path, &node_id) in &self.node_map {
            let module_name = self.path_to_module(path);
            if import.contains(&module_name) {
                return Some(node_id);
            }
        }
        None
    }
}
