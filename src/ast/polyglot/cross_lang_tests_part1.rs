// Tests for cross language dependencies
// Extracted for file health compliance (CB-040)

use crate::ast::polyglot::unified_node::SourcePosition;
use std::path::PathBuf;

    fn create_test_node(
        id: &str,
        kind: NodeKind,
        name: &str,
        fqn: &str,
        language: Language,
    ) -> UnifiedNode {
        UnifiedNode {
            id: id.to_string(),
            kind,
            name: name.to_string(),
            fqn: fqn.to_string(),
            language,
            file_path: PathBuf::from("/test/path"),
            position: SourcePosition::default(),
            attributes: HashMap::new(),
            children: Vec::new(),
            parent: None,
            references: Vec::new(),
            type_info: None,
            signature: None,
            documentation: None,
            original_item: None,
            metadata: HashMap::new(),
        }
    }

    fn create_test_node_with_references(
        id: &str,
        kind: NodeKind,
        name: &str,
        fqn: &str,
        language: Language,
        references: Vec<crate::ast::polyglot::unified_node::NodeReference>,
    ) -> UnifiedNode {
        UnifiedNode {
            id: id.to_string(),
            kind,
            name: name.to_string(),
            fqn: fqn.to_string(),
            language,
            file_path: PathBuf::from("/test/path"),
            position: SourcePosition::default(),
            attributes: HashMap::new(),
            children: Vec::new(),
            parent: None,
            references,
            type_info: None,
            signature: None,
            documentation: None,
            original_item: None,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_detect_dependencies() {
        // Create Java class
        let mut java_class = create_test_node(
            "Java:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Java,
        );

        // Create Kotlin class
        let kotlin_class = create_test_node(
            "Kotlin:class:KotlinUser",
            NodeKind::Class,
            "KotlinUser",
            "com.example.KotlinUser",
            Language::Kotlin,
        );

        // Add reference from Java to Kotlin
        java_class.add_reference(
            ReferenceKind::Inherits,
            "com.example.KotlinUser".to_string(),
            None,
        );

        // Detect dependencies
        let mut dependencies = CrossLanguageDependencies::detect(&[java_class], &[kotlin_class]);

        // Sort dependencies by source_id to make test deterministic
        dependencies.sort_by(|a, b| {
            a.source_id
                .cmp(&b.source_id)
                .then(a.target_id.cmp(&b.target_id))
                .then(a.kind.cmp(&b.kind))
        });

        // Verify - exactly one dependency
        assert_eq!(
            dependencies.len(),
            1,
            "Expected exactly 1 dependency, found {}",
            dependencies.len()
        );
        assert_eq!(dependencies[0].source_id, "Java:class:User");
        assert_eq!(dependencies[0].target_id, "Kotlin:class:KotlinUser");
        assert_eq!(dependencies[0].kind, ReferenceKind::Inherits);
        assert_eq!(dependencies[0].source_language, Language::Java);
        assert_eq!(dependencies[0].target_language, Language::Kotlin);
    }

    #[test]
    fn test_name_resolvers() {
        // Java-Kotlin resolver
        let java_kotlin_resolver = JavaKotlinResolver;

        let java_node = create_test_node(
            "Java:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Java,
        );

        let kotlin_node = create_test_node(
            "Kotlin:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Kotlin,
        );

        let reference = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Inherits,
            target_id: String::new(),
            target_name: "com.example.User".to_string(),
            target_language: Some(Language::Kotlin),
        };

        assert!(java_kotlin_resolver.can_resolve(
            Language::Java,
            Language::Kotlin,
            &java_node,
            &reference,
            &kotlin_node,
        ));
    }

    // EXTREME TDD TESTS - Cross-Language Dependency Coverage

    // Test CrossLanguageDependencies::new() and basic initialization
    #[test]
    fn test_new_creates_empty_instance() {
        let deps = CrossLanguageDependencies::new();
        assert!(deps.get_dependencies().is_empty());
    }

    // Test add_nodes functionality
    #[test]
    fn test_add_nodes_basic() {
        let mut deps = CrossLanguageDependencies::new();
        let node1 = create_test_node(
            "Java:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Java,
        );
        let node2 = create_test_node(
            "Kotlin:class:Service",
            NodeKind::Class,
            "Service",
            "com.example.Service",
            Language::Kotlin,
        );

        deps.add_nodes(vec![node1, node2]);
        // Verify nodes were added by running detect_all
        let result = deps.detect_all();
        // No references = no dependencies
        assert!(result.is_empty());
    }

    // Test add_name_resolver functionality
    #[test]
    fn test_add_name_resolver() {
        let mut deps = CrossLanguageDependencies::new();
        deps.add_name_resolver(Language::Java, Box::new(JavaKotlinResolver));
        deps.add_name_resolver(Language::Kotlin, Box::new(JavaKotlinResolver));

        // Create nodes with references that can be resolved
        let mut java_node = create_test_node(
            "Java:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Java,
        );
        java_node.add_reference(
            ReferenceKind::Uses,
            "com.example.KotlinService".to_string(),
            None,
        );

        let kotlin_node = create_test_node(
            "Kotlin:class:KotlinService",
            NodeKind::Class,
            "KotlinService",
            "com.example.KotlinService",
            Language::Kotlin,
        );

        deps.add_nodes(vec![java_node, kotlin_node]);
        let result = deps.detect_all();
        assert!(!result.is_empty());
    }

    // Test filter_by_source_language
    #[test]
    fn test_filter_by_source_language() {
        let mut java_class = create_test_node(
            "Java:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Java,
        );
        java_class.add_reference(ReferenceKind::Inherits, "KotlinBase".to_string(), None);

        let kotlin_class = create_test_node(
            "Kotlin:class:KotlinBase",
            NodeKind::Class,
            "KotlinBase",
            "com.example.KotlinBase",
            Language::Kotlin,
        );

        let mut deps = CrossLanguageDependencies::new();
        deps.add_nodes(vec![java_class, kotlin_class]);
        deps.detect_all();

        let java_deps = deps.filter_by_source_language(Language::Java);
        assert!(!java_deps.is_empty());
        for dep in &java_deps {
            assert_eq!(dep.source_language, Language::Java);
        }

        let python_deps = deps.filter_by_source_language(Language::Python);
        assert!(python_deps.is_empty());
    }

    // Test filter_by_target_language
    #[test]
    fn test_filter_by_target_language() {
        let mut java_class = create_test_node(
            "Java:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Java,
        );
        java_class.add_reference(
            ReferenceKind::Implements,
            "KotlinInterface".to_string(),
            None,
        );

        let kotlin_interface = create_test_node(
            "Kotlin:interface:KotlinInterface",
            NodeKind::Interface,
            "KotlinInterface",
            "com.example.KotlinInterface",
            Language::Kotlin,
        );

        let mut deps = CrossLanguageDependencies::new();
        deps.add_nodes(vec![java_class, kotlin_interface]);
        deps.detect_all();

        let kotlin_deps = deps.filter_by_target_language(Language::Kotlin);
        assert!(!kotlin_deps.is_empty());
        for dep in &kotlin_deps {
            assert_eq!(dep.target_language, Language::Kotlin);
        }

        let rust_deps = deps.filter_by_target_language(Language::Rust);
        assert!(rust_deps.is_empty());
    }

    // Test filter_by_kind
    #[test]
    fn test_filter_by_kind() {
        let mut java_class = create_test_node(
            "Java:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Java,
        );
        java_class.add_reference(ReferenceKind::Inherits, "KotlinBase".to_string(), None);
        java_class.add_reference(
            ReferenceKind::Implements,
            "KotlinInterface".to_string(),
            None,
        );

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

        let mut deps = CrossLanguageDependencies::new();
        deps.add_nodes(vec![java_class, kotlin_base, kotlin_interface]);
        deps.detect_all();

        let inherits_deps = deps.filter_by_kind(ReferenceKind::Inherits);
        let implements_deps = deps.filter_by_kind(ReferenceKind::Implements);
        let calls_deps = deps.filter_by_kind(ReferenceKind::Calls);

        // At least one inherits and one implements dependency
        assert!(!inherits_deps.is_empty());
        assert!(!implements_deps.is_empty());
        assert!(calls_deps.is_empty());
    }

    // Test get_dependencies_between
    #[test]
    fn test_get_dependencies_between() {
        let mut java_class = create_test_node(
            "Java:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Java,
        );
        java_class.add_reference(ReferenceKind::Inherits, "KotlinBase".to_string(), None);

        let kotlin_base = create_test_node(
            "Kotlin:class:KotlinBase",
            NodeKind::Class,
            "KotlinBase",
            "com.example.KotlinBase",
            Language::Kotlin,
        );

        let mut deps = CrossLanguageDependencies::new();
        deps.add_nodes(vec![java_class, kotlin_base]);
        deps.detect_all();

        let java_to_kotlin = deps.get_dependencies_between(Language::Java, Language::Kotlin);
        assert!(!java_to_kotlin.is_empty());

        let kotlin_to_java = deps.get_dependencies_between(Language::Kotlin, Language::Java);
        assert!(kotlin_to_java.is_empty());

        let java_to_python = deps.get_dependencies_between(Language::Java, Language::Python);
        assert!(java_to_python.is_empty());
    }

    // Test to_dot graph generation
    #[test]
    fn test_to_dot_generation() {
        let mut java_class = create_test_node(
            "Java:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Java,
        );
        java_class.add_reference(ReferenceKind::Inherits, "KotlinBase".to_string(), None);

        let kotlin_base = create_test_node(
            "Kotlin:class:KotlinBase",
            NodeKind::Class,
            "KotlinBase",
            "com.example.KotlinBase",
            Language::Kotlin,
        );

        let mut deps = CrossLanguageDependencies::new();
        deps.add_nodes(vec![java_class, kotlin_base]);
        deps.detect_all();

        let dot = deps.to_dot();

        // Verify basic DOT structure
        assert!(dot.starts_with("digraph CrossLanguageDependencies {"));
        assert!(dot.ends_with("}\n"));
        assert!(dot.contains("\"Java:class:User\""));
        assert!(dot.contains("\"Kotlin:class:KotlinBase\""));
        assert!(dot.contains("User (Java)"));
        assert!(dot.contains("KotlinBase (Kotlin)"));
        // Should have an edge for the inheritance
        assert!(dot.contains("->"));
        assert!(dot.contains("Inherits"));
        assert!(dot.contains("bold")); // Inherits style
    }

    // Test to_dot with different node kinds
    #[test]
    fn test_to_dot_node_shapes() {
        let class_node = create_test_node(
            "Java:class:MyClass",
            NodeKind::Class,
            "MyClass",
            "com.example.MyClass",
            Language::Java,
        );

        let interface_node = create_test_node(
            "Java:interface:MyInterface",
            NodeKind::Interface,
            "MyInterface",
            "com.example.MyInterface",
            Language::Java,
        );

        let trait_node = create_test_node(
            "Rust:trait:MyTrait",
            NodeKind::Trait,
            "MyTrait",
            "crate::MyTrait",
            Language::Rust,
        );

        let method_node = create_test_node(
            "Java:method:doSomething",
            NodeKind::Method,
            "doSomething",
            "com.example.MyClass.doSomething",
            Language::Java,
        );

        let function_node = create_test_node(
            "Python:function:process",
            NodeKind::Function,
            "process",
            "module.process",
            Language::Python,
        );

        let field_node = create_test_node(
            "Java:field:name",
            NodeKind::Field,
            "name",
            "com.example.MyClass.name",
            Language::Java,
        );

        let mut deps = CrossLanguageDependencies::new();
        deps.add_nodes(vec![
            class_node,
            interface_node,
            trait_node,
            method_node,
            function_node,
            field_node,
        ]);

        let dot = deps.to_dot();

        // Check shapes
        assert!(dot.contains("shape=box")); // Class
        assert!(dot.contains("shape=ellipse")); // Interface/Trait
        assert!(dot.contains("shape=octagon")); // Method/Function
        assert!(dot.contains("shape=plaintext")); // Field and others
    }

