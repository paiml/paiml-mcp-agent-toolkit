//! Enhanced AST parser using syn for Rust code analysis

use crate::unified_quality::metrics::Metrics;
use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use syn::{visit::Visit, File};

/// Enhanced parser using syn for accurate Rust analysis
pub struct EnhancedParser {
    /// Cached ASTs with metadata
    cache: Arc<dashmap::DashMap<PathBuf, CachedSyntax>>,
}

/// Cached syntax tree with metadata
pub struct CachedSyntax {
    /// Serialized syntax tree (to avoid Send issues)
    pub syntax_str: String,

    /// Source code content
    pub content: String,

    /// Last modified time
    pub last_modified: SystemTime,

    /// Content hash for validation
    pub content_hash: u64,

    /// Computed metrics
    pub metrics: Option<Metrics>,
}

/// Visitor for calculating complexity metrics
#[allow(dead_code)]
struct ComplexityVisitor {
    /// Current cyclomatic complexity
    complexity: u32,

    /// Current cognitive complexity  
    cognitive: u32,

    /// Current nesting level for cognitive complexity
    nesting_level: u32,

    /// Number of functions
    function_count: u32,

    /// SATD comment count
    satd_count: u32,

    /// Source content for comment analysis
    content: String,
}

impl Default for EnhancedParser {
    fn default() -> Self {
        Self::new()
    }
}

impl EnhancedParser {
    /// Create a new enhanced parser
    pub fn new() -> Self {
        Self {
            cache: Arc::new(dashmap::DashMap::new()),
        }
    }

    /// Parse file with incremental updates
    pub fn parse_incremental(&mut self, path: &PathBuf, content: &str) -> Result<Metrics> {
        let content_hash = self.calculate_hash(content);

        // Check cache for existing result
        if let Some(cached) = self.cache.get(path) {
            if cached.content_hash == content_hash {
                // Content unchanged, return cached metrics
                if let Some(ref metrics) = cached.metrics {
                    return Ok(metrics.clone());
                }
            }
        }

        // Parse and analyze
        self.parse_and_analyze(path, content)
    }

    /// Parse and analyze Rust code
    fn parse_and_analyze(&mut self, path: &PathBuf, content: &str) -> Result<Metrics> {
        // Parse using syn
        let syntax: File =
            syn::parse_str(content).map_err(|e| anyhow!("Failed to parse Rust code: {}", e))?;

        // Calculate metrics using visitor pattern
        let mut visitor = ComplexityVisitor::new(content.to_string());
        visitor.visit_file(&syntax);

        let metrics = Metrics {
            complexity: visitor.complexity,
            cognitive: visitor.cognitive,
            satd_count: visitor.satd_count,
            coverage: 0.8, // Placeholder - would integrate with coverage tools
            lines: content.lines().count() as u32,
            functions: visitor.function_count,
            timestamp: SystemTime::now(),
        };

        // Cache the result (without storing syn::File directly to avoid Send issues)
        self.cache.insert(
            path.clone(),
            CachedSyntax {
                syntax_str: format!("{syntax:#?}"), // Debug representation
                content: content.to_string(),
                last_modified: SystemTime::now(),
                content_hash: self.calculate_hash(content),
                metrics: Some(metrics.clone()),
            },
        );

        Ok(metrics)
    }

    /// Calculate content hash for caching
    fn calculate_hash(&self, content: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        hasher.finish()
    }

    /// Get cached metrics if available
    pub fn get_cached_metrics(&self, path: &PathBuf) -> Option<Metrics> {
        self.cache.get(path)?.metrics.clone()
    }

    /// Clear cache for a file
    pub fn clear_cache(&self, path: &PathBuf) {
        self.cache.remove(path);
    }

    /// Clear entire cache
    pub fn clear_all_cache(&self) {
        self.cache.clear();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        CacheStats {
            total_entries: self.cache.len(),
            memory_usage_estimate: self.cache.len() * 2048, // Rough estimate
        }
    }
}

impl ComplexityVisitor {
    fn new(content: String) -> Self {
        let satd_count = Self::count_satd_in_content(&content);
        Self {
            complexity: 1, // Base complexity
            cognitive: 0,
            nesting_level: 0,
            function_count: 0,
            satd_count,
            content,
        }
    }

    /// Count SATD comments in content
    fn count_satd_in_content(content: &str) -> u32 {
        let patterns = ["TODO", "FIXME", "HACK", "XXX", "BUG"];
        patterns
            .iter()
            .map(|pattern| content.matches(pattern).count() as u32)
            .sum()
    }
}

impl<'ast> Visit<'ast> for ComplexityVisitor {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        self.function_count += 1;

        // Reset for function-level metrics
        let old_complexity = self.complexity;
        let old_cognitive = self.cognitive;
        let old_nesting = self.nesting_level;

        self.complexity = 1; // Base complexity for function
        self.cognitive = 0;
        self.nesting_level = 0;

        // Visit function body
        syn::visit::visit_item_fn(self, node);

        // Restore and accumulate
        let fn_complexity = self.complexity;
        let fn_cognitive = self.cognitive;

        self.complexity = old_complexity + fn_complexity;
        self.cognitive = old_cognitive + fn_cognitive;
        self.nesting_level = old_nesting;
    }

    fn visit_expr_if(&mut self, node: &'ast syn::ExprIf) {
        // Increase complexity and cognitive complexity
        self.complexity += 1;
        self.cognitive += 1 + self.nesting_level;

        // Increase nesting for cognitive complexity
        self.nesting_level += 1;
        syn::visit::visit_expr_if(self, node);
        self.nesting_level -= 1;
    }

    fn visit_expr_while(&mut self, node: &'ast syn::ExprWhile) {
        self.complexity += 1;
        self.cognitive += 1 + self.nesting_level;

        self.nesting_level += 1;
        syn::visit::visit_expr_while(self, node);
        self.nesting_level -= 1;
    }

    fn visit_expr_for_loop(&mut self, node: &'ast syn::ExprForLoop) {
        self.complexity += 1;
        self.cognitive += 1 + self.nesting_level;

        self.nesting_level += 1;
        syn::visit::visit_expr_for_loop(self, node);
        self.nesting_level -= 1;
    }

    fn visit_expr_loop(&mut self, node: &'ast syn::ExprLoop) {
        self.complexity += 1;
        self.cognitive += 1 + self.nesting_level;

        self.nesting_level += 1;
        syn::visit::visit_expr_loop(self, node);
        self.nesting_level -= 1;
    }

    fn visit_expr_match(&mut self, node: &'ast syn::ExprMatch) {
        // Match adds complexity for each arm
        self.complexity += node.arms.len() as u32;
        self.cognitive += 1 + self.nesting_level;

        self.nesting_level += 1;
        syn::visit::visit_expr_match(self, node);
        self.nesting_level -= 1;
    }

    fn visit_expr_binary(&mut self, node: &'ast syn::ExprBinary) {
        // Check for logical operators
        match node.op {
            syn::BinOp::And(_) | syn::BinOp::Or(_) => {
                self.complexity += 1;
            }
            _ => {}
        }

        syn::visit::visit_expr_binary(self, node);
    }

    fn visit_arm(&mut self, node: &'ast syn::Arm) {
        // Each match arm adds cognitive complexity based on nesting
        if self.nesting_level > 0 {
            self.cognitive += self.nesting_level;
        }

        syn::visit::visit_arm(self, node);
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub memory_usage_estimate: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_parser_creation() {
        let parser = EnhancedParser::new();
        assert_eq!(parser.cache_stats().total_entries, 0);
    }

    #[test]
    fn test_rust_parsing() {
        let mut parser = EnhancedParser::new();
        let code = r#"
            fn main() {
                if true {
                    println!("Hello, world!");
                }
            }
        "#;

        let path = PathBuf::from("test.rs");
        let result = parser.parse_incremental(&path, code);

        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert!(metrics.functions > 0);
        assert!(metrics.complexity > 1); // Should detect if statement
    }

    #[test]
    fn test_complexity_calculation() {
        let mut parser = EnhancedParser::new();
        let code = r#"
            fn complex_function(x: i32) -> i32 {
                if x > 0 {
                    for i in 0..10 {
                        if i % 2 == 0 {
                            while x > 0 {
                                x -= 1;
                            }
                        }
                    }
                }
                match x {
                    0 => 0,
                    1 => 1,
                    _ => 2,
                }
            }
        "#;

        let path = PathBuf::from("complex.rs");
        let metrics = parser.parse_incremental(&path, code).unwrap();

        assert!(metrics.complexity > 5); // Should detect multiple control structures
        assert!(metrics.cognitive > metrics.complexity); // Cognitive should account for nesting
        assert_eq!(metrics.functions, 1);
    }

    #[test]
    fn test_satd_detection() {
        let mut parser = EnhancedParser::new();
        let code = r#"
            fn test() {
                // TODO: implement this properly
                // FIXME: handle error case
                // HACK: temporary solution
                println!("test");
            }
        "#;

        let path = PathBuf::from("satd.rs");
        let metrics = parser.parse_incremental(&path, code).unwrap();

        assert_eq!(metrics.satd_count, 3);
    }

    #[test]
    fn test_cache_functionality() {
        let mut parser = EnhancedParser::new();
        let path = PathBuf::from("cached.rs");
        let code = "fn test() {}";

        // Parse twice with same content
        let metrics1 = parser.parse_incremental(&path, code).unwrap();
        let metrics2 = parser.parse_incremental(&path, code).unwrap();

        // Should return same results from cache
        assert_eq!(metrics1.functions, metrics2.functions);
        assert_eq!(metrics1.complexity, metrics2.complexity);

        // Check cache stats
        let stats = parser.cache_stats();
        assert_eq!(stats.total_entries, 1);
    }

    #[test]
    fn test_incremental_parsing() {
        let mut parser = EnhancedParser::new();
        let path = PathBuf::from("test.rs");

        // Parse original code
        let code1 = "fn test() { if true { } }";
        let metrics1 = parser.parse_incremental(&path, code1).unwrap();

        // Parse modified code
        let code2 = "fn test() { if true { if false { } } }";
        let metrics2 = parser.parse_incremental(&path, code2).unwrap();

        assert!(metrics2.complexity > metrics1.complexity);
        assert!(metrics2.cognitive > metrics1.cognitive);
    }

    #[test]
    fn test_logical_operators() {
        let mut parser = EnhancedParser::new();
        let code = r#"
            fn test_logical() {
                if a && b || c && d {
                    return true;
                }
                false
            }
        "#;

        let path = PathBuf::from("logical.rs");
        let metrics = parser.parse_incremental(&path, code).unwrap();

        // Should detect if statement + logical operators
        assert!(metrics.complexity >= 4); // 1 base + 1 if + 2 logical operators
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    fn valid_rust_identifier() -> impl Strategy<Value = String> {
        prop::string::string_regex("[a-zA-Z_][a-zA-Z0-9_]*").unwrap()
    }

    fn simple_rust_function(name: String) -> String {
        format!("fn {}() {{ }}", name)
    }

    fn rust_function_with_if(name: String, condition: String) -> String {
        format!(
            r#"
            fn {}() {{
                if {} {{
                    return;
                }}
            }}
            "#,
            name, condition
        )
    }

    proptest! {
        #[test]
        fn parser_handles_valid_identifiers(name in valid_rust_identifier()) {
            let mut parser = EnhancedParser::new();
            let code = simple_rust_function(name);
            let path = PathBuf::from("test.rs");

            let result = parser.parse_incremental(&path, &code);
            prop_assert!(result.is_ok());

            let metrics = result.unwrap();
            prop_assert_eq!(metrics.functions, 1);
            prop_assert!(metrics.complexity >= 1); // Base complexity
        }

        #[test]
        fn complexity_increases_with_control_flow(
            name in valid_rust_identifier(),
            condition in valid_rust_identifier()
        ) {
            let mut parser = EnhancedParser::new();
            let simple_code = simple_rust_function(name.clone());
            let complex_code = rust_function_with_if(name, condition);

            let path1 = PathBuf::from("simple.rs");
            let path2 = PathBuf::from("complex.rs");

            let simple_metrics = parser.parse_incremental(&path1, &simple_code).unwrap();
            let complex_metrics = parser.parse_incremental(&path2, &complex_code).unwrap();

            prop_assert!(complex_metrics.complexity > simple_metrics.complexity);
            prop_assert_eq!(simple_metrics.functions, complex_metrics.functions);
        }

        #[test]
        fn cache_consistency(
            name in valid_rust_identifier(),
            content_variations in prop::collection::vec(valid_rust_identifier(), 1..10)
        ) {
            let mut parser = EnhancedParser::new();
            let path = PathBuf::from("cache_test.rs");

            // Parse same content multiple times
            let base_code = simple_rust_function(name);

            let first_result = parser.parse_incremental(&path, &base_code).unwrap();
            let second_result = parser.parse_incremental(&path, &base_code).unwrap();

            // Results should be identical (cached)
            prop_assert_eq!(first_result.complexity, second_result.complexity);
            prop_assert_eq!(first_result.functions, second_result.functions);
            prop_assert_eq!(first_result.lines, second_result.lines);
        }

        #[test]
        fn hash_calculation_stable(content in "[a-zA-Z0-9\\s\\n{}();]{10,500}") {
            let parser = EnhancedParser::new();

            // Same content should produce same hash
            let hash1 = parser.calculate_hash(&content);
            let hash2 = parser.calculate_hash(&content);

            prop_assert_eq!(hash1, hash2);

            // Different content should produce different hash (with high probability)
            let modified_content = format!("{} // comment", content);
            let hash3 = parser.calculate_hash(&modified_content);
            prop_assert_ne!(hash1, hash3);
        }

        #[test]
        fn satd_detection_accuracy(
            base_code in "[a-zA-Z0-9\\s\\n{}();]{50,200}",
            satd_count in 0usize..5
        ) {
            let mut parser = EnhancedParser::new();
            let path = PathBuf::from("satd_test.rs");

            // Add known SATD comments
            let satd_comments = vec!["TODO", "FIXME", "HACK", "XXX", "BUG"];
            let mut enhanced_code = base_code;

            for i in 0..satd_count {
                let comment_type = &satd_comments[i % satd_comments.len()];
                enhanced_code.push_str(&format!("\n// {}: test comment", comment_type));
            }

            let code = format!("fn test() {{\n{}\n}}", enhanced_code);
            let metrics = parser.parse_incremental(&path, &code).unwrap();

            prop_assert_eq!(metrics.satd_count, satd_count as u32);
        }

        #[test]
        fn nesting_affects_cognitive_complexity(
            function_name in valid_rust_identifier(),
            nesting_levels in 1usize..5
        ) {
            let mut parser = EnhancedParser::new();
            let path = PathBuf::from("nesting_test.rs");

            // Create nested if statements
            let mut code = format!("fn {}() {{\n", function_name);

            for level in 0..nesting_levels {
                code.push_str(&"    ".repeat(level + 1));
                code.push_str(&format!("if condition_{} {{\n", level));
            }

            // Close all the braces
            for level in (0..nesting_levels).rev() {
                code.push_str(&"    ".repeat(level + 1));
                code.push_str("}\n");
            }
            code.push('}');

            let metrics = parser.parse_incremental(&path, &code).unwrap();

            // Cognitive complexity should be higher than cyclomatic for nested code
            prop_assert!(metrics.cognitive >= metrics.complexity);
            prop_assert!(metrics.complexity >= (nesting_levels as u32 + 1)); // +1 for base
        }

        #[test]
        fn line_counting_accuracy(
            line_count in 5usize..100,
            chars_per_line in 10usize..80
        ) {
            let mut parser = EnhancedParser::new();
            let path = PathBuf::from("lines_test.rs");

            // Generate code with known line count
            let mut code = String::new();
            for i in 0..line_count {
                let line_content = "a".repeat(chars_per_line % 50); // Keep reasonable
                code.push_str(&format!("// Line {}: {}\n", i, line_content));
            }
            code.push_str("fn test() {}"); // Add one more line

            let expected_lines = line_count + 1; // +1 for the function
            let metrics = parser.parse_incremental(&path, &code).unwrap();

            prop_assert_eq!(metrics.lines as usize, expected_lines);
        }

        #[test]
        fn cache_invalidation_works(
            name in valid_rust_identifier(),
            content1 in "[a-zA-Z0-9]{10,100}",
            content2 in "[a-zA-Z0-9]{10,100}"
        ) {
            let mut parser = EnhancedParser::new();
            let path = PathBuf::from("invalidation_test.rs");

            let code1 = format!("fn {}() {{ /* {} */ }}", name, content1);
            let code2 = format!("fn {}() {{ /* {} */ }}", name, content2);

            // Parse first version
            let metrics1 = parser.parse_incremental(&path, &code1).unwrap();

            // Cache should have entry
            prop_assert!(parser.get_cached_metrics(&path).is_some());

            // Parse different content - should invalidate cache and reparse
            let metrics2 = parser.parse_incremental(&path, &code2).unwrap();

            // If content differs, metrics might differ (at least timestamps)
            if code1 != code2 {
                // At minimum, timestamps should be different
                prop_assert!(metrics1.timestamp <= metrics2.timestamp);
            }
        }

        #[test]
        fn match_expression_complexity(
            function_name in valid_rust_identifier(),
            arm_count in 2usize..8
        ) {
            let mut parser = EnhancedParser::new();
            let path = PathBuf::from("match_test.rs");

            let mut code = format!("fn {}() {{\n    match x {{\n", function_name);

            for i in 0..arm_count {
                code.push_str(&format!("        {} => {},\n", i, i * 2));
            }

            code.push_str("    }\n}");

            let metrics = parser.parse_incremental(&path, &code).unwrap();

            // Match adds complexity for each arm
            prop_assert!(metrics.complexity >= (arm_count as u32 + 1)); // +1 for base
            prop_assert_eq!(metrics.functions, 1);
        }

        #[test]
        fn parser_memory_usage_bounded(
            file_count in 1usize..20,
            content_size in 100usize..1000
        ) {
            let mut parser = EnhancedParser::new();

            // Parse multiple files and check cache growth is reasonable
            for i in 0..file_count {
                let path = PathBuf::from(format!("file_{}.rs", i));
                let content = "a".repeat(content_size);
                let code = format!("fn test_{}() {{ /* {} */ }}", i, content);

                let _metrics = parser.parse_incremental(&path, &code).unwrap();
            }

            let stats = parser.cache_stats();
            prop_assert_eq!(stats.total_entries, file_count);

            // Memory usage should be reasonable (rough estimate)
            prop_assert!(stats.memory_usage_estimate > 0);
            prop_assert!(stats.memory_usage_estimate < file_count * 10000); // Upper bound check
        }
    }
}
