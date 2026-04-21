    // JavaScriptMapper Tests

    #[test]
    fn test_javascript_mapper_new() {
        let mapper = JavaScriptMapper::new();
        assert_eq!(mapper.language(), Language::JavaScript);
    }

    #[test]
    fn test_javascript_mapper_clone() {
        let mapper = JavaScriptMapper::new();
        let cloned = mapper.clone();
        assert_eq!(cloned.language(), Language::JavaScript);
    }

    #[test]
    fn test_javascript_mapper_clone_box() {
        let mapper = JavaScriptMapper::new();
        let boxed = mapper.clone_box();
        assert_eq!(boxed.language(), Language::JavaScript);
    }

    #[test]
    fn test_javascript_mapper_process_javascript_specific_class() {
        let mapper = JavaScriptMapper::new();
        let mut nodes = vec![UnifiedNode::new(
            NodeKind::Class,
            "MyComponent",
            Language::JavaScript,
        )];

        mapper.process_javascript_specific(&mut nodes);

        assert_eq!(
            nodes[0].metadata.get("javascript:isClass"),
            Some(&"true".to_string())
        );
    }

    #[test]
    fn test_javascript_mapper_process_javascript_specific_arrow_function() {
        let mapper = JavaScriptMapper::new();
        let mut nodes = vec![UnifiedNode::new(
            NodeKind::Function,
            "handleClick",
            Language::JavaScript,
        )];

        // Add arrow modifier
        nodes[0]
            .attributes
            .insert("modifier:arrow".to_string(), "true".to_string());

        mapper.process_javascript_specific(&mut nodes);

        assert_eq!(nodes[0].kind, NodeKind::Lambda);
    }

    #[test]
    fn test_javascript_mapper_process_javascript_specific_regular_function() {
        let mapper = JavaScriptMapper::new();
        let mut nodes = vec![UnifiedNode::new(
            NodeKind::Function,
            "processData",
            Language::JavaScript,
        )];

        mapper.process_javascript_specific(&mut nodes);

        assert_eq!(nodes[0].kind, NodeKind::Function);
        assert!(nodes[0].metadata.is_empty());
    }

    #[test]
    fn test_javascript_mapper_process_javascript_specific_mixed() {
        let mapper = JavaScriptMapper::new();
        let mut nodes = vec![
            UnifiedNode::new(NodeKind::Class, "UserList", Language::JavaScript),
            UnifiedNode::new(NodeKind::Function, "render", Language::JavaScript),
            UnifiedNode::new(NodeKind::Function, "onClick", Language::JavaScript),
            UnifiedNode::new(NodeKind::Module, "utils", Language::JavaScript),
        ];

        nodes[2]
            .attributes
            .insert("modifier:arrow".to_string(), "true".to_string());

        mapper.process_javascript_specific(&mut nodes);

        assert!(nodes[0].metadata.contains_key("javascript:isClass"));
        assert_eq!(nodes[1].kind, NodeKind::Function);
        assert_eq!(nodes[2].kind, NodeKind::Lambda);
        assert!(nodes[3].metadata.is_empty());
    }

    #[test]
    fn test_javascript_mapper_convert_ast_items() {
        let mapper = JavaScriptMapper::new();
        let path = Path::new("/test/app.js");
        let items = vec![
            create_function_item("initialize", false, 1),
            create_module_item("helpers", 20),
        ];

        let nodes = mapper.convert_ast_items(&items, path);

        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].language, Language::JavaScript);
    }

    #[tokio::test]
    async fn test_javascript_mapper_map_file_not_found() {
        let mapper = JavaScriptMapper::new();
        let result = mapper.map_file(Path::new("/nonexistent/script.js")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_javascript_mapper_map_directory() {
        let temp_dir = TempDir::new().unwrap();
        let mapper = JavaScriptMapper::new();

        let result = mapper.map_directory(temp_dir.path(), false).await;
        assert!(result.is_ok());
    }

    /// language_mapper_web.rs:124 — JavaScriptMapper::map_source.
    /// Cannot use `#[tokio::test]`: map_source calls JavaScriptAstVisitor::
    /// analyze_javascript_source which builds its own tokio Runtime::new(),
    /// and nested runtimes panic. map_source body is sync-under-async
    /// (no .await points), so a single noop-poll drives it to completion.
    #[test]
    fn test_javascript_mapper_map_source_executes() {
        use std::future::Future;
        use std::pin::Pin;
        use std::task::{Context, Poll};

        let mapper = JavaScriptMapper::new();
        let source = "class Widget {}\nconst arrow = () => 42;\nfunction plain() {}\n";
        let fut = mapper.map_source(source, Path::new("widget.js"));
        let mut fut = Box::pin(fut);
        let waker = futures_test::task::noop_waker();
        let mut cx = Context::from_waker(&waker);
        match Pin::as_mut(&mut fut).poll(&mut cx) {
            Poll::Ready(_) => {}
            Poll::Pending => panic!("map_source must complete in one poll"),
        }
    }

    /// language_mapper_web.rs:124 — JavaScriptMapper::map_source with garbage
    /// input. Drives the function body + match regardless of Ok/Err variant.
    #[test]
    fn test_javascript_mapper_map_source_invalid_source() {
        use std::future::Future;
        use std::pin::Pin;
        use std::task::{Context, Poll};

        let mapper = JavaScriptMapper::new();
        let source = "@@@ function ((( { unterminated";
        let fut = mapper.map_source(source, Path::new("bad.js"));
        let mut fut = Box::pin(fut);
        let waker = futures_test::task::noop_waker();
        let mut cx = Context::from_waker(&waker);
        match Pin::as_mut(&mut fut).poll(&mut cx) {
            Poll::Ready(_) => {}
            Poll::Pending => panic!("map_source must complete in one poll"),
        }
    }

    // CSharpMapper Tests

    #[test]
    fn test_csharp_mapper_new() {
        let mapper = CSharpMapper::new();
        assert_eq!(mapper.language(), Language::CSharp);
    }

    #[test]
    fn test_csharp_mapper_clone() {
        let mapper = CSharpMapper::new();
        let cloned = mapper.clone();
        assert_eq!(cloned.language(), Language::CSharp);
    }

    #[test]
    fn test_csharp_mapper_clone_box() {
        let mapper = CSharpMapper::new();
        let boxed = mapper.clone_box();
        assert_eq!(boxed.language(), Language::CSharp);
    }

    #[test]
    fn test_csharp_mapper_convert_ast_items() {
        let mapper = CSharpMapper::new();
        let path = Path::new("/test/Program.cs");
        let items = vec![
            create_function_item("Main", false, 1),
            create_struct_item("Config", 5, vec![], 10),
        ];

        let nodes = mapper.convert_ast_items(&items, path);

        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].language, Language::CSharp);
    }

    #[tokio::test]
    async fn test_csharp_mapper_map_source() {
        let mapper = CSharpMapper::new();
        let source = "public class Test { }";
        let result = mapper.map_source(source, Path::new("Test.cs")).await;

        // Uses base implementation which returns error
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_csharp_mapper_map_file_not_found() {
        let mapper = CSharpMapper::new();
        let result = mapper.map_file(Path::new("/nonexistent/Program.cs")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_csharp_mapper_map_directory() {
        let temp_dir = TempDir::new().unwrap();
        let mapper = CSharpMapper::new();

        let result = mapper.map_directory(temp_dir.path(), true).await;
        assert!(result.is_ok());
    }

    // RubyMapper Tests

    #[test]
    fn test_ruby_mapper_new() {
        let mapper = RubyMapper::new();
        assert_eq!(mapper.language(), Language::Ruby);
    }

    #[test]
    fn test_ruby_mapper_clone() {
        let mapper = RubyMapper::new();
        let cloned = mapper.clone();
        assert_eq!(cloned.language(), Language::Ruby);
    }

    #[test]
    fn test_ruby_mapper_clone_box() {
        let mapper = RubyMapper::new();
        let boxed = mapper.clone_box();
        assert_eq!(boxed.language(), Language::Ruby);
    }

    #[test]
    fn test_ruby_mapper_convert_ast_items() {
        let mapper = RubyMapper::new();
        let path = Path::new("/test/app.rb");
        let items = vec![
            create_function_item("initialize", false, 1),
            create_module_item("Helpers", 20),
        ];

        let nodes = mapper.convert_ast_items(&items, path);

        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].language, Language::Ruby);
    }

    #[tokio::test]
    async fn test_ruby_mapper_map_source() {
        let mapper = RubyMapper::new();
        let source = "class User; def name; @name; end; end";
        let result = mapper.map_source(source, Path::new("user.rb")).await;

        // Uses base implementation which returns error
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_ruby_mapper_map_file_not_found() {
        let mapper = RubyMapper::new();
        let result = mapper.map_file(Path::new("/nonexistent/app.rb")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_ruby_mapper_map_directory() {
        let temp_dir = TempDir::new().unwrap();
        let mapper = RubyMapper::new();

        let result = mapper.map_directory(temp_dir.path(), false).await;
        assert!(result.is_ok());
    }

