// C# unit tests - included from csharp.rs

#[cfg(all(test, feature = "csharp-ast"))]
mod tests {
    use super::*;
    use std::path::Path;

    const SIMPLE_CSHARP_CLASS: &str = r#"
using System;

namespace Example
{
    public class HelloWorld
    {
        public static void Main(string[] args)
        {
            Console.WriteLine("Hello, World!");
        }
    }
}
"#;

    const CSHARP_CLASS_WITH_METHODS: &str = r#"
using System;

namespace Example.Calculator
{
    public class Calculator
    {
        private double result;

        public double Add(double x, double y)
        {
            this.result = x + y;
            return this.result;
        }

        public double Multiply(double x, double y)
        {
            this.result = x * y;
            return this.result;
        }

        public double Result => this.result;
    }
}
"#;

    const CSHARP_INTERFACE_DEFINITION: &str = r#"
using System;

namespace Example.Shapes
{
    public interface IShape
    {
        double Area { get; }
        double Perimeter { get; }
    }

    public class Circle : IShape
    {
        private readonly double radius;

        public Circle(double radius)
        {
            this.radius = radius;
        }

        public double Area => Math.PI * radius * radius;

        public double Perimeter => 2 * Math.PI * radius;
    }
}
"#;

    #[test]
    fn test_simple_csharp_class_analysis() {
        let visitor = CSharpAstVisitor::new(Path::new("HelloWorld.cs"));
        let items = visitor
            .analyze_csharp_source(SIMPLE_CSHARP_CLASS)
            .expect("Should parse C# class");

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
                name, "Example::HelloWorld",
                "Should have qualified class name"
            );
            assert_eq!(visibility, "public", "C# classes have public visibility");
        } else {
            panic!("Expected class item");
        }
    }

    #[test]
    fn test_csharp_class_with_methods_analysis() {
        let visitor = CSharpAstVisitor::new(Path::new("Calculator.cs"));
        let items = visitor
            .analyze_csharp_source(CSHARP_CLASS_WITH_METHODS)
            .expect("Should parse C# class");

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
                name, "Example.Calculator::Calculator",
                "Should have qualified class name"
            );
            assert_eq!(
                *fields_count, 3,
                "Should count methods and properties as fields for C# classes"
            );
        }

        let method_items: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Function { .. }))
            .collect();

        assert_eq!(
            method_items.len(),
            3,
            "Should extract all three methods/properties"
        );
    }

    #[test]
    fn test_csharp_interface_analysis() {
        let visitor = CSharpAstVisitor::new(Path::new("IShape.cs"));
        let items = visitor
            .analyze_csharp_source(CSHARP_INTERFACE_DEFINITION)
            .expect("Should parse C# interface");

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
                name, "Example.Shapes::IShape",
                "Should have qualified interface name"
            );
        }
    }

    #[test]
    fn test_csharp_complexity_analysis() {
        let mut analyzer = CSharpComplexityAnalyzer::new();
        let (cyclomatic, cognitive) = analyzer
            .analyze_complexity(SIMPLE_CSHARP_CLASS)
            .expect("Should analyze C# complexity");

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
    fn test_csharp_namespace_name_extraction() {
        let visitor = CSharpAstVisitor::new(Path::new("test.cs"));
        let items = visitor
            .analyze_csharp_source(SIMPLE_CSHARP_CLASS)
            .expect("Should parse C# source");

        // Check that namespace name is included in qualified names
        let has_example_namespace = items.iter().any(|item| match item {
            AstItem::Struct { name, .. } => name.starts_with("Example::"),
            _ => false,
        });

        assert!(
            has_example_namespace,
            "Should include namespace name in qualified names"
        );
    }

    #[test]
    fn test_empty_csharp_source() {
        let visitor = CSharpAstVisitor::new(Path::new("empty.cs"));
        let items = visitor
            .analyze_csharp_source("")
            .expect("Should handle empty source");

        assert!(items.is_empty(), "Empty source should produce no AST items");
    }

    #[test]
    fn test_invalid_csharp_syntax() {
        let visitor = CSharpAstVisitor::new(Path::new("invalid.cs"));
        let result = visitor.analyze_csharp_source("invalid csharp syntax {{{ !!!");

        assert!(result.is_err(), "Should return error for invalid C# syntax");
    }
}
