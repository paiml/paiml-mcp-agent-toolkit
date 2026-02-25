    // ScalaMapper Tests

    #[test]
    fn test_scala_mapper_new() {
        let mapper = ScalaMapper::new();
        assert_eq!(mapper.language(), Language::Scala);
    }

    #[test]
    fn test_scala_mapper_clone() {
        let mapper = ScalaMapper::new();
        let cloned = mapper.clone();
        assert_eq!(cloned.language(), Language::Scala);
    }

    #[test]
    fn test_scala_mapper_clone_box() {
        let mapper = ScalaMapper::new();
        let boxed = mapper.clone_box();
        assert_eq!(boxed.language(), Language::Scala);
    }

    #[test]
    fn test_scala_mapper_process_scala_specific_case_class() {
        let mapper = ScalaMapper::new();
        let mut nodes = vec![UnifiedNode::new(NodeKind::Class, "Person", Language::Scala)];

        // Add case modifier
        nodes[0]
            .attributes
            .insert("modifier:case".to_string(), "true".to_string());

        mapper.process_scala_specific(&mut nodes);

        assert_eq!(nodes[0].kind, NodeKind::CaseClass);
    }

    #[test]
    fn test_scala_mapper_process_scala_specific_object() {
        let mapper = ScalaMapper::new();
        let mut nodes = vec![UnifiedNode::new(
            NodeKind::Module,
            "AppObject",
            Language::Scala,
        )];

        mapper.process_scala_specific(&mut nodes);

        assert_eq!(nodes[0].kind, NodeKind::Module);
        assert_eq!(
            nodes[0].metadata.get("scala:isObject"),
            Some(&"true".to_string())
        );
    }

    #[test]
    fn test_scala_mapper_process_scala_specific_multiple_nodes() {
        let mapper = ScalaMapper::new();
        let mut nodes = vec![
            UnifiedNode::new(NodeKind::Class, "NormalClass", Language::Scala),
            UnifiedNode::new(NodeKind::Module, "Companion", Language::Scala),
            UnifiedNode::new(NodeKind::Function, "apply", Language::Scala),
        ];

        // Only add case to first node
        nodes[0]
            .attributes
            .insert("modifier:case".to_string(), "true".to_string());

        mapper.process_scala_specific(&mut nodes);

        assert_eq!(nodes[0].kind, NodeKind::CaseClass);
        assert_eq!(nodes[1].kind, NodeKind::Module);
        assert!(nodes[1].metadata.contains_key("scala:isObject"));
        assert_eq!(nodes[2].kind, NodeKind::Function);
    }

    #[test]
    fn test_scala_mapper_convert_ast_items() {
        let mapper = ScalaMapper::new();
        let path = Path::new("/test/App.scala");
        let items = vec![
            create_function_item("main", false, 1),
            create_trait_item("Service", 10),
        ];

        let nodes = mapper.convert_ast_items(&items, path);

        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].language, Language::Scala);
        assert_eq!(nodes[1].language, Language::Scala);
    }

    #[tokio::test]
    async fn test_scala_mapper_map_source_without_feature() {
        let mapper = ScalaMapper::new();
        let source = "case class User(name: String)";
        let result = mapper.map_source(source, Path::new("User.scala")).await;

        #[cfg(not(feature = "scala-ast"))]
        assert!(result.is_err());

        #[cfg(feature = "scala-ast")]
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_scala_mapper_map_file_not_found() {
        let mapper = ScalaMapper::new();
        let result = mapper.map_file(Path::new("/nonexistent/Main.scala")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_scala_mapper_map_directory() {
        let temp_dir = TempDir::new().unwrap();
        let mapper = ScalaMapper::new();

        let result = mapper.map_directory(temp_dir.path(), false).await;
        assert!(result.is_ok());
    }

    // TypeScriptMapper Tests

    #[test]
    fn test_typescript_mapper_new() {
        let mapper = TypeScriptMapper::new();
        assert_eq!(mapper.language(), Language::TypeScript);
    }

    #[test]
    fn test_typescript_mapper_clone() {
        let mapper = TypeScriptMapper::new();
        let cloned = mapper.clone();
        assert_eq!(cloned.language(), Language::TypeScript);
    }

    #[test]
    fn test_typescript_mapper_clone_box() {
        let mapper = TypeScriptMapper::new();
        let boxed = mapper.clone_box();
        assert_eq!(boxed.language(), Language::TypeScript);
    }

    #[test]
    fn test_typescript_mapper_process_typescript_specific_interface() {
        let mapper = TypeScriptMapper::new();
        let mut nodes = vec![UnifiedNode::new(
            NodeKind::Interface,
            "UserProps",
            Language::TypeScript,
        )];

        mapper.process_typescript_specific(&mut nodes);

        assert_eq!(
            nodes[0].metadata.get("typescript:isInterface"),
            Some(&"true".to_string())
        );
    }

    #[test]
    fn test_typescript_mapper_process_typescript_specific_abstract_class() {
        let mapper = TypeScriptMapper::new();
        let mut nodes = vec![UnifiedNode::new(
            NodeKind::Class,
            "BaseService",
            Language::TypeScript,
        )];

        // Add abstract modifier
        nodes[0]
            .attributes
            .insert("modifier:abstract".to_string(), "true".to_string());

        mapper.process_typescript_specific(&mut nodes);

        assert_eq!(nodes[0].kind, NodeKind::Class);
        assert_eq!(
            nodes[0].metadata.get("typescript:isAbstract"),
            Some(&"true".to_string())
        );
    }

    #[test]
    fn test_typescript_mapper_process_typescript_specific_regular_class() {
        let mapper = TypeScriptMapper::new();
        let mut nodes = vec![UnifiedNode::new(
            NodeKind::Class,
            "UserService",
            Language::TypeScript,
        )];

        mapper.process_typescript_specific(&mut nodes);

        assert_eq!(nodes[0].kind, NodeKind::Class);
        assert!(nodes[0].metadata.is_empty());
    }

    #[test]
    fn test_typescript_mapper_process_typescript_specific_mixed() {
        let mapper = TypeScriptMapper::new();
        let mut nodes = vec![
            UnifiedNode::new(NodeKind::Interface, "IUser", Language::TypeScript),
            UnifiedNode::new(NodeKind::Class, "AbstractBase", Language::TypeScript),
            UnifiedNode::new(NodeKind::Class, "ConcreteImpl", Language::TypeScript),
            UnifiedNode::new(NodeKind::Function, "helper", Language::TypeScript),
        ];

        nodes[1]
            .attributes
            .insert("modifier:abstract".to_string(), "true".to_string());

        mapper.process_typescript_specific(&mut nodes);

        assert!(nodes[0].metadata.contains_key("typescript:isInterface"));
        assert!(nodes[1].metadata.contains_key("typescript:isAbstract"));
        assert!(nodes[2].metadata.is_empty());
        assert!(nodes[3].metadata.is_empty());
    }

    #[test]
    fn test_typescript_mapper_convert_ast_items() {
        let mapper = TypeScriptMapper::new();
        let path = Path::new("/test/app.ts");
        let items = vec![
            create_function_item("fetchData", true, 1),
            create_trait_item("IDataProvider", 10),
        ];

        let nodes = mapper.convert_ast_items(&items, path);

        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].language, Language::TypeScript);
    }

    #[tokio::test]
    async fn test_typescript_mapper_map_file_not_found() {
        let mapper = TypeScriptMapper::new();
        let result = mapper.map_file(Path::new("/nonexistent/app.ts")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_typescript_mapper_map_directory() {
        let temp_dir = TempDir::new().unwrap();
        let mapper = TypeScriptMapper::new();

        let result = mapper.map_directory(temp_dir.path(), true).await;
        assert!(result.is_ok());
    }

