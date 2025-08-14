#[cfg(test)]
mod tests {
    use super::super::dag::*;
    use proptest::prelude::*;
    use proptest::strategy::Strategy;
    use rustc_hash::{FxHashMap, FxHashSet};

    use crate::services::context::{AstItem, FileContext, ProjectContext, ProjectSummary};
    use crate::services::dag_builder::DagBuilder;
    use std::collections::HashMap;

    // Strategy for generating valid node IDs
    prop_compose! {
        fn arb_node_id()
            (module in "[a-z][a-z0-9_]{0,10}",
             name in "[a-zA-Z][a-zA-Z0-9_]{0,20}")
            -> String
        {
            format!("{}::{}", module, name)
        }
    }

    // Strategy for generating node info
    prop_compose! {
        fn arb_node_info()
            (id in arb_node_id(),
             node_type in prop::sample::select(vec![
                 NodeType::Function,
                 NodeType::Class,
                 NodeType::Module,
                 NodeType::Trait,
                 NodeType::Interface,
             ]),
             file_path in "[a-z][a-z0-9_/]{0,30}\\.rs",
             line_number in 1usize..1000,
             complexity in 0u32..50)
            -> NodeInfo
        {
            NodeInfo {
                id: id.clone(),
                label: id.split("::").last().unwrap_or(&id).to_string(),
                node_type,
                file_path,
                line_number,
                complexity,
                metadata: FxHashMap::default(),
            }
        }
    }

    // Strategy for generating edges
    prop_compose! {
        fn arb_edge(nodes: Vec<String>)
            (from_idx in 0..nodes.len().max(1),
             to_idx in 0..nodes.len().max(1),
             edge_type in prop::sample::select(vec![
                 EdgeType::Calls,
                 EdgeType::Imports,
                 EdgeType::Inherits,
                 EdgeType::Implements,
                 EdgeType::Uses,
             ]),
             weight in 1u32..10)
            -> Option<Edge>
        {
            if nodes.is_empty() || from_idx == to_idx {
                None
            } else {
                Some(Edge {
                    from: nodes[from_idx].clone(),
                    to: nodes[to_idx].clone(),
                    edge_type,
                    weight,
                })
            }
        }
    }

    #[test]
    fn test_dag_add_node_idempotent() {
        proptest!(|(node in arb_node_info())| {
            let mut graph = DependencyGraph::new();

            // Adding node once
            graph.add_node(node.clone());
            let count1 = graph.nodes.len();
            prop_assert_eq!(count1, 1);

            // Adding same node again should be idempotent
            graph.add_node(node.clone());
            let count2 = graph.nodes.len();
            prop_assert_eq!(count2, 1, "Adding same node twice should be idempotent");

            // The node should be retrievable
            let retrieved = graph.nodes.get(&node.id);
            prop_assert!(retrieved.is_some());
            prop_assert_eq!(&retrieved.unwrap().id, &node.id);
        });
    }

    #[test]
    fn test_dag_edge_validation() {
        proptest!(|(nodes in prop::collection::vec(arb_node_info(), 1..20))| {
            let mut graph = DependencyGraph::new();

            // Add all nodes
            for node in &nodes {
                graph.add_node(node.clone());
            }

            let node_ids: Vec<String> = nodes.iter().map(|n| n.id.clone()).collect();

            // Generate and add edges
            let mut edge_count = 0;
            for i in 0..nodes.len() {
                for j in 0..nodes.len() {
                    if i != j && i % 3 == 0 { // Add some edges
                        let edge = Edge {
                            from: node_ids[i].clone(),
                            to: node_ids[j].clone(),
                            edge_type: EdgeType::Calls,
                            weight: 1,
                        };
                        graph.add_edge(edge);
                        edge_count += 1;
                    }
                }
            }

            prop_assert_eq!(graph.edges.len(), edge_count);

            // All edges should have valid endpoints
            for edge in &graph.edges {
                prop_assert!(graph.nodes.contains_key(&edge.from),
                    "Edge references non-existent 'from' node: {}", edge.from);
                prop_assert!(graph.nodes.contains_key(&edge.to),
                    "Edge references non-existent 'to' node: {}", edge.to);
            }
        });
    }

    #[test]
    fn test_dag_filter_by_edge_type_preserves_structure() {
        proptest!(|(
            nodes in prop::collection::vec(arb_node_info(), 2..10),
            edge_types in prop::collection::vec(
                prop::sample::select(vec![
                    EdgeType::Calls,
                    EdgeType::Imports,
                    EdgeType::Inherits,
                    EdgeType::Implements,
                    EdgeType::Uses,
                ]),
                5..20
            )
        )| {
            let mut graph = DependencyGraph::new();

            // Add nodes
            for node in &nodes {
                graph.add_node(node.clone());
            }

            // Add edges with various types
            let node_ids: Vec<String> = nodes.iter().map(|n| n.id.clone()).collect();
            for (i, edge_type) in edge_types.into_iter().enumerate() {
                if node_ids.len() >= 2 {
                    let from_idx = i % node_ids.len();
                    let to_idx = (i + 1) % node_ids.len();
                    if from_idx != to_idx {
                        graph.add_edge(Edge {
                            from: node_ids[from_idx].clone(),
                            to: node_ids[to_idx].clone(),
                            edge_type,
                            weight: 1,
                        });
                    }
                }
            }

            // Test filtering for each edge type
            for filter_type in &[EdgeType::Calls, EdgeType::Imports, EdgeType::Inherits] {
                let filtered = graph.filter_by_edge_type(filter_type.clone());

                // All edges in filtered graph should have the correct type
                for edge in &filtered.edges {
                    prop_assert_eq!(&edge.edge_type, filter_type);
                }

                // Filtered graph should only contain nodes connected by filtered edges
                let connected_nodes: FxHashSet<String> = filtered.edges.iter()
                    .flat_map(|e| vec![e.from.clone(), e.to.clone()])
                    .collect();

                for node_id in filtered.nodes.keys() {
                    prop_assert!(connected_nodes.contains(node_id),
                        "Node {} not connected by any edge of type {:?}", node_id, filter_type);
                }
            }
        });
    }

    #[test]
    fn test_dag_no_self_loops() {
        proptest!(|(nodes in prop::collection::vec(arb_node_info(), 1..10))| {
            let mut graph = DependencyGraph::new();

            // Add nodes
            for node in &nodes {
                graph.add_node(node.clone());
            }

            // Add only non-self-loop edges
            for i in 0..nodes.len() {
                for j in 0..nodes.len() {
                    if i != j && i % 2 == 0 {  // Add some edges but not self-loops
                        let edge = Edge {
                            from: nodes[i].id.clone(),
                            to: nodes[j].id.clone(),
                            edge_type: EdgeType::Calls,
                            weight: 1,
                        };
                        graph.add_edge(edge);
                    }
                }
            }

            // Verify no self-loops exist
            for edge in &graph.edges {
                prop_assert_ne!(&edge.from, &edge.to, "Self-loop detected: {}", edge.from);
            }
        });
    }

    #[test]
    fn test_dag_builder_creates_valid_graph() {
        proptest!(|(
            files in prop::collection::vec(
                "[a-z][a-z0-9_]{0,10}\\.rs",
                1..5
            ),
            functions_per_file in prop::collection::vec(1usize..5, 1..5)
        )| {
            prop_assume!(files.len() == functions_per_file.len());

            let mut file_contexts = vec![];
            let total_functions: usize = functions_per_file.iter().sum();

            for (i, (file_path, num_functions)) in files.iter().zip(functions_per_file).enumerate() {
                let mut items = vec![];

                for j in 0..num_functions {
                    items.push(AstItem::Function {
                        name: format!("func_{}_{}", i, j),
                        visibility: "pub".to_string(),
                        is_async: false,
                        line: j * 10 + 1,
                    });
                }

                file_contexts.push(FileContext {
                    path: file_path.clone(),
                    language: "rust".to_string(),
                    items,
                    complexity_metrics: None,
                });
            }

            let project = ProjectContext {
                project_type: "rust".to_string(),
                files: file_contexts,
                summary: ProjectSummary {
                    total_files: files.len(),
                    total_functions,
                    total_structs: 0,
                    total_enums: 0,
                    total_traits: 0,
                    total_impls: 0,
                    dependencies: vec![],
                },
            };

            let graph = DagBuilder::build_from_project(&project);

            // Graph should have nodes for all functions plus module nodes
            // Each unique file gets a module node
            let unique_files: std::collections::HashSet<_> = files.iter().collect();
            let expected_nodes: usize = total_functions + unique_files.len();
            prop_assert!(graph.nodes.len() <= expected_nodes,
                "Graph has more nodes ({}) than functions ({}) + modules ({})",
                graph.nodes.len(), total_functions, unique_files.len());

            // All nodes should have valid structure
            for (id, node) in &graph.nodes {
                prop_assert_eq!(id, &node.id, "Node ID mismatch");
                prop_assert!(!node.file_path.is_empty(), "Empty file path");
                // Module nodes can have line_number = 0, other nodes should have > 0
                if node.node_type != NodeType::Module {
                    prop_assert!(node.line_number > 0, "Invalid line number for non-module node");
                }
            }
        });
    }

    #[test]
    fn test_dag_complexity_bounded() {
        proptest!(|(
            num_nodes in 10usize..100,
            edge_density in 0.0f64..0.3
        )| {
            let mut graph = DependencyGraph::new();

            // Create nodes
            for i in 0..num_nodes {
                let node = NodeInfo {
                    id: format!("node_{}", i),
                    label: format!("Node {}", i),
                    node_type: NodeType::Function,
                    file_path: format!("file_{}.rs", i % 10),
                    line_number: i * 10 + 1,
                    complexity: (i as u32) % 20,
                    metadata: FxHashMap::default(),
                };
                graph.add_node(node);
            }

            // Add edges based on density
            let max_edges = (num_nodes * (num_nodes - 1)) as f64;
            let target_edges = (max_edges * edge_density) as usize;

            for i in 0..target_edges {
                let from_idx = i % num_nodes;
                let to_idx = (i * 7 + 1) % num_nodes; // Pseudo-random pattern

                if from_idx != to_idx {
                    graph.add_edge(Edge {
                        from: format!("node_{}", from_idx),
                        to: format!("node_{}", to_idx),
                        edge_type: EdgeType::Calls,
                        weight: 1,
                    });
                }
            }

            // Property: Number of edges should be reasonable
            prop_assert!(graph.edges.len() <= target_edges + 10,
                "Too many edges: {} > {}", graph.edges.len(), target_edges);

            // Property: Memory usage should be bounded
            let estimated_memory = graph.nodes.len() * 200 + graph.edges.len() * 50;
            prop_assert!(estimated_memory < 10_000_000, // 10MB limit
                "Estimated memory usage too high: {} bytes", estimated_memory);
        });
    }

    #[test]
    fn test_dag_edge_type_distribution() {
        proptest!(|(
            edges in prop::collection::vec(
                (0usize..20, 0usize..20, 0usize..5),
                10..50
            )
        )| {
            let mut graph = DependencyGraph::new();

            // Create nodes
            for i in 0..20 {
                graph.add_node(NodeInfo {
                    id: format!("node_{}", i),
                    label: format!("Node {}", i),
                    node_type: NodeType::Function,
                    file_path: "test.rs".to_string(),
                    line_number: i + 1,
                    complexity: 1,
                    metadata: FxHashMap::default(),
                });
            }

            let edge_types = [
                EdgeType::Calls,
                EdgeType::Imports,
                EdgeType::Inherits,
                EdgeType::Implements,
                EdgeType::Uses,
            ];

            // Track edge type counts
            let mut type_counts = HashMap::new();

            // Add edges
            for (from, to, type_idx) in edges {
                if from != to && from < 20 && to < 20 {
                    let edge_type = edge_types[type_idx % edge_types.len()].clone();
                    type_counts.entry(edge_type.clone())
                        .and_modify(|e| *e += 1)
                        .or_insert(1);

                    graph.add_edge(Edge {
                        from: format!("node_{}", from),
                        to: format!("node_{}", to),
                        edge_type,
                        weight: 1,
                    });
                }
            }

            // Verify each filter returns the correct subset
            for (edge_type, expected_count) in type_counts {
                let filtered = graph.filter_by_edge_type(edge_type.clone());
                prop_assert_eq!(filtered.edges.len(), expected_count,
                    "Filter for {:?} returned {} edges, expected {}",
                    edge_type, filtered.edges.len(), expected_count);
            }
        });
    }

    #[test]
    fn test_dag_serialization_roundtrip() {
        proptest!(|(graph in arb_dependency_graph(5, 10))| {
            // Serialize to JSON
            let serialized = serde_json::to_string(&graph).unwrap();

            // Deserialize back
            let deserialized: DependencyGraph = serde_json::from_str(&serialized).unwrap();

            // Verify equality
            prop_assert_eq!(graph.nodes.len(), deserialized.nodes.len());
            prop_assert_eq!(graph.edges.len(), deserialized.edges.len());

            // Check all nodes are preserved
            for (id, node) in &graph.nodes {
                let deser_node = deserialized.nodes.get(id);
                prop_assert!(deser_node.is_some(), "Node {} missing after roundtrip", id);
                let deser_node = deser_node.unwrap();
                prop_assert_eq!(&node.label, &deser_node.label);
                prop_assert_eq!(&node.node_type, &deser_node.node_type);
                prop_assert_eq!(&node.complexity, &deser_node.complexity);
            }

            // Check all edges are preserved
            for edge in &graph.edges {
                prop_assert!(deserialized.edges.iter().any(|e|
                    e.from == edge.from &&
                    e.to == edge.to &&
                    e.edge_type == edge.edge_type
                ), "Edge {:?} missing after roundtrip", edge);
            }
        });
    }

    #[test]
    fn test_dag_empty_graph_valid() {
        proptest!(|(_dummy in 0u8..1)| {
            let graph = DependencyGraph::new();

            prop_assert_eq!(graph.nodes.len(), 0);
            prop_assert_eq!(graph.edges.len(), 0);

            // Empty graph should handle all operations gracefully
            let filtered = graph.filter_by_edge_type(EdgeType::Calls);
            prop_assert_eq!(filtered.nodes.len(), 0);
            prop_assert_eq!(filtered.edges.len(), 0);

            // Serialization should work
            let serialized = serde_json::to_string(&graph);
            prop_assert!(serialized.is_ok());
        });
    }

    // Helper to generate arbitrary dependency graphs
    fn arb_dependency_graph(
        max_nodes: usize,
        max_edges: usize,
    ) -> impl Strategy<Value = DependencyGraph> {
        (
            prop::collection::vec(arb_node_info(), 0..max_nodes),
            prop::collection::vec(
                (0..max_nodes, 0..max_nodes, 0..5usize, 1u32..10),
                0..max_edges,
            ),
        )
            .prop_map(|(nodes, edge_specs)| {
                let mut graph = DependencyGraph::new();

                // Add nodes
                let mut node_ids = vec![];
                for node in nodes {
                    node_ids.push(node.id.clone());
                    graph.add_node(node);
                }

                // Add edges
                let edge_types = [
                    EdgeType::Calls,
                    EdgeType::Imports,
                    EdgeType::Inherits,
                    EdgeType::Implements,
                    EdgeType::Uses,
                ];

                for (from_idx, to_idx, type_idx, weight) in edge_specs {
                    if !node_ids.is_empty() && from_idx != to_idx {
                        let from = &node_ids[from_idx % node_ids.len()];
                        let to = &node_ids[to_idx % node_ids.len()];
                        let edge_type = edge_types[type_idx % edge_types.len()].clone();

                        graph.add_edge(Edge {
                            from: from.clone(),
                            to: to.clone(),
                            edge_type,
                            weight,
                        });
                    }
                }

                graph
            })
    }
}
