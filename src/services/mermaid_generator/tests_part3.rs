#![cfg_attr(coverage_nightly, coverage(off))]
//! Additional tests: output format, regression, and label escaping tests

#[cfg(test)]
mod tests {
    use crate::models::dag::{DependencyGraph, NodeInfo, NodeType};
    use crate::services::mermaid_generator::{MermaidGenerator, MermaidOptions};
    use rustc_hash::FxHashMap;

    #[test]
    fn test_mermaid_output_format() {
        let mut graph = DependencyGraph::new();

        graph.add_node(NodeInfo {
            id: "test".to_string(),
            label: "Test".to_string(),
            node_type: NodeType::Module,
            file_path: "test.rs".to_string(),
            line_number: 1,
            complexity: 3,
            metadata: FxHashMap::default(),
        });

        let generator = MermaidGenerator::new(MermaidOptions {
            show_complexity: true,
            ..Default::default()
        });

        let output = generator.generate(&graph);

        assert!(output.starts_with("graph TD\n"));
        assert!(output.contains("test"));
        assert!(output.contains("style test fill:#90EE90"));
    }

    #[test]
    fn test_regression_empty_nodes_bug() {
        let mut graph = DependencyGraph::new();

        let test_cases = vec![
            ("fn|process", "Function with pipe"),
            ("struct <T>", "Generic struct"),
            ("impl Display for &'a str", "Complex impl"),
            ("async fn handle_request()", "Async function"),
            ("mod tests { #[test] }", "Module with attributes"),
            ("trait Iterator<Item=T>", "Associated type"),
            ("use std::io::{Read, Write}", "Multiple imports"),
        ];

        for (id, label) in test_cases.iter() {
            graph.add_node(NodeInfo {
                id: id.to_string(),
                label: label.to_string(),
                node_type: NodeType::Function,
                file_path: "test.rs".to_string(),
                line_number: 1,
                complexity: 5,
                metadata: FxHashMap::default(),
            });
        }

        let generator = MermaidGenerator::new(MermaidOptions::default());
        let output = generator.generate(&graph);

        for (id, _label) in test_cases {
            let sanitized_id = generator.sanitize_id(id);
            assert!(
                output.contains(&format!("{sanitized_id}[")),
                "Node '{id}' is missing its label brackets in output"
            );
        }

        let lines: Vec<&str> = output.lines().collect();
        for line in lines {
            let trimmed = line.trim();
            if trimmed.is_empty()
                || trimmed.starts_with("graph")
                || trimmed.contains("-->")
                || trimmed.contains("-.->")
                || trimmed.contains("---")
                || trimmed.starts_with("style")
            {
                continue;
            }

            if !trimmed.contains("graph") {
                assert!(
                    trimmed.contains('[') || trimmed.contains("(("),
                    "Found bare node ID without label: {trimmed}"
                );
            }
        }
    }

    #[test]
    fn test_escape_mermaid_label() {
        let generator = MermaidGenerator::default();

        assert_eq!(generator.escape_mermaid_label("simple"), "simple");
        assert_eq!(generator.escape_mermaid_label("with|pipe"), "with - pipe");
        assert_eq!(
            generator.escape_mermaid_label("with\"quotes\""),
            "with'quotes'"
        );
        assert_eq!(
            generator.escape_mermaid_label("with'apostrophe"),
            "with'apostrophe"
        );
        assert_eq!(
            generator.escape_mermaid_label("with[brackets]"),
            "with(brackets)"
        );
        assert_eq!(
            generator.escape_mermaid_label("with{braces}"),
            "with(braces)"
        );
        assert_eq!(generator.escape_mermaid_label("with<angle>"), "with(angle)");
        assert_eq!(
            generator.escape_mermaid_label("with&ampersand"),
            "with and ampersand"
        );
        assert_eq!(generator.escape_mermaid_label("line\nbreak"), "line break");
        assert_eq!(
            generator.escape_mermaid_label("Function: test | Complexity: 5"),
            "Function: test  -  Complexity: 5"
        );
    }
}
