#![cfg_attr(coverage_nightly, coverage(off))]
// Graph structural analysis using aprender v0.5.0 + trueno-graph
// Implements 4 structural statistics (Phase 4: Graph Migration)

use super::aprender_adapter::to_aprender_graph;
use super::*;
use serde::Serialize;
use trueno_graph::{connected_components, is_cyclic, kosaraju_scc};

#[derive(Debug, Clone, Serialize)]
/// Structural metrics.
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

/// Structural analyzer.
pub struct StructuralAnalyzer {
    pub directed: bool,
}

impl StructuralAnalyzer {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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

        // Compute component counts using trueno-graph algorithms
        let (components, strongly_connected_components) = if self.directed {
            let scc_count = kosaraju_scc(graph.inner()).len();
            (scc_count, scc_count)
        } else {
            let comp_count = connected_components(graph.inner());
            (comp_count, comp_count)
        };

        // Check for cycles using trueno-graph
        let has_cycles = if self.directed {
            is_cyclic(graph.inner())
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
            is_cyclic: has_cycles,
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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // StructuralMetrics Tests
    // =========================================================================

    #[test]
    fn test_structural_metrics_all_fields() {
        let metrics = StructuralMetrics {
            density: 0.5,
            diameter: Some(3),
            radius: Some(2),
            average_degree: 2.5,
            clustering_coefficient: 0.3,
            assortativity: 0.1,
            components: 1,
            strongly_connected_components: 1,
            is_cyclic: false,
            transitivity: 0.3,
            reciprocity: Some(0.5),
        };

        assert!((metrics.density - 0.5).abs() < 0.001);
        assert_eq!(metrics.diameter, Some(3));
        assert_eq!(metrics.radius, Some(2));
        assert!(!metrics.is_cyclic);
    }

    #[test]
    fn test_structural_metrics_none_values() {
        let metrics = StructuralMetrics {
            density: 0.0,
            diameter: None,
            radius: None,
            average_degree: 0.0,
            clustering_coefficient: 0.0,
            assortativity: 0.0,
            components: 0,
            strongly_connected_components: 0,
            is_cyclic: false,
            transitivity: 0.0,
            reciprocity: None,
        };

        assert!(metrics.diameter.is_none());
        assert!(metrics.radius.is_none());
        assert!(metrics.reciprocity.is_none());
    }

    #[test]
    fn test_structural_metrics_clone() {
        let metrics = StructuralMetrics {
            density: 0.5,
            diameter: Some(3),
            radius: None,
            average_degree: 2.0,
            clustering_coefficient: 0.3,
            assortativity: 0.1,
            components: 1,
            strongly_connected_components: 1,
            is_cyclic: true,
            transitivity: 0.3,
            reciprocity: Some(0.5),
        };

        let cloned = metrics.clone();
        assert!((cloned.density - metrics.density).abs() < 0.001);
        assert_eq!(cloned.diameter, metrics.diameter);
    }

    #[test]
    fn test_structural_metrics_debug() {
        let metrics = StructuralMetrics {
            density: 0.5,
            diameter: None,
            radius: None,
            average_degree: 0.0,
            clustering_coefficient: 0.0,
            assortativity: 0.0,
            components: 0,
            strongly_connected_components: 0,
            is_cyclic: false,
            transitivity: 0.0,
            reciprocity: None,
        };

        let debug_str = format!("{:?}", metrics);
        assert!(debug_str.contains("StructuralMetrics"));
    }

    // =========================================================================
    // StructuralAnalyzer Tests
    // =========================================================================

    #[test]
    fn test_structural_analyzer_new_directed() {
        let analyzer = StructuralAnalyzer::new(true);
        assert!(analyzer.directed);
    }

    #[test]
    fn test_structural_analyzer_new_undirected() {
        let analyzer = StructuralAnalyzer::new(false);
        assert!(!analyzer.directed);
    }

    #[test]
    fn test_analyze_empty_graph() {
        let analyzer = StructuralAnalyzer::new(true);
        let graph = DependencyGraph::new();

        let metrics = analyzer.analyze(&graph);

        assert!((metrics.density - 0.0).abs() < 0.001);
        assert!((metrics.average_degree - 0.0).abs() < 0.001);
    }

    // =========================================================================
    // compute_reciprocity Tests
    // =========================================================================

    use crate::graph::types::{EdgeData, NodeData, Visibility};
    use std::path::PathBuf;

    fn make_test_node(name: &str) -> NodeData {
        NodeData {
            path: PathBuf::from(format!("{}.rs", name)),
            module: name.to_string(),
            symbols: vec![],
            loc: 10,
            complexity: 1.0,
            ast_hash: 0,
        }
    }

    fn make_test_edge() -> EdgeData {
        EdgeData::Import {
            weight: 1.0,
            visibility: Visibility::Public,
        }
    }

    #[test]
    fn test_compute_reciprocity_empty_graph() {
        let graph = DependencyGraph::new();
        let result = compute_reciprocity(&graph);
        assert!((result - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_compute_reciprocity_no_reciprocal_edges() {
        let mut graph = DependencyGraph::new();
        let a = graph.add_node(make_test_node("a"));
        let b = graph.add_node(make_test_node("b"));
        graph.add_edge(a, b, make_test_edge());

        let result = compute_reciprocity(&graph);
        assert!((result - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_compute_reciprocity_all_reciprocal() {
        let mut graph = DependencyGraph::new();
        let a = graph.add_node(make_test_node("a"));
        let b = graph.add_node(make_test_node("b"));
        graph.add_edge(a, b, make_test_edge());
        graph.add_edge(b, a, make_test_edge());

        let result = compute_reciprocity(&graph);
        // Both edges have reciprocal, so 2/2 = 1.0
        assert!((result - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_compute_reciprocity_partial() {
        let mut graph = DependencyGraph::new();
        let a = graph.add_node(make_test_node("a"));
        let b = graph.add_node(make_test_node("b"));
        let c = graph.add_node(make_test_node("c"));
        graph.add_edge(a, b, make_test_edge());
        graph.add_edge(b, a, make_test_edge());
        graph.add_edge(b, c, make_test_edge());

        let result = compute_reciprocity(&graph);
        // 2 out of 3 edges have reciprocal
        assert!(result > 0.0);
        assert!(result < 1.0);
    }
}
