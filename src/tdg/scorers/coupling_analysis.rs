// CouplingAnalyzer impl: afferent/efferent coupling calculation, abstractness,
// visibility checks, import extraction, builtin type detection, and dependency graph building.

impl CouplingAnalyzer {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self
    }

    fn calculate_afferent_coupling(&self, root: Node, source: &str) -> usize {
        let mut incoming = HashSet::new();

        walk_tree(root, |node| {
            match node.kind() {
                "function_item" | "impl_item" | "struct_item" | "trait_item" => {
                    if let Some(name) = node.child_by_field_name("name") {
                        let fn_name = get_node_text(name, source);
                        if self.is_public(node, source) {
                            incoming.insert(fn_name.to_string());
                        }
                    }
                }
                _ => {}
            }
        });

        incoming.len()
    }

    fn calculate_efferent_coupling(&self, root: Node, source: &str) -> usize {
        let mut outgoing = HashSet::new();

        walk_tree(root, |node| {
            match node.kind() {
                "use_declaration" | "use" | "import" | "extern_crate_declaration" => {
                    if let Some(path) = self.extract_import_path(node, source) {
                        outgoing.insert(path);
                    }
                }
                "call_expression" => {
                    if let Some(function) = node.child_by_field_name("function") {
                        let fn_text = get_node_text(function, source);
                        if fn_text.contains("::") {
                            outgoing.insert(fn_text.to_string());
                        }
                    }
                }
                "type_identifier" | "generic_type" => {
                    let type_text = get_node_text(node, source);
                    if !self.is_builtin_type(type_text) {
                        outgoing.insert(type_text.to_string());
                    }
                }
                _ => {}
            }
        });

        outgoing.len()
    }

    fn is_public(&self, node: Node, source: &str) -> bool {
        if let Some(visibility) = node.child_by_field_name("visibility_modifier") {
            let vis_text = get_node_text(visibility, source);
            vis_text.contains("pub")
        } else {
            false
        }
    }

    fn extract_import_path(&self, node: Node, source: &str) -> Option<String> {
        if let Some(path) = node.child_by_field_name("path") {
            Some(get_node_text(path, source).to_string())
        } else if let Some(argument) = node.child_by_field_name("argument") {
            Some(get_node_text(argument, source).to_string())
        } else {
            let text = get_node_text(node, source);
            let parts: Vec<&str> = text.split_whitespace().collect();
            if parts.len() > 1 {
                Some(parts[1].to_string())
            } else {
                None
            }
        }
    }

    fn is_builtin_type(&self, type_name: &str) -> bool {
        matches!(
            type_name,
            "i8" | "i16" | "i32" | "i64" | "i128" | "isize" |
            "u8" | "u16" | "u32" | "u64" | "u128" | "usize" |
            "f32" | "f64" | "bool" | "char" | "str" | "String" |
            "Vec" | "HashMap" | "HashSet" | "Option" | "Result" |
            "Box" | "Rc" | "Arc" | "Cell" | "RefCell" | "Mutex" |
            "int" | "float" | "double" | "void" | "auto" | "const"
        )
    }

    fn calculate_abstractness(&self, root: Node, source: &str) -> f32 {
        let mut abstract_count = 0;
        let mut total_count = 0;

        walk_tree(root, |node| {
            match node.kind() {
                "trait_item" => {
                    abstract_count += 1;
                    total_count += 1;
                }
                "impl_item" => {
                    total_count += 1;
                    if node.child_by_field_name("trait").is_some() {
                        abstract_count += 1;
                    }
                }
                "struct_item" | "enum_item" => {
                    total_count += 1;
                }
                _ => {}
            }
        });

        if total_count > 0 {
            abstract_count as f32 / total_count as f32
        } else {
            0.0
        }
    }

    fn build_dependency_graph(&self, root: Node, source: &str) -> DependencyGraph {
        let mut graph = DependencyGraph::new();
        let mut current_module = None;

        walk_tree(root, |node| {
            match node.kind() {
                "mod_item" | "module" => {
                    if let Some(name) = node.child_by_field_name("name") {
                        current_module = Some(get_node_text(name, source).to_string());
                    }
                }
                "use_declaration" | "use" | "import" => {
                    if let Some(module) = &current_module {
                        if let Some(imported) = self.extract_import_path(node, source) {
                            graph.add_edge(module.clone(), imported);
                        }
                    }
                }
                _ => {}
            }
        });

        graph
    }
}
