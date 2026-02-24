    // ==========================================================================
    // Modifier tests
    // ==========================================================================

    #[test]
    fn test_has_modifier() {
        let mut node = UnifiedNode::new(NodeKind::Function, "test", Language::Java);
        node.attributes
            .insert("modifier:abstract".to_string(), "true".to_string());
        node.attributes
            .insert("modifier:static".to_string(), "true".to_string());
        node.attributes
            .insert("modifier:final".to_string(), "true".to_string());

        assert!(node.has_modifier("abstract"));
        assert!(node.has_modifier("static"));
        assert!(node.has_modifier("final"));
        assert!(!node.has_modifier("synchronized"));
    }

    #[test]
    fn test_is_abstract() {
        let mut node = UnifiedNode::new(NodeKind::Class, "AbstractClass", Language::Java);
        assert!(!node.is_abstract());

        node.attributes
            .insert("modifier:abstract".to_string(), "true".to_string());
        assert!(node.is_abstract());
    }

    #[test]
    fn test_is_static() {
        let mut node = UnifiedNode::new(NodeKind::Method, "staticMethod", Language::Java);
        assert!(!node.is_static());

        node.attributes
            .insert("modifier:static".to_string(), "true".to_string());
        assert!(node.is_static());
    }

    #[test]
    fn test_is_final() {
        let mut node = UnifiedNode::new(NodeKind::Class, "FinalClass", Language::Java);
        assert!(!node.is_final());

        node.attributes
            .insert("modifier:final".to_string(), "true".to_string());
        assert!(node.is_final());
    }

    #[test]
    fn test_access_none() {
        let node = UnifiedNode::new(NodeKind::Function, "test", Language::Rust);
        assert!(node.access().is_none());
    }

    #[test]
    fn test_access_some() {
        let mut node = UnifiedNode::new(NodeKind::Function, "test", Language::Rust);
        node.attributes
            .insert("access".to_string(), "private".to_string());
        assert_eq!(node.access(), Some("private"));
    }

    // ==========================================================================
    // Child/Parent relationship tests
    // ==========================================================================

    #[test]
    fn test_add_child() {
        let mut node = UnifiedNode::new(NodeKind::Class, "Parent", Language::Java);
        assert!(node.children.is_empty());

        node.add_child("child1".to_string());
        assert_eq!(node.children.len(), 1);
        assert_eq!(node.children[0], "child1");

        node.add_child("child2".to_string());
        node.add_child("child3".to_string());
        assert_eq!(node.children.len(), 3);
    }

    #[test]
    fn test_set_parent() {
        let mut node = UnifiedNode::new(NodeKind::Method, "child", Language::Java);
        assert!(node.parent.is_none());

        node.set_parent("parent_id".to_string());
        assert_eq!(node.parent, Some("parent_id".to_string()));

        // Setting again should overwrite
        node.set_parent("new_parent".to_string());
        assert_eq!(node.parent, Some("new_parent".to_string()));
    }

    // ==========================================================================
    // Reference tests
    // ==========================================================================

    #[test]
    fn test_add_reference_all_kinds() {
        let mut node = UnifiedNode::new(NodeKind::Class, "Test", Language::Java);

        let reference_kinds = vec![
            ReferenceKind::Inherits,
            ReferenceKind::Implements,
            ReferenceKind::Calls,
            ReferenceKind::Uses,
            ReferenceKind::Creates,
            ReferenceKind::Imports,
            ReferenceKind::Annotates,
            ReferenceKind::DependsOn,
        ];

        for (i, kind) in reference_kinds.iter().enumerate() {
            node.add_reference(*kind, format!("target_{}", i), Some(format!("id_{}", i)));
        }

        assert_eq!(node.references.len(), 8);

        // Test get_references_by_kind
        assert_eq!(
            node.get_references_by_kind(ReferenceKind::Inherits).len(),
            1
        );
        assert_eq!(
            node.get_references_by_kind(ReferenceKind::Implements).len(),
            1
        );
        assert_eq!(node.get_references_by_kind(ReferenceKind::Calls).len(), 1);
    }

    #[test]
    fn test_add_reference_without_target_id() {
        let mut node = UnifiedNode::new(NodeKind::Class, "Test", Language::Java);
        node.add_reference(ReferenceKind::Inherits, "BaseClass".to_string(), None);

        assert_eq!(node.references.len(), 1);
        assert_eq!(node.references[0].target_id, "");
        assert_eq!(node.references[0].target_name, "BaseClass");
        assert!(node.references[0].target_language.is_none());
    }

    #[test]
    fn test_get_references_by_kind_empty() {
        let node = UnifiedNode::new(NodeKind::Class, "Test", Language::Java);
        assert!(node
            .get_references_by_kind(ReferenceKind::Inherits)
            .is_empty());
    }

    #[test]
    fn test_get_references_by_kind_multiple() {
        let mut node = UnifiedNode::new(NodeKind::Class, "Test", Language::Java);
        node.add_reference(ReferenceKind::Implements, "Interface1".to_string(), None);
        node.add_reference(ReferenceKind::Implements, "Interface2".to_string(), None);
        node.add_reference(ReferenceKind::Implements, "Interface3".to_string(), None);
        node.add_reference(ReferenceKind::Inherits, "BaseClass".to_string(), None);

        let implements = node.get_references_by_kind(ReferenceKind::Implements);
        assert_eq!(implements.len(), 3);

        let inherits = node.get_references_by_kind(ReferenceKind::Inherits);
        assert_eq!(inherits.len(), 1);
    }

    // ==========================================================================
    // TypeInfo tests
    // ==========================================================================

    #[test]
    fn test_set_type_info() {
        let mut node = UnifiedNode::new(NodeKind::Field, "myField", Language::Java);
        assert!(node.type_info.is_none());

        let type_info = TypeInfo {
            name: "String".to_string(),
            fqn: "java.lang.String".to_string(),
            type_parameters: vec![],
            is_primitive: false,
            is_collection: false,
            is_nullable: true,
            original_type_string: "String?".to_string(),
        };

        node.set_type_info(type_info.clone());

        assert!(node.type_info.is_some());
        let ti = node.type_info.as_ref().unwrap();
        assert_eq!(ti.name, "String");
        assert_eq!(ti.fqn, "java.lang.String");
        assert!(!ti.is_primitive);
        assert!(ti.is_nullable);
    }

    #[test]
    fn test_type_info_with_generics() {
        let inner_type = TypeInfo {
            name: "String".to_string(),
            fqn: "java.lang.String".to_string(),
            type_parameters: vec![],
            is_primitive: false,
            is_collection: false,
            is_nullable: false,
            original_type_string: "String".to_string(),
        };

        let type_info = TypeInfo {
            name: "List".to_string(),
            fqn: "java.util.List".to_string(),
            type_parameters: vec![inner_type],
            is_primitive: false,
            is_collection: true,
            is_nullable: false,
            original_type_string: "List<String>".to_string(),
        };

        assert!(type_info.is_collection);
        assert_eq!(type_info.type_parameters.len(), 1);
        assert_eq!(type_info.type_parameters[0].name, "String");
    }

    // ==========================================================================
    // Metadata tests
    // ==========================================================================

    #[test]
    fn test_add_metadata() {
        let mut node = UnifiedNode::new(NodeKind::Function, "test", Language::Python);
        assert!(node.metadata.is_empty());

        node.add_metadata("decorator", "@pytest.fixture");
        node.add_metadata("docstring", "Test function");

        assert_eq!(node.metadata.len(), 2);
        assert_eq!(
            node.metadata.get("decorator"),
            Some(&"@pytest.fixture".to_string())
        );
        assert_eq!(
            node.metadata.get("docstring"),
            Some(&"Test function".to_string())
        );
    }

    #[test]
    fn test_add_metadata_overwrite() {
        let mut node = UnifiedNode::new(NodeKind::Function, "test", Language::Python);

        node.add_metadata("key", "value1");
        assert_eq!(node.metadata.get("key"), Some(&"value1".to_string()));

        node.add_metadata("key", "value2");
        assert_eq!(node.metadata.get("key"), Some(&"value2".to_string()));
    }

    // ==========================================================================
    // SourcePosition tests
    // ==========================================================================

    #[test]
    fn test_source_position_default() {
        let pos = SourcePosition::default();
        assert_eq!(pos.start_line, 0);
        assert_eq!(pos.start_col, 0);
        assert_eq!(pos.end_line, 0);
        assert_eq!(pos.end_col, 0);
    }

    #[test]
    fn test_source_position_clone() {
        let pos = SourcePosition {
            start_line: 10,
            start_col: 5,
            end_line: 15,
            end_col: 20,
        };
        let cloned = pos.clone();
        assert_eq!(pos.start_line, cloned.start_line);
        assert_eq!(pos.start_col, cloned.start_col);
        assert_eq!(pos.end_line, cloned.end_line);
        assert_eq!(pos.end_col, cloned.end_col);
    }

    // ==========================================================================
    // ReferenceKind tests
    // ==========================================================================

    #[test]
    fn test_reference_kind_ordering() {
        // ReferenceKind derives PartialOrd and Ord
        assert!(ReferenceKind::Inherits < ReferenceKind::Implements);
        assert!(ReferenceKind::Implements < ReferenceKind::Calls);

        let mut kinds = vec![
            ReferenceKind::DependsOn,
            ReferenceKind::Inherits,
            ReferenceKind::Calls,
        ];
        kinds.sort();
        assert_eq!(kinds[0], ReferenceKind::Inherits);
    }

    #[test]
    fn test_reference_kind_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(ReferenceKind::Inherits);
        set.insert(ReferenceKind::Implements);
        set.insert(ReferenceKind::Inherits); // Duplicate

        assert_eq!(set.len(), 2);
    }

    // ==========================================================================
    // Serialization tests
    // ==========================================================================

    #[test]
    fn test_unified_node_serialize() {
        let node = UnifiedNode::new(NodeKind::Function, "test_func", Language::Rust);
        let json = serde_json::to_string(&node).unwrap();

        assert!(json.contains("test_func"));
        assert!(json.contains("function"));
        assert!(json.contains("Rust"));
    }

    #[test]
    fn test_unified_node_deserialize() {
        let node = UnifiedNode::new(NodeKind::Function, "test_func", Language::Rust);
        let json = serde_json::to_string(&node).unwrap();
        let deserialized: UnifiedNode = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, "test_func");
        assert_eq!(deserialized.kind, NodeKind::Function);
        assert_eq!(deserialized.language, Language::Rust);
    }

    #[test]
    fn test_node_reference_serialize() {
        let reference = NodeReference {
            kind: ReferenceKind::Inherits,
            target_id: "id_123".to_string(),
            target_name: "BaseClass".to_string(),
            target_language: Some(Language::Java),
        };

        let json = serde_json::to_string(&reference).unwrap();
        let deserialized: NodeReference = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.kind, ReferenceKind::Inherits);
        assert_eq!(deserialized.target_id, "id_123");
        assert_eq!(deserialized.target_language, Some(Language::Java));
    }

    #[test]
    fn test_type_info_serialize() {
        let type_info = TypeInfo {
            name: "HashMap".to_string(),
            fqn: "std::collections::HashMap".to_string(),
            type_parameters: vec![],
            is_primitive: false,
            is_collection: true,
            is_nullable: false,
            original_type_string: "HashMap<K, V>".to_string(),
        };

        let json = serde_json::to_string(&type_info).unwrap();
        let deserialized: TypeInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, "HashMap");
        assert!(deserialized.is_collection);
    }

    // ==========================================================================
    // Clone tests
    // ==========================================================================

    #[test]
    fn test_unified_node_clone() {
        let mut node = UnifiedNode::new(NodeKind::Class, "MyClass", Language::Java);
        node.add_child("child1".to_string());
        node.add_reference(ReferenceKind::Inherits, "Parent".to_string(), None);
        node.add_metadata("key", "value");

        let cloned = node.clone();

        assert_eq!(node.id, cloned.id);
        assert_eq!(node.name, cloned.name);
        assert_eq!(node.children.len(), cloned.children.len());
        assert_eq!(node.references.len(), cloned.references.len());
        assert_eq!(node.metadata.len(), cloned.metadata.len());
    }

    #[test]
    fn test_source_position_debug() {
        let pos = SourcePosition {
            start_line: 10,
            start_col: 5,
            end_line: 15,
            end_col: 20,
        };
        let debug_str = format!("{:?}", pos);
        assert!(debug_str.contains("SourcePosition"));
        assert!(debug_str.contains("10"));
    }

    #[test]
    fn test_reference_kind_debug() {
        let kind = ReferenceKind::Inherits;
        let debug_str = format!("{:?}", kind);
        assert_eq!(debug_str, "Inherits");
    }
