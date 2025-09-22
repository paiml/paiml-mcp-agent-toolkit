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

        // Calculate PageRank for importance
        let matrices = GraphMatrices::from(graph);
        let pagerank = PageRankComputer::default();
        let importance_scores = pagerank.compute(&matrices);

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

        // Sort by importance
        annotations.sort_by(|a, b| b.importance_score.partial_cmp(&a.importance_score).unwrap());

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
