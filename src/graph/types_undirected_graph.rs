// UndirectedGraph implementation methods
// Included by types.rs - shares parent module scope (no `use` imports)

impl Default for UndirectedGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl UndirectedGraph {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self {
            graph: CsrGraph::new(),
            node_data: HashMap::new(),
            edge_weights: HashMap::new(),
            next_id: 0,
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Add node.
    pub fn add_node(&mut self, data: NodeData) -> NodeId {
        let id = TruenoNodeId(self.next_id);
        self.next_id += 1;
        self.node_data.insert(id, data);
        id
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Add edge.
    pub fn add_edge(&mut self, from: NodeId, to: NodeId, weight: f64) {
        // Store both directions for undirected access
        self.edge_weights.insert((from, to), weight);
        self.edge_weights.insert((to, from), weight);
        // Add to trueno-graph
        let _ = self.graph.add_edge(from, to, weight as f32);
        let _ = self.graph.add_edge(to, from, weight as f32);
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Node count.
    pub fn node_count(&self) -> usize {
        self.node_data.len()
    }

    /// Edge count.
    pub fn edge_count(&self) -> usize {
        // Divide by 2 since we store both directions
        self.edge_weights.len() / 2
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Node weight.
    pub fn node_weight(&self, id: NodeId) -> Option<&NodeData> {
        self.node_data.get(&id)
    }

    /// Edge weight.
    pub fn edge_weight(&self, from: NodeId, to: NodeId) -> Option<f64> {
        self.edge_weights.get(&(from, to)).copied()
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Node indices.
    pub fn node_indices(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.node_data.keys().copied()
    }

    /// Neighbors.
    pub fn neighbors(&self, node: NodeId) -> Vec<NodeId> {
        self.graph
            .outgoing_neighbors(node)
            .map(|neighbors| neighbors.iter().map(|&n| TruenoNodeId(n)).collect())
            .unwrap_or_default()
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Edge references.
    pub fn edge_references(&self) -> impl Iterator<Item = UndirectedEdgeRef<'_>> + '_ {
        // Only return each edge once (from < to)
        self.edge_weights
            .iter()
            .filter(|((from, to), _)| from.0 < to.0)
            .map(|((from, to), weight)| UndirectedEdgeRef {
                source: *from,
                target: *to,
                weight: *weight,
                _phantom: std::marker::PhantomData,
            })
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Inner.
    pub fn inner(&self) -> &CsrGraph {
        &self.graph
    }

    /// Node references.
    pub fn node_references(&self) -> impl Iterator<Item = (NodeId, &NodeData)> + '_ {
        self.node_data.iter().map(|(id, data)| (*id, data))
    }
}

impl UndirectedEdgeRef<'_> {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Source.
    pub fn source(&self) -> NodeId {
        self.source
    }

    /// Target.
    pub fn target(&self) -> NodeId {
        self.target
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Weight.
    pub fn weight(&self) -> f64 {
        self.weight
    }
}
