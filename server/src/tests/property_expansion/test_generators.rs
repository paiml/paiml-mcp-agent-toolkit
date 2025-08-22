//! Test data generators for property-based testing
//! 
//! This module provides reusable generators for creating test data
//! across different property test suites.

use proptest::prelude::*;
use std::path::PathBuf;

/// Generate valid Rust identifiers
pub fn rust_identifier() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,30}".prop_map(|s| s.replace("__", "_"))
}

/// Generate valid TypeScript/JavaScript identifiers  
pub fn js_identifier() -> impl Strategy<Value = String> {
    "[a-zA-Z_$][a-zA-Z0-9_$]{0,30}"
}

/// Generate valid Python identifiers
pub fn python_identifier() -> impl Strategy<Value = String> {
    "[a-z_][a-z0-9_]{0,30}".prop_map(|s| s.replace("__", "_"))
}

/// Generate file paths
pub fn file_path() -> impl Strategy<Value = PathBuf> {
    prop_oneof![
        "[a-z0-9_]+\\.rs",
        "src/[a-z0-9_]+\\.rs",
        "src/[a-z0-9_]+/[a-z0-9_]+\\.rs",
        "tests/[a-z0-9_]+\\.rs",
    ].prop_map(PathBuf::from)
}

/// Generate source code snippets
#[derive(Debug, Clone)]
pub enum CodeSnippet {
    Function { name: String, body: String },
    Struct { name: String, fields: Vec<String> },
    Enum { name: String, variants: Vec<String> },
    Impl { type_name: String, methods: Vec<String> },
    Module { name: String, items: Vec<String> },
}

impl CodeSnippet {
    /// Convert to Rust source code
    pub fn to_rust(&self) -> String {
        match self {
            CodeSnippet::Function { name, body } => {
                format!("fn {}() {{\n    {}\n}}", name, body)
            }
            CodeSnippet::Struct { name, fields } => {
                let field_str = fields
                    .iter()
                    .map(|f| format!("    {}: String,", f))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("struct {} {{\n{}\n}}", name, field_str)
            }
            CodeSnippet::Enum { name, variants } => {
                let variant_str = variants
                    .iter()
                    .map(|v| format!("    {},", v))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("enum {} {{\n{}\n}}", name, variant_str)
            }
            CodeSnippet::Impl { type_name, methods } => {
                let method_str = methods
                    .iter()
                    .map(|m| format!("    fn {}(&self) {{}}", m))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("impl {} {{\n{}\n}}", type_name, method_str)
            }
            CodeSnippet::Module { name, items } => {
                let item_str = items
                    .iter()
                    .map(|i| format!("    use {};", i))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("mod {} {{\n{}\n}}", name, item_str)
            }
        }
    }
}

impl Arbitrary for CodeSnippet {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;
    
    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        prop_oneof![
            (rust_identifier(), "[a-z0-9_ ]+").prop_map(|(name, body)| {
                CodeSnippet::Function { name, body }
            }),
            (
                rust_identifier(),
                prop::collection::vec(rust_identifier(), 1..5)
            ).prop_map(|(name, fields)| {
                CodeSnippet::Struct { name, fields }
            }),
            (
                rust_identifier(),
                prop::collection::vec(rust_identifier(), 1..5)
            ).prop_map(|(name, variants)| {
                CodeSnippet::Enum { name, variants }
            }),
        ].boxed()
    }
}

/// Generate complexity scores
pub fn complexity_score() -> impl Strategy<Value = u32> {
    prop_oneof![
        1..=10u32,  // Low complexity
        11..=20u32, // Medium complexity  
        21..=50u32, // High complexity
        51..=100u32, // Very high complexity
    ]
}

/// Generate test configuration
#[derive(Debug, Clone)]
pub struct TestConfig {
    pub timeout_ms: u32,
    pub max_iterations: usize,
    pub parallel: bool,
    pub seed: Option<u64>,
}

impl Arbitrary for TestConfig {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;
    
    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        (
            100u32..60000,  // timeout between 100ms and 60s
            10usize..1000,  // iterations
            prop::bool::ANY,
            prop::option::of(0u64..1000000),
        ).prop_map(|(timeout_ms, max_iterations, parallel, seed)| {
            TestConfig {
                timeout_ms,
                max_iterations,
                parallel,
                seed,
            }
        }).boxed()
    }
}

/// Generate version strings
pub fn version_string() -> impl Strategy<Value = String> {
    (0u32..100, 0u32..100, 0u32..100)
        .prop_map(|(major, minor, patch)| format!("{}.{}.{}", major, minor, patch))
}

/// Generate error messages
pub fn error_message() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("File not found".to_string()),
        Just("Permission denied".to_string()),
        Just("Invalid syntax".to_string()),
        Just("Timeout exceeded".to_string()),
        Just("Memory limit exceeded".to_string()),
        "[a-zA-Z ]{10,50}".prop_map(|s| format!("Error: {}", s)),
    ]
}

/// Generate test cases for AST analysis
#[derive(Debug, Clone)]
pub struct AstTestCase {
    pub source: String,
    pub expected_complexity: u32,
    pub has_errors: bool,
}

impl Arbitrary for AstTestCase {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;
    
    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        (
            any::<CodeSnippet>(),
            complexity_score(),
            prop::bool::ANY,
        ).prop_map(|(snippet, expected_complexity, has_errors)| {
            AstTestCase {
                source: snippet.to_rust(),
                expected_complexity,
                has_errors,
            }
        }).boxed()
    }
}

/// Generate cache keys for testing
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct TestCacheKey {
    pub category: String,
    pub id: u64,
    pub version: u32,
}

impl Arbitrary for TestCacheKey {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;
    
    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        (
            prop::sample::select(vec!["ast", "complexity", "satd", "cache"]),
            0u64..1000000,
            1u32..10,
        ).prop_map(|(category, id, version)| {
            TestCacheKey {
                category,
                id,
                version,
            }
        }).boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rust_identifier_generation() {
        let runner = proptest::test_runner::TestRunner::default();
        let strategy = rust_identifier();
        
        for _ in 0..10 {
            let value = strategy.new_tree(&mut runner.clone()).unwrap().current();
            // Should be valid Rust identifier
            assert!(value.chars().next().unwrap().is_alphabetic());
            assert!(!value.contains("__")); // No double underscores
        }
    }
    
    #[test]
    fn test_code_snippet_generation() {
        let snippet = CodeSnippet::Function {
            name: "test_func".to_string(),
            body: "let x = 42;".to_string(),
        };
        
        let rust_code = snippet.to_rust();
        assert!(rust_code.contains("fn test_func()"));
        assert!(rust_code.contains("let x = 42;"));
    }
    
    #[test]
    fn test_version_string_generation() {
        let runner = proptest::test_runner::TestRunner::default();
        let strategy = version_string();
        
        let value = strategy.new_tree(&mut runner.clone()).unwrap().current();
        // Should match semantic version format
        let parts: Vec<&str> = value.split('.').collect();
        assert_eq!(parts.len(), 3);
        for part in parts {
            assert!(part.parse::<u32>().is_ok());
        }
    }
}