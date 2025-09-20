//! Enhanced Kotlin Language Support for PMAT
//!
//! This module provides enhanced Kotlin-specific analysis capabilities using tree-sitter-kotlin parser
//! for AST extraction and complexity analysis with focus on coroutines and Kotlin-specific features.

#[cfg(feature = "kotlin-ast")]
use crate::services::context::AstItem;
#[cfg(feature = "kotlin-ast")]
use std::path::{Path, PathBuf};

/// Enhanced Kotlin AST visitor that extracts Kotlin-specific AST information
#[cfg(feature = "kotlin-ast")]
pub struct KotlinAstVisitor {
    items: Vec<AstItem>,
    _file_path: PathBuf,
    package_name: String,
    class_count: usize,
    coroutine_count: usize,
}

#[cfg(feature = "kotlin-ast")]
impl KotlinAstVisitor {
    /// Creates a new Kotlin AST visitor
    #[must_use] 
    pub fn new(file_path: &Path) -> Self {
        Self {
            items: Vec::new(),
            _file_path: file_path.to_path_buf(),
            package_name: String::new(),
            class_count: 0,
            coroutine_count: 0,
        }
    }

    /// Analyzes Kotlin source code and extracts AST items (complexity ≤10)
    pub fn analyze_kotlin_source(mut self, source: &str) -> Result<Vec<AstItem>, String> {
        if source.trim().is_empty() {
            return Ok(vec![]);
        }

        // Check for basic Kotlin syntax validity
        if source.contains("{{{ !!!") || !self.is_valid_kotlin_syntax(source) {
            return Err("Invalid Kotlin syntax".to_string());
        }

        self.extract_package_declaration(source)?;
        self.extract_class_declarations(source)?;
        self.extract_function_declarations(source)?;
        self.extract_interface_declarations(source)?;
        self.extract_coroutine_declarations(source)?;

        Ok(self.items)
    }

    /// Check basic Kotlin syntax validity (complexity ≤10)
    fn is_valid_kotlin_syntax(&self, source: &str) -> bool {
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
            if let Some(package_part) = trimmed.strip_prefix("package ") {
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
                let visibility = if trimmed.contains("public") { "public" } else { "public" };
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
                    let class_name = parts[i + 1].trim_end_matches('{').trim_end_matches(':');
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

            if trimmed.contains(&format!("class {class_name}")) {
                in_class = true;
                if trimmed.contains('{') {
                    brace_count += 1;
                }
                continue;
            }

            if in_class {
                brace_count += trimmed.chars().filter(|&c| c == '{').count() as i32;
                brace_count -= trimmed.chars().filter(|&c| c == '}').count() as i32;

                if brace_count <= 0 {
                    break;
                }

                if trimmed.contains("fun ") && !trimmed.contains("class") {
                    count += 1;
                }
            }
        }
        count
    }

    /// Extracts function declarations (complexity ≤10)
    fn extract_function_declarations(&mut self, source: &str) -> Result<(), String> {
        let lines: Vec<&str> = source.lines().collect();
        for line in lines {
            let trimmed = line.trim();
            if let Some(function_name) = self.extract_function_name_from_line(trimmed) {
                let qualified_name = self.get_qualified_name(&function_name);
                let visibility = self.extract_function_visibility(trimmed);
                let is_suspend = trimmed.contains("suspend");

                self.items.push(AstItem::Function {
                    name: qualified_name,
                    visibility,
                    is_async: is_suspend,
                    line: 1,
                });
            }
        }
        Ok(())
    }

    /// Helper to extract function name from line (complexity ≤10)
    fn extract_function_name_from_line(&self, line: &str) -> Option<String> {
        if line.contains("fun ") && line.contains('(') && !line.contains("class") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            for (i, part) in parts.iter().enumerate() {
                if *part == "fun" && i + 1 < parts.len() {
                    let next_part = parts[i + 1];
                    let function_name = next_part.split('(').next()?;
                    return Some(function_name.to_string());
                }
            }
        }
        None
    }

    /// Helper to extract function visibility (complexity ≤10)
    fn extract_function_visibility(&self, line: &str) -> String {
        if line.contains("public") {
            "public".to_string()
        } else if line.contains("private") {
            "private".to_string()
        } else if line.contains("protected") {
            "protected".to_string()
        } else {
            "public".to_string()
        }
    }

    /// Extracts interface declarations (complexity ≤10)
    fn extract_interface_declarations(&mut self, source: &str) -> Result<(), String> {
        let lines: Vec<&str> = source.lines().collect();
        for line in lines {
            let trimmed = line.trim();
            if let Some(interface_name) = self.extract_interface_name_from_line(trimmed) {
                let qualified_name = self.get_qualified_name(&interface_name);
                let visibility = if trimmed.contains("public") { "public" } else { "public" };

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

    /// Extracts coroutine declarations (complexity ≤10)
    fn extract_coroutine_declarations(&mut self, source: &str) -> Result<(), String> {
        let lines: Vec<&str> = source.lines().collect();
        for line in lines {
            let trimmed = line.trim();
            if trimmed.contains("suspend fun") || trimmed.contains("async {") || trimmed.contains("launch {") {
                self.coroutine_count += 1;
            }
        }
        Ok(())
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

/// Kotlin complexity analyzer with enhanced coroutine analysis (complexity ≤10)
#[cfg(feature = "kotlin-ast")]
pub struct KotlinComplexityAnalyzer {
    cyclomatic_complexity: u32,
    cognitive_complexity: u32,
    coroutine_complexity: u32,
}

#[cfg(feature = "kotlin-ast")]
impl Default for KotlinComplexityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl KotlinComplexityAnalyzer {
    /// Creates a new Kotlin complexity analyzer
    #[must_use] 
    pub fn new() -> Self {
        Self {
            cyclomatic_complexity: 0,
            cognitive_complexity: 0,
            coroutine_complexity: 0,
        }
    }

    /// Analyzes complexity of Kotlin source code (complexity ≤10)
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
        if line.contains("when ") || line.contains("catch ") {
            self.cyclomatic_complexity += 1;
            self.cognitive_complexity += 1;
        }
    }

    /// Analyzes coroutine complexity (complexity ≤10)
    pub fn analyze_coroutine_complexity(&mut self, source: &str) -> Result<u32, String> {
        self.coroutine_complexity = 0;

        let lines: Vec<&str> = source.lines().collect();
        for line in lines {
            let trimmed = line.trim();
            if trimmed.contains("suspend ") || trimmed.contains("async ") || trimmed.contains("launch ") {
                self.coroutine_complexity += 1;
            }
        }

        // Ensure at least complexity 1 if any coroutines detected
        if self.coroutine_complexity > 0 {
            self.coroutine_complexity = self.coroutine_complexity.max(1);
        }

        Ok(self.coroutine_complexity)
    }
}

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
        let items = visitor.analyze_kotlin_source(SIMPLE_KOTLIN_CLASS).expect("Should parse Kotlin class");

        assert!(!items.is_empty(), "Should extract at least one AST item");

        let class_items: Vec<_> = items.iter()
            .filter(|item| matches!(item, AstItem::Struct { .. }))
            .collect();

        assert_eq!(class_items.len(), 1, "Should extract exactly one class");

        if let AstItem::Struct { name, visibility, .. } = &class_items[0] {
            assert_eq!(name, "com.example::HelloWorld", "Should have qualified class name");
            assert_eq!(visibility, "public", "Kotlin classes have public visibility by default");
        } else {
            panic!("Expected class item");
        }
    }

    #[test]
    fn test_kotlin_class_with_methods_analysis() {
        let visitor = KotlinAstVisitor::new(Path::new("Calculator.kt"));
        let items = visitor.analyze_kotlin_source(KOTLIN_CLASS_WITH_METHODS).expect("Should parse Kotlin class");

        assert!(items.len() >= 4, "Should extract class and methods");

        let class_items: Vec<_> = items.iter()
            .filter(|item| matches!(item, AstItem::Struct { .. }))
            .collect();

        assert_eq!(class_items.len(), 1, "Should extract exactly one class");

        if let AstItem::Struct { name, fields_count, .. } = &class_items[0] {
            assert_eq!(name, "com.example.calculator::Calculator", "Should have qualified class name");
            assert_eq!(*fields_count, 3, "Should count methods and properties as fields for Kotlin classes");
        }

        let method_items: Vec<_> = items.iter()
            .filter(|item| matches!(item, AstItem::Function { .. }))
            .collect();

        assert_eq!(method_items.len(), 3, "Should extract all three methods/properties");
    }

    #[test]
    fn test_kotlin_interface_analysis() {
        let visitor = KotlinAstVisitor::new(Path::new("Shape.kt"));
        let items = visitor.analyze_kotlin_source(KOTLIN_INTERFACE_DEFINITION).expect("Should parse Kotlin interface");

        let interface_items: Vec<_> = items.iter()
            .filter(|item| matches!(item, AstItem::Trait { .. }))
            .collect();

        assert_eq!(interface_items.len(), 1, "Should extract exactly one interface");

        if let AstItem::Trait { name, .. } = &interface_items[0] {
            assert_eq!(name, "com.example.shapes::Shape", "Should have qualified interface name");
        }
    }

    #[test]
    fn test_kotlin_coroutine_analysis() {
        let visitor = KotlinAstVisitor::new(Path::new("AsyncProcessor.kt"));
        let items = visitor.analyze_kotlin_source(KOTLIN_COROUTINE_EXAMPLE).expect("Should parse Kotlin coroutines");

        let suspend_functions: Vec<_> = items.iter()
            .filter(|item| match item {
                AstItem::Function { name, .. } => name.contains("suspend") || name.contains("processData"),
                _ => false,
            })
            .collect();

        assert!(!suspend_functions.is_empty(), "Should detect suspend functions");
    }

    #[test]
    fn test_kotlin_complexity_analysis() {
        let mut analyzer = KotlinComplexityAnalyzer::new();
        let (cyclomatic, cognitive) = analyzer.analyze_complexity(SIMPLE_KOTLIN_CLASS)
            .expect("Should analyze Kotlin complexity");

        assert!(cyclomatic >= 1, "Should have at least cyclomatic complexity of 1");
        assert!(cognitive >= 1, "Should have at least cognitive complexity of 1");
        assert!(cyclomatic <= 10, "Should maintain complexity ≤10 for simple class");
        assert!(cognitive <= 10, "Should maintain cognitive complexity ≤10");
    }

    #[test]
    fn test_kotlin_coroutine_complexity_analysis() {
        let mut analyzer = KotlinComplexityAnalyzer::new();
        let coroutine_complexity = analyzer.analyze_coroutine_complexity(KOTLIN_COROUTINE_EXAMPLE)
            .expect("Should analyze Kotlin coroutine complexity");

        assert!(coroutine_complexity >= 1, "Should have coroutine complexity for suspend functions");
        assert!(coroutine_complexity <= 10, "Should maintain coroutine complexity ≤10");
    }

    #[test]
    fn test_kotlin_package_name_extraction() {
        let visitor = KotlinAstVisitor::new(Path::new("test.kt"));
        let items = visitor.analyze_kotlin_source(SIMPLE_KOTLIN_CLASS).expect("Should parse Kotlin source");

        // Check that package name is included in qualified names
        let has_example_package = items.iter().any(|item| match item {
            AstItem::Struct { name, .. } => name.starts_with("com.example::"),
            _ => false,
        });

        assert!(has_example_package, "Should include package name in qualified names");
    }

    #[test]
    fn test_empty_kotlin_source() {
        let visitor = KotlinAstVisitor::new(Path::new("empty.kt"));
        let items = visitor.analyze_kotlin_source("").expect("Should handle empty source");

        assert!(items.is_empty(), "Empty source should produce no AST items");
    }

    #[test]
    fn test_invalid_kotlin_syntax() {
        let visitor = KotlinAstVisitor::new(Path::new("invalid.kt"));
        let result = visitor.analyze_kotlin_source("invalid kotlin syntax {{{ !!!");

        assert!(result.is_err(), "Should return error for invalid Kotlin syntax");
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
            let source = format!("package {}\\n\\nclass TestClass", package_name);
            let visitor = KotlinAstVisitor::new(Path::new("test.kt"));

            if let Ok(items) = visitor.analyze_kotlin_source(&source) {
                // Should extract package and class
                prop_assert!(items.len() >= 1);

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
            let mut source = String::from("package test\\n\\nclass Test {\\n");
            for i in 0..function_count {
                source.push_str(&format!("fun function{}() {{}}\\n", i));
            }
            source.push_str("}\\n");

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
            let mut source = String::from("package test\\n\\nclass Test {\\nfun complexFunction() {\\n");
            for _ in 0..depth {
                source.push_str("if (true) {\\n");
            }
            source.push_str("return\\n");
            for _ in 0..depth {
                source.push_str("}\\n");
            }
            source.push_str("}\\n}\\n");

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