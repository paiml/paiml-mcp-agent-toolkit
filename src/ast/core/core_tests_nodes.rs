    // ==================== RelativeLocation Tests ====================

    #[test]
    fn test_relative_location_function() {
        let loc = RelativeLocation::Function {
            name: "test_fn".to_string(),
            module: Some("test_mod".to_string()),
        };

        let json = serde_json::to_string(&loc).expect("serialization failed");
        let deserialized: RelativeLocation =
            serde_json::from_str(&json).expect("deserialization failed");

        match deserialized {
            RelativeLocation::Function { name, module } => {
                assert_eq!(name, "test_fn");
                assert_eq!(module, Some("test_mod".to_string()));
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_relative_location_function_no_module() {
        let loc = RelativeLocation::Function {
            name: "test_fn".to_string(),
            module: None,
        };

        let json = serde_json::to_string(&loc).expect("serialization failed");
        assert!(!json.contains("module") || json.contains("null"));
    }

    #[test]
    fn test_relative_location_symbol() {
        let loc = RelativeLocation::Symbol {
            qualified_name: "crate::module::Type::method".to_string(),
        };

        let json = serde_json::to_string(&loc).expect("serialization failed");
        let deserialized: RelativeLocation =
            serde_json::from_str(&json).expect("deserialization failed");

        match deserialized {
            RelativeLocation::Symbol { qualified_name } => {
                assert_eq!(qualified_name, "crate::module::Type::method");
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_relative_location_span() {
        let loc = RelativeLocation::Span {
            start: 100,
            end: 200,
        };

        let json = serde_json::to_string(&loc).expect("serialization failed");
        let deserialized: RelativeLocation =
            serde_json::from_str(&json).expect("deserialization failed");

        match deserialized {
            RelativeLocation::Span { start, end } => {
                assert_eq!(start, 100);
                assert_eq!(end, 200);
            }
            _ => panic!("Wrong variant"),
        }
    }

    // ==================== NodeMetadata Tests ====================

    #[test]
    fn test_node_metadata_default() {
        let metadata = NodeMetadata::default();
        // SAFETY: accessing raw field for test
        unsafe {
            assert_eq!(metadata.raw, 0);
        }
    }

    #[test]
    fn test_node_metadata_clone() {
        let metadata = NodeMetadata { complexity: 42 };
        let cloned = metadata.clone();
        // SAFETY: accessing complexity field for test
        unsafe {
            assert_eq!(cloned.complexity, 42);
        }
    }

    #[test]
    fn test_node_metadata_copy() {
        let metadata = NodeMetadata { hash: 0xDEADBEEF };
        let copied = metadata;
        // SAFETY: accessing hash field for test
        unsafe {
            assert_eq!(copied.hash, 0xDEADBEEF);
        }
    }

    #[test]
    fn test_node_metadata_union_views() {
        let mut metadata = NodeMetadata::default();
        metadata.complexity = 100;

        // SAFETY: all union fields share the same memory
        unsafe {
            assert_eq!(metadata.raw, 100);
            assert_eq!(metadata.hash, 100);
            assert_eq!(metadata.flags, 100);
        }
    }

    // ==================== ProofAnnotation Tests ====================

    fn create_test_proof_annotation() -> ProofAnnotation {
        ProofAnnotation {
            annotation_id: Uuid::new_v4(),
            property_proven: PropertyType::MemorySafety,
            specification_id: Some("spec_001".to_string()),
            method: VerificationMethod::BorrowChecker,
            tool_name: "rustc".to_string(),
            tool_version: "1.70.0".to_string(),
            confidence_level: ConfidenceLevel::High,
            assumptions: vec!["no unsafe".to_string()],
            evidence_type: EvidenceType::ImplicitTypeSystemGuarantee,
            evidence_location: Some("/path/to/evidence".to_string()),
            date_verified: Utc::now(),
        }
    }

    #[test]
    fn test_proof_annotation_serialization() {
        let annotation = create_test_proof_annotation();
        let json = serde_json::to_string(&annotation).expect("serialization failed");
        let deserialized: ProofAnnotation =
            serde_json::from_str(&json).expect("deserialization failed");

        assert_eq!(annotation.property_proven, deserialized.property_proven);
        assert_eq!(annotation.tool_name, deserialized.tool_name);
        assert_eq!(annotation.confidence_level, deserialized.confidence_level);
    }

    #[test]
    fn test_proof_annotation_optional_fields() {
        let annotation = ProofAnnotation {
            annotation_id: Uuid::new_v4(),
            property_proven: PropertyType::ThreadSafety,
            specification_id: None,
            method: VerificationMethod::AbstractInterpretation,
            tool_name: "tool".to_string(),
            tool_version: "1.0".to_string(),
            confidence_level: ConfidenceLevel::Low,
            assumptions: vec![],
            evidence_type: EvidenceType::ImplicitTypeSystemGuarantee,
            evidence_location: None,
            date_verified: Utc::now(),
        };

        let json = serde_json::to_string(&annotation).expect("serialization failed");
        // Optional fields should be skipped if None/empty
        assert!(!json.contains("specification_id") || json.contains("null"));
    }

    // ==================== UnifiedAstNode Tests ====================

    #[test]
    fn test_unified_ast_node_new() {
        let node = UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Rust);

        assert!(matches!(
            node.kind,
            AstKind::Function(FunctionKind::Regular)
        ));
        assert_eq!(node.lang, Language::Rust);
        assert_eq!(node.parent, 0);
        assert_eq!(node.first_child, 0);
        assert_eq!(node.next_sibling, 0);
        assert_eq!(node.source_range, 0..0);
        assert_eq!(node.semantic_hash, 0);
        assert_eq!(node.structural_hash, 0);
        assert_eq!(node.name_vector, 0);
        assert!(!node.has_proof_annotations());
    }

    #[test]
    fn test_unified_ast_node_is_function() {
        let func = UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Rust);
        let method = UnifiedAstNode::new(AstKind::Function(FunctionKind::Method), Language::Python);
        let class = UnifiedAstNode::new(AstKind::Class(ClassKind::Struct), Language::Rust);

        assert!(func.is_function());
        assert!(method.is_function());
        assert!(!class.is_function());
    }

    #[test]
    fn test_unified_ast_node_is_type_definition() {
        let struct_node = UnifiedAstNode::new(AstKind::Class(ClassKind::Struct), Language::Rust);
        let interface =
            UnifiedAstNode::new(AstKind::Class(ClassKind::Interface), Language::TypeScript);
        let type_alias = UnifiedAstNode::new(AstKind::Type(TypeKind::Alias), Language::Rust);
        let module = UnifiedAstNode::new(AstKind::Module(ModuleKind::File), Language::Python);
        let func = UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Rust);

        assert!(struct_node.is_type_definition());
        assert!(interface.is_type_definition());
        assert!(type_alias.is_type_definition());
        assert!(module.is_type_definition());
        assert!(!func.is_type_definition());
    }

    #[test]
    fn test_unified_ast_node_complexity() {
        let mut node =
            UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Rust);

        assert_eq!(node.complexity(), 0);

        node.set_complexity(42);
        assert_eq!(node.complexity(), 42);

        node.set_complexity(u32::MAX);
        assert_eq!(node.complexity(), u32::MAX);
    }

    #[test]
    fn test_unified_ast_node_proof_annotations() {
        let mut node =
            UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Rust);

        assert!(!node.has_proof_annotations());
        assert_eq!(node.proof_annotations().len(), 0);

        let annotation1 = create_test_proof_annotation();
        node.add_proof_annotation(annotation1);

        assert!(node.has_proof_annotations());
        assert_eq!(node.proof_annotations().len(), 1);

        let annotation2 = create_test_proof_annotation();
        node.add_proof_annotation(annotation2);

        assert_eq!(node.proof_annotations().len(), 2);
    }

    #[test]
    fn test_unified_ast_node_location() {
        let mut node =
            UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Rust);
        node.source_range = 100..200;

        let location = node.location(Path::new("src/main.rs"));
        assert_eq!(location.file_path, PathBuf::from("src/main.rs"));
        assert_eq!(location.span.start.0, 100);
        assert_eq!(location.span.end.0, 200);
    }

    #[test]
    fn test_unified_ast_node_debug() {
        let node = UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Rust);
        let debug_str = format!("{:?}", node);

        assert!(debug_str.contains("UnifiedAstNode"));
        assert!(debug_str.contains("kind"));
        assert!(debug_str.contains("lang"));
    }

    #[test]
    fn test_unified_ast_node_clone() {
        let mut node =
            UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Rust);
        node.semantic_hash = 12345;
        node.set_complexity(100);

        let cloned = node.clone();

        assert_eq!(cloned.semantic_hash, 12345);
        assert_eq!(cloned.complexity(), 100);
    }

    // ==================== ColumnStore Tests ====================

    #[test]
    fn test_column_store_new() {
        let store: ColumnStore<i32> = ColumnStore::new(100);
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_column_store_push_and_get() {
        let mut store: ColumnStore<String> = ColumnStore::new(10);

        let key0 = store.push("first".to_string());
        let key1 = store.push("second".to_string());

        assert_eq!(key0, 0);
        assert_eq!(key1, 1);
        assert_eq!(store.len(), 2);
        assert!(!store.is_empty());

        assert_eq!(store.get(key0), Some(&"first".to_string()));
        assert_eq!(store.get(key1), Some(&"second".to_string()));
        assert_eq!(store.get(999), None);
    }

    #[test]
    fn test_column_store_get_mut() {
        let mut store: ColumnStore<i32> = ColumnStore::new(10);
        let key = store.push(42);

        if let Some(value) = store.get_mut(key) {
            *value = 100;
        }

        assert_eq!(store.get(key), Some(&100));
        assert!(store.get_mut(999).is_none());
    }

    #[test]
    fn test_column_store_iter() {
        let mut store: ColumnStore<i32> = ColumnStore::new(10);
        store.push(1);
        store.push(2);
        store.push(3);

        let sum: i32 = store.iter().sum();
        assert_eq!(sum, 6);

        let collected: Vec<_> = store.iter().collect();
        assert_eq!(collected, vec![&1, &2, &3]);
    }

    // ==================== AstDag Tests ====================

    #[test]
    fn test_ast_dag_new() {
        let dag = AstDag::new();
        assert_eq!(dag.nodes.len(), 0);
        assert!(dag.nodes.is_empty());
        assert_eq!(dag.generation(), 0);
    }

    #[test]
    fn test_ast_dag_default() {
        let dag = AstDag::default();
        assert_eq!(dag.nodes.len(), 0);
        assert_eq!(dag.generation(), 0);
    }

    #[test]
    fn test_ast_dag_add_node() {
        let mut dag = AstDag::new();

        let node = UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Rust);
        let key = dag.add_node(node);

        assert_eq!(key, 0);
        assert_eq!(dag.nodes.len(), 1);
        assert_eq!(dag.generation(), 1);
    }

    #[test]
    fn test_ast_dag_dirty_nodes() {
        let mut dag = AstDag::new();

        let node1 = UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Rust);
        let node2 = UnifiedAstNode::new(AstKind::Class(ClassKind::Struct), Language::TypeScript);

        let key1 = dag.add_node(node1);
        let key2 = dag.add_node(node2);

        let dirty: Vec<_> = dag.dirty_nodes().collect();
        assert_eq!(dirty.len(), 2);
        assert!(dirty.contains(&key1));
        assert!(dirty.contains(&key2));
    }

    #[test]
    fn test_ast_dag_mark_clean() {
        let mut dag = AstDag::new();

        let node = UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Rust);
        let key = dag.add_node(node);

        assert!(dag.dirty_nodes().any(|k| k == key));

        dag.mark_clean(key);

        assert!(!dag.dirty_nodes().any(|k| k == key));
        // Node still exists
        assert!(dag.nodes.get(key).is_some());
    }

    #[test]
    fn test_ast_dag_generation_increments() {
        let mut dag = AstDag::new();

        assert_eq!(dag.generation(), 0);

        dag.add_node(UnifiedAstNode::new(
            AstKind::Function(FunctionKind::Regular),
            Language::Rust,
        ));
        assert_eq!(dag.generation(), 1);

        dag.add_node(UnifiedAstNode::new(
            AstKind::Class(ClassKind::Struct),
            Language::Python,
        ));
        assert_eq!(dag.generation(), 2);

        // mark_clean does not change generation
        dag.mark_clean(0);
        assert_eq!(dag.generation(), 2);
    }

    #[test]
    fn test_ast_dag_multiple_operations() {
        let mut dag = AstDag::new();

        // Add several nodes
        let keys: Vec<_> = (0..5)
            .map(|_| {
                dag.add_node(UnifiedAstNode::new(
                    AstKind::Function(FunctionKind::Regular),
                    Language::Rust,
                ))
            })
            .collect();

        assert_eq!(dag.nodes.len(), 5);
        assert_eq!(dag.generation(), 5);
        assert_eq!(dag.dirty_nodes().count(), 5);

        // Clean some
        dag.mark_clean(keys[0]);
        dag.mark_clean(keys[2]);
        dag.mark_clean(keys[4]);

        assert_eq!(dag.dirty_nodes().count(), 2);
        let dirty: Vec<_> = dag.dirty_nodes().collect();
        assert!(dirty.contains(&keys[1]));
        assert!(dirty.contains(&keys[3]));
    }

    // ==================== INVALID_NODE_KEY Tests ====================

    #[test]
    fn test_invalid_node_key_constant() {
        assert_eq!(INVALID_NODE_KEY, u32::MAX);
    }

    // ==================== ClassKind Tests ====================

    #[test]
    fn test_class_kind_all_variants() {
        let variants = [
            ClassKind::Regular,
            ClassKind::Abstract,
            ClassKind::Interface,
            ClassKind::Trait,
            ClassKind::Enum,
            ClassKind::Struct,
        ];
        assert_eq!(variants.len(), 6);
    }

    #[test]
    fn test_class_kind_serialization() {
        for kind in [
            ClassKind::Regular,
            ClassKind::Abstract,
            ClassKind::Interface,
            ClassKind::Trait,
            ClassKind::Enum,
            ClassKind::Struct,
        ] {
            let json = serde_json::to_string(&kind).expect("serialization failed");
            let deserialized: ClassKind =
                serde_json::from_str(&json).expect("deserialization failed");
            assert_eq!(kind, deserialized);
        }
    }

