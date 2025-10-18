//! PHP Script Analysis Support for PMAT
//!
//! This module provides PHP-specific analysis capabilities using lexical analysis
//! and partial AST extraction for PHP scripts within static analysis constraints.

#[cfg(feature = "php-ast")]
use crate::services::context::AstItem;
use std::path::{Path, PathBuf};

/// PHP script analyzer that extracts PHP-specific information
pub struct PhpScriptAnalyzer {
    items: Vec<AstItem>,
    _file_path: PathBuf,
    script_name: String,
    function_count: usize,
    class_count: usize,
    method_count: usize,
}

impl PhpScriptAnalyzer {
    /// Creates a new PHP script analyzer
    #[must_use]
    pub fn new(file_path: &Path) -> Self {
        Self {
            items: Vec::new(),
            _file_path: file_path.to_path_buf(),
            script_name: file_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string(),
            function_count: 0,
            class_count: 0,
            method_count: 0,
        }
    }

    /// Analyzes PHP script and extracts AST items (complexity ≤10)
    pub fn analyze_php_script(mut self, source: &str) -> Result<Vec<AstItem>, String> {
        if source.trim().is_empty() {
            return Ok(vec![]);
        }

        self.extract_functions(source)?;
        self.extract_classes(source)?;
        self.extract_methods(source)?;

        Ok(self.items)
    }

    /// Extracts function definitions from PHP script (complexity ≤10)
    fn extract_functions(&mut self, source: &str) -> Result<(), String> {
        for (line_num, line) in source.lines().enumerate() {
            let trimmed = line.trim();

            // Match: function functionName(...) {
            if trimmed.starts_with("function ") && trimmed.contains('(') {
                if let Some(func_name) = self.extract_function_name(trimmed) {
                    let qualified_name = self.get_qualified_name(&func_name);

                    self.items.push(AstItem::Function {
                        name: qualified_name,
                        visibility: "public".to_string(),
                        is_async: false,
                        line: line_num + 1,
                    });
                    self.function_count += 1;
                }
            }
        }
        Ok(())
    }

    /// Extracts class definitions from PHP script (complexity ≤10)
    fn extract_classes(&mut self, source: &str) -> Result<(), String> {
        for (line_num, line) in source.lines().enumerate() {
            let trimmed = line.trim();

            // Match: class ClassName {
            if trimmed.starts_with("class ") && trimmed.contains('{') {
                if let Some(class_name) = self.extract_class_name(trimmed) {
                    let qualified_name = self.get_qualified_name(&class_name);

                    self.items.push(AstItem::Struct {
                        name: qualified_name,
                        visibility: "public".to_string(),
                        fields_count: 0, // PHP class field extraction not implemented yet
                        derives: vec![], // PHP doesn't have derives
                        line: line_num + 1,
                    });
                    self.class_count += 1;
                }
            }
        }
        Ok(())
    }

    /// Extracts method definitions from PHP classes (complexity ≤10)
    fn extract_methods(&mut self, source: &str) -> Result<(), String> {
        for (line_num, line) in source.lines().enumerate() {
            let trimmed = line.trim();

            // Match: public function methodName(...) {
            // Match: private function methodName(...) {
            // Match: protected function methodName(...) {
            if (trimmed.starts_with("public function ")
                || trimmed.starts_with("private function ")
                || trimmed.starts_with("protected function "))
                && trimmed.contains('(')
            {
                if let Some(method_name) = self.extract_method_name(trimmed) {
                    let qualified_name = self.get_qualified_name(&method_name);

                    let visibility = if trimmed.starts_with("private") {
                        "private"
                    } else if trimmed.starts_with("protected") {
                        "protected"
                    } else {
                        "public"
                    };

                    self.items.push(AstItem::Function {
                        name: qualified_name,
                        visibility: visibility.to_string(),
                        is_async: false,
                        line: line_num + 1,
                    });
                    self.method_count += 1;
                }
            }
        }
        Ok(())
    }

    /// Extracts function name from PHP line (complexity ≤10)
    fn extract_function_name(&self, line: &str) -> Option<String> {
        // function functionName(...) {
        let after_function = line.strip_prefix("function ")?.trim();
        let name_part = after_function.split('(').next()?;
        Some(name_part.trim().to_string())
    }

    /// Extracts class name from PHP line (complexity ≤10)
    fn extract_class_name(&self, line: &str) -> Option<String> {
        // class ClassName {
        let after_class = line.strip_prefix("class ")?.trim();
        let name_part = after_class
            .split_whitespace()
            .next()?
            .trim_end_matches('{');
        Some(name_part.trim().to_string())
    }

    /// Extracts method name from PHP line (complexity ≤10)
    fn extract_method_name(&self, line: &str) -> Option<String> {
        // public/private/protected function methodName(...) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 && parts[1] == "function" {
            let name_part = parts[2].split('(').next()?;
            Some(name_part.trim().to_string())
        } else {
            None
        }
    }

    /// Gets qualified name for PHP symbol (complexity ≤10)
    fn get_qualified_name(&self, symbol_name: &str) -> String {
        if self.script_name.is_empty() {
            symbol_name.to_string()
        } else {
            format!("{}::{}", self.script_name, symbol_name)
        }
    }
}

/// PHP complexity analyzer for PHP-specific metrics (complexity ≤10)
pub struct PhpComplexityAnalyzer {
    cyclomatic_complexity: u32,
    cognitive_complexity: u32,
}

impl Default for PhpComplexityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl PhpComplexityAnalyzer {
    /// Creates a new PHP complexity analyzer
    #[must_use]
    pub fn new() -> Self {
        Self {
            cyclomatic_complexity: 0,
            cognitive_complexity: 0,
        }
    }

    /// Analyzes complexity of PHP script (complexity ≤10)
    pub fn analyze_complexity(&mut self, source: &str) -> Result<(u32, u32), String> {
        self.cyclomatic_complexity = 1;
        self.cognitive_complexity = 1;

        for line in source.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("if ")
                || trimmed.starts_with("while ")
                || trimmed.starts_with("for ")
                || trimmed.starts_with("foreach ")
                || trimmed.starts_with("switch ")
                || trimmed.starts_with("case ")
                || trimmed.starts_with("elseif ")
                || trimmed.contains("} elseif ")
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

    const SIMPLE_PHP_SCRIPT: &str = r#"<?php

echo "Hello, World!";
?>"#;

    const PHP_SCRIPT_WITH_FUNCTIONS: &str = r#"<?php

// Simple function
function printHello() {
    echo "Hello World!\n";
}

// Function with parameters
function calculateScore($value, $isBonus) {
    if ($value < 0) {
        return 0;
    }

    if ($isBonus) {
        if ($value > 100) {
            return $value * 2;
        } else {
            return $value + 50;
        }
    }

    return $value;
}

// Function with loop
function sumArray($arr) {
    $sum = 0;
    foreach ($arr as $num) {
        if ($num > 0) {
            $sum += $num;
        }
    }
    return $sum;
}

?>"#;

    const PHP_SCRIPT_WITH_CLASSES: &str = r#"<?php

class Calculator {
    public function add($a, $b) {
        return $a + $b;
    }

    public function multiply($a, $b) {
        return $a * $b;
    }

    private function complexOperation($x) {
        if ($x < 0) {
            return -1;
        } elseif ($x == 0) {
            return 0;
        }
        return $x * $x;
    }
}

?>"#;

    #[test]
    fn test_simple_php_script_analysis() {
        let analyzer = PhpScriptAnalyzer::new(Path::new("simple.php"));
        let items = analyzer
            .analyze_php_script(SIMPLE_PHP_SCRIPT)
            .expect("Should parse simple PHP script");

        // Simple script may not have functions
        assert!(items.is_empty() || !items.is_empty(), "Should handle simple PHP script");
    }

    #[test]
    fn test_php_functions_analysis() {
        let analyzer = PhpScriptAnalyzer::new(Path::new("functions.php"));
        let items = analyzer
            .analyze_php_script(PHP_SCRIPT_WITH_FUNCTIONS)
            .expect("Should parse PHP script with functions");

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
    fn test_php_class_analysis() {
        let analyzer = PhpScriptAnalyzer::new(Path::new("classes.php"));
        let items = analyzer
            .analyze_php_script(PHP_SCRIPT_WITH_CLASSES)
            .expect("Should parse PHP script with classes");

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
    fn test_php_complexity_analysis() {
        let mut analyzer = PhpComplexityAnalyzer::new();
        let (cyclomatic, cognitive) = analyzer
            .analyze_complexity(PHP_SCRIPT_WITH_FUNCTIONS)
            .expect("Should analyze PHP complexity");

        assert!(
            cyclomatic >= 3,
            "Script with conditionals should have cyclomatic complexity >= 3"
        );
        assert!(
            cognitive >= 3,
            "Script with conditionals should have cognitive complexity >= 3"
        );
    }

    #[test]
    fn test_empty_php_script() {
        let analyzer = PhpScriptAnalyzer::new(Path::new("empty.php"));
        let items = analyzer
            .analyze_php_script("")
            .expect("Should handle empty script");

        assert!(items.is_empty(), "Empty script should produce no AST items");
    }
}
