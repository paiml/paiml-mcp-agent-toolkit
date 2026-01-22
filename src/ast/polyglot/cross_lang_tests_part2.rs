    // Test to_dot with implements relationship (dashed style)
    #[test]
    fn test_to_dot_implements_style() {
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

        let dot = deps.to_dot();
        assert!(dot.contains("dashed")); // Implements style
    }

    // Test language_color for all languages
    #[test]
    fn test_language_colors() {
        let deps = CrossLanguageDependencies::new();

        // Test all specific language colors
        assert_eq!(deps.language_color(Language::Java), "\"#b07219\"");
        assert_eq!(deps.language_color(Language::Kotlin), "\"#A97BFF\"");
        assert_eq!(deps.language_color(Language::Scala), "\"#c22d40\"");
        assert_eq!(deps.language_color(Language::TypeScript), "\"#2b7489\"");
        assert_eq!(deps.language_color(Language::JavaScript), "\"#f1e05a\"");
        assert_eq!(deps.language_color(Language::Python), "\"#3572A5\"");
        assert_eq!(deps.language_color(Language::Rust), "\"#dea584\"");
        assert_eq!(deps.language_color(Language::Go), "\"#00ADD8\"");
        assert_eq!(deps.language_color(Language::Cpp), "\"#f34b7d\"");
        // Test fallback for other languages
        assert_eq!(deps.language_color(Language::Ruby), "\"#bbbbbb\"");
        assert_eq!(deps.language_color(Language::Swift), "\"#bbbbbb\"");
        assert_eq!(deps.language_color(Language::Php), "\"#bbbbbb\"");
        assert_eq!(deps.language_color(Language::CSharp), "\"#bbbbbb\"");
        assert_eq!(deps.language_color(Language::Other(999)), "\"#bbbbbb\"");
    }

    // Test JavaScalaResolver
    #[test]
    fn test_java_scala_resolver_direct_name_match() {
        let resolver = JavaScalaResolver;

        let java_node = create_test_node(
            "Java:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Java,
        );

        let scala_node = create_test_node(
            "Scala:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Scala,
        );

        let reference = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Inherits,
            target_id: String::new(),
            target_name: "User".to_string(),
            target_language: Some(Language::Scala),
        };

        assert!(resolver.can_resolve(
            Language::Java,
            Language::Scala,
            &java_node,
            &reference,
            &scala_node,
        ));
    }

    #[test]
    fn test_java_scala_resolver_fqn_match() {
        let resolver = JavaScalaResolver;

        let java_node = create_test_node(
            "Java:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Java,
        );

        let scala_node = create_test_node(
            "Scala:class:ScalaUser",
            NodeKind::Class,
            "ScalaUser",
            "com.example.ScalaUser",
            Language::Scala,
        );

        let reference = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Uses,
            target_id: String::new(),
            target_name: "com.example.ScalaUser".to_string(),
            target_language: Some(Language::Scala),
        };

        assert!(resolver.can_resolve(
            Language::Java,
            Language::Scala,
            &java_node,
            &reference,
            &scala_node,
        ));
    }

    #[test]
    fn test_java_scala_resolver_package_conversion() {
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
            "com.example.Service",
            Language::Scala,
        );

        let reference = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Uses,
            target_id: String::new(),
            target_name: "com.example.Service".to_string(),
            target_language: Some(Language::Scala),
        };

        assert!(resolver.can_resolve(
            Language::Java,
            Language::Scala,
            &java_node,
            &reference,
            &scala_node,
        ));
    }

    #[test]
    fn test_java_scala_resolver_wrong_languages() {
        let resolver = JavaScalaResolver;

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
            target_name: "User".to_string(),
            target_language: Some(Language::Kotlin),
        };

        // Should return false for non Java-Scala combination
        assert!(!resolver.can_resolve(
            Language::Java,
            Language::Kotlin,
            &java_node,
            &reference,
            &kotlin_node,
        ));
    }

    #[test]
    fn test_scala_to_java_resolver() {
        let resolver = JavaScalaResolver;

        let scala_node = create_test_node(
            "Scala:class:ScalaUser",
            NodeKind::Class,
            "ScalaUser",
            "com.example.ScalaUser",
            Language::Scala,
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

        // Scala to Java should also work
        assert!(resolver.can_resolve(
            Language::Scala,
            Language::Java,
            &scala_node,
            &reference,
            &java_node,
        ));
    }

    // Test TypeScriptJavaResolver
    #[test]
    fn test_typescript_java_resolver_direct_name_match() {
        let resolver = TypeScriptJavaResolver;

        let ts_node = create_test_node(
            "TypeScript:class:UserService",
            NodeKind::Class,
            "UserService",
            "UserService",
            Language::TypeScript,
        );

        let java_node = create_test_node(
            "Java:class:UserService",
            NodeKind::Class,
            "UserService",
            "com.example.UserService",
            Language::Java,
        );

        let reference = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Uses,
            target_id: String::new(),
            target_name: "UserService".to_string(),
            target_language: Some(Language::Java),
        };

        assert!(resolver.can_resolve(
            Language::TypeScript,
            Language::Java,
            &ts_node,
            &reference,
            &java_node,
        ));
    }

    #[test]
    fn test_typescript_java_resolver_fqn_ends_with_match() {
        let resolver = TypeScriptJavaResolver;

        let ts_node = create_test_node(
            "TypeScript:class:UserService",
            NodeKind::Class,
            "UserService",
            "UserService",
            Language::TypeScript,
        );

        let java_node = create_test_node(
            "Java:class:UserService",
            NodeKind::Class,
            "UserService",
            "com.example.api.UserService",
            Language::Java,
        );

        let reference = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Uses,
            target_id: String::new(),
            target_name: "UserService".to_string(),
            target_language: Some(Language::Java),
        };

        assert!(resolver.can_resolve(
            Language::TypeScript,
            Language::Java,
            &ts_node,
            &reference,
            &java_node,
        ));
    }

    #[test]
    fn test_typescript_java_resolver_interface_prefix() {
        let resolver = TypeScriptJavaResolver;

        let ts_node = create_test_node(
            "TypeScript:interface:IUser",
            NodeKind::Interface,
            "IUser",
            "IUser",
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
            target_name: "IUser".to_string(),
            target_language: Some(Language::Java),
        };

        // IUser -> User mapping
        assert!(resolver.can_resolve(
            Language::TypeScript,
            Language::Java,
            &ts_node,
            &reference,
            &java_node,
        ));
    }

    #[test]
    fn test_typescript_java_resolver_interface_prefix_fqn() {
        let resolver = TypeScriptJavaResolver;

        let ts_node = create_test_node(
            "TypeScript:interface:IUserService",
            NodeKind::Interface,
            "IUserService",
            "IUserService",
            Language::TypeScript,
        );

        let java_node = create_test_node(
            "Java:class:UserService",
            NodeKind::Class,
            "UserService",
            "com.example.api.UserService",
            Language::Java,
        );

        let reference = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Uses,
            target_id: String::new(),
            target_name: "IUserService".to_string(),
            target_language: Some(Language::Java),
        };

        // IUserService -> UserService FQN ends_with mapping
        assert!(resolver.can_resolve(
            Language::TypeScript,
            Language::Java,
            &ts_node,
            &reference,
            &java_node,
        ));
    }

    #[test]
    fn test_typescript_java_resolver_wrong_languages() {
        let resolver = TypeScriptJavaResolver;

        let ts_node = create_test_node(
            "TypeScript:class:UserService",
            NodeKind::Class,
            "UserService",
            "UserService",
            Language::TypeScript,
        );

        let python_node = create_test_node(
            "Python:class:UserService",
            NodeKind::Class,
            "UserService",
            "user_service.UserService",
            Language::Python,
        );

        let reference = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Uses,
            target_id: String::new(),
            target_name: "UserService".to_string(),
            target_language: Some(Language::Python),
        };

        // Should return false for non TypeScript-Java combination
        assert!(!resolver.can_resolve(
            Language::TypeScript,
            Language::Python,
            &ts_node,
            &reference,
            &python_node,
        ));
    }

    #[test]
    fn test_java_to_typescript_resolver() {
        let resolver = TypeScriptJavaResolver;

        let java_node = create_test_node(
            "Java:class:JavaService",
            NodeKind::Class,
            "JavaService",
            "com.example.JavaService",
            Language::Java,
        );

        let ts_node = create_test_node(
            "TypeScript:class:TypeScriptClient",
            NodeKind::Class,
            "TypeScriptClient",
            "TypeScriptClient",
            Language::TypeScript,
        );

        let reference = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Uses,
            target_id: String::new(),
            target_name: "TypeScriptClient".to_string(),
            target_language: Some(Language::TypeScript),
        };

        // Java to TypeScript should also work
        assert!(resolver.can_resolve(
            Language::Java,
            Language::TypeScript,
            &java_node,
            &reference,
            &ts_node,
        ));
    }

    // Test JavaKotlinResolver with all edge cases
    #[test]
    fn test_java_kotlin_resolver_direct_name() {
        let resolver = JavaKotlinResolver;

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
            target_name: "User".to_string(),
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

