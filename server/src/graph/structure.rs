// Graph structural analysis using aprender v0.5.0
// Implements 4 structural statistics (Phase 4: Graph Migration)

use super::aprender_adapter::to_aprender_graph;
use super::*;
use petgraph::algo::{connected_components, is_cyclic_directed, kosaraju_scc};
use petgraph::visit::EdgeRef;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct StructuralMetrics {
    pub density: f64,
    pub diameter: Option<usize>,
    pub radius: Option<usize>,
    pub average_degree: f64,
    pub clustering_coefficient: f64,
    pub assortativity: f64,
    pub components: usize,
    pub strongly_connected_components: usize,
    pub is_cyclic: bool,
    pub transitivity: f64,
    pub reciprocity: Option<f64>,
}

pub struct StructuralAnalyzer {
    pub directed: bool,
}

impl StructuralAnalyzer {
    pub fn new(directed: bool) -> Self {
        StructuralAnalyzer { directed }
    }

    /// Analyze graph structure using aprender v0.5.0 + petgraph.
    ///
    /// # Arguments
    /// * `graph` - PMAT dependency graph
    ///
    /// # Returns
    /// StructuralMetrics with 11 structural properties
    ///
    /// # Implementation
    /// - Uses aprender for: density, diameter, clustering, assortativity
    /// - Uses petgraph for: components, cycles, reciprocity
    pub fn analyze(&self, graph: &DependencyGraph) -> StructuralMetrics {
        // Convert to aprender graph
        let aprender_graph = to_aprender_graph(graph, self.directed);

        // Compute aprender-based metrics
        let density = aprender_graph.density();
        let diameter = aprender_graph.diameter();
        let clustering_coefficient = aprender_graph.clustering_coefficient();
        let assortativity = aprender_graph.assortativity();

        // Compute average degree
        let average_degree = if graph.node_count() > 0 {
            graph.edge_count() as f64 / graph.node_count() as f64
        } else {
            0.0
        };

        // Compute component counts
        let (components, strongly_connected_components) = if self.directed {
            let scc_count = kosaraju_scc(graph).len();
            (scc_count, scc_count)
        } else {
            let comp_count = connected_components(graph);
            (comp_count, comp_count)
        };

        // Check for cycles
        let is_cyclic = if self.directed {
            is_cyclic_directed(graph)
        } else {
            // For undirected graphs, any edge is a cycle
            graph.edge_count() > 0
        };

        // Compute transitivity (same as clustering coefficient for global measure)
        let transitivity = clustering_coefficient;

        // Compute reciprocity (for directed graphs)
        let reciprocity = if self.directed {
            Some(compute_reciprocity(graph))
        } else {
            None
        };

        // Radius not implemented yet (requires all-pairs shortest paths)
        let radius = None;

        StructuralMetrics {
            density,
            diameter,
            radius,
            average_degree,
            clustering_coefficient,
            assortativity,
            components,
            strongly_connected_components,
            is_cyclic,
            transitivity,
            reciprocity,
        }
    }
}

/// Compute reciprocity: fraction of edges that have a reverse edge.
fn compute_reciprocity(graph: &DependencyGraph) -> f64 {
    if graph.edge_count() == 0 {
        return 0.0;
    }

    let mut reciprocal_count = 0;
    let total_edges = graph.edge_count();

    for edge in graph.edge_references() {
        let source = edge.source();
        let target = edge.target();

        // Check if reverse edge exists
        if graph.contains_edge(target, source) {
            reciprocal_count += 1;
        }
    }

    // Each reciprocal pair is counted twice, so divide by 2
    reciprocal_count as f64 / total_edges as f64
}
