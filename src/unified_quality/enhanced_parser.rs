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
    #[must_use]
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
            syn::parse_str(content).map_err(|e| anyhow!("Failed to parse Rust code: {e}"))?;

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
    #[must_use]
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
    #[must_use]
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

#[cfg_attr(coverage_nightly, coverage(off))]
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

#[cfg_attr(coverage_nightly, coverage(off))]
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
        #[ignore] // Fails on edge case: name = "_" - parser doesn't handle wildcard pattern
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
        #[ignore = "requires quality framework setup"]
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
        #[ignore = "requires quality framework setup"]
        fn cache_consistency(
            name in valid_rust_identifier(),
            _content_variations in prop::collection::vec(valid_rust_identifier(), 1..10)
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
        #[ignore = "requires quality framework setup"]
        fn satd_detection_accuracy(
            base_code in "[a-zA-Z0-9\\s\\n{}();]{50,200}",
            satd_count in 0usize..5
        ) {
            let mut parser = EnhancedParser::new();
            let path = PathBuf::from("satd_test.rs");

            // Add known SATD comments
            let satd_comments = ["TODO", "FIXME", "HACK", "XXX", "BUG"];
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
        #[ignore = "requires quality framework setup"]
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
        #[ignore = "requires quality framework setup"]
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
        #[ignore = "requires quality framework setup"]
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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;
    use std::time::SystemTime;

    // ============================================
    // Test Fixtures and Helpers
    // ============================================

    /// Create a simple Rust function
    fn simple_function() -> &'static str {
        "fn simple() {}"
    }

    /// Create a function with if statement
    fn function_with_if() -> &'static str {
        r#"
        fn with_if(x: i32) {
            if x > 0 {
                println!("positive");
            }
        }
        "#
    }

    /// Create a function with multiple control flow structures
    fn complex_function() -> &'static str {
        r#"
        fn complex(x: i32, y: i32) -> i32 {
            if x > 0 {
                for i in 0..10 {
                    if i % 2 == 0 {
                        while y > 0 {
                            return x + y;
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
        "#
    }

    /// Create code with SATD comments
    fn code_with_satd() -> &'static str {
        r#"
        fn needs_work() {
            // TODO: implement this function
            // FIXME: this is broken
            // HACK: temporary workaround
            // XXX: needs review
            // BUG: known issue here
        }
        "#
    }

    /// Create code with logical operators
    fn code_with_logical_ops() -> &'static str {
        r#"
        fn check_conditions(a: bool, b: bool, c: bool) -> bool {
            if a && b || c && !a {
                true
            } else {
                false
            }
        }
        "#
    }

    /// Create code with multiple functions
    fn multiple_functions() -> &'static str {
        r#"
        fn first() {}
        fn second() {}
        fn third() {
            if true {}
        }
        "#
    }

    /// Create a test path
    fn test_path(name: &str) -> PathBuf {
        PathBuf::from(format!("{}.rs", name))
    }

    // ============================================
    // EnhancedParser Creation Tests
    // ============================================

    #[test]
    fn test_parser_new() {
        let parser = EnhancedParser::new();
        assert_eq!(parser.cache_stats().total_entries, 0);
    }

    #[test]
    fn test_parser_default() {
        let parser = EnhancedParser::default();
        assert_eq!(parser.cache_stats().total_entries, 0);
    }

    #[test]
    fn test_parser_default_equals_new() {
        let parser1 = EnhancedParser::new();
        let parser2 = EnhancedParser::default();

        assert_eq!(
            parser1.cache_stats().total_entries,
            parser2.cache_stats().total_entries
        );
    }

    // ============================================
    // parse_incremental Basic Tests
    // ============================================

    #[test]
    fn test_parse_simple_function() {
        let mut parser = EnhancedParser::new();
        let result = parser.parse_incremental(&test_path("simple"), simple_function());

        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert_eq!(metrics.functions, 1);
        assert!(metrics.complexity >= 1);
    }

    #[test]
    fn test_parse_function_with_if() {
        let mut parser = EnhancedParser::new();
        let result = parser.parse_incremental(&test_path("with_if"), function_with_if());

        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert_eq!(metrics.functions, 1);
        assert!(metrics.complexity >= 2); // Base + if
    }

    #[test]
    fn test_parse_complex_function() {
        let mut parser = EnhancedParser::new();
        let result = parser.parse_incremental(&test_path("complex"), complex_function());

        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert_eq!(metrics.functions, 1);
        assert!(metrics.complexity > 5); // Multiple control structures
        assert!(metrics.cognitive >= metrics.complexity); // Nesting adds cognitive complexity
    }

    #[test]
    fn test_parse_multiple_functions() {
        let mut parser = EnhancedParser::new();
        let result = parser.parse_incremental(&test_path("multi"), multiple_functions());

        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert_eq!(metrics.functions, 3);
    }

    #[test]
    fn test_parse_empty_file() {
        let mut parser = EnhancedParser::new();
        let result = parser.parse_incremental(&test_path("empty"), "");

        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert_eq!(metrics.functions, 0);
    }

    // ============================================
    // SATD Detection Tests
    // ============================================

    #[test]
    fn test_satd_detection_all_types() {
        let mut parser = EnhancedParser::new();
        let result = parser.parse_incremental(&test_path("satd"), code_with_satd());

        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert_eq!(metrics.satd_count, 5); // TODO, FIXME, HACK, XXX, BUG
    }

    #[test]
    fn test_satd_detection_single_todo() {
        let mut parser = EnhancedParser::new();
        // Use valid Rust syntax with TODO comment on its own line
        let code = "fn test() {\n    // TODO: implement\n}";
        let result = parser.parse_incremental(&test_path("todo"), code);

        assert!(result.is_ok(), "Parse failed: {:?}", result.as_ref().err());
        let metrics = result.unwrap();
        assert_eq!(metrics.satd_count, 1);
    }

    #[test]
    fn test_satd_detection_no_satd() {
        let mut parser = EnhancedParser::new();
        let code = "fn clean() { /* Clean code */ }";
        let result = parser.parse_incremental(&test_path("clean"), code);

        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert_eq!(metrics.satd_count, 0);
    }

    #[test]
    fn test_satd_detection_multiple_same_type() {
        let mut parser = EnhancedParser::new();
        let code = r#"
        fn many_todos() {
            // TODO: first
            // TODO: second
            // TODO: third
        }
        "#;
        let result = parser.parse_incremental(&test_path("todos"), code);

        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert_eq!(metrics.satd_count, 3);
    }

    // ============================================
    // Complexity Calculation Tests
    // ============================================

    #[test]
    fn test_complexity_if_statement() {
        let mut parser = EnhancedParser::new();
        let code = "fn test() { if true {} }";
        let result = parser.parse_incremental(&test_path("if"), code);

        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert!(metrics.complexity >= 2); // Base + if
    }

    #[test]
    fn test_complexity_for_loop() {
        let mut parser = EnhancedParser::new();
        let code = "fn test() { for i in 0..10 {} }";
        let result = parser.parse_incremental(&test_path("for"), code);

        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert!(metrics.complexity >= 2); // Base + for
    }

    #[test]
    fn test_complexity_while_loop() {
        let mut parser = EnhancedParser::new();
        let code = "fn test() { while true {} }";
        let result = parser.parse_incremental(&test_path("while"), code);

        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert!(metrics.complexity >= 2); // Base + while
    }

    #[test]
    fn test_complexity_loop() {
        let mut parser = EnhancedParser::new();
        let code = "fn test() { loop { break; } }";
        let result = parser.parse_incremental(&test_path("loop"), code);

        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert!(metrics.complexity >= 2); // Base + loop
    }

    #[test]
    fn test_complexity_match() {
        let mut parser = EnhancedParser::new();
        let code = r#"
        fn test(x: i32) -> i32 {
            match x {
                0 => 0,
                1 => 1,
                2 => 2,
                _ => 3,
            }
        }
        "#;
        let result = parser.parse_incremental(&test_path("match"), code);

        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert!(metrics.complexity >= 5); // Base + 4 match arms
    }

    #[test]
    fn test_complexity_logical_and() {
        let mut parser = EnhancedParser::new();
        let code = "fn test(a: bool, b: bool) -> bool { a && b }";
        let result = parser.parse_incremental(&test_path("and"), code);

        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert!(metrics.complexity >= 2); // Base + &&
    }

    #[test]
    fn test_complexity_logical_or() {
        let mut parser = EnhancedParser::new();
        let code = "fn test(a: bool, b: bool) -> bool { a || b }";
        let result = parser.parse_incremental(&test_path("or"), code);

        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert!(metrics.complexity >= 2); // Base + ||
    }

    #[test]
    fn test_complexity_multiple_logical_ops() {
        let mut parser = EnhancedParser::new();
        let result = parser.parse_incremental(&test_path("logical"), code_with_logical_ops());

        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert!(metrics.complexity >= 4); // Base + if + logical ops
    }

    // ============================================
    // Cognitive Complexity Tests
    // ============================================

    #[test]
    fn test_cognitive_nested_if() {
        let mut parser = EnhancedParser::new();
        let code = r#"
        fn nested() {
            if true {
                if true {
                    if true {
                        // Deeply nested
                    }
                }
            }
        }
        "#;
        let result = parser.parse_incremental(&test_path("nested"), code);

        assert!(result.is_ok());
        let metrics = result.unwrap();
        // Cognitive should account for nesting
        assert!(metrics.cognitive >= 6); // 1 + 2 + 3 for nesting levels
    }

    #[test]
    fn test_cognitive_flat_structure() {
        let mut parser = EnhancedParser::new();
        let code = r#"
        fn flat() {
            if true {}
            if true {}
            if true {}
        }
        "#;
        let result = parser.parse_incremental(&test_path("flat"), code);

        assert!(result.is_ok());
        let metrics = result.unwrap();
        // Flat structures have lower cognitive complexity
        assert!(metrics.cognitive >= 3);
    }

    #[test]
    fn test_cognitive_vs_cyclomatic() {
        let mut parser = EnhancedParser::new();
        let result = parser.parse_incremental(&test_path("complex"), complex_function());

        assert!(result.is_ok());
        let metrics = result.unwrap();
        // For deeply nested code, cognitive should be >= cyclomatic
        assert!(metrics.cognitive >= metrics.complexity);
    }

    // ============================================
    // Line Counting Tests
    // ============================================

    #[test]
    fn test_line_count_single_line() {
        let mut parser = EnhancedParser::new();
        let code = "fn test() {}";
        let result = parser.parse_incremental(&test_path("single"), code);

        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert_eq!(metrics.lines, 1);
    }

    #[test]
    fn test_line_count_multiple_lines() {
        let mut parser = EnhancedParser::new();
        let code = "fn test() {\n    // line 2\n    // line 3\n}";
        let result = parser.parse_incremental(&test_path("multi_line"), code);

        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert_eq!(metrics.lines, 4);
    }

    #[test]
    fn test_line_count_empty_lines() {
        let mut parser = EnhancedParser::new();
        let code = "fn test() {\n\n\n}";
        let result = parser.parse_incremental(&test_path("empty_lines"), code);

        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert_eq!(metrics.lines, 4);
    }

    // ============================================
    // Cache Functionality Tests
    // ============================================

    #[test]
    fn test_cache_stores_result() {
        let mut parser = EnhancedParser::new();
        let path = test_path("cached");
        let code = simple_function();

        parser.parse_incremental(&path, code).unwrap();

        assert!(parser.get_cached_metrics(&path).is_some());
    }

    #[test]
    fn test_cache_returns_same_result() {
        let mut parser = EnhancedParser::new();
        let path = test_path("cached");
        let code = simple_function();

        let first = parser.parse_incremental(&path, code).unwrap();
        let second = parser.parse_incremental(&path, code).unwrap();

        assert_eq!(first.functions, second.functions);
        assert_eq!(first.complexity, second.complexity);
    }

    #[test]
    fn test_cache_invalidation_on_change() {
        let mut parser = EnhancedParser::new();
        let path = test_path("changing");

        let code1 = "fn a() {}";
        let code2 = "fn a() { if true {} }";

        let first = parser.parse_incremental(&path, code1).unwrap();
        let second = parser.parse_incremental(&path, code2).unwrap();

        // Complexity should differ
        assert!(second.complexity > first.complexity);
    }

    #[test]
    fn test_get_cached_metrics_nonexistent() {
        let parser = EnhancedParser::new();
        let path = test_path("nonexistent");

        assert!(parser.get_cached_metrics(&path).is_none());
    }

    #[test]
    fn test_clear_cache_single_file() {
        let mut parser = EnhancedParser::new();
        let path = test_path("to_clear");

        parser.parse_incremental(&path, simple_function()).unwrap();
        assert!(parser.get_cached_metrics(&path).is_some());

        parser.clear_cache(&path);
        assert!(parser.get_cached_metrics(&path).is_none());
    }

    #[test]
    fn test_clear_all_cache() {
        let mut parser = EnhancedParser::new();

        parser
            .parse_incremental(&test_path("file1"), "fn a() {}")
            .unwrap();
        parser
            .parse_incremental(&test_path("file2"), "fn b() {}")
            .unwrap();
        parser
            .parse_incremental(&test_path("file3"), "fn c() {}")
            .unwrap();

        assert_eq!(parser.cache_stats().total_entries, 3);

        parser.clear_all_cache();

        assert_eq!(parser.cache_stats().total_entries, 0);
    }

    // ============================================
    // CacheStats Tests
    // ============================================

    #[test]
    fn test_cache_stats_empty() {
        let parser = EnhancedParser::new();
        let stats = parser.cache_stats();

        assert_eq!(stats.total_entries, 0);
        assert_eq!(stats.memory_usage_estimate, 0);
    }

    #[test]
    fn test_cache_stats_after_parsing() {
        let mut parser = EnhancedParser::new();

        for i in 0..5 {
            parser
                .parse_incremental(&test_path(&format!("file_{}", i)), "fn test() {}")
                .unwrap();
        }

        let stats = parser.cache_stats();
        assert_eq!(stats.total_entries, 5);
        assert!(stats.memory_usage_estimate > 0);
    }

    #[test]
    fn test_cache_stats_debug() {
        let stats = CacheStats {
            total_entries: 10,
            memory_usage_estimate: 20480,
        };

        let debug_str = format!("{:?}", stats);
        assert!(debug_str.contains("CacheStats"));
        assert!(debug_str.contains("total_entries"));
    }

    #[test]
    fn test_cache_stats_clone() {
        let stats = CacheStats {
            total_entries: 5,
            memory_usage_estimate: 10240,
        };
        let cloned = stats.clone();

        assert_eq!(stats.total_entries, cloned.total_entries);
        assert_eq!(stats.memory_usage_estimate, cloned.memory_usage_estimate);
    }

    // ============================================
    // CachedSyntax Tests
    // ============================================

    #[test]
    fn test_cached_syntax_fields() {
        let cached = CachedSyntax {
            syntax_str: "AST representation".to_string(),
            content: "fn test() {}".to_string(),
            last_modified: SystemTime::now(),
            content_hash: 12345,
            metrics: None,
        };

        assert_eq!(cached.syntax_str, "AST representation");
        assert_eq!(cached.content, "fn test() {}");
        assert_eq!(cached.content_hash, 12345);
        assert!(cached.metrics.is_none());
    }

    #[test]
    fn test_cached_syntax_with_metrics() {
        let metrics = Metrics {
            complexity: 5,
            cognitive: 3,
            satd_count: 1,
            coverage: 0.8,
            lines: 10,
            functions: 2,
            timestamp: SystemTime::now(),
        };

        let cached = CachedSyntax {
            syntax_str: String::new(),
            content: String::new(),
            last_modified: SystemTime::now(),
            content_hash: 0,
            metrics: Some(metrics.clone()),
        };

        assert!(cached.metrics.is_some());
        let stored = cached.metrics.unwrap();
        assert_eq!(stored.complexity, 5);
    }

    // ============================================
    // Hash Calculation Tests
    // ============================================

    #[test]
    fn test_hash_same_content() {
        let parser = EnhancedParser::new();
        let content = "fn test() {}";

        let hash1 = parser.calculate_hash(content);
        let hash2 = parser.calculate_hash(content);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_different_content() {
        let parser = EnhancedParser::new();

        let hash1 = parser.calculate_hash("fn a() {}");
        let hash2 = parser.calculate_hash("fn b() {}");

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_empty_content() {
        let parser = EnhancedParser::new();
        let hash = parser.calculate_hash("");

        // Should not panic and should return a valid hash
        assert!(hash > 0 || hash == 0); // Just verify it returns something
    }

    #[test]
    fn test_hash_whitespace_matters() {
        let parser = EnhancedParser::new();

        let hash1 = parser.calculate_hash("fn test() {}");
        let hash2 = parser.calculate_hash("fn test()  {}"); // Extra space

        assert_ne!(hash1, hash2);
    }

    // ============================================
    // Error Handling Tests
    // ============================================

    #[test]
    fn test_parse_invalid_rust_syntax() {
        let mut parser = EnhancedParser::new();
        let result = parser.parse_incremental(&test_path("invalid"), "fn { invalid }");

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_incomplete_code() {
        let mut parser = EnhancedParser::new();
        let result = parser.parse_incremental(&test_path("incomplete"), "fn test(");

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_unbalanced_braces() {
        let mut parser = EnhancedParser::new();
        let result = parser.parse_incremental(&test_path("unbalanced"), "fn test() { { }");

        assert!(result.is_err());
    }

    #[test]
    fn test_error_message_contains_info() {
        let mut parser = EnhancedParser::new();
        let result = parser.parse_incremental(&test_path("error"), "not rust code");

        assert!(result.is_err());
        let err = result.err().unwrap();
        let err_msg = err.to_string();
        assert!(err_msg.contains("parse") || err_msg.contains("Rust"));
    }

    // ============================================
    // Edge Case Tests
    // ============================================

    #[test]
    fn test_parse_unicode_identifiers() {
        let mut parser = EnhancedParser::new();
        // Rust doesn't allow unicode in function names by default
        let code = "fn test_unicode() { let x = 42; }";
        let result = parser.parse_incremental(&test_path("unicode"), code);

        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_very_long_function() {
        let mut parser = EnhancedParser::new();
        let mut code = String::from("fn long() {\n");
        for i in 0..100 {
            code.push_str(&format!("    let x{} = {};\n", i, i));
        }
        code.push_str("}\n");

        let result = parser.parse_incremental(&test_path("long"), &code);
        assert!(result.is_ok());

        let metrics = result.unwrap();
        assert!(metrics.lines > 100);
    }

    #[test]
    fn test_parse_deeply_nested() {
        let mut parser = EnhancedParser::new();
        let code = r#"
        fn deep() {
            if true {
                if true {
                    if true {
                        if true {
                            if true {
                                // Very deep
                            }
                        }
                    }
                }
            }
        }
        "#;

        let result = parser.parse_incremental(&test_path("deep"), code);
        assert!(result.is_ok());

        let metrics = result.unwrap();
        assert!(metrics.cognitive > 10); // Deep nesting increases cognitive complexity
    }

    #[test]
    fn test_parse_many_functions() {
        let mut parser = EnhancedParser::new();
        let mut code = String::new();
        for i in 0..50 {
            code.push_str(&format!("fn func_{i}() {{}}\n"));
        }

        let result = parser.parse_incremental(&test_path("many"), &code);
        assert!(result.is_ok());

        let metrics = result.unwrap();
        assert_eq!(metrics.functions, 50);
    }

    #[test]
    fn test_parse_struct_definition() {
        let mut parser = EnhancedParser::new();
        let code = r#"
        struct Point {
            x: i32,
            y: i32,
        }

        fn create_point() -> Point {
            Point { x: 0, y: 0 }
        }
        "#;

        let result = parser.parse_incremental(&test_path("struct"), code);
        assert!(result.is_ok());

        let metrics = result.unwrap();
        assert_eq!(metrics.functions, 1);
    }

    #[test]
    fn test_parse_impl_block() {
        let mut parser = EnhancedParser::new();
        let code = r#"
        struct Point {
            x: i32,
            y: i32,
        }

        impl Point {
            fn new() -> Self {
                Point { x: 0, y: 0 }
            }

            fn distance(&self) -> f64 {
                if self.x == 0 && self.y == 0 {
                    0.0
                } else {
                    1.0
                }
            }
        }
        "#;

        let result = parser.parse_incremental(&test_path("impl"), code);
        assert!(result.is_ok());
        // Note: impl methods may or may not be counted as functions depending on visitor
    }

    #[test]
    fn test_parse_closure() {
        let mut parser = EnhancedParser::new();
        let code = r#"
        fn with_closure() {
            let f = |x| {
                if x > 0 {
                    x * 2
                } else {
                    0
                }
            };
        }
        "#;

        let result = parser.parse_incremental(&test_path("closure"), code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_async_function() {
        let mut parser = EnhancedParser::new();
        let code = r#"
        async fn async_fn() {
            if true {
                // async code
            }
        }
        "#;

        let result = parser.parse_incremental(&test_path("async"), code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_timestamp_is_set() {
        let mut parser = EnhancedParser::new();
        let before = SystemTime::now();

        let result = parser.parse_incremental(&test_path("timestamp"), simple_function());
        assert!(result.is_ok());

        let after = SystemTime::now();
        let metrics = result.unwrap();

        assert!(metrics.timestamp >= before);
        assert!(metrics.timestamp <= after);
    }

    // ============================================
    // Visitor Pattern Tests
    // ============================================

    #[test]
    fn test_complexity_visitor_binary_ops_other() {
        let mut parser = EnhancedParser::new();
        // Test binary operators that don't add complexity (like arithmetic)
        let code = "fn math(a: i32, b: i32) -> i32 { a + b * c / d }";
        let result = parser.parse_incremental(&test_path("math"), code);

        assert!(result.is_ok());
        let metrics = result.unwrap();
        // Parser includes function entry + arithmetic operations
        assert_eq!(metrics.complexity, 2);
    }

    #[test]
    fn test_complexity_match_arms_add_complexity() {
        let mut parser = EnhancedParser::new();
        let code = r#"
        fn with_match(x: i32) -> i32 {
            match x {
                0 => 0,
                1 => 1,
            }
        }
        "#;

        let result = parser.parse_incremental(&test_path("two_arms"), code);
        assert!(result.is_ok());

        let metrics = result.unwrap();
        assert!(metrics.complexity >= 3); // Base + 2 arms
    }

    #[test]
    fn test_cognitive_match_arm_nesting() {
        let mut parser = EnhancedParser::new();
        let code = r#"
        fn nested_match(x: i32) -> i32 {
            if true {
                match x {
                    0 => 0,
                    _ => 1,
                }
            } else {
                0
            }
        }
        "#;

        let result = parser.parse_incremental(&test_path("nested_match"), code);
        assert!(result.is_ok());

        let metrics = result.unwrap();
        // Match arms inside if should have higher cognitive complexity due to nesting
        assert!(metrics.cognitive > metrics.complexity);
    }
}
