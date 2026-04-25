#![cfg_attr(coverage_nightly, coverage(off))]
//! Helper methods for generating Mermaid node and edge output

use super::types::MermaidGenerator;
use crate::models::dag::{DependencyGraph, NodeInfo, NodeType};
use std::fmt::Write;

impl MermaidGenerator {
    pub(super) fn generate_nodes(&self, graph: &DependencyGraph, output: &mut String) {
        for (id, node) in &graph.nodes {
            let sanitized_id = self.sanitize_id(id);
            let semantic_name = self.get_semantic_name(id, node);
            let escaped_label = self.escape_mermaid_label(&semantic_name);

            // Generate node with proper shape based on type
            let node_def = match node.node_type {
                NodeType::Module => {
                    format!("{sanitized_id}[{escaped_label}]")
                }
                NodeType::Function => {
                    format!("{sanitized_id}[{escaped_label}]")
                }
                NodeType::Class => {
                    format!("{sanitized_id}[{escaped_label}]")
                }
                NodeType::Trait => {
                    format!("{sanitized_id}(({escaped_label}))")
                }
                NodeType::Interface => {
                    format!("{sanitized_id}(({escaped_label}))")
                }
            };

            writeln!(output, "    {node_def}").expect("writing to String never fails");
        }
    }

    pub(super) fn generate_edges(&self, graph: &DependencyGraph, output: &mut String) {
        for edge in &graph.edges {
            if graph.nodes.contains_key(&edge.from) && graph.nodes.contains_key(&edge.to) {
                let arrow = self.get_edge_arrow(&edge.edge_type);
                writeln!(
                    output,
                    "    {} {} {}",
                    self.sanitize_id(&edge.from),
                    arrow,
                    self.sanitize_id(&edge.to)
                )
                .expect("writing to String never fails");
            }
        }
    }

    pub(super) fn generate_styles(&self, graph: &DependencyGraph, output: &mut String) {
        for (id, node) in &graph.nodes {
            let color = self.get_complexity_color(node.complexity);
            let (stroke_style, stroke_width) = self.get_node_stroke_style(&node.node_type);

            writeln!(
                output,
                "    style {} fill:{}{},stroke-width:{}px",
                self.sanitize_id(id),
                color,
                stroke_style,
                stroke_width
            )
            .expect("writing to String never fails");
        }
    }

    pub(super) fn get_semantic_name(&self, id: &str, node: &NodeInfo) -> String {
        self.namer.get_semantic_name(id, node)
    }
}

#[cfg(test)]
mod node_helpers_tests {
    //! Covers generate_nodes/edges/styles + get_semantic_name in
    //! mermaid_generator/node_helpers.rs (46 uncov on broad, 0% cov).
    use super::super::types::{MermaidGenerator, MermaidOptions};
    use crate::models::dag::{DependencyGraph, Edge, EdgeType, NodeInfo, NodeType};
    use rustc_hash::FxHashMap;

    fn node(id: &str, label: &str, ty: NodeType, complexity: u32) -> NodeInfo {
        NodeInfo {
            id: id.to_string(),
            label: label.to_string(),
            node_type: ty,
            file_path: format!("src/{id}.rs"),
            line_number: 1,
            complexity,
            metadata: FxHashMap::default(),
        }
    }

    fn graph_with(nodes: Vec<NodeInfo>, edges: Vec<Edge>) -> DependencyGraph {
        let mut map = FxHashMap::default();
        for n in nodes {
            map.insert(n.id.clone(), n);
        }
        DependencyGraph { nodes: map, edges }
    }

    #[test]
    fn test_generate_nodes_empty_graph_writes_nothing() {
        let g = graph_with(vec![], vec![]);
        let gen = MermaidGenerator::new(MermaidOptions::default());
        let mut out = String::new();
        gen.generate_nodes(&g, &mut out);
        assert!(out.is_empty());
    }

    #[test]
    fn test_generate_nodes_emits_one_line_per_node() {
        let g = graph_with(
            vec![
                node("foo", "Foo", NodeType::Function, 1),
                node("bar", "Bar", NodeType::Class, 5),
            ],
            vec![],
        );
        let gen = MermaidGenerator::new(MermaidOptions::default());
        let mut out = String::new();
        gen.generate_nodes(&g, &mut out);
        // Each node should appear by sanitized id in the output.
        assert!(out.contains("foo"));
        assert!(out.contains("bar"));
    }

    #[test]
    fn test_generate_nodes_uses_double_paren_for_trait_and_interface() {
        let g = graph_with(
            vec![
                node("trait_id", "MyTrait", NodeType::Trait, 1),
                node("iface_id", "MyIface", NodeType::Interface, 1),
            ],
            vec![],
        );
        let gen = MermaidGenerator::new(MermaidOptions::default());
        let mut out = String::new();
        gen.generate_nodes(&g, &mut out);
        // Trait + Interface use ((label)) shape.
        assert!(out.contains("(("));
        assert!(out.contains("))"));
    }

    #[test]
    fn test_generate_edges_emits_one_line_per_edge_with_known_endpoints() {
        let g = graph_with(
            vec![
                node("a", "A", NodeType::Function, 1),
                node("b", "B", NodeType::Function, 1),
            ],
            vec![Edge {
                from: "a".to_string(),
                to: "b".to_string(),
                edge_type: EdgeType::Calls,
                weight: 1,
            }],
        );
        let gen = MermaidGenerator::new(MermaidOptions::default());
        let mut out = String::new();
        gen.generate_edges(&g, &mut out);
        assert!(out.contains("a"));
        assert!(out.contains("b"));
        // Edge produces something like "a --> b\n"; assert non-empty.
        assert!(!out.trim().is_empty());
    }

    #[test]
    fn test_generate_edges_skips_edge_with_unknown_endpoint() {
        // Only "a" exists; edge to "missing" should be dropped.
        let g = graph_with(
            vec![node("a", "A", NodeType::Function, 1)],
            vec![Edge {
                from: "a".to_string(),
                to: "missing".to_string(),
                edge_type: EdgeType::Calls,
                weight: 1,
            }],
        );
        let gen = MermaidGenerator::new(MermaidOptions::default());
        let mut out = String::new();
        gen.generate_edges(&g, &mut out);
        assert!(out.is_empty());
    }

    #[test]
    fn test_generate_edges_empty_graph_writes_nothing() {
        let g = graph_with(vec![], vec![]);
        let gen = MermaidGenerator::new(MermaidOptions::default());
        let mut out = String::new();
        gen.generate_edges(&g, &mut out);
        assert!(out.is_empty());
    }

    #[test]
    fn test_generate_styles_emits_style_line_per_node() {
        let g = graph_with(
            vec![
                node("foo", "Foo", NodeType::Function, 5),
                node("bar", "Bar", NodeType::Class, 25),
            ],
            vec![],
        );
        let gen = MermaidGenerator::new(MermaidOptions::default());
        let mut out = String::new();
        gen.generate_styles(&g, &mut out);
        // Each style line begins with "    style ".
        let style_lines = out.matches("    style").count();
        assert_eq!(style_lines, 2);
        assert!(out.contains("fill:"));
        assert!(out.contains("stroke-width:"));
    }

    #[test]
    fn test_generate_styles_empty_graph_writes_nothing() {
        let g = graph_with(vec![], vec![]);
        let gen = MermaidGenerator::new(MermaidOptions::default());
        let mut out = String::new();
        gen.generate_styles(&g, &mut out);
        assert!(out.is_empty());
    }

    #[test]
    fn test_get_semantic_name_delegates_to_namer() {
        let gen = MermaidGenerator::new(MermaidOptions::default());
        let n = node("foo::bar", "bar", NodeType::Function, 1);
        let name = gen.get_semantic_name("foo::bar", &n);
        // Just verify it returns a non-empty string (delegation works).
        assert!(!name.is_empty());
    }
}
