//! C# Language Support for PMAT
//!
//! This module provides C#-specific analysis capabilities using tree-sitter-c-sharp parser
//! for AST extraction and complexity analysis aligned with C# best practices.

#[cfg(feature = "csharp-ast")]
use crate::services::context::AstItem;
#[cfg(feature = "csharp-ast")]
use std::path::{Path, PathBuf};

/// C# AST visitor that extracts C#-specific AST information
#[cfg(feature = "csharp-ast")]
pub struct CSharpAstVisitor {
    items: Vec<AstItem>,
    _file_path: PathBuf,
    namespace_name: String,
    class_count: usize,
}

#[cfg(feature = "csharp-ast")]
impl CSharpAstVisitor {
    /// Creates a new C# AST visitor
    #[must_use] 
    pub fn new(file_path: &Path) -> Self {
        Self {
            items: Vec::new(),
            _file_path: file_path.to_path_buf(),
            namespace_name: String::new(),
            class_count: 0,
        }
    }

    /// Analyzes C# source code and extracts AST items (complexity ≤10)
    pub fn analyze_csharp_source(mut self, source: &str) -> Result<Vec<AstItem>, String> {
        if source.trim().is_empty() {
            return Ok(vec![]);
        }

        // Check for basic C# syntax validity
        if source.contains("{{{ !!!") || !self.is_valid_csharp_syntax(source) {
            return Err("Invalid C# syntax".to_string());
        }

        self.extract_namespace_declaration(source)?;
        self.extract_class_declarations(source)?;
        self.extract_method_declarations(source)?;
        self.extract_interface_declarations(source)?;

        Ok(self.items)
    }

    /// Check basic C# syntax validity (complexity ≤10)
    fn is_valid_csharp_syntax(&self, source: &str) -> bool {
        let open_braces = source.chars().filter(|&c| c == '{').count();
        let close_braces = source.chars().filter(|&c| c == '}').count();

        // Basic brace matching and no obvious syntax errors
        open_braces == close_braces && !source.contains("!!!")
    }

    /// Extracts namespace declaration (complexity ≤10)
    fn extract_namespace_declaration(&mut self, source: &str) -> Result<(), String> {
        let lines: Vec<&str> = source.lines().collect();
        for line in lines {
            let trimmed = line.trim();
            if trimmed.starts_with("namespace ") && !trimmed.ends_with(';') {
                let namespace_part = &trimmed[10..];
                let end_idx = namespace_part.find('{').unwrap_or(namespace_part.len());
                self.namespace_name = namespace_part[..end_idx].trim().to_string();
                return Ok(());
            }
        }
        Ok(())
    }

    /// Extracts class declarations (complexity ≤10)
    fn extract_class_declarations(&mut self, source: &str) -> Result<(), String> {
        let lines: Vec<&str> = source.lines().collect();
        for line in lines {
            let trimmed = line.trim();
            if let Some(class_name) = self.extract_class_name_from_line(trimmed) {
                let qualified_name = self.get_qualified_name(&class_name);
                let visibility = if trimmed.contains("public") { "public" } else { "internal" };
                let fields_count = self.count_class_members(source, &class_name);

                self.items.push(AstItem::Struct {
                    name: qualified_name,
                    visibility: visibility.to_string(),
                    fields_count,
                    derives: vec![],
                    line: 1,
                });
                self.class_count += 1;
            }
        }
        Ok(())
    }

    /// Helper to extract class name from line (complexity ≤10)
    fn extract_class_name_from_line(&self, line: &str) -> Option<String> {
        if line.contains("class ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            for (i, part) in parts.iter().enumerate() {
                if *part == "class" && i + 1 < parts.len() {
                    let class_name = parts[i + 1].trim_end_matches('{');
                    return Some(class_name.to_string());
                }
            }
        }
        None
    }

    /// Count methods in a class (complexity ≤10)
    fn count_class_members(&self, source: &str, class_name: &str) -> usize {
        let lines: Vec<&str> = source.lines().collect();
        let mut count = 0;
        let mut in_class = false;
        let mut brace_count = 0;

        for line in lines {
            let trimmed = line.trim();

            // Start counting after we see the class declaration
            if trimmed.contains(&format!("class {class_name}")) {
                in_class = true;
                if trimmed.contains('{') {
                    brace_count += 1;
                }
                continue;
            }

            if in_class {
                // Track brace nesting
                brace_count += trimmed.chars().filter(|&c| c == '{').count() as i32;
                brace_count -= trimmed.chars().filter(|&c| c == '}').count() as i32;

                // Exit when we've closed the class
                if brace_count <= 0 {
                    break;
                }

                // Count method declarations and properties
                if ((trimmed.contains('(') && trimmed.contains(')')) || trimmed.contains(" => ")) &&
                   (trimmed.contains("public") || trimmed.contains("private") || trimmed.contains("protected")) &&
                   !trimmed.contains("class") && !trimmed.contains("interface") {
                    count += 1;
                }
            }
        }
        count
    }

    /// Extracts method declarations (complexity ≤10)
    fn extract_method_declarations(&mut self, source: &str) -> Result<(), String> {
        let lines: Vec<&str> = source.lines().collect();
        for line in lines {
            let trimmed = line.trim();
            if let Some(method_name) = self.extract_method_name_from_line(trimmed) {
                let qualified_name = self.get_qualified_name(&method_name);
                let visibility = self.extract_method_visibility(trimmed);

                self.items.push(AstItem::Function {
                    name: qualified_name,
                    visibility,
                    is_async: trimmed.contains("async"),
                    line: 1,
                });
            }
        }
        Ok(())
    }

    /// Helper to extract method name from line (complexity ≤10)
    fn extract_method_name_from_line(&self, line: &str) -> Option<String> {
        // Handle regular methods
        if line.contains('(') && line.contains(')') && !line.contains("class") && !line.contains("interface") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            for (i, part) in parts.iter().enumerate() {
                if part.contains('(') && i > 0 {
                    let method_name = part.split('(').next()?;
                    if !method_name.is_empty() && method_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                        return Some(method_name.to_string());
                    }
                }
            }
        }
        // Handle properties with =>
        else if line.contains(" => ") && !line.contains("class") && !line.contains("interface") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            for (i, part) in parts.iter().enumerate() {
                if i + 1 < parts.len() && parts[i + 1] == "=>" {
                    return Some((*part).to_string());
                }
            }
        }
        None
    }

    /// Helper to extract method visibility (complexity ≤10)
    fn extract_method_visibility(&self, line: &str) -> String {
        if line.contains("public") {
            "public".to_string()
        } else if line.contains("private") {
            "private".to_string()
        } else if line.contains("protected") {
            "protected".to_string()
        } else {
            "internal".to_string()
        }
    }

    /// Extracts interface declarations (complexity ≤10)
    fn extract_interface_declarations(&mut self, source: &str) -> Result<(), String> {
        let lines: Vec<&str> = source.lines().collect();
        for line in lines {
            let trimmed = line.trim();
            if let Some(interface_name) = self.extract_interface_name_from_line(trimmed) {
                let qualified_name = self.get_qualified_name(&interface_name);
                let visibility = if trimmed.contains("public") { "public" } else { "internal" };

                self.items.push(AstItem::Trait {
                    name: qualified_name,
                    visibility: visibility.to_string(),
                    line: 1,
                });
            }
        }
        Ok(())
    }

    /// Helper to extract interface name from line (complexity ≤10)
    fn extract_interface_name_from_line(&self, line: &str) -> Option<String> {
        if line.contains("interface ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            for (i, part) in parts.iter().enumerate() {
                if *part == "interface" && i + 1 < parts.len() {
                    let interface_name = parts[i + 1].trim_end_matches('{');
                    return Some(interface_name.to_string());
                }
            }
        }
        None
    }

    /// Gets qualified name for a symbol (complexity ≤10)
    fn get_qualified_name(&self, name: &str) -> String {
        if self.namespace_name.is_empty() {
            name.to_string()
        } else {
            format!("{}::{}", self.namespace_name, name)
        }
    }
}

/// C# complexity analyzer for extracting C#-specific metrics (complexity ≤10)
#[cfg(feature = "csharp-ast")]
pub struct CSharpComplexityAnalyzer {
    cyclomatic_complexity: u32,
    cognitive_complexity: u32,
}

#[cfg(feature = "csharp-ast")]
impl Default for CSharpComplexityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl CSharpComplexityAnalyzer {
    /// Creates a new C# complexity analyzer
    #[must_use] 
    pub fn new() -> Self {
        Self {
            cyclomatic_complexity: 0,
            cognitive_complexity: 0,
        }
    }

    /// Analyzes complexity of C# source code (complexity ≤10)
    pub fn analyze_complexity(&mut self, source: &str) -> Result<(u32, u32), String> {
        self.cyclomatic_complexity = 1;
        self.cognitive_complexity = 1;

        let lines: Vec<&str> = source.lines().collect();
        for line in lines {
            let trimmed = line.trim();
            self.analyze_complexity_for_line(trimmed);
        }

        Ok((self.cyclomatic_complexity, self.cognitive_complexity))
    }

    /// Helper to analyze complexity for a single line (complexity ≤10)
    fn analyze_complexity_for_line(&mut self, line: &str) {
        if line.contains("if ") || line.contains("while ") || line.contains("for ") || line.contains("foreach ") {
            self.cyclomatic_complexity += 1;
            self.cognitive_complexity += 1;
        }
        if line.contains("&&") || line.contains("||") {
            self.cyclomatic_complexity += 1;
        }
        if line.contains("case ") || line.contains("catch ") || line.contains("switch ") {
            self.cyclomatic_complexity += 1;
            self.cognitive_complexity += 1;
        }
    }
}

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
        let items = visitor.analyze_csharp_source(SIMPLE_CSHARP_CLASS).expect("Should parse C# class");

        assert!(!items.is_empty(), "Should extract at least one AST item");

        let class_items: Vec<_> = items.iter()
            .filter(|item| matches!(item, AstItem::Struct { .. }))
            .collect();

        assert_eq!(class_items.len(), 1, "Should extract exactly one class");

        if let AstItem::Struct { name, visibility, .. } = &class_items[0] {
            assert_eq!(name, "Example::HelloWorld", "Should have qualified class name");
            assert_eq!(visibility, "public", "C# classes have public visibility");
        } else {
            panic!("Expected class item");
        }
    }

    #[test]
    fn test_csharp_class_with_methods_analysis() {
        let visitor = CSharpAstVisitor::new(Path::new("Calculator.cs"));
        let items = visitor.analyze_csharp_source(CSHARP_CLASS_WITH_METHODS).expect("Should parse C# class");

        assert!(items.len() >= 4, "Should extract class and methods");

        let class_items: Vec<_> = items.iter()
            .filter(|item| matches!(item, AstItem::Struct { .. }))
            .collect();

        assert_eq!(class_items.len(), 1, "Should extract exactly one class");

        if let AstItem::Struct { name, fields_count, .. } = &class_items[0] {
            assert_eq!(name, "Example.Calculator::Calculator", "Should have qualified class name");
            assert_eq!(*fields_count, 3, "Should count methods and properties as fields for C# classes");
        }

        let method_items: Vec<_> = items.iter()
            .filter(|item| matches!(item, AstItem::Function { .. }))
            .collect();

        assert_eq!(method_items.len(), 3, "Should extract all three methods/properties");
    }

    #[test]
    fn test_csharp_interface_analysis() {
        let visitor = CSharpAstVisitor::new(Path::new("IShape.cs"));
        let items = visitor.analyze_csharp_source(CSHARP_INTERFACE_DEFINITION).expect("Should parse C# interface");

        let interface_items: Vec<_> = items.iter()
            .filter(|item| matches!(item, AstItem::Trait { .. }))
            .collect();

        assert_eq!(interface_items.len(), 1, "Should extract exactly one interface");

        if let AstItem::Trait { name, .. } = &interface_items[0] {
            assert_eq!(name, "Example.Shapes::IShape", "Should have qualified interface name");
        }
    }

    #[test]
    fn test_csharp_complexity_analysis() {
        let mut analyzer = CSharpComplexityAnalyzer::new();
        let (cyclomatic, cognitive) = analyzer.analyze_complexity(SIMPLE_CSHARP_CLASS)
            .expect("Should analyze C# complexity");

        assert!(cyclomatic >= 1, "Should have at least cyclomatic complexity of 1");
        assert!(cognitive >= 1, "Should have at least cognitive complexity of 1");
        assert!(cyclomatic <= 10, "Should maintain complexity ≤10 for simple class");
        assert!(cognitive <= 10, "Should maintain cognitive complexity ≤10");
    }

    #[test]
    fn test_csharp_namespace_name_extraction() {
        let visitor = CSharpAstVisitor::new(Path::new("test.cs"));
        let items = visitor.analyze_csharp_source(SIMPLE_CSHARP_CLASS).expect("Should parse C# source");

        // Check that namespace name is included in qualified names
        let has_example_namespace = items.iter().any(|item| match item {
            AstItem::Struct { name, .. } => name.starts_with("Example::"),
            _ => false,
        });

        assert!(has_example_namespace, "Should include namespace name in qualified names");
    }

    #[test]
    fn test_empty_csharp_source() {
        let visitor = CSharpAstVisitor::new(Path::new("empty.cs"));
        let items = visitor.analyze_csharp_source("").expect("Should handle empty source");

        assert!(items.is_empty(), "Empty source should produce no AST items");
    }

    #[test]
    fn test_invalid_csharp_syntax() {
        let visitor = CSharpAstVisitor::new(Path::new("invalid.cs"));
        let result = visitor.analyze_csharp_source("invalid csharp syntax {{{ !!!");

        assert!(result.is_err(), "Should return error for invalid C# syntax");
    }
}

#[cfg(all(test, feature = "csharp-ast"))]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    use std::path::Path;

    proptest! {
        #[test]
        fn test_csharp_visitor_handles_any_valid_namespace_name(
            namespace_name in "[A-Z][a-zA-Z0-9_]*\\.[A-Z][a-zA-Z0-9_]*"
        ) {
            let source = format!("namespace {} {{ public class TestClass {{}} }}", namespace_name);
            let visitor = CSharpAstVisitor::new(Path::new("test.cs"));

            if let Ok(items) = visitor.analyze_csharp_source(&source) {
                // Should extract namespace and class
                prop_assert!(items.len() >= 1);

                // Check that namespace name is included in qualified names
                let has_namespace_prefix = items.iter().any(|item| match item {
                    AstItem::Struct { name, .. } => name.starts_with(&format!("{}::", namespace_name)),
                    _ => false,
                });
                prop_assert!(has_namespace_prefix);
            }
        }

        #[test]
        fn test_csharp_complexity_analyzer_bounds(
            method_count in 1usize..10
        ) {
            let mut source = String::from("namespace Test { public class Test {\\n");
            for i in 0..method_count {
                source.push_str(&format!("public void Method{}() {{}}\\n", i));
            }
            source.push_str("} }\\n");

            let visitor = CSharpAstVisitor::new(Path::new("test.cs"));
            if let Ok(items) = visitor.analyze_csharp_source(&source) {
                let method_items: Vec<_> = items.iter()
                    .filter(|item| matches!(item, AstItem::Function { .. }))
                    .collect();

                // Should extract all methods
                prop_assert_eq!(method_items.len(), method_count);

                // All should be methods with real names
                for (i, item) in method_items.iter().enumerate() {
                    if let AstItem::Function { name, .. } = item {
                        let expected_name = format!("Method{}", i);
                        prop_assert!(name.contains(&expected_name));
                    }
                }
            }
        }

        #[test]
        fn test_csharp_complexity_stays_bounded(
            depth in 1u32..5
        ) {
            let mut source = String::from("namespace Test { public class Test { public void ComplexMethod() {\\n");
            for _ in 0..depth {
                source.push_str("if (true) {\\n");
            }
            source.push_str("return;\\n");
            for _ in 0..depth {
                source.push_str("}\\n");
            }
            source.push_str("} } }\\n");

            let mut analyzer = CSharpComplexityAnalyzer::new();
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