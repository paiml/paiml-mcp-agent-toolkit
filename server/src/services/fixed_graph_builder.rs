use anyhow::Result;
use std::collections::{BTreeMap, HashMap, HashSet};

use crate::models::dag::{DependencyGraph, Edge, NodeInfo, NodeType};
use crate::services::semantic_naming::SemanticNamer;

/// Configuration for graph building
#[derive(Debug, Clone)]
pub struct GraphConfig {
    pub max_nodes: usize,
    pub max_edges: usize,
    pub grouping: GroupingStrategy,
}

impl Default for GraphConfig {
    fn default() -> Self {
        Self {
            max_nodes: 20,
            max_edges: 60,
            grouping: GroupingStrategy::Module,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GroupingStrategy {
    Module,
    Directory,
    None,
}

/// A fixed-size graph with semantic names
#[derive(Debug, Clone, PartialEq)]
pub struct FixedGraph {
    pub nodes: BTreeMap<String, FixedNode>,
    pub edges: Vec<Edge>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FixedNode {
    pub id: String,
    pub display_name: String,
    pub node_type: NodeType,
    pub complexity: u64,
}

/// Builds fixed-size graphs with PageRank-based node selection
pub struct FixedGraphBuilder {
    max_nodes: usize,
    max_edges: usize,
    namer: SemanticNamer,
}

impl FixedGraphBuilder {
    pub fn new(config: GraphConfig) -> Self {
        Self {
            max_nodes: config.max_nodes,
            max_edges: config.max_edges,
            namer: SemanticNamer::new(),
        }
    }

    pub fn with_max_nodes(mut self, max_nodes: usize) -> Self {
        self.max_nodes = max_nodes;
        self
    }

    pub fn with_max_edges(mut self, max_edges: usize) -> Self {
        self.max_edges = max_edges;
        self
    }

    /// Build a fixed-size graph from a dependency graph
    pub fn build(&self, graph: &DependencyGraph) -> Result<FixedGraph> {
        // 1. Group nodes by module
        let groups = self.group_by_module(graph);

        // 2. Calculate PageRank scores
        let scores = self.calculate_pagerank(graph, &groups);

        // 3. Select top N nodes
        let selected_nodes = self.select_top_nodes(scores, &groups);

        // 4. Build graph with edge budget
        self.build_with_budget(selected_nodes, graph)
    }

    /// Group nodes by module path
    fn group_by_module(&self, graph: &DependencyGraph) -> HashMap<String, Vec<String>> {
        let mut groups: HashMap<String, Vec<String>> = HashMap::new();

        for (node_id, node) in &graph.nodes {
            let module_name = self.get_module_name(node);
            groups.entry(module_name).or_default().push(node_id.clone());
        }

        groups
    }

    /// Get the module name for a node
    fn get_module_name(&self, node: &NodeInfo) -> String {
        // Extract module from file path
        if !node.file_path.is_empty() {
            let parts: Vec<&str> = node.file_path.split('/').collect();
            if parts.len() > 1 {
                // For src/services/foo.rs -> services
                // For src/models/bar.rs -> models
                if let Some(module) = parts.get(1) {
                    return module.to_string();
                }
            }
        }

        // Fallback to node type
        match node.node_type {
            NodeType::Module => "modules".to_string(),
            NodeType::Function => "functions".to_string(),
            NodeType::Class => "classes".to_string(),
            NodeType::Trait => "traits".to_string(),
            NodeType::Interface => "interfaces".to_string(),
        }
    }

    /// Calculate PageRank scores for nodes
    fn calculate_pagerank(
        &self,
        graph: &DependencyGraph,
        groups: &HashMap<String, Vec<String>>,
    ) -> HashMap<String, f64> {
        let damping_factor = 0.85;
        let iterations = 10;
        let num_nodes = graph.nodes.len() as f64;

        // Initialize scores
        let mut scores: HashMap<String, f64> = HashMap::new();
        for node_id in graph.nodes.keys() {
            scores.insert(node_id.clone(), 1.0 / num_nodes);
        }

        // Build adjacency lists
        let mut incoming: HashMap<String, Vec<String>> = HashMap::new();
        let mut outgoing_count: HashMap<String, usize> = HashMap::new();

        for edge in &graph.edges {
            incoming
                .entry(edge.to.clone())
                .or_default()
                .push(edge.from.clone());
            *outgoing_count.entry(edge.from.clone()).or_default() += 1;
        }

        // PageRank iterations
        for _ in 0..iterations {
            let mut new_scores = HashMap::new();

            for node_id in graph.nodes.keys() {
                let mut rank = (1.0 - damping_factor) / num_nodes;

                if let Some(incoming_nodes) = incoming.get(node_id) {
                    for incoming_node in incoming_nodes {
                        if let (Some(&score), Some(&count)) =
                            (scores.get(incoming_node), outgoing_count.get(incoming_node))
                        {
                            rank += damping_factor * score / count as f64;
                        }
                    }
                }

                new_scores.insert(node_id.clone(), rank);
            }

            scores = new_scores;
        }

        // Aggregate scores by group
        let mut group_scores: HashMap<String, f64> = HashMap::new();
        for (group_name, node_ids) in groups {
            let group_score: f64 = node_ids.iter().filter_map(|id| scores.get(id)).sum();
            group_scores.insert(group_name.clone(), group_score);
        }

        group_scores
    }

    /// Select top nodes based on PageRank scores
    fn select_top_nodes(
        &self,
        scores: HashMap<String, f64>,
        groups: &HashMap<String, Vec<String>>,
    ) -> Vec<String> {
        // Sort groups by score
        let mut sorted_groups: Vec<(String, f64)> = scores.into_iter().collect();
        sorted_groups.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Select top groups up to max_nodes
        let mut selected_nodes = Vec::new();
        let mut node_count = 0;

        for (group_name, _score) in sorted_groups {
            if let Some(node_ids) = groups.get(&group_name) {
                // Add all nodes from this group if we have room
                if node_count + node_ids.len() <= self.max_nodes {
                    selected_nodes.extend(node_ids.clone());
                    node_count += node_ids.len();
                } else {
                    // Add as many as we can
                    let remaining = self.max_nodes - node_count;
                    selected_nodes.extend(node_ids.iter().take(remaining).cloned());
                    break;
                }
            }
        }

        selected_nodes
    }

    /// Build the final graph with edge budget
    fn build_with_budget(
        &self,
        selected_nodes: Vec<String>,
        original_graph: &DependencyGraph,
    ) -> Result<FixedGraph> {
        let selected_set: HashSet<_> = selected_nodes.iter().cloned().collect();
        let mut nodes = BTreeMap::new();
        let mut edges = Vec::new();

        // Add selected nodes with semantic names
        for node_id in &selected_nodes {
            if let Some(node) = original_graph.nodes.get(node_id) {
                let semantic_name = self.namer.get_semantic_name(node_id, node);

                let fixed_node = FixedNode {
                    id: node_id.clone(),
                    display_name: semantic_name.clone(),
                    node_type: node.node_type.clone(),
                    complexity: node.complexity as u64,
                };

                nodes.insert(semantic_name, fixed_node);
            }
        }

        // Add edges between selected nodes
        let mut edge_count = 0;
        for edge in &original_graph.edges {
            if selected_set.contains(&edge.from) && selected_set.contains(&edge.to) {
                edges.push(edge.clone());
                edge_count += 1;

                if edge_count >= self.max_edges {
                    break;
                }
            }
        }

        Ok(FixedGraph { nodes, edges })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_graph() -> DependencyGraph {
        let mut nodes = HashMap::new();

        // Add some test nodes
        nodes.insert(
            "node1".to_string(),
            NodeInfo {
                id: "node1".to_string(),
                label: "foo".to_string(),
                node_type: NodeType::Module,
                file_path: "src/services/foo.rs".to_string(),
                line_number: 1,
                complexity: 10,
                metadata: HashMap::new(),
            },
        );

        nodes.insert(
            "node2".to_string(),
            NodeInfo {
                id: "node2".to_string(),
                label: "bar".to_string(),
                node_type: NodeType::Module,
                file_path: "src/models/bar.rs".to_string(),
                line_number: 1,
                complexity: 5,
                metadata: HashMap::new(),
            },
        );

        let edges = vec![Edge {
            from: "node1".to_string(),
            to: "node2".to_string(),
            edge_type: crate::models::dag::EdgeType::Imports,
            weight: 1,
        }];

        DependencyGraph { nodes, edges }
    }

    #[test]
    fn test_deterministic_build() {
        let config = GraphConfig::default();
        let builder = FixedGraphBuilder::new(config);
        let graph = create_test_graph();

        // Multiple runs should produce identical output
        let result1 = builder.build(&graph).unwrap();
        let result2 = builder.build(&graph).unwrap();

        assert_eq!(result1, result2);
    }

    #[test]
    fn test_node_limit() {
        let config = GraphConfig {
            max_nodes: 1,
            max_edges: 10,
            grouping: GroupingStrategy::Module,
        };
        let builder = FixedGraphBuilder::new(config);
        let graph = create_test_graph();

        let result = builder.build(&graph).unwrap();
        assert!(result.nodes.len() <= 1);
    }
}
