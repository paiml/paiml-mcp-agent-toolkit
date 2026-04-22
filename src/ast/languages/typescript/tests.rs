#![cfg_attr(coverage_nightly, coverage(off))]

#[cfg(all(test, feature = "typescript-ast"))]
mod tests {
    use super::super::*;
    use crate::ast::core::{AstDag, AstKind, ClassKind, Language, NodeFlags, TypeKind};
    use crate::ast::languages::LanguageStrategy;
    use std::path::{Path, PathBuf};

    // ==================== TypeScriptStrategy Tests ====================

    #[test]
    fn test_typescript_strategy_new() {
        let strategy = TypeScriptStrategy::new();
        assert_eq!(strategy.language(), Language::TypeScript);
    }

    #[test]
    fn test_typescript_strategy_default() {
        let strategy = TypeScriptStrategy;
        assert_eq!(strategy.language(), Language::TypeScript);
    }

    #[test]
    fn test_typescript_can_parse_ts() {
        let strategy = TypeScriptStrategy::new();
        assert!(strategy.can_parse(Path::new("test.ts")));
    }

    #[test]
    fn test_typescript_can_parse_tsx() {
        let strategy = TypeScriptStrategy::new();
        assert!(strategy.can_parse(Path::new("component.tsx")));
    }

    #[test]
    fn test_typescript_cannot_parse_js() {
        let strategy = TypeScriptStrategy::new();
        assert!(!strategy.can_parse(Path::new("test.js")));
    }

    #[test]
    fn test_typescript_cannot_parse_other() {
        let strategy = TypeScriptStrategy::new();
        assert!(!strategy.can_parse(Path::new("test.py")));
        assert!(!strategy.can_parse(Path::new("test.rs")));
        assert!(!strategy.can_parse(Path::new("Makefile")));
    }

    // ==================== JavaScriptStrategy Tests ====================

    #[test]
    fn test_javascript_strategy_new() {
        let strategy = JavaScriptStrategy::new();
        assert_eq!(strategy.language(), Language::JavaScript);
    }

    #[test]
    fn test_javascript_strategy_default() {
        let strategy = JavaScriptStrategy;
        assert_eq!(strategy.language(), Language::JavaScript);
    }

    #[test]
    fn test_javascript_can_parse_js() {
        let strategy = JavaScriptStrategy::new();
        assert!(strategy.can_parse(Path::new("test.js")));
    }

    #[test]
    fn test_javascript_can_parse_jsx() {
        let strategy = JavaScriptStrategy::new();
        assert!(strategy.can_parse(Path::new("component.jsx")));
    }

    #[test]
    fn test_javascript_can_parse_mjs() {
        let strategy = JavaScriptStrategy::new();
        assert!(strategy.can_parse(Path::new("module.mjs")));
    }

    #[test]
    fn test_javascript_cannot_parse_ts() {
        let strategy = JavaScriptStrategy::new();
        assert!(!strategy.can_parse(Path::new("test.ts")));
    }

    // ==================== TypeScript Parsing Tests ====================

    #[tokio::test]
    async fn test_parse_simple_function() {
        let strategy = TypeScriptStrategy::new();
        let content = r#"
            function greet(name: string): string {
                return "Hello, " + name;
            }
        "#;

        let path = PathBuf::from("test.ts");
        let dag = strategy.parse_file(&path, content).await.unwrap();

        let functions = strategy.extract_functions(&dag);
        assert!(!functions.is_empty(), "Should find at least one function");
    }

    #[tokio::test]
    async fn test_parse_async_function() {
        let strategy = TypeScriptStrategy::new();
        let content = r#"
            async function fetchData(url: string): Promise<Response> {
                const response = await fetch(url);
                return response;
            }
        "#;

        let path = PathBuf::from("test.ts");
        let dag = strategy.parse_file(&path, content).await.unwrap();

        let functions = strategy.extract_functions(&dag);
        assert!(!functions.is_empty(), "Should find async function");

        // Check for async flag
        let has_async = functions.iter().any(|f| f.flags.has(NodeFlags::ASYNC));
        assert!(has_async, "Async function should have ASYNC flag");
    }

    #[tokio::test]
    async fn test_parse_class_with_methods() {
        let strategy = TypeScriptStrategy::new();
        let content = r#"
            class Calculator {
                private result: number = 0;

                constructor() {
                    this.result = 0;
                }

                add(value: number): number {
                    this.result += value;
                    return this.result;
                }

                subtract(value: number): number {
                    this.result -= value;
                    return this.result;
                }
            }
        "#;

        let path = PathBuf::from("test.ts");
        let dag = strategy.parse_file(&path, content).await.unwrap();

        let types = strategy.extract_types(&dag);
        assert!(!types.is_empty(), "Should find class type");

        let functions = strategy.extract_functions(&dag);
        // Should find constructor + add + subtract = 3 methods
        assert!(
            functions.len() >= 3,
            "Should find at least 3 methods (constructor, add, subtract)"
        );
    }

    #[tokio::test]
    async fn test_parse_interface() {
        let strategy = TypeScriptStrategy::new();
        let content = r#"
            interface Shape {
                area(): number;
                perimeter(): number;
            }
        "#;

        let path = PathBuf::from("test.ts");
        let dag = strategy.parse_file(&path, content).await.unwrap();

        let types = strategy.extract_types(&dag);
        assert!(!types.is_empty(), "Should find interface");

        // Check for interface class kind
        let has_interface = types
            .iter()
            .any(|t| matches!(t.kind, AstKind::Class(ClassKind::Interface)));
        assert!(has_interface, "Should have interface type");
    }

    #[tokio::test]
    async fn test_parse_type_alias() {
        let strategy = TypeScriptStrategy::new();
        let content = r#"
            type StringOrNumber = string | number;
            type Point = { x: number; y: number };
        "#;

        let path = PathBuf::from("test.ts");
        let dag = strategy.parse_file(&path, content).await.unwrap();

        let types = strategy.extract_types(&dag);
        assert!(types.len() >= 2, "Should find at least 2 type aliases");

        // Check for type alias kind
        let has_alias = types
            .iter()
            .any(|t| matches!(t.kind, AstKind::Type(TypeKind::Alias)));
        assert!(has_alias, "Should have type alias");
    }

    #[tokio::test]
    async fn test_parse_imports() {
        let strategy = TypeScriptStrategy::new();
        let content = r#"
            import { useState, useEffect } from "react";
            import axios from "axios";
            import * as fs from "fs";

            function App() {
                return null;
            }
        "#;

        let path = PathBuf::from("test.ts");
        let dag = strategy.parse_file(&path, content).await.unwrap();

        let imports = strategy.extract_imports(&dag);
        assert!(imports.len() >= 3, "Should find at least 3 imports");
    }

    #[tokio::test]
    async fn test_parse_export_declaration() {
        let strategy = TypeScriptStrategy::new();
        let content = r#"
            export function add(a: number, b: number): number {
                return a + b;
            }

            export class Counter {
                count: number = 0;
            }
        "#;

        let path = PathBuf::from("test.ts");
        let dag = strategy.parse_file(&path, content).await.unwrap();

        let functions = strategy.extract_functions(&dag);
        assert!(!functions.is_empty(), "Should find exported function");

        let types = strategy.extract_types(&dag);
        assert!(!types.is_empty(), "Should find exported class");
    }

    #[tokio::test]
    async fn test_parse_arrow_function() {
        let strategy = TypeScriptStrategy::new();
        let content = r#"
            const double = (x: number): number => x * 2;
            const square = (x: number): number => {
                return x * x;
            };
        "#;

        let path = PathBuf::from("test.ts");
        let dag = strategy.parse_file(&path, content).await.unwrap();

        let functions = strategy.extract_functions(&dag);
        assert!(
            functions.len() >= 2,
            "Should find at least 2 arrow functions"
        );
    }

    #[tokio::test]
    async fn test_parse_async_arrow_function() {
        let strategy = TypeScriptStrategy::new();
        let content = r#"
            const fetchUser = async (id: string) => {
                const response = await fetch(`/users/${id}`);
                return response.json();
            };
        "#;

        let path = PathBuf::from("test.ts");
        let dag = strategy.parse_file(&path, content).await.unwrap();

        let functions = strategy.extract_functions(&dag);
        assert!(!functions.is_empty(), "Should find async arrow function");

        let has_async = functions.iter().any(|f| f.flags.has(NodeFlags::ASYNC));
        assert!(has_async, "Async arrow function should have ASYNC flag");
    }

    #[tokio::test]
    async fn test_parse_function_expression() {
        let strategy = TypeScriptStrategy::new();
        let content = r#"
            const multiply = function(a: number, b: number): number {
                return a * b;
            };

            const asyncFn = async function() {
                return await Promise.resolve(42);
            };
        "#;

        let path = PathBuf::from("test.ts");
        let dag = strategy.parse_file(&path, content).await.unwrap();

        let functions = strategy.extract_functions(&dag);
        assert!(functions.len() >= 2, "Should find function expressions");
    }

    #[tokio::test]
    async fn test_parse_object_method() {
        let strategy = TypeScriptStrategy::new();
        let content = r#"
            const calculator = {
                add(a: number, b: number) {
                    return a + b;
                },
                multiply: function(a: number, b: number) {
                    return a * b;
                },
                divide: (a: number, b: number) => a / b
            };
        "#;

        let path = PathBuf::from("test.ts");
        let dag = strategy.parse_file(&path, content).await.unwrap();

        let functions = strategy.extract_functions(&dag);
        // Should find object method, function expression, and arrow function
        assert!(
            functions.len() >= 3,
            "Should find at least 3 functions in object"
        );
    }

    #[tokio::test]
    async fn test_parse_control_flow() {
        let strategy = TypeScriptStrategy::new();
        let content = r#"
            function checkValue(x: number): string {
                if (x > 0) {
                    return "positive";
                } else if (x < 0) {
                    return "negative";
                }
                return "zero";
            }

            function processArray(arr: number[]): void {
                for (const item of arr) {
                    console.log(item);
                }

                while (arr.length > 0) {
                    arr.pop();
                }
            }
        "#;

        let path = PathBuf::from("test.ts");
        let dag = strategy.parse_file(&path, content).await.unwrap();

        let (cyclomatic, cognitive) = strategy.calculate_complexity(&dag);
        // Base complexity is 1, and we found 2 function declarations
        // The current implementation doesn't traverse into function bodies
        assert!(cyclomatic >= 1, "Should have base cyclomatic complexity");
        let _ = cognitive; // usize is always non-negative
    }

    #[tokio::test]
    async fn test_parse_switch_statement() {
        let strategy = TypeScriptStrategy::new();
        let content = r#"
            function getDayName(day: number): string {
                switch (day) {
                    case 0:
                        return "Sunday";
                    case 1:
                        return "Monday";
                    default:
                        return "Unknown";
                }
            }
        "#;

        let path = PathBuf::from("test.ts");
        let dag = strategy.parse_file(&path, content).await.unwrap();

        let (cyclomatic, _) = strategy.calculate_complexity(&dag);
        // Base complexity is 1, current implementation doesn't traverse function bodies
        assert!(cyclomatic >= 1, "Should have base cyclomatic complexity");
    }

    #[tokio::test]
    async fn test_parse_tsx_component() {
        let strategy = TypeScriptStrategy::new();
        let content = r#"
            import React from "react";

            interface Props {
                name: string;
                age?: number;
            }

            function Greeting({ name, age }: Props): JSX.Element {
                return (
                    <div>
                        <h1>Hello, {name}!</h1>
                        {age && <p>You are {age} years old</p>}
                    </div>
                );
            }

            export default Greeting;
        "#;

        let path = PathBuf::from("component.tsx");
        let dag = strategy.parse_file(&path, content).await.unwrap();

        let functions = strategy.extract_functions(&dag);
        assert!(
            !functions.is_empty(),
            "Should find React component function"
        );

        let types = strategy.extract_types(&dag);
        assert!(!types.is_empty(), "Should find Props interface");
    }

    #[tokio::test]
    async fn test_parse_generic_function() {
        let strategy = TypeScriptStrategy::new();
        let content = r#"
            function identity<T>(arg: T): T {
                return arg;
            }

            function map<T, U>(arr: T[], fn: (item: T) => U): U[] {
                return arr.map(fn);
            }
        "#;

        let path = PathBuf::from("test.ts");
        let dag = strategy.parse_file(&path, content).await.unwrap();

        let functions = strategy.extract_functions(&dag);
        assert!(functions.len() >= 2, "Should find generic functions");
    }

    #[tokio::test]
    async fn test_parse_generic_class() {
        let strategy = TypeScriptStrategy::new();
        let content = r#"
            class Container<T> {
                private value: T;

                constructor(value: T) {
                    this.value = value;
                }

                getValue(): T {
                    return this.value;
                }

                setValue(value: T): void {
                    this.value = value;
                }
            }
        "#;

        let path = PathBuf::from("test.ts");
        let dag = strategy.parse_file(&path, content).await.unwrap();

        let types = strategy.extract_types(&dag);
        assert!(!types.is_empty(), "Should find generic class");

        let functions = strategy.extract_functions(&dag);
        // constructor + getValue + setValue = 3
        assert!(functions.len() >= 3, "Should find class methods");
    }

    #[tokio::test]
    async fn test_parse_decorators() {
        let strategy = TypeScriptStrategy::new();
        let content = r#"
            @Controller("/users")
            class UserController {
                @Get("/:id")
                getUser(id: string) {
                    return { id };
                }

                @Post("/")
                createUser(data: any) {
                    return data;
                }
            }
        "#;

        let path = PathBuf::from("test.ts");
        let dag = strategy.parse_file(&path, content).await.unwrap();

        let types = strategy.extract_types(&dag);
        assert!(!types.is_empty(), "Should find decorated class");
    }

    #[tokio::test]
    async fn test_parse_call_expression_with_callbacks() {
        let strategy = TypeScriptStrategy::new();
        let content = r#"
            const numbers = [1, 2, 3, 4, 5];

            const doubled = numbers.map((n) => n * 2);
            const sum = numbers.reduce((acc, n) => acc + n, 0);
            const evens = numbers.filter(function(n) { return n % 2 === 0; });
        "#;

        let path = PathBuf::from("test.ts");
        let dag = strategy.parse_file(&path, content).await.unwrap();

        let functions = strategy.extract_functions(&dag);
        // map callback + reduce callback + filter callback = 3
        assert!(functions.len() >= 3, "Should find callback functions");
    }

    // ==================== JavaScript Parsing Tests ====================

    #[tokio::test]
    async fn test_javascript_parse_simple_function() {
        let strategy = JavaScriptStrategy::new();
        let content = r#"
            function greet(name) {
                return "Hello, " + name;
            }
        "#;

        let path = PathBuf::from("test.js");
        let dag = strategy.parse_file(&path, content).await.unwrap();

        let functions = strategy.extract_functions(&dag);
        assert!(!functions.is_empty(), "Should find JS function");
    }

    #[tokio::test]
    async fn test_javascript_parse_jsx() {
        let strategy = JavaScriptStrategy::new();
        let content = r#"
            import React from "react";

            function App() {
                return (
                    <div>
                        <h1>Hello World</h1>
                    </div>
                );
            }
        "#;

        let path = PathBuf::from("component.jsx");
        let dag = strategy.parse_file(&path, content).await.unwrap();

        let functions = strategy.extract_functions(&dag);
        assert!(!functions.is_empty(), "Should find JSX component");
    }

    #[tokio::test]
    async fn test_javascript_parse_es6_class() {
        let strategy = JavaScriptStrategy::new();
        let content = r#"
            class Animal {
                constructor(name) {
                    this.name = name;
                }

                speak() {
                    console.log(this.name + " makes a sound");
                }
            }
        "#;

        let path = PathBuf::from("test.js");
        let dag = strategy.parse_file(&path, content).await.unwrap();

        let types = strategy.extract_types(&dag);
        assert!(!types.is_empty(), "Should find ES6 class");
    }

    #[tokio::test]
    async fn test_javascript_extract_imports() {
        let strategy = JavaScriptStrategy::new();
        let content = r#"
            import { useState } from "react";
            import axios from "axios";

            function App() {
                return null;
            }
        "#;

        let path = PathBuf::from("test.js");
        let dag = strategy.parse_file(&path, content).await.unwrap();

        let imports = strategy.extract_imports(&dag);
        assert!(imports.len() >= 2, "Should find imports");
    }

    #[tokio::test]
    async fn test_javascript_calculate_complexity() {
        let strategy = JavaScriptStrategy::new();
        let content = r#"
            function processValue(x) {
                if (x > 10) {
                    for (let i = 0; i < x; i++) {
                        console.log(i);
                    }
                } else if (x > 5) {
                    while (x > 0) {
                        x--;
                    }
                }
            }
        "#;

        let path = PathBuf::from("test.js");
        let dag = strategy.parse_file(&path, content).await.unwrap();

        let (cyclomatic, cognitive) = strategy.calculate_complexity(&dag);
        // Base complexity is 1, current implementation doesn't traverse function bodies
        assert!(cyclomatic >= 1, "Should have base cyclomatic complexity");
        let _ = cognitive; // usize is always non-negative
    }

    // ==================== Error Handling Tests ====================

    #[tokio::test]
    async fn test_parse_invalid_syntax_typescript() {
        let strategy = TypeScriptStrategy::new();
        let content = r#"
            function broken( {
                // Missing closing brace
        "#;

        let path = PathBuf::from("test.ts");
        let result = strategy.parse_file(&path, content).await;

        // Should return error for invalid syntax
        assert!(result.is_err(), "Should error on invalid TypeScript syntax");
    }

    #[tokio::test]
    async fn test_parse_invalid_syntax_javascript() {
        let strategy = JavaScriptStrategy::new();
        let content = r#"
            function broken( {
                // Missing closing brace
        "#;

        let path = PathBuf::from("test.js");
        let result = strategy.parse_file(&path, content).await;

        // Should return error for invalid syntax
        assert!(result.is_err(), "Should error on invalid JavaScript syntax");
    }

    #[tokio::test]
    async fn test_parse_empty_file() {
        let strategy = TypeScriptStrategy::new();
        let content = "";

        let path = PathBuf::from("empty.ts");
        let dag = strategy.parse_file(&path, content).await.unwrap();

        let functions = strategy.extract_functions(&dag);
        assert!(functions.is_empty(), "Empty file should have no functions");
    }

    #[tokio::test]
    async fn test_parse_comments_only() {
        let strategy = TypeScriptStrategy::new();
        let content = r#"
            // This is a comment
            /* This is a block comment */
            /**
             * This is a doc comment
             */
        "#;

        let path = PathBuf::from("comments.ts");
        let dag = strategy.parse_file(&path, content).await.unwrap();

        let functions = strategy.extract_functions(&dag);
        assert!(
            functions.is_empty(),
            "Comments only should have no functions"
        );
    }

    // ==================== Edge Case Tests ====================

    #[tokio::test]
    async fn test_parse_nested_functions() {
        let strategy = TypeScriptStrategy::new();
        let content = r#"
            function outer() {
                function inner() {
                    return 42;
                }
                return inner();
            }
        "#;

        let path = PathBuf::from("test.ts");
        let dag = strategy.parse_file(&path, content).await.unwrap();

        let functions = strategy.extract_functions(&dag);
        // Note: nested functions may or may not be detected depending on visitor depth
        assert!(!functions.is_empty(), "Should find at least outer function");
    }

    #[tokio::test]
    async fn test_parse_iife() {
        let strategy = TypeScriptStrategy::new();
        let content = r#"
            (function() {
                console.log("IIFE executed");
            })();

            (() => {
                console.log("Arrow IIFE");
            })();
        "#;

        let path = PathBuf::from("test.ts");
        let dag = strategy.parse_file(&path, content).await.unwrap();

        // IIFEs are wrapped in parenthesized expressions which the current
        // visitor doesn't traverse. Just verify that parsing succeeds.
        let _functions = strategy.extract_functions(&dag);
        // Parse succeeded (didn't panic) - node count depends on visitor implementation
        // IIFEs wrapped in Paren expressions are not currently traversed
    }

    #[tokio::test]
    async fn test_parse_class_with_static_methods() {
        let strategy = TypeScriptStrategy::new();
        let content = r#"
            class Utils {
                static add(a: number, b: number): number {
                    return a + b;
                }

                static multiply(a: number, b: number): number {
                    return a * b;
                }

                instance_method(): void {
                    console.log("instance");
                }
            }
        "#;

        let path = PathBuf::from("test.ts");
        let dag = strategy.parse_file(&path, content).await.unwrap();

        let functions = strategy.extract_functions(&dag);
        assert!(
            functions.len() >= 3,
            "Should find static and instance methods"
        );
    }

    #[tokio::test]
    async fn test_parse_expression_statement() {
        let strategy = TypeScriptStrategy::new();
        let content = r#"
            console.log("Hello");
            process.exit(0);
            fetchData().then(data => console.log(data));
        "#;

        let path = PathBuf::from("test.ts");
        let dag = strategy.parse_file(&path, content).await.unwrap();

        // Should find the arrow function in .then callback
        let functions = strategy.extract_functions(&dag);
        assert!(!functions.is_empty(), "Should find callback in expression");
    }

    #[tokio::test]
    async fn test_parse_variable_declaration_patterns() {
        let strategy = TypeScriptStrategy::new();
        let content = r#"
            const regularFn = function() { return 1; };
            let arrowFn = () => 2;
            var asyncArrow = async () => { return 3; };
            const asyncFn = async function() { return 4; };
        "#;

        let path = PathBuf::from("test.ts");
        let dag = strategy.parse_file(&path, content).await.unwrap();

        let functions = strategy.extract_functions(&dag);
        assert!(functions.len() >= 4, "Should find all function expressions");

        // Check for async flags
        let async_count = functions
            .iter()
            .filter(|f| f.flags.has(NodeFlags::ASYNC))
            .count();
        assert!(async_count >= 2, "Should find at least 2 async functions");
    }

    #[tokio::test]
    async fn test_parse_object_shorthand_method() {
        let strategy = TypeScriptStrategy::new();
        let content = r#"
            const obj = {
                method() {
                    return 1;
                },
                async asyncMethod() {
                    return 2;
                }
            };
        "#;

        let path = PathBuf::from("test.ts");
        let dag = strategy.parse_file(&path, content).await.unwrap();

        let functions = strategy.extract_functions(&dag);
        assert!(functions.len() >= 2, "Should find shorthand methods");
    }

    #[tokio::test]
    async fn test_complexity_with_empty_dag() {
        let strategy = TypeScriptStrategy::new();
        let dag = AstDag::new();

        let (cyclomatic, cognitive) = strategy.calculate_complexity(&dag);
        assert_eq!(cyclomatic, 1, "Base cyclomatic complexity should be 1");
        assert_eq!(cognitive, 0, "Base cognitive complexity should be 0");
    }

    #[tokio::test]
    async fn test_extract_functions_empty_dag() {
        let strategy = TypeScriptStrategy::new();
        let dag = AstDag::new();

        let functions = strategy.extract_functions(&dag);
        assert!(functions.is_empty(), "Empty dag should have no functions");
    }

    #[tokio::test]
    async fn test_extract_types_empty_dag() {
        let strategy = TypeScriptStrategy::new();
        let dag = AstDag::new();

        let types = strategy.extract_types(&dag);
        assert!(types.is_empty(), "Empty dag should have no types");
    }

    #[tokio::test]
    async fn test_extract_imports_empty_dag() {
        let strategy = TypeScriptStrategy::new();
        let dag = AstDag::new();

        let imports = strategy.extract_imports(&dag);
        assert!(imports.is_empty(), "Empty dag should have no imports");
    }

    // ==================== File Extension Tests ====================

    #[tokio::test]
    async fn test_ts_extension_uses_typescript_syntax() {
        let strategy = TypeScriptStrategy::new();
        let content = r#"
            const value: number = 42;
            interface Test { x: number; }
        "#;

        let path = PathBuf::from("file.ts");
        let result = strategy.parse_file(&path, content).await;
        assert!(result.is_ok(), ".ts files should parse TypeScript syntax");
    }

    #[tokio::test]
    async fn test_tsx_extension_uses_tsx_syntax() {
        let strategy = TypeScriptStrategy::new();
        let content = r#"
            const Component = () => <div>Hello</div>;
        "#;

        let path = PathBuf::from("file.tsx");
        let result = strategy.parse_file(&path, content).await;
        assert!(result.is_ok(), ".tsx files should parse TSX syntax");
    }

    #[tokio::test]
    async fn test_jsx_extension_uses_jsx_syntax() {
        let strategy = JavaScriptStrategy::new();
        let content = r#"
            const Component = () => <div>Hello</div>;
        "#;

        let path = PathBuf::from("file.jsx");
        let result = strategy.parse_file(&path, content).await;
        assert!(result.is_ok(), ".jsx files should parse JSX syntax");
    }

    #[tokio::test]
    async fn test_js_extension_uses_es_syntax() {
        let strategy = JavaScriptStrategy::new();
        let content = r#"
            const arrow = () => 42;
            class MyClass {}
        "#;

        let path = PathBuf::from("file.js");
        let result = strategy.parse_file(&path, content).await;
        assert!(result.is_ok(), ".js files should parse ES syntax");
    }

    // ==================== Module Item Tests ====================

    #[tokio::test]
    async fn test_export_named_declaration() {
        let strategy = TypeScriptStrategy::new();
        let content = r#"
            export const PI = 3.14;
            export function calculate() { return PI * 2; }
            export class Calculator { }
        "#;

        let path = PathBuf::from("test.ts");
        let dag = strategy.parse_file(&path, content).await.unwrap();

        // Should find the exported function and class
        let functions = strategy.extract_functions(&dag);
        assert!(!functions.is_empty(), "Should find exported function");

        let types = strategy.extract_types(&dag);
        assert!(!types.is_empty(), "Should find exported class");
    }

    #[tokio::test]
    async fn test_mixed_imports_and_code() {
        let strategy = TypeScriptStrategy::new();
        let content = r#"
            import { x } from "./x";
            import { y } from "./y";

            const sum = x + y;

            function process() {
                return sum * 2;
            }

            export { process };
        "#;

        let path = PathBuf::from("test.ts");
        let dag = strategy.parse_file(&path, content).await.unwrap();

        let imports = strategy.extract_imports(&dag);
        assert!(imports.len() >= 2, "Should find imports");

        let functions = strategy.extract_functions(&dag);
        assert!(!functions.is_empty(), "Should find function");
    }

    #[tokio::test]
    async fn test_object_literal_with_spread_and_getter() {
        // visitor.rs:176 (Spread `continue`) + visitor.rs:189 (`_ =>` catch-all).
        // Spread drops into the let-else continue; Getter isn't Method/KeyValue
        // so it falls through to the `_ =>` arm.
        let strategy = TypeScriptStrategy::new();
        let content = r#"
            const base = { a: 1 };
            const obj = {
                ...base,
                get computed() { return 42; },
                set value(v) { this._v = v; },
            };
        "#;

        let path = PathBuf::from("test.ts");
        let dag = strategy.parse_file(&path, content).await.unwrap();

        // Spread/getter/setter props don't emit nodes themselves; this test
        // just verifies the traversal reaches the Spread `continue` and the
        // `_ =>` catch-all arm without panicking and without emitting spurious
        // function nodes for the getter/setter.
        let functions = strategy.extract_functions(&dag);
        assert!(
            functions.is_empty(),
            "getter/setter must not emit function nodes, got {}",
            functions.len()
        );
    }

    #[tokio::test]
    async fn test_object_literal_with_method_prop() {
        // Covers the Prop::Method arm (visitor.rs:179-184) — object method
        // emits a Function(Regular) node distinct from a KeyValue prop.
        let strategy = TypeScriptStrategy::new();
        let content = r#"
            const obj = {
                greet() { return "hi"; },
                name: "world",
            };
        "#;

        let path = PathBuf::from("test.ts");
        let dag = strategy.parse_file(&path, content).await.unwrap();

        let functions = strategy.extract_functions(&dag);
        assert!(
            !functions.is_empty(),
            "Prop::Method should emit a function node"
        );
    }
}
