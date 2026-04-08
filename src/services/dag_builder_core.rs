// DagBuilder core: constructor, build_from_project, finalize_graph, build_from_project_with_limit

impl DagBuilder {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self {
            graph: DependencyGraph::new(),
            function_map: FxHashMap::default(),
            type_map: FxHashMap::default(),
            namer: SemanticNamer::new(),
        }
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Build from project.
    pub fn build_from_project(project: &ProjectContext) -> DependencyGraph {
        let mut builder = Self::new();

        // First pass: collect all nodes and build lookup maps
        for file in &project.files {
            builder.collect_nodes(file);
        }

        // Second pass: create edges based on relationships
        for file in &project.files {
            builder.process_relationships(file);
        }

        builder.finalize_graph()
    }

    const EDGE_BUDGET: usize = 400; // Empirically derived Mermaid limit

    fn finalize_graph(mut self) -> DependencyGraph {
        // First, remove edges that reference non-existent nodes
        let valid_nodes: FxHashSet<&String> = self.graph.nodes.keys().collect();
        self.graph
            .edges
            .retain(|edge| valid_nodes.contains(&edge.from) && valid_nodes.contains(&edge.to));

        if self.graph.edges.len() > Self::EDGE_BUDGET {
            self.prune_edges_by_priority();
        }

        self.graph
    }

    /// Prune edges when over budget, keeping highest-priority edge types
    fn prune_edges_by_priority(&mut self) {
        // Priority-based edge sorting (Inherits > Uses > Implements > Call > Import)
        let priority = |edge_type: &EdgeType| -> u8 {
            match edge_type {
                EdgeType::Inherits => 0,
                EdgeType::Uses => 1,
                EdgeType::Implements => 2,
                EdgeType::Calls => 3,
                EdgeType::Imports => 4,
            }
        };

        // Sort edges by priority (lower number = higher priority)
        self.graph
            .edges
            .sort_unstable_by_key(|e| priority(&e.edge_type));

        // Truncate to budget limit
        self.graph.edges.truncate(Self::EDGE_BUDGET);

        // Maintain node consistency - only keep nodes referenced in remaining edges
        let retained_nodes: FxHashSet<String> = self
            .graph
            .edges
            .iter()
            .flat_map(|e| [e.from.clone(), e.to.clone()])
            .collect();

        self.graph.nodes.retain(|id, _| retained_nodes.contains(id));
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Build from project with limit.
    pub fn build_from_project_with_limit(
        project: &ProjectContext,
        max_nodes: usize,
    ) -> DependencyGraph {
        let graph = Self::build_from_project(project);

        // Always calculate PageRank scores for centrality (takes ownership, no clone)
        let graph = add_pagerank_scores(graph);

        if graph.edges.len() > 400 {
            // Safety margin for Mermaid - prune but keep scores
            prune_graph_pagerank(&graph, max_nodes)
        } else {
            graph
        }
    }
}

impl Default for DagBuilder {
    fn default() -> Self {
        Self::new()
    }
}
