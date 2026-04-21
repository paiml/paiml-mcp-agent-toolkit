    #[test]
    fn test_java_kotlin_resolver_wrong_languages() {
        let resolver = JavaKotlinResolver;

        let java_node = create_test_node(
            "Java:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Java,
        );

        let python_node = create_test_node(
            "Python:class:User",
            NodeKind::Class,
            "User",
            "user.User",
            Language::Python,
        );

        let reference = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Inherits,
            target_id: String::new(),
            target_name: "User".to_string(),
            target_language: Some(Language::Python),
        };

        // Should return false for non Java-Kotlin combination
        assert!(!resolver.can_resolve(
            Language::Java,
            Language::Python,
            &java_node,
            &reference,
            &python_node,
        ));
    }

    #[test]
    fn test_kotlin_to_java_resolver() {
        let resolver = JavaKotlinResolver;

        let kotlin_node = create_test_node(
            "Kotlin:class:KotlinUser",
            NodeKind::Class,
            "KotlinUser",
            "com.example.KotlinUser",
            Language::Kotlin,
        );

        let java_node = create_test_node(
            "Java:class:JavaService",
            NodeKind::Class,
            "JavaService",
            "com.example.JavaService",
            Language::Java,
        );

        let reference = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Uses,
            target_id: String::new(),
            target_name: "JavaService".to_string(),
            target_language: Some(Language::Java),
        };

        // Kotlin to Java should also work
        assert!(resolver.can_resolve(
            Language::Kotlin,
            Language::Java,
            &kotlin_node,
            &reference,
            &java_node,
        ));
    }

    #[test]
    fn test_java_kotlin_resolver_package_name_conversion() {
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
            "com.example.Service",
            Language::Kotlin,
        );

        let reference = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Uses,
            target_id: String::new(),
            target_name: "com.example.Service".to_string(),
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

    // Test direct ID match in is_reference_match
    #[test]
    fn test_is_reference_match_direct_id() {
        let deps = CrossLanguageDependencies::new();

        let source = create_test_node(
            "Java:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Java,
        );

        let target = create_test_node(
            "Kotlin:class:KotlinUser",
            NodeKind::Class,
            "KotlinUser",
            "com.example.KotlinUser",
            Language::Kotlin,
        );

        let reference = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Inherits,
            target_id: "Kotlin:class:KotlinUser".to_string(),
            target_name: "".to_string(),
            target_language: Some(Language::Kotlin),
        };

        assert!(deps.is_reference_match(
            &source,
            &reference,
            &target,
            Language::Java,
            Language::Kotlin
        ));
    }

    // Test unresolved reference resolution
    #[test]
    fn test_resolve_references_by_name() {
        let java_ref = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Uses,
            target_id: String::new(), // Empty ID = unresolved
            target_name: "SharedComponent".to_string(),
            target_language: None,
        };

        let java_node = create_test_node_with_references(
            "Java:class:JavaClient",
            NodeKind::Class,
            "JavaClient",
            "com.example.JavaClient",
            Language::Java,
            vec![java_ref],
        );

        // Create a TypeScript node with the same name
        let ts_node = create_test_node(
            "TypeScript:class:SharedComponent",
            NodeKind::Class,
            "SharedComponent",
            "SharedComponent",
            Language::TypeScript,
        );

        let mut deps = CrossLanguageDependencies::new();
        deps.add_nodes(vec![java_node, ts_node]);
        deps.detect_all();

        let resolved_deps = deps.get_dependencies();
        // Should have resolved the reference by name
        assert!(!resolved_deps.is_empty());

        // Find the specific dependency
        let dep = resolved_deps.iter().find(|d| {
            d.source_id == "Java:class:JavaClient"
                && d.target_id == "TypeScript:class:SharedComponent"
        });
        assert!(dep.is_some());
        // Name-based resolution should have reasonable confidence
        // Value may vary based on detection order (0.8 for name-based, 1.0 for certain matches)
        let confidence = dep.unwrap().confidence;
        assert!(
            (0.8..=1.0).contains(&confidence),
            "Confidence should be in valid range, got: {confidence}"
        );
    }

    // Test resolve_references with FQN match
    #[test]
    fn test_resolve_references_by_fqn() {
        let java_ref = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Imports,
            target_id: String::new(), // Empty ID = unresolved
            target_name: "com.example.SharedModule".to_string(),
            target_language: None,
        };

        let java_node = create_test_node_with_references(
            "Java:class:JavaClient",
            NodeKind::Class,
            "JavaClient",
            "com.example.JavaClient",
            Language::Java,
            vec![java_ref],
        );

        // Create a Kotlin node with the same FQN
        let kotlin_node = create_test_node(
            "Kotlin:module:SharedModule",
            NodeKind::Module,
            "SharedModule",
            "com.example.SharedModule", // Same FQN
            Language::Kotlin,
        );

        let mut deps = CrossLanguageDependencies::new();
        deps.add_nodes(vec![java_node, kotlin_node]);
        deps.detect_all();

        let resolved_deps = deps.get_dependencies();
        assert!(!resolved_deps.is_empty());
    }

    // Test deduplication of dependencies
    #[test]
    fn test_dependency_deduplication() {
        // Create a node with multiple references to the same target
        let ref1 = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Uses,
            target_id: String::new(),
            target_name: "SharedService".to_string(),
            target_language: None,
        };

        let ref2 = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Uses,
            target_id: String::new(),
            target_name: "SharedService".to_string(), // Same target, same kind
            target_language: None,
        };

        let java_node = create_test_node_with_references(
            "Java:class:JavaClient",
            NodeKind::Class,
            "JavaClient",
            "com.example.JavaClient",
            Language::Java,
            vec![ref1, ref2],
        );

        let kotlin_node = create_test_node(
            "Kotlin:class:SharedService",
            NodeKind::Class,
            "SharedService",
            "com.example.SharedService",
            Language::Kotlin,
        );

        let mut deps = CrossLanguageDependencies::new();
        deps.add_nodes(vec![java_node, kotlin_node]);
        deps.detect_all();

        // Count dependencies with same source_id, target_id, and kind
        let uses_deps: Vec<_> = deps
            .get_dependencies()
            .iter()
            .filter(|d| {
                d.source_id == "Java:class:JavaClient"
                    && d.target_id == "Kotlin:class:SharedService"
                    && d.kind == ReferenceKind::Uses
            })
            .collect();

        // Should be deduplicated to 1
        assert_eq!(uses_deps.len(), 1);
    }

    // Test multiple languages interaction
    #[test]
    fn test_multiple_languages() {
        let mut java_node = create_test_node(
            "Java:class:JavaService",
            NodeKind::Class,
            "JavaService",
            "com.example.JavaService",
            Language::Java,
        );
        java_node.add_reference(ReferenceKind::Uses, "KotlinHelper".to_string(), None);
        java_node.add_reference(ReferenceKind::Uses, "ScalaProcessor".to_string(), None);

        let kotlin_node = create_test_node(
            "Kotlin:class:KotlinHelper",
            NodeKind::Class,
            "KotlinHelper",
            "com.example.KotlinHelper",
            Language::Kotlin,
        );

        let scala_node = create_test_node(
            "Scala:class:ScalaProcessor",
            NodeKind::Class,
            "ScalaProcessor",
            "com.example.ScalaProcessor",
            Language::Scala,
        );

        let mut deps = CrossLanguageDependencies::new();
        deps.add_nodes(vec![java_node, kotlin_node, scala_node]);
        deps.detect_all();

        let java_to_kotlin = deps.get_dependencies_between(Language::Java, Language::Kotlin);
        let java_to_scala = deps.get_dependencies_between(Language::Java, Language::Scala);

        assert!(!java_to_kotlin.is_empty());
        assert!(!java_to_scala.is_empty());
    }

    // Test empty nodes case
    #[test]
    fn test_empty_nodes() {
        let deps = CrossLanguageDependencies::detect(&[], &[]);
        assert!(deps.is_empty());
    }

    // Test same language nodes (no cross-language deps)
    #[test]
    fn test_same_language_no_cross_deps() {
        let mut java1 = create_test_node(
            "Java:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Java,
        );
        java1.add_reference(ReferenceKind::Uses, "Service".to_string(), None);

        let java2 = create_test_node(
            "Java:class:Service",
            NodeKind::Class,
            "Service",
            "com.example.Service",
            Language::Java,
        );

        let deps = CrossLanguageDependencies::detect(&[java1], &[java2]);
        // Same language references should not create cross-language dependencies
        assert!(deps.is_empty());
    }

    // Covers the `let Some(target_ids) = name_map.get(target_name) else { continue }`
    // branch in `resolve_against_name_map` — an unresolved reference whose
    // target name exists nowhere in the node set.
    #[test]
    fn test_resolve_against_name_map_target_missing_from_name_map() {
        let mut java_node = create_test_node(
            "Java:class:Orphan",
            NodeKind::Class,
            "Orphan",
            "com.example.Orphan",
            Language::Java,
        );
        // Reference to a name that is not in the node set at all.
        java_node.add_reference(ReferenceKind::Uses, "NowhereType".to_string(), None);

        let ts_node = create_test_node(
            "TypeScript:class:Unrelated",
            NodeKind::Class,
            "Unrelated",
            "Unrelated",
            Language::TypeScript,
        );

        let deps = CrossLanguageDependencies::detect(&[java_node], &[ts_node]);
        // Reference resolves to nothing → no cross-language dependency produced.
        assert!(deps.is_empty());
    }

    // Test different ReferenceKind types in DOT output
