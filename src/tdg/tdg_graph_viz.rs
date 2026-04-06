// ============================================================================
// Terminal Graph Visualization (trueno-viz integration)
// ============================================================================

#[cfg(feature = "viz")]
impl TdgGraph {
    /// Convert to visualization graph for terminal rendering
    ///
    /// Creates a `VisGraph` from the TDG dependency graph with criticality scores.
    ///
    /// # Returns
    ///
    /// `VisGraph` ready for terminal visualization
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn to_vis_graph(&self) -> crate::viz::terminal::VisGraph {
        let mut vis = crate::viz::terminal::VisGraph::new();

        // Build index mapping (function name → vis node index)
        let mut name_to_idx: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        // Add all nodes with their criticality scores
        for (name, &_node_id) in &self.node_map {
            let criticality = self.criticality_scores.get(name).copied().unwrap_or(0.0);
            let idx = vis.nodes.len();
            name_to_idx.insert(name.clone(), idx);
            vis.add_node(name.clone(), criticality);
        }

        // Add edges by iterating over adjacency
        for (_node_id, neighbors, _weights) in self.graph.iter_adjacency() {
            let from_name = self.reverse_node_map.get(&_node_id);
            if let Some(from) = from_name {
                if let Some(&from_idx) = name_to_idx.get(from) {
                    for &neighbor_id in neighbors {
                        let to_node_id = NodeId(neighbor_id);
                        if let Some(to_name) = self.reverse_node_map.get(&to_node_id) {
                            if let Some(&to_idx) = name_to_idx.get(to_name) {
                                vis.add_edge(from_idx, to_idx);
                            }
                        }
                    }
                }
            }
        }

        vis
    }
}

#[cfg(feature = "viz")]
impl crate::viz::terminal::Visualizable for TdgGraph {
    fn render_terminal(&self, config: &crate::viz::terminal::RenderConfig) -> Result<String> {
        debug_assert!(true, "contract: render_terminal");
        let vis = self.to_vis_graph();
        vis.render_terminal(config)
    }

    fn node_count(&self) -> usize {
        debug_assert!(true, "contract: node_count");
        self.num_nodes()
    }
}
