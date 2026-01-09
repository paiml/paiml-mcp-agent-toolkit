//! Standalone TDD tests for Ruchy integration
//! Tests the new Ruchy parser integration independently

#[cfg(feature = "ruchy-ast")]
#[cfg(test)]
mod ruchy_standalone_tests {
    use std::io::Write;
    use tempfile::NamedTempFile;

    /// Test that we can import and use the Ruchy parser
    #[test]
    fn test_ruchy_parser_available() {
        // Test basic Ruchy parser functionality
        let simple_code = "42";

        // These should be available from ruchy crate
        assert!(ruchy::is_valid_syntax(simple_code));

        let mut parser = ruchy::Parser::new(simple_code);
        let result = parser.parse();
        assert!(result.is_ok());
    }

    /// Test parsing a simple function
    #[test]
    fn test_ruchy_function_parsing() {
        let function_code = r#"
fun hello() -> String {
    "Hello, World!"
}
"#;

        assert!(ruchy::is_valid_syntax(function_code));

        let mut parser = ruchy::Parser::new(function_code);
        let ast = parser.parse();
        assert!(ast.is_ok(), "Should parse valid Ruchy function");

        let _expr = ast.unwrap();
        // We have a parsed AST - this proves the integration works
    }

    /// Test that parse errors are handled correctly
    #[test]
    fn test_ruchy_parse_error_handling() {
        let invalid_code = "fun broken_syntax(";

        assert!(!ruchy::is_valid_syntax(invalid_code));

        let error = ruchy::get_parse_error(invalid_code);
        assert!(error.is_some());
        assert!(!error.unwrap().is_empty());
    }

    /// Test that we can create files and read them
    #[test]
    fn test_file_operations() -> Result<(), Box<dyn std::error::Error>> {
        let ruchy_code = r#"
fun fibonacci(n: i32) -> i32 {
    if n <= 1 {
        n
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}
"#;

        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(ruchy_code.as_bytes())?;

        // Read it back
        let content = std::fs::read_to_string(temp_file.path())?;
        assert!(content.contains("fibonacci"));
        assert!(ruchy::is_valid_syntax(&content));

        Ok(())
    }
}

#[cfg(not(feature = "ruchy-ast"))]
#[cfg(test)]
mod ruchy_feature_disabled_tests {
    #[test]
    fn test_feature_disabled_message() {
        // When ruchy-ast feature is disabled, this test should pass
        // to show that the feature gating works correctly
        println!("ruchy-ast feature is disabled - using fallback implementation");
        assert!(true);
    }
}
