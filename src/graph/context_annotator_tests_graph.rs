// ============================================================
// Convert to Undirected Tests
// ============================================================

#[test]
fn test_convert_to_undirected_empty() {
    let annotator = GraphContextAnnotator::new();
    let graph = DependencyGraph::new();

    let undirected = annotator.convert_to_undirected(&graph);
    assert_eq!(undirected.node_count(), 0);
    assert_eq!(undirected.edge_count(), 0);
}

#[test]
fn test_convert_to_undirected_nodes_preserved() {
    let annotator = GraphContextAnnotator::new();
    let graph = create_connected_graph();

    let undirected = annotator.convert_to_undirected(&graph);

    // Node count should be preserved
    assert_eq!(undirected.node_count(), graph.node_count());
}

#[test]
fn test_convert_to_undirected_edges_combined() {
    let annotator = GraphContextAnnotator::new();
    let graph = create_connected_graph();

    let undirected = annotator.convert_to_undirected(&graph);

    // Undirected graph should have edges
    assert!(undirected.edge_count() > 0);
}

// ============================================================
// Find Related Files Tests
// ============================================================

#[test]
fn test_find_related_files_isolated_node() {
    let annotator = GraphContextAnnotator::new();
    let mut graph = DependencyGraph::new();
    let node = graph.add_node(create_test_node("isolated.rs", 5.0));

    let related = annotator.find_related_files(&graph, node);
    assert!(related.is_empty());
}

#[test]
fn test_find_related_files_with_outgoing() {
    let annotator = GraphContextAnnotator::new();
    let mut graph = DependencyGraph::new();

    let n1 = graph.add_node(create_test_node("caller.rs", 5.0));
    let n2 = graph.add_node(create_test_node("callee.rs", 3.0));

    graph.add_edge(
        n1,
        n2,
        EdgeData::Import {
            weight: 1.0,
            visibility: Visibility::Public,
        },
    );

    let related = annotator.find_related_files(&graph, n1);
    assert_eq!(related.len(), 1);
    assert!(related[0].contains("callee.rs"));
}

#[test]
fn test_find_related_files_with_incoming() {
    let annotator = GraphContextAnnotator::new();
    let mut graph = DependencyGraph::new();

    let n1 = graph.add_node(create_test_node("caller.rs", 5.0));
    let n2 = graph.add_node(create_test_node("callee.rs", 3.0));

    graph.add_edge(
        n1,
        n2,
        EdgeData::Import {
            weight: 1.0,
            visibility: Visibility::Public,
        },
    );

    let related = annotator.find_related_files(&graph, n2);
    assert_eq!(related.len(), 1);
    assert!(related[0].contains("caller.rs"));
}

#[test]
fn test_find_related_files_bidirectional() {
    let annotator = GraphContextAnnotator::new();
    let mut graph = DependencyGraph::new();

    let n1 = graph.add_node(create_test_node("a.rs", 5.0));
    let n2 = graph.add_node(create_test_node("b.rs", 3.0));
    let n3 = graph.add_node(create_test_node("c.rs", 4.0));

    // n2 depends on n1 and n3
    graph.add_edge(
        n1,
        n2,
        EdgeData::Import {
            weight: 1.0,
            visibility: Visibility::Public,
        },
    );
    graph.add_edge(
        n2,
        n3,
        EdgeData::Import {
            weight: 1.0,
            visibility: Visibility::Public,
        },
    );

    let related = annotator.find_related_files(&graph, n2);
    assert_eq!(related.len(), 2);
}

#[test]
fn test_find_related_files_deduplicates() {
    let annotator = GraphContextAnnotator::new();
    let mut graph = DependencyGraph::new();

    let n1 = graph.add_node(create_test_node("a.rs", 5.0));
    let n2 = graph.add_node(create_test_node("b.rs", 3.0));

    // Add multiple edges between same nodes
    graph.add_edge(
        n1,
        n2,
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
            async_call: false,
        },
    );
    graph.add_edge(
        n2,
        n1,
        EdgeData::DataFlow {
            confidence: 0.9,
            direction: FlowDirection::Forward,
        },
    );

    let related = annotator.find_related_files(&graph, n1);
    // Should be deduplicated and sorted
    assert!(related.len() <= 2);
}

// ============================================================
// Multi-Edge Type Graph Tests
// ============================================================

#[test]
fn test_annotate_context_multi_edge_types() {
    let annotator = GraphContextAnnotator::new();
    let graph = create_multi_edge_graph();

    let annotations = annotator.annotate_context(&graph);

    // Should have 3 annotations
    assert_eq!(annotations.len(), 3);

    // All should have valid importance scores
    for annotation in &annotations {
        assert!(annotation.importance_score >= 0.0);
    }
}

// ============================================================
// Property-Based Tests
// ============================================================

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_classify_complexity_always_returns_valid_rank(complexity in 0.0f64..1000.0) {
            let annotator = GraphContextAnnotator::new();
            let rank = annotator.classify_complexity(complexity);
            prop_assert!(
                rank == "Low" || rank == "Medium" || rank == "High" || rank == "Very High"
            );
        }

        #[test]
        fn test_pagerank_threshold_filters_correctly(
            threshold in 0.0f64..1.0,
            score in 0.0f64..1.0
        ) {
            let annotator = GraphContextAnnotator {
                pagerank_threshold: threshold,
                community_relevance: 0.8,
            };

            let annotations = vec![ContextAnnotation {
                file_path: "test.rs".to_string(),
                importance_score: score,
                community_id: 0,
                related_files: vec![],
                complexity_rank: "Low".to_string(),
            }];

            let high_importance = annotator.get_high_importance_files(&annotations);

            if score > threshold {
                prop_assert_eq!(high_importance.len(), 1);
            } else {
                prop_assert!(high_importance.is_empty());
            }
        }

        #[test]
        fn test_community_clusters_preserve_count(
            community_count in 1usize..10,
            files_per_community in 1usize..5
        ) {
            let annotator = GraphContextAnnotator::new();

            let mut annotations = Vec::new();
            for c in 0..community_count {
                for f in 0..files_per_community {
                    annotations.push(ContextAnnotation {
                        file_path: format!("file_{}_{}.rs", c, f),
                        importance_score: 0.5,
                        community_id: c,
                        related_files: vec![],
                        complexity_rank: "Low".to_string(),
                    });
                }
            }

            let clusters = annotator.get_community_clusters(&annotations);

            prop_assert_eq!(clusters.len(), community_count);
            for files in clusters.values() {
                prop_assert_eq!(files.len(), files_per_community);
            }
        }

        #[test]
        fn test_complexity_boundaries(complexity in prop::num::f64::ANY) {
            if complexity.is_nan() || complexity.is_infinite() {
                return Ok(()); // Skip NaN and infinite values
            }

            let annotator = GraphContextAnnotator::new();
            let rank = annotator.classify_complexity(complexity);

            if complexity < 5.0 {
                prop_assert_eq!(rank, "Low");
            } else if complexity < 10.0 {
                prop_assert_eq!(rank, "Medium");
            } else if complexity < 20.0 {
                prop_assert_eq!(rank, "High");
            } else {
                prop_assert_eq!(rank, "Very High");
            }
        }
    }
}

// ============================================================
// Edge Case Tests
// ============================================================

#[test]
fn test_annotate_context_handles_nan_importance() {
    let annotator = GraphContextAnnotator::new();
    let mut graph = DependencyGraph::new();

    // Add nodes that might produce NaN in PageRank (e.g., disconnected)
    graph.add_node(create_test_node("a.rs", 5.0));
    graph.add_node(create_test_node("b.rs", 5.0));

    // No edges - disconnected nodes
    let annotations = annotator.annotate_context(&graph);

    // Should still produce valid annotations
    assert_eq!(annotations.len(), 2);
}

#[test]
fn test_annotate_context_large_complexity_values() {
    let annotator = GraphContextAnnotator::new();
    let mut graph = DependencyGraph::new();

    graph.add_node(create_test_node("huge.rs", 1000.0));

    let annotations = annotator.annotate_context(&graph);

    assert_eq!(annotations.len(), 1);
    assert_eq!(annotations[0].complexity_rank, "Very High");
}

#[test]
fn test_annotate_context_zero_complexity() {
    let annotator = GraphContextAnnotator::new();
    let mut graph = DependencyGraph::new();

    graph.add_node(create_test_node("empty.rs", 0.0));

    let annotations = annotator.annotate_context(&graph);

    assert_eq!(annotations.len(), 1);
    assert_eq!(annotations[0].complexity_rank, "Low");
}
