// PageRank and graph algorithm methods
// Included by deterministic_mermaid_engine.rs - shares parent module scope

impl DeterministicMermaidEngine {
    /// Compute `PageRank` scores for graph nodes
    fn compute_pagerank(
        &self,
        graph: &SimpleStableGraph<ModuleNode, EdgeType>,
        damping: f32,
        iterations: usize,
    ) -> BTreeMap<NodeIndex, f32> {
        debug_assert!(true, "contract: compute_pagerank");
        let node_count = graph.node_count();
        if node_count == 0 {
            return BTreeMap::new();
        }

        let initial_score = 1.0 / node_count as f32;
        let mut scores: BTreeMap<NodeIndex, f32> = graph
            .node_indices()
            .map(|idx| (idx, initial_score))
            .collect();

        // Build adjacency information
        let mut outgoing: BTreeMap<NodeIndex, Vec<NodeIndex>> = BTreeMap::new();
        let mut incoming: BTreeMap<NodeIndex, Vec<NodeIndex>> = BTreeMap::new();

        for idx in graph.node_indices() {
            outgoing.insert(idx, Vec::new());
            incoming.insert(idx, Vec::new());
        }

        for edge in graph.edge_references() {
            outgoing
                .get_mut(&edge.source())
                .expect("node exists in outgoing map (inserted above)")
                .push(edge.target());
            incoming
                .get_mut(&edge.target())
                .expect("node exists in incoming map (inserted above)")
                .push(edge.source());
        }

        // Iterative PageRank computation
        for _ in 0..iterations {
            let mut new_scores = BTreeMap::new();

            let node_indices: Vec<_> = graph.node_indices().collect();
            for &node in &node_indices {
                let mut score = (1.0 - damping) / node_count as f32;

                if let Some(incoming_nodes) = incoming.get(&node) {
                    for &incoming_node in incoming_nodes {
                        if let Some(outgoing_nodes) = outgoing.get(&incoming_node) {
                            let outgoing_count = outgoing_nodes.len() as f32;
                            if outgoing_count > 0.0 {
                                if let Some(&incoming_score) = scores.get(&incoming_node) {
                                    score += damping * incoming_score / outgoing_count;
                                }
                            }
                        }
                    }
                }

                new_scores.insert(node, score);
            }

            scores = new_scores;
        }

        scores
    }
}
