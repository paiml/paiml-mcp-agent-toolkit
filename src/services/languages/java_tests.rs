// Java unit tests and property tests
// Included from java.rs - NO use imports or #! inner attributes

#[cfg(all(test, feature = "java-ast"))]
mod tests {
    use super::*;
    use std::path::Path;

    const SIMPLE_JAVA_CLASS: &str = r#"
package com.example;

public class HelloWorld {
    public static void main(String[] args) {
        System.out.println("Hello, World!");
    }
}
"#;

    const JAVA_CLASS_WITH_METHODS: &str = r#"
package com.example.calculator;

public class Calculator {
    private double result;

    public double add(double x, double y) {
        this.result = x + y;
        return this.result;
    }

    public double multiply(double x, double y) {
        this.result = x * y;
        return this.result;
    }

    public double getResult() {
        return this.result;
    }
}
"#;

    const JAVA_INTERFACE_DEFINITION: &str = r#"
package com.example.shapes;

public interface Shape {
    double area();
    double perimeter();
}

public class Circle implements Shape {
    private double radius;

    public Circle(double radius) {
        this.radius = radius;
    }

    @Override
    public double area() {
        return Math.PI * radius * radius;
    }

    @Override
    public double perimeter() {
        return 2 * Math.PI * radius;
    }
}
"#;

    #[test]
    fn test_simple_java_class_analysis() {
        let visitor = JavaAstVisitor::new(Path::new("HelloWorld.java"));
        let items = visitor
            .analyze_java_source(SIMPLE_JAVA_CLASS)
            .expect("Should parse Java class");

        assert!(!items.is_empty(), "Should extract at least one AST item");

        let class_items: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Struct { .. }))
            .collect();

        assert_eq!(class_items.len(), 1, "Should extract exactly one class");

        if let AstItem::Struct {
            name, visibility, ..
        } = &class_items[0]
        {
            assert_eq!(
                name, "com.example::HelloWorld",
                "Should have qualified class name"
            );
            assert_eq!(visibility, "public", "Java classes have public visibility");
        } else {
            panic!("Expected class item");
        }
    }

    #[test]
    fn test_java_class_with_methods_analysis() {
        let visitor = JavaAstVisitor::new(Path::new("Calculator.java"));
        let items = visitor
            .analyze_java_source(JAVA_CLASS_WITH_METHODS)
            .expect("Should parse Java class");

        assert!(items.len() >= 4, "Should extract class and methods");

        let class_items: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Struct { .. }))
            .collect();

        assert_eq!(class_items.len(), 1, "Should extract exactly one class");

        if let AstItem::Struct {
            name, fields_count, ..
        } = &class_items[0]
        {
            assert_eq!(
                name, "com.example.calculator::Calculator",
                "Should have qualified class name"
            );
            assert_eq!(
                *fields_count, 3,
                "Should count methods as fields for Java classes"
            );
        }

        let method_items: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Function { .. }))
            .collect();

        assert_eq!(method_items.len(), 3, "Should extract all three methods");
    }

    #[test]
    fn test_java_interface_analysis() {
        let visitor = JavaAstVisitor::new(Path::new("Shape.java"));
        let items = visitor
            .analyze_java_source(JAVA_INTERFACE_DEFINITION)
            .expect("Should parse Java interface");

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
                name, "com.example.shapes::Shape",
                "Should have qualified interface name"
            );
        }
    }

    #[test]
    fn test_java_complexity_analysis() {
        let mut analyzer = JavaComplexityAnalyzer::new();
        let (cyclomatic, cognitive) = analyzer
            .analyze_complexity(SIMPLE_JAVA_CLASS)
            .expect("Should analyze Java complexity");

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
            "Should maintain complexity ≤10 for simple class"
        );
        assert!(cognitive <= 10, "Should maintain cognitive complexity ≤10");
    }

    #[test]
    fn test_java_package_name_extraction() {
        let visitor = JavaAstVisitor::new(Path::new("test.java"));
        let items = visitor
            .analyze_java_source(SIMPLE_JAVA_CLASS)
            .expect("Should parse Java source");

        // Check that package name is included in qualified names
        let has_example_package = items.iter().any(|item| match item {
            AstItem::Struct { name, .. } => name.starts_with("com.example::"),
            _ => false,
        });

        assert!(
            has_example_package,
            "Should include package name in qualified names"
        );
    }

    #[test]
    fn test_empty_java_source() {
        let visitor = JavaAstVisitor::new(Path::new("empty.java"));
        let items = visitor
            .analyze_java_source("")
            .expect("Should handle empty source");

        assert!(items.is_empty(), "Empty source should produce no AST items");
    }

    #[test]
    fn test_invalid_java_syntax() {
        let visitor = JavaAstVisitor::new(Path::new("invalid.java"));
        let result = visitor.analyze_java_source("invalid java syntax {{{ !!!");

        assert!(
            result.is_err(),
            "Should return error for invalid Java syntax"
        );
    }
}

#[cfg(all(test, feature = "java-ast"))]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    use std::path::Path;

    proptest! {
        #[test]
        fn test_java_visitor_handles_any_valid_package_name(
            package_name in "[a-zA-Z_][a-zA-Z0-9_]*\\.[a-zA-Z_][a-zA-Z0-9_]*"
        ) {
            let source = format!("package {};\n\npublic class TestClass {{}}", package_name);
            let visitor = JavaAstVisitor::new(Path::new("test.java"));

            if let Ok(items) = visitor.analyze_java_source(&source) {
                // Should extract package and class
                prop_assert!(!items.is_empty());

                // Check that package name is included in qualified names
                let has_package_prefix = items.iter().any(|item| match item {
                    AstItem::Struct { name, .. } => name.starts_with(&format!("{}::", package_name)),
                    _ => false,
                });
                prop_assert!(has_package_prefix);
            }
        }

        #[test]
        fn test_java_complexity_analyzer_bounds(
            method_count in 1usize..10
        ) {
            let mut source = String::from("package test;\n\npublic class Test {\n");
            for i in 0..method_count {
                source.push_str(&format!("public void method{}() {{}}\n", i));
            }
            source.push_str("}\n");

            let visitor = JavaAstVisitor::new(Path::new("test.java"));
            if let Ok(items) = visitor.analyze_java_source(&source) {
                let method_items: Vec<_> = items.iter()
                    .filter(|item| matches!(item, AstItem::Function { .. }))
                    .collect();

                // Should extract all methods
                prop_assert_eq!(method_items.len(), method_count);

                // All should be methods with real names
                for (i, item) in method_items.iter().enumerate() {
                    if let AstItem::Function { name, .. } = item {
                        let expected_name = format!("method{}", i);
                        prop_assert!(name.contains(&expected_name));
                    }
                }
            }
        }

        #[test]
        fn test_java_complexity_stays_bounded(
            depth in 1u32..5
        ) {
            let mut source = String::from("package test;\n\npublic class Test {\npublic void complexMethod() {\n");
            for _ in 0..depth {
                source.push_str("if (true) {\n");
            }
            source.push_str("return;\n");
            for _ in 0..depth {
                source.push_str("}\n");
            }
            source.push_str("}\n}\n");

            let mut analyzer = JavaComplexityAnalyzer::new();
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
