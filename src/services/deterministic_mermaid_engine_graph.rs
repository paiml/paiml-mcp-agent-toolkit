// SimpleStableGraph implementation methods
// Included by deterministic_mermaid_engine.rs - shares parent module scope

impl<N: Clone, E: Clone> SimpleStableGraph<N, E> {
    /// Create a new empty stable graph
    #[must_use]
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Add a node to the graph and return its index
    pub fn add_node(&mut self, node: N) -> NodeIndex {
        let idx = self.nodes.len();
        self.nodes.push(Some(node));
        NodeIndex(idx)
    }

    /// Add an edge between two nodes
    pub fn add_edge(&mut self, source: NodeIndex, target: NodeIndex, weight: E) {
        self.edges.push(Edge {
            source,
            target,
            weight,
        });
    }

    fn node_count(&self) -> usize {
        self.nodes.iter().filter(|n| n.is_some()).count()
    }

    fn node_indices(&self) -> impl Iterator<Item = NodeIndex> + '_ {
        self.nodes
            .iter()
            .enumerate()
            .filter_map(|(i, n)| n.as_ref().map(|_| NodeIndex(i)))
    }

    fn edge_references(&self) -> impl Iterator<Item = EdgeRef<'_, E>> + '_ {
        self.edges.iter().map(|e| EdgeRef {
            source: e.source,
            target: e.target,
            weight: &e.weight,
        })
    }

    #[allow(dead_code)]
    fn get_node(&self, idx: NodeIndex) -> Option<&N> {
        self.nodes.get(idx.0).and_then(|n| n.as_ref())
    }
}
