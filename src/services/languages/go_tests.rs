// Go tests — included from go.rs
// Contains: unit tests and property tests for Go language support

#[cfg(all(test, feature = "go-ast"))]
mod tests {
    use super::*;
    use std::path::Path;

    const SIMPLE_GO_FUNCTION: &str = r#"
package main

import "fmt"

func helloWorld() {
    fmt.Println("Hello, World!")
}
"#;

    const GO_STRUCT_WITH_METHODS: &str = r#"
package calculator

type Calculator struct {
    result float64
}

func (c *Calculator) Add(x, y float64) float64 {
    c.result = x + y
    return c.result
}

func (c *Calculator) Multiply(x, y float64) float64 {
    c.result = x * y
    return c.result
}
"#;

    const GO_INTERFACE_DEFINITION: &str = r#"
package shapes

type Shape interface {
    Area() float64
    Perimeter() float64
}

type Circle struct {
    Radius float64
}

func (c Circle) Area() float64 {
    return 3.14159 * c.Radius * c.Radius
}

func (c Circle) Perimeter() float64 {
    return 2 * 3.14159 * c.Radius
}
"#;

    #[test]
    fn test_simple_go_function_analysis() {
        let visitor = GoAstVisitor::new(Path::new("test.go"));
        let items = visitor
            .analyze_go_source(SIMPLE_GO_FUNCTION)
            .expect("Should parse Go function");

        assert!(!items.is_empty(), "Should extract at least one AST item");

        let function_items: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Function { .. }))
            .collect();

        assert_eq!(
            function_items.len(),
            1,
            "Should extract exactly one function"
        );

        if let AstItem::Function {
            name,
            visibility,
            is_async,
            ..
        } = &items[0]
        {
            assert_eq!(
                name, "main::helloWorld",
                "Should have qualified function name"
            );
            assert_eq!(visibility, "public", "Go functions are public by default");
            assert!(!is_async, "Regular Go functions are not async");
        } else {
            panic!("Expected function item");
        }
    }

    #[test]
    fn test_go_struct_with_methods_analysis() {
        let visitor = GoAstVisitor::new(Path::new("calculator.go"));
        let items = visitor
            .analyze_go_source(GO_STRUCT_WITH_METHODS)
            .expect("Should parse Go struct");

        assert!(items.len() >= 3, "Should extract struct and methods");

        let struct_items: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Struct { .. }))
            .collect();

        assert_eq!(struct_items.len(), 1, "Should extract exactly one struct");

        if let AstItem::Struct {
            name, fields_count, ..
        } = &struct_items[0]
        {
            assert_eq!(
                name, "calculator::Calculator",
                "Should have qualified struct name"
            );
            assert_eq!(
                *fields_count, 2,
                "Should count methods as fields for Go structs"
            );
        }

        let method_items: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Function { .. }))
            .collect();

        assert_eq!(method_items.len(), 2, "Should extract both methods");
    }

    #[test]
    fn test_go_interface_analysis() {
        let visitor = GoAstVisitor::new(Path::new("shapes.go"));
        let items = visitor
            .analyze_go_source(GO_INTERFACE_DEFINITION)
            .expect("Should parse Go interface");

        let interface_items: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Trait { .. }))
            .collect();

        assert_eq!(
            interface_items.len(),
            1,
            "Should extract exactly one interface"
        );

        if let AstItem::Trait { name, .. } = &interface_items[0] {
            assert_eq!(
                name, "shapes::Shape",
                "Should have qualified interface name"
            );
        }
    }

    #[test]
    fn test_go_complexity_analysis() {
        let mut analyzer = GoComplexityAnalyzer::new();
        let (cyclomatic, cognitive) = analyzer
            .analyze_complexity(SIMPLE_GO_FUNCTION)
            .expect("Should analyze Go complexity");

        assert!(
            cyclomatic >= 1,
            "Should have at least cyclomatic complexity of 1"
        );
        assert!(
            cognitive >= 1,
            "Should have at least cognitive complexity of 1"
        );
        assert!(
            cyclomatic <= 10,
            "Should maintain complexity ≤10 for simple function"
        );
        assert!(cognitive <= 10, "Should maintain cognitive complexity ≤10");
    }

    #[test]
    fn test_go_package_name_extraction() {
        let visitor = GoAstVisitor::new(Path::new("test.go"));
        let items = visitor
            .analyze_go_source(SIMPLE_GO_FUNCTION)
            .expect("Should parse Go source");

        // Check that package name is included in qualified names
        let has_main_package = items.iter().any(|item| match item {
            AstItem::Function { name, .. } => name.starts_with("main::"),
            _ => false,
        });

        assert!(
            has_main_package,
            "Should include package name in qualified names"
        );
    }

    #[test]
    fn test_empty_go_source() {
        let visitor = GoAstVisitor::new(Path::new("empty.go"));
        let items = visitor
            .analyze_go_source("")
            .expect("Should handle empty source");

        assert!(items.is_empty(), "Empty source should produce no AST items");
    }

    #[test]
    fn test_invalid_go_syntax() {
        let visitor = GoAstVisitor::new(Path::new("invalid.go"));
        let result = visitor.analyze_go_source("invalid go syntax {{{ !!!");

        // The Go visitor uses pattern matching, not full parsing,
        // so invalid syntax returns Ok with empty results rather than an error
        assert!(result.is_ok(), "Should handle invalid syntax gracefully");
        assert!(
            result.unwrap().is_empty(),
            "Invalid syntax should produce no AST items"
        );
    }
}

#[cfg(all(test, feature = "go-ast"))]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    use std::path::Path;

    proptest! {
        #[test]
        fn test_go_visitor_handles_any_valid_package_name(
            package_name in "[a-zA-Z_][a-zA-Z0-9_]*"
        ) {
            let source = format!("package {}\n\nfunc main() {{}}", package_name);
            let visitor = GoAstVisitor::new(Path::new("test.go"));

            if let Ok(items) = visitor.analyze_go_source(&source) {
                // Should extract package and function
                prop_assert!(!items.is_empty());

                // Check that package name is included in qualified names
                let has_package_prefix = items.iter().any(|item| match item {
                    AstItem::Function { name, .. } => name.starts_with(&format!("{}::", package_name)),
                    _ => false,
                });
                prop_assert!(has_package_prefix);
            }
        }

        #[test]
        fn test_go_complexity_analyzer_bounds(
            function_count in 1usize..10
        ) {
            let mut source = String::from("package test\n\n");
            for i in 0..function_count {
                source.push_str(&format!("func function{}() {{}}\n", i));
            }

            let visitor = GoAstVisitor::new(Path::new("test.go"));
            if let Ok(items) = visitor.analyze_go_source(&source) {
                let function_items: Vec<_> = items.iter()
                    .filter(|item| matches!(item, AstItem::Function { .. }))
                    .collect();

                // Should extract all functions
                prop_assert_eq!(function_items.len(), function_count);

                // All should be functions with real names
                for (i, item) in function_items.iter().enumerate() {
                    if let AstItem::Function { name, .. } = item {
                        let expected_name = format!("function{}", i);
                        prop_assert!(name.contains(&expected_name));
                    }
                }
            }
        }

        #[test]
        fn test_go_complexity_stays_bounded(
            depth in 1u32..5
        ) {
            let mut source = String::from("package test\n\nfunc complexFunction() {\n");
            for _ in 0..depth {
                source.push_str("if true {\n");
            }
            source.push_str("return\n");
            for _ in 0..depth {
                source.push_str("}\n");
            }
            source.push_str("}\n");

            let mut analyzer = GoComplexityAnalyzer::new();
            if let Ok((cyclomatic, cognitive)) = analyzer.analyze_complexity(&source) {
                // Complexity should grow but stay reasonable
                prop_assert!(cyclomatic >= depth);
                prop_assert!(cognitive >= depth);
                prop_assert!(cyclomatic <= depth * 2 + 5); // Reasonable upper bound
                prop_assert!(cognitive <= depth * 3 + 5); // Reasonable upper bound
            }
        }
    }
}
