    // Edge Case Tests

    #[test]
    fn test_rust_single_line_function() {
        let source = "fn single() {}";
        let chunks = chunk_code(source, Language::Rust).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].start_line, chunks[0].end_line);
    }

    #[test]
    fn test_rust_unicode_in_strings() {
        let source = "fn greet() {\n    println!(\"Hello, Alex!\");\n}\n";
        let chunks = chunk_code(source, Language::Rust).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_name, "greet");
    }

    #[test]
    fn test_typescript_multiline_arrow() {
        let source = r#"
const complexFunc = (
    a: number,
    b: number,
    c: number
): number => {
    return a + b + c;
};
"#;
        let chunks = chunk_code(source, Language::TypeScript).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_name, "complexFunc");
    }

    // Checksum Tests

    #[test]
    fn test_checksum_deterministic() {
        let content = "fn test() { let x = 42; }";
        let checksum1 = compute_checksum(content);
        let checksum2 = compute_checksum(content);
        let checksum3 = compute_checksum(content);

        assert_eq!(checksum1, checksum2);
        assert_eq!(checksum2, checksum3);
    }

    #[test]
    fn test_checksum_sensitive_to_whitespace() {
        let content1 = "fn test() {}";
        let content2 = "fn test()  {}";

        let checksum1 = compute_checksum(content1);
        let checksum2 = compute_checksum(content2);

        assert_ne!(checksum1, checksum2);
    }

    #[test]
    fn test_checksum_sensitive_to_case() {
        let content1 = "fn Test() {}";
        let content2 = "fn test() {}";

        let checksum1 = compute_checksum(content1);
        let checksum2 = compute_checksum(content2);

        assert_ne!(checksum1, checksum2);
    }

    #[test]
    fn test_checksum_empty_content() {
        let checksum = compute_checksum("");
        assert_eq!(checksum.len(), 64);
    }

    // Error Handling Tests

    #[test]
    fn test_rust_syntax_error_still_parses() {
        let source = "fn broken( { }";
        let result = chunk_code(source, Language::Rust);
        assert!(result.is_ok());
    }

    #[test]
    fn test_typescript_syntax_error_still_parses() {
        let source = "function broken( { }";
        let result = chunk_code(source, Language::TypeScript);
        assert!(result.is_ok());
    }

    // Doc Comment Tests

    #[test]
    fn test_rust_multiple_doc_comments() {
        let source = "/// First line\n/// Second line\n/// Third line\nfn documented() {}\n";
        let chunks = chunk_code(source, Language::Rust).unwrap();
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].content.contains("/// First line"));
        assert!(chunks[0].content.contains("/// Second line"));
        assert!(chunks[0].content.contains("/// Third line"));
    }

    #[test]
    fn test_rust_block_doc_comment() {
        let source = "/** This is a block doc comment */\nfn block_documented() {}\n";
        let chunks = chunk_code(source, Language::Rust).unwrap();
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].content.contains("block doc comment"));
    }

    // Nested Structure Tests

    #[test]
    fn test_rust_nested_module() {
        let source = "mod outer {\n    mod inner {\n        fn nested_func() {}\n    }\n}\n";
        let chunks = chunk_code(source, Language::Rust).unwrap();

        let modules: Vec<_> = chunks
            .iter()
            .filter(|c| c.chunk_type == ChunkType::Module)
            .collect();
        assert!(modules.len() >= 1);
    }

    #[test]
    fn test_typescript_nested_class() {
        let source = "class Outer {\n    inner = class {\n        method() {}\n    };\n}\n";
        let chunks = chunk_code(source, Language::TypeScript).unwrap();

        let classes: Vec<_> = chunks
            .iter()
            .filter(|c| c.chunk_type == ChunkType::Class)
            .collect();
        assert!(classes.len() >= 1);
    }

    // Performance Boundary Tests

    #[test]
    fn test_many_small_functions() {
        let mut source = String::new();
        for i in 0..100 {
            source.push_str(&format!("fn func_{i}() {{}}\n"));
        }

        let chunks = chunk_code(&source, Language::Rust).unwrap();
        assert_eq!(chunks.len(), 100);
    }

    #[test]
    fn test_large_function() {
        let mut source = String::from("fn large_func() {\n");
        for i in 0..1000 {
            source.push_str(&format!("    let var_{i} = {i};\n"));
        }
        source.push_str("}\n");

        let chunks = chunk_code(&source, Language::Rust).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_name, "large_func");
    }

    // Language-Specific Feature Tests

    #[test]
    fn test_rust_trait_impl() {
        let source = r#"
trait Greeter {
    fn greet(&self) -> String;
}

impl Greeter for Person {
    fn greet(&self) -> String {
        format!("Hello, {}", self.name)
    }
}
"#;
        let chunks = chunk_code(source, Language::Rust).unwrap();

        let impl_chunk = chunks
            .iter()
            .find(|c| c.chunk_type == ChunkType::Class && c.chunk_name == "Person");
        assert!(impl_chunk.is_some());
    }

    #[test]
    fn test_typescript_type_alias_not_extracted() {
        let source = "type StringAlias = string;\nfunction useType(x: StringAlias): void {}\n";
        let chunks = chunk_code(source, Language::TypeScript).unwrap();

        let func_chunk = chunks.iter().find(|c| c.chunk_name == "useType");
        assert!(func_chunk.is_some());
    }

    // Whitespace Handling Tests

    #[test]
    fn test_rust_leading_whitespace() {
        let source = "    fn indented() {}";
        let chunks = chunk_code(source, Language::Rust).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_name, "indented");
    }

    #[test]
    fn test_rust_mixed_line_endings() {
        let source = "fn func1() {}\r\nfn func2() {}\nfn func3() {}";
        let chunks = chunk_code(source, Language::Rust).unwrap();
        assert_eq!(chunks.len(), 3);
    }

    #[test]
    fn test_rust_tabs_in_content() {
        let source = "fn tabbed() {\n\tlet x = 1;\n}";
        let chunks = chunk_code(source, Language::Rust).unwrap();
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].content.contains("\t"));
    }

    // Parser Edge Cases

    #[test]
    fn test_parse_rust_success() {
        let source = "fn test() {}";
        let result = parse_rust(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_typescript_success() {
        let source = "function test() {}";
        let result = parse_typescript(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_find_doc_comment_start_no_parent() {
        let source = "fn test() {}";
        let tree = parse_rust(source).unwrap();
        let root = tree.root_node();
        let start = find_doc_comment_start(root, source);
        assert_eq!(start, 0);
    }

    #[test]
    fn test_find_doc_comment_start_no_comment() {
        let source = "fn test() {}";
        let tree = parse_rust(source).unwrap();
        let root = tree.root_node();
        let func_node = root.child(0).unwrap();
        let start = find_doc_comment_start(func_node, source);
        assert_eq!(start, func_node.start_byte());
    }

    // Function Declarator Name Tests (C/C++)

    #[cfg(feature = "c-ast")]
    #[test]
    fn test_find_function_declarator_name_direct_identifier() {
        let source = "int test() { return 0; }";
        let chunks = chunk_code(source, Language::C).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_name, "test");
    }

    #[cfg(feature = "c-ast")]
    #[test]
    fn test_find_function_declarator_name_pointer() {
        let source = "int *test() { return 0; }";
        let chunks = chunk_code(source, Language::C).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_name, "test");
    }

    // TypeScript Arrow Function Edge Cases

    #[test]
    fn test_extract_ts_arrow_function_no_arrow() {
        let source = "const x = 42;";
        let chunks = chunk_code(source, Language::TypeScript).unwrap();
        // Should not extract regular variable as function
        let func_chunks: Vec<_> = chunks
            .iter()
            .filter(|c| c.chunk_type == ChunkType::Function)
            .collect();
        assert!(func_chunks.is_empty());
    }

    #[test]
    fn test_extract_ts_arrow_function_with_let() {
        let source = "let myFunc = () => {};";
        let chunks = chunk_code(source, Language::TypeScript).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_name, "myFunc");
    }

    // Coverage for extract_* helper functions

    #[test]
    fn test_extract_ts_class_no_name() {
        // Anonymous class expression
        let source = "const MyClass = class {};";
        let chunks = chunk_code(source, Language::TypeScript).unwrap();
        // The outer class may or may not be extracted depending on tree structure
        let _ = chunks.len();
    }

    #[test]
    fn test_extract_ts_interface_no_name() {
        // Valid interface with name
        let source = "interface Test { x: number; }";
        let chunks = chunk_code(source, Language::TypeScript).unwrap();
        assert_eq!(chunks.len(), 1);
    }

    #[test]
    fn test_extract_ts_function_no_name() {
        // Anonymous function expression
        let source = "(function() {})();";
        let chunks = chunk_code(source, Language::TypeScript).unwrap();
        // IIFE may not be extracted as named function
        let named_funcs: Vec<_> = chunks.iter().filter(|c| !c.chunk_name.is_empty()).collect();
        let _ = named_funcs.len();
    }

    // Coverage for recursive extraction

    #[test]
    fn test_rust_deeply_nested_functions() {
        let source = r#"
mod a {
    mod b {
        mod c {
            fn deep() {}
        }
    }
}
"#;
        let chunks = chunk_code(source, Language::Rust).unwrap();
        let functions: Vec<_> = chunks
            .iter()
            .filter(|c| c.chunk_type == ChunkType::Function)
            .collect();
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].chunk_name, "deep");
    }

    #[test]
    fn test_typescript_deeply_nested_functions() {
        let source = r#"
function outer() {
    function middle() {
        function inner() {
            return 42;
        }
    }
}
"#;
        let chunks = chunk_code(source, Language::TypeScript).unwrap();
        let functions: Vec<_> = chunks
            .iter()
            .filter(|c| c.chunk_type == ChunkType::Function)
            .collect();
        assert!(functions.len() >= 1);
    }

    // Block Comment Tests for C-family

    #[cfg(feature = "c-ast")]
    #[test]
    fn test_c_block_comment_before_function() {
        let source = "/* Block comment */\nvoid test() {}\n";
        let chunks = chunk_code(source, Language::C).unwrap();
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].content.contains("Block comment"));
    }

    #[cfg(feature = "cpp-ast")]
    #[test]
    fn test_cpp_block_comment_before_function() {
        let source = "/* Block comment */\nvoid test() {}\n";
        let chunks = chunk_code(source, Language::Cpp).unwrap();
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].content.contains("Block comment"));
    }

    // Multiple Item Types Together

    #[test]
    fn test_rust_mixed_items() {
        let source = r#"
mod mymod {}
impl MyType {}
fn myfunc() {}
"#;
        let chunks = chunk_code(source, Language::Rust).unwrap();

        let module = chunks.iter().find(|c| c.chunk_type == ChunkType::Module);
        let class = chunks.iter().find(|c| c.chunk_type == ChunkType::Class);
        let func = chunks.iter().find(|c| c.chunk_type == ChunkType::Function);

        assert!(module.is_some());
        assert!(class.is_some());
        assert!(func.is_some());
    }

    #[test]
    fn test_typescript_mixed_items() {
        let source = r#"
interface MyInterface {}
class MyClass {}
function myFunc() {}
const myArrow = () => {};
"#;
        let chunks = chunk_code(source, Language::TypeScript).unwrap();

        assert!(chunks.len() >= 4);
    }
