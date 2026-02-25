// Tests for EnhancedTypeScriptVisitor: unit tests and property-based tests

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "typescript-ast")]
    mod typescript_tests {
        use super::*;
        use std::sync::Arc;
        use swc_common::{FileName, SourceMap};
        use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsSyntax};

        fn parse_typescript(code: &str) -> Module {
            let source_map = Arc::new(SourceMap::default());
            let source_file = source_map
                .new_source_file(FileName::Custom("test.ts".into()).into(), code.to_string());

            let lexer = Lexer::new(
                Syntax::Typescript(TsSyntax {
                    tsx: false,
                    decorators: true,
                    dts: false,
                    no_early_errors: true,
                    disallow_ambiguous_jsx_like: true,
                }),
                Default::default(),
                StringInput::from(&*source_file),
                None,
            );

            let mut parser = Parser::new_from(lexer);
            parser.parse_module().expect("Failed to parse TypeScript")
        }

        /// Test enhanced visitor extracts real function names
        #[test]
        fn test_extract_real_function_names() {
            let code = r#"
                function calculateComplexity(): number { return 42; }
                async function processData(input: string): Promise<string> {
                    return input.toUpperCase();
                }
                const helperFunction = () => {};
                const asyncHelper = async (x: number) => x * 2;
            "#;

            let module = parse_typescript(code);
            let visitor = EnhancedTypeScriptVisitor::new(Path::new("test.ts"));
            let items = visitor.extract_items(&module);

            // Should extract all real function names
            let function_names: Vec<String> = items
                .iter()
                .filter_map(|item| match item {
                    AstItem::Function { name, .. } => Some(name.clone()),
                    _ => None,
                })
                .collect();

            assert!(function_names.contains(&"calculateComplexity".to_string()));
            assert!(function_names.contains(&"processData".to_string()));
            assert!(function_names.contains(&"helperFunction".to_string()));
            assert!(function_names.contains(&"asyncHelper".to_string()));
        }

        /// Test enhanced visitor handles classes correctly
        #[test]
        fn test_extract_class_details() {
            let code = r#"
                class DataProcessor {
                    constructor(private threshold: number) {}

                    async process(data: string[]): Promise<string[]> {
                        return data.filter(item => item.length > this.threshold);
                    }

                    private validateInput(input: string): boolean {
                        return input.length > 0;
                    }
                }
            "#;

            let module = parse_typescript(code);
            let visitor = EnhancedTypeScriptVisitor::new(Path::new("test.ts"));
            let items = visitor.extract_items(&module);

            // Find the class
            let class_item = items.iter().find(
                |item| matches!(item, AstItem::Struct { name, .. } if name == "DataProcessor"),
            );
            assert!(class_item.is_some());

            // Find class methods
            let method_names: Vec<String> = items
                .iter()
                .filter_map(|item| match item {
                    AstItem::Function { name, .. } if name.contains("DataProcessor::") => {
                        Some(name.clone())
                    }
                    _ => None,
                })
                .collect();

            // Enhanced TypeScript visitor should extract actual method names
            assert!(
                method_names.contains(&"DataProcessor::constructor".to_string()),
                "Should extract constructor"
            );
            assert!(
                method_names.contains(&"DataProcessor::process".to_string()),
                "Should extract async process method name"
            );
            assert!(
                method_names.contains(&"DataProcessor::validateInput".to_string()),
                "Should extract private method name"
            );

            // Filter out duplicates caused by visiting both class methods and functions
            let unique_methods: std::collections::HashSet<String> =
                method_names.into_iter().collect();
            assert_eq!(
                unique_methods.len(),
                3,
                "Should have exactly 3 unique methods, got: {:?}",
                unique_methods
            );
        }

        /// Test enhanced visitor handles interfaces
        #[test]
        fn test_extract_interface_details() {
            let code = r#"
                interface Processor<T> {
                    process(data: T): T;
                    validate(input: T): boolean;
                }
            "#;

            let module = parse_typescript(code);
            let visitor = EnhancedTypeScriptVisitor::new(Path::new("test.ts"));
            let items = visitor.extract_items(&module);

            let interface_item = items
                .iter()
                .find(|item| matches!(item, AstItem::Trait { name, .. } if name == "Processor"));
            assert!(interface_item.is_some());
        }

        /// Test enhanced visitor handles imports/exports
        #[test]
        fn test_extract_imports() {
            let code = r#"
                import { Component } from 'react';
                import * as utils from './utils';
                export { DataProcessor } from './processor';
            "#;

            let module = parse_typescript(code);
            let visitor = EnhancedTypeScriptVisitor::new(Path::new("test.ts"));
            let items = visitor.extract_items(&module);

            let import_items: Vec<&AstItem> = items
                .iter()
                .filter(|item| matches!(item, AstItem::Use { .. }))
                .collect();

            assert!(import_items.len() >= 2);
        }
    }

    /// Property tests for enhanced TypeScript visitor
    #[cfg_attr(coverage_nightly, coverage(off))]
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            /// Property: visitor always produces valid AST items for valid TypeScript
            #[test]
            fn visitor_produces_valid_items(seed in 0u64..100) {
                let code = generate_typescript_code(seed);

                // Only test valid TypeScript code
                if is_valid_typescript(&code) {
                    #[cfg(feature = "typescript-ast")]
                    {
                        if let Ok(module) = try_parse_typescript(&code) {
                            let visitor = EnhancedTypeScriptVisitor::new(Path::new("test.ts"));
                            let items = visitor.extract_items(&module);

                            // All function items should have non-empty names
                            for item in &items {
                                match item {
                                    AstItem::Function { name, .. } |
                                    AstItem::Struct { name, .. } |
                                    AstItem::Enum { name, .. } |
                                    AstItem::Trait { name, .. } => {
                                        prop_assert!(!name.is_empty());
                                        prop_assert!(!name.starts_with("function_"));
                                        prop_assert!(!name.starts_with("class_"));
                                    }
                                    AstItem::Use { path, .. } => {
                                        prop_assert!(!path.is_empty());
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }

            /// Property: async functions are correctly detected
            #[test]
            fn async_detection_property(is_async in any::<bool>()) {
                let async_keyword = if is_async { "async " } else { "" };
                let code = format!("{}function testFunc() {{ return 42; }}", async_keyword);

                #[cfg(feature = "typescript-ast")]
                {
                    if let Ok(module) = try_parse_typescript(&code) {
                        let visitor = EnhancedTypeScriptVisitor::new(Path::new("test.ts"));
                        let items = visitor.extract_items(&module);

                        if let Some(AstItem::Function { is_async: detected, .. }) =
                            items.iter().find(|item| matches!(item, AstItem::Function { .. })) {
                            prop_assert_eq!(*detected, is_async);
                        }
                    }
                }
            }
        }

        fn generate_typescript_code(seed: u64) -> String {
            let templates = [
                "function func{}() { return {}; }",
                "const func{} = () => {};",
                "class Class{} { method() {} }",
                "interface Interface{} { prop: number; }",
            ];

            let template = &templates[seed as usize % templates.len()];
            template.replace("{}", &seed.to_string())
        }

        fn is_valid_typescript(code: &str) -> bool {
            // Basic validation - not empty, has valid characters
            !code.is_empty() && code.is_ascii()
        }

        #[cfg(feature = "typescript-ast")]
        fn try_parse_typescript(code: &str) -> Result<Module, ()> {
            use std::sync::Arc;
            use swc_common::{FileName, SourceMap};
            use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsSyntax};

            let source_map = Arc::new(SourceMap::default());
            let source_file = source_map
                .new_source_file(FileName::Custom("test.ts".into()).into(), code.to_string());

            let lexer = Lexer::new(
                Syntax::Typescript(TsSyntax {
                    tsx: false,
                    decorators: true,
                    dts: false,
                    no_early_errors: true,
                    disallow_ambiguous_jsx_like: true,
                }),
                Default::default(),
                StringInput::from(&*source_file),
                None,
            );

            let mut parser = Parser::new_from(lexer);
            parser.parse_module().map_err(|_| ())
        }
    }
}
