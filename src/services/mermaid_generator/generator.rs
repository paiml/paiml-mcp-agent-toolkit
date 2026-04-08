#![cfg_attr(coverage_nightly, coverage(off))]
//! Core diagram generation logic for the Mermaid generator

use super::types::MermaidGenerator;
use crate::models::dag::{DependencyGraph, NodeType};
use crate::services::fixed_graph_builder::{FixedGraphBuilder, GraphConfig};
use std::fmt::Write;

impl MermaidGenerator {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Generate.
    pub fn generate(&self, graph: &DependencyGraph) -> String {
        // Use the deterministic fixed graph builder
        let config = GraphConfig {
            max_nodes: 50,  // Default reasonable limit
            max_edges: 400, // Below Mermaid's 500 edge limit
            grouping: crate::services::fixed_graph_builder::GroupingStrategy::Module,
        };
        self.generate_with_config(graph, &config)
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Generate with config.
    pub fn generate_with_config(&self, graph: &DependencyGraph, config: &GraphConfig) -> String {
        // Build fixed-size graph with PageRank selection
        let builder = FixedGraphBuilder::new(config.clone());
        let fixed_graph = match builder.build(graph) {
            Ok(fixed) => fixed,
            Err(_) => {
                // Fallback to original implementation if builder fails
                return self.generate_legacy(graph);
            }
        };

        let mut output = String::from("graph TD\n");

        // Generate nodes with deterministic ordering
        for node in fixed_graph.nodes.values() {
            let sanitized_id = self.sanitize_id(&node.id);
            let escaped_label = self.escape_mermaid_label(&node.display_name);

            // Generate node with proper shape based on type
            let node_def = match node.node_type {
                NodeType::Module => format!("{sanitized_id}[{escaped_label}]"),
                NodeType::Function => format!("{sanitized_id}[{escaped_label}]"),
                NodeType::Class => format!("{sanitized_id}[{escaped_label}]"),
                NodeType::Trait => format!("{sanitized_id}(({escaped_label}))"),
                NodeType::Interface => format!("{sanitized_id}(({escaped_label}))"),
            };

            writeln!(output, "    {node_def}").expect("writing to String never fails");
        }

        // Add blank line between nodes and edges
        output.push('\n');

        // Generate edges
        for edge in &fixed_graph.edges {
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

        // Add styling based on complexity if enabled
        if self.options.show_complexity {
            output.push('\n');
            for node in fixed_graph.nodes.values() {
                let color = self.get_complexity_color(node.complexity as u32);
                let (stroke_style, stroke_width) = self.get_node_stroke_style(&node.node_type);

                writeln!(
                    output,
                    "    style {} fill:{}{},stroke-width:{}px",
                    self.sanitize_id(&node.id),
                    color,
                    stroke_style,
                    stroke_width
                )
                .expect("writing to String never fails");
            }
        }

        output
    }

    // Legacy implementation for fallback
    pub(super) fn generate_legacy(&self, graph: &DependencyGraph) -> String {
        let mut output = String::from("graph TD\n");

        // Generate nodes
        self.generate_nodes(graph, &mut output);

        // Add blank line between nodes and edges
        output.push('\n');

        // Generate edges
        self.generate_edges(graph, &mut output);

        // Add styling based on complexity if enabled
        if self.options.show_complexity {
            output.push('\n');
            self.generate_styles(graph, &mut output);
        }

        output
    }
}
