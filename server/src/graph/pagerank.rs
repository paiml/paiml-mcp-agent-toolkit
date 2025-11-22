// PageRank algorithm implementation using aprender v0.5.0
// Following Google's original algorithm with power iteration
// Implements Task 4.1 (Phase 4: Graph Migration)

use super::aprender_adapter::to_aprender_graph;
use super::*;

pub struct PageRankComputer {
    pub damping: f64,
    pub tolerance: f64,
    pub max_iterations: usize,
}

impl Default for PageRankComputer {
    fn default() -> Self {
        PageRankComputer {
            damping: 0.85,
            tolerance: 1e-6,
            max_iterations: 100,
        }
    }
}

impl PageRankComputer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_damping(mut self, damping: f64) -> Self {
        self.damping = damping;
        self
    }

    /// Compute PageRank using aprender v0.5.0.
    ///
    /// # Arguments
    /// * `graph` - PMAT dependency graph
    ///
    /// # Returns
    /// PageRank scores (Vec<f64>) or error fallback (zeros)
    ///
    /// # Algorithm
    /// - Power iteration with damping factor (default: 0.85)
    /// - Handles dangling nodes (no outgoing edges)
    /// - Convergence tolerance: 1e-6
    pub fn compute(&self, graph: &DependencyGraph) -> Vec<f64> {
        if graph.node_count() == 0 {
            return Vec::new();
        }

        // Convert to aprender graph (directed=true for PageRank)
        let aprender_graph = to_aprender_graph(graph, true);

        // Compute PageRank using aprender
        aprender_graph
            .pagerank(self.damping, self.max_iterations, self.tolerance)
            .unwrap_or_else(|_| vec![0.0; aprender_graph.num_nodes()])
    }

    /// Compute PageRank from legacy GraphMatrices (backward compatibility).
    ///
    /// This method is deprecated and will be removed in a future version.
    /// Use `compute(&DependencyGraph)` instead.
    #[deprecated(since = "2.201.0", note = "Use compute(&DependencyGraph) instead")]
    pub fn compute_legacy(&self, matrices: &GraphMatrices) -> Vec<f64> {
        // Build temporary DependencyGraph from GraphMatrices
        let mut graph = DependencyGraph::new();

        // Add nodes
        for _ in 0..matrices.node_count {
            graph.add_node(NodeData {
                path: std::path::PathBuf::from("legacy"),
                module: "legacy".to_string(),
                symbols: vec![],
                loc: 0,
                complexity: 0.0,
                ast_hash: 0,
            });
        }

        // Add edges
        for (from, to, weight) in &matrices.edges {
            let from_idx = petgraph::graph::NodeIndex::new(*from);
            let to_idx = petgraph::graph::NodeIndex::new(*to);

            graph.add_edge(
                from_idx,
                to_idx,
                EdgeData::Import {
                    weight: *weight,
                    visibility: Visibility::Public,
                },
            );
        }

        self.compute(&graph)
    }
}
