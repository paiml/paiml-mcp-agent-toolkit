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
            module_map: FxHashMap::default(),
            namer: SemanticNamer::new(),
        }
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Build from project, with the Mermaid edge budget already enforced.
    pub fn build_from_project(project: &ProjectContext) -> DependencyGraph {
        enforce_edge_budget(Self::build_from_project_unbudgeted(project))
    }

    /// Build the COMPLETE graph: every declared node, every resolved edge, with
    /// no presentation budget applied.
    ///
    /// #1020: [`Self::build_from_project`] truncates to [`EDGE_BUDGET`] edges and
    /// then keeps only the nodes those surviving edges touch. On any tree with
    /// more than 400 import/inheritance edges — `src/services` has thousands —
    /// that deletes every `Function` node, so the call-edge pass that runs
    /// *after* the build had no function to walk from, produced zero `Calls`
    /// edges, and `--dag-type call-graph` came back "0 nodes, 0 edges" on both
    /// the CLI and the MCP tool while `full-dependency` on the same directory
    /// answered 369/400. Any caller that enriches or filters the graph after
    /// building MUST start from this, and apply the budget last — see
    /// [`crate::services::dag_pipeline::build_typed_dag`].
    #[must_use]
    pub fn build_from_project_unbudgeted(project: &ProjectContext) -> DependencyGraph {
        let mut builder = Self::new();

        // First pass: collect all nodes and build lookup maps
        for file in &project.files {
            builder.collect_nodes(file);
        }

        // Second pass: create edges based on relationships
        for file in &project.files {
            builder.process_relationships(file);
        }

        builder.drop_dangling_edges()
    }

    /// Remove edges that reference nodes which were never created.
    fn drop_dangling_edges(mut self) -> DependencyGraph {
        let valid_nodes: FxHashSet<&String> = self.graph.nodes.keys().collect();
        self.graph
            .edges
            .retain(|edge| valid_nodes.contains(&edge.from) && valid_nodes.contains(&edge.to));
        self.graph
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

/// Empirically derived Mermaid rendering limit.
pub const EDGE_BUDGET: usize = 400;

/// Cut an over-sized graph down to something Mermaid can draw.
///
/// This is a PRESENTATION step and it is destructive twice over: it drops the
/// lowest-priority edges, and it then drops every node those surviving edges do
/// not touch. Run it LAST, on the graph the caller actually asked for — running
/// it before the `--dag-type` filter is what made `call-graph` return an empty
/// graph on every real tree (#1020).
#[must_use]
pub fn enforce_edge_budget(mut graph: DependencyGraph) -> DependencyGraph {
    if graph.edges.len() <= EDGE_BUDGET {
        return graph;
    }

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
    graph.edges.sort_by_key(|e| priority(&e.edge_type));
    graph.edges.truncate(EDGE_BUDGET);

    // Maintain node consistency - only keep nodes referenced in remaining edges
    let retained_nodes: FxHashSet<String> = graph
        .edges
        .iter()
        .flat_map(|e| [e.from.clone(), e.to.clone()])
        .collect();

    graph.nodes.retain(|id, _| retained_nodes.contains(id));
    graph
}
