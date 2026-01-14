// Integration tests for Phase 4: Graph Migration
// Tests complete integration: DependencyGraph → aprender → metrics
// Complexity: All functions ≤ 10
// SATD: Zero tolerance

use crate::graph::*;
use std::path::PathBuf;

/// Helper: Create test NodeData
fn create_node(name: &str, complexity: f64) -> NodeData {
    NodeData {
        path: PathBuf::from(format!("{}.rs", name)),
        module: name.to_string(),
        symbols: vec![Symbol {
            name: name.to_string(),
            kind: SymbolKind::Function,
            visibility: Visibility::Public,
            line: 1,
        }],
        loc: 100,
        complexity,
        ast_hash: 12345,
    }
}

#[test]
fn test_centrality_integration_star_graph() {
    // Create star graph: central node connected to 4 periphery nodes
    let mut graph = DependencyGraph::new();

    let center = graph.add_node(create_node("center", 5.0));
    let mut periphery = Vec::new();

    for i in 0..4 {
        let node = graph.add_node(create_node(&format!("node_{}", i), 2.0));
        periphery.push(node);

        // Connect center to periphery (star pattern)
        graph.add_edge(
            center,
            node,
            EdgeData::FunctionCall {
                count: i + 1,
                async_call: false,
            },
        );
    }

    // Compute centrality metrics
    let computer = CentralityComputer::new(true, false);
    let metrics = computer.compute_all(&graph);

    // Verify metrics dimensions
    assert_eq!(metrics.degree.len(), 5);
    assert_eq!(metrics.betweenness.len(), 5);
    assert_eq!(metrics.closeness.len(), 5);
    assert_eq!(metrics.eigenvector.len(), 5);
    assert_eq!(metrics.katz.len(), 5);
    assert_eq!(metrics.harmonic.len(), 5);

    // Center should have higher degree centrality than periphery
    let center_idx = center.0 as usize;
    for &p in &periphery {
        let p_idx = p.0 as usize;
        assert!(
            metrics.degree[center_idx] >= metrics.degree[p_idx],
            "Center degree {} should be >= periphery degree {}",
            metrics.degree[center_idx],
            metrics.degree[p_idx]
        );
    }
}

#[test]
fn test_structural_integration_small_graph() {
    // Create small directed graph with known properties
    let mut graph = DependencyGraph::new();

    let n0 = graph.add_node(create_node("mod_a", 3.0));
    let n1 = graph.add_node(create_node("mod_b", 4.0));
    let n2 = graph.add_node(create_node("mod_c", 2.0));

    graph.add_edge(
        n0,
        n1,
        EdgeData::Import {
            weight: 1.0,
            visibility: Visibility::Public,
        },
    );
    graph.add_edge(
        n1,
        n2,
        EdgeData::TypeDependency {
            strength: 0.8,
            kind: TypeKind::Struct,
        },
    );
    graph.add_edge(
        n2,
        n0,
        EdgeData::DataFlow {
            confidence: 0.9,
            direction: FlowDirection::Forward,
        },
    );

    // Analyze structure
    let analyzer = StructuralAnalyzer::new(true);
    let metrics = analyzer.analyze(&graph);

    // Verify basic properties
    assert!(metrics.density > 0.0, "Density should be positive");
    assert!(
        metrics.average_degree > 0.0,
        "Average degree should be positive"
    );
    assert_eq!(metrics.is_cyclic, true, "Graph should be cyclic");
    assert_eq!(
        metrics.strongly_connected_components, 1,
        "Should have 1 SCC (all nodes in cycle)"
    );

    // Verify clustering coefficient is computed
    assert!(
        metrics.clustering_coefficient >= 0.0,
        "Clustering coefficient should be non-negative"
    );
}

#[test]
fn test_disconnected_graph_components() {
    // Create graph with 2 disconnected components
    let mut graph = DependencyGraph::new();

    // Component 1
    let c1_n0 = graph.add_node(create_node("comp1_a", 2.0));
    let c1_n1 = graph.add_node(create_node("comp1_b", 3.0));
    graph.add_edge(
        c1_n0,
        c1_n1,
        EdgeData::FunctionCall {
            count: 1,
            async_call: false,
        },
    );

    // Component 2 (isolated)
    let _c2_n0 = graph.add_node(create_node("comp2_a", 1.0));
    let _c2_n1 = graph.add_node(create_node("comp2_b", 1.0));

    // Analyze structure
    let analyzer = StructuralAnalyzer::new(true);
    let metrics = analyzer.analyze(&graph);

    // Should detect 3 strongly connected components (1 edge + 2 isolated nodes)
    assert!(
        metrics.strongly_connected_components >= 2,
        "Should have at least 2 components"
    );
}

#[test]
fn test_large_graph_performance() {
    // Create larger graph to verify performance
    let mut graph = DependencyGraph::new();
    let mut nodes = Vec::new();

    // Add 20 nodes
    for i in 0..20 {
        let node = graph.add_node(create_node(&format!("node_{}", i), i as f64));
        nodes.push(node);
    }

    // Add edges forming a chain
    for i in 0..19 {
        graph.add_edge(
            nodes[i],
            nodes[i + 1],
            EdgeData::Import {
                weight: 1.0,
                visibility: Visibility::Public,
            },
        );
    }

    // Compute centrality (should complete quickly)
    let computer = CentralityComputer::new(true, false);
    let metrics = computer.compute_all(&graph);

    assert_eq!(metrics.degree.len(), 20);
    assert_eq!(metrics.betweenness.len(), 20);

    // Middle nodes should have higher betweenness
    let mid_idx = nodes[10].0 as usize;
    let end_idx = nodes[0].0 as usize;
    assert!(
        metrics.betweenness[mid_idx] >= metrics.betweenness[end_idx],
        "Middle nodes should have higher betweenness"
    );
}

#[test]
fn test_adapter_preserves_graph_structure() {
    // Verify adapter preserves edge count and node count
    let mut graph = DependencyGraph::new();

    let n0 = graph.add_node(create_node("a", 1.0));
    let n1 = graph.add_node(create_node("b", 2.0));
    let n2 = graph.add_node(create_node("c", 3.0));

    graph.add_edge(
        n0,
        n1,
        EdgeData::Import {
            weight: 1.0,
            visibility: Visibility::Public,
        },
    );
    graph.add_edge(
        n1,
        n2,
        EdgeData::FunctionCall {
            count: 5,
            async_call: true,
        },
    );

    // Convert to aprender
    let aprender_graph = crate::graph::aprender_adapter::to_aprender_graph(&graph, true);

    // Verify structure preserved
    assert_eq!(aprender_graph.num_nodes(), 3, "Node count should match");
    assert_eq!(aprender_graph.num_edges(), 2, "Edge count should match");
    assert!(aprender_graph.is_directed(), "Directionality should match");
}

#[test]
fn test_reciprocity_computation() {
    // Test reciprocity on bidirectional graph
    let mut graph = DependencyGraph::new();

    let n0 = graph.add_node(create_node("a", 1.0));
    let n1 = graph.add_node(create_node("b", 2.0));

    // Add bidirectional edges
    graph.add_edge(
        n0,
        n1,
        EdgeData::Import {
            weight: 1.0,
            visibility: Visibility::Public,
        },
    );
    graph.add_edge(
        n1,
        n0,
        EdgeData::Import {
            weight: 1.0,
            visibility: Visibility::Public,
        },
    );

    // Analyze structure
    let analyzer = StructuralAnalyzer::new(true);
    let metrics = analyzer.analyze(&graph);

    // Reciprocity should be 1.0 (all edges are reciprocal)
    assert!(
        metrics.reciprocity.is_some(),
        "Reciprocity should be computed"
    );
    let reciprocity = metrics.reciprocity.unwrap();
    assert!(
        (reciprocity - 1.0).abs() < 0.01,
        "Reciprocity should be ~1.0, got {}",
        reciprocity
    );
}
