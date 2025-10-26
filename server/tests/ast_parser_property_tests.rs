// Sprint 60: Enhanced Coverage - Property-Based Testing for AST Parsers
//
// This module demonstrates property-based testing for PMAT's AST parsers using proptest.
// Property tests validate invariants that should hold for all valid inputs, catching
// edge cases that traditional unit tests might miss.
//
// Reference: docs/sprints/SPRINT-60-ENHANCED-COVERAGE-STRATEGY.md

use proptest::prelude::*;
use server::services::ast::languages::{
    rust::RustAstStrategy,
    python::PythonAstStrategy,
    javascript::JavaScriptAstStrategy,
    typescript::TypeScriptAstStrategy,
};
use server::services::ast::strategy::AstStrategy;
use std::path::PathBuf;

// ==============================================================================
// Property Test Generators
// ==============================================================================

/// Generate valid Rust function names (identifiers)
fn rust_identifier() -> impl Strategy<Value = String> {
    // Rust identifiers: [a-zA-Z_][a-zA-Z0-9_]*
    prop::string::string_regex("[a-zA-Z_][a-zA-Z0-9_]{0,50}").unwrap()
}

/// Generate valid Python function names
fn python_identifier() -> impl Strategy<Value = String> {
    // Python identifiers: [a-zA-Z_][a-zA-Z0-9_]*
    prop::string::string_regex("[a-zA-Z_][a-zA-Z0-9_]{0,50}").unwrap()
}

/// Generate simple Rust function code
fn rust_function_code() -> impl Strategy<Value = String> {
    rust_identifier().prop_map(|name| {
        format!(
            "pub fn {name}() {{\n    println!(\"Hello from {name}\");\n}}\n"
        )
    })
}

/// Generate simple Python function code
fn python_function_code() -> impl Strategy<Value = String> {
    python_identifier().prop_map(|name| {
        format!(
            "def {name}():\n    print('Hello from {name}')\n    pass\n"
        )
    })
}

/// Generate JavaScript function code (both function declarations and arrow functions)
fn javascript_function_code() -> impl Strategy<Value = String> {
    prop_oneof![
        // Function declaration
        rust_identifier().prop_map(|name| {
            format!("function {name}() {{\n  console.log('Hello from {name}');\n}}\n")
        }),
        // Arrow function
        rust_identifier().prop_map(|name| {
            format!("const {name} = () => {{\n  console.log('Hello from {name}');\n}};\n")
        }),
    ]
}

/// Generate TypeScript function code with type annotations
fn typescript_function_code() -> impl Strategy<Value = String> {
    prop_oneof![
        // Function with return type
        rust_identifier().prop_map(|name| {
            format!("function {name}(): void {{\n  console.log('Hello from {name}');\n}}\n")
        }),
        // Arrow function with types
        rust_identifier().prop_map(|name| {
            format!("const {name} = (): string => {{\n  return 'Hello from {name}';\n}};\n")
        }),
    ]
}

/// Generate code with varying complexity (0-10 nested blocks)
fn nested_blocks(depth: usize) -> String {
    let mut code = String::new();
    for i in 0..depth {
        code.push_str(&format!("if true {{\n{}", "  ".repeat(i + 1)));
    }
    code.push_str("println!(\"nested\");\n");
    for _ in 0..depth {
        code.push_str("}\n");
    }
    code
}

// ==============================================================================
// Property Tests: AST Parser Invariants
// ==============================================================================

/// Property 1: Parsers should never panic on valid code
///
/// Invariant: For all valid function code, the parser should either:
/// - Successfully parse and return items, OR
/// - Return an empty result (no panic)
mod parser_robustness_tests {
    use super::*;

    proptest! {
        #[test]
        fn rust_parser_never_panics_on_valid_code(
            func_code in rust_function_code()
        ) {
            let strategy = RustAstStrategy;
            let result = std::panic::catch_unwind(|| {
                strategy.parse_content(&func_code, &PathBuf::from("test.rs"))
            });

            // Parser should not panic
            prop_assert!(result.is_ok(), "Rust parser panicked on valid code");

            // If parsing succeeded, result should be Ok
            if let Ok(parse_result) = result {
                prop_assert!(parse_result.is_ok(), "Rust parser returned error on valid code");
            }
        }

        #[test]
        fn python_parser_never_panics_on_valid_code(
            func_code in python_function_code()
        ) {
            let strategy = PythonAstStrategy;
            let result = std::panic::catch_unwind(|| {
                strategy.parse_content(&func_code, &PathBuf::from("test.py"))
            });

            prop_assert!(result.is_ok(), "Python parser panicked on valid code");

            if let Ok(parse_result) = result {
                prop_assert!(parse_result.is_ok(), "Python parser returned error on valid code");
            }
        }

        #[test]
        fn javascript_parser_never_panics_on_valid_code(
            func_code in javascript_function_code()
        ) {
            let strategy = JavaScriptAstStrategy;
            let result = std::panic::catch_unwind(|| {
                strategy.parse_content(&func_code, &PathBuf::from("test.js"))
            });

            prop_assert!(result.is_ok(), "JavaScript parser panicked on valid code");

            if let Ok(parse_result) = result {
                prop_assert!(parse_result.is_ok(), "JavaScript parser returned error on valid code");
            }
        }

        #[test]
        fn typescript_parser_never_panics_on_valid_code(
            func_code in typescript_function_code()
        ) {
            let strategy = TypeScriptAstStrategy;
            let result = std::panic::catch_unwind(|| {
                strategy.parse_content(&func_code, &PathBuf::from("test.ts"))
            });

            prop_assert!(result.is_ok(), "TypeScript parser panicked on valid code");

            if let Ok(parse_result) = result {
                prop_assert!(parse_result.is_ok(), "TypeScript parser returned error on valid code");
            }
        }
    }
}

/// Property 2: Parser output should have consistent naming
///
/// Invariant: If parser extracts a function with name X, then:
/// - The name field should exactly match X
/// - The name should be a valid identifier
mod parser_naming_consistency_tests {
    use super::*;

    proptest! {
        #[test]
        fn rust_parser_preserves_function_names(
            func_name in rust_identifier()
        ) {
            let code = format!("pub fn {func_name}() {{}}\n");
            let strategy = RustAstStrategy;

            if let Ok(items) = strategy.parse_content(&code, &PathBuf::from("test.rs")) {
                if !items.is_empty() {
                    let parsed_name = &items[0].name;
                    prop_assert_eq!(
                        parsed_name, &func_name,
                        "Rust parser altered function name: expected {func_name}, got {parsed_name}"
                    );
                }
            }
        }

        #[test]
        fn python_parser_preserves_function_names(
            func_name in python_identifier()
        ) {
            let code = format!("def {func_name}():\n    pass\n");
            let strategy = PythonAstStrategy;

            if let Ok(items) = strategy.parse_content(&code, &PathBuf::from("test.py")) {
                if !items.is_empty() {
                    let parsed_name = &items[0].name;
                    prop_assert_eq!(
                        parsed_name, &func_name,
                        "Python parser altered function name: expected {func_name}, got {parsed_name}"
                    );
                }
            }
        }
    }
}

/// Property 3: Complexity should increase monotonically with nesting
///
/// Invariant: For all depths d1 < d2, complexity(code_with_depth_d1) ≤ complexity(code_with_depth_d2)
mod complexity_monotonicity_tests {
    use super::*;

    proptest! {
        #[test]
        fn rust_complexity_increases_with_nesting(
            depth1 in 0usize..5,
            depth2 in 5usize..10
        ) {
            prop_assume!(depth1 < depth2);

            let code1 = format!("fn test() {{\n{}\n}}\n", nested_blocks(depth1));
            let code2 = format!("fn test() {{\n{}\n}}\n", nested_blocks(depth2));

            let strategy = RustAstStrategy;

            if let (Ok(items1), Ok(items2)) = (
                strategy.parse_content(&code1, &PathBuf::from("test.rs")),
                strategy.parse_content(&code2, &PathBuf::from("test.rs")),
            ) {
                if !items1.is_empty() && !items2.is_empty() {
                    let complexity1 = items1[0].metrics.cyclomatic_complexity.unwrap_or(0);
                    let complexity2 = items2[0].metrics.cyclomatic_complexity.unwrap_or(0);

                    prop_assert!(
                        complexity1 <= complexity2,
                        "Complexity did not increase monotonically: depth {depth1} -> {complexity1}, depth {depth2} -> {complexity2}"
                    );
                }
            }
        }

        #[test]
        fn python_complexity_increases_with_nesting(
            depth1 in 0usize..5,
            depth2 in 5usize..10
        ) {
            prop_assume!(depth1 < depth2);

            let code1 = format!("def test():\n{}\n", nested_blocks(depth1).replace("println!", "print"));
            let code2 = format!("def test():\n{}\n", nested_blocks(depth2).replace("println!", "print"));

            let strategy = PythonAstStrategy;

            if let (Ok(items1), Ok(items2)) = (
                strategy.parse_content(&code1, &PathBuf::from("test.py")),
                strategy.parse_content(&code2, &PathBuf::from("test.py")),
            ) {
                if !items1.is_empty() && !items2.is_empty() {
                    let complexity1 = items1[0].metrics.cyclomatic_complexity.unwrap_or(0);
                    let complexity2 = items2[0].metrics.cyclomatic_complexity.unwrap_or(0);

                    prop_assert!(
                        complexity1 <= complexity2,
                        "Complexity did not increase monotonically: depth {depth1} -> {complexity1}, depth {depth2} -> {complexity2}"
                    );
                }
            }
        }
    }
}

/// Property 4: Idempotence - Parsing the same code twice should yield identical results
///
/// Invariant: For all valid code C, parse(C) == parse(C)
mod parser_idempotence_tests {
    use super::*;

    proptest! {
        #[test]
        fn rust_parser_is_idempotent(
            func_code in rust_function_code()
        ) {
            let strategy = RustAstStrategy;
            let path = PathBuf::from("test.rs");

            if let (Ok(result1), Ok(result2)) = (
                strategy.parse_content(&func_code, &path),
                strategy.parse_content(&func_code, &path),
            ) {
                prop_assert_eq!(
                    result1.len(), result2.len(),
                    "Rust parser produced different number of items on re-parse"
                );

                for (item1, item2) in result1.iter().zip(result2.iter()) {
                    prop_assert_eq!(
                        &item1.name, &item2.name,
                        "Function names differ on re-parse"
                    );
                    prop_assert_eq!(
                        item1.metrics.cyclomatic_complexity,
                        item2.metrics.cyclomatic_complexity,
                        "Complexity metrics differ on re-parse"
                    );
                }
            }
        }

        #[test]
        fn python_parser_is_idempotent(
            func_code in python_function_code()
        ) {
            let strategy = PythonAstStrategy;
            let path = PathBuf::from("test.py");

            if let (Ok(result1), Ok(result2)) = (
                strategy.parse_content(&func_code, &path),
                strategy.parse_content(&func_code, &path),
            ) {
                prop_assert_eq!(
                    result1.len(), result2.len(),
                    "Python parser produced different number of items on re-parse"
                );

                for (item1, item2) in result1.iter().zip(result2.iter()) {
                    prop_assert_eq!(
                        &item1.name, &item2.name,
                        "Function names differ on re-parse"
                    );
                }
            }
        }
    }
}

/// Property 5: Empty input handling
///
/// Invariant: For all empty strings or whitespace-only strings, parser returns empty list (not error)
mod parser_empty_input_tests {
    use super::*;

    proptest! {
        #[test]
        fn rust_parser_handles_empty_input(
            whitespace in prop::string::string_regex("[ \t\n\r]{0,100}").unwrap()
        ) {
            let strategy = RustAstStrategy;

            let result = strategy.parse_content(&whitespace, &PathBuf::from("test.rs"));

            prop_assert!(
                result.is_ok(),
                "Rust parser failed on empty/whitespace input"
            );

            if let Ok(items) = result {
                prop_assert!(
                    items.is_empty(),
                    "Rust parser found items in empty/whitespace input"
                );
            }
        }

        #[test]
        fn python_parser_handles_empty_input(
            whitespace in prop::string::string_regex("[ \t\n\r]{0,100}").unwrap()
        ) {
            let strategy = PythonAstStrategy;

            let result = strategy.parse_content(&whitespace, &PathBuf::from("test.py"));

            prop_assert!(
                result.is_ok(),
                "Python parser failed on empty/whitespace input"
            );

            if let Ok(items) = result {
                prop_assert!(
                    items.is_empty(),
                    "Python parser found items in empty/whitespace input"
                );
            }
        }
    }
}

// ==============================================================================
// Example: Running Property Tests
// ==============================================================================
//
// To run these tests:
//
//   cargo test --lib ast_parser_property_tests
//
// To run with more cases (default is 256):
//
//   PROPTEST_CASES=10000 cargo test --lib ast_parser_property_tests
//
// To save regression test cases:
//
//   PROPTEST_REGRESSIONS=overwrite cargo test --lib ast_parser_property_tests
//
// Sprint 60 Target: 85-87% line coverage, 75-80% mutation score
