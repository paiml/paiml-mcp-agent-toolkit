//! Function-level complexity analyzer for TDG --explain mode (Issue #78)
//!
//! Extracts function-level metrics from Rust source files using tree-sitter.
//!
//! # Usage
//!
//! ```no_run
//! use pmat::tdg::function_analyzer::FunctionAnalyzer;
//! use std::path::Path;
//!
//! let mut analyzer = FunctionAnalyzer::new().unwrap();
//! let functions = analyzer.analyze_file(Path::new("src/lib.rs")).unwrap();
//!
//! for func in functions {
//!     println!("{}: cyclomatic={}, line={}", func.name, func.cyclomatic, func.line_number);
//! }
//! ```

use anyhow::{Context, Result};
use std::path::Path;
use tree_sitter::{Node, Parser};

use super::explain::{ComplexitySeverity, FunctionComplexity};

/// Function-level complexity analyzer
///
/// Uses tree-sitter to parse Rust source files and extract
/// function-level complexity metrics.
pub struct FunctionAnalyzer {
    parser: Parser,
}

impl FunctionAnalyzer {
    /// Create a new FunctionAnalyzer
    ///
    /// # Errors
    ///
    /// Returns error if tree-sitter Rust parser cannot be initialized
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .context("Failed to set Rust language for parser")?;

        Ok(Self { parser })
    }

    /// Analyze a Rust source file and extract function complexity
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to Rust source file
    ///
    /// # Returns
    ///
    /// Vector of `FunctionComplexity` sorted by TDG impact (descending)
    ///
    /// # Errors
    ///
    /// Returns error if file cannot be read or parsed
    pub fn analyze_file(&mut self, file_path: &Path) -> Result<Vec<FunctionComplexity>> {
        debug_assert!(
            file_path.exists(),
            "file_path must exist: {}",
            file_path.display()
        );
        // Read source file
        let source_code = std::fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

        self.analyze_source(&source_code)
    }

    /// Analyze Rust source code and extract function complexity
    ///
    /// # Arguments
    ///
    /// * `source_code` - Rust source code as string
    ///
    /// # Returns
    ///
    /// Vector of `FunctionComplexity` sorted by TDG impact (descending)
    pub fn analyze_source(&mut self, source_code: &str) -> Result<Vec<FunctionComplexity>> {
        // Parse source code
        let tree = self
            .parser
            .parse(source_code, None)
            .context("Failed to parse Rust source code")?;

        let root_node = tree.root_node();

        // Extract all functions by processing them inline
        let mut functions = Vec::new();
        self.walk_tree(root_node, &mut |node| {
            // Rust function items
            if node.kind() == "function_item" {
                if let Some(func) = self.analyze_function(node, source_code) {
                    functions.push(func);
                }
            }
        });

        // Sort by TDG impact (descending)
        functions.sort_by(|a, b| {
            b.tdg_impact
                .partial_cmp(&a.tdg_impact)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(functions)
    }

    /// Walk tree and apply visitor function to each node
    fn walk_tree<F>(&self, node: Node, visitor: &mut F)
    where
        F: FnMut(Node),
    {
        visitor(node);

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.walk_tree(child, visitor);
        }
    }

    /// Analyze a single function node
    fn analyze_function(&self, node: Node, source_code: &str) -> Option<FunctionComplexity> {
        // Extract function name
        let name = self.extract_function_name(node, source_code)?;

        // Get line number (1-indexed)
        let line_number = node.start_position().row + 1;

        // Calculate cyclomatic complexity
        let cyclomatic = self.calculate_cyclomatic_complexity(node);

        // Cognitive complexity approximation: cyclomatic + nesting penalty
        // Full Sonar cognitive complexity would require tracking nesting depth
        // and incrementing for breaks in linear flow (see SonarSource rule).
        // Current approximation is acceptable for TDG scoring.
        let cognitive = cyclomatic;

        // Estimate TDG impact
        let tdg_impact = self.estimate_tdg_impact(cyclomatic, cognitive);

        // Classify severity
        let severity = ComplexitySeverity::from_cyclomatic(cyclomatic);

        Some(FunctionComplexity {
            name,
            line_number,
            cyclomatic,
            cognitive,
            tdg_impact,
            severity,
        })
    }

    /// Extract function name from function_item node
    fn extract_function_name(&self, node: Node, source_code: &str) -> Option<String> {
        // function_item has a "name" field with identifier
        node.child_by_field_name("name").and_then(|name_node| {
            let start = name_node.start_byte();
            let end = name_node.end_byte();
            source_code.get(start..end).map(|s| s.to_string())
        })
    }

    /// Calculate cyclomatic complexity for a function
    ///
    /// McCabe's Cyclomatic Complexity: M = E - N + 2P
    /// Simplified: Count decision points + 1
    ///
    /// Decision points:
    /// - if/else
    /// - match arms
    /// - while/for loops
    /// - && and || operators
    /// - ? operator (Result unwrap)
    fn calculate_cyclomatic_complexity(&self, node: Node) -> u32 {
        let mut complexity = 1; // Base complexity

        self.walk_tree(node, &mut |n| {
            match n.kind() {
                // Conditionals
                "if_expression" => complexity += 1,
                "else" => {} // Don't count else separately (already counted with if)

                // Match expression (count each arm)
                "match_arm" => complexity += 1,

                // Loops
                "while_expression" | "for_expression" | "loop_expression" => complexity += 1,

                // Boolean operators in conditions
                "||" | "&&" => complexity += 1,

                // Question mark operator (Result/Option unwrap)
                "?" => complexity += 1,

                _ => {}
            }
        });

        complexity
    }

    /// Estimate TDG impact based on complexity metrics
    ///
    /// TDG impact formula:
    /// - Cyclomatic complexity contributes 60%
    /// - Cognitive complexity contributes 40%
    /// - Scaled to 0.0-5.0 range
    ///
    /// Impact levels:
    /// - 0.0-1.0: Low impact (simple functions)
    /// - 1.0-2.5: Medium impact
    /// - 2.5-4.0: High impact
    /// - 4.0-5.0: Critical impact
    fn estimate_tdg_impact(&self, cyclomatic: u32, cognitive: u32) -> f64 {
        let cyclomatic_factor = (cyclomatic as f64) / 10.0;
        let cognitive_factor = (cognitive as f64) / 15.0;

        (cyclomatic_factor * 0.6 + cognitive_factor * 0.4).min(5.0)
    }
}

impl Default for FunctionAnalyzer {
    fn default() -> Self {
        Self::new().expect("Failed to create FunctionAnalyzer")
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_analyzer_default() {
        let mut analyzer = FunctionAnalyzer::default();
        // Default should successfully create an analyzer
        let functions = analyzer.analyze_source("fn foo() {}").unwrap();
        assert_eq!(functions.len(), 1);
    }

    #[test]
    fn test_analyze_simple_function() {
        let source = r#"
            fn simple_function() -> i32 {
                return 42;
            }
        "#;

        let mut analyzer = FunctionAnalyzer::new().unwrap();
        let functions = analyzer.analyze_source(source).unwrap();

        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name, "simple_function");
        assert_eq!(functions[0].cyclomatic, 1); // No branches
        assert_eq!(functions[0].severity, ComplexitySeverity::Low);
    }

    #[test]
    fn test_analyze_if_else_function() {
        let source = r#"
            fn conditional_function(x: i32) -> i32 {
                if x > 10 {
                    if x > 20 {
                        return x * 2;
                    } else {
                        return x + 5;
                    }
                } else {
                    return x - 3;
                }
            }
        "#;

        let mut analyzer = FunctionAnalyzer::new().unwrap();
        let functions = analyzer.analyze_source(source).unwrap();

        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name, "conditional_function");
        assert_eq!(
            functions[0].cyclomatic, 3,
            "Should have 3 complexity (2 if statements + base)"
        );
        // Complexity 3 is Low per McCabe standards (Low: 0-5)
        assert_eq!(functions[0].severity, ComplexitySeverity::Low);
    }

    #[test]
    fn test_analyze_match_expression() {
        let source = r#"
            fn match_function(value: i32) -> String {
                match value {
                    0 => "zero".to_string(),
                    1 => "one".to_string(),
                    2 => "two".to_string(),
                    3 => "three".to_string(),
                    4 => "four".to_string(),
                    5 => "five".to_string(),
                    _ => "many".to_string(),
                }
            }
        "#;

        let mut analyzer = FunctionAnalyzer::new().unwrap();
        let functions = analyzer.analyze_source(source).unwrap();

        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name, "match_function");
        // Match with 7 arms = 7 decision points + 1 base = 8
        assert_eq!(
            functions[0].cyclomatic, 8,
            "Should count match arms (7 arms + 1 base)"
        );
        // Complexity 8 is Medium per McCabe standards (Medium: 6-10)
        assert_eq!(functions[0].severity, ComplexitySeverity::Medium);
    }

    #[test]
    fn test_analyze_multiple_functions() {
        let source = r#"
            fn simple() -> i32 {
                42
            }

            fn complex(x: i32) -> i32 {
                if x > 0 {
                    x * 2
                } else {
                    x - 1
                }
            }
        "#;

        let mut analyzer = FunctionAnalyzer::new().unwrap();
        let functions = analyzer.analyze_source(source).unwrap();

        assert_eq!(functions.len(), 2);

        // Should be sorted by TDG impact (complex first)
        assert_eq!(functions[0].name, "complex");
        assert!(functions[0].cyclomatic > 1);

        assert_eq!(functions[1].name, "simple");
        assert_eq!(functions[1].cyclomatic, 1);
    }

    #[test]
    fn test_tdg_impact_calculation() {
        let analyzer = FunctionAnalyzer::new().unwrap();

        // Simple function (complexity 1)
        let impact_simple = analyzer.estimate_tdg_impact(1, 1);
        assert!(
            impact_simple < 0.5,
            "Simple function should have low impact"
        );

        // Medium complexity (10)
        let impact_medium = analyzer.estimate_tdg_impact(10, 10);
        assert!(
            impact_medium >= 0.5 && impact_medium <= 3.0,
            "Medium complexity should have moderate impact"
        );

        // High complexity (25/30)
        // Formula: (25/10)*0.6 + (30/15)*0.4 = 2.5*0.6 + 2.0*0.4 = 1.5 + 0.8 = 2.3
        let impact_high = analyzer.estimate_tdg_impact(25, 30);
        assert!(
            impact_high >= 2.0 && impact_high <= 2.5,
            "High complexity should have impact ~2.3"
        );
    }

    #[test]
    fn test_line_number_extraction() {
        let source = r#"
fn first() {}

fn second() {}

fn third() {}
        "#;

        let mut analyzer = FunctionAnalyzer::new().unwrap();
        let functions = analyzer.analyze_source(source).unwrap();

        assert_eq!(functions.len(), 3);

        // Check that line numbers are extracted correctly
        // Note: Order may vary due to sorting by TDG impact, so check by name
        let first = functions.iter().find(|f| f.name == "first").unwrap();
        let second = functions.iter().find(|f| f.name == "second").unwrap();
        let third = functions.iter().find(|f| f.name == "third").unwrap();

        assert_eq!(first.line_number, 2);
        assert_eq!(second.line_number, 4);
        assert_eq!(third.line_number, 6);
    }
}
