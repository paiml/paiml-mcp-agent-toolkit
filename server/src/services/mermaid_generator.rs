use crate::models::dag::{DependencyGraph, EdgeType, NodeType};
use std::fmt::Write;

pub struct MermaidGenerator {
    options: MermaidOptions,
}

#[derive(Default)]
pub struct MermaidOptions {
    pub max_depth: Option<usize>,
    pub filter_external: bool,
    pub group_by_module: bool,
    pub show_complexity: bool,
}

impl MermaidGenerator {
    pub fn new(options: MermaidOptions) -> Self {
        Self { options }
    }

    pub fn generate(&self, graph: &DependencyGraph) -> String {
        let mut output = String::from("graph TD\n");

        // Generate nodes
        for (id, node) in &graph.nodes {
            let shape = match node.node_type {
                NodeType::Class => "[{}]",
                NodeType::Function => "({})",
                NodeType::Module => "{{{}}}",
                NodeType::Trait => "[/{}\\]",
                NodeType::Interface => "[\\{}/]",
            };

            let label = if self.options.show_complexity && node.complexity > 1 {
                format!("{}<br/>⚡{}", node.label, node.complexity)
            } else {
                node.label.clone()
            };

            writeln!(
                &mut output,
                "    {} {}",
                self.sanitize_id(id),
                shape.replace("{}", &label)
            )
            .unwrap();
        }

        // Add blank line between nodes and edges
        output.push('\n');

        // Generate edges
        for edge in &graph.edges {
            let arrow = match edge.edge_type {
                EdgeType::Calls => "-->",
                EdgeType::Imports => "-.->",
                EdgeType::Inherits => "==>",
                EdgeType::Implements => "-.->>",
                EdgeType::Uses => "-->",
            };

            // Only add edge if both nodes exist in the graph
            if graph.nodes.contains_key(&edge.from) && graph.nodes.contains_key(&edge.to) {
                writeln!(
                    &mut output,
                    "    {} {} {}",
                    self.sanitize_id(&edge.from),
                    arrow,
                    self.sanitize_id(&edge.to)
                )
                .unwrap();
            }
        }

        // Add styling based on complexity if enabled
        if self.options.show_complexity {
            output.push('\n');
            for (id, node) in &graph.nodes {
                let color = match node.complexity {
                    1..=3 => "#90EE90",  // Light green for low complexity
                    4..=7 => "#FFD700",  // Gold for medium complexity
                    8..=12 => "#FFA500", // Orange for high complexity
                    _ => "#FF6347",      // Tomato for very high complexity
                };
                writeln!(
                    &mut output,
                    "    style {} fill:{}",
                    self.sanitize_id(id),
                    color
                )
                .unwrap();
            }
        }

        output
    }

    fn sanitize_id(&self, id: &str) -> String {
        // Replace invalid Mermaid characters and ensure valid identifier
        let sanitized = id.replace("::", "_").replace(['/', '.', '-', ' '], "_");

        // Ensure it starts with a letter or underscore
        if sanitized.chars().next().is_some_and(|c| c.is_numeric()) {
            format!("_{}", sanitized)
        } else {
            sanitized
        }
    }
}

impl Default for MermaidGenerator {
    fn default() -> Self {
        Self::new(MermaidOptions::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::dag::{Edge, EdgeType, NodeInfo, NodeType};

    #[test]
    fn test_mermaid_generation() {
        let mut graph = DependencyGraph::new();

        graph.add_node(NodeInfo {
            id: "main.rs::main".to_string(),
            label: "main".to_string(),
            node_type: NodeType::Function,
            file_path: "main.rs".to_string(),
            line_number: 1,
            complexity: 2,
        });

        graph.add_node(NodeInfo {
            id: "lib.rs::process".to_string(),
            label: "process".to_string(),
            node_type: NodeType::Function,
            file_path: "lib.rs".to_string(),
            line_number: 10,
            complexity: 5,
        });

        graph.add_edge(Edge {
            from: "main.rs::main".to_string(),
            to: "lib.rs::process".to_string(),
            edge_type: EdgeType::Calls,
            weight: 1,
        });

        let generator = MermaidGenerator::new(MermaidOptions {
            show_complexity: true,
            ..Default::default()
        });

        let output = generator.generate(&graph);

        assert!(output.contains("graph TD"));
        assert!(output.contains("main_rs_main"));
        assert!(output.contains("lib_rs_process"));
        assert!(output.contains("-->")); // Calls arrow
        assert!(output.contains("⚡")); // Complexity indicator
    }
}
