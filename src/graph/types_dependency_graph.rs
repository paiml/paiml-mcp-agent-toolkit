// DependencyGraph implementation methods
// Included by types.rs - shares parent module scope (no `use` imports)

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyGraph {
    /// Create a new empty dependency graph
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            graph: CsrGraph::new(),
            node_data: HashMap::new(),
            edge_data: HashMap::new(),
            next_id: 0,
        }
    }

    /// Add a node to the graph and return its ID
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn add_node(&mut self, data: NodeData) -> NodeId {
        let id = TruenoNodeId(self.next_id);
        self.next_id += 1;
        self.node_data.insert(id, data);
        id
    }

    /// Add an edge between two nodes
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn add_edge(&mut self, from: NodeId, to: NodeId, data: EdgeData) {
        // Store edge data
        self.edge_data.insert((from, to), data);
        // Add to trueno-graph with weight 1.0
        let _ = self.graph.add_edge(from, to, 1.0);
    }

    /// Get node count
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn node_count(&self) -> usize {
        self.node_data.len()
    }

    /// Get edge count
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn edge_count(&self) -> usize {
        self.edge_data.len()
    }

    /// Get node data by ID
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn node_weight(&self, id: NodeId) -> Option<&NodeData> {
        self.node_data.get(&id)
    }

    /// Get mutable node data by ID
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn node_weight_mut(&mut self, id: NodeId) -> Option<&mut NodeData> {
        self.node_data.get_mut(&id)
    }

    /// Get edge data between two nodes
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn edge_weight(&self, from: NodeId, to: NodeId) -> Option<&EdgeData> {
        self.edge_data.get(&(from, to))
    }

    /// Check if an edge exists
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn contains_edge(&self, from: NodeId, to: NodeId) -> bool {
        self.edge_data.contains_key(&(from, to))
    }

    /// Get all node IDs
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn node_indices(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.node_data.keys().copied()
    }

    /// Iterate over all edges with their data
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn edge_references(&self) -> impl Iterator<Item = EdgeRef<'_>> + '_ {
        self.edge_data.iter().map(|((from, to), data)| EdgeRef {
            source: *from,
            target: *to,
            weight: data,
        })
    }

    /// Get outgoing neighbors of a node
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn neighbors(&self, node: NodeId) -> Vec<NodeId> {
        self.graph
            .outgoing_neighbors(node)
            .map(|neighbors| neighbors.iter().map(|&n| TruenoNodeId(n)).collect())
            .unwrap_or_default()
    }

    /// Get outgoing neighbors (alias for neighbors)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn neighbors_directed_outgoing(&self, node: NodeId) -> Vec<NodeId> {
        self.neighbors(node)
    }

    /// Get incoming neighbors of a node
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn neighbors_directed_incoming(&self, node: NodeId) -> Vec<NodeId> {
        self.graph
            .incoming_neighbors(node)
            .map(|neighbors| neighbors.iter().map(|&n| TruenoNodeId(n)).collect())
            .unwrap_or_default()
    }

    /// Get edges from a node (outgoing)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn edges(&self, node: NodeId) -> impl Iterator<Item = EdgeRef<'_>> + '_ {
        self.edge_data
            .iter()
            .filter(move |((from, _), _)| *from == node)
            .map(|((from, to), data)| EdgeRef {
                source: *from,
                target: *to,
                weight: data,
            })
    }

    /// Get underlying trueno-graph (for algorithms)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn inner(&self) -> &CsrGraph {
        &self.graph
    }

    /// Iterate over nodes with their data
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn node_references(&self) -> impl Iterator<Item = (NodeId, &NodeData)> + '_ {
        self.node_data.iter().map(|(id, data)| (*id, data))
    }
}

impl<'a> EdgeRef<'a> {
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
    pub fn weight(&self) -> &'a EdgeData {
        self.weight
    }
}
