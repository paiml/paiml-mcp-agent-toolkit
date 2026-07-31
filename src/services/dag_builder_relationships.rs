// DagBuilder relationship processing: edges, imports, inheritance, and helper methods

impl DagBuilder {
    fn process_relationships(&mut self, file: &FileContext) {
        // The module node for the file itself is created during node collection
        // (pass 1) so that imports can resolve against it here in pass 2.
        let file_module_id = self.normalize_path(&file.path);

        for item in &file.items {
            self.process_single_relationship(item, &file_module_id);
        }
    }

    fn process_single_relationship(&mut self, item: &AstItem, file_module_id: &str) {
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
        // Extract just the file name without extension
        std::path::Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(path)
            .to_string()
    }

    /// Resolve a `use`/import path to a node that actually exists in the graph.
    ///
    /// Defect #653: the old version ended with
    /// `Some(import_path.replace("::", "_"))` — an id for a node that was never
    /// created — so `finalize_graph` dropped every one of those edges and
    /// `analyze dag` reported "0 edges" on every project. Unresolvable (i.e.
    /// external) imports now return None instead of an invented target.
    fn resolve_import_path(&self, import_path: &str) -> Option<String> {
        // Direct hit on a declared symbol (rare, but cheap to check).
        if let Some(type_id) = self.type_map.get(import_path) {
            return Some(type_id.clone());
        }
        if let Some(func_id) = self.function_map.get(import_path) {
            return Some(func_id.clone());
        }

        let (segments, intra_crate) = normalize_import_segments(import_path);
        if segments.is_empty() {
            return None;
        }

        // Longest-prefix match against the module paths of analyzed files:
        // "crate::services::dag_builder::DagBuilder" -> module "services::dag_builder".
        if let Some(module_id) = self.resolve_module_prefix(&segments) {
            return Some(module_id);
        }

        // Intra-crate paths may name an item defined in a file we did analyze even
        // when the module path itself does not match; external crates never do.
        if intra_crate {
            if let Some(last) = segments.last() {
                if let Some(type_id) = self.type_map.get(*last) {
                    return Some(type_id.clone());
                }
                if let Some(func_id) = self.function_map.get(*last) {
                    return Some(func_id.clone());
                }
            }
        }

        None
    }

    /// Find the analyzed file whose module path matches the longest prefix of
    /// `segments`. Ambiguous matches resolve to nothing rather than to a guess.
    fn resolve_module_prefix(&self, segments: &[&str]) -> Option<String> {
        for len in (1..=segments.len()).rev() {
            let key = segments[..len].join("::");
            if let Some(candidates) = self.module_map.get(&key) {
                if candidates.len() == 1 {
                    return Some(candidates[0].clone());
                }
                return None; // ambiguous — do not invent an edge
            }
        }
        None
    }
}

/// Split a file path into module path segments: `src/utils/helpers.rs` ->
/// ["src", "utils", "helpers"], `src/utils/mod.rs` -> ["src", "utils"].
fn module_path_segments(file_path: &str) -> Vec<String> {
    let trimmed = file_path.trim_start_matches("./").trim_start_matches('/');
    let path = std::path::Path::new(trimmed);

    let mut segments: Vec<String> = path
        .parent()
        .map(|parent| {
            parent
                .components()
                .filter_map(|c| c.as_os_str().to_str())
                .filter(|s| !s.is_empty() && *s != "." && *s != "..")
                .map(std::string::ToString::to_string)
                .collect()
        })
        .unwrap_or_default();

    // `foo/mod.rs` IS module `foo`, so it contributes no extra segment.
    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
        if stem != "mod" {
            segments.push(stem.to_string());
        }
    }

    segments
}

/// Strip `crate::`/`self::`/`super::`/glob decorations from an import path and
/// report whether the path was crate-relative.
fn normalize_import_segments(import_path: &str) -> (Vec<&str>, bool) {
    let mut segments: Vec<&str> = import_path
        .split("::")
        .filter(|s| !s.is_empty() && *s != "*")
        .collect();

    let mut intra_crate = false;
    while let Some(first) = segments.first() {
        if matches!(*first, "crate" | "$crate" | "self" | "super") {
            intra_crate = true;
            segments.remove(0);
        } else {
            break;
        }
    }

    (segments, intra_crate)
}

/// Detect programming language from file path extension
fn detect_language_from_path(file_path: &str) -> &'static str {
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
