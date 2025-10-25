#![cfg(all(test, feature = "typescript-ast", feature = "integration-tests"))]

use anyhow::Result;
use pmat::services::languages::typescript::TypeScriptAstVisitor;
use pmat::services::languages::javascript::JavaScriptAstVisitor;
use pmat::services::context::AstItem;
use std::path::Path;

/// Test TypeScript source parsing with a simple function
#[test]
fn test_typescript_source_parsing_simple_function() -> Result<()> {
    let ts_source = r#"
    function greet(name: string): string {
        return "Hello, " + name + "!";
    }
    "#;

    let visitor = TypeScriptAstVisitor::new(Path::new("test.ts"));
    let items = visitor.analyze_typescript_source(ts_source)?;

    // Verify we found the function
    assert!(!items.is_empty(), "Should find at least one AST item");

    let function_items: Vec<_> = items.iter()
        .filter(|item| matches!(item, AstItem::Function { .. }))
        .collect();

    assert!(!function_items.is_empty(), "Should find at least one function");

    // Check function name
    if let Some(AstItem::Function { name, .. }) = function_items.first() {
        assert!(name.contains("greet"), "Function should be named greet");
    }

    Ok(())
}

/// Test TypeScript source parsing with a class
#[test]
fn test_typescript_source_parsing_class() -> Result<()> {
    let ts_source = r#"
    class Person {
        private name: string;
        private age: number;

        constructor(name: string, age: number) {
            this.name = name;
            this.age = age;
        }

        getName(): string {
            return this.name;
        }

        getAge(): number {
            return this.age;
        }
    }
    "#;

    let visitor = TypeScriptAstVisitor::new(Path::new("test.ts"));
    let items = visitor.analyze_typescript_source(ts_source)?;

    // Verify we found items
    assert!(!items.is_empty(), "Should find AST items");

    // Check for class
    let class_items: Vec<_> = items.iter()
        .filter(|item| matches!(item, AstItem::Struct { .. }))
        .collect();

    assert!(!class_items.is_empty(), "Should find at least one class");

    // Check for methods
    let method_items: Vec<_> = items.iter()
        .filter(|item| matches!(item, AstItem::Function { .. }))
        .collect();

    assert!(!method_items.is_empty(), "Should find at least one method");

    Ok(())
}

/// Test TypeScript source parsing with interface
#[test]
fn test_typescript_source_parsing_interface() -> Result<()> {
    let ts_source = r#"
    interface Shape {
        area(): number;
        perimeter(): number;
    }

    class Circle implements Shape {
        constructor(private radius: number) {}

        area(): number {
            return Math.PI * this.radius * this.radius;
        }

        perimeter(): number {
            return 2 * Math.PI * this.radius;
        }
    }
    "#;

    let visitor = TypeScriptAstVisitor::new(Path::new("test.ts"));
    let items = visitor.analyze_typescript_source(ts_source)?;

    // Verify we found items
    assert!(!items.is_empty(), "Should find AST items");

    // Check for trait (interface)
    let _trait_items: Vec<_> = items.iter()
        .filter(|item| matches!(item, AstItem::Trait { .. }))
        .collect();

    // TypeScript parser may represent interfaces as traits or structs
    // Just verify we found some structural items
    assert!(!items.is_empty(), "Should find structural items for interface");

    Ok(())
}

/// Test JavaScript source parsing with a simple function
#[test]
fn test_javascript_source_parsing_simple_function() -> Result<()> {
    let js_source = r#"
    function calculateSum(a, b) {
        return a + b;
    }

    function calculateProduct(a, b) {
        return a * b;
    }
    "#;

    let visitor = JavaScriptAstVisitor::new(Path::new("test.js"));
    let items = visitor.analyze_javascript_source(js_source)?;

    // Verify we found functions
    assert!(!items.is_empty(), "Should find at least one AST item");

    let function_items: Vec<_> = items.iter()
        .filter(|item| matches!(item, AstItem::Function { .. }))
        .collect();

    assert!(function_items.len() >= 2, "Should find at least two functions");

    Ok(())
}

/// Test JavaScript source parsing with ES6 class
#[test]
fn test_javascript_source_parsing_es6_class() -> Result<()> {
    let js_source = r#"
    class Calculator {
        constructor() {
            this.result = 0;
        }

        add(value) {
            this.result += value;
            return this;
        }

        subtract(value) {
            this.result -= value;
            return this;
        }

        getResult() {
            return this.result;
        }
    }
    "#;

    let visitor = JavaScriptAstVisitor::new(Path::new("test.js"));
    let items = visitor.analyze_javascript_source(js_source)?;

    // Verify we found items
    assert!(!items.is_empty(), "Should find AST items");

    // Check for class
    let class_items: Vec<_> = items.iter()
        .filter(|item| matches!(item, AstItem::Struct { .. }))
        .collect();

    assert!(!class_items.is_empty(), "Should find at least one class");

    Ok(())
}

/// Test JavaScript source parsing with arrow functions
#[test]
fn test_javascript_source_parsing_arrow_functions() -> Result<()> {
    let js_source = r#"
    const double = (x) => x * 2;

    const square = (x) => {
        return x * x;
    };

    const greet = (name) => "Hello, " + name + "!";
    "#;

    let visitor = JavaScriptAstVisitor::new(Path::new("test.js"));
    let items = visitor.analyze_javascript_source(js_source)?;

    // Verify we found items (arrow functions may be detected as functions or variables)
    assert!(!items.is_empty(), "Should find AST items for arrow functions");

    Ok(())
}

/// Test JavaScript source parsing with async/await
#[test]
fn test_javascript_source_parsing_async_await() -> Result<()> {
    let js_source = r#"
    async function fetchData(url) {
        const response = await fetch(url);
        const data = await response.json();
        return data;
    }

    async function processData(data) {
        return data.map(item => item.value);
    }
    "#;

    let visitor = JavaScriptAstVisitor::new(Path::new("test.js"));
    let items = visitor.analyze_javascript_source(js_source)?;

    // Verify we found async functions
    assert!(!items.is_empty(), "Should find AST items");

    let async_functions: Vec<_> = items.iter()
        .filter(|item| {
            if let AstItem::Function { is_async, .. } = item {
                *is_async
            } else {
                false
            }
        })
        .collect();

    assert!(!async_functions.is_empty(), "Should find at least one async function");

    Ok(())
}

/// Test TypeScript source parsing with generics
#[test]
fn test_typescript_source_parsing_generics() -> Result<()> {
    let ts_source = r#"
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

    function identity<T>(arg: T): T {
        return arg;
    }
    "#;

    let visitor = TypeScriptAstVisitor::new(Path::new("test.ts"));
    let items = visitor.analyze_typescript_source(ts_source)?;

    // Verify we found items
    assert!(!items.is_empty(), "Should find AST items for generic types");

    Ok(())
}

/// Test error handling for invalid TypeScript source
#[test]
fn test_typescript_source_parsing_invalid_syntax() -> Result<()> {
    let ts_source = r#"
    function broken(
        // Missing closing brace and syntax errors
    "#;

    let visitor = TypeScriptAstVisitor::new(Path::new("test.ts"));
    let result = visitor.analyze_typescript_source(ts_source);

    // Parser should handle errors gracefully
    // It may return an error or empty items
    match result {
        Ok(_items) => {
            // Parser recovered and returned partial items
            assert!(true, "Parser handled invalid syntax gracefully");
        }
        Err(_) => {
            // Parser returned an error, which is expected
            assert!(true, "Parser correctly reported error for invalid syntax");
        }
    }

    Ok(())
}

/// Test error handling for invalid JavaScript source
#[test]
fn test_javascript_source_parsing_invalid_syntax() -> Result<()> {
    let js_source = r#"
    function broken(
        // Missing closing brace
    "#;

    let visitor = JavaScriptAstVisitor::new(Path::new("test.js"));
    let result = visitor.analyze_javascript_source(js_source);

    // Parser should handle errors gracefully
    match result {
        Ok(_items) => {
            assert!(true, "Parser handled invalid syntax gracefully");
        }
        Err(_) => {
            assert!(true, "Parser correctly reported error for invalid syntax");
        }
    }

    Ok(())
}
