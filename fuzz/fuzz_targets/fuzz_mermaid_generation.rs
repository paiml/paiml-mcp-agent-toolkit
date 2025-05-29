#![no_main]

use libfuzzer_sys::fuzz_target;
use arbitrary::{Arbitrary, Unstructured};
use paiml_mcp_agent_toolkit::models::dag::{DependencyGraph, NodeInfo, Edge, EdgeType, NodeType};
use paiml_mcp_agent_toolkit::services::mermaid_generator::{MermaidGenerator, MermaidOptions};

// Structure-aware fuzzing input
#[derive(Arbitrary, Debug)]
struct FuzzInput {
    nodes: Vec<FuzzNode>,
    edges: Vec<FuzzEdge>,
    options: FuzzOptions,
}

#[derive(Arbitrary, Debug)]
struct FuzzNode {
    id: String,
    label: String,
    node_type: FuzzNodeType,
    complexity: u8, // Bounded to prevent overflow
    file_path: String,
    line_number: u16,
}

#[derive(Arbitrary, Debug)]
enum FuzzNodeType {
    Function,
    Class,
    Module,
    Trait,
    Interface,
}

#[derive(Arbitrary, Debug)]
struct FuzzEdge {
    from_idx: u8, // Index into nodes array
    to_idx: u8,
    edge_type: FuzzEdgeType,
}

#[derive(Arbitrary, Debug)]
enum FuzzEdgeType {
    Calls,
    Imports,
    Inherits,
    Implements,
    Uses,
}

#[derive(Arbitrary, Debug)]
struct FuzzOptions {
    show_complexity: bool,
    max_depth: Option<u8>,
    filter_external: bool,
    group_by_module: bool,
}

fuzz_target!(|data: &[u8]| {
    let mut u = Unstructured::new(data);
    if let Ok(input) = FuzzInput::arbitrary(&mut u) {
        // Skip if no nodes
        if input.nodes.is_empty() {
            return;
        }
        
        // Convert to domain types
        let graph = build_dependency_graph(input);
        
        // This should never panic
        let options = MermaidOptions {
            show_complexity: graph.options.show_complexity,
            max_depth: graph.options.max_depth.map(|d| d as usize),
            filter_external: graph.options.filter_external,
            group_by_module: graph.options.group_by_module,
        };
        
        let generator = MermaidGenerator::new(options);
        let output = generator.generate(&graph.graph);
        
        // Verify invariants
        assert_invariants(&output, &graph.graph);
    }
});

struct ConvertedGraph {
    graph: DependencyGraph,
    options: FuzzOptions,
}

fn build_dependency_graph(input: FuzzInput) -> ConvertedGraph {
    let mut graph = DependencyGraph::new();
    
    // Add nodes
    for node in &input.nodes {
        graph.add_node(NodeInfo {
            id: sanitize_id(&node.id),
            label: node.label.clone(),
            node_type: convert_node_type(&node.node_type),
            file_path: node.file_path.clone(),
            line_number: node.line_number as usize,
            complexity: node.complexity as u32,
        });
    }
    
    // Add edges (only between valid nodes)
    for edge in &input.edges {
        let from_idx = edge.from_idx as usize;
        let to_idx = edge.to_idx as usize;
        
        if from_idx < input.nodes.len() && to_idx < input.nodes.len() {
            let from_id = sanitize_id(&input.nodes[from_idx].id);
            let to_id = sanitize_id(&input.nodes[to_idx].id);
            
            graph.add_edge(Edge {
                from: from_id,
                to: to_id,
                edge_type: convert_edge_type(&edge.edge_type),
                weight: 1,
            });
        }
    }
    
    ConvertedGraph {
        graph,
        options: input.options,
    }
}

fn sanitize_id(id: &str) -> String {
    if id.is_empty() {
        "empty".to_string()
    } else {
        // Limit length to prevent DOS
        id.chars().take(100).collect()
    }
}

fn convert_node_type(fuzz_type: &FuzzNodeType) -> NodeType {
    match fuzz_type {
        FuzzNodeType::Function => NodeType::Function,
        FuzzNodeType::Class => NodeType::Class,
        FuzzNodeType::Module => NodeType::Module,
        FuzzNodeType::Trait => NodeType::Trait,
        FuzzNodeType::Interface => NodeType::Interface,
    }
}

fn convert_edge_type(fuzz_type: &FuzzEdgeType) -> EdgeType {
    match fuzz_type {
        FuzzEdgeType::Calls => EdgeType::Calls,
        FuzzEdgeType::Imports => EdgeType::Imports,
        FuzzEdgeType::Inherits => EdgeType::Inherits,
        FuzzEdgeType::Implements => EdgeType::Implements,
        FuzzEdgeType::Uses => EdgeType::Uses,
    }
}

fn assert_invariants(mermaid: &str, graph: &DependencyGraph) {
    // Critical: Verify syntax validity
    assert!(mermaid.starts_with("graph TD\n"), "Missing graph header");
    
    // Verify no unescaped special chars in node definitions
    let lines: Vec<&str> = mermaid.lines().collect();
    for line in &lines[1..] {
        if line.trim().is_empty() {
            continue;
        }
        
        // Skip edge lines
        if line.contains("-->") || line.contains("-.->") || 
           line.contains("--|>") || line.contains("-->>") || line.contains("---") {
            continue;
        }
        
        // Skip style lines
        if line.trim().starts_with("style ") {
            continue;
        }
        
        // Node lines must have proper quoting
        if line.contains('[') {
            assert!(line.contains("[\"") && line.contains("\"]"), 
                "Improperly quoted node: {}", line);
        } else if line.contains("{{") {
            assert!(line.contains("{{\"") && line.contains("\"}}"), 
                "Improperly quoted module: {}", line);
        }
    }
    
    // Verify all nodes are represented
    for (id, _) in &graph.nodes {
        let sanitized_id = id.replace("::", "_")
            .replace(['/', '.', '-', ' '], "_");
        assert!(mermaid.contains(&sanitized_id), 
            "Node {} not found in output", id);
    }
    
    // Verify no raw pipe characters in labels
    assert!(!has_unescaped_pipes(mermaid), 
        "Found unescaped pipe character in output");
    
    // Verify edges reference existing nodes
    for edge in &graph.edges {
        if graph.nodes.contains_key(&edge.from) && graph.nodes.contains_key(&edge.to) {
            let from_id = edge.from.replace("::", "_")
                .replace(['/', '.', '-', ' '], "_");
            let _to_id = edge.to.replace("::", "_")
                .replace(['/', '.', '-', ' '], "_");
            
            // Edge should be present
            let edge_pattern = format!("{} ", from_id);
            assert!(mermaid.contains(&edge_pattern), 
                "Edge from {} not found", edge.from);
        }
    }
}

fn has_unescaped_pipes(mermaid: &str) -> bool {
    // Check for unescaped pipes (not &#124;)
    for line in mermaid.lines() {
        if line.contains('[') || line.contains("{{") {
            // Within node definitions
            if let Some(start) = line.find("[\"") {
                if let Some(end) = line.rfind("\"]") {
                    let content = &line[start+2..end];
                    if content.contains('|') && !content.contains("&#124;") {
                        return true;
                    }
                }
            }
            if let Some(start) = line.find("{{\"") {
                if let Some(end) = line.rfind("\"}}") {
                    let content = &line[start+3..end];
                    if content.contains('|') && !content.contains("&#124;") {
                        return true;
                    }
                }
            }
        }
    }
    false
}