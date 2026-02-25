#[cfg(all(test, feature = "kotlin-ast"))]
mod tests {
    use super::*;
    use std::path::Path;

    const SIMPLE_KOTLIN_CLASS: &str = r#"
package com.example

class HelloWorld {
    fun main() {
        println("Hello, World!")
    }
}
"#;

    const KOTLIN_CLASS_WITH_METHODS: &str = r#"
package com.example.calculator

class Calculator {
    private var result: Double = 0.0

    fun add(x: Double, y: Double): Double {
        result = x + y
        return result
    }

    fun multiply(x: Double, y: Double): Double {
        result = x * y
        return result
    }

    val currentResult: Double
        get() = result
}
"#;

    const KOTLIN_INTERFACE_DEFINITION: &str = r#"
package com.example.shapes

interface Shape {
    val area: Double
    val perimeter: Double
}

class Circle(private val radius: Double) : Shape {
    override val area: Double
        get() = kotlin.math.PI * radius * radius

    override val perimeter: Double
        get() = 2 * kotlin.math.PI * radius
}
"#;

    const KOTLIN_COROUTINE_EXAMPLE: &str = r#"
package com.example.async

import kotlinx.coroutines.*

class AsyncProcessor {
    suspend fun processData(data: String): String {
        delay(100)
        return data.uppercase()
    }

    fun launchProcessing() = runBlocking {
        val job = launch {
            val result = processData("hello")
            println(result)
        }
        job.join()
    }
}
"#;

    #[test]
    fn test_simple_kotlin_class_analysis() {
        let visitor = KotlinAstVisitor::new(Path::new("HelloWorld.kt"));
        let items = visitor
            .analyze_kotlin_source(SIMPLE_KOTLIN_CLASS)
            .expect("Should parse Kotlin class");

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
            assert_eq!(
                visibility, "public",
                "Kotlin classes have public visibility by default"
            );
        } else {
            panic!("Expected class item");
        }
    }

    #[test]
    fn test_kotlin_class_with_methods_analysis() {
        let visitor = KotlinAstVisitor::new(Path::new("Calculator.kt"));
        let items = visitor
            .analyze_kotlin_source(KOTLIN_CLASS_WITH_METHODS)
            .expect("Should parse Kotlin class");

        assert!(items.len() >= 3, "Should extract class and methods");

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
                *fields_count, 2,
                "Should count methods as fields for Kotlin classes"
            );
        }

        let method_items: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Function { .. }))
            .collect();

        assert_eq!(method_items.len(), 2, "Should extract all two methods");
    }

    #[test]
    fn test_kotlin_interface_analysis() {
        let visitor = KotlinAstVisitor::new(Path::new("Shape.kt"));
        let items = visitor
            .analyze_kotlin_source(KOTLIN_INTERFACE_DEFINITION)
            .expect("Should parse Kotlin interface");

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
    fn test_kotlin_coroutine_analysis() {
        let visitor = KotlinAstVisitor::new(Path::new("AsyncProcessor.kt"));
        let items = visitor
            .analyze_kotlin_source(KOTLIN_COROUTINE_EXAMPLE)
            .expect("Should parse Kotlin coroutines");

        let suspend_functions: Vec<_> = items
            .iter()
            .filter(|item| match item {
                AstItem::Function { name, .. } => {
                    name.contains("suspend") || name.contains("processData")
                }
                _ => false,
            })
            .collect();

        assert!(
            !suspend_functions.is_empty(),
            "Should detect suspend functions"
        );
    }

    #[test]
    fn test_kotlin_complexity_analysis() {
        let mut analyzer = KotlinComplexityAnalyzer::new();
        let (cyclomatic, cognitive) = analyzer
            .analyze_complexity(SIMPLE_KOTLIN_CLASS)
            .expect("Should analyze Kotlin complexity");

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
    fn test_kotlin_coroutine_complexity_analysis() {
        let mut analyzer = KotlinComplexityAnalyzer::new();
        let coroutine_complexity = analyzer
            .analyze_coroutine_complexity(KOTLIN_COROUTINE_EXAMPLE)
            .expect("Should analyze Kotlin coroutine complexity");

        assert!(
            coroutine_complexity >= 1,
            "Should have coroutine complexity for suspend functions"
        );
        assert!(
            coroutine_complexity <= 10,
            "Should maintain coroutine complexity ≤10"
        );
    }

    #[test]
    fn test_kotlin_package_name_extraction() {
        let visitor = KotlinAstVisitor::new(Path::new("test.kt"));
        let items = visitor
            .analyze_kotlin_source(SIMPLE_KOTLIN_CLASS)
            .expect("Should parse Kotlin source");

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
    fn test_empty_kotlin_source() {
        let visitor = KotlinAstVisitor::new(Path::new("empty.kt"));
        let items = visitor
            .analyze_kotlin_source("")
            .expect("Should handle empty source");

        assert!(items.is_empty(), "Empty source should produce no AST items");
    }

    #[test]
    fn test_invalid_kotlin_syntax() {
        let visitor = KotlinAstVisitor::new(Path::new("invalid.kt"));
        let result = visitor.analyze_kotlin_source("invalid kotlin syntax {{{ !!!");

        assert!(
            result.is_err(),
            "Should return error for invalid Kotlin syntax"
        );
    }
}

#[cfg(all(test, feature = "kotlin-ast"))]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    use std::path::Path;

    proptest! {
        #[test]
        fn test_kotlin_visitor_handles_any_valid_package_name(
            package_name in "[a-z][a-z0-9_]*\\.[a-z][a-z0-9_]*"
        ) {
            let source = format!("package {}\n\nclass TestClass", package_name);
            let visitor = KotlinAstVisitor::new(Path::new("test.kt"));

            if let Ok(items) = visitor.analyze_kotlin_source(&source) {
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
        fn test_kotlin_complexity_analyzer_bounds(
            function_count in 1usize..10
        ) {
            let mut source = String::from("package test\n\nclass Test {\n");
            for i in 0..function_count {
                source.push_str(&format!("fun function{}() {{}}\n", i));
            }
            source.push_str("}\n");

            let visitor = KotlinAstVisitor::new(Path::new("test.kt"));
            if let Ok(items) = visitor.analyze_kotlin_source(&source) {
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
        fn test_kotlin_complexity_stays_bounded(
            depth in 1u32..5
        ) {
            let mut source = String::from("package test\n\nclass Test {\nfun complexFunction() {\n");
            for _ in 0..depth {
                source.push_str("if (true) {\n");
            }
            source.push_str("return\n");
            for _ in 0..depth {
                source.push_str("}\n");
            }
            source.push_str("}\n}\n");

            let mut analyzer = KotlinComplexityAnalyzer::new();
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
