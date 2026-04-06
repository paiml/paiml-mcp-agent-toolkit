// Mermaid diagram rendering methods
// Included by deterministic_mermaid_engine.rs - shares parent module scope

impl DeterministicMermaidEngine {
    /// Generate deterministic codebase modules Mermaid diagram
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn generate_codebase_modules_mmd(
        &self,
        graph: &SimpleStableGraph<ModuleNode, EdgeType>,
    ) -> String {
        // Compute PageRank with fixed iterations for deterministic results
        let pagerank = self.compute_pagerank(graph, 0.85, self.pagerank_iterations);

        // Quantize scores to avoid floating-point drift
        let quantized: BTreeMap<NodeIndex, u32> = pagerank
            .into_iter()
            .map(|(idx, score)| (idx, (score * self.quantization_factor as f32) as u32))
            .collect();

        // Generate deterministic output
        let mut mermaid = String::from("graph TD\n");

        // Generate nodes in stable order (by PageRank score, then by name)
        let mut nodes: Vec<_> = graph.node_indices().collect();
        nodes.sort_by_key(|&idx| {
            (
                std::cmp::Reverse(quantized.get(&idx).copied().unwrap_or(0)),
                graph[idx].name.clone(),
            )
        });

        for idx in nodes {
            let node = &graph[idx];
            let sanitized_id = self.sanitize_id(&node.name);
            let escaped_label = self.escape_mermaid_label(&node.name);

            writeln!(&mut mermaid, "    {sanitized_id}[{escaped_label}]")
                .expect("writing to String never fails");
        }

        // Add blank line between nodes and edges
        mermaid.push('\n');

        // Generate edges in stable order
        let mut edges: Vec<_> = graph.edge_references().collect();
        edges.sort_by_key(|e| {
            (
                graph[e.source()].name.clone(),
                graph[e.target()].name.clone(),
            )
        });

        for edge in edges {
            let arrow = self.get_edge_arrow(edge.weight());
            writeln!(
                &mut mermaid,
                "    {} {} {}",
                self.sanitize_id(&graph[edge.source()].name),
                arrow,
                self.sanitize_id(&graph[edge.target()].name)
            )
            .expect("writing to String never fails");
        }

        mermaid
    }

    /// Generate service interaction diagram with complexity-based styling
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn generate_service_interactions_mmd(
        &self,
        graph: &SimpleStableGraph<ModuleNode, EdgeType>,
        _metrics: &ProjectMetrics,
    ) -> String {
        // Filter to service modules only
        let service_graph = self.filter_to_services(graph);

        // Compute complexity-based styling buckets
        let complexity_scores: BTreeMap<NodeIndex, ComplexityBucket> = service_graph
            .node_indices()
            .map(|idx| {
                let node = &service_graph[idx];
                let score = node.metrics.complexity;
                let bucket = match score {
                    0..=10 => ComplexityBucket::Low,
                    11..=20 => ComplexityBucket::Medium,
                    _ => ComplexityBucket::High,
                };
                (idx, bucket)
            })
            .collect();

        // Generate with styling
        let mut mermaid = String::from("graph TD\n");

        // Generate nodes with deterministic ordering
        let mut nodes: Vec<_> = service_graph.node_indices().collect();
        nodes.sort_by_key(|&idx| &service_graph[idx].name);

        for idx in nodes {
            let node = &service_graph[idx];
            let sanitized_id = self.sanitize_id(&node.name);
            let escaped_label = self.escape_mermaid_label(&node.name);
            writeln!(&mut mermaid, "    {sanitized_id}[{escaped_label}]")
                .expect("writing to String never fails");
        }

        // Add blank line
        mermaid.push('\n');

        // Add edges in deterministic order
        let mut edges: Vec<_> = service_graph.edge_references().collect();
        edges.sort_by_key(|e| {
            (
                service_graph[e.source()].name.clone(),
                service_graph[e.target()].name.clone(),
            )
        });

        for edge in edges {
            let arrow = match edge.weight() {
                EdgeType::Calls => "-->",
                EdgeType::Imports => "---",
                EdgeType::Inherits => "-.->",
                EdgeType::Implements => "-.->",
                EdgeType::Uses => "---",
            };
            writeln!(
                &mut mermaid,
                "    {} {} {}",
                self.sanitize_id(&service_graph[edge.source()].name),
                arrow,
                self.sanitize_id(&service_graph[edge.target()].name)
            )
            .expect("writing to String never fails");
        }

        // Add deterministic styling
        mermaid.push('\n');
        for (idx, bucket) in &complexity_scores {
            let color = match bucket {
                ComplexityBucket::Low => "#90EE90",
                ComplexityBucket::Medium => "#FFA500",
                ComplexityBucket::High => "#FF6347",
            };
            writeln!(
                &mut mermaid,
                "    style {} fill:{},stroke-width:2px",
                self.sanitize_id(&service_graph[*idx].name),
                color
            )
            .expect("writing to String never fails");
        }

        mermaid
    }
}
