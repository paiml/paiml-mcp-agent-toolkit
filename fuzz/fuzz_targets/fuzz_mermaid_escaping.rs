#![no_main]

use libfuzzer_sys::fuzz_target;
use arbitrary::{Arbitrary, Unstructured};
use paiml_mcp_agent_toolkit::models::dag::{DependencyGraph, NodeInfo, NodeType};
use paiml_mcp_agent_toolkit::services::mermaid_generator::{MermaidGenerator, MermaidOptions};

// Focus on special character handling
#[derive(Arbitrary, Debug)]
struct EscapeFuzzInput {
    labels: Vec<FuzzLabel>,
}

#[derive(Arbitrary, Debug)]
struct FuzzLabel {
    // Various special character combinations
    content: String,
    include_pipe: bool,
    include_quotes: bool,
    include_brackets: bool,
    include_braces: bool,
    include_angle: bool,
    include_ampersand: bool,
    include_newline: bool,
}

fuzz_target!(|data: &[u8]| {
    let mut u = Unstructured::new(data);
    if let Ok(input) = EscapeFuzzInput::arbitrary(&mut u) {
        let graph = build_escape_test_graph(input);
        
        let generator = MermaidGenerator::new(MermaidOptions {
            show_complexity: true,
            ..Default::default()
        });
        
        let output = generator.generate(&graph);
        
        // Verify all special characters are properly escaped
        assert_proper_escaping(&output);
        
        // Verify the output is parseable (no syntax errors)
        assert_valid_mermaid_syntax(&output);
    }
});

fn build_escape_test_graph(input: EscapeFuzzInput) -> DependencyGraph {
    let mut graph = DependencyGraph::new();
    
    for (i, fuzz_label) in input.labels.into_iter().take(100).enumerate() {
        let label = build_label_with_special_chars(fuzz_label);
        
        graph.add_node(NodeInfo {
            id: format!("node_{}", i),
            label,
            node_type: match i % 5 {
                0 => NodeType::Function,
                1 => NodeType::Class,
                2 => NodeType::Module,
                3 => NodeType::Trait,
                _ => NodeType::Interface,
            },
            file_path: format!("file{}.rs", i),
            line_number: i,
            complexity: (i as u32 % 20) + 1,
        });
    }
    
    graph
}

fn build_label_with_special_chars(fuzz: FuzzLabel) -> String {
    let mut label = fuzz.content.chars().take(100).collect::<String>();
    
    // Inject special characters based on flags
    if fuzz.include_pipe {
        label.push_str(" | pipe");
    }
    if fuzz.include_quotes {
        label.push_str(r#" "quoted" text"#);
    }
    if fuzz.include_brackets {
        label.push_str(" [bracketed]");
    }
    if fuzz.include_braces {
        label.push_str(" {braced}");
    }
    if fuzz.include_angle {
        label.push_str(" <angled>");
    }
    if fuzz.include_ampersand {
        label.push_str(" & ampersand");
    }
    if fuzz.include_newline {
        label.push_str("\nnewline");
    }
    
    // Ensure non-empty
    if label.is_empty() {
        "empty".to_string()
    } else {
        label
    }
}

fn assert_proper_escaping(mermaid: &str) {
    // Check that within node definitions, special characters are escaped
    for line in mermaid.lines() {
        if line.contains("[\"") && line.contains("\"]") {
            // This is a node definition
            let start = line.find("[\"").unwrap() + 2;
            let end = line.rfind("\"]").unwrap();
            let content = &line[start..end];
            
            // These characters should be escaped
            assert!(!content.contains('"') || content.contains("&quot;"), 
                "Unescaped quote in: {}", line);
            assert!(!content.contains('|') || content.contains("&#124;"), 
                "Unescaped pipe in: {}", line);
            assert!(!content.contains('[') || content.contains("&#91;"), 
                "Unescaped bracket in: {}", line);
            assert!(!content.contains(']') || content.contains("&#93;"), 
                "Unescaped bracket in: {}", line);
            assert!(!content.contains('{') || content.contains("&#123;"), 
                "Unescaped brace in: {}", line);
            assert!(!content.contains('}') || content.contains("&#125;"), 
                "Unescaped brace in: {}", line);
            assert!(!content.contains('<') || content.contains("&lt;"), 
                "Unescaped angle in: {}", line);
            assert!(!content.contains('>') || content.contains("&gt;"), 
                "Unescaped angle in: {}", line);
            
            // Raw ampersand should only appear as part of escape sequences
            let amp_count = content.matches('&').count();
            let escape_count = content.matches("&amp;").count() +
                              content.matches("&quot;").count() +
                              content.matches("&#").count() +
                              content.matches("&lt;").count() +
                              content.matches("&gt;").count();
            assert!(amp_count <= escape_count * 2, 
                "Unescaped ampersand in: {}", line);
        }
    }
}

fn assert_valid_mermaid_syntax(mermaid: &str) {
    let lines: Vec<&str> = mermaid.lines().collect();
    
    // First line must be graph declaration
    assert_eq!(lines[0], "graph TD", "Invalid graph declaration");
    
    // Track open/close of node definitions
    let mut bracket_depth = 0;
    let mut brace_depth = 0;
    
    for line in &lines[1..] {
        if line.trim().is_empty() {
            continue;
        }
        
        // Count brackets and braces
        for ch in line.chars() {
            match ch {
                '[' => {
                    bracket_depth += 1;
                }
                ']' => {
                    bracket_depth -= 1;
                }
                '{' => brace_depth += 1,
                '}' => brace_depth -= 1,
                _ => {}
            }
        }
        
        // Brackets and braces must be balanced per line for Mermaid
        assert_eq!(bracket_depth, 0, "Unbalanced brackets in line: {}", line);
        assert_eq!(brace_depth, 0, "Unbalanced braces in line: {}", line);
    }
    
    // No quotes should appear outside of node definitions
    for line in &lines[1..] {
        if !line.contains("[\"") && !line.contains("{{\"") {
            // This is not a node definition line
            assert!(!line.contains('"'), 
                "Quote found outside node definition: {}", line);
        }
    }
}