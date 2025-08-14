#![no_main]

use libfuzzer_sys::fuzz_target;
use arbitrary::{Arbitrary, Unstructured};
use pmat::models::dag::{DependencyGraph, NodeInfo, Edge, EdgeType, NodeType};
use pmat::services::mermaid_generator::{MermaidGenerator, MermaidOptions};
use std::time::Instant;

// Bounded input for performance testing
#[derive(Arbitrary, Debug)]
struct PerfFuzzInput {
    node_count: u16, // Up to 65k nodes
    edge_ratio: u8,  // Edges as percentage of n^2 (0-100)
    complexity_range: (u8, u8),
    show_complexity: bool,
}

fuzz_target!(|data: &[u8]| {
    let mut u = Unstructured::new(data);
    if let Ok(input) = PerfFuzzInput::arbitrary(&mut u) {
        // Limit to reasonable sizes for performance testing
        let node_count = (input.node_count as usize).min(10_000);
        if node_count == 0 {
            return;
        }
        
        // Build graph
        let graph = build_performance_graph(node_count, input.edge_ratio, input.complexity_range);
        
        // Measure generation time
        let options = MermaidOptions {
            show_complexity: input.show_complexity,
            ..Default::default()
        };
        
        let generator = MermaidGenerator::new(options);
        let start = Instant::now();
        let output = generator.generate(&graph);
        let duration = start.elapsed();
        
        // Verify performance constraints
        assert_performance_bounds(&graph, duration);
        
        // Basic output validation
        assert!(!output.is_empty());
        assert!(output.starts_with("graph TD\n"));
    }
});

fn build_performance_graph(
    node_count: usize,
    edge_ratio: u8,
    complexity_range: (u8, u8)
) -> DependencyGraph {
    let mut graph = DependencyGraph::new();
    
    // Add nodes
    for i in 0..node_count {
        let node_type = match i % 5 {
            0 => NodeType::Function,
            1 => NodeType::Class,
            2 => NodeType::Module,
            3 => NodeType::Trait,
            _ => NodeType::Interface,
        };
        
        graph.add_node(NodeInfo {
            id: format!("node_{}", i),
            label: format!("Node {}", i),
            node_type,
            file_path: format!("file{}.rs", i / 100),
            line_number: i % 1000,
            complexity: complexity_range.0 as u32 + 
                       (i as u32 % (complexity_range.1 - complexity_range.0) as u32),
        });
    }
    
    // Add edges based on ratio
    let max_edges = (node_count * node_count) / 100 * (edge_ratio as usize).min(10);
    let edges_to_add = max_edges.min(node_count * 5); // Limit to 5x nodes
    
    // Use deterministic pattern for edges
    for i in 0..edges_to_add {
        let from = i % node_count;
        let to = (i * 7 + 13) % node_count; // Simple hash for distribution
        
        if from != to {
            let edge_type = match i % 5 {
                0 => EdgeType::Calls,
                1 => EdgeType::Imports,
                2 => EdgeType::Inherits,
                3 => EdgeType::Implements,
                _ => EdgeType::Uses,
            };
            
            graph.add_edge(Edge {
                from: format!("node_{}", from),
                to: format!("node_{}", to),
                edge_type,
                weight: 1,
            });
        }
    }
    
    graph
}

fn assert_performance_bounds(graph: &DependencyGraph, duration: std::time::Duration) {
    let n = graph.nodes.len();
    let e = graph.edges.len();
    
    // Expected O(n + e) complexity
    // Allow 1 microsecond per element as baseline
    let expected_max_micros = ((n + e) as u64).saturating_mul(1);
    
    // Add base overhead of 100 microseconds
    let allowed_micros = expected_max_micros.saturating_add(100);
    
    // Allow 10x margin for debug builds and system variance
    let margin = if cfg!(debug_assertions) { 10 } else { 5 };
    let max_allowed = allowed_micros.saturating_mul(margin);
    
    assert!(
        duration.as_micros() <= max_allowed as u128,
        "Performance regression: {}µs for {} nodes, {} edges (expected max {}µs)",
        duration.as_micros(),
        n,
        e,
        max_allowed
    );
    
    // Also check for suspiciously fast times that might indicate skipped work
    assert!(
        duration.as_nanos() > 1000, // At least 1µs
        "Suspiciously fast generation: {}ns",
        duration.as_nanos()
    );
}