#[cfg(test)]
mod tests {
    use crate::services::ast_rust::analyze_rust_file_with_complexity;
    use crate::services::ast_rust_unified::RustAstParser;
    use crate::services::unified_ast_parser::{ParserConfig, UnifiedAstParser};
    use proptest::prelude::*;
    use std::panic;
    use std::path::Path;
    use syn::{parse_file, visit::Visit};
    use tempfile::NamedTempFile;

    // Strategy for generating valid Rust identifiers
    prop_compose! {
        fn arb_identifier()
            (s in "[a-zA-Z][a-zA-Z0-9_]{0,30}")
            -> String
        {
            s
        }
    }

    // Strategy for generating valid Rust literals
    prop_compose! {
        fn arb_literal()
            (choice in 0usize..5,
             int_val in any::<i64>(),
             float_val in any::<f64>(),
             bool_val in any::<bool>(),
             string_val in "[a-zA-Z0-9 ]{0,50}")
            -> String
        {
            match choice {
                0 => int_val.to_string(),
                1 => format!("{:.2}", float_val),
                2 => bool_val.to_string(),
                3 => format!(r#""{string_val}""#),
                _ => "()".to_string(),
            }
        }
    }

    // Test-specific complexity visitor
    struct TestComplexityVisitor {
        cyclomatic: u16,
        cognitive: u16,
        nesting_max: u8,
        nesting_level: u8,
    }

    impl TestComplexityVisitor {
        fn new() -> Self {
            Self {
                cyclomatic: 1, // Base complexity
                cognitive: 0,
                nesting_max: 0,
                nesting_level: 0,
            }
        }

        fn enter_nesting(&mut self) {
            self.nesting_level = self.nesting_level.saturating_add(1);
            if self.nesting_level > self.nesting_max {
                self.nesting_max = self.nesting_level;
            }
        }

        fn exit_nesting(&mut self) {
            self.nesting_level = self.nesting_level.saturating_sub(1);
        }

        fn add_complexity(&mut self, cyclomatic: u16, cognitive_base: u16) {
            self.cyclomatic = self.cyclomatic.saturating_add(cyclomatic);
            let cognitive = if self.nesting_level > 0 {
                cognitive_base + self.nesting_level.saturating_sub(1) as u16
            } else {
                cognitive_base
            };
            self.cognitive = self.cognitive.saturating_add(cognitive);
        }
    }

    impl<'ast> Visit<'ast> for TestComplexityVisitor {
        fn visit_expr(&mut self, node: &'ast syn::Expr) {
            match node {
                syn::Expr::If(_) | syn::Expr::Match(_) => {
                    self.add_complexity(1, 1);
                    self.enter_nesting();
                    syn::visit::visit_expr(self, node);
                    self.exit_nesting();
                }
                syn::Expr::While(_) | syn::Expr::Loop(_) | syn::Expr::ForLoop(_) => {
                    self.add_complexity(1, 1);
                    self.enter_nesting();
                    syn::visit::visit_expr(self, node);
                    self.exit_nesting();
                }
                _ => syn::visit::visit_expr(self, node),
            }
        }
    }

    // Strategy for generating Rust source code structures
    prop_compose! {
        fn arb_rust_source()
            (num_functions in 0..10usize,
             num_structs in 0..5usize,
             num_enums in 0..5usize)
            -> String
        {
            let mut code = String::new();

            // Generate structs
            for i in 0..num_structs {
                code.push_str(&format!("struct Struct{} {{\n", i));
                code.push_str("    field1: i32,\n");
                code.push_str("    field2: String,\n");
                code.push_str("}\n\n");
            }

            // Generate enums
            for i in 0..num_enums {
                code.push_str(&format!("enum Enum{} {{\n", i));
                code.push_str("    Variant1,\n");
                code.push_str("    Variant2(i32),\n");
                code.push_str("}\n\n");
            }

            // Generate functions with varying complexity
            for i in 0..num_functions {
                code.push_str(&format!("fn func{}() {{\n", i));

                // Add some complexity based on index
                if i % 2 == 0 {
                    code.push_str("    if true {\n");
                    code.push_str("        println!(\"branch\");\n");
                    code.push_str("    }\n");
                }

                if i % 3 == 0 {
                    code.push_str("    for i in 0..10 {\n");
                    code.push_str("        println!(\"{}\", i);\n");
                    code.push_str("    }\n");
                }

                code.push_str(&format!("    let _x = {};\n", i));

                code.push_str("}\n\n");
            }

            code
        }
    }

    // Strategy for generating a single Rust statement
    prop_compose! {
        fn arb_statement()
            (var_name in arb_identifier(),
             literal in arb_literal())
            -> String
        {
            format!("let {} = {};", var_name, literal)
        }
    }

    proptest! {
        #[test]
        fn ast_parser_total_function(source in arb_rust_source()) {
            // Property 1: Parser is total (never panics)
            let result = panic::catch_unwind(|| {
                parse_file(&source)
            });
            prop_assert!(result.is_ok(), "Parser panicked on input");

            // Property 2: Valid parse ⇒ AST traversal terminates
            if let Ok(Ok(ast)) = result {
                let visitor_result = panic::catch_unwind(|| {
                    let mut visitor = TestComplexityVisitor::new();
                    visitor.visit_file(&ast);
                    visitor
                });
                prop_assert!(visitor_result.is_ok(), "Visitor panicked during traversal");
            }
        }

        #[test]
        fn ast_complexity_monotonic(base in arb_rust_source(), insertion in arb_statement()) {
            let base_ast = parse_file(&base);
            let extended = format!("{}\nfn extra_function() {{ {} }}", base, insertion);
            let extended_ast = parse_file(&extended);

            if let (Ok(ast1), Ok(ast2)) = (base_ast, extended_ast) {
                let mut visitor1 = TestComplexityVisitor::new();
                visitor1.visit_file(&ast1);
                let c1 = visitor1.cyclomatic;

                let mut visitor2 = TestComplexityVisitor::new();
                visitor2.visit_file(&ast2);
                let c2 = visitor2.cyclomatic;

                // Complexity is monotonic: adding code never decreases complexity
                prop_assert!(c2 >= c1, "Complexity decreased after adding code: {} -> {}", c1, c2);
            }
        }

        #[test]
        fn rust_parser_never_panics_on_arbitrary_input(code in any::<String>()) {
            // The parser should be robust enough to handle any string input
            // without panicking. It is expected to return an error for invalid
            // code, but it must not crash the process.
            let parser = RustAstParser::new();
            let config = ParserConfig {
                extract_complexity: false, // Don't try to read from disk
                ..Default::default()
            };
            let result = panic::catch_unwind(|| {
                // We need to use async context for the parser
                let runtime = tokio::runtime::Runtime::new().unwrap();
                runtime.block_on(async {
                    parser.parse_content(&code, Path::new("test.rs"), &config).await
                })
            });
            prop_assert!(result.is_ok(), "Parser panicked on arbitrary input");
        }

        #[test]
        fn rust_parser_handles_valid_but_unconventional_formatting(
            fn_name in arb_identifier(),
            var_name in arb_identifier(),
            num_spaces in 0usize..20,
            num_newlines in 0usize..20
        ) {
            // This test generates syntactically valid but unconventionally
            // formatted Rust code to ensure the parser is resilient to
            // whitespace variations.
            let spaces = " ".repeat(num_spaces);
            let newlines = "\n".repeat(num_newlines);

            let code = format!(
                "fn {fn_name}{spaces}({newlines}) {{{newlines}let {var_name}{spaces}={newlines}42;{spaces}}}",
                fn_name = fn_name,
                spaces = spaces,
                newlines = newlines,
                var_name = var_name
            );

            let parser = RustAstParser::new();
            let config = ParserConfig {
                extract_complexity: false, // Don't try to read from disk
                ..Default::default()
            };
            let runtime = tokio::runtime::Runtime::new().unwrap();
            let result = runtime.block_on(async {
                parser.parse_content(&code, Path::new("test.rs"), &config).await
            });

            prop_assert!(
                result.is_ok(),
                "Parser failed on valid but unconventionally formatted code. Error: {:?}",
                result.err()
            );
        }

        #[test]
        fn complexity_metrics_bounded(source in arb_rust_source()) {
            if let Ok(ast) = parse_file(&source) {
                let mut visitor = TestComplexityVisitor::new();
                visitor.visit_file(&ast);

                // Property: All metrics should be reasonable for generated code
                prop_assert!(visitor.cyclomatic <= 1000, "Cyclomatic complexity too high: {}", visitor.cyclomatic);
                prop_assert!(visitor.cognitive <= 1000, "Cognitive complexity too high: {}", visitor.cognitive);
                prop_assert!(visitor.nesting_max <= 10, "Nesting level too high: {}", visitor.nesting_max);

                // Property: Cyclomatic >= 1 for any non-empty file
                if !source.trim().is_empty() {
                    prop_assert!(visitor.cyclomatic >= 1, "Cyclomatic complexity should be at least 1");
                }
            }
        }

        #[test]
        fn parser_deterministic(source in arb_rust_source()) {
            let parser = RustAstParser::new();
            let config = ParserConfig {
                extract_complexity: false, // Don't try to read from disk
                ..Default::default()
            };
            let runtime = tokio::runtime::Runtime::new().unwrap();

            // Parse the same source twice
            let result1 = runtime.block_on(async {
                parser.parse_content(&source, Path::new("test.rs"), &config).await
            });

            let result2 = runtime.block_on(async {
                parser.parse_content(&source, Path::new("test.rs"), &config).await
            });

            // Property: Parsing is deterministic
            match (result1, result2) {
                (Ok(r1), Ok(r2)) => {
                    prop_assert_eq!(r1.unified_nodes.len(), r2.unified_nodes.len(),
                        "Different number of AST nodes on repeated parse");
                    prop_assert_eq!(r1.warnings.len(), r2.warnings.len(),
                        "Different number of warnings on repeated parse");
                },
                (Err(_), Err(_)) => {
                    // Both failed, which is consistent
                },
                _ => {
                    prop_assert!(false, "Parser gave different results for same input");
                }
            }
        }

        #[test]
        fn empty_input_handled_gracefully(whitespace in prop::collection::vec(
            prop::sample::select(vec![" ", "\t", "\n", "\r\n"]), 0..100
        )) {
            let code = whitespace.join("");

            let parser = RustAstParser::new();
            let config = ParserConfig {
                extract_complexity: false, // Don't try to read from disk
                ..Default::default()
            };
            let runtime = tokio::runtime::Runtime::new().unwrap();

            let result = runtime.block_on(async {
                parser.parse_content(&code, Path::new("test.rs"), &config).await
            });

            // Empty or whitespace-only files should parse successfully
            prop_assert!(result.is_ok(), "Failed to parse empty/whitespace file");

            if let Ok(parse_result) = result {
                // Should produce minimal AST
                prop_assert!(parse_result.unified_nodes.len() <= 1,
                    "Too many nodes for empty file: {}", parse_result.unified_nodes.len());
            }
        }

        #[test]
        fn file_parser_integration(content in arb_rust_source()) {
            // Create a temporary file
            let temp_file = NamedTempFile::new().unwrap();
            let path = temp_file.path();

            // Write content to file
            std::fs::write(path, &content).unwrap();

            // Test the file-based parser
            let result = panic::catch_unwind(|| {
                let runtime = tokio::runtime::Runtime::new().unwrap();
                runtime.block_on(async {
                    analyze_rust_file_with_complexity(path).await
                })
            });

            prop_assert!(result.is_ok(), "File parser panicked");

            if let Ok(Ok(metrics)) = result {
                // Verify we got valid metrics
                // Cyclomatic complexity is always non-negative by definition
                prop_assert_eq!(metrics.path, path.display().to_string());
            }
        }
    }

    // Additional targeted property tests for edge cases
    proptest! {
        #[test]
        fn handles_deeply_nested_structures(depth in 1usize..20) {
            let mut code = String::new();

            // Generate deeply nested if statements
            for i in 0..depth {
                code.push_str(&format!("{}if true {{\n", "    ".repeat(i)));
            }

            code.push_str(&format!("{}println!(\"deep\");\n", "    ".repeat(depth)));

            for i in (0..depth).rev() {
                code.push_str(&format!("{}}}\n", "    ".repeat(i)));
            }

            let code = format!("fn deeply_nested() {{\n{}}}", code);

            let result = parse_file(&code);
            prop_assert!(result.is_ok(), "Failed to parse deeply nested code");

            if let Ok(ast) = result {
                let mut visitor = TestComplexityVisitor::new();
                let visitor_result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                    visitor.visit_file(&ast);
                    visitor
                }));

                prop_assert!(visitor_result.is_ok(), "Visitor panicked on deeply nested code");

                if let Ok(visitor) = visitor_result {
                    prop_assert_eq!(visitor.nesting_max as usize, depth,
                        "Incorrect nesting level calculation");
                }
            }
        }

        #[test]
        fn handles_unicode_identifiers(
            prefix in "[a-zA-Z_]",
            unicode_chars in prop::collection::vec(any::<char>().prop_filter("Valid identifier char",
                |c| c.is_alphanumeric() && !c.is_ascii()), 0..10)
        ) {
            let ident = format!("{}{}", prefix, unicode_chars.into_iter().collect::<String>());
            let code = format!("fn {}() {{ }}", ident);

            // Note: Rust doesn't actually support arbitrary Unicode in identifiers,
            // but the parser should handle this gracefully
            let _result = parse_file(&code);

            // Either parses successfully or gives a proper error (not panic)
            let did_not_panic = panic::catch_unwind(|| parse_file(&code)).is_ok();
            prop_assert!(did_not_panic, "Parser panicked on Unicode identifier");
        }
    }
}
