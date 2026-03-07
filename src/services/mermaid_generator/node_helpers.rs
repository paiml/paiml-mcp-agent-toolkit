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
