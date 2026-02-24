    #[test]
    fn test_call_graph_creation() {
        let mut graph = CallGraph::default();

        graph.nodes.push(CallNode {
            id: "1".to_string(),
            name: "main".to_string(),
            module_path: "app".to_string(),
            node_type: CallNodeType::Function,
        });

        graph.nodes.push(CallNode {
            id: "2".to_string(),
            name: "helper".to_string(),
            module_path: "app::utils".to_string(),
            node_type: CallNodeType::Function,
        });

        graph.edges.push(CallEdge {
            from: "1".to_string(),
            to: "2".to_string(),
            edge_type: CallEdgeType::FunctionCall,
            weight: 1,
        });

        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 1);
    }

    #[test]
    fn test_call_node_types() {
        let node_types = vec![
            CallNodeType::Function,
            CallNodeType::Method,
            CallNodeType::Struct,
            CallNodeType::Module,
            CallNodeType::Trait,
        ];

        for node_type in node_types {
            let node = CallNode {
                id: "test".to_string(),
                name: "test".to_string(),
                module_path: "test".to_string(),
                node_type,
            };

            match node.node_type {
                CallNodeType::Function => {}
                CallNodeType::Method => {}
                CallNodeType::Struct => {}
                CallNodeType::Module => {}
                CallNodeType::Trait => {}
            }
        }
    }

    #[test]
    fn test_call_edge_types() {
        let edge_types = vec![
            CallEdgeType::FunctionCall,
            CallEdgeType::MethodCall,
            CallEdgeType::StructInstantiation,
            CallEdgeType::TraitImpl,
            CallEdgeType::ModuleImport,
        ];

        for edge_type in edge_types {
            let edge = CallEdge {
                from: "a".to_string(),
                to: "b".to_string(),
                edge_type,
                weight: 1,
            };

            assert_eq!(edge.from, "a");
            assert_eq!(edge.to, "b");
            assert_eq!(edge.weight, 1);
        }
    }

    #[test]
    fn test_graph_metadata() {
        let metadata = GraphMetadata {
            nodes: 10,
            edges: 15,
            max_depth: 5,
            timestamp: Utc::now(),
            query_version: "1.0".to_string(),
            analysis_time_ms: 100,
        };

        assert_eq!(metadata.nodes, 10);
        assert_eq!(metadata.edges, 15);
        assert_eq!(metadata.max_depth, 5);
        assert_eq!(metadata.query_version, "1.0");
        assert_eq!(metadata.analysis_time_ms, 100);
    }

    #[test]
    fn test_query_result() {
        let result = QueryResult {
            diagram: "graph TD\n  A --> B".to_string(),
            metadata: GraphMetadata {
                nodes: 2,
                edges: 1,
                max_depth: 2,
                timestamp: Utc::now(),
                query_version: "1.0".to_string(),
                analysis_time_ms: 50,
            },
        };

        assert!(result.diagram.contains("graph TD"));
        assert_eq!(result.metadata.nodes, 2);
    }

    #[test]
    fn test_analysis_context() {
        let context = AnalysisContext {
            project_path: PathBuf::from("/test/project"),
            ast_dag: crate::models::dag::DependencyGraph::new(),
            call_graph: CallGraph::default(),
            complexity_map: FxHashMap::default(),
            churn_analysis: None,
        };

        assert_eq!(context.project_path, PathBuf::from("/test/project"));
        assert!(context.churn_analysis.is_none());
    }

    // Test for the trait implementation
    struct TestQuery;

    impl CanonicalQuery for TestQuery {
        fn query_id(&self) -> &'static str {
            "test_query"
        }

        fn execute(&self, _ctx: &AnalysisContext) -> Result<QueryResult> {
            Ok(QueryResult {
                diagram: "test".to_string(),
                metadata: GraphMetadata {
                    nodes: 0,
                    edges: 0,
                    max_depth: 0,
                    timestamp: Utc::now(),
                    query_version: "1.0".to_string(),
                    analysis_time_ms: 10,
                },
            })
        }
    }

    #[test]
    fn test_canonical_query_trait() {
        let query = TestQuery;
        let path = Path::new("/test/project");

        assert_eq!(query.query_id(), "test_query");
        assert_eq!(query.cache_key(path), "test_query:/test/project");
    }

    #[test]
    fn test_component_edge_type_hash() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let edge_type = ComponentEdgeType::Import;
        let mut hasher = DefaultHasher::new();
        edge_type.hash(&mut hasher);
        let hash1 = hasher.finish();

        let mut hasher2 = DefaultHasher::new();
        ComponentEdgeType::Import.hash(&mut hasher2);
        let hash2 = hasher2.finish();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_component_edge_type_equality() {
        assert_eq!(ComponentEdgeType::Import, ComponentEdgeType::Import);
        assert_ne!(ComponentEdgeType::Import, ComponentEdgeType::Composition);
        assert_eq!(
            ComponentEdgeType::Inheritance,
            ComponentEdgeType::Inheritance
        );
    }

