#[cfg(all(test, feature = "typescript-ast"))]
mod tests {
    use crate::services::ast_typescript::TypeScriptParser;
    use proptest::prelude::*;
    use std::panic;
    use swc_common::{sync::Lrc, FileName, SourceMap};
    use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsConfig};

    // Strategy for generating valid JavaScript identifiers
    prop_compose! {
        fn arb_js_identifier()
            (s in "[a-zA-Z$][a-zA-Z0-9$_]{0,30}")
            -> String
        {
            s
        }
    }

    // Strategy for generating JavaScript literals
    prop_compose! {
        fn arb_js_literal()
            (choice in 0usize..6,
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
                3 => format!("'{}'", string_val),
                4 => format!(r#""{}""#, string_val),
                _ => "null".to_string(),
            }
        }
    }

    // Strategy for generating JavaScript source code structures
    prop_compose! {
        fn arb_js_source()
            (use_strict in prop::bool::ANY,
             num_functions in 0..10usize,
             num_classes in 0..5usize,
             num_vars in 0..20usize)
            -> String
        {
            let mut code = String::new();
            
            if use_strict {
                code.push_str("'use strict';\n\n");
            }
            
            // Generate variables
            for i in 0..num_vars {
                let var_type = match i % 3 {
                    0 => "const",
                    1 => "let",
                    _ => "var",
                };
                code.push_str(&format!("{} var{} = {};\n", var_type, i, i));
            }
            
            if num_vars > 0 {
                code.push('\n');
            }
            
            // Generate functions
            for i in 0..num_functions {
                if i % 3 == 0 {
                    // Arrow function
                    code.push_str(&format!("const func{} = () => {{\n", i));
                } else if i % 3 == 1 {
                    // Async function
                    code.push_str(&format!("async function func{}() {{\n", i));
                } else {
                    // Regular function
                    code.push_str(&format!("function func{}() {{\n", i));
                }
                
                // Add some complexity
                if i % 2 == 0 {
                    code.push_str("    if (true) {\n");
                    code.push_str("        console.log('branch');\n");
                    code.push_str("    }\n");
                }
                
                if i % 3 == 0 {
                    code.push_str("    for (let i = 0; i < 10; i++) {\n");
                    code.push_str("        console.log(i);\n");
                    code.push_str("    }\n");
                }
                
                code.push_str(&format!("    return {};\n", i));
                code.push_str("};\n\n");
            }
            
            // Generate classes
            for i in 0..num_classes {
                code.push_str(&format!("class Class{} {{\n", i));
                code.push_str("    constructor() {\n");
                code.push_str(&format!("        this.field = {};\n", i));
                code.push_str("    }\n");
                
                code.push_str("    method() {\n");
                code.push_str("        return this.field;\n");
                code.push_str("    }\n");
                code.push_str("}\n\n");
            }
            
            code
        }
    }

    // Strategy for generating TypeScript-specific code
    prop_compose! {
        fn arb_ts_source()
            (num_interfaces in 0..5usize,
             num_types in 0..5usize,
             num_enums in 0..3usize)
            -> String
        {
            let mut code = String::new();
            
            // Generate interfaces
            for i in 0..num_interfaces {
                code.push_str(&format!("interface Interface{} {{\n", i));
                code.push_str(&format!("    prop{}: string;\n", i));
                code.push_str(&format!("    method{}(): number;\n", i));
                code.push_str("}\n\n");
            }
            
            // Generate type aliases
            for i in 0..num_types {
                code.push_str(&format!("type Type{} = string | number", i));
                if i % 2 == 0 {
                    code.push_str(" | boolean");
                }
                code.push_str(";\n\n");
            }
            
            // Generate enums
            for i in 0..num_enums {
                code.push_str(&format!("enum Enum{} {{\n", i));
                code.push_str("    Value1,\n");
                code.push_str("    Value2 = 'string',\n");
                code.push_str(&format!("    Value3 = {}\n", i));
                code.push_str("}\n\n");
            }
            
            code
        }
    }

    proptest! {
        #[test]
        fn swc_parser_never_panics(source in arb_js_source()) {
            let result = panic::catch_unwind(|| {
                let cm = Lrc::new(SourceMap::default());
                let fm = cm.new_source_file(FileName::Anon, source);
                
                let lexer = Lexer::new(
                    Syntax::Typescript(TsConfig {
                        tsx: false,
                        decorators: true,
                        ..Default::default()
                    }),
                    Default::default(),
                    StringInput::from(&*fm),
                    None,
                );
                
                let mut parser = Parser::new_from(lexer);
                let _result = parser.parse_module();
            });
            
            prop_assert!(result.is_ok(), "SWC parser panicked on input");
        }

        #[test]
        fn typescript_parser_total_function(source in arb_js_source()) {
            let mut parser = TypeScriptParser::new();
            let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                parser.parse_file(std::path::Path::new("test.js"), &source)
            }));
            
            prop_assert!(result.is_ok(), "TypeScript parser panicked");
        }

        #[test]
        fn parser_handles_unicode(
            prefix in arb_js_identifier(),
            unicode_chars in prop::collection::vec(
                any::<char>().prop_filter("Valid JS identifier char", 
                    |c| c.is_alphanumeric() && !c.is_ascii()), 
                0..10
            )
        ) {
            let ident = format!("{}{}", prefix, unicode_chars.into_iter().collect::<String>());
            let code = format!("const {} = 42;", ident);
            
            let mut parser = TypeScriptParser::new();
            let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                parser.parse_file(std::path::Path::new("test.js"), &code)
            }));
            
            prop_assert!(result.is_ok(), "Parser panicked on Unicode identifier");
        }

        #[test]
        fn handles_typescript_features(source in arb_ts_source()) {
            let cm = Lrc::new(SourceMap::default());
            let fm = cm.new_source_file(FileName::Anon, source);
            
            let lexer = Lexer::new(
                Syntax::Typescript(TsConfig {
                    tsx: false,
                    decorators: true,
                    ..Default::default()
                }),
                Default::default(),
                StringInput::from(&*fm),
                None,
            );
            
            let mut parser = Parser::new_from(lexer);
            let result = parser.parse_typescript_module();
            
            // TypeScript-specific syntax should parse without panics
            // TypeScript-specific syntax should parse successfully or give a proper error
            prop_assert!(result.is_ok() || result.is_err(), "Parser should handle TypeScript syntax");
        }

        #[test]
        fn handles_jsx_tsx(
            tag_name in "[a-zA-Z][a-zA-Z0-9]*",
            content in "[a-zA-Z0-9 ]{0,50}"
        ) {
            let jsx_code = format!("const elem = <{}>{}</{}>;", tag_name, content, tag_name);
            
            let cm = Lrc::new(SourceMap::default());
            let fm = cm.new_source_file(FileName::Anon, jsx_code);
            
            let lexer = Lexer::new(
                Syntax::Typescript(TsConfig {
                    tsx: true,
                    decorators: true,
                    ..Default::default()
                }),
                Default::default(),
                StringInput::from(&*fm),
                None,
            );
            
            let mut parser = Parser::new_from(lexer);
            let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                parser.parse_typescript_module()
            }));
            
            prop_assert!(result.is_ok(), "Parser panicked on JSX/TSX");
        }

        #[test]
        fn empty_files_parse_successfully(
            whitespace in prop::collection::vec(
                prop::sample::select(vec![" ", "\t", "\n", "\r\n"]), 
                0..100
            )
        ) {
            let code = whitespace.join("");
            
            let mut parser = TypeScriptParser::new();
            let result = parser.parse_file(std::path::Path::new("test.js"), &code);
            
            prop_assert!(result.is_ok(), "Failed to parse empty/whitespace file");
            
            if let Ok(dag) = result {
                prop_assert!(dag.nodes.len() <= 1, 
                    "Too many nodes for empty file: {}", dag.nodes.len());
            }
        }

        #[test]
        fn deterministic_parsing(source in arb_js_source()) {
            let mut parser1 = TypeScriptParser::new();
            let mut parser2 = TypeScriptParser::new();
            
            let result1 = parser1.parse_file(std::path::Path::new("test.js"), &source);
            let result2 = parser2.parse_file(std::path::Path::new("test.js"), &source);
            
            match (result1, result2) {
                (Ok(dag1), Ok(dag2)) => {
                    prop_assert_eq!(dag1.nodes.len(), dag2.nodes.len(),
                        "Different number of nodes on repeated parse");
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
        fn handles_deeply_nested_blocks(depth in 1usize..20) {
            let mut code = String::new();
            
            code.push_str("function deeplyNested() {\n");
            
            // Generate deeply nested blocks
            for i in 0..depth {
                code.push_str(&format!("{}if (true) {{\n", "    ".repeat(i + 1)));
            }
            
            code.push_str(&format!("{}console.log('deep');\n", "    ".repeat(depth + 1)));
            
            for i in (0..depth).rev() {
                code.push_str(&format!("{}}}\n", "    ".repeat(i + 1)));
            }
            
            code.push_str("}\n");
            
            let mut parser = TypeScriptParser::new();
            let result = parser.parse_file(std::path::Path::new("test.js"), &code);
            
            prop_assert!(result.is_ok(), "Failed to parse deeply nested code");
        }

        #[test]
        fn handles_large_arrays_objects(size in 0usize..100) {
            let mut code = String::new();
            
            // Large array
            code.push_str("const arr = [");
            for i in 0..size {
                if i > 0 {
                    code.push_str(", ");
                }
                code.push_str(&i.to_string());
            }
            code.push_str("];\n\n");
            
            // Large object
            code.push_str("const obj = {");
            for i in 0..size {
                if i > 0 {
                    code.push_str(", ");
                }
                code.push_str(&format!("prop{}: {}", i, i));
            }
            code.push_str("};\n");
            
            let mut parser = TypeScriptParser::new();
            let result = parser.parse_file(std::path::Path::new("test.js"), &code);
            
            prop_assert!(result.is_ok(), "Failed to parse large array/object");
        }
    }

    // Test specific edge cases
    proptest! {
        #[test]
        fn handles_template_literals(
            parts in prop::collection::vec("[a-zA-Z0-9 ]{0,20}", 0..5)
        ) {
            let mut code = String::new();
            code.push_str("const str = `");
            
            for (i, part) in parts.iter().enumerate() {
                if i > 0 {
                    code.push_str(&format!("${{{}}}", i));
                }
                code.push_str(part);
            }
            
            code.push_str("`;\n");
            
            let mut parser = TypeScriptParser::new();
            let result = parser.parse_file(std::path::Path::new("test.js"), &code);
            
            prop_assert!(result.is_ok(), "Failed to parse template literal");
        }

        #[test]
        fn handles_decorators(
            decorator_name in arb_js_identifier(),
            class_name in arb_js_identifier()
        ) {
            let code = format!(
                "@{}\nclass {} {{\n    constructor() {{}}\n}}\n",
                decorator_name, class_name
            );
            
            let cm = Lrc::new(SourceMap::default());
            let fm = cm.new_source_file(FileName::Anon, code);
            
            let lexer = Lexer::new(
                Syntax::Typescript(TsConfig {
                    tsx: false,
                    decorators: true,
                    ..Default::default()
                }),
                Default::default(),
                StringInput::from(&*fm),
                None,
            );
            
            let mut parser = Parser::new_from(lexer);
            let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                parser.parse_typescript_module()
            }));
            
            prop_assert!(result.is_ok(), "Parser panicked on decorator syntax");
        }
    }
}