#[cfg(test)]
mod tests {
    use super::super::wat::WatParser;
    use proptest::prelude::*;

    proptest! {
        fn parser_never_panics_on_arbitrary_input(
            input in ".*"
        ) {
            let mut parser = WatParser::new();

            // Should not panic
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                parser.parse(&input)
            }));

            prop_assert!(result.is_ok());
        }

        fn valid_wat_modules_parse_successfully(
            sections in prop::collection::vec(
                prop_oneof![
                    Just("func"),
                    Just("memory"),
                    Just("table"),
                    Just("global"),
                    Just("export"),
                    Just("import"),
                    Just("type"),
                ],
                0..10,
            )
        ) {
            // Build module from sections
            let mut wat_content = "(module".to_string();
            for (i, section) in sections.into_iter().enumerate() {
                wat_content.push_str(&format!("\n  ({} $item{})", section, i));
            }
            wat_content.push_str("\n)");
            let mut parser = WatParser::new();
            let result = parser.parse(&wat_content);

            // Should successfully parse valid WAT modules
            prop_assert!(result.is_ok(), "Failed to parse valid WAT: {}", wat_content);
        }

        fn invalid_wat_is_rejected(
            invalid_type in 0usize..5
        ) {
            let invalid_content = match invalid_type {
                0 => "module without parens".to_string(),
                1 => "(module (func".to_string(), // Unclosed
                2 => "random text here".to_string(),
                3 => "".to_string(),
                _ => "   \n\t  ".to_string(), // Only whitespace
            };
            let mut parser = WatParser::new();
            let result = parser.parse(&invalid_content);

            // Should reject invalid WAT
            prop_assert!(result.is_err(), "Accepted invalid WAT: {}", invalid_content);
        }

        fn parser_respects_size_limits(
            repeat_count in 1usize..2000,
        ) {
            let base_content = "(module (func $test))";
            let mut parser = WatParser::new();

            // Create large content by repeating
            let large_content = base_content.repeat(repeat_count);
            let result = parser.parse(&large_content);

            if large_content.len() > 10 * 1024 * 1024 {
                // Should reject files larger than 10MB
                prop_assert!(result.is_err());
                if let Err(e) = result {
                    prop_assert!(e.to_string().contains("too large"));
                }
            } else {
                // Size is ok, check format
                if large_content.trim_start().starts_with('(') {
                    prop_assert!(result.is_ok());
                } else {
                    prop_assert!(result.is_err());
                }
            }
        }

        fn deeply_nested_structures_handled(
            depth in 0u32..10,
            count in 1usize..5,
        ) {
            let mut parser = WatParser::new();

            // Generate deeply nested module
            let mut content = "(module".to_string();
            for i in 0..count {
                // Create a simple nested structure based on depth
                let mut nested = String::new();
                for _ in 0..depth {
                    nested.push('(');
                }
                nested.push_str(&format!("item{}", i));
                for _ in 0..depth {
                    nested.push(')');
                }
                content.push_str(&format!("\n  {}", nested));
            }
            content.push(')');

            let result = parser.parse(&content);

            // Should handle any depth without panicking
            if content.len() <= 10 * 1024 * 1024 {
                prop_assert!(result.is_ok());
            }
        }

        fn whitespace_handling(
            prefix_ws in "[ \t\n\r]*",
            suffix_ws in "[ \t\n\r]*",
        ) {
            let wat_content = "(module)";
            let mut parser = WatParser::new();

            // Test with various whitespace
            let content_with_ws = format!("{}{}{}", prefix_ws, wat_content, suffix_ws);
            let result = parser.parse(&content_with_ws);

            // Should handle whitespace correctly
            prop_assert!(result.is_ok());
        }

        fn unicode_handling(
            unicode_chars in prop::collection::vec(any::<char>(), 0..20),
        ) {
            let wat_base = "(module)";
            let mut parser = WatParser::new();

            // Create unicode string
            let unicode_string: String = unicode_chars.into_iter().collect();

            // Insert unicode string as a comment
            let wat_with_unicode = format!("{} ;; {}", wat_base, unicode_string);

            let result = parser.parse(&wat_with_unicode);

            // Should handle unicode without panicking
            if wat_with_unicode.len() <= 10 * 1024 * 1024 {
                // Parser may accept or reject unicode, but shouldn't panic
                let _ = result;
            }
        }

    }

    #[test]
    fn empty_module_handling() {
        let mut parser = WatParser::new();

        let empty_modules = vec![
            "(module)",
            "(module )",
            "(module\n)",
            "(module\n\n)",
            "( module )",
            "(\nmodule\n)",
        ];

        for module in empty_modules {
            let result = parser.parse(module);
            assert!(result.is_ok(), "Failed to parse empty module: {}", module);
        }
    }

    // Property tests for additional trait implementations should be added
    // when the following methods are implemented:
    // - WasmAwareParser::parse_content (async)
    // - WasmAwareParser::extract_wasm_metrics
    // - WasmAwareParser::analyze_memory_patterns
    // - WasmAwareParser::calculate_wasm_complexity
}
