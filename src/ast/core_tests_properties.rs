    // ==================== Property-Based Tests ====================

    proptest! {
        #[test]
        fn prop_byte_pos_roundtrip(pos: u32) {
            let byte_pos = BytePos(pos);
            prop_assert_eq!(byte_pos.0, pos);
            prop_assert_eq!(byte_pos.to_usize(), pos as usize);
        }

        #[test]
        fn prop_span_len_consistency(start in 0u32..1000, len in 0u32..1000) {
            let end = start.saturating_add(len);
            let span = Span::new(start, end);
            prop_assert_eq!(span.len(), end - start);
        }

        #[test]
        fn prop_span_contains_boundary(start in 0u32..1000, len in 1u32..1000) {
            let end = start.saturating_add(len);
            let span = Span::new(start, end);
            prop_assert!(span.contains(BytePos(start)));     // start is inclusive
            prop_assert!(!span.contains(BytePos(end)));       // end is exclusive
            if start > 0 {
                prop_assert!(!span.contains(BytePos(start - 1)));
            }
        }

        #[test]
        fn prop_span_is_empty_consistency(start in 0u32..1000, end in 0u32..1000) {
            let span = Span::new(start, end);
            prop_assert_eq!(span.is_empty(), start >= end);
        }

        #[test]
        fn prop_location_contains_reflexive(
            start in 0u32..1000,
            end in 0u32..2000
        ) {
            let file = PathBuf::from("test.rs");
            let loc = Location::new(file, start, end);
            prop_assert!(loc.contains(&loc));
        }

        #[test]
        fn prop_location_overlaps_reflexive(
            start in 0u32..1000,
            len in 1u32..1000
        ) {
            let file = PathBuf::from("test.rs");
            let end = start.saturating_add(len);
            let loc = Location::new(file, start, end);
            prop_assert!(loc.overlaps(&loc));
        }

        #[test]
        fn prop_qualified_name_roundtrip(
            path_parts in proptest::collection::vec("[a-z]+", 0..5),
            name in "[a-z]+"
        ) {
            let qname = QualifiedName::new(path_parts.clone(), name.clone());
            let qualified_str = qname.to_qualified_string();
            let parsed = QualifiedName::from_string(&qualified_str).expect("parse should succeed");

            prop_assert_eq!(parsed.name, name);
            prop_assert_eq!(parsed.module_path, path_parts);
        }

        #[test]
        fn prop_node_flags_set_unset_idempotent(flag1 in 0u8..8, flag2 in 0u8..8) {
            let flag_value1 = 1u8 << flag1;
            let flag_value2 = 1u8 << flag2;

            let mut flags = NodeFlags::new();
            flags.set(flag_value1);
            flags.set(flag_value1); // Set twice
            prop_assert!(flags.has(flag_value1));

            flags.unset(flag_value2);
            flags.unset(flag_value2); // Unset twice
            if flag_value1 != flag_value2 {
                prop_assert!(flags.has(flag_value1));
            }
        }

        #[test]
        fn prop_unified_ast_node_complexity_roundtrip(complexity in 0u32..=u32::MAX) {
            let mut node = UnifiedAstNode::new(
                AstKind::Function(FunctionKind::Regular),
                Language::Rust
            );
            node.set_complexity(complexity);
            prop_assert_eq!(node.complexity(), complexity);
        }

        #[test]
        fn prop_column_store_push_returns_sequential_keys(count in 1usize..100) {
            let mut store: ColumnStore<i32> = ColumnStore::new(count);
            let keys: Vec<_> = (0..count).map(|i| store.push(i as i32)).collect();

            for (expected_key, actual_key) in keys.iter().enumerate() {
                prop_assert_eq!(*actual_key, expected_key as NodeKey);
            }
        }

        #[test]
        fn prop_ast_dag_generation_monotonic(node_count in 1usize..50) {
            let mut dag = AstDag::new();
            let mut prev_gen = dag.generation();

            for _ in 0..node_count {
                dag.add_node(UnifiedAstNode::new(
                    AstKind::Function(FunctionKind::Regular),
                    Language::Rust,
                ));
                let new_gen = dag.generation();
                prop_assert!(new_gen > prev_gen);
                prev_gen = new_gen;
            }
        }
    }

    // ==================== Edge Case Tests ====================

    #[test]
    fn test_span_max_values() {
        let span = Span::new(u32::MAX - 1, u32::MAX);
        assert_eq!(span.len(), 1);
        assert!(!span.is_empty());
    }

    #[test]
    fn test_byte_pos_from_usize_overflow() {
        // Test truncation behavior for values larger than u32::MAX
        let large_value = (u32::MAX as usize) + 100;
        let pos = BytePos::from_usize(large_value);
        // Truncated to u32
        assert_eq!(pos.0, (large_value as u32));
    }

    #[test]
    fn test_qualified_name_empty_segments() {
        // Edge case: path with only empty segments gets filtered
        let result = QualifiedName::from_string("::");
        assert!(result.is_err());
    }

    #[test]
    fn test_location_zero_length() {
        let loc = Location::new(PathBuf::from("test.rs"), 100, 100);
        assert_eq!(loc.span.len(), 0);
        assert!(loc.span.is_empty());
    }

    #[test]
    fn test_proof_annotation_with_all_optional_fields() {
        let annotation = ProofAnnotation {
            annotation_id: Uuid::new_v4(),
            property_proven: PropertyType::FunctionalCorrectness("spec".to_string()),
            specification_id: Some("spec_id".to_string()),
            method: VerificationMethod::FormalProof {
                prover: "Coq".to_string(),
            },
            tool_name: "coqc".to_string(),
            tool_version: "8.15".to_string(),
            confidence_level: ConfidenceLevel::High,
            assumptions: vec!["assumption1".to_string(), "assumption2".to_string()],
            evidence_type: EvidenceType::ProofScriptReference {
                uri: "file://proof.v".to_string(),
            },
            evidence_location: Some("/proofs/memory.v".to_string()),
            date_verified: Utc::now(),
        };

        let json = serde_json::to_string(&annotation).expect("serialization failed");
        let deserialized: ProofAnnotation =
            serde_json::from_str(&json).expect("deserialization failed");

        assert_eq!(annotation.specification_id, deserialized.specification_id);
        assert_eq!(annotation.assumptions.len(), deserialized.assumptions.len());
        assert_eq!(annotation.evidence_location, deserialized.evidence_location);
    }

    #[test]
    fn test_unified_ast_node_all_function_kinds_are_functions() {
        let function_kinds = [
            FunctionKind::Regular,
            FunctionKind::Method,
            FunctionKind::Constructor,
            FunctionKind::Getter,
            FunctionKind::Setter,
            FunctionKind::Lambda,
            FunctionKind::Closure,
            FunctionKind::Destructor,
            FunctionKind::Operator,
        ];

        for kind in function_kinds {
            let node = UnifiedAstNode::new(AstKind::Function(kind.clone()), Language::Rust);
            assert!(
                node.is_function(),
                "FunctionKind::{:?} should be recognized as function",
                kind
            );
        }
    }

    #[test]
    fn test_unified_ast_node_all_type_definitions() {
        let class_kinds = [
            ClassKind::Regular,
            ClassKind::Abstract,
            ClassKind::Interface,
            ClassKind::Trait,
            ClassKind::Enum,
            ClassKind::Struct,
        ];

        for kind in class_kinds {
            let node = UnifiedAstNode::new(AstKind::Class(kind.clone()), Language::Rust);
            assert!(
                node.is_type_definition(),
                "ClassKind::{:?} should be recognized as type definition",
                kind
            );
        }

        let type_kinds = [
            TypeKind::Primitive,
            TypeKind::Array,
            TypeKind::Alias,
            TypeKind::Interface,
        ];

        for kind in type_kinds {
            let node = UnifiedAstNode::new(AstKind::Type(kind.clone()), Language::TypeScript);
            assert!(
                node.is_type_definition(),
                "TypeKind::{:?} should be recognized as type definition",
                kind
            );
        }

        let module_kinds = [ModuleKind::File, ModuleKind::Namespace, ModuleKind::Package];

        for kind in module_kinds {
            let node = UnifiedAstNode::new(AstKind::Module(kind.clone()), Language::Python);
            assert!(
                node.is_type_definition(),
                "ModuleKind::{:?} should be recognized as type definition",
                kind
            );
        }
    }

    #[test]
    fn test_column_store_with_complex_type() {
        let mut store: ColumnStore<UnifiedAstNode> = ColumnStore::new(10);

        let node1 = UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Rust);
        let node2 = UnifiedAstNode::new(AstKind::Class(ClassKind::Struct), Language::TypeScript);

        let key1 = store.push(node1);
        let key2 = store.push(node2);

        assert!(store.get(key1).expect("node1 should exist").is_function());
        assert!(store
            .get(key2)
            .expect("node2 should exist")
            .is_type_definition());
    }

    #[test]
    fn test_language_parsers_default() {
        let parsers = LanguageParsers::default();
        // Just verify it can be created - it's a placeholder struct
        let _ = parsers;
    }

    #[test]
    fn test_proof_map_type_alias() {
        // ProofMap is HashMap<Location, Vec<ProofAnnotation>>
        let mut proof_map: ProofMap = HashMap::new();

        let location = Location::new(PathBuf::from("test.rs"), 0, 100);
        let annotation = create_test_proof_annotation();

        proof_map.insert(location.clone(), vec![annotation]);

        assert!(proof_map.contains_key(&location));
        assert_eq!(proof_map.get(&location).map(|v| v.len()), Some(1));
    }
