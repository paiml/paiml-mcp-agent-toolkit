// GraphData - internal graph representation for Louvain algorithm
// Included by parallel_louvain.rs - shares parent scope

/// Internal graph representation optimized for Louvain algorithm.
#[derive(Debug)]
struct GraphData {
    /// Number of nodes
    n: usize,
    /// Adjacency list: neighbors[i] = [(neighbor_idx, weight), ...]
    neighbors: Vec<Vec<(usize, f64)>>,
    /// Node degrees (sum of edge weights)
    degrees: Vec<f64>,
    /// Total graph weight (sum of all edge weights)
    total_weight: f64,
    /// Edge weights for quick lookup
    edge_weights: HashMap<(usize, usize), f64>,
}

impl GraphData {
    /// Build graph data from an undirected graph.
    fn from_graph(graph: &UndirectedGraph) -> Self {
        let n = graph.node_count();
        let mut neighbors: Vec<Vec<(usize, f64)>> = vec![Vec::new(); n];
        let mut degrees = vec![0.0; n];
        let mut total_weight = 0.0;
        let mut edge_weights: HashMap<(usize, usize), f64> = HashMap::new();

        for edge in graph.edge_references() {
            let source = edge.source().0 as usize;
            let target = edge.target().0 as usize;
            let weight = edge.weight();

            neighbors[source].push((target, weight));
            neighbors[target].push((source, weight));

            degrees[source] += weight;
            degrees[target] += weight;
            total_weight += weight;

            // Store edge in both directions for lookup
            let key = if source <= target {
                (source, target)
            } else {
                (target, source)
            };
            edge_weights.insert(key, weight);
        }

        GraphData {
            n,
            neighbors,
            degrees,
            total_weight,
            edge_weights,
        }
    }

    /// Calculate sum of weights from a node to nodes in a specific community.
    fn neighbor_weight_to_community(
        &self,
        node: usize,
        community: usize,
        node_to_community: &[usize],
    ) -> f64 {
        self.neighbors[node]
            .iter()
            .filter(|(neighbor, _)| node_to_community[*neighbor] == community)
            .map(|(_, weight)| weight)
            .sum()
    }
}
