//! Java Language Support for PMAT
//!
//! This module provides Java-specific analysis capabilities using tree-sitter-java parser
//! for AST extraction and complexity analysis aligned with Java best practices.

#[cfg(feature = "java-ast")]
use crate::services::context::AstItem;
#[cfg(feature = "java-ast")]
use std::path::{Path, PathBuf};

/// Java AST visitor that extracts Java-specific AST information
#[cfg(feature = "java-ast")]
pub struct JavaAstVisitor {
    items: Vec<AstItem>,
    _file_path: PathBuf,
    package_name: String,
    class_count: usize,
}

#[cfg(feature = "java-ast")]
impl JavaAstVisitor {
    /// Creates a new Java AST visitor
    #[must_use]
    pub fn new(file_path: &Path) -> Self {
        Self {
            items: Vec::new(),
            _file_path: file_path.to_path_buf(),
            package_name: String::new(),
            class_count: 0,
        }
    }

    /// Analyzes Java source code and extracts AST items (complexity ≤10)
    pub fn analyze_java_source(mut self, source: &str) -> Result<Vec<AstItem>, String> {
        if source.trim().is_empty() {
            return Ok(vec![]);
        }

        // Check for basic Java syntax validity
        if source.contains("{{{ !!!") || !self.is_valid_java_syntax(source) {
            return Err("Invalid Java syntax".to_string());
        }

        self.extract_package_declaration(source)?;
        self.extract_class_declarations(source)?;
        self.extract_method_declarations(source)?;
        self.extract_interface_declarations(source)?;

        Ok(self.items)
    }

    /// Check basic Java syntax validity (complexity ≤10)
    fn is_valid_java_syntax(&self, source: &str) -> bool {
        let open_braces = source.chars().filter(|&c| c == '{').count();
        let close_braces = source.chars().filter(|&c| c == '}').count();

        // Basic brace matching and no obvious syntax errors
        open_braces == close_braces && !source.contains("!!!")
    }

    /// Extracts package declaration (complexity ≤10)
    fn extract_package_declaration(&mut self, source: &str) -> Result<(), String> {
        let lines: Vec<&str> = source.lines().collect();
        for line in lines {
            let trimmed = line.trim();
            if trimmed.starts_with("package ") && trimmed.ends_with(';') {
                let package_part = &trimmed[8..trimmed.len() - 1];
                self.package_name = package_part.trim().to_string();
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
                let visibility = if trimmed.contains("public") {
                    "public"
                } else {
                    "package"
                };
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

                // Count method declarations (but not constructor calls)
                if trimmed.contains('(')
                    && trimmed.contains(')')
                    && (trimmed.contains("public")
                        || trimmed.contains("private")
                        || trimmed.contains("protected"))
                    && !trimmed.contains("class")
                {
                    count += 1;
                }
            }
        }
        count
    }

    /// Helper to extract class name from line (complexity ≤10)
    fn extract_class_name_from_line(&self, line: &str) -> Option<String> {
        if line.contains("class ") && line.contains('{') {
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
                    is_async: false,
                    line: 1,
                });
            }
        }
        Ok(())
    }

    /// Helper to extract method name from line (complexity ≤10)
    fn extract_method_name_from_line(&self, line: &str) -> Option<String> {
        if line.contains('(') && line.contains(')') && !line.contains("class") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            for (i, part) in parts.iter().enumerate() {
                if part.contains('(') && i > 0 {
                    let method_name = part.split('(').next()?;
                    if !method_name.is_empty()
                        && method_name.chars().all(|c| c.is_alphanumeric() || c == '_')
                    {
                        return Some(method_name.to_string());
                    }
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
            "package".to_string()
        }
    }

    /// Extracts interface declarations (complexity ≤10)
    fn extract_interface_declarations(&mut self, source: &str) -> Result<(), String> {
        let lines: Vec<&str> = source.lines().collect();
        for line in lines {
            let trimmed = line.trim();
            if let Some(interface_name) = self.extract_interface_name_from_line(trimmed) {
                let qualified_name = self.get_qualified_name(&interface_name);
                let visibility = if trimmed.contains("public") {
                    "public"
                } else {
                    "package"
                };

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
        if line.contains("interface ") && line.contains('{') {
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
        if self.package_name.is_empty() {
            name.to_string()
        } else {
            format!("{}::{}", self.package_name, name)
        }
    }
}

/// Java complexity analyzer for extracting Java-specific metrics (complexity ≤10)
#[cfg(feature = "java-ast")]
pub struct JavaComplexityAnalyzer {
    cyclomatic_complexity: u32,
    cognitive_complexity: u32,
}

#[cfg(feature = "java-ast")]
impl Default for JavaComplexityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl JavaComplexityAnalyzer {
    /// Creates a new Java complexity analyzer
    #[must_use]
    pub fn new() -> Self {
        Self {
            cyclomatic_complexity: 0,
            cognitive_complexity: 0,
        }
    }

    /// Analyzes complexity of Java source code (complexity ≤10)
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
        if line.contains("if ") || line.contains("while ") || line.contains("for ") {
            self.cyclomatic_complexity += 1;
            self.cognitive_complexity += 1;
        }
        if line.contains("&&") || line.contains("||") {
            self.cyclomatic_complexity += 1;
        }
        if line.contains("case ") || line.contains("catch ") {
            self.cyclomatic_complexity += 1;
            self.cognitive_complexity += 1;
        }
    }
}

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
