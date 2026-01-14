#[cfg(test)]
mod tests {
    use super::super::assemblyscript::*;
    use proptest::prelude::*;
    use std::io::Write;
    use std::path::PathBuf;
    use std::time::Duration;
    use tempfile::NamedTempFile;

    proptest! {
        fn parser_never_panics_on_arbitrary_input(
            input in ".*"
        ) {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let parser = AssemblyScriptParser::new();
                if let Ok(mut p) = parser {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    let path = PathBuf::from("test.as");
                    rt.block_on(p.parse_file(&path, &input))
                } else {
                    Err(anyhow::anyhow!("Failed to create parser"))
                }
            }));

            prop_assert!(result.is_ok());
        }

        fn valid_assemblyscript_parses_successfully(
            func_count in 0usize..10
        ) {
            let mut code = String::new();
            for i in 0..func_count {
                code.push_str(&format!("function test{i}(): i32 {{ return {i}; }}\n\n"));
            }
            let parser = AssemblyScriptParser::new();
            prop_assert!(parser.is_ok());

            let mut parser = parser.unwrap();
            let runtime = tokio::runtime::Runtime::new().unwrap();
            let path = PathBuf::from("test.as");

            let result = runtime.block_on(parser.parse_file(&path, &code));

            // Should successfully parse valid AssemblyScript
            prop_assert!(result.is_ok(), "Failed to parse valid AssemblyScript: {}", code);
        }

        fn parser_respects_size_limits(
            repeat_count in 1usize..2000,
        ) {
            let base_code = "function test(): i32 { return 42; }\n";
            let large_code = base_code.repeat(repeat_count);

            let parser = AssemblyScriptParser::new();
            prop_assert!(parser.is_ok());

            let mut parser = parser.unwrap();
            let runtime = tokio::runtime::Runtime::new().unwrap();
            let path = PathBuf::from("test.as");

            let result = runtime.block_on(parser.parse_file(&path, &large_code));

            if large_code.len() > 10 * 1024 * 1024 {
                // Should reject files larger than 10MB
                prop_assert!(result.is_err());
                if let Err(e) = result {
                    prop_assert!(e.to_string().contains("too large"));
                }
            } else {
                // Should accept files under 10MB
                prop_assert!(result.is_ok());
            }
        }

        fn complexity_analysis_never_panics(
            input in ".*"
        ) {
            let parser = AssemblyScriptParser::new_with_timeout(Duration::from_secs(1));

            // Should not panic
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                parser.analyze_complexity(&input)
            }));

            prop_assert!(result.is_ok());
        }

        fn complexity_increases_with_functions(
            function_count in 0usize..50,
        ) {
            let mut code = String::new();
            for i in 0..function_count {
                code.push_str(&format!("function test{}(): i32 {{ return {}; }}\n", i, i));
            }

            let parser = AssemblyScriptParser::new_with_timeout(Duration::from_secs(1));
            let complexity = parser.analyze_complexity(&code);

            prop_assert!(complexity.is_ok());
            let complexity = complexity.unwrap();

            // Complexity should increase with more functions
            if function_count > 0 {
                prop_assert!(complexity.cyclomatic > 0);
                prop_assert!(complexity.cognitive > 0);

                // Check that complexity increases roughly with function count
                let expected_min = function_count as u32 * 2;
                prop_assert!(complexity.cyclomatic >= expected_min);
            }
        }

        fn complexity_bounds_are_reasonable(
            func_count in 0usize..50
        ) {
            let mut code = String::new();
            for i in 0..func_count {
                code.push_str(&format!("function test{}(): i32 {{ return {}; }}\n", i, i));
            }
            let parser = AssemblyScriptParser::new_with_timeout(Duration::from_secs(1));
            let complexity = parser.analyze_complexity(&code);

            prop_assert!(complexity.is_ok());
            let complexity = complexity.unwrap();

            // Complexity values should be within reasonable bounds
            prop_assert!(complexity.cyclomatic <= 10000);
            prop_assert!(complexity.cognitive <= 10000);
            prop_assert!(complexity.memory_pressure >= 0.0);
            prop_assert!(complexity.memory_pressure <= 1000.0);
            prop_assert!(complexity.hot_path_score >= 0.0);
            prop_assert!(complexity.estimated_gas >= 0.0);
            prop_assert!(complexity.indirect_call_overhead >= 0.0);
            prop_assert!(complexity.max_loop_depth <= 100);
        }

        fn special_characters_handled(
            prefix in "[a-zA-Z_][a-zA-Z0-9_]*",
            special_chars in prop::collection::vec(
                prop_oneof![
                    Just("'"),
                    Just("\""),
                    Just("\\"),
                    Just("\n"),
                    Just("\r"),
                    Just("\t"),
                    Just("/*"),
                    Just("*/"),
                    Just("//"),
                ],
                0..5
            ),
        ) {
            let mut code = format!("function {}(): void {{", prefix);
            for special in special_chars {
                code.push_str(&format!(" // Comment with {} \n", special));
            }
            code.push_str(" }");

            let parser = AssemblyScriptParser::new();
            prop_assert!(parser.is_ok());

            let mut parser = parser.unwrap();
            let runtime = tokio::runtime::Runtime::new().unwrap();
            let path = PathBuf::from("test.as");

            // Should handle special characters without panicking
            let _ = runtime.block_on(parser.parse_file(&path, &code));
        }

        fn file_operations_work(
            func_count in 0usize..10
        ) {
            let mut code = String::new();
            for i in 0..func_count {
                code.push_str(&format!("function test{}(): void {{}}\n", i));
            }
            let parser = AssemblyScriptParser::new();
            prop_assert!(parser.is_ok());

            let mut parser = parser.unwrap();
            let runtime = tokio::runtime::Runtime::new().unwrap();

            // Write to temp file
            let mut temp_file = NamedTempFile::new().unwrap();
            writeln!(temp_file, "{}", code).unwrap();
            temp_file.flush().unwrap();

            // Parse from file
            let content = std::fs::read_to_string(temp_file.path()).unwrap();
            let result = runtime.block_on(parser.parse_file(temp_file.path(), &content));

            // Should handle file operations
            prop_assert!(result.is_ok());
        }
    }

    #[test]
    fn empty_file_handling() {
        let _parser = AssemblyScriptParser::new().unwrap();
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let path = PathBuf::from("empty.as");

        let empty_contents = vec!["", " ", "\n", "\n\n", "\t", "  \n  "];

        for content in empty_contents {
            let mut p = AssemblyScriptParser::new().unwrap();
            let result = runtime.block_on(p.parse_file(&path, content));
            assert!(
                result.is_ok(),
                "Failed to parse empty content: {:?}",
                content
            );
        }
    }

    #[test]
    fn decorators_handled() {
        let _parser = AssemblyScriptParser::new().unwrap();
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let path = PathBuf::from("decorators.as");

        let code_with_decorators = r#"
            @inline
            function add(a: i32, b: i32): i32 {
                return a + b;
            }
            
            @external("env", "log")
            declare function log(s: string): void;
            
            @unmanaged
            class Vector3 {
                x: f32;
                y: f32;
                z: f32;
            }
        "#;

        let mut p = AssemblyScriptParser::new().unwrap();
        let result = runtime.block_on(p.parse_file(&path, code_with_decorators));
        assert!(result.is_ok(), "Failed to parse code with decorators");
    }
}
