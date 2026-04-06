// Utility methods for Mermaid engine
// Included by deterministic_mermaid_engine.rs - shares parent module scope

impl DeterministicMermaidEngine {
    /// Filter graph to service modules only (heuristic)
    fn filter_to_services(
        &self,
        graph: &SimpleStableGraph<ModuleNode, EdgeType>,
    ) -> SimpleStableGraph<ModuleNode, EdgeType> {
        let mut service_graph = SimpleStableGraph::new();
        let mut node_mapping = BTreeMap::new();

        // Add nodes that look like services
        for idx in graph.node_indices() {
            let node = &graph[idx];
            if self.is_service_module(&node.name) {
                let new_idx = service_graph.add_node(node.clone());
                node_mapping.insert(idx, new_idx);
            }
        }

        // Add edges between service nodes
        for edge in graph.edge_references() {
            if let (Some(&source_idx), Some(&target_idx)) = (
                node_mapping.get(&edge.source()),
                node_mapping.get(&edge.target()),
            ) {
                service_graph.add_edge(source_idx, target_idx, edge.weight().clone());
            }
        }

        service_graph
    }

    /// Heuristic to determine if a module is a service
    fn is_service_module(&self, name: &str) -> bool {
        name.contains("service")
            || name.contains("handler")
            || name.contains("controller")
            || name.contains("api")
            || name.contains("engine")
    }

    /// Get Mermaid arrow style for edge type
    fn get_edge_arrow(&self, edge_type: &EdgeType) -> &'static str {
        match edge_type {
            EdgeType::Calls => "-->",
            EdgeType::Imports => "-.->",
            EdgeType::Inherits => "-->",
            EdgeType::Implements => "-.->",
            EdgeType::Uses => "---",
        }
    }

    /// Sanitize ID for Mermaid compatibility
    #[must_use]
    pub fn sanitize_id(&self, id: &str) -> String {
        debug_assert!(!id.is_empty(), "id must not be empty");
        // Replace common multi-character patterns
        let sanitized = id.replace("::", "_").replace(['/', '.', '-', ' '], "_");

        // Replace any remaining non-alphanumeric characters with underscores
        let sanitized: String = sanitized
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect();

        // Ensure it starts with a letter or underscore
        if sanitized.is_empty() {
            "_empty".to_string()
        } else if sanitized
            .chars()
            .next()
            .expect("sanitized is non-empty (checked above)")
            .is_numeric()
        {
            format!("_{sanitized}")
        } else {
            sanitized
        }
    }

    /// Escape label for Mermaid compatibility
    #[must_use]
    pub fn escape_mermaid_label(&self, label: &str) -> String {
        // For maximum compatibility, use simple character replacements
        label
            .replace('&', " and ")
            .replace('"', "'")
            .replace('<', "(")
            .replace('>', ")")
            .replace('|', " - ")
            .replace('[', "(")
            .replace(']', ")")
            .replace('{', "(")
            .replace('}', ")")
            .replace('\n', " ")
    }
}
