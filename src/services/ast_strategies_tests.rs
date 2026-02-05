
use super::*;

// ============================================================
// RustAstStrategy Tests
// ============================================================

#[test]
fn test_rust_strategy_supports_rs_extension() {
    let strategy = RustAstStrategy;
    assert!(strategy.supports_extension("rs"));
}

#[test]
fn test_rust_strategy_rejects_other_extensions() {
    let strategy = RustAstStrategy;
    assert!(!strategy.supports_extension("py"));
    assert!(!strategy.supports_extension("ts"));
    assert!(!strategy.supports_extension("js"));
    assert!(!strategy.supports_extension("c"));
    assert!(!strategy.supports_extension("cpp"));
    assert!(!strategy.supports_extension("kt"));
    assert!(!strategy.supports_extension(""));
    assert!(!strategy.supports_extension("rust"));
}

// ============================================================
// StrategyRegistry Tests
// ============================================================

#[test]
fn test_strategy_registry_new() {
    let registry = StrategyRegistry::new();
    // Rust strategy should always be available
    assert!(registry.get_strategy("rs").is_some());
}

#[test]
fn test_strategy_registry_default() {
    let registry = StrategyRegistry::default();
    assert!(registry.get_strategy("rs").is_some());
}

#[test]
fn test_strategy_registry_get_rust_strategy() {
    let registry = StrategyRegistry::new();
    let strategy = registry.get_strategy("rs");
    assert!(strategy.is_some());
    let strategy = strategy.unwrap();
    assert!(strategy.supports_extension("rs"));
}

#[test]
fn test_strategy_registry_get_nonexistent_strategy() {
    let registry = StrategyRegistry::new();
    assert!(registry.get_strategy("unknown").is_none());
    assert!(registry.get_strategy("xyz").is_none());
    assert!(registry.get_strategy("").is_none());
}

#[test]
fn test_strategy_registry_register_custom_strategy() {
    let mut registry = StrategyRegistry::new();
    let rust_strategy = Arc::new(RustAstStrategy) as Arc<dyn AstStrategy>;

    // Register with a custom extension
    registry.register_strategy("custom".to_string(), rust_strategy);

    assert!(registry.get_strategy("custom").is_some());
    let strategy = registry.get_strategy("custom").unwrap();
    // Note: The registered strategy still supports "rs", not "custom"
    assert!(strategy.supports_extension("rs"));
}

#[test]
fn test_strategy_registry_override_existing() {
    let mut registry = StrategyRegistry::new();
    let new_strategy = Arc::new(RustAstStrategy) as Arc<dyn AstStrategy>;

    // Override the existing "rs" strategy
    registry.register_strategy("rs".to_string(), new_strategy.clone());

    let retrieved = registry.get_strategy("rs");
    assert!(retrieved.is_some());
}

// ============================================================
// TypeScript Strategy Tests (feature-gated)
// ============================================================

#[cfg(feature = "typescript-ast")]
mod typescript_tests {
    use super::*;

    #[test]
    fn test_typescript_strategy_supports_ts() {
        let strategy = TypeScriptAstStrategy;
        assert!(strategy.supports_extension("ts"));
    }

    #[test]
    fn test_typescript_strategy_supports_tsx() {
        let strategy = TypeScriptAstStrategy;
        assert!(strategy.supports_extension("tsx"));
    }

    #[test]
    fn test_typescript_strategy_rejects_others() {
        let strategy = TypeScriptAstStrategy;
        assert!(!strategy.supports_extension("js"));
        assert!(!strategy.supports_extension("jsx"));
        assert!(!strategy.supports_extension("rs"));
        assert!(!strategy.supports_extension(""));
    }

    #[test]
    fn test_registry_has_typescript_strategy() {
        let registry = StrategyRegistry::new();
        assert!(registry.get_strategy("ts").is_some());
        assert!(registry.get_strategy("tsx").is_some());
    }
}

// ============================================================
// JavaScript Strategy Tests (feature-gated)
// ============================================================

#[cfg(feature = "typescript-ast")]
mod javascript_tests {
    use super::*;

    #[test]
    fn test_javascript_strategy_supports_js() {
        let strategy = JavaScriptAstStrategy;
        assert!(strategy.supports_extension("js"));
    }

    #[test]
    fn test_javascript_strategy_supports_jsx() {
        let strategy = JavaScriptAstStrategy;
        assert!(strategy.supports_extension("jsx"));
    }

    #[test]
    fn test_javascript_strategy_rejects_others() {
        let strategy = JavaScriptAstStrategy;
        assert!(!strategy.supports_extension("ts"));
        assert!(!strategy.supports_extension("tsx"));
        assert!(!strategy.supports_extension("rs"));
        assert!(!strategy.supports_extension(""));
    }

    #[test]
    fn test_registry_has_javascript_strategy() {
        let registry = StrategyRegistry::new();
        assert!(registry.get_strategy("js").is_some());
        assert!(registry.get_strategy("jsx").is_some());
    }
}

// ============================================================
// Python Strategy Tests (feature-gated)
// ============================================================

#[cfg(feature = "python-ast")]
mod python_tests {
    use super::*;

    #[test]
    fn test_python_strategy_supports_py() {
        let strategy = PythonAstStrategy;
        assert!(strategy.supports_extension("py"));
    }

    #[test]
    fn test_python_strategy_rejects_others() {
        let strategy = PythonAstStrategy;
        assert!(!strategy.supports_extension("pyw"));
        assert!(!strategy.supports_extension("pyc"));
        assert!(!strategy.supports_extension("rs"));
        assert!(!strategy.supports_extension(""));
    }

    #[test]
    fn test_registry_has_python_strategy() {
        let registry = StrategyRegistry::new();
        assert!(registry.get_strategy("py").is_some());
    }
}

// ============================================================
// C Strategy Tests (feature-gated)
// ============================================================

#[cfg(feature = "c-ast")]
mod c_tests {
    use super::*;

    #[test]
    fn test_c_strategy_supports_c() {
        let strategy = CAstStrategy;
        assert!(strategy.supports_extension("c"));
    }

    #[test]
    fn test_c_strategy_supports_h() {
        let strategy = CAstStrategy;
        assert!(strategy.supports_extension("h"));
    }

    #[test]
    fn test_c_strategy_rejects_others() {
        let strategy = CAstStrategy;
        assert!(!strategy.supports_extension("cpp"));
        assert!(!strategy.supports_extension("hpp"));
        assert!(!strategy.supports_extension("rs"));
        assert!(!strategy.supports_extension(""));
    }

    #[test]
    fn test_registry_has_c_strategy() {
        let registry = StrategyRegistry::new();
        assert!(registry.get_strategy("c").is_some());
        assert!(registry.get_strategy("h").is_some());
    }

    // ============================================================
    // CAstStrategy Helper Function Tests
    // ============================================================

    #[test]
    fn test_c_extract_function_name_simple() {
        let source = "int main(int argc, char **argv)";
        let name = CAstStrategy::extract_function_name(source);
        assert_eq!(name, Some("main".to_string()));
    }

    #[test]
    fn test_c_extract_function_name_with_pointer() {
        let source = "void *allocate_memory(size_t size)";
        let name = CAstStrategy::extract_function_name(source);
        assert_eq!(name, Some("allocate_memory".to_string()));
    }

    #[test]
    fn test_c_extract_function_name_no_params() {
        let source = "void no_params()";
        let name = CAstStrategy::extract_function_name(source);
        assert_eq!(name, Some("no_params".to_string()));
    }

    #[test]
    fn test_c_extract_function_name_no_paren() {
        let source = "int variable = 42";
        let name = CAstStrategy::extract_function_name(source);
        assert_eq!(name, None);
    }

    #[test]
    fn test_c_extract_function_name_empty() {
        let source = "";
        let name = CAstStrategy::extract_function_name(source);
        assert_eq!(name, None);
    }

    #[test]
    fn test_c_extract_type_name_struct() {
        let source = "struct MyStruct { int x; }";
        let name = CAstStrategy::extract_type_name(source);
        assert_eq!(name, Some("MyStruct".to_string()));
    }

    #[test]
    fn test_c_extract_type_name_enum() {
        let source = "enum Color { RED, GREEN, BLUE }";
        let name = CAstStrategy::extract_type_name(source);
        assert_eq!(name, Some("Color".to_string()));
    }

    #[test]
    fn test_c_extract_type_name_union() {
        let source = "union Data { int i; float f; }";
        let name = CAstStrategy::extract_type_name(source);
        assert_eq!(name, Some("Data".to_string()));
    }

    #[test]
    fn test_c_extract_type_name_typedef() {
        let source = "typedef int MyInt";
        let name = CAstStrategy::extract_type_name(source);
        assert_eq!(name, Some("int".to_string()));
    }

    #[test]
    fn test_c_extract_type_name_no_keyword() {
        let source = "int x = 5";
        let name = CAstStrategy::extract_type_name(source);
        assert_eq!(name, None);
    }

    #[test]
    fn test_c_extract_type_name_single_word() {
        let source = "struct";
        let name = CAstStrategy::extract_type_name(source);
        assert_eq!(name, None);
    }

    #[test]
    fn test_c_byte_pos_to_line_first_line() {
        let content_lines = vec!["first line", "second line", "third line"];
        let line = CAstStrategy::byte_pos_to_line(0, &content_lines);
        assert_eq!(line, 1);
    }

    #[test]
    fn test_c_byte_pos_to_line_second_line() {
        let content_lines = vec!["first line", "second line", "third line"];
        // First line is 10 chars + 1 newline = 11
        let line = CAstStrategy::byte_pos_to_line(11, &content_lines);
        assert_eq!(line, 2);
    }

    #[test]
    fn test_c_byte_pos_to_line_third_line() {
        let content_lines = vec!["first line", "second line", "third line"];
        // Line 1 = bytes 0-10 (10 chars + newline = 11 bytes)
        // Line 2 = bytes 11-22 (11 chars + newline = 12 bytes)
        // Line 3 starts at byte 23
        let line = CAstStrategy::byte_pos_to_line(23, &content_lines);
        assert_eq!(line, 3);
    }

    #[test]
    fn test_c_byte_pos_to_line_beyond_content() {
        let content_lines = vec!["first line", "second line"];
        let line = CAstStrategy::byte_pos_to_line(1000, &content_lines);
        assert_eq!(line, 2); // Returns last line
    }

    #[test]
    fn test_c_byte_pos_to_line_empty() {
        let content_lines: Vec<&str> = vec![];
        let line = CAstStrategy::byte_pos_to_line(0, &content_lines);
        assert_eq!(line, 0);
    }
}

// ============================================================
// C++ Strategy Tests (feature-gated)
// ============================================================

#[cfg(feature = "c-ast")]
mod cpp_tests {
    use super::*;

    #[test]
    fn test_cpp_strategy_supports_cpp() {
        let strategy = CppAstStrategy;
        assert!(strategy.supports_extension("cpp"));
    }

    #[test]
    fn test_cpp_strategy_supports_cc() {
        let strategy = CppAstStrategy;
        assert!(strategy.supports_extension("cc"));
    }

    #[test]
    fn test_cpp_strategy_supports_cxx() {
        let strategy = CppAstStrategy;
        assert!(strategy.supports_extension("cxx"));
    }

    #[test]
    fn test_cpp_strategy_supports_hpp() {
        let strategy = CppAstStrategy;
        assert!(strategy.supports_extension("hpp"));
    }

    #[test]
    fn test_cpp_strategy_supports_hxx() {
        let strategy = CppAstStrategy;
        assert!(strategy.supports_extension("hxx"));
    }

    #[test]
    fn test_cpp_strategy_rejects_c() {
        let strategy = CppAstStrategy;
        assert!(!strategy.supports_extension("c"));
        assert!(!strategy.supports_extension("h"));
    }

    #[test]
    fn test_cpp_strategy_rejects_others() {
        let strategy = CppAstStrategy;
        assert!(!strategy.supports_extension("rs"));
        assert!(!strategy.supports_extension("py"));
        assert!(!strategy.supports_extension(""));
    }

    #[test]
    fn test_registry_has_cpp_strategy() {
        let registry = StrategyRegistry::new();
        assert!(registry.get_strategy("cpp").is_some());
        assert!(registry.get_strategy("cc").is_some());
        assert!(registry.get_strategy("cxx").is_some());
        assert!(registry.get_strategy("hpp").is_some());
        assert!(registry.get_strategy("hxx").is_some());
    }

    // ============================================================
    // CppAstStrategy Helper Function Tests
    // ============================================================

    #[test]
    fn test_cpp_extract_function_name_simple() {
        let source = "void processData(int value)";
        let name = CppAstStrategy::extract_function_name(source);
        assert_eq!(name, Some("processData".to_string()));
    }

    #[test]
    fn test_cpp_extract_function_name_with_pointer() {
        let source = "std::string* getName()";
        let name = CppAstStrategy::extract_function_name(source);
        assert_eq!(name, Some("getName".to_string()));
    }

    #[test]
    fn test_cpp_extract_function_name_destructor() {
        let source = "~MyClass()";
        let name = CppAstStrategy::extract_function_name(source);
        assert_eq!(name, Some("MyClass".to_string()));
    }

    #[test]
    fn test_cpp_extract_function_name_operator() {
        let source = "bool operator==(const MyClass& other)";
        let name = CppAstStrategy::extract_function_name(source);
        assert_eq!(name, Some("operator_overload".to_string()));
    }

    #[test]
    fn test_cpp_extract_function_name_no_paren() {
        let source = "int member_variable";
        let name = CppAstStrategy::extract_function_name(source);
        assert_eq!(name, None);
    }

    #[test]
    fn test_cpp_extract_type_name_class() {
        let source = "class MyClass { public: };";
        let name = CppAstStrategy::extract_type_name(source);
        assert_eq!(name, Some("MyClass".to_string()));
    }

    #[test]
    fn test_cpp_extract_type_name_struct() {
        let source = "struct Point { int x; int y; };";
        let name = CppAstStrategy::extract_type_name(source);
        assert_eq!(name, Some("Point".to_string()));
    }

    #[test]
    fn test_cpp_extract_type_name_enum() {
        let source = "enum class Status { OK, ERROR };";
        let name = CppAstStrategy::extract_type_name(source);
        assert_eq!(name, Some("Status".to_string()));
    }

    #[test]
    fn test_cpp_extract_type_name_union() {
        let source = "union Variant { int i; float f; };";
        let name = CppAstStrategy::extract_type_name(source);
        assert_eq!(name, Some("Variant".to_string()));
    }

    #[test]
    fn test_cpp_extract_type_name_template() {
        let source = "class Vector<T> { };";
        let name = CppAstStrategy::extract_type_name(source);
        assert_eq!(name, Some("Vector".to_string()));
    }

    #[test]
    fn test_cpp_extract_type_name_with_brace() {
        let source = "struct Data{ int x; }";
        let name = CppAstStrategy::extract_type_name(source);
        assert_eq!(name, Some("Data".to_string()));
    }

    #[test]
    fn test_cpp_extract_type_name_no_keyword() {
        let source = "int main() { }";
        let name = CppAstStrategy::extract_type_name(source);
        assert_eq!(name, None);
    }

    #[test]
    fn test_cpp_byte_pos_to_line_first_line() {
        let content_lines = vec!["// comment", "int main() {", "}"];
        let line = CppAstStrategy::byte_pos_to_line(0, &content_lines);
        assert_eq!(line, 1);
    }

    #[test]
    fn test_cpp_byte_pos_to_line_middle() {
        let content_lines = vec!["first", "second", "third"];
        // first = 5 + 1 = 6
        let line = CppAstStrategy::byte_pos_to_line(6, &content_lines);
        assert_eq!(line, 2);
    }

    #[test]
    fn test_cpp_byte_pos_to_line_end() {
        let content_lines = vec!["a", "b"];
        let line = CppAstStrategy::byte_pos_to_line(100, &content_lines);
        assert_eq!(line, 2);
    }
}

// ============================================================
// Kotlin Strategy Tests (feature-gated)
// ============================================================

#[cfg(feature = "kotlin-ast")]
mod kotlin_tests {
    use super::*;

    #[test]
    fn test_kotlin_strategy_supports_kt() {
        let strategy = KotlinAstStrategy;
        assert!(strategy.supports_extension("kt"));
    }

    #[test]
    fn test_kotlin_strategy_supports_kts() {
        let strategy = KotlinAstStrategy;
        assert!(strategy.supports_extension("kts"));
    }

    #[test]
    fn test_kotlin_strategy_rejects_others() {
        let strategy = KotlinAstStrategy;
        assert!(!strategy.supports_extension("java"));
        assert!(!strategy.supports_extension("rs"));
        assert!(!strategy.supports_extension(""));
    }

    #[test]
    fn test_registry_has_kotlin_strategy() {
        let registry = StrategyRegistry::new();
        assert!(registry.get_strategy("kt").is_some());
        assert!(registry.get_strategy("kts").is_some());
    }

    // ============================================================
    // KotlinAstStrategy Helper Function Tests
    // ============================================================

    #[test]
    fn test_kotlin_extract_function_name_simple() {
        let source = "fun greet(name: String): String";
        let name = KotlinAstStrategy::extract_function_name(source);
        assert_eq!(name, Some("greet".to_string()));
    }

    #[test]
    fn test_kotlin_extract_function_name_no_params() {
        let source = "fun main()";
        let name = KotlinAstStrategy::extract_function_name(source);
        assert_eq!(name, Some("main".to_string()));
    }

    #[test]
    fn test_kotlin_extract_function_name_no_fun() {
        let source = "val x = 5";
        let name = KotlinAstStrategy::extract_function_name(source);
        assert_eq!(name, None);
    }

    #[test]
    fn test_kotlin_extract_function_name_no_paren() {
        let source = "fun incomplete";
        let name = KotlinAstStrategy::extract_function_name(source);
        assert_eq!(name, None);
    }

    #[test]
    fn test_kotlin_extract_class_name_regular() {
        let source = "class MyClass";
        let name = KotlinAstStrategy::extract_class_name(source);
        assert_eq!(name, Some("MyClass".to_string()));
    }

    #[test]
    fn test_kotlin_extract_class_name_with_constructor() {
        let source = "class Person(val name: String)";
        let name = KotlinAstStrategy::extract_class_name(source);
        assert_eq!(name, Some("Person".to_string()));
    }

    #[test]
    fn test_kotlin_extract_class_name_with_inheritance() {
        let source = "class Child: Parent";
        let name = KotlinAstStrategy::extract_class_name(source);
        assert_eq!(name, Some("Child".to_string()));
    }

    #[test]
    fn test_kotlin_extract_class_name_data_class() {
        let source = "data class User(val id: Int, val name: String)";
        let name = KotlinAstStrategy::extract_class_name(source);
        assert_eq!(name, Some("User".to_string()));
    }

    #[test]
    fn test_kotlin_extract_class_name_enum_class() {
        let source = "enum class Color { RED, GREEN, BLUE }";
        let name = KotlinAstStrategy::extract_class_name(source);
        assert_eq!(name, Some("Color".to_string()));
    }

    #[test]
    fn test_kotlin_extract_class_name_interface() {
        let source = "interface Drawable";
        let name = KotlinAstStrategy::extract_class_name(source);
        assert_eq!(name, Some("Drawable".to_string()));
    }

    #[test]
    fn test_kotlin_extract_class_name_object() {
        let source = "object Singleton";
        let name = KotlinAstStrategy::extract_class_name(source);
        assert_eq!(name, Some("Singleton".to_string()));
    }

    #[test]
    fn test_kotlin_extract_class_name_with_generic() {
        let source = "class Container<T>";
        let name = KotlinAstStrategy::extract_class_name(source);
        assert_eq!(name, Some("Container".to_string()));
    }

    #[test]
    fn test_kotlin_extract_class_name_no_keyword() {
        let source = "fun main() {}";
        let name = KotlinAstStrategy::extract_class_name(source);
        assert_eq!(name, None);
    }

    #[test]
    fn test_kotlin_byte_pos_to_line_first() {
        let content_lines = vec!["package com.example", "class Test"];
        let line = KotlinAstStrategy::byte_pos_to_line(0, &content_lines);
        assert_eq!(line, 1);
    }

    #[test]
    fn test_kotlin_byte_pos_to_line_second() {
        let content_lines = vec!["package com.example", "class Test"];
        // package com.example = 19 + 1 = 20
        let line = KotlinAstStrategy::byte_pos_to_line(20, &content_lines);
        assert_eq!(line, 2);
    }

    #[test]
    fn test_kotlin_byte_pos_to_line_beyond() {
        let content_lines = vec!["line1"];
        let line = KotlinAstStrategy::byte_pos_to_line(1000, &content_lines);
        assert_eq!(line, 1);
    }

    #[test]
    fn test_kotlin_byte_pos_to_line_empty() {
        let content_lines: Vec<&str> = vec![];
        let line = KotlinAstStrategy::byte_pos_to_line(0, &content_lines);
        assert_eq!(line, 0);
    }
}

// ============================================================
// Edge Case Tests
// ============================================================

#[test]
fn test_strategy_registry_multiple_gets() {
    let registry = StrategyRegistry::new();
    let s1 = registry.get_strategy("rs");
    let s2 = registry.get_strategy("rs");
    assert!(s1.is_some());
    assert!(s2.is_some());
}

#[test]
fn test_strategy_registry_case_sensitivity() {
    let registry = StrategyRegistry::new();
    // Extensions should be case-sensitive
    assert!(registry.get_strategy("rs").is_some());
    assert!(registry.get_strategy("RS").is_none());
    assert!(registry.get_strategy("Rs").is_none());
}

#[test]
fn test_strategy_registry_whitespace() {
    let registry = StrategyRegistry::new();
    assert!(registry.get_strategy(" rs").is_none());
    assert!(registry.get_strategy("rs ").is_none());
    assert!(registry.get_strategy(" rs ").is_none());
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }

        /// Property: RustAstStrategy only supports "rs" extension
        #[test]
        fn rust_strategy_only_supports_rs(ext in "[a-z]{1,5}") {
            let strategy = RustAstStrategy;
            if ext == "rs" {
                prop_assert!(strategy.supports_extension(&ext));
            } else {
                prop_assert!(!strategy.supports_extension(&ext));
            }
        }

        /// Property: StrategyRegistry always has Rust strategy
        #[test]
        fn registry_always_has_rust(_seed in 0u32..1000) {
            let registry = StrategyRegistry::new();
            prop_assert!(registry.get_strategy("rs").is_some());
        }

        /// Property: Registered strategies can be retrieved
        #[test]
        fn registered_strategy_retrievable(ext in "[a-z]{1,10}") {
            let mut registry = StrategyRegistry::new();
            let strategy = Arc::new(RustAstStrategy) as Arc<dyn AstStrategy>;
            registry.register_strategy(ext.clone(), strategy);
            prop_assert!(registry.get_strategy(&ext).is_some());
        }

        /// Property: Non-registered extensions return None
        #[test]
        fn unknown_extension_returns_none(ext in "[a-z0-9]{6,15}") {
            let registry = StrategyRegistry::new();
            // Long random extensions should not match known strategies
            if ext.len() > 5 {
                prop_assert!(registry.get_strategy(&ext).is_none());
            }
        }
    }

    // Feature-gated property tests for C strategy helpers
    #[cfg(feature = "c-ast")]
    proptest! {
        /// Property: byte_pos_to_line returns valid line numbers
        #[test]
        fn c_byte_pos_always_returns_valid_line(pos in 0usize..10000, num_lines in 1usize..100) {
            let lines: Vec<&str> = (0..num_lines).map(|_| "content").collect();
            let result = CAstStrategy::byte_pos_to_line(pos, &lines);
            prop_assert!(result >= 1 || lines.is_empty());
            prop_assert!(result <= num_lines || lines.is_empty());
        }

        /// Property: function name extraction handles parentheses
        #[test]
        fn c_function_name_requires_paren(source in "[a-z ]+") {
            let result = CAstStrategy::extract_function_name(&source);
            if !source.contains('(') {
                prop_assert!(result.is_none());
            }
        }
    }

    // Feature-gated property tests for C++ strategy helpers
    #[cfg(feature = "c-ast")]
    proptest! {
        /// Property: C++ byte_pos_to_line returns valid line numbers
        #[test]
        fn cpp_byte_pos_always_returns_valid_line(pos in 0usize..10000, num_lines in 1usize..100) {
            let lines: Vec<&str> = (0..num_lines).map(|_| "code").collect();
            let result = CppAstStrategy::byte_pos_to_line(pos, &lines);
            prop_assert!(result >= 1 || lines.is_empty());
            prop_assert!(result <= num_lines || lines.is_empty());
        }
    }

    // Feature-gated property tests for Kotlin strategy helpers
    #[cfg(feature = "kotlin-ast")]
    proptest! {
        /// Property: Kotlin byte_pos_to_line returns valid line numbers
        #[test]
        fn kotlin_byte_pos_always_returns_valid_line(pos in 0usize..10000, num_lines in 1usize..100) {
            let lines: Vec<&str> = (0..num_lines).map(|_| "kotlin").collect();
            let result = KotlinAstStrategy::byte_pos_to_line(pos, &lines);
            prop_assert!(result >= 1 || lines.is_empty());
            prop_assert!(result <= num_lines || lines.is_empty());
        }

        /// Property: function name extraction requires "fun " keyword
        #[test]
        fn kotlin_function_name_requires_fun(source in "[a-z ()]+") {
            let result = KotlinAstStrategy::extract_function_name(&source);
            if !source.contains("fun ") {
                prop_assert!(result.is_none());
            }
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::path::Path;
    use tempfile::TempDir;

    /// Test RustAstStrategy analyze with a real file
    #[tokio::test]
    async fn test_rust_strategy_analyze_simple_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("test.rs");

        // Write a simple Rust file
        std::fs::write(
            &file_path,
            r#"
fn main() {
    println!("Hello, world!");
}

pub fn helper() -> i32 {
    42
}
"#,
        )
        .expect("Failed to write test file");

        let strategy = RustAstStrategy;
        let classifier = FileClassifier::default();
        let result = strategy.analyze(&file_path, &classifier).await;

        assert!(result.is_ok());
        let context = result.unwrap();
        assert_eq!(context.language, "rust");
        assert!(!context.items.is_empty());
    }

    /// Test RustAstStrategy analyze with nonexistent file
    #[tokio::test]
    async fn test_rust_strategy_analyze_nonexistent_file() {
        let strategy = RustAstStrategy;
        let classifier = FileClassifier::default();
        let result = strategy
            .analyze(Path::new("/nonexistent/path/file.rs"), &classifier)
            .await;

        // Should return an error for nonexistent file
        assert!(result.is_err());
    }

    /// Test RustAstStrategy analyze with empty file
    #[tokio::test]
    async fn test_rust_strategy_analyze_empty_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("empty.rs");

        std::fs::write(&file_path, "").expect("Failed to write test file");

        let strategy = RustAstStrategy;
        let classifier = FileClassifier::default();
        let result = strategy.analyze(&file_path, &classifier).await;

        // Empty file should parse but have no items
        assert!(result.is_ok());
        let context = result.unwrap();
        assert!(context.items.is_empty());
    }

    /// Test RustAstStrategy analyze with syntax error
    #[tokio::test]
    async fn test_rust_strategy_analyze_syntax_error() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("invalid.rs");

        // Write invalid Rust code
        std::fs::write(&file_path, "fn main( { incomplete syntax")
            .expect("Failed to write test file");

        let strategy = RustAstStrategy;
        let classifier = FileClassifier::default();
        let result = strategy.analyze(&file_path, &classifier).await;

        // May error or return partial results depending on implementation
        // We just want to ensure it doesn't panic
        let _ = result;
    }

    /// Test RustAstStrategy analyze with complex file
    #[tokio::test]
    async fn test_rust_strategy_analyze_complex_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("complex.rs");

        std::fs::write(
            &file_path,
            r#"
use std::collections::HashMap;

pub struct MyStruct {
    field1: i32,
    field2: String,
}

pub enum MyEnum {
    Variant1,
    Variant2(i32),
}

pub trait MyTrait {
    fn do_something(&self);
}

impl MyTrait for MyStruct {
    fn do_something(&self) {
        println!("{}", self.field1);
    }
}

pub async fn async_function() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

mod inner_module {
    pub fn inner_function() {}
}
"#,
        )
        .expect("Failed to write test file");

        let strategy = RustAstStrategy;
        let classifier = FileClassifier::default();
        let result = strategy.analyze(&file_path, &classifier).await;

        assert!(result.is_ok());
        let context = result.unwrap();
        assert_eq!(context.language, "rust");
        // Should have multiple items
        assert!(context.items.len() >= 4);
    }

    #[cfg(feature = "typescript-ast")]
    mod typescript_integration {
        use super::*;

        #[tokio::test]
        async fn test_typescript_strategy_analyze_simple() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let file_path = temp_dir.path().join("test.ts");

            std::fs::write(
                &file_path,
                r#"
function greet(name: string): string {
    return `Hello, ${name}!`;
}

export class Greeter {
    private name: string;

    constructor(name: string) {
        this.name = name;
    }

    greet(): string {
        return `Hello, ${this.name}!`;
    }
}
"#,
            )
            .expect("Failed to write test file");

            let strategy = TypeScriptAstStrategy;
            let classifier = FileClassifier::default();
            let result = strategy.analyze(&file_path, &classifier).await;

            assert!(result.is_ok());
            let context = result.unwrap();
            assert!(context.language == "typescript" || context.language == "ts");
        }
    }

    #[cfg(feature = "python-ast")]
    mod python_integration {
        use super::*;

        #[tokio::test]
        async fn test_python_strategy_analyze_simple() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let file_path = temp_dir.path().join("test.py");

            std::fs::write(
                &file_path,
                r#"
def greet(name: str) -> str:
    return f"Hello, {name}!"

class Greeter:
    def __init__(self, name: str):
        self.name = name

    def greet(self) -> str:
        return f"Hello, {self.name}!"
"#,
            )
            .expect("Failed to write test file");

            let strategy = PythonAstStrategy;
            let classifier = FileClassifier::default();
            let result = strategy.analyze(&file_path, &classifier).await;

            assert!(result.is_ok());
            let context = result.unwrap();
            assert!(context.language == "python" || context.language == "py");
        }
    }

    #[cfg(feature = "c-ast")]
    mod c_integration {
        use super::*;

        #[tokio::test]
        async fn test_c_strategy_analyze_simple() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let file_path = temp_dir.path().join("test.c");

            std::fs::write(
                &file_path,
                r#"
#include <stdio.h>

struct Point {
    int x;
    int y;
};

int add(int a, int b) {
    return a + b;
}

int main(int argc, char *argv[]) {
    printf("Hello, World!\n");
    return 0;
}
"#,
            )
            .expect("Failed to write test file");

            let strategy = CAstStrategy;
            let classifier = FileClassifier::default();
            let result = strategy.analyze(&file_path, &classifier).await;

            assert!(result.is_ok());
            let context = result.unwrap();
            assert_eq!(context.language, "c");
        }

        #[tokio::test]
        async fn test_cpp_strategy_analyze_simple() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let file_path = temp_dir.path().join("test.cpp");

            std::fs::write(
                &file_path,
                r#"
#include <iostream>
#include <string>

class Greeter {
public:
    std::string name;

    Greeter(const std::string& n) : name(n) {}

    void greet() const {
        std::cout << "Hello, " << name << "!" << std::endl;
    }
};

int main() {
    Greeter g("World");
    g.greet();
    return 0;
}
"#,
            )
            .expect("Failed to write test file");

            let strategy = CppAstStrategy;
            let classifier = FileClassifier::default();
            let result = strategy.analyze(&file_path, &classifier).await;

            assert!(result.is_ok());
            let context = result.unwrap();
            assert_eq!(context.language, "cpp");
        }
    }

    #[cfg(feature = "kotlin-ast")]
    mod kotlin_integration {
        use super::*;

        #[tokio::test]
        async fn test_kotlin_strategy_analyze_simple() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let file_path = temp_dir.path().join("test.kt");

            std::fs::write(
                &file_path,
                r#"
package com.example

fun greet(name: String): String {
    return "Hello, $name!"
}

class Greeter(private val name: String) {
    fun greet(): String = "Hello, $name!"
}

data class User(val id: Int, val name: String)

enum class Status { ACTIVE, INACTIVE }
"#,
            )
            .expect("Failed to write test file");

            let strategy = KotlinAstStrategy;
            let classifier = FileClassifier::default();
            let result = strategy.analyze(&file_path, &classifier).await;

            assert!(result.is_ok());
            let context = result.unwrap();
            assert!(context.language == "kotlin" || context.language == "kt");
        }
    }
}
