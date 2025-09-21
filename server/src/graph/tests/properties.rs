// Property-based tests for graph algorithms
// Testing invariants with proptest

#[cfg(test)]
mod tests {
    use super::super::super::*;
    use crate::graph::symbol_table::{SymbolTable, SymbolEntry};
    use petgraph::graph::DiGraph;
    use proptest::prelude::*;

    /// Strategy for generating random graphs
    fn arbitrary_graph() -> impl Strategy<Value = DependencyGraph> {
        (1..20usize, 0..100usize).prop_flat_map(|(nodes, edges)| {
            prop::collection::vec(any::<bool>(), edges)
                .prop_map(move |edge_mask| {
                    let mut graph = DiGraph::new();

                    // Add nodes
                    let node_indices: Vec<_> = (0..nodes)
                        .map(|i| graph.add_node(NodeData::test_node(i)))
                        .collect();

                    // Add edges based on mask
                    for (i, &should_add) in edge_mask.iter().enumerate().take(edges) {
                        if should_add && !node_indices.is_empty() {
                            let from = node_indices[i % node_indices.len()];
                            let to = node_indices[(i + 1) % node_indices.len()];
                            graph.add_edge(from, to, EdgeData::test_edge(1.0));
                        }
                    }

                    graph
                })
        })
    }

    proptest! {
        #[test]
        fn test_pagerank_sum_invariant(graph in arbitrary_graph()) {
            prop_assume!(graph.node_count() > 0);

            let matrices = GraphMatrices::from(&graph);
            let pr = PageRankComputer::default();
            let ranks = pr.compute(&matrices);

            // PageRank must sum to 1.0
            let sum: f64 = ranks.iter().sum();
            prop_assert!((sum - 1.0).abs() < 1e-5,
                "PageRank sum = {}, expected 1.0", sum);
        }

        #[test]
        fn test_pagerank_non_negative(graph in arbitrary_graph()) {
            prop_assume!(graph.node_count() > 0);

            let matrices = GraphMatrices::from(&graph);
            let pr = PageRankComputer::default();
            let ranks = pr.compute(&matrices);

            // All PageRank values must be non-negative
            for rank in ranks {
                prop_assert!(rank >= 0.0,
                    "PageRank value {} should be non-negative", rank);
            }
        }

        #[test]
        fn test_pagerank_damping_range(
            graph in arbitrary_graph(),
            damping in 0.0..1.0
        ) {
            prop_assume!(graph.node_count() > 0);

            let matrices = GraphMatrices::from(&graph);
            let pr = PageRankComputer::new().with_damping(damping);
            let ranks = pr.compute(&matrices);

            // Should work with any valid damping factor
            let sum: f64 = ranks.iter().sum();
            prop_assert!((sum - 1.0).abs() < 1e-5);
        }

        #[test]
        fn test_matrix_conversion_dimensions(graph in arbitrary_graph()) {
            let n = graph.node_count();
            let matrices = GraphMatrices::from(&graph);

            // All matrices should have correct dimensions
            prop_assert_eq!(matrices.adjacency.nrows(), n);
            prop_assert_eq!(matrices.adjacency.ncols(), n);
            prop_assert_eq!(matrices.transition.nrows(), n);
            prop_assert_eq!(matrices.transition.ncols(), n);
            prop_assert_eq!(matrices.laplacian.nrows(), n);
            prop_assert_eq!(matrices.laplacian.ncols(), n);
            prop_assert_eq!(matrices.out_degrees.len(), n);
            prop_assert_eq!(matrices.node_count, n);
        }

        #[test]
        fn test_symbol_table_consistency(
            symbols in prop::collection::vec(
                "[a-z]+",
                0..50
            )
        ) {
            let mut table = SymbolTable::new();

            for (i, name) in symbols.iter().enumerate() {
                let entry = SymbolEntry {
                    symbol: Symbol {
                        name: name.clone(),
                        kind: SymbolKind::Function,
                        visibility: Visibility::Public,
                        line: i,
                    },
                    file_path: format!("file_{}.rs", i).into(),
                    module_path: format!("mod_{}", i),
                    usage_count: 0,
                    is_exported: true,
                };

                table.insert(name.clone(), entry);
            }

            // Table size should match unique symbols
            let unique_symbols: std::collections::HashSet<_> =
                symbols.iter().collect();
            prop_assert_eq!(table.len(), unique_symbols.len());
        }

        #[test]
        fn test_graph_construction_determinism(
            node_count in 1..20usize
        ) {
            // Test that graph construction is deterministic
            let mut graph1: DependencyGraph = DiGraph::new();
            let mut graph2: DependencyGraph = DiGraph::new();

            // Add same nodes to both graphs
            for i in 0..node_count {
                graph1.add_node(NodeData::test_node(i));
                graph2.add_node(NodeData::test_node(i));
            }

            // Both graphs should be identical
            prop_assert_eq!(graph1.node_count(), graph2.node_count());
            prop_assert_eq!(graph1.node_count(), node_count);
        }
    }
}