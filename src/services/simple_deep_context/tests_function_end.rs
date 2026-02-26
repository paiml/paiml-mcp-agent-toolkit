    // ============ find_function_end Tests ============

    #[test]
    fn test_find_function_end_python() {
        let analyzer = SimpleDeepContext;

        // Test Python function detection by indentation
        let code = r#"def my_func():
    print("hello")
    if True:
        print("nested")
    return 42

def next_func():
    pass"#;

        let result = analyzer.find_function_end(code, "py");
        assert!(result.is_some());
        let end_pos = result.unwrap();
        // The end should be before "def next_func"
        assert!(end_pos < code.len());
    }

    #[test]
    fn test_find_function_end_python_empty() {
        let analyzer = SimpleDeepContext;
        let code = "";
        let result = analyzer.find_function_end(code, "py");
        assert!(result.is_none());
    }

    #[test]
    fn test_find_function_end_c_style() {
        let analyzer = SimpleDeepContext;

        let code = r#"void my_func() {
    if (true) {
        printf("hello");
    }
}

void next_func() {"#;

        let result = analyzer.find_function_end(code, "c");
        assert!(result.is_some());
        let end_pos = result.unwrap();
        // Should end at the closing brace of my_func
        assert!(end_pos < code.len());
    }

    #[test]
    fn test_find_function_end_c_with_strings() {
        let analyzer = SimpleDeepContext;

        // Test handling of braces inside strings
        let code = r#"void my_func() {
    printf("{ this brace is in a string }");
}

void next();"#;

        let result = analyzer.find_function_end(code, "cpp");
        assert!(result.is_some());
    }

    #[test]
    fn test_find_function_end_unmatched_braces() {
        let analyzer = SimpleDeepContext;

        // Test with unmatched braces (should return None)
        let code = r#"void my_func() {
    if (true) {
        printf("hello");
    // missing closing brace"#;

        let result = analyzer.find_function_end(code, "c");
        assert!(result.is_none());
    }

    #[test]
    fn test_find_function_end_js() {
        let analyzer = SimpleDeepContext;

        let code = r#"function myFunc() {
    const x = { nested: "object" };
    return x;
}

const other = () => {"#;

        let result = analyzer.find_function_end(code, "js");
        assert!(result.is_some());
    }
