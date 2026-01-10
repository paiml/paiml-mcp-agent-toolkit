// Deep context annotation with graph metrics
// Sprint 3: Integration with graph analysis
// Complexity: All functions ≤ 8

use super::*;
use petgraph::visit::EdgeRef;
use std::collections::HashMap;

/// Annotates code with graph-derived context information
#[derive(Debug, Clone)]
pub struct GraphContextAnnotator {
    pub pagerank_threshold: f64,
    pub community_relevance: f64,
}

impl Default for GraphContextAnnotator {
    fn default() -> Self {
        GraphContextAnnotator {
            pagerank_threshold: 0.1,
            community_relevance: 0.8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ContextAnnotation {
    pub file_path: String,
    pub importance_score: f64,
    pub community_id: usize,
    pub related_files: Vec<String>,
    pub complexity_rank: String,
}

impl GraphContextAnnotator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Annotate files with graph-derived context
    /// Complexity: 8 (PageRank + community detection + ranking)
    pub fn annotate_context(&self, graph: &DependencyGraph) -> Vec<ContextAnnotation> {
        if graph.node_count() == 0 {
            return Vec::new();
        }

        // Convert to undirected for community detection
        let undirected = self.convert_to_undirected(graph);

        // Calculate PageRank for importance using aprender
        let pagerank = PageRankComputer::default();
        let importance_scores = pagerank.compute(graph);

        // Detect communities
        let mut community_detector = LouvainDetector::default();
        let communities = community_detector.detect_communities(&undirected);

        // Generate annotations
        let mut annotations = Vec::new();

        for (idx, node_weight) in graph.node_indices().enumerate() {
            if let Some(node_data) = graph.node_weight(node_weight) {
                let importance = importance_scores.get(idx).unwrap_or(&0.0);
                let community = communities.get(idx).unwrap_or(&0);

                let annotation = ContextAnnotation {
                    file_path: node_data.path.to_string_lossy().to_string(),
                    importance_score: *importance,
                    community_id: *community,
                    related_files: self.find_related_files(graph, node_weight),
                    complexity_rank: self.classify_complexity(node_data.complexity),
                };

                annotations.push(annotation);
            }
        }

        // Sort by importance (NaN values sorted last)
        annotations.sort_by(|a, b| {
            b.importance_score
                .partial_cmp(&a.importance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        annotations
    }

    /// Convert directed graph to undirected for community detection
    /// Complexity: 5
    fn convert_to_undirected(&self, graph: &DependencyGraph) -> UndirectedGraph {
        let mut undirected = petgraph::Graph::new_undirected();
        let mut node_map = HashMap::new();

        // Add nodes
        for node_idx in graph.node_indices() {
            if let Some(node_data) = graph.node_weight(node_idx) {
                let new_node = undirected.add_node(node_data.clone());
                node_map.insert(node_idx, new_node);
            }
        }

        // Add edges (combine bidirectional edges)
        for edge in graph.edge_references() {
            if let (Some(&source), Some(&target)) =
                (node_map.get(&edge.source()), node_map.get(&edge.target()))
            {
                let weight = edge.weight().to_numeric_weight();
                undirected.add_edge(source, target, weight);
            }
        }

        undirected
    }

    /// Find related files through graph connections
    /// Complexity: 6
    fn find_related_files(
        &self,
        graph: &DependencyGraph,
        node: petgraph::graph::NodeIndex,
    ) -> Vec<String> {
        let mut related = Vec::new();

        // Get neighbors (both incoming and outgoing)
        for edge in graph.edges(node) {
            if let Some(neighbor_data) = graph.node_weight(edge.target()) {
                related.push(neighbor_data.path.to_string_lossy().to_string());
            }
        }

        // Get reverse neighbors
        for edge in graph.edges_directed(node, petgraph::Direction::Incoming) {
            if let Some(neighbor_data) = graph.node_weight(edge.source()) {
                related.push(neighbor_data.path.to_string_lossy().to_string());
            }
        }

        related.sort();
        related.dedup();
        related
    }

    /// Classify complexity into readable categories
    /// Complexity: 3
    fn classify_complexity(&self, complexity: f64) -> String {
        match complexity {
            c if c < 5.0 => "Low".to_string(),
            c if c < 10.0 => "Medium".to_string(),
            c if c < 20.0 => "High".to_string(),
            _ => "Very High".to_string(),
        }
    }

    /// Get high-importance files for focused analysis
    /// Complexity: 4
    pub fn get_high_importance_files(&self, annotations: &[ContextAnnotation]) -> Vec<String> {
        annotations
            .iter()
            .filter(|a| a.importance_score > self.pagerank_threshold)
            .map(|a| a.file_path.clone())
            .collect()
    }

    /// Get community clusters for analysis grouping
    /// Complexity: 5
    pub fn get_community_clusters(
        &self,
        annotations: &[ContextAnnotation],
    ) -> HashMap<usize, Vec<String>> {
        let mut clusters = HashMap::new();

        for annotation in annotations {
            clusters
                .entry(annotation.community_id)
                .or_insert_with(Vec::new)
                .push(annotation.file_path.clone());
        }

        clusters
    }
}

#[cfg(test)]
mod coverage_tests {
    use super::*;
    use std::path::PathBuf;

    // ============================================================
    // Test Fixtures
    // ============================================================

    /// Create a minimal test node
    fn create_test_node(path: &str, complexity: f64) -> NodeData {
        NodeData {
            path: PathBuf::from(path),
            module: path.to_string(),
            symbols: vec![],
            loc: 100,
            complexity,
            ast_hash: 0,
        }
    }

    /// Create a simple graph with connected nodes
    fn create_connected_graph() -> DependencyGraph {
        let mut graph = DependencyGraph::new();

        let n1 = graph.add_node(create_test_node("src/main.rs", 5.0));
        let n2 = graph.add_node(create_test_node("src/lib.rs", 12.0));
        let n3 = graph.add_node(create_test_node("src/utils.rs", 3.0));
        let n4 = graph.add_node(create_test_node("src/complex.rs", 25.0));

        // Create import edges
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
            n3,
            EdgeData::Import {
                weight: 0.5,
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
        graph.add_edge(
            n4,
            n2,
            EdgeData::Import {
                weight: 2.0,
                visibility: Visibility::Public,
            },
        );

        graph
    }

    /// Create a graph with multiple edge types
    fn create_multi_edge_graph() -> DependencyGraph {
        let mut graph = DependencyGraph::new();

        let n1 = graph.add_node(create_test_node("src/caller.rs", 8.0));
        let n2 = graph.add_node(create_test_node("src/callee.rs", 6.0));
        let n3 = graph.add_node(create_test_node("src/base.rs", 4.0));

        // Import edge
        graph.add_edge(
            n1,
            n2,
            EdgeData::Import {
                weight: 1.5,
                visibility: Visibility::Public,
            },
        );

        // Function call edge
        graph.add_edge(
            n1,
            n2,
            EdgeData::FunctionCall {
                count: 5,
                async_call: true,
            },
        );

        // Type dependency edge
        graph.add_edge(
            n2,
            n3,
            EdgeData::TypeDependency {
                strength: 0.8,
                kind: TypeKind::Struct,
            },
        );

        // Inheritance edge
        graph.add_edge(n1, n3, EdgeData::Inheritance { depth: 2 });

        // DataFlow edge
        graph.add_edge(
            n2,
            n1,
            EdgeData::DataFlow {
                confidence: 0.9,
                direction: FlowDirection::Backward,
            },
        );

        graph
    }

    // ============================================================
    // GraphContextAnnotator Tests
    // ============================================================

    #[test]
    fn test_annotator_default_values() {
        let annotator = GraphContextAnnotator::default();
        assert!((annotator.pagerank_threshold - 0.1).abs() < f64::EPSILON);
        assert!((annotator.community_relevance - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn test_annotator_new_equals_default() {
        let new = GraphContextAnnotator::new();
        let default = GraphContextAnnotator::default();

        assert!((new.pagerank_threshold - default.pagerank_threshold).abs() < f64::EPSILON);
        assert!((new.community_relevance - default.community_relevance).abs() < f64::EPSILON);
    }

    #[test]
    fn test_annotate_context_empty_graph() {
        let annotator = GraphContextAnnotator::new();
        let graph = DependencyGraph::new();

        let annotations = annotator.annotate_context(&graph);
        assert!(annotations.is_empty());
    }

    #[test]
    fn test_annotate_context_single_node() {
        let annotator = GraphContextAnnotator::new();
        let mut graph = DependencyGraph::new();
        graph.add_node(create_test_node("src/only.rs", 7.5));

        let annotations = annotator.annotate_context(&graph);
        assert_eq!(annotations.len(), 1);
        assert!(annotations[0].file_path.contains("only.rs"));
        assert_eq!(annotations[0].complexity_rank, "Medium");
    }

    #[test]
    fn test_annotate_context_connected_graph() {
        let annotator = GraphContextAnnotator::new();
        let graph = create_connected_graph();

        let annotations = annotator.annotate_context(&graph);

        // Should have 4 annotations
        assert_eq!(annotations.len(), 4);

        // Annotations should be sorted by importance (descending)
        for i in 1..annotations.len() {
            assert!(
                annotations[i - 1].importance_score >= annotations[i].importance_score,
                "Annotations should be sorted by importance descending"
            );
        }
    }

    #[test]
    fn test_annotate_context_has_related_files() {
        let annotator = GraphContextAnnotator::new();
        let graph = create_connected_graph();

        let annotations = annotator.annotate_context(&graph);

        // Main.rs should have related files (it has outgoing edges to lib.rs and utils.rs)
        let main_annotation = annotations.iter().find(|a| a.file_path.contains("main.rs"));
        assert!(main_annotation.is_some());

        let related = &main_annotation.unwrap().related_files;
        assert!(!related.is_empty(), "main.rs should have related files");
    }

    #[test]
    fn test_annotate_context_community_assignment() {
        let annotator = GraphContextAnnotator::new();
        let graph = create_connected_graph();

        let annotations = annotator.annotate_context(&graph);

        // All nodes should have a community ID
        for annotation in &annotations {
            // community_id is usize, so it's always >= 0
            // Just verify the field exists and is accessible
            let _ = annotation.community_id;
        }
    }

    // ============================================================
    // Complexity Classification Tests
    // ============================================================

    #[test]
    fn test_classify_complexity_low() {
        let annotator = GraphContextAnnotator::new();
        assert_eq!(annotator.classify_complexity(0.0), "Low");
        assert_eq!(annotator.classify_complexity(4.9), "Low");
        assert_eq!(annotator.classify_complexity(1.0), "Low");
    }

    #[test]
    fn test_classify_complexity_medium() {
        let annotator = GraphContextAnnotator::new();
        assert_eq!(annotator.classify_complexity(5.0), "Medium");
        assert_eq!(annotator.classify_complexity(7.5), "Medium");
        assert_eq!(annotator.classify_complexity(9.9), "Medium");
    }

    #[test]
    fn test_classify_complexity_high() {
        let annotator = GraphContextAnnotator::new();
        assert_eq!(annotator.classify_complexity(10.0), "High");
        assert_eq!(annotator.classify_complexity(15.0), "High");
        assert_eq!(annotator.classify_complexity(19.9), "High");
    }

    #[test]
    fn test_classify_complexity_very_high() {
        let annotator = GraphContextAnnotator::new();
        assert_eq!(annotator.classify_complexity(20.0), "Very High");
        assert_eq!(annotator.classify_complexity(50.0), "Very High");
        assert_eq!(annotator.classify_complexity(100.0), "Very High");
    }

    // ============================================================
    // High Importance Files Tests
    // ============================================================

    #[test]
    fn test_get_high_importance_files_empty() {
        let annotator = GraphContextAnnotator::new();
        let annotations: Vec<ContextAnnotation> = vec![];

        let high_importance = annotator.get_high_importance_files(&annotations);
        assert!(high_importance.is_empty());
    }

    #[test]
    fn test_get_high_importance_files_all_below_threshold() {
        let annotator = GraphContextAnnotator::new(); // threshold = 0.1

        let annotations = vec![
            ContextAnnotation {
                file_path: "low1.rs".to_string(),
                importance_score: 0.05,
                community_id: 0,
                related_files: vec![],
                complexity_rank: "Low".to_string(),
            },
            ContextAnnotation {
                file_path: "low2.rs".to_string(),
                importance_score: 0.09,
                community_id: 0,
                related_files: vec![],
                complexity_rank: "Low".to_string(),
            },
        ];

        let high_importance = annotator.get_high_importance_files(&annotations);
        assert!(high_importance.is_empty());
    }

    #[test]
    fn test_get_high_importance_files_some_above_threshold() {
        let annotator = GraphContextAnnotator::new(); // threshold = 0.1

        let annotations = vec![
            ContextAnnotation {
                file_path: "high.rs".to_string(),
                importance_score: 0.5,
                community_id: 0,
                related_files: vec![],
                complexity_rank: "High".to_string(),
            },
            ContextAnnotation {
                file_path: "low.rs".to_string(),
                importance_score: 0.05,
                community_id: 0,
                related_files: vec![],
                complexity_rank: "Low".to_string(),
            },
            ContextAnnotation {
                file_path: "medium.rs".to_string(),
                importance_score: 0.2,
                community_id: 0,
                related_files: vec![],
                complexity_rank: "Medium".to_string(),
            },
        ];

        let high_importance = annotator.get_high_importance_files(&annotations);
        assert_eq!(high_importance.len(), 2);
        assert!(high_importance.contains(&"high.rs".to_string()));
        assert!(high_importance.contains(&"medium.rs".to_string()));
        assert!(!high_importance.contains(&"low.rs".to_string()));
    }

    #[test]
    fn test_get_high_importance_files_at_threshold() {
        let annotator = GraphContextAnnotator::new(); // threshold = 0.1

        let annotations = vec![ContextAnnotation {
            file_path: "exactly_at.rs".to_string(),
            importance_score: 0.1, // exactly at threshold (not > threshold)
            community_id: 0,
            related_files: vec![],
            complexity_rank: "Low".to_string(),
        }];

        let high_importance = annotator.get_high_importance_files(&annotations);
        // 0.1 is NOT > 0.1, so it should be empty
        assert!(high_importance.is_empty());
    }

    // ============================================================
    // Community Clusters Tests
    // ============================================================

    #[test]
    fn test_get_community_clusters_empty() {
        let annotator = GraphContextAnnotator::new();
        let annotations: Vec<ContextAnnotation> = vec![];

        let clusters = annotator.get_community_clusters(&annotations);
        assert!(clusters.is_empty());
    }

    #[test]
    fn test_get_community_clusters_single_community() {
        let annotator = GraphContextAnnotator::new();

        let annotations = vec![
            ContextAnnotation {
                file_path: "a.rs".to_string(),
                importance_score: 0.5,
                community_id: 0,
                related_files: vec![],
                complexity_rank: "Low".to_string(),
            },
            ContextAnnotation {
                file_path: "b.rs".to_string(),
                importance_score: 0.3,
                community_id: 0,
                related_files: vec![],
                complexity_rank: "Low".to_string(),
            },
            ContextAnnotation {
                file_path: "c.rs".to_string(),
                importance_score: 0.2,
                community_id: 0,
                related_files: vec![],
                complexity_rank: "Low".to_string(),
            },
        ];

        let clusters = annotator.get_community_clusters(&annotations);
        assert_eq!(clusters.len(), 1);
        assert!(clusters.contains_key(&0));
        assert_eq!(clusters.get(&0).unwrap().len(), 3);
    }

    #[test]
    fn test_get_community_clusters_multiple_communities() {
        let annotator = GraphContextAnnotator::new();

        let annotations = vec![
            ContextAnnotation {
                file_path: "a1.rs".to_string(),
                importance_score: 0.5,
                community_id: 0,
                related_files: vec![],
                complexity_rank: "Low".to_string(),
            },
            ContextAnnotation {
                file_path: "a2.rs".to_string(),
                importance_score: 0.3,
                community_id: 0,
                related_files: vec![],
                complexity_rank: "Low".to_string(),
            },
            ContextAnnotation {
                file_path: "b1.rs".to_string(),
                importance_score: 0.4,
                community_id: 1,
                related_files: vec![],
                complexity_rank: "Medium".to_string(),
            },
            ContextAnnotation {
                file_path: "c1.rs".to_string(),
                importance_score: 0.2,
                community_id: 2,
                related_files: vec![],
                complexity_rank: "High".to_string(),
            },
        ];

        let clusters = annotator.get_community_clusters(&annotations);
        assert_eq!(clusters.len(), 3);
        assert_eq!(clusters.get(&0).unwrap().len(), 2);
        assert_eq!(clusters.get(&1).unwrap().len(), 1);
        assert_eq!(clusters.get(&2).unwrap().len(), 1);
    }

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
    // ContextAnnotation Struct Tests
    // ============================================================

    #[test]
    fn test_context_annotation_clone() {
        let annotation = ContextAnnotation {
            file_path: "test.rs".to_string(),
            importance_score: 0.5,
            community_id: 1,
            related_files: vec!["related.rs".to_string()],
            complexity_rank: "Medium".to_string(),
        };

        let cloned = annotation.clone();
        assert_eq!(cloned.file_path, annotation.file_path);
        assert!((cloned.importance_score - annotation.importance_score).abs() < f64::EPSILON);
        assert_eq!(cloned.community_id, annotation.community_id);
        assert_eq!(cloned.related_files, annotation.related_files);
        assert_eq!(cloned.complexity_rank, annotation.complexity_rank);
    }

    #[test]
    fn test_context_annotation_debug() {
        let annotation = ContextAnnotation {
            file_path: "test.rs".to_string(),
            importance_score: 0.5,
            community_id: 1,
            related_files: vec![],
            complexity_rank: "Low".to_string(),
        };

        let debug = format!("{:?}", annotation);
        assert!(debug.contains("test.rs"));
        assert!(debug.contains("0.5"));
    }

    // ============================================================
    // GraphContextAnnotator Clone Tests
    // ============================================================

    #[test]
    fn test_annotator_clone() {
        let annotator = GraphContextAnnotator {
            pagerank_threshold: 0.2,
            community_relevance: 0.9,
        };

        let cloned = annotator.clone();
        assert!((cloned.pagerank_threshold - 0.2).abs() < f64::EPSILON);
        assert!((cloned.community_relevance - 0.9).abs() < f64::EPSILON);
    }

    #[test]
    fn test_annotator_debug() {
        let annotator = GraphContextAnnotator::new();
        let debug = format!("{:?}", annotator);
        assert!(debug.contains("GraphContextAnnotator"));
        assert!(debug.contains("pagerank_threshold"));
    }

    // ============================================================
    // Property-Based Tests
    // ============================================================

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
                for (_, files) in &clusters {
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
}
