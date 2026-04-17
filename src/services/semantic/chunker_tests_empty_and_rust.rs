    // Empty and Edge Case Tests

    #[test]
    fn test_chunk_code_empty_input() {
        let result = chunk_code("", Language::Rust).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_chunk_code_whitespace_only() {
        let result = chunk_code("   \n\t  \n  ", Language::Rust).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_chunk_code_empty_input_all_languages() {
        for lang in [Language::Rust, Language::TypeScript] {
            let result = chunk_code("", lang).unwrap();
            assert!(
                result.is_empty(),
                "Empty input for {:?} should return empty vec",
                lang
            );
        }
    }

    // Rust Language Tests

    #[test]
    fn test_rust_simple_function() {
        let source = r#"
fn hello_world() {
    println!("Hello, world!");
}
"#;
        let chunks = chunk_code(source, Language::Rust).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_type, ChunkType::Function);
        assert_eq!(chunks[0].chunk_name, "hello_world");
        assert_eq!(chunks[0].language, "rust");
        assert!(chunks[0].content.contains("println!"));
    }

    #[test]
    fn test_rust_function_with_doc_comment() {
        let source = r#"
/// This is a doc comment
/// with multiple lines
fn documented_function() {
    let x = 42;
}
"#;
        let chunks = chunk_code(source, Language::Rust).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_name, "documented_function");
        assert!(chunks[0].content.contains("/// This is a doc comment"));
        assert!(chunks[0].content.contains("/// with multiple lines"));
    }

    #[test]
    fn test_rust_function_with_regular_comment_not_included() {
        let source = r#"
// This is a regular comment (should not be included)
fn regular_function() {
    let x = 42;
}
"#;
        let chunks = chunk_code(source, Language::Rust).unwrap();
        assert_eq!(chunks.len(), 1);
        assert!(!chunks[0].content.contains("// This is a regular comment"));
    }

    #[test]
    fn test_rust_impl_block() {
        let source = r#"
struct MyStruct;

impl MyStruct {
    fn new() -> Self {
        MyStruct
    }

    fn method(&self) {
        println!("method");
    }
}
"#;
        let chunks = chunk_code(source, Language::Rust).unwrap();
        assert!(!chunks.is_empty());

        let impl_chunk = chunks.iter().find(|c| c.chunk_type == ChunkType::Class);
        assert!(impl_chunk.is_some());
        assert_eq!(impl_chunk.unwrap().chunk_name, "MyStruct");
    }

    #[test]
    fn test_rust_module() {
        let source = r#"
mod my_module {
    fn inner_function() {}
}
"#;
        let chunks = chunk_code(source, Language::Rust).unwrap();

        let module_chunk = chunks.iter().find(|c| c.chunk_type == ChunkType::Module);
        assert!(module_chunk.is_some());
        assert_eq!(module_chunk.unwrap().chunk_name, "my_module");
    }

    #[test]
    fn test_rust_multiple_functions() {
        let source = r#"
fn func1() {}
fn func2() {}
fn func3() {}
"#;
        let chunks = chunk_code(source, Language::Rust).unwrap();
        assert_eq!(chunks.len(), 3);

        let names: Vec<&str> = chunks.iter().map(|c| c.chunk_name.as_str()).collect();
        assert!(names.contains(&"func1"));
        assert!(names.contains(&"func2"));
        assert!(names.contains(&"func3"));
    }

    #[test]
    fn test_rust_async_function() {
        let source = r#"
async fn async_function() {
    tokio::time::sleep(Duration::from_secs(1)).await;
}
"#;
        let chunks = chunk_code(source, Language::Rust).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_name, "async_function");
        assert!(chunks[0].content.contains("async fn"));
    }

    #[test]
    fn test_rust_generic_function() {
        let source = r#"
fn generic_function<T: Clone>(value: T) -> T {
    value.clone()
}
"#;
        let chunks = chunk_code(source, Language::Rust).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_name, "generic_function");
        assert!(chunks[0].content.contains("<T: Clone>"));
    }

    #[test]
    fn test_rust_function_line_numbers() {
        let source = "fn line_one() {}\nfn line_two() {}\nfn line_three() {}";
        let chunks = chunk_code(source, Language::Rust).unwrap();
        assert_eq!(chunks.len(), 3);

        assert_eq!(chunks[0].start_line, 1);
        assert_eq!(chunks[1].start_line, 2);
        assert_eq!(chunks[2].start_line, 3);
    }

    #[test]
    fn test_rust_nested_impl_functions() {
        let source = r#"
impl Foo {
    fn method_a(&self) {}
    fn method_b(&mut self) {}
}
"#;
        let chunks = chunk_code(source, Language::Rust).unwrap();
        let functions: Vec<_> = chunks
            .iter()
            .filter(|c| c.chunk_type == ChunkType::Function)
            .collect();
        assert_eq!(functions.len(), 2);
    }
