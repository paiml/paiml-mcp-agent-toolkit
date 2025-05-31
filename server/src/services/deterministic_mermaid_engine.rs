//! Deterministic Mermaid Generation Engine
//!
//! This module implements PageRank-based layout and deterministic Mermaid
//! diagram generation as specified in deterministic-graphs-mmd-spec.md

use crate::models::dag::EdgeType;
use crate::services::unified_ast_engine::{ModuleNode, ProjectMetrics};
use petgraph::stable_graph::{NodeIndex, StableGraph};
use petgraph::visit::{EdgeRef, IntoEdgeReferences};
use std::collections::BTreeMap;
use std::fmt::Write;

/// Deterministic Mermaid engine with PageRank-based layout
pub struct DeterministicMermaidEngine {
    /// Number of PageRank iterations for stable results
    pagerank_iterations: usize,
    /// Quantization factor to avoid floating-point drift
    quantization_factor: u32,
}

impl Default for DeterministicMermaidEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl DeterministicMermaidEngine {
    pub fn new() -> Self {
        Self {
            pagerank_iterations: 100,
            quantization_factor: 10000,
        }
    }

    /// Generate deterministic codebase modules Mermaid diagram
    pub fn generate_codebase_modules_mmd(
        &self,
        graph: &StableGraph<ModuleNode, EdgeType>,
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

            writeln!(&mut mermaid, "    {}[{}]", sanitized_id, escaped_label).unwrap();
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
            .unwrap();
        }

        mermaid
    }

    /// Generate service interaction diagram with complexity-based styling
    pub fn generate_service_interactions_mmd(
        &self,
        graph: &StableGraph<ModuleNode, EdgeType>,
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
            writeln!(&mut mermaid, "    {}[{}]", sanitized_id, escaped_label).unwrap();
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
            .unwrap();
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
            .unwrap();
        }

        mermaid
    }

    /// Compute PageRank scores for graph nodes
    fn compute_pagerank(
        &self,
        graph: &StableGraph<ModuleNode, EdgeType>,
        damping: f32,
        iterations: usize,
    ) -> BTreeMap<NodeIndex, f32> {
        let node_count = graph.node_count();
        if node_count == 0 {
            return BTreeMap::new();
        }

        let initial_score = 1.0 / node_count as f32;
        let mut scores: BTreeMap<NodeIndex, f32> = graph
            .node_indices()
            .map(|idx| (idx, initial_score))
            .collect();

        // Build adjacency information
        let mut outgoing: BTreeMap<NodeIndex, Vec<NodeIndex>> = BTreeMap::new();
        let mut incoming: BTreeMap<NodeIndex, Vec<NodeIndex>> = BTreeMap::new();

        for idx in graph.node_indices() {
            outgoing.insert(idx, Vec::new());
            incoming.insert(idx, Vec::new());
        }

        for edge in graph.edge_references() {
            outgoing
                .get_mut(&edge.source())
                .unwrap()
                .push(edge.target());
            incoming
                .get_mut(&edge.target())
                .unwrap()
                .push(edge.source());
        }

        // Iterative PageRank computation
        for _ in 0..iterations {
            let mut new_scores = BTreeMap::new();

            for &node in graph.node_indices().collect::<Vec<_>>().iter() {
                let mut score = (1.0 - damping) / node_count as f32;

                if let Some(incoming_nodes) = incoming.get(&node) {
                    for &incoming_node in incoming_nodes {
                        if let Some(outgoing_nodes) = outgoing.get(&incoming_node) {
                            let outgoing_count = outgoing_nodes.len() as f32;
                            if outgoing_count > 0.0 {
                                if let Some(&incoming_score) = scores.get(&incoming_node) {
                                    score += damping * incoming_score / outgoing_count;
                                }
                            }
                        }
                    }
                }

                new_scores.insert(node, score);
            }

            scores = new_scores;
        }

        scores
    }

    /// Filter graph to service modules only (heuristic)
    fn filter_to_services(
        &self,
        graph: &StableGraph<ModuleNode, EdgeType>,
    ) -> StableGraph<ModuleNode, EdgeType> {
        let mut service_graph = StableGraph::new();
        let mut node_mapping = BTreeMap::new();

        // Add nodes that look like services
        for idx in graph.node_indices() {
            let node = &graph[idx];
            if self.is_service_module(&node.name) {
                let new_idx = service_graph.add_node(node.clone());
                node_mapping.insert(idx, new_idx);
            }
        }

        // Add edges between service nodes
        for edge in graph.edge_references() {
            if let (Some(&source_idx), Some(&target_idx)) = (
                node_mapping.get(&edge.source()),
                node_mapping.get(&edge.target()),
            ) {
                service_graph.add_edge(source_idx, target_idx, edge.weight().clone());
            }
        }

        service_graph
    }

    /// Heuristic to determine if a module is a service
    fn is_service_module(&self, name: &str) -> bool {
        name.contains("service")
            || name.contains("handler")
            || name.contains("controller")
            || name.contains("api")
            || name.contains("engine")
    }

    /// Get Mermaid arrow style for edge type
    fn get_edge_arrow(&self, edge_type: &EdgeType) -> &'static str {
        match edge_type {
            EdgeType::Calls => "-->",
            EdgeType::Imports => "-.->",
            EdgeType::Inherits => "-->",
            EdgeType::Implements => "-.->",
            EdgeType::Uses => "---",
        }
    }

    /// Sanitize ID for Mermaid compatibility
    pub fn sanitize_id(&self, id: &str) -> String {
        // Replace common multi-character patterns
        let sanitized = id.replace("::", "_").replace(['/', '.', '-', ' '], "_");

        // Replace any remaining non-alphanumeric characters with underscores
        let sanitized: String = sanitized
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect();

        // Ensure it starts with a letter or underscore
        if sanitized.is_empty() {
            "_empty".to_string()
        } else if sanitized.chars().next().unwrap().is_numeric() {
            format!("_{}", sanitized)
        } else {
            sanitized
        }
    }

    /// Escape label for Mermaid compatibility
    pub fn escape_mermaid_label(&self, label: &str) -> String {
        // For maximum compatibility, use simple character replacements
        label
            .replace('&', " and ")
            .replace('"', "'")
            .replace('<', "(")
            .replace('>', ")")
            .replace('|', " - ")
            .replace('[', "(")
            .replace(']', ")")
            .replace('{', "(")
            .replace('}', ")")
            .replace('\n', " ")
    }
}

/// Complexity buckets for styling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ComplexityBucket {
    Low,
    Medium,
    High,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::unified_ast_engine::ModuleMetrics;
    use std::path::PathBuf;

    #[test]
    fn test_pagerank_determinism() {
        let engine = DeterministicMermaidEngine::new();
        let mut graph = StableGraph::new();

        // Create a simple 3-node graph
        let node1 = graph.add_node(ModuleNode {
            name: "node1".to_string(),
            path: PathBuf::from("node1.rs"),
            visibility: "public".to_string(),
            metrics: ModuleMetrics::default(),
        });

        let node2 = graph.add_node(ModuleNode {
            name: "node2".to_string(),
            path: PathBuf::from("node2.rs"),
            visibility: "public".to_string(),
            metrics: ModuleMetrics::default(),
        });

        let node3 = graph.add_node(ModuleNode {
            name: "node3".to_string(),
            path: PathBuf::from("node3.rs"),
            visibility: "public".to_string(),
            metrics: ModuleMetrics::default(),
        });

        graph.add_edge(node1, node2, EdgeType::Calls);
        graph.add_edge(node2, node3, EdgeType::Calls);
        graph.add_edge(node3, node1, EdgeType::Calls);

        // Compute PageRank multiple times
        let scores1 = engine.compute_pagerank(&graph, 0.85, 100);
        let scores2 = engine.compute_pagerank(&graph, 0.85, 100);

        // Results should be identical
        assert_eq!(
            scores1, scores2,
            "PageRank computation must be deterministic"
        );

        // All scores should sum to approximately 1.0
        let sum: f32 = scores1.values().sum();
        assert!(
            (sum - 1.0).abs() < 0.001,
            "PageRank scores should sum to 1.0, got {}",
            sum
        );
    }

    #[test]
    fn test_mermaid_output_determinism() {
        let engine = DeterministicMermaidEngine::new();
        let mut graph = StableGraph::new();

        // Add nodes in non-alphabetical order to test sorting
        let node_z = graph.add_node(ModuleNode {
            name: "z_module".to_string(),
            path: PathBuf::from("z.rs"),
            visibility: "public".to_string(),
            metrics: ModuleMetrics {
                complexity: 5,
                ..ModuleMetrics::default()
            },
        });

        let node_a = graph.add_node(ModuleNode {
            name: "a_module".to_string(),
            path: PathBuf::from("a.rs"),
            visibility: "public".to_string(),
            metrics: ModuleMetrics {
                complexity: 10,
                ..ModuleMetrics::default()
            },
        });

        graph.add_edge(node_z, node_a, EdgeType::Imports);

        // Generate diagram multiple times
        let mermaid1 = engine.generate_codebase_modules_mmd(&graph);
        let mermaid2 = engine.generate_codebase_modules_mmd(&graph);

        assert_eq!(mermaid1, mermaid2, "Mermaid output must be deterministic");

        // Check that output is well-formed
        assert!(mermaid1.starts_with("graph TD\n"));
        assert!(mermaid1.contains("a_module"));
        assert!(mermaid1.contains("z_module"));
        assert!(mermaid1.contains("-.->"));
    }

    #[test]
    fn test_sanitize_id() {
        let engine = DeterministicMermaidEngine::new();

        assert_eq!(engine.sanitize_id("foo::bar"), "foo_bar");
        assert_eq!(engine.sanitize_id("foo/bar.rs"), "foo_bar_rs");
        assert_eq!(engine.sanitize_id("foo-bar"), "foo_bar");
        assert_eq!(engine.sanitize_id("foo bar"), "foo_bar");
        assert_eq!(engine.sanitize_id("123foo"), "_123foo");
        assert_eq!(engine.sanitize_id("_foo"), "_foo");
        assert_eq!(engine.sanitize_id(""), "_empty");
    }

    #[test]
    fn test_escape_mermaid_label() {
        let engine = DeterministicMermaidEngine::new();

        assert_eq!(engine.escape_mermaid_label("simple"), "simple");
        assert_eq!(engine.escape_mermaid_label("with|pipe"), "with - pipe");
        assert_eq!(
            engine.escape_mermaid_label("with\"quotes\""),
            "with'quotes'"
        );
        assert_eq!(
            engine.escape_mermaid_label("with[brackets]"),
            "with(brackets)"
        );
        assert_eq!(engine.escape_mermaid_label("with{braces}"), "with(braces)");
        assert_eq!(engine.escape_mermaid_label("with<angle>"), "with(angle)");
        assert_eq!(
            engine.escape_mermaid_label("with&ampersand"),
            "with and ampersand"
        );
        assert_eq!(engine.escape_mermaid_label("line\nbreak"), "line break");
    }

    #[test]
    fn test_is_service_module() {
        let engine = DeterministicMermaidEngine::new();

        assert!(engine.is_service_module("user_service"));
        assert!(engine.is_service_module("api_handler"));
        assert!(engine.is_service_module("payment_controller"));
        assert!(engine.is_service_module("template_engine"));
        assert!(!engine.is_service_module("utils"));
        assert!(!engine.is_service_module("config"));
        assert!(!engine.is_service_module("models"));
    }

    #[test]
    fn test_complexity_styling() {
        let engine = DeterministicMermaidEngine::new();
        let mut graph = StableGraph::new();

        // Add service modules with different complexities
        let _low_complexity = graph.add_node(ModuleNode {
            name: "simple_service".to_string(),
            path: PathBuf::from("simple.rs"),
            visibility: "public".to_string(),
            metrics: ModuleMetrics {
                complexity: 5,
                ..ModuleMetrics::default()
            },
        });

        let _high_complexity = graph.add_node(ModuleNode {
            name: "complex_service".to_string(),
            path: PathBuf::from("complex.rs"),
            visibility: "public".to_string(),
            metrics: ModuleMetrics {
                complexity: 25,
                ..ModuleMetrics::default()
            },
        });

        let metrics = ProjectMetrics {
            file_count: 2,
            function_count: 10,
            avg_complexity: 15.0,
            max_complexity: 25,
        };

        let mermaid = engine.generate_service_interactions_mmd(&graph, &metrics);

        // Should contain complexity-based styling
        assert!(mermaid.contains("style simple_service fill:#90EE90")); // Low complexity - green
        assert!(mermaid.contains("style complex_service fill:#FF6347")); // High complexity - red
    }

    #[test]
    fn test_empty_graph() {
        let engine = DeterministicMermaidEngine::new();
        let graph = StableGraph::new();

        let mermaid = engine.generate_codebase_modules_mmd(&graph);
        assert_eq!(mermaid.trim(), "graph TD");

        let scores = engine.compute_pagerank(&graph, 0.85, 100);
        assert!(scores.is_empty());
    }
}
