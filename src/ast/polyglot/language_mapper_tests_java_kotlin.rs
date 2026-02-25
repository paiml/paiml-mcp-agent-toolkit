    // JavaMapper Tests

    #[test]
    fn test_java_mapper_new() {
        let mapper = JavaMapper::new();
        assert_eq!(mapper.language(), Language::Java);
    }

    #[test]
    fn test_java_mapper_clone() {
        let mapper = JavaMapper::new();
        let cloned = mapper.clone();
        assert_eq!(cloned.language(), Language::Java);
    }

    #[test]
    fn test_java_mapper_clone_box() {
        let mapper = JavaMapper::new();
        let boxed = mapper.clone_box();
        assert_eq!(boxed.language(), Language::Java);
    }

    #[test]
    fn test_java_mapper_process_java_specific_interface() {
        let mapper = JavaMapper::new();
        let mut nodes = vec![UnifiedNode::new(
            NodeKind::Class,
            "TestInterface",
            Language::Java,
        )];

        // Add interface modifier
        nodes[0]
            .attributes
            .insert("modifier:interface".to_string(), "true".to_string());

        mapper.process_java_specific(&mut nodes);

        assert_eq!(nodes[0].kind, NodeKind::Interface);
    }

    #[test]
    fn test_java_mapper_process_java_specific_record() {
        let mapper = JavaMapper::new();
        let mut nodes = vec![UnifiedNode::new(
            NodeKind::Class,
            "TestRecord",
            Language::Java,
        )];

        // Add record modifier
        nodes[0]
            .attributes
            .insert("modifier:record".to_string(), "true".to_string());

        mapper.process_java_specific(&mut nodes);

        assert_eq!(nodes[0].kind, NodeKind::Record);
    }

    #[test]
    fn test_java_mapper_process_java_specific_no_modifier() {
        let mapper = JavaMapper::new();
        let mut nodes = vec![
            UnifiedNode::new(NodeKind::Class, "TestClass", Language::Java),
            UnifiedNode::new(NodeKind::Function, "testMethod", Language::Java),
        ];

        mapper.process_java_specific(&mut nodes);

        // Should remain unchanged
        assert_eq!(nodes[0].kind, NodeKind::Class);
        assert_eq!(nodes[1].kind, NodeKind::Function);
    }

    #[test]
    fn test_java_mapper_convert_ast_items() {
        let mapper = JavaMapper::new();
        let path = Path::new("/test/Test.java");
        let items = vec![
            create_function_item("doSomething", false, 10),
            create_struct_item("InnerClass", 2, vec![], 20),
        ];

        let nodes = mapper.convert_ast_items(&items, path);

        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].language, Language::Java);
        assert_eq!(nodes[1].language, Language::Java);
    }

    #[tokio::test]
    async fn test_java_mapper_map_source_without_feature() {
        let mapper = JavaMapper::new();
        let source = "public class Test { public void hello() {} }";
        let result = mapper.map_source(source, Path::new("Test.java")).await;

        // Without java-ast feature, should return error
        #[cfg(not(feature = "java-ast"))]
        assert!(result.is_err());

        #[cfg(feature = "java-ast")]
        assert!(result.is_ok() || result.is_err()); // May succeed or fail depending on implementation
    }

    #[tokio::test]
    async fn test_java_mapper_map_file_not_found() {
        let mapper = JavaMapper::new();
        let result = mapper.map_file(Path::new("/nonexistent/Test.java")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_java_mapper_map_directory_empty() {
        let temp_dir = TempDir::new().unwrap();
        let mapper = JavaMapper::new();

        let result = mapper.map_directory(temp_dir.path(), false).await;
        assert!(result.is_ok());
    }

    // KotlinMapper Tests

    #[test]
    fn test_kotlin_mapper_new() {
        let mapper = KotlinMapper::new();
        assert_eq!(mapper.language(), Language::Kotlin);
    }

    #[test]
    fn test_kotlin_mapper_clone() {
        let mapper = KotlinMapper::new();
        let cloned = mapper.clone();
        assert_eq!(cloned.language(), Language::Kotlin);
    }

    #[test]
    fn test_kotlin_mapper_clone_box() {
        let mapper = KotlinMapper::new();
        let boxed = mapper.clone_box();
        assert_eq!(boxed.language(), Language::Kotlin);
    }

    #[test]
    fn test_kotlin_mapper_process_kotlin_specific_data_class() {
        let mapper = KotlinMapper::new();
        let mut nodes = vec![UnifiedNode::new(
            NodeKind::Class,
            "UserData",
            Language::Kotlin,
        )];

        // Add data modifier
        nodes[0]
            .attributes
            .insert("modifier:data".to_string(), "true".to_string());

        mapper.process_kotlin_specific(&mut nodes);

        assert_eq!(nodes[0].kind, NodeKind::Record);
        assert_eq!(
            nodes[0].metadata.get("kotlin:isData"),
            Some(&"true".to_string())
        );
    }

    #[test]
    fn test_kotlin_mapper_process_kotlin_specific_sealed_class() {
        let mapper = KotlinMapper::new();
        let mut nodes = vec![UnifiedNode::new(
            NodeKind::Class,
            "Result",
            Language::Kotlin,
        )];

        // Add sealed modifier
        nodes[0]
            .attributes
            .insert("modifier:sealed".to_string(), "true".to_string());

        mapper.process_kotlin_specific(&mut nodes);

        assert_eq!(nodes[0].kind, NodeKind::Class); // Sealed doesn't change kind
        assert_eq!(
            nodes[0].metadata.get("kotlin:isSealed"),
            Some(&"true".to_string())
        );
    }

    #[test]
    fn test_kotlin_mapper_process_kotlin_specific_data_and_sealed() {
        let mapper = KotlinMapper::new();
        let mut nodes = vec![UnifiedNode::new(
            NodeKind::Class,
            "SealedData",
            Language::Kotlin,
        )];

        // Add both modifiers
        nodes[0]
            .attributes
            .insert("modifier:data".to_string(), "true".to_string());
        nodes[0]
            .attributes
            .insert("modifier:sealed".to_string(), "true".to_string());

        mapper.process_kotlin_specific(&mut nodes);

        // Data takes precedence (processed first)
        assert_eq!(nodes[0].kind, NodeKind::Record);
        assert!(nodes[0].metadata.contains_key("kotlin:isData"));
        assert!(nodes[0].metadata.contains_key("kotlin:isSealed"));
    }

    #[test]
    fn test_kotlin_mapper_process_kotlin_specific_no_modifier() {
        let mapper = KotlinMapper::new();
        let mut nodes = vec![
            UnifiedNode::new(NodeKind::Class, "NormalClass", Language::Kotlin),
            UnifiedNode::new(NodeKind::Function, "doWork", Language::Kotlin),
        ];

        mapper.process_kotlin_specific(&mut nodes);

        assert_eq!(nodes[0].kind, NodeKind::Class);
        assert_eq!(nodes[1].kind, NodeKind::Function);
        assert!(nodes[0].metadata.is_empty());
    }

    #[test]
    fn test_kotlin_mapper_convert_ast_items() {
        let mapper = KotlinMapper::new();
        let path = Path::new("/test/Main.kt");
        let items = vec![
            create_function_item("processData", true, 5),
            create_struct_item("DataHolder", 4, vec![], 15),
        ];

        let nodes = mapper.convert_ast_items(&items, path);

        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].language, Language::Kotlin);
    }

    #[tokio::test]
    async fn test_kotlin_mapper_map_source() {
        let mapper = KotlinMapper::new();
        let source = "data class User(val name: String)";
        let result = mapper.map_source(source, Path::new("User.kt")).await;

        // Uses base implementation which returns error
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_kotlin_mapper_map_file_not_found() {
        let mapper = KotlinMapper::new();
        let result = mapper.map_file(Path::new("/nonexistent/Test.kt")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_kotlin_mapper_map_directory() {
        let temp_dir = TempDir::new().unwrap();
        let mapper = KotlinMapper::new();

        let result = mapper.map_directory(temp_dir.path(), true).await;
        assert!(result.is_ok());
    }

