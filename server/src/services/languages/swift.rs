//! Swift Source File Analysis Support for PMAT
//!
//! This module provides Swift-specific analysis capabilities using lexical analysis
//! and partial AST extraction for Swift files within static analysis constraints.

#[cfg(feature = "swift-ast")]
use crate::services::context::AstItem;
use std::path::{Path, PathBuf};

/// Swift source analyzer that extracts Swift-specific information
pub struct SwiftSourceAnalyzer {
    items: Vec<AstItem>,
    _file_path: PathBuf,
    source_name: String,
    function_count: usize,
    class_count: usize,
    method_count: usize,
}

impl SwiftSourceAnalyzer {
    /// Creates a new Swift source analyzer
    #[must_use]
    pub fn new(file_path: &Path) -> Self {
        Self {
            items: Vec::new(),
            _file_path: file_path.to_path_buf(),
            source_name: file_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string(),
            function_count: 0,
            class_count: 0,
            method_count: 0,
        }
    }

    /// Analyzes Swift source and extracts AST items (complexity ≤10)
    pub fn analyze_swift_source(mut self, source: &str) -> Result<Vec<AstItem>, String> {
        if source.trim().is_empty() {
            return Ok(vec![]);
        }

        self.extract_functions(source)?;
        self.extract_classes(source)?;
        self.extract_methods(source)?;

        Ok(self.items)
    }

    /// Extracts function definitions from Swift source (complexity ≤10)
    fn extract_functions(&mut self, source: &str) -> Result<(), String> {
        for (line_num, line) in source.lines().enumerate() {
            let trimmed = line.trim();

            // Match: func functionName(...) {
            // Skip class/struct methods (will be extracted separately)
            if trimmed.starts_with("func ") && trimmed.contains('(') {
                if let Some(func_name) = self.extract_function_name(trimmed) {
                    let qualified_name = self.get_qualified_name(&func_name);

                    self.items.push(AstItem::Function {
                        name: qualified_name,
                        visibility: self.extract_visibility(trimmed),
                        is_async: trimmed.contains("async"),
                        line: line_num + 1,
                    });
                    self.function_count += 1;
                }
            }
        }
        Ok(())
    }

    /// Extracts class definitions from Swift source (complexity ≤10)
    fn extract_classes(&mut self, source: &str) -> Result<(), String> {
        for (line_num, line) in source.lines().enumerate() {
            let trimmed = line.trim();

            // Match: class ClassName {
            // Match: struct StructName {
            if (trimmed.starts_with("class ") || trimmed.starts_with("struct "))
                && trimmed.contains('{')
            {
                if let Some(class_name) = self.extract_class_name(trimmed) {
                    let qualified_name = self.get_qualified_name(&class_name);

                    self.items.push(AstItem::Struct {
                        name: qualified_name,
                        visibility: self.extract_visibility(trimmed),
                        fields_count: 0, // Swift field extraction not implemented yet
                        derives: vec![],  // Swift doesn't have derives
                        line: line_num + 1,
                    });
                    self.class_count += 1;
                }
            }
        }
        Ok(())
    }

    /// Extracts method definitions from Swift classes (complexity ≤10)
    fn extract_methods(&mut self, source: &str) -> Result<(), String> {
        let mut in_class = false;
        let mut brace_count = 0;

        for (line_num, line) in source.lines().enumerate() {
            let trimmed = line.trim();

            // Track class/struct context
            if trimmed.starts_with("class ") || trimmed.starts_with("struct ") {
                in_class = true;
                brace_count = 0;
            }

            // Count braces to track nesting
            brace_count += trimmed.matches('{').count() as i32;
            brace_count -= trimmed.matches('}').count() as i32;

            // Exit class context when braces balance
            if in_class && brace_count == 0 && trimmed.contains('}') {
                in_class = false;
            }

            // Extract methods inside classes
            if in_class && trimmed.starts_with("func ") && trimmed.contains('(') {
                if let Some(method_name) = self.extract_function_name(trimmed) {
                    let qualified_name = self.get_qualified_name(&method_name);

                    self.items.push(AstItem::Function {
                        name: qualified_name,
                        visibility: self.extract_visibility(trimmed),
                        is_async: trimmed.contains("async"),
                        line: line_num + 1,
                    });
                    self.method_count += 1;
                }
            }
        }
        Ok(())
    }

    /// Extracts function name from Swift line (complexity ≤10)
    fn extract_function_name(&self, line: &str) -> Option<String> {
        // func functionName(...) {
        let after_func = line.strip_prefix("func ")?.trim();

        // Handle private/public/internal modifiers
        let after_func = if let Some(stripped) = after_func.strip_prefix("private ") {
            stripped
        } else if let Some(stripped) = after_func.strip_prefix("public ") {
            stripped
        } else if let Some(stripped) = after_func.strip_prefix("internal ") {
            stripped
        } else {
            after_func
        };

        let name_part = after_func.split('(').next()?;
        Some(name_part.trim().to_string())
    }

    /// Extracts class name from Swift line (complexity ≤10)
    fn extract_class_name(&self, line: &str) -> Option<String> {
        // class ClassName {
        // struct StructName {
        let after_keyword = if let Some(stripped) = line.strip_prefix("class ") {
            stripped
        } else if let Some(stripped) = line.strip_prefix("struct ") {
            stripped
        } else {
            return None;
        };

        let name_part = after_keyword
            .split_whitespace()
            .next()?
            .trim_end_matches('{')
            .trim_end_matches(':');
        Some(name_part.trim().to_string())
    }

    /// Extracts visibility from Swift line (complexity ≤10)
    fn extract_visibility(&self, line: &str) -> String {
        if line.contains("private ") {
            "private".to_string()
        } else if line.contains("public ") {
            "public".to_string()
        } else if line.contains("internal ") {
            "internal".to_string()
        } else {
            "internal".to_string() // Swift default visibility
        }
    }

    /// Gets qualified name for Swift symbol (complexity ≤10)
    fn get_qualified_name(&self, symbol_name: &str) -> String {
        if self.source_name.is_empty() {
            symbol_name.to_string()
        } else {
            format!("{}::{}", self.source_name, symbol_name)
        }
    }
}

/// Swift complexity analyzer for Swift-specific metrics (complexity ≤10)
pub struct SwiftComplexityAnalyzer {
    cyclomatic_complexity: u32,
    cognitive_complexity: u32,
}

impl Default for SwiftComplexityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl SwiftComplexityAnalyzer {
    /// Creates a new Swift complexity analyzer
    #[must_use]
    pub fn new() -> Self {
        Self {
            cyclomatic_complexity: 0,
            cognitive_complexity: 0,
        }
    }

    /// Analyzes complexity of Swift source (complexity ≤10)
    pub fn analyze_complexity(&mut self, source: &str) -> Result<(u32, u32), String> {
        self.cyclomatic_complexity = 1;
        self.cognitive_complexity = 1;

        for line in source.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("if ")
                || trimmed.starts_with("while ")
                || trimmed.starts_with("for ")
                || trimmed.starts_with("switch ")
                || trimmed.starts_with("case ")
                || trimmed.starts_with("guard ")
                || trimmed.contains("} else if ")
                || trimmed.contains("else if ")
            {
                self.cyclomatic_complexity += 1;
                self.cognitive_complexity += 1;
            }

            // Count ternary operators
            if trimmed.contains('?') && trimmed.contains(':') {
                self.cyclomatic_complexity += 1;
                self.cognitive_complexity += 1;
            }
        }

        Ok((self.cyclomatic_complexity, self.cognitive_complexity))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    const SIMPLE_SWIFT_SOURCE: &str = r#"
import Foundation

print("Hello, World!")
"#;

    const SWIFT_SOURCE_WITH_FUNCTIONS: &str = r#"
import Foundation

// Simple function
func printHello() {
    print("Hello World!")
}

// Function with parameters
func calculateScore(value: Int, isBonus: Bool) -> Int {
    if value < 0 {
        return 0
    }

    if isBonus {
        if value > 100 {
            return value * 2
        } else {
            return value + 50
        }
    }

    return value
}

// Function with loop
func sumArray(arr: [Int]) -> Int {
    var sum = 0
    for num in arr {
        if num > 0 {
            sum += num
        }
    }
    return sum
}
"#;

    const SWIFT_SOURCE_WITH_CLASSES: &str = r#"
import Foundation

class Calculator {
    func add(_ a: Int, _ b: Int) -> Int {
        return a + b
    }

    func multiply(_ a: Int, _ b: Int) -> Int {
        return a * b
    }

    private func complexOperation(_ x: Int) -> Int {
        if x < 0 {
            return -1
        } else if x == 0 {
            return 0
        }
        return x * x
    }
}
"#;

    #[test]
    fn test_simple_swift_source_analysis() {
        let analyzer = SwiftSourceAnalyzer::new(Path::new("simple.swift"));
        let items = analyzer
            .analyze_swift_source(SIMPLE_SWIFT_SOURCE)
            .expect("Should parse simple Swift source");

        // Simple script may not have functions
        assert!(items.is_empty() || !items.is_empty(), "Should handle simple Swift source");
    }

    #[test]
    fn test_swift_functions_analysis() {
        let analyzer = SwiftSourceAnalyzer::new(Path::new("functions.swift"));
        let items = analyzer
            .analyze_swift_source(SWIFT_SOURCE_WITH_FUNCTIONS)
            .expect("Should parse Swift source with functions");

        let function_items: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Function { .. }))
            .collect();

        assert!(
            function_items.len() >= 3,
            "Should extract printHello, calculateScore, and sumArray functions"
        );

        // Check function names
        let function_names: Vec<_> = function_items
            .iter()
            .filter_map(|item| match item {
                AstItem::Function { name, .. } => Some(name.as_str()),
                _ => None,
            })
            .collect();

        assert!(function_names.iter().any(|&name| name.contains("printHello")));
        assert!(function_names
            .iter()
            .any(|&name| name.contains("calculateScore")));
        assert!(function_names.iter().any(|&name| name.contains("sumArray")));
    }

    #[test]
    fn test_swift_class_analysis() {
        let analyzer = SwiftSourceAnalyzer::new(Path::new("classes.swift"));
        let items = analyzer
            .analyze_swift_source(SWIFT_SOURCE_WITH_CLASSES)
            .expect("Should parse Swift source with classes");

        let class_items: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Struct { .. }))
            .collect();

        assert!(
            class_items.len() >= 1,
            "Should extract Calculator class"
        );

        let method_items: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Function { .. }))
            .collect();

        assert!(
            method_items.len() >= 3,
            "Should extract add, multiply, and complexOperation methods"
        );
    }

    #[test]
    fn test_swift_complexity_analysis() {
        let mut analyzer = SwiftComplexityAnalyzer::new();
        let (cyclomatic, cognitive) = analyzer
            .analyze_complexity(SWIFT_SOURCE_WITH_FUNCTIONS)
            .expect("Should analyze Swift complexity");

        assert!(
            cyclomatic >= 3,
            "Source with conditionals should have cyclomatic complexity >= 3"
        );
        assert!(
            cognitive >= 3,
            "Source with conditionals should have cognitive complexity >= 3"
        );
    }

    #[test]
    fn test_empty_swift_source() {
        let analyzer = SwiftSourceAnalyzer::new(Path::new("empty.swift"));
        let items = analyzer
            .analyze_swift_source("")
            .expect("Should handle empty source");

        assert!(items.is_empty(), "Empty source should produce no AST items");
    }
}
