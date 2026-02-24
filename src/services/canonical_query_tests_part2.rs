    // --- Tests for infer_component_relationships ---

    fn make_component(id: &str, nodes: Vec<&str>) -> Component {
        Component {
            id: id.to_string(),
            label: id.to_string(),
            nodes: nodes.into_iter().map(|s| s.to_string()).collect(),
            complexity: 0.0,
            loc: 0,
            functions: 0,
        }
    }

    fn make_call_edge(from: &str, to: &str, edge_type: CallEdgeType, weight: u32) -> CallEdge {
        CallEdge {
            from: from.to_string(),
            to: to.to_string(),
            edge_type,
            weight,
        }
    }

    #[test]
    fn test_infer_component_relationships_empty() {
        let components: Vec<Component> = vec![];
        let call_graph = CallGraph::default();
        let edges = infer_component_relationships(&components, &call_graph).unwrap();
        assert!(edges.is_empty());
    }

    #[test]
    fn test_infer_component_relationships_no_cross_component_edges() {
        let components = vec![
            make_component("comp_a", vec!["fn_a1", "fn_a2"]),
            make_component("comp_b", vec!["fn_b1", "fn_b2"]),
        ];
        // Edges within same component should not produce component edges
        let call_graph = CallGraph {
            nodes: vec![],
            edges: vec![
                make_call_edge("fn_a1", "fn_a2", CallEdgeType::FunctionCall, 1),
                make_call_edge("fn_b1", "fn_b2", CallEdgeType::MethodCall, 2),
            ],
        };
        let edges = infer_component_relationships(&components, &call_graph).unwrap();
        assert!(
            edges.is_empty(),
            "Intra-component edges should be filtered out"
        );
    }

    #[test]
    fn test_infer_component_relationships_cross_component_function_call() {
        let components = vec![
            make_component("comp_a", vec!["fn_a1"]),
            make_component("comp_b", vec!["fn_b1"]),
        ];
        let call_graph = CallGraph {
            nodes: vec![],
            edges: vec![make_call_edge(
                "fn_a1",
                "fn_b1",
                CallEdgeType::FunctionCall,
                3,
            )],
        };
        let edges = infer_component_relationships(&components, &call_graph).unwrap();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].from, "comp_a");
        assert_eq!(edges[0].to, "comp_b");
        assert_eq!(edges[0].weight, 3);
        assert_eq!(edges[0].edge_type, ComponentEdgeType::Call);
    }

    #[test]
    fn test_infer_component_relationships_method_call_maps_to_call() {
        let components = vec![
            make_component("svc", vec!["service::handle"]),
            make_component("db", vec!["db::query"]),
        ];
        let call_graph = CallGraph {
            nodes: vec![],
            edges: vec![make_call_edge(
                "service::handle",
                "db::query",
                CallEdgeType::MethodCall,
                1,
            )],
        };
        let edges = infer_component_relationships(&components, &call_graph).unwrap();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].edge_type, ComponentEdgeType::Call);
    }

    #[test]
    fn test_infer_component_relationships_module_import() {
        let components = vec![
            make_component("cli", vec!["cli::main"]),
            make_component("lib", vec!["lib::utils"]),
        ];
        let call_graph = CallGraph {
            nodes: vec![],
            edges: vec![make_call_edge(
                "cli::main",
                "lib::utils",
                CallEdgeType::ModuleImport,
                1,
            )],
        };
        let edges = infer_component_relationships(&components, &call_graph).unwrap();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].edge_type, ComponentEdgeType::Import);
    }

    #[test]
    fn test_infer_component_relationships_trait_impl() {
        let components = vec![
            make_component("impl_mod", vec!["impl_mod::MyStruct"]),
            make_component("trait_mod", vec!["trait_mod::MyTrait"]),
        ];
        let call_graph = CallGraph {
            nodes: vec![],
            edges: vec![make_call_edge(
                "impl_mod::MyStruct",
                "trait_mod::MyTrait",
                CallEdgeType::TraitImpl,
                1,
            )],
        };
        let edges = infer_component_relationships(&components, &call_graph).unwrap();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].edge_type, ComponentEdgeType::Inheritance);
    }

    #[test]
    fn test_infer_component_relationships_struct_instantiation() {
        let components = vec![
            make_component("factory", vec!["factory::create"]),
            make_component("model", vec!["model::Widget"]),
        ];
        let call_graph = CallGraph {
            nodes: vec![],
            edges: vec![make_call_edge(
                "factory::create",
                "model::Widget",
                CallEdgeType::StructInstantiation,
                2,
            )],
        };
        let edges = infer_component_relationships(&components, &call_graph).unwrap();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].edge_type, ComponentEdgeType::Composition);
        assert_eq!(edges[0].weight, 2);
    }

    #[test]
    fn test_infer_component_relationships_weight_aggregation() {
        // Multiple edges between same components of same type should aggregate weights
        let components = vec![
            make_component("comp_a", vec!["fn_a1", "fn_a2"]),
            make_component("comp_b", vec!["fn_b1", "fn_b2"]),
        ];
        let call_graph = CallGraph {
            nodes: vec![],
            edges: vec![
                make_call_edge("fn_a1", "fn_b1", CallEdgeType::FunctionCall, 2),
                make_call_edge("fn_a2", "fn_b2", CallEdgeType::FunctionCall, 5),
            ],
        };
        let edges = infer_component_relationships(&components, &call_graph).unwrap();
        // Both are FunctionCall from comp_a to comp_b, should be aggregated
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].weight, 7);
        assert_eq!(edges[0].edge_type, ComponentEdgeType::Call);
    }

    #[test]
    fn test_infer_component_relationships_different_types_not_aggregated() {
        let components = vec![
            make_component("comp_a", vec!["fn_a1", "fn_a2"]),
            make_component("comp_b", vec!["fn_b1"]),
        ];
        let call_graph = CallGraph {
            nodes: vec![],
            edges: vec![
                make_call_edge("fn_a1", "fn_b1", CallEdgeType::FunctionCall, 1),
                make_call_edge("fn_a2", "fn_b1", CallEdgeType::ModuleImport, 1),
            ],
        };
        let edges = infer_component_relationships(&components, &call_graph).unwrap();
        // Different edge types should NOT be aggregated
        assert_eq!(edges.len(), 2);
    }

    #[test]
    fn test_infer_component_relationships_unknown_nodes_ignored() {
        // Edges referencing nodes not in any component should be skipped
        let components = vec![make_component("comp_a", vec!["fn_a1"])];
        let call_graph = CallGraph {
            nodes: vec![],
            edges: vec![
                make_call_edge("fn_a1", "fn_unknown", CallEdgeType::FunctionCall, 1),
                make_call_edge("fn_unknown", "fn_a1", CallEdgeType::FunctionCall, 1),
            ],
        };
        let edges = infer_component_relationships(&components, &call_graph).unwrap();
        assert!(
            edges.is_empty(),
            "Edges with unknown nodes should be ignored"
        );
    }

    #[test]
    fn test_infer_component_relationships_bidirectional() {
        let components = vec![
            make_component("comp_a", vec!["fn_a1"]),
            make_component("comp_b", vec!["fn_b1"]),
        ];
        let call_graph = CallGraph {
            nodes: vec![],
            edges: vec![
                make_call_edge("fn_a1", "fn_b1", CallEdgeType::FunctionCall, 1),
                make_call_edge("fn_b1", "fn_a1", CallEdgeType::FunctionCall, 1),
            ],
        };
        let edges = infer_component_relationships(&components, &call_graph).unwrap();
        // Bidirectional edges: comp_a->comp_b and comp_b->comp_a are different keys
        assert_eq!(edges.len(), 2);
    }

    // --- Tests for aggregate_component_metrics ---

    #[test]
    fn test_aggregate_component_metrics_empty_components() {
        let components: Vec<Component> = vec![];
        let complexity_map = FxHashMap::default();
        let metrics = aggregate_component_metrics(&components, &complexity_map).unwrap();
        assert!(metrics.is_empty());
    }

    #[test]
    fn test_aggregate_component_metrics_no_matching_complexity() {
        // Component nodes not in complexity_map
        let components = vec![make_component("orphan", vec!["fn_orphan"])];
        let complexity_map = FxHashMap::default();
        let metrics = aggregate_component_metrics(&components, &complexity_map).unwrap();
        let m = metrics.get("orphan").unwrap();
        assert_eq!(m.total_complexity, 0.0);
        assert_eq!(m.avg_complexity, 0.0);
        assert_eq!(m.max_complexity, 0.0);
        assert_eq!(m.total_loc, 0);
        assert_eq!(m.function_count, 0);
    }

    #[test]
    fn test_aggregate_component_metrics_single_function() {
        let components = vec![make_component("comp", vec!["fn_a"])];
        let mut complexity_map = FxHashMap::default();
        complexity_map.insert(
            "fn_a".to_string(),
            crate::services::complexity::ComplexityMetrics::new(10, 5, 2, 50),
        );
        let metrics = aggregate_component_metrics(&components, &complexity_map).unwrap();
        let m = metrics.get("comp").unwrap();
        assert_eq!(m.total_complexity, 10.0);
        assert_eq!(m.avg_complexity, 10.0);
        assert_eq!(m.max_complexity, 10.0);
        assert_eq!(m.total_loc, 50);
        assert_eq!(m.function_count, 1);
    }

    #[test]
    fn test_aggregate_component_metrics_multiple_functions() {
        let components = vec![make_component("comp", vec!["fn_a", "fn_b", "fn_c"])];
        let mut complexity_map = FxHashMap::default();
        // cyclomatic values: 6, 12, 3
        complexity_map.insert(
            "fn_a".to_string(),
            crate::services::complexity::ComplexityMetrics::new(6, 2, 1, 20),
        );
        complexity_map.insert(
            "fn_b".to_string(),
            crate::services::complexity::ComplexityMetrics::new(12, 8, 3, 80),
        );
        complexity_map.insert(
            "fn_c".to_string(),
            crate::services::complexity::ComplexityMetrics::new(3, 1, 1, 15),
        );
        let metrics = aggregate_component_metrics(&components, &complexity_map).unwrap();
        let m = metrics.get("comp").unwrap();
        assert_eq!(m.total_complexity, 21.0); // 6 + 12 + 3
        assert!((m.avg_complexity - 7.0).abs() < f64::EPSILON); // 21 / 3
        assert_eq!(m.max_complexity, 12.0);
        assert_eq!(m.total_loc, 115); // 20 + 80 + 15
        assert_eq!(m.function_count, 3);
    }

    #[test]
    fn test_aggregate_component_metrics_partial_coverage() {
        // Some nodes have complexity data, some do not
        let components = vec![make_component("comp", vec!["fn_a", "fn_missing", "fn_b"])];
        let mut complexity_map = FxHashMap::default();
        complexity_map.insert(
            "fn_a".to_string(),
            crate::services::complexity::ComplexityMetrics::new(4, 2, 1, 30),
        );
        complexity_map.insert(
            "fn_b".to_string(),
            crate::services::complexity::ComplexityMetrics::new(8, 4, 2, 60),
        );
        // fn_missing is NOT in the map
        let metrics = aggregate_component_metrics(&components, &complexity_map).unwrap();
        let m = metrics.get("comp").unwrap();
        assert_eq!(m.total_complexity, 12.0); // 4 + 8
        assert!((m.avg_complexity - 6.0).abs() < f64::EPSILON); // 12 / 2
        assert_eq!(m.max_complexity, 8.0);
        assert_eq!(m.total_loc, 90); // 30 + 60
        assert_eq!(m.function_count, 2); // only 2 had metrics
    }

    #[test]
    fn test_aggregate_component_metrics_multiple_components() {
        let components = vec![
            make_component("comp_a", vec!["fn_a"]),
            make_component("comp_b", vec!["fn_b"]),
        ];
        let mut complexity_map = FxHashMap::default();
        complexity_map.insert(
            "fn_a".to_string(),
            crate::services::complexity::ComplexityMetrics::new(5, 3, 1, 25),
        );
        complexity_map.insert(
            "fn_b".to_string(),
            crate::services::complexity::ComplexityMetrics::new(15, 10, 4, 100),
        );
        let metrics = aggregate_component_metrics(&components, &complexity_map).unwrap();
        assert_eq!(metrics.len(), 2);

        let ma = metrics.get("comp_a").unwrap();
        assert_eq!(ma.total_complexity, 5.0);
        assert_eq!(ma.total_loc, 25);

        let mb = metrics.get("comp_b").unwrap();
        assert_eq!(mb.total_complexity, 15.0);
        assert_eq!(mb.max_complexity, 15.0);
        assert_eq!(mb.total_loc, 100);
    }

    #[test]
    fn test_aggregate_component_metrics_zero_complexity() {
        let components = vec![make_component("comp", vec!["fn_trivial"])];
        let mut complexity_map = FxHashMap::default();
        complexity_map.insert(
            "fn_trivial".to_string(),
            crate::services::complexity::ComplexityMetrics::new(0, 0, 0, 1),
        );
        let metrics = aggregate_component_metrics(&components, &complexity_map).unwrap();
        let m = metrics.get("comp").unwrap();
        assert_eq!(m.total_complexity, 0.0);
        assert_eq!(m.avg_complexity, 0.0);
        assert_eq!(m.max_complexity, 0.0);
        assert_eq!(m.total_loc, 1);
        assert_eq!(m.function_count, 1);
    }
