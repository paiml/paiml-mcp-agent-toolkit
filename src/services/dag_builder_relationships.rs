// DagBuilder relationship processing: edges, imports, inheritance, and helper methods

impl DagBuilder {
    fn process_relationships(&mut self, file: &FileContext) {
        // Create module node for the file itself
        let file_module_id = self.normalize_path(&file.path);
        let node = NodeInfo {
            id: file_module_id.clone(),
            label: self.extract_module_name(&file.path),
            node_type: NodeType::Module,
            file_path: file.path.clone(),
            line_number: 0,
            complexity: file
                .complexity_metrics
                .as_ref()
                .map_or(1, |m| u32::from(m.total_complexity.cognitive)),
            metadata: FxHashMap::default(),
        };
        self.add_node(self.enrich_node(node));

        for item in &file.items {
            self.process_single_relationship(item, &file_module_id);
        }
    }

    fn process_single_relationship(&mut self, item: &AstItem, file_module_id: &str) {
        debug_assert!(!file_module_id.is_empty(), "file_module_id must not be empty");
        match item {
            AstItem::Use { path, line: _ } => {
                self.process_use_import(path, file_module_id);
            }
            AstItem::Import {
                module,
                items,
                alias: _,
                line: _,
            } => {
                self.process_language_import(module, items, file_module_id);
            }
            AstItem::Impl {
                type_name,
                trait_name,
                ..
            } => {
                self.process_impl_relationship(type_name, trait_name);
            }
            _ => {}
        }
    }

    fn process_use_import(&mut self, path: &str, file_module_id: &str) {
        debug_assert!(!path.is_empty(), "path must not be empty");
        debug_assert!(!file_module_id.is_empty(), "file_module_id must not be empty");
        // Create import edges from the file module to imported items
        if let Some(target_id) = self.resolve_import_path(path) {
            self.add_edge(Edge {
                from: file_module_id.to_string(),
                to: target_id,
                edge_type: EdgeType::Imports,
                weight: 1,
            });
        }
    }

    fn process_language_import(
        &mut self,
        module: &str,
        items: &[String],
        file_module_id: &str,
    ) {
        debug_assert!(!module.is_empty(), "module must not be empty");
        debug_assert!(!file_module_id.is_empty(), "file_module_id must not be empty");
        debug_assert!(!items.is_empty(), "items must not be empty");
        // Handle language-specific imports (Python, JavaScript, etc.)
        // Create import edge to the module
        if let Some(target_id) = self.resolve_import_path(module) {
            self.add_edge(Edge {
                from: file_module_id.to_string(),
                to: target_id.clone(),
                edge_type: EdgeType::Imports,
                weight: 1,
            });
        }

        // Also create edges for specific imported items
        for item in items {
            let full_path = format!("{module}.{item}");
            if let Some(target_id) = self.resolve_import_path(&full_path) {
                self.add_edge(Edge {
                    from: file_module_id.to_string(),
                    to: target_id,
                    edge_type: EdgeType::Imports,
                    weight: 1,
                });
            }
        }
    }

    fn process_impl_relationship(
        &mut self,
        type_name: &str,
        trait_name: &Option<String>,
    ) {
        debug_assert!(!type_name.is_empty(), "type_name must not be empty");
        // Create inheritance edges for trait implementations
        if let (Some(trait_name), Some(struct_id)) =
            (trait_name.as_ref(), self.type_map.get(type_name))
        {
            if let Some(trait_id) = self.type_map.get(trait_name) {
                self.add_edge(Edge {
                    from: struct_id.clone(),
                    to: trait_id.clone(),
                    edge_type: EdgeType::Inherits,
                    weight: 1,
                });
            }
        }
    }

    fn add_node(&mut self, node: NodeInfo) {
        self.graph.add_node(node);
    }

    fn add_edge(&mut self, edge: Edge) {
        self.graph.add_edge(edge);
    }

    /// Enrich node with semantic naming and metadata
    fn enrich_node(&self, mut node: NodeInfo) -> NodeInfo {
        // Apply semantic naming
        let semantic_name = self.namer.get_semantic_name(&node.id, &node);
        if semantic_name != node.id && !semantic_name.is_empty() {
            node.label = semantic_name;
        }

        // Add comprehensive metadata as specified in the bug report
        node.metadata
            .insert("file_path".to_string(), node.file_path.clone());
        node.metadata.insert(
            "module_path".to_string(),
            self.path_to_module(&node.file_path),
        );
        node.metadata
            .insert("display_name".to_string(), node.label.clone());
        node.metadata
            .insert("node_type".to_string(), format!("{:?}", node.node_type));
        node.metadata
            .insert("line_number".to_string(), node.line_number.to_string());
        node.metadata
            .insert("complexity".to_string(), node.complexity.to_string());

        // Add language-specific metadata
        let language = detect_language_from_path(&node.file_path);
        node.metadata
            .insert("language".to_string(), language.to_string());

        node
    }

    fn normalize_path(&self, path: &str) -> String {
        debug_assert!(!path.is_empty(), "path must not be empty");
        // Convert file path to a module-like identifier
        path.trim_start_matches("./")
            .trim_start_matches('/')
            .trim_end_matches(".rs")
            .trim_end_matches(".ts")
            .trim_end_matches(".py")
            .trim_end_matches(".js")
            .trim_end_matches(".tsx")
            .trim_end_matches(".jsx")
            .replace(['/', '.', '-'], "_")
    }

    fn path_to_module(&self, path: &str) -> String {
        debug_assert!(!path.is_empty(), "path must not be empty");
        // Convert file path to module notation using semantic namer
        let ext = std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let language = SemanticNamer::detect_language(ext);

        // Use the semantic namer's path_to_module logic indirectly
        let clean_path = path
            .trim_start_matches("./")
            .trim_start_matches('/')
            .trim_start_matches("src/")
            .trim_start_matches("lib/")
            .trim_start_matches("app/");

        let without_ext = std::path::Path::new(clean_path)
            .with_extension("")
            .to_string_lossy()
            .into_owned();

        let separator = match language {
            "rust" => "::",
            "python" => ".",
            "typescript" | "javascript" => ".",
            "go" => "/",
            "java" => ".",
            _ => "::",
        };

        without_ext.replace(['/', '\\'], separator)
    }

    fn extract_module_name(&self, path: &str) -> String {
        debug_assert!(!path.is_empty(), "path must not be empty");
        // Extract just the file name without extension
        std::path::Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(path)
            .to_string()
    }

    fn resolve_import_path(&self, import_path: &str) -> Option<String> {
        debug_assert!(!import_path.is_empty(), "import_path must not be empty");
        // Try to resolve the import to a known node
        // First check if it's a direct type reference
        if let Some(type_id) = self.type_map.get(import_path) {
            return Some(type_id.clone());
        }

        // Check if it's a function reference
        if let Some(func_id) = self.function_map.get(import_path) {
            return Some(func_id.clone());
        }

        // For module paths like "crate::models::dag", try to find a matching module
        let parts: Vec<&str> = import_path.split("::").collect();
        if let Some(last_part) = parts.last() {
            // Try as type
            if let Some(type_id) = self.type_map.get(*last_part) {
                return Some(type_id.clone());
            }
            // Try as function
            if let Some(func_id) = self.function_map.get(*last_part) {
                return Some(func_id.clone());
            }
        }

        // If nothing found, create a module node for the import
        Some(import_path.replace("::", "_"))
    }
}

/// Detect programming language from file path extension
fn detect_language_from_path(file_path: &str) -> &'static str {
    debug_assert!(!file_path.is_empty(), "file_path must not be empty");
    let ext = std::path::Path::new(file_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    match ext {
        "rs" => "rust",
        "ts" | "tsx" => "typescript",
        "js" | "jsx" => "javascript",
        "py" => "python",
        "go" => "go",
        "java" => "java",
        _ => "unknown",
    }
}
