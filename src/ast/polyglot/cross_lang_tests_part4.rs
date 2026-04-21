    #[test]
    fn test_to_dot_different_reference_kinds() {
        let mut java_class = create_test_node(
            "Java:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Java,
        );
        // Add various reference types
        java_class.add_reference(ReferenceKind::Inherits, "KotlinBase".to_string(), None);
        java_class.add_reference(
            ReferenceKind::Implements,
            "KotlinInterface".to_string(),
            None,
        );
        java_class.add_reference(ReferenceKind::Calls, "KotlinFunction".to_string(), None);

        let kotlin_base = create_test_node(
            "Kotlin:class:KotlinBase",
            NodeKind::Class,
            "KotlinBase",
            "com.example.KotlinBase",
            Language::Kotlin,
        );

        let kotlin_interface = create_test_node(
            "Kotlin:interface:KotlinInterface",
            NodeKind::Interface,
            "KotlinInterface",
            "com.example.KotlinInterface",
            Language::Kotlin,
        );

        let kotlin_function = create_test_node(
            "Kotlin:function:KotlinFunction",
            NodeKind::Function,
            "KotlinFunction",
            "com.example.KotlinFunction",
            Language::Kotlin,
        );

        let mut deps = CrossLanguageDependencies::new();
        deps.add_nodes(vec![
            java_class,
            kotlin_base,
            kotlin_interface,
            kotlin_function,
        ]);
        deps.detect_all();

        let dot = deps.to_dot();

        // Check that different styles are used
        assert!(dot.contains("bold")); // Inherits
        assert!(dot.contains("dashed")); // Implements
        assert!(dot.contains("solid")); // Calls (and default)
    }

    // Test TypeScriptJavaResolver with short interface name
    #[test]
    fn test_typescript_interface_single_char_after_i() {
        let resolver = TypeScriptJavaResolver;

        let ts_node = create_test_node(
            "TypeScript:interface:IA",
            NodeKind::Interface,
            "IA",
            "IA",
            Language::TypeScript,
        );

        let java_node = create_test_node(
            "Java:class:A",
            NodeKind::Class,
            "A",
            "com.example.A",
            Language::Java,
        );

        let reference = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Uses,
            target_id: String::new(),
            target_name: "IA".to_string(),
            target_language: Some(Language::Java),
        };

        // IA -> A mapping should work
        assert!(resolver.can_resolve(
            Language::TypeScript,
            Language::Java,
            &ts_node,
            &reference,
            &java_node,
        ));
    }

    // Test JavaKotlinResolver with mismatched package depths
    #[test]
    fn test_java_kotlin_resolver_different_package_depth() {
        let resolver = JavaKotlinResolver;

        let java_node = create_test_node(
            "Java:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Java,
        );

        let kotlin_node = create_test_node(
            "Kotlin:class:Service",
            NodeKind::Class,
            "Service",
            "com.example.api.Service",
            Language::Kotlin,
        );

        let reference = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Uses,
            target_id: String::new(),
            target_name: "com.example.Service".to_string(), // Different depth
            target_language: Some(Language::Kotlin),
        };

        // Should not match due to different package depths
        assert!(!resolver.can_resolve(
            Language::Java,
            Language::Kotlin,
            &java_node,
            &reference,
            &kotlin_node,
        ));
    }

    // Test with resolver that uses package parts comparison
    #[test]
    fn test_java_scala_resolver_package_parts_mismatch() {
        let resolver = JavaScalaResolver;

        let java_node = create_test_node(
            "Java:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Java,
        );

        let scala_node = create_test_node(
            "Scala:class:DifferentService",
            NodeKind::Class,
            "DifferentService",
            "com.example.DifferentService",
            Language::Scala,
        );

        let reference = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Uses,
            target_id: String::new(),
            target_name: "com.example.Service".to_string(), // Different last part
            target_language: Some(Language::Scala),
        };

        // Should not match because class names differ
        assert!(!resolver.can_resolve(
            Language::Java,
            Language::Scala,
            &java_node,
            &reference,
            &scala_node,
        ));
    }

    // Test TypeScriptJavaResolver no I prefix
    #[test]
    fn test_typescript_java_no_interface_prefix() {
        let resolver = TypeScriptJavaResolver;

        let ts_node = create_test_node(
            "TypeScript:interface:UserInterface",
            NodeKind::Interface,
            "UserInterface",
            "UserInterface",
            Language::TypeScript,
        );

        let java_node = create_test_node(
            "Java:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Java,
        );

        let reference = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Uses,
            target_id: String::new(),
            target_name: "UserInterface".to_string(),
            target_language: Some(Language::Java),
        };

        // Should not match - no I prefix
        assert!(!resolver.can_resolve(
            Language::TypeScript,
            Language::Java,
            &ts_node,
            &reference,
            &java_node,
        ));
    }

    // Test that CrossLanguageDependency struct fields are accessible
    #[test]
    fn test_cross_language_dependency_struct() {
        let dep = CrossLanguageDependency {
            source_id: "source".to_string(),
            target_id: "target".to_string(),
            source_language: Language::Java,
            target_language: Language::Kotlin,
            kind: ReferenceKind::Inherits,
            confidence: 0.95,
            metadata: {
                let mut m = HashMap::new();
                m.insert("key".to_string(), "value".to_string());
                m
            },
        };

        assert_eq!(dep.source_id, "source");
        assert_eq!(dep.target_id, "target");
        assert_eq!(dep.source_language, Language::Java);
        assert_eq!(dep.target_language, Language::Kotlin);
        assert_eq!(dep.kind, ReferenceKind::Inherits);
        assert!((dep.confidence - 0.95).abs() < f64::EPSILON);
        assert_eq!(dep.metadata.get("key"), Some(&"value".to_string()));
    }

    // Test Default trait implementation for CrossLanguageDependencies
    #[test]
    fn test_cross_language_dependencies_default() {
        let deps = CrossLanguageDependencies::default();
        assert!(deps.get_dependencies().is_empty());
    }

    // Test with name resolver integration
    #[test]
    fn test_detect_with_name_resolver_integration() {
        let mut deps = CrossLanguageDependencies::new();

        // Add Java->Kotlin resolver
        deps.add_name_resolver(Language::Java, Box::new(JavaKotlinResolver));

        // Create a Java node with reference
        let mut java_node = create_test_node(
            "Java:class:Client",
            NodeKind::Class,
            "Client",
            "com.example.Client",
            Language::Java,
        );

        // Reference using package pattern that the resolver handles
        let ref1 = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Uses,
            target_id: String::new(),
            target_name: "com.example.KotlinService".to_string(),
            target_language: Some(Language::Kotlin),
        };
        java_node.references.push(ref1);

        let kotlin_node = create_test_node(
            "Kotlin:class:KotlinService",
            NodeKind::Class,
            "KotlinService",
            "com.example.KotlinService",
            Language::Kotlin,
        );

        deps.add_nodes(vec![java_node, kotlin_node]);
        deps.detect_all();

        let result = deps.get_dependencies();
        assert!(!result.is_empty());
    }

    // Test with all ReferenceKind variants
    #[test]
    fn test_all_reference_kinds() {
        let all_kinds = [
            ReferenceKind::Inherits,
            ReferenceKind::Implements,
            ReferenceKind::Calls,
            ReferenceKind::Uses,
            ReferenceKind::Creates,
            ReferenceKind::Imports,
            ReferenceKind::Annotates,
            ReferenceKind::DependsOn,
        ];

        for kind in all_kinds {
            let mut java_node = create_test_node(
                &format!("Java:class:Source{:?}", kind),
                NodeKind::Class,
                &format!("Source{:?}", kind),
                &format!("com.example.Source{:?}", kind),
                Language::Java,
            );
            java_node.add_reference(kind, format!("Target{:?}", kind), None);

            let kotlin_node = create_test_node(
                &format!("Kotlin:class:Target{:?}", kind),
                NodeKind::Class,
                &format!("Target{:?}", kind),
                &format!("com.example.Target{:?}", kind),
                Language::Kotlin,
            );

            let mut deps = CrossLanguageDependencies::new();
            deps.add_nodes(vec![java_node, kotlin_node]);
            deps.detect_all();

            let filtered = deps.filter_by_kind(kind);
            assert!(
                !filtered.is_empty(),
                "Expected at least one dependency of kind {:?}",
                kind
            );
            assert_eq!(filtered[0].kind, kind);
        }
    }

    // Test FQN map building in add_nodes
    #[test]
    fn test_fqn_map_building() {
        let mut deps = CrossLanguageDependencies::new();

        // Create two nodes with same FQN but different IDs (simulating overloads)
        let node1 = create_test_node(
            "Java:method:process1",
            NodeKind::Method,
            "process",
            "com.example.Service.process",
            Language::Java,
        );

        let node2 = create_test_node(
            "Java:method:process2",
            NodeKind::Method,
            "process",
            "com.example.Service.process",
            Language::Java,
        );

        deps.add_nodes(vec![node1, node2]);

        // Both should be added and detectable
        let result = deps.detect_all();
        // No cross-language deps expected (same language)
        assert!(result.is_empty());
    }

    // Test nodes grouped by language correctly
    #[test]
    fn test_nodes_grouped_by_language() {
        let mut deps = CrossLanguageDependencies::new();

        let java_node = create_test_node(
            "Java:class:JavaClass",
            NodeKind::Class,
            "JavaClass",
            "com.example.JavaClass",
            Language::Java,
        );

        let kotlin_node = create_test_node(
            "Kotlin:class:KotlinClass",
            NodeKind::Class,
            "KotlinClass",
            "com.example.KotlinClass",
            Language::Kotlin,
        );

        let scala_node = create_test_node(
            "Scala:class:ScalaClass",
            NodeKind::Class,
            "ScalaClass",
            "com.example.ScalaClass",
            Language::Scala,
        );

        let ts_node = create_test_node(
            "TypeScript:class:TsClass",
            NodeKind::Class,
            "TsClass",
            "TsClass",
            Language::TypeScript,
        );

        deps.add_nodes(vec![java_node, kotlin_node, scala_node, ts_node]);
        deps.detect_all();

        // Verify we can filter by all languages
        let java_deps = deps.filter_by_source_language(Language::Java);
        let kotlin_deps = deps.filter_by_source_language(Language::Kotlin);
        let scala_deps = deps.filter_by_source_language(Language::Scala);
        let ts_deps = deps.filter_by_source_language(Language::TypeScript);

        // No references added, so all should be empty
        assert!(java_deps.is_empty());
        assert!(kotlin_deps.is_empty());
        assert!(scala_deps.is_empty());
        assert!(ts_deps.is_empty());
    }

    // Test DOT graph with no nodes
    #[test]
    fn test_to_dot_empty() {
        let deps = CrossLanguageDependencies::new();
        let dot = deps.to_dot();

        assert!(dot.starts_with("digraph CrossLanguageDependencies {"));
        assert!(dot.ends_with("}\n"));
        // No nodes or edges
        assert!(!dot.contains("->"));
    }

    // Test Clone and Debug for CrossLanguageDependency
    #[test]
    fn test_cross_language_dependency_clone_debug() {
        let dep = CrossLanguageDependency {
            source_id: "source".to_string(),
            target_id: "target".to_string(),
            source_language: Language::Java,
            target_language: Language::Kotlin,
            kind: ReferenceKind::Inherits,
            confidence: 1.0,
            metadata: HashMap::new(),
        };

        let cloned = dep.clone();
        assert_eq!(dep.source_id, cloned.source_id);
        assert_eq!(dep.target_id, cloned.target_id);

        // Test Debug
        let debug_str = format!("{:?}", dep);
        assert!(debug_str.contains("CrossLanguageDependency"));
        assert!(debug_str.contains("source"));
        assert!(debug_str.contains("target"));
    }

    // Test with deeply nested package names
    #[test]
    fn test_deeply_nested_packages() {
        let resolver = JavaKotlinResolver;

        let java_node = create_test_node(
            "Java:class:User",
            NodeKind::Class,
            "User",
            "com.example.api.v2.internal.User",
            Language::Java,
        );

        let kotlin_node = create_test_node(
            "Kotlin:class:Service",
            NodeKind::Class,
            "Service",
            "com.example.api.v2.internal.Service",
            Language::Kotlin,
        );

        let reference = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Uses,
            target_id: String::new(),
            target_name: "com.example.api.v2.internal.Service".to_string(),
            target_language: Some(Language::Kotlin),
        };

        assert!(resolver.can_resolve(
            Language::Java,
            Language::Kotlin,
            &java_node,
            &reference,
            &kotlin_node,
        ));
    }

    // Resolver fall-through: same-length packages, same last part, DIFFERENT
    // package prefix. Exercises the `src_parts[0..-1] != tgt_parts[0..-1]`
    // branch in JavaKotlinResolver.
    #[test]
    fn test_java_kotlin_resolver_same_len_different_prefix() {
        let resolver = JavaKotlinResolver;

        let java_node = create_test_node(
            "Java:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Java,
        );
        let kotlin_node = create_test_node(
            "Kotlin:class:Service",
            NodeKind::Class,
            "Service",
            "com.other.Service",
            Language::Kotlin,
        );
        let reference = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Uses,
            target_id: String::new(),
            target_name: "com.example.Service".to_string(),
            target_language: Some(Language::Kotlin),
        };

        assert!(!resolver.can_resolve(
            Language::Java,
            Language::Kotlin,
            &java_node,
            &reference,
            &kotlin_node,
        ));
    }

    // Resolver fall-through: same-length packages, same prefix, DIFFERENT last
    // part. Exercises the inner `src_parts.last() != tgt_parts.last()` branch
    // in JavaKotlinResolver (JavaScala analog is already covered).
    #[test]
    fn test_java_kotlin_resolver_same_prefix_different_last() {
        let resolver = JavaKotlinResolver;

        let java_node = create_test_node(
            "Java:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Java,
        );
        let kotlin_node = create_test_node(
            "Kotlin:class:Other",
            NodeKind::Class,
            "Other",
            "com.example.Other",
            Language::Kotlin,
        );
        let reference = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Uses,
            target_id: String::new(),
            target_name: "com.example.Service".to_string(),
            target_language: Some(Language::Kotlin),
        };

        assert!(!resolver.can_resolve(
            Language::Java,
            Language::Kotlin,
            &java_node,
            &reference,
            &kotlin_node,
        ));
    }

    // Resolver fall-through: same-length packages, different prefix for
    // JavaScalaResolver. The same-len + different-last branch is already
    // covered; this hits the different-prefix branch.
    #[test]
    fn test_java_scala_resolver_same_len_different_prefix() {
        let resolver = JavaScalaResolver;

        let java_node = create_test_node(
            "Java:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Java,
        );
        let scala_node = create_test_node(
            "Scala:class:Service",
            NodeKind::Class,
            "Service",
            "com.other.Service",
            Language::Scala,
        );
        let reference = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Uses,
            target_id: String::new(),
            target_name: "com.example.Service".to_string(),
            target_language: Some(Language::Scala),
        };

        assert!(!resolver.can_resolve(
            Language::Java,
            Language::Scala,
            &java_node,
            &reference,
            &scala_node,
        ));
    }

    // TypeScript I-prefix branch fall-through: name starts with 'I', len > 1,
    // but stripped name matches neither target.name nor target.fqn.ends_with.
    #[test]
    fn test_typescript_java_resolver_i_prefix_no_match() {
        let resolver = TypeScriptJavaResolver;

        let ts_node = create_test_node(
            "TypeScript:interface:IFoo",
            NodeKind::Interface,
            "IFoo",
            "IFoo",
            Language::TypeScript,
        );
        let java_node = create_test_node(
            "Java:class:Bar",
            NodeKind::Class,
            "Bar",
            "com.example.Bar",
            Language::Java,
        );
        let reference = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Uses,
            target_id: String::new(),
            target_name: "IFoo".to_string(),
            target_language: Some(Language::Java),
        };

        // IFoo stripped → "Foo", which matches neither "Bar" nor "com.example.Bar"
        assert!(!resolver.can_resolve(
            Language::TypeScript,
            Language::Java,
            &ts_node,
            &reference,
            &java_node,
        ));
    }

    // TypeScript bare "I" — starts_with('I') passes but `len() > 1` fails,
    // so the interface-stripping branch is skipped entirely.
    #[test]
    fn test_typescript_java_resolver_bare_i_prefix_skipped() {
        let resolver = TypeScriptJavaResolver;

        let ts_node = create_test_node(
            "TypeScript:interface:I",
            NodeKind::Interface,
            "I",
            "I",
            Language::TypeScript,
        );
        let java_node = create_test_node(
            "Java:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Java,
        );
        let reference = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Uses,
            target_id: String::new(),
            target_name: "I".to_string(),
            target_language: Some(Language::Java),
        };

        assert!(!resolver.can_resolve(
            Language::TypeScript,
            Language::Java,
            &ts_node,
            &reference,
            &java_node,
        ));
    }

    // Test serialization/deserialization of CrossLanguageDependency
    #[test]
    fn test_cross_language_dependency_serde() {
        let dep = CrossLanguageDependency {
            source_id: "source".to_string(),
            target_id: "target".to_string(),
            source_language: Language::Java,
            target_language: Language::Kotlin,
            kind: ReferenceKind::Inherits,
            confidence: 0.9,
            metadata: {
                let mut m = HashMap::new();
                m.insert("key".to_string(), "value".to_string());
                m
            },
        };

        let json = serde_json::to_string(&dep).unwrap();
        let deserialized: CrossLanguageDependency = serde_json::from_str(&json).unwrap();

        assert_eq!(dep.source_id, deserialized.source_id);
        assert_eq!(dep.target_id, deserialized.target_id);
        assert_eq!(dep.kind, deserialized.kind);
        assert!((dep.confidence - deserialized.confidence).abs() < f64::EPSILON);
    }
