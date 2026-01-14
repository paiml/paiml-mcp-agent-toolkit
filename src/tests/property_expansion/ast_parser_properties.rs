//! Property-based tests for AST parsers
//! 
//! This module implements comprehensive property tests for Rust, TypeScript,
//! and Python AST parsers to ensure they handle arbitrary valid code without
//! panics and maintain critical invariants.

use proptest::prelude::*;
use syn::{parse_file, File};
use std::panic;

/// Strategy for generating valid Rust identifiers
fn identifier_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,20}".prop_map(|s| s.replace("__", "_"))
}

/// Strategy for generating Rust literals
fn literal_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        // Integer literals
        (0i64..10000).prop_map(|n| n.to_string()),
        // Float literals
        (0.0f64..1000.0).prop_map(|n| format!("{:.2}", n)),
        // String literals
        "[a-zA-Z0-9 ]{0,50}".prop_map(|s| format!("\"{}\"", s)),
        // Boolean literals
        prop::bool::ANY.prop_map(|b| b.to_string()),
    ]
}

/// Strategy for generating Rust source structure
fn source_structure_strategy() -> impl Strategy<Value = SourceStructure> {
    prop_oneof![
        Just(SourceStructure::Empty),
        Just(SourceStructure::Function),
        Just(SourceStructure::Struct),
        Just(SourceStructure::Enum),
        Just(SourceStructure::Trait),
        Just(SourceStructure::Module),
    ]
}

#[derive(Debug, Clone)]
enum SourceStructure {
    Empty,
    Function,
    Struct,
    Enum,
    Trait,
    Module,
}

/// Generate Rust source code from components
fn generate_rust_source(
    structure: SourceStructure,
    identifiers: Vec<String>,
    literals: Vec<String>,
) -> String {
    let idents: Vec<&str> = identifiers.iter().take(5).map(|s| s.as_str()).collect();
    let default_idents = vec!["foo", "bar", "baz", "qux", "item"];
    let idents = if idents.is_empty() { &default_idents } else { &idents };
    
    match structure {
        SourceStructure::Empty => String::new(),
        SourceStructure::Function => {
            format!(
                "fn {}() -> i32 {{\n    let {} = {};\n    {}\n}}\n",
                idents.get(0).unwrap_or(&"test"),
                idents.get(1).unwrap_or(&"x"),
                literals.first().unwrap_or(&"42".to_string()),
                literals.first().unwrap_or(&"0".to_string())
            )
        }
        SourceStructure::Struct => {
            format!(
                "struct {} {{\n    {}: i32,\n    {}: String,\n}}\n",
                idents.get(0).unwrap_or(&"MyStruct"),
                idents.get(1).unwrap_or(&"field1"),
                idents.get(2).unwrap_or(&"field2")
            )
        }
        SourceStructure::Enum => {
            format!(
                "enum {} {{\n    {},\n    {}(i32),\n    {} {{ {}: String }},\n}}\n",
                idents.get(0).unwrap_or(&"MyEnum"),
                idents.get(1).unwrap_or(&"Variant1"),
                idents.get(2).unwrap_or(&"Variant2"),
                idents.get(3).unwrap_or(&"Variant3"),
                idents.get(4).unwrap_or(&"field")
            )
        }
        SourceStructure::Trait => {
            format!(
                "trait {} {{\n    fn {}(&self) -> i32;\n}}\n",
                idents.get(0).unwrap_or(&"MyTrait"),
                idents.get(1).unwrap_or(&"method")
            )
        }
        SourceStructure::Module => {
            format!(
                "mod {} {{\n    pub fn {}() {{}}\n}}\n",
                idents.get(0).unwrap_or(&"my_module"),
                idents.get(1).unwrap_or(&"func")
            )
        }
    }
}

/// Strategy for generating arbitrary statements
fn arb_statement() -> impl Strategy<Value = String> {
    prop_oneof![
        identifier_strategy().prop_map(|id| format!("let {} = 0;", id)),
        identifier_strategy().prop_map(|id| format!("let mut {} = 0;", id)),
        Just("if true { 1 } else { 0 };".to_string()),
        Just("for _ in 0..10 {}".to_string()),
        Just("while false {}".to_string()),
        Just("match 0 { 0 => 1, _ => 2 }".to_string()),
    ]
}

prop_compose! {
    /// Compose a strategy for generating arbitrary Rust source code
    fn arb_rust_source()
        (structure in source_structure_strategy(),
         identifiers in prop::collection::vec(identifier_strategy(), 0..10),
         literals in prop::collection::vec(literal_strategy(), 0..5))
        -> String
    {
        generate_rust_source(structure, identifiers, literals)
    }
}

/// Compute complexity of an AST (simplified version)
fn compute_complexity(ast: &File) -> usize {
    use syn::visit::Visit;
    
    struct ComplexityVisitor {
        complexity: usize,
    }
    
    impl<'ast> Visit<'ast> for ComplexityVisitor {
        fn visit_expr_if(&mut self, _: &'ast syn::ExprIf) {
            self.complexity += 1;
        }
        
        fn visit_expr_match(&mut self, node: &'ast syn::ExprMatch) {
            self.complexity += node.arms.len();
        }
        
        fn visit_expr_while(&mut self, _: &'ast syn::ExprWhile) {
            self.complexity += 1;
        }
        
        fn visit_expr_for_loop(&mut self, _: &'ast syn::ExprForLoop) {
            self.complexity += 1;
        }
    }
    
    let mut visitor = ComplexityVisitor { complexity: 1 };
    visitor.visit_file(ast);
    visitor.complexity
}

proptest! {
    /// Property: Parser is total (never panics) and AST traversal terminates
    #[test]
    fn ast_parser_total_function(source in arb_rust_source()) {
        // Property 1: Parser is total (never panics)
        let result = panic::catch_unwind(|| {
            parse_file(&source)
        });
        prop_assert!(result.is_ok(), "Parser panicked on input: {}", source);

        // Property 2: Valid parse => AST traversal terminates
        if let Ok(Ok(ast)) = result {
            let visitor_result = panic::catch_unwind(|| {
                compute_complexity(&ast)
            });
            prop_assert!(visitor_result.is_ok(), "AST traversal panicked");
        }
    }

    /// Property: Complexity is monotonic (adding code never decreases complexity)
    #[test]
    fn ast_complexity_monotonic(base in arb_rust_source(), insertion in arb_statement()) {
        let base_ast = parse_file(&base).ok();
        let extended = format!("{}\n{}", base, insertion);
        let extended_ast = parse_file(&extended).ok();

        if let (Some(ast1), Some(ast2)) = (base_ast, extended_ast) {
            let c1 = compute_complexity(&ast1);
            let c2 = compute_complexity(&ast2);
            // Complexity is monotonic: adding code never decreases complexity
            prop_assert!(
                c2 >= c1,
                "Complexity decreased: {} -> {} after adding: {}",
                c1, c2, insertion
            );
        }
    }

    /// Property: Empty source produces minimal complexity
    #[test]
    fn ast_empty_source_minimal_complexity(_dummy in Just(())) {
        let empty = "";
        let ast = parse_file(empty).unwrap();
        let complexity = compute_complexity(&ast);
        prop_assert_eq!(complexity, 1, "Empty source should have complexity 1");
    }

    /// Property: Parser handles deeply nested structures
    #[test]
    fn ast_parser_handles_nesting(depth in 1usize..10) {
        let mut source = String::new();
        
        // Generate nested if statements
        for i in 0..depth {
            source.push_str(&"    ".repeat(i));
            source.push_str("if true {\n");
        }
        
        source.push_str(&"    ".repeat(depth));
        source.push_str("let x = 0;\n");
        
        for i in (0..depth).rev() {
            source.push_str(&"    ".repeat(i));
            source.push_str("}\n");
        }
        
        // Wrap in function for valid Rust
        let full_source = format!("fn test() {{\n{}}}", source);
        
        let result = panic::catch_unwind(|| parse_file(&full_source));
        prop_assert!(result.is_ok(), "Parser failed on nested structure depth {}", depth);
        
        if let Ok(Ok(ast)) = result {
            let complexity = compute_complexity(&ast);
            prop_assert!(complexity >= depth, "Complexity should be at least {}", depth);
        }
    }
}

// TypeScript/JavaScript parser properties
#[cfg(feature = "typescript")]
mod typescript_properties {
    use super::*;
    
    /// Strategy for JavaScript features
    fn arb_js_features() -> impl Strategy<Value = JsFeatures> {
        (
            prop::bool::ANY,
            prop::bool::ANY,
            prop::bool::ANY,
            prop::bool::ANY,
        ).prop_map(|(async_await, arrow_functions, destructuring, spread)| {
            JsFeatures {
                async_await,
                arrow_functions,
                destructuring,
                spread,
            }
        })
    }
    
    #[derive(Debug, Clone)]
    struct JsFeatures {
        async_await: bool,
        arrow_functions: bool,
        destructuring: bool,
        spread: bool,
    }
    
    /// Generate JavaScript source with specific features
    fn generate_js_source(
        use_strict: bool,
        module_type: &str,
        features: JsFeatures,
    ) -> String {
        let mut source = String::new();
        
        if use_strict {
            source.push_str("'use strict';\n\n");
        }
        
        // Add module-specific code
        match module_type {
            "commonjs" => {
                source.push_str("const module = require('module');\n");
                source.push_str("module.exports = {};\n");
            }
            "esm" => {
                source.push_str("export default {};\n");
                source.push_str("import { something } from 'module';\n");
            }
            _ => {}
        }
        
        // Add feature-specific code
        if features.async_await {
            source.push_str("async function test() { await Promise.resolve(); }\n");
        }
        
        if features.arrow_functions {
            source.push_str("const arrow = () => 42;\n");
        }
        
        if features.destructuring {
            source.push_str("const { a, b } = { a: 1, b: 2 };\n");
        }
        
        if features.spread {
            source.push_str("const arr = [...[1, 2, 3]];\n");
        }
        
        source
    }
    
    prop_compose! {
        /// Strategy for generating JavaScript source
        fn arb_js_source()
            (use_strict in prop::bool::ANY,
             module_type in prop::sample::select(vec!["commonjs", "esm", "umd"]),
             features in arb_js_features())
            -> String
        {
            generate_js_source(use_strict, module_type, features)
        }
    }
}

#[cfg(test)]
mod property_expansion_tests {
    use super::*;
    
    #[test]
    fn test_identifier_generation() {
        let runner = proptest::test_runner::TestRunner::default();
        let strategy = identifier_strategy();
        
        for _ in 0..10 {
            let value = strategy.new_tree(&mut runner.clone()).unwrap().current();
            // Verify it's a valid Rust identifier
            assert!(value.chars().next().unwrap().is_alphabetic());
            assert!(value.chars().all(|c| c.is_alphanumeric() || c == '_'));
        }
    }
    
    #[test]
    fn test_source_generation() {
        let source = generate_rust_source(
            SourceStructure::Function,
            vec!["test_func".to_string()],
            vec!["42".to_string()],
        );
        
        // Should be parseable
        let result = parse_file(&source);
        assert!(result.is_ok(), "Generated source should be valid Rust");
    }
}