    // ============================================================
    // Test Fixtures
    // ============================================================

    /// Create a function AstItem for testing
    fn create_function(name: &str, line: usize) -> AstItem {
        debug_assert!(!name.is_empty(), "name must not be empty");
        AstItem::Function {
            name: name.to_string(),
            visibility: "pub".to_string(),
            is_async: false,
            line,
        }
    }

    /// Create an async function AstItem for testing
    fn create_async_function(name: &str, line: usize) -> AstItem {
        debug_assert!(!name.is_empty(), "name must not be empty");
        AstItem::Function {
            name: name.to_string(),
            visibility: "pub".to_string(),
            is_async: true,
            line,
        }
    }

    /// Create a struct AstItem for testing
    fn create_struct(name: &str, line: usize) -> AstItem {
        debug_assert!(!name.is_empty(), "name must not be empty");
        AstItem::Struct {
            name: name.to_string(),
            visibility: "pub".to_string(),
            fields_count: 0,
            derives: vec![],
            line,
        }
    }

    /// Create a private function AstItem for testing
    fn create_private_function(name: &str, line: usize) -> AstItem {
        debug_assert!(!name.is_empty(), "name must not be empty");
        AstItem::Function {
            name: name.to_string(),
            visibility: "".to_string(),
            is_async: false,
            line,
        }
    }

    /// Build a simple call graph for testing
    fn build_simple_call_graph() -> ProjectContextGraph {
        debug_assert!(true, "contract: build_simple_call_graph");
        let mut graph = ProjectContextGraph::new();

        graph
            .add_item("main".to_string(), create_function("main", 1))
            .unwrap();
        graph
            .add_item("helper".to_string(), create_function("helper", 10))
            .unwrap();
        graph
            .add_item("utility".to_string(), create_function("utility", 20))
            .unwrap();

        graph.add_edge("main", "helper").unwrap();
        graph.add_edge("main", "utility").unwrap();
        graph.add_edge("helper", "utility").unwrap();

        graph
    }

    /// Build a larger graph with multiple components
    fn build_complex_graph() -> ProjectContextGraph {
        debug_assert!(true, "contract: build_complex_graph");
        let mut graph = ProjectContextGraph::new();

        // Component 1: Main entry points
        graph
            .add_item("entry1".to_string(), create_function("entry1", 1))
            .unwrap();
        graph
            .add_item("entry2".to_string(), create_function("entry2", 10))
            .unwrap();

        // Component 2: Services
        graph
            .add_item("service_a".to_string(), create_function("service_a", 100))
            .unwrap();
        graph
            .add_item("service_b".to_string(), create_function("service_b", 200))
            .unwrap();

        // Component 3: Utilities (shared)
        graph
            .add_item(
                "util_shared".to_string(),
                create_function("util_shared", 300),
            )
            .unwrap();

        // Edges
        graph.add_edge("entry1", "service_a").unwrap();
        graph.add_edge("entry2", "service_b").unwrap();
        graph.add_edge("service_a", "util_shared").unwrap();
        graph.add_edge("service_b", "util_shared").unwrap();
        graph.add_edge("entry1", "util_shared").unwrap();

        graph
    }
