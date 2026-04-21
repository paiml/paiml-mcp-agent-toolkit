// Extracted from unified_ast_types.rs for file health (CB-040)
#[cfg_attr(coverage_nightly, coverage(off))]
mod unified_ast_types_tests {
    use super::*;

    // ============================================================================
    // Language Tests
    // ============================================================================

    #[test]
    fn test_language_variants() {
        assert_eq!(Language::Rust as u8, 0);
        assert_eq!(Language::TypeScript as u8, 1);
        assert_eq!(Language::JavaScript as u8, 2);
        assert_eq!(Language::Python as u8, 3);
        assert_eq!(Language::Markdown as u8, 4);
        assert_eq!(Language::WebAssembly as u8, 15);
    }

    #[test]
    fn test_language_equality() {
        assert_eq!(Language::Rust, Language::Rust);
        assert_ne!(Language::Rust, Language::Python);
    }

    #[test]
    fn test_language_serialization() {
        let lang = Language::TypeScript;
        let json = serde_json::to_string(&lang).unwrap();
        let deserialized: Language = serde_json::from_str(&json).unwrap();
        assert_eq!(lang, deserialized);
    }

    // ============================================================================
    // NodeFlags Tests
    // ============================================================================

    #[test]
    fn test_node_flags_new() {
        let flags = NodeFlags::new();
        assert!(!flags.has(NodeFlags::ASYNC));
        assert!(!flags.has(NodeFlags::EXPORTED));
        assert!(!flags.has(NodeFlags::PRIVATE));
    }

    #[test]
    fn test_node_flags_set() {
        let mut flags = NodeFlags::new();
        flags.set(NodeFlags::ASYNC);
        assert!(flags.has(NodeFlags::ASYNC));
        assert!(!flags.has(NodeFlags::EXPORTED));
    }

    #[test]
    fn test_node_flags_set_multiple() {
        let mut flags = NodeFlags::new();
        flags.set(NodeFlags::ASYNC | NodeFlags::EXPORTED);
        assert!(flags.has(NodeFlags::ASYNC));
        assert!(flags.has(NodeFlags::EXPORTED));
        assert!(!flags.has(NodeFlags::PRIVATE));
    }

    #[test]
    fn test_node_flags_unset() {
        let mut flags = NodeFlags::new();
        flags.set(NodeFlags::ASYNC | NodeFlags::EXPORTED);
        flags.unset(NodeFlags::ASYNC);
        assert!(!flags.has(NodeFlags::ASYNC));
        assert!(flags.has(NodeFlags::EXPORTED));
    }

    #[test]
    fn test_node_flags_default() {
        let flags = NodeFlags::default();
        assert!(!flags.has(NodeFlags::ASYNC));
    }

    #[test]
    fn test_node_flags_all_flags() {
        let mut flags = NodeFlags::new();
        flags.set(NodeFlags::ASYNC);
        flags.set(NodeFlags::GENERATOR);
        flags.set(NodeFlags::ABSTRACT);
        flags.set(NodeFlags::STATIC);
        flags.set(NodeFlags::CONST);
        flags.set(NodeFlags::EXPORTED);
        flags.set(NodeFlags::PRIVATE);
        flags.set(NodeFlags::DEPRECATED);

        assert!(flags.has(NodeFlags::ASYNC));
        assert!(flags.has(NodeFlags::GENERATOR));
        assert!(flags.has(NodeFlags::ABSTRACT));
        assert!(flags.has(NodeFlags::STATIC));
        assert!(flags.has(NodeFlags::CONST));
        assert!(flags.has(NodeFlags::EXPORTED));
        assert!(flags.has(NodeFlags::PRIVATE));
        assert!(flags.has(NodeFlags::DEPRECATED));
    }

    // ============================================================================
    // FunctionKind Tests
    // ============================================================================

    #[test]
    fn test_function_kind_variants() {
        assert_eq!(FunctionKind::Regular, FunctionKind::Regular);
        assert_ne!(FunctionKind::Regular, FunctionKind::Method);
        assert_ne!(FunctionKind::Constructor, FunctionKind::Destructor);
    }

    #[test]
    fn test_function_kind_serialization() {
        let kind = FunctionKind::Lambda;
        let json = serde_json::to_string(&kind).unwrap();
        let deserialized: FunctionKind = serde_json::from_str(&json).unwrap();
        assert_eq!(kind, deserialized);
    }

    // ============================================================================
    // ClassKind Tests
    // ============================================================================

    #[test]
    fn test_class_kind_variants() {
        assert_eq!(ClassKind::Regular, ClassKind::Regular);
        assert_ne!(ClassKind::Interface, ClassKind::Trait);
        assert_ne!(ClassKind::Struct, ClassKind::Enum);
    }

    #[test]
    fn test_class_kind_serialization() {
        let kind = ClassKind::Abstract;
        let json = serde_json::to_string(&kind).unwrap();
        let deserialized: ClassKind = serde_json::from_str(&json).unwrap();
        assert_eq!(kind, deserialized);
    }

    // ============================================================================
    // BytePos Tests
    // ============================================================================

    #[test]
    fn test_byte_pos_creation() {
        let pos = BytePos(100);
        assert_eq!(pos.0, 100);
    }

    #[test]
    fn test_byte_pos_to_usize() {
        let pos = BytePos(42);
        assert_eq!(pos.to_usize(), 42);
    }

    #[test]
    fn test_byte_pos_from_usize() {
        let pos = BytePos::from_usize(256);
        assert_eq!(pos.0, 256);
    }

    #[test]
    fn test_byte_pos_ordering() {
        let pos1 = BytePos(10);
        let pos2 = BytePos(20);
        assert!(pos1 < pos2);
        assert!(pos2 > pos1);
        assert!(pos1 <= pos1);
        assert!(pos1 >= pos1);
    }

    #[test]
    fn test_byte_pos_equality() {
        let pos1 = BytePos(50);
        let pos2 = BytePos(50);
        assert_eq!(pos1, pos2);
    }

    // ============================================================================
    // Span Tests
    // ============================================================================

    #[test]
    fn test_span_new() {
        let span = Span::new(10, 50);
        assert_eq!(span.start.0, 10);
        assert_eq!(span.end.0, 50);
    }

    #[test]
    fn test_span_len() {
        let span = Span::new(10, 50);
        assert_eq!(span.len(), 40);
    }

    #[test]
    fn test_span_is_empty() {
        let empty = Span::new(10, 10);
        assert!(empty.is_empty());

        let non_empty = Span::new(10, 20);
        assert!(!non_empty.is_empty());

        let invalid = Span::new(20, 10); // end < start
        assert!(invalid.is_empty());
    }

    #[test]
    fn test_span_contains() {
        let span = Span::new(10, 50);
        assert!(span.contains(BytePos(10)));
        assert!(span.contains(BytePos(25)));
        assert!(span.contains(BytePos(49)));
        assert!(!span.contains(BytePos(50)));
        assert!(!span.contains(BytePos(9)));
    }

    // ============================================================================
    // Location Tests
    // ============================================================================

    #[test]
    fn test_location_new() {
        let loc = Location::new(PathBuf::from("test.rs"), 100, 200);
        assert_eq!(loc.file_path, PathBuf::from("test.rs"));
        assert_eq!(loc.span.start.0, 100);
        assert_eq!(loc.span.end.0, 200);
    }

    #[test]
    fn test_location_contains() {
        let outer = Location::new(PathBuf::from("test.rs"), 0, 100);
        let inner = Location::new(PathBuf::from("test.rs"), 10, 50);
        let other_file = Location::new(PathBuf::from("other.rs"), 10, 50);

        assert!(outer.contains(&inner));
        assert!(!inner.contains(&outer));
        assert!(!outer.contains(&other_file));
    }

    #[test]
    fn test_location_overlaps() {
        let loc1 = Location::new(PathBuf::from("test.rs"), 0, 50);
        let loc2 = Location::new(PathBuf::from("test.rs"), 25, 75);
        let loc3 = Location::new(PathBuf::from("test.rs"), 100, 150);
        let other_file = Location::new(PathBuf::from("other.rs"), 0, 100);

        assert!(loc1.overlaps(&loc2));
        assert!(loc2.overlaps(&loc1));
        assert!(!loc1.overlaps(&loc3));
        assert!(!loc1.overlaps(&other_file));
    }

    // ============================================================================
    // QualifiedName Tests
    // ============================================================================

    #[test]
    fn test_qualified_name_new() {
        let qname = QualifiedName::new(
            vec!["std".to_string(), "collections".to_string()],
            "HashMap".to_string(),
        );
        assert_eq!(qname.module_path, vec!["std", "collections"]);
        assert_eq!(qname.name, "HashMap");
        assert!(qname.disambiguator.is_none());
    }

    #[test]
    fn test_qualified_name_with_disambiguator() {
        let qname = QualifiedName::new(vec![], "function".to_string())
            .with_disambiguator(1);
        assert_eq!(qname.disambiguator, Some(1));
    }

    #[test]
    fn test_qualified_name_from_string() {
        let qname = QualifiedName::from_string("std::collections::HashMap").unwrap();
        assert_eq!(qname.module_path, vec!["std", "collections"]);
        assert_eq!(qname.name, "HashMap");
    }

    #[test]
    fn test_qualified_name_from_string_simple() {
        let qname = QualifiedName::from_string("main").unwrap();
        assert!(qname.module_path.is_empty());
        assert_eq!(qname.name, "main");
    }

    #[test]
    fn test_qualified_name_from_string_empty() {
        let result = QualifiedName::from_string("");
        assert!(result.is_err());
    }

    #[test]
    fn test_qualified_name_to_qualified_string() {
        let qname = QualifiedName::new(
            vec!["crate".to_string(), "module".to_string()],
            "function".to_string(),
        );
        assert_eq!(qname.to_qualified_string(), "crate::module::function");
    }

    #[test]
    fn test_qualified_name_to_qualified_string_with_disambiguator() {
        let qname = QualifiedName::new(vec!["mod".to_string()], "func".to_string())
            .with_disambiguator(2);
        assert_eq!(qname.to_qualified_string(), "mod::func#2");
    }

    #[test]
    fn test_qualified_name_to_qualified_string_simple() {
        let qname = QualifiedName::new(vec![], "main".to_string());
        assert_eq!(qname.to_qualified_string(), "main");
    }

    #[test]
    fn test_qualified_name_from_str() {
        let qname: QualifiedName = "std::io::Error".parse().unwrap();
        assert_eq!(qname.module_path, vec!["std", "io"]);
        assert_eq!(qname.name, "Error");
    }

    #[test]
    fn test_qualified_name_display() {
        let qname = QualifiedName::new(
            vec!["std".to_string()],
            "Vec".to_string(),
        );
        assert_eq!(format!("{}", qname), "std::Vec");
    }

    // ============================================================================
    // AstKind Tests
    // ============================================================================

    #[test]
    fn test_ast_kind_variants() {
        let func = AstKind::Function(FunctionKind::Regular);
        let class = AstKind::Class(ClassKind::Struct);
        assert_ne!(func, class);
    }

    #[test]
    fn test_ast_kind_serialization() {
        let kind = AstKind::Function(FunctionKind::Method);
        let json = serde_json::to_string(&kind).unwrap();
        let deserialized: AstKind = serde_json::from_str(&json).unwrap();
        assert_eq!(kind, deserialized);
    }

    // ============================================================================
    // ConfidenceLevel Tests
    // ============================================================================

    #[test]
    fn test_confidence_level_ordering() {
        assert!(ConfidenceLevel::Low < ConfidenceLevel::Medium);
        assert!(ConfidenceLevel::Medium < ConfidenceLevel::High);
    }

    #[test]
    fn test_confidence_level_values() {
        assert_eq!(ConfidenceLevel::Low as u8, 1);
        assert_eq!(ConfidenceLevel::Medium as u8, 2);
        assert_eq!(ConfidenceLevel::High as u8, 3);
    }

    // ============================================================================
    // ColumnStore Tests
    // ============================================================================

    #[test]
    fn test_column_store_new() {
        let store: ColumnStore<i32> = ColumnStore::new(100);
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_column_store_push() {
        let mut store: ColumnStore<i32> = ColumnStore::new(10);
        let key0 = store.push(42);
        let key1 = store.push(100);

        assert_eq!(key0, 0);
        assert_eq!(key1, 1);
        assert_eq!(store.len(), 2);
    }

    #[test]
    fn test_column_store_get() {
        let mut store: ColumnStore<String> = ColumnStore::new(10);
        store.push("hello".to_string());
        store.push("world".to_string());

        assert_eq!(store.get(0), Some(&"hello".to_string()));
        assert_eq!(store.get(1), Some(&"world".to_string()));
        assert_eq!(store.get(2), None);
    }

    #[test]
    fn test_column_store_get_mut() {
        let mut store: ColumnStore<i32> = ColumnStore::new(10);
        store.push(10);

        if let Some(val) = store.get_mut(0) {
            *val = 20;
        }

        assert_eq!(store.get(0), Some(&20));
    }

    #[test]
    fn test_column_store_iter() {
        let mut store: ColumnStore<i32> = ColumnStore::new(10);
        store.push(1);
        store.push(2);
        store.push(3);

        let sum: i32 = store.iter().sum();
        assert_eq!(sum, 6);
    }

    // ============================================================================
    // UnifiedAstNode Tests
    // ============================================================================

    #[test]
    fn test_unified_ast_node_new() {
        let node = UnifiedAstNode::new(
            AstKind::Function(FunctionKind::Regular),
            Language::Rust,
        );
        assert!(node.is_function());
        assert!(!node.is_type_definition());
        assert_eq!(node.lang, Language::Rust);
        assert_eq!(node.parent, 0);
        assert_eq!(node.source_range, 0..0);
    }

    #[test]
    fn test_unified_ast_node_is_function() {
        let func = UnifiedAstNode::new(
            AstKind::Function(FunctionKind::Method),
            Language::TypeScript,
        );
        assert!(func.is_function());

        let class = UnifiedAstNode::new(
            AstKind::Class(ClassKind::Regular),
            Language::TypeScript,
        );
        assert!(!class.is_function());
    }

    #[test]
    fn test_unified_ast_node_is_type_definition() {
        let class = UnifiedAstNode::new(
            AstKind::Class(ClassKind::Interface),
            Language::TypeScript,
        );
        assert!(class.is_type_definition());

        let module = UnifiedAstNode::new(
            AstKind::Module(ModuleKind::File),
            Language::Python,
        );
        assert!(module.is_type_definition());

        let func = UnifiedAstNode::new(
            AstKind::Function(FunctionKind::Regular),
            Language::Rust,
        );
        assert!(!func.is_type_definition());
    }

    #[test]
    fn test_unified_ast_node_complexity() {
        let mut node = UnifiedAstNode::new(
            AstKind::Function(FunctionKind::Regular),
            Language::Rust,
        );

        assert_eq!(node.complexity(), 0);

        node.set_complexity(25);
        assert_eq!(node.complexity(), 25);
    }

    #[test]
    fn test_unified_ast_node_proof_annotations() {
        let mut node = UnifiedAstNode::new(
            AstKind::Function(FunctionKind::Regular),
            Language::Rust,
        );

        assert!(!node.has_proof_annotations());
        assert!(node.proof_annotations().is_empty());

        let annotation = ProofAnnotation {
            annotation_id: uuid::Uuid::new_v4(),
            property_proven: PropertyType::MemorySafety,
            specification_id: None,
            method: VerificationMethod::BorrowChecker,
            tool_name: "rustc".to_string(),
            tool_version: "1.70.0".to_string(),
            confidence_level: ConfidenceLevel::High,
            assumptions: vec![],
            evidence_type: EvidenceType::ImplicitTypeSystemGuarantee,
            evidence_location: None,
            date_verified: chrono::Utc::now(),
        };

        node.add_proof_annotation(annotation);

        assert!(node.has_proof_annotations());
        assert_eq!(node.proof_annotations().len(), 1);
    }

    #[test]
    fn test_unified_ast_node_location() {
        let mut node = UnifiedAstNode::new(
            AstKind::Function(FunctionKind::Regular),
            Language::Rust,
        );
        node.source_range = 100..200;

        let loc = node.location(Path::new("src/main.rs"));
        assert_eq!(loc.file_path, PathBuf::from("src/main.rs"));
        assert_eq!(loc.span.start.0, 100);
        assert_eq!(loc.span.end.0, 200);
    }

    // ============================================================================
    // AstDag Tests
    // ============================================================================

    #[test]
    fn test_ast_dag_new() {
        let dag = AstDag::new();
        assert!(dag.nodes.is_empty());
        assert_eq!(dag.generation(), 0);
    }

    #[test]
    fn test_ast_dag_default() {
        let dag = AstDag::default();
        assert!(dag.nodes.is_empty());
    }

    #[test]
    fn test_ast_dag_add_node() {
        let mut dag = AstDag::new();
        let initial_gen = dag.generation();

        let node = UnifiedAstNode::new(
            AstKind::Function(FunctionKind::Regular),
            Language::Rust,
        );
        let key = dag.add_node(node);

        assert_eq!(dag.nodes.len(), 1);
        assert_eq!(dag.generation(), initial_gen + 1);
        assert!(dag.nodes.get(key).is_some());
    }

    #[test]
    fn test_ast_dag_mark_clean() {
        let mut dag = AstDag::new();
        let node = UnifiedAstNode::new(
            AstKind::Function(FunctionKind::Regular),
            Language::Rust,
        );
        let key = dag.add_node(node);

        // Initially dirty
        assert!(dag.dirty_nodes().any(|k| k == key));

        dag.mark_clean(key);

        // No longer dirty
        assert!(!dag.dirty_nodes().any(|k| k == key));
    }

    #[test]
    fn test_ast_dag_dirty_nodes() {
        let mut dag = AstDag::new();

        let keys: Vec<_> = (0..3).map(|_| {
            dag.add_node(UnifiedAstNode::new(
                AstKind::Function(FunctionKind::Regular),
                Language::Rust,
            ))
        }).collect();

        assert_eq!(dag.dirty_nodes().count(), 3);

        dag.mark_clean(keys[1]);

        assert_eq!(dag.dirty_nodes().count(), 2);
    }

    // ============================================================================
    // NodeMetadata Tests
    // ============================================================================

    #[test]
    fn test_node_metadata_default() {
        let metadata = NodeMetadata::default();
        // SAFETY: Test-only: NodeMetadata is a Copy union initialized to zero; reading `raw` is sound.
        unsafe {
            assert_eq!(metadata.raw, 0);
        }
    }

    #[test]
    fn test_node_metadata_clone() {
        let metadata = NodeMetadata { complexity: 42 };
        // Invoke the explicit Clone impl (not the implicit Copy) so the
        // `impl Clone for NodeMetadata` body is exercised for coverage.
        let cloned: NodeMetadata = Clone::clone(&metadata);
        // SAFETY: Test-only: reading the same union field (`complexity`) that was just written, no UB.
        unsafe {
            assert_eq!(metadata.complexity, cloned.complexity);
        }
    }

    // ============================================================================
    // LanguageParsers Tests
    // ============================================================================

    #[test]
    fn test_language_parsers_default() {
        let _parsers = LanguageParsers::default();
        // Just verify it can be instantiated
    }

    // ============================================================================
    // PropertyType Tests
    // ============================================================================

    #[test]
    fn test_property_type_equality() {
        assert_eq!(PropertyType::MemorySafety, PropertyType::MemorySafety);
        assert_ne!(PropertyType::MemorySafety, PropertyType::ThreadSafety);
    }

    #[test]
    fn test_property_type_serialization() {
        let prop = PropertyType::Termination;
        let json = serde_json::to_string(&prop).unwrap();
        let deserialized: PropertyType = serde_json::from_str(&json).unwrap();
        assert_eq!(prop, deserialized);
    }

    // ============================================================================
    // VerificationMethod Tests
    // ============================================================================

    #[test]
    fn test_verification_method_equality() {
        assert_eq!(
            VerificationMethod::BorrowChecker,
            VerificationMethod::BorrowChecker
        );
    }

    #[test]
    fn test_verification_method_serialization() {
        let method = VerificationMethod::BorrowChecker;
        let json = serde_json::to_string(&method).unwrap();
        let deserialized: VerificationMethod = serde_json::from_str(&json).unwrap();
        assert_eq!(method, deserialized);
    }

    // ============================================================================
    // EvidenceType Tests
    // ============================================================================

    #[test]
    fn test_evidence_type_equality() {
        assert_eq!(
            EvidenceType::ImplicitTypeSystemGuarantee,
            EvidenceType::ImplicitTypeSystemGuarantee
        );
    }

    #[test]
    fn test_evidence_type_serialization() {
        let evidence = EvidenceType::ProofScriptReference {
            uri: "proof.v".to_string(),
        };
        let json = serde_json::to_string(&evidence).unwrap();
        let deserialized: EvidenceType = serde_json::from_str(&json).unwrap();
        assert_eq!(evidence, deserialized);
    }

    // ============================================================================
    // RelativeLocation Tests
    // ============================================================================

    #[test]
    fn test_relative_location_function() {
        let loc = RelativeLocation::Function {
            name: "main".to_string(),
            module: Some("app".to_string()),
        };

        let json = serde_json::to_string(&loc).unwrap();
        let deserialized: RelativeLocation = serde_json::from_str(&json).unwrap();
        if let RelativeLocation::Function { name, module } = deserialized {
            assert_eq!(name, "main");
            assert_eq!(module, Some("app".to_string()));
        } else {
            panic!("Wrong variant");
        }
    }

    #[test]
    fn test_relative_location_span() {
        let loc = RelativeLocation::Span { start: 100, end: 200 };
        let json = serde_json::to_string(&loc).unwrap();
        let deserialized: RelativeLocation = serde_json::from_str(&json).unwrap();
        if let RelativeLocation::Span { start, end } = deserialized {
            assert_eq!(start, 100);
            assert_eq!(end, 200);
        } else {
            panic!("Wrong variant");
        }
    }
}
