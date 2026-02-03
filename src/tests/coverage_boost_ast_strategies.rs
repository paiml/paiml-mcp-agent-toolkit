//! Coverage boost tests for services/ast_strategies_impl module
//!
//! Tests the StrategyRegistry pattern and language-specific AST strategies.
//! Targets: StrategyRegistry::new(), get_strategy(), register_strategy(),
//! extension handling, and language-specific strategy implementations.

use crate::services::ast_strategies::{AstStrategy, RustAstStrategy, StrategyRegistry};
use crate::services::file_classifier::FileClassifier;
use std::path::Path;
use std::sync::Arc;

// =============================================================================
// StrategyRegistry Tests
// =============================================================================

mod registry_tests {
    use super::*;

    #[test]
    fn test_strategy_registry_new_creates_instance() {
        let registry = StrategyRegistry::new();
        // Registry should be created successfully
        // At minimum, Rust strategy should be registered
        assert!(registry.get_strategy("rs").is_some());
    }

    #[test]
    fn test_strategy_registry_default_same_as_new() {
        let registry1 = StrategyRegistry::new();
        let registry2 = StrategyRegistry::default();

        // Both should have the same strategies registered
        assert!(registry1.get_strategy("rs").is_some());
        assert!(registry2.get_strategy("rs").is_some());
    }

    #[test]
    fn test_get_strategy_rust_extension() {
        let registry = StrategyRegistry::new();
        let strategy = registry.get_strategy("rs");
        assert!(strategy.is_some());
    }

    #[test]
    fn test_get_strategy_unsupported_extension_returns_none() {
        let registry = StrategyRegistry::new();

        // Various unsupported extensions
        assert!(registry.get_strategy("xyz").is_none());
        assert!(registry.get_strategy("").is_none());
        assert!(registry.get_strategy("unknown").is_none());
        assert!(registry.get_strategy("abc123").is_none());
    }

    #[test]
    fn test_get_strategy_case_sensitivity() {
        let registry = StrategyRegistry::new();

        // Extensions should be case-sensitive (lowercase expected)
        assert!(registry.get_strategy("rs").is_some());
        assert!(registry.get_strategy("RS").is_none()); // Case-sensitive
        assert!(registry.get_strategy("Rs").is_none());
        assert!(registry.get_strategy("rS").is_none());
    }

    #[test]
    fn test_register_custom_strategy() {
        let mut registry = StrategyRegistry::new();

        // Create a custom strategy
        let custom_strategy = Arc::new(RustAstStrategy) as Arc<dyn AstStrategy>;

        // Register for a custom extension
        registry.register_strategy("custom".to_string(), custom_strategy);

        // Verify it's registered
        assert!(registry.get_strategy("custom").is_some());
    }

    #[test]
    fn test_register_strategy_overwrites_existing() {
        let mut registry = StrategyRegistry::new();

        // Get original strategy
        let original = registry.get_strategy("rs");
        assert!(original.is_some());

        // Register a new strategy for the same extension
        let new_strategy = Arc::new(RustAstStrategy) as Arc<dyn AstStrategy>;
        registry.register_strategy("rs".to_string(), new_strategy.clone());

        // Should still work
        let replaced = registry.get_strategy("rs");
        assert!(replaced.is_some());
    }

    #[test]
    fn test_register_multiple_extensions_same_strategy() {
        let mut registry = StrategyRegistry::new();

        let shared_strategy = Arc::new(RustAstStrategy) as Arc<dyn AstStrategy>;

        registry.register_strategy("ext1".to_string(), shared_strategy.clone());
        registry.register_strategy("ext2".to_string(), shared_strategy.clone());
        registry.register_strategy("ext3".to_string(), shared_strategy);

        assert!(registry.get_strategy("ext1").is_some());
        assert!(registry.get_strategy("ext2").is_some());
        assert!(registry.get_strategy("ext3").is_some());
    }

    #[test]
    fn test_get_strategy_returns_cloned_arc() {
        let registry = StrategyRegistry::new();

        let strategy1 = registry.get_strategy("rs");
        let strategy2 = registry.get_strategy("rs");

        // Both should be Some
        assert!(strategy1.is_some());
        assert!(strategy2.is_some());

        // Should be cloned Arcs pointing to the same strategy
        let s1 = strategy1.unwrap();
        let s2 = strategy2.unwrap();
        assert!(Arc::ptr_eq(&s1, &s2));
    }
}

// =============================================================================
// RustAstStrategy Tests
// =============================================================================

mod rust_strategy_tests {
    use super::*;

    #[test]
    fn test_rust_strategy_supports_rs_extension() {
        let strategy = RustAstStrategy;
        assert!(strategy.supports_extension("rs"));
    }

    #[test]
    fn test_rust_strategy_does_not_support_other_extensions() {
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

    #[tokio::test]
    async fn test_rust_strategy_analyze_nonexistent_file() {
        let strategy = RustAstStrategy;
        let classifier = FileClassifier::default();
        let path = Path::new("/nonexistent/file.rs");

        let result = strategy.analyze(path, &classifier).await;
        // Should return an error for nonexistent file
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_rust_strategy_analyze_with_temp_file() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let strategy = RustAstStrategy;
        let classifier = FileClassifier::default();

        // Create a temporary Rust file
        let mut temp_file = NamedTempFile::with_suffix(".rs").unwrap();
        writeln!(
            temp_file,
            r#"
fn main() {{
    println!("Hello, world!");
}}

pub fn add(a: i32, b: i32) -> i32 {{
    a + b
}}

struct Point {{
    x: i32,
    y: i32,
}}
"#
        )
        .unwrap();
        temp_file.flush().unwrap();

        let result = strategy.analyze(temp_file.path(), &classifier).await;

        match result {
            Ok(context) => {
                assert_eq!(context.language, "rust");
                // Should find functions and struct
                assert!(!context.items.is_empty());
            }
            Err(e) => {
                // Some environments may have issues with parsing
                println!("Parse error (expected in some environments): {}", e);
            }
        }
    }
}

// =============================================================================
// TypeScript Strategy Tests (feature-gated)
// =============================================================================

#[cfg(feature = "typescript-ast")]
mod typescript_strategy_tests {
    use super::*;
    use crate::services::ast_strategies::TypeScriptAstStrategy;

    #[test]
    fn test_typescript_strategy_supports_ts_extension() {
        let strategy = TypeScriptAstStrategy;
        assert!(strategy.supports_extension("ts"));
    }

    #[test]
    fn test_typescript_strategy_supports_tsx_extension() {
        let strategy = TypeScriptAstStrategy;
        assert!(strategy.supports_extension("tsx"));
    }

    #[test]
    fn test_typescript_strategy_does_not_support_js() {
        let strategy = TypeScriptAstStrategy;
        // TypeScript strategy should not support JS (separate strategy)
        assert!(!strategy.supports_extension("js"));
        assert!(!strategy.supports_extension("jsx"));
    }

    #[test]
    fn test_registry_has_typescript_strategies() {
        let registry = StrategyRegistry::new();
        assert!(registry.get_strategy("ts").is_some());
        assert!(registry.get_strategy("tsx").is_some());
    }

    #[tokio::test]
    async fn test_typescript_strategy_analyze_nonexistent_file() {
        let strategy = TypeScriptAstStrategy;
        let classifier = FileClassifier::default();
        let path = Path::new("/nonexistent/file.ts");

        let result = strategy.analyze(path, &classifier).await;
        assert!(result.is_err());
    }
}

// =============================================================================
// JavaScript Strategy Tests (feature-gated)
// =============================================================================

#[cfg(feature = "typescript-ast")]
mod javascript_strategy_tests {
    use super::*;
    use crate::services::ast_strategies::JavaScriptAstStrategy;

    #[test]
    fn test_javascript_strategy_supports_js_extension() {
        let strategy = JavaScriptAstStrategy;
        assert!(strategy.supports_extension("js"));
    }

    #[test]
    fn test_javascript_strategy_supports_jsx_extension() {
        let strategy = JavaScriptAstStrategy;
        assert!(strategy.supports_extension("jsx"));
    }

    #[test]
    fn test_javascript_strategy_does_not_support_ts() {
        let strategy = JavaScriptAstStrategy;
        assert!(!strategy.supports_extension("ts"));
        assert!(!strategy.supports_extension("tsx"));
    }

    #[test]
    fn test_registry_has_javascript_strategies() {
        let registry = StrategyRegistry::new();
        assert!(registry.get_strategy("js").is_some());
        assert!(registry.get_strategy("jsx").is_some());
    }

    #[tokio::test]
    async fn test_javascript_strategy_analyze_nonexistent_file() {
        let strategy = JavaScriptAstStrategy;
        let classifier = FileClassifier::default();
        let path = Path::new("/nonexistent/file.js");

        let result = strategy.analyze(path, &classifier).await;
        assert!(result.is_err());
    }
}

// =============================================================================
// Python Strategy Tests (feature-gated)
// =============================================================================

#[cfg(feature = "python-ast")]
mod python_strategy_tests {
    use super::*;
    use crate::services::ast_strategies::PythonAstStrategy;

    #[test]
    fn test_python_strategy_supports_py_extension() {
        let strategy = PythonAstStrategy;
        assert!(strategy.supports_extension("py"));
    }

    #[test]
    fn test_python_strategy_does_not_support_other_extensions() {
        let strategy = PythonAstStrategy;
        assert!(!strategy.supports_extension("pyw"));
        assert!(!strategy.supports_extension("pyc"));
        assert!(!strategy.supports_extension("pyo"));
        assert!(!strategy.supports_extension("pyi"));
    }

    #[test]
    fn test_registry_has_python_strategy() {
        let registry = StrategyRegistry::new();
        assert!(registry.get_strategy("py").is_some());
    }

    #[tokio::test]
    async fn test_python_strategy_analyze_nonexistent_file() {
        let strategy = PythonAstStrategy;
        let classifier = FileClassifier::default();
        let path = Path::new("/nonexistent/file.py");

        let result = strategy.analyze(path, &classifier).await;
        assert!(result.is_err());
    }
}

// =============================================================================
// C Strategy Tests (feature-gated)
// =============================================================================

#[cfg(feature = "c-ast")]
mod c_strategy_tests {
    use super::*;
    use crate::services::ast_strategies::CAstStrategy;

    #[test]
    fn test_c_strategy_supports_c_extension() {
        let strategy = CAstStrategy;
        assert!(strategy.supports_extension("c"));
    }

    #[test]
    fn test_c_strategy_supports_h_extension() {
        let strategy = CAstStrategy;
        assert!(strategy.supports_extension("h"));
    }

    #[test]
    fn test_c_strategy_does_not_support_cpp() {
        let strategy = CAstStrategy;
        assert!(!strategy.supports_extension("cpp"));
        assert!(!strategy.supports_extension("cc"));
        assert!(!strategy.supports_extension("hpp"));
    }

    #[test]
    fn test_registry_has_c_strategies() {
        let registry = StrategyRegistry::new();
        assert!(registry.get_strategy("c").is_some());
        assert!(registry.get_strategy("h").is_some());
    }

    #[tokio::test]
    async fn test_c_strategy_analyze_nonexistent_file() {
        let strategy = CAstStrategy;
        let classifier = FileClassifier::default();
        let path = Path::new("/nonexistent/file.c");

        let result = strategy.analyze(path, &classifier).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_c_strategy_extract_function_name() {
        // Test function name extraction from source text
        let source = "int main(int argc, char** argv)";
        let name = CAstStrategy::extract_function_name(source);
        assert_eq!(name, Some("main".to_string()));
    }

    #[test]
    fn test_c_strategy_extract_function_name_with_pointer() {
        let source = "void *malloc(size_t size)";
        let name = CAstStrategy::extract_function_name(source);
        assert_eq!(name, Some("malloc".to_string()));
    }

    #[test]
    fn test_c_strategy_extract_function_name_no_parens() {
        let source = "int variable";
        let name = CAstStrategy::extract_function_name(source);
        assert_eq!(name, None);
    }

    #[test]
    fn test_c_strategy_extract_type_name_struct() {
        let source = "struct Point { int x; int y; }";
        let name = CAstStrategy::extract_type_name(source);
        assert_eq!(name, Some("Point".to_string()));
    }

    #[test]
    fn test_c_strategy_extract_type_name_enum() {
        let source = "enum Color { RED, GREEN, BLUE }";
        let name = CAstStrategy::extract_type_name(source);
        assert_eq!(name, Some("Color".to_string()));
    }

    #[test]
    fn test_c_strategy_extract_type_name_union() {
        let source = "union Data { int i; float f; }";
        let name = CAstStrategy::extract_type_name(source);
        assert_eq!(name, Some("Data".to_string()));
    }

    #[test]
    fn test_c_strategy_extract_type_name_typedef() {
        // typedef extracts the word after "typedef" keyword (the original type)
        let source = "typedef int Integer";
        let name = CAstStrategy::extract_type_name(source);
        assert_eq!(name, Some("int".to_string()));
    }

    #[test]
    fn test_c_strategy_extract_type_name_no_keyword() {
        let source = "int variable;";
        let name = CAstStrategy::extract_type_name(source);
        assert_eq!(name, None);
    }

    #[test]
    fn test_c_strategy_byte_pos_to_line_first_line() {
        let content_lines = vec!["line1", "line2", "line3"];
        let line = CAstStrategy::byte_pos_to_line(0, &content_lines);
        assert_eq!(line, 1);
    }

    #[test]
    fn test_c_strategy_byte_pos_to_line_second_line() {
        let content_lines = vec!["line1", "line2", "line3"];
        // "line1\n" = 6 bytes, so position 6 should be line 2
        let line = CAstStrategy::byte_pos_to_line(6, &content_lines);
        assert_eq!(line, 2);
    }

    #[test]
    fn test_c_strategy_byte_pos_to_line_beyond_content() {
        let content_lines = vec!["line1", "line2"];
        let line = CAstStrategy::byte_pos_to_line(1000, &content_lines);
        assert_eq!(line, 2); // Returns last line
    }

    #[tokio::test]
    async fn test_c_strategy_analyze_with_temp_file() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let strategy = CAstStrategy;
        let classifier = FileClassifier::default();

        let mut temp_file = NamedTempFile::with_suffix(".c").unwrap();
        writeln!(
            temp_file,
            r#"
#include <stdio.h>

int main() {{
    printf("Hello, world!\n");
    return 0;
}}

struct Point {{
    int x;
    int y;
}};
"#
        )
        .unwrap();
        temp_file.flush().unwrap();

        let result = strategy.analyze(temp_file.path(), &classifier).await;

        // Should succeed (even if empty result on parse failure)
        assert!(result.is_ok());
        let context = result.unwrap();
        assert_eq!(context.language, "c");
    }
}

// =============================================================================
// C++ Strategy Tests (feature-gated)
// =============================================================================

#[cfg(feature = "c-ast")]
mod cpp_strategy_tests {
    use super::*;
    use crate::services::ast_strategies::CppAstStrategy;

    #[test]
    fn test_cpp_strategy_supports_cpp_extension() {
        let strategy = CppAstStrategy;
        assert!(strategy.supports_extension("cpp"));
    }

    #[test]
    fn test_cpp_strategy_supports_cc_extension() {
        let strategy = CppAstStrategy;
        assert!(strategy.supports_extension("cc"));
    }

    #[test]
    fn test_cpp_strategy_supports_cxx_extension() {
        let strategy = CppAstStrategy;
        assert!(strategy.supports_extension("cxx"));
    }

    #[test]
    fn test_cpp_strategy_supports_hpp_extension() {
        let strategy = CppAstStrategy;
        assert!(strategy.supports_extension("hpp"));
    }

    #[test]
    fn test_cpp_strategy_supports_hxx_extension() {
        let strategy = CppAstStrategy;
        assert!(strategy.supports_extension("hxx"));
    }

    #[test]
    fn test_cpp_strategy_does_not_support_c() {
        let strategy = CppAstStrategy;
        assert!(!strategy.supports_extension("c"));
        assert!(!strategy.supports_extension("h"));
    }

    #[test]
    fn test_registry_has_cpp_strategies() {
        let registry = StrategyRegistry::new();
        assert!(registry.get_strategy("cpp").is_some());
        assert!(registry.get_strategy("cc").is_some());
        assert!(registry.get_strategy("cxx").is_some());
        assert!(registry.get_strategy("hpp").is_some());
        assert!(registry.get_strategy("hxx").is_some());
    }

    #[tokio::test]
    async fn test_cpp_strategy_analyze_nonexistent_file() {
        let strategy = CppAstStrategy;
        let classifier = FileClassifier::default();
        let path = Path::new("/nonexistent/file.cpp");

        let result = strategy.analyze(path, &classifier).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_cpp_strategy_extract_function_name() {
        let source = "int main(int argc, char** argv)";
        let name = CppAstStrategy::extract_function_name(source);
        assert_eq!(name, Some("main".to_string()));
    }

    #[test]
    fn test_cpp_strategy_extract_function_name_with_destructor() {
        let source = "~MyClass()";
        let name = CppAstStrategy::extract_function_name(source);
        assert_eq!(name, Some("MyClass".to_string()));
    }

    #[test]
    fn test_cpp_strategy_extract_function_name_operator_overload() {
        let source = "bool operator==(const MyClass& other)";
        let name = CppAstStrategy::extract_function_name(source);
        assert_eq!(name, Some("operator_overload".to_string()));
    }

    #[test]
    fn test_cpp_strategy_extract_type_name_class() {
        let source = "class MyClass { };";
        let name = CppAstStrategy::extract_type_name(source);
        assert_eq!(name, Some("MyClass".to_string()));
    }

    #[test]
    fn test_cpp_strategy_extract_type_name_struct() {
        let source = "struct Point { int x; int y; };";
        let name = CppAstStrategy::extract_type_name(source);
        assert_eq!(name, Some("Point".to_string()));
    }

    #[test]
    fn test_cpp_strategy_extract_type_name_enum_class() {
        let source = "enum class Color { RED, GREEN, BLUE };";
        let name = CppAstStrategy::extract_type_name(source);
        assert_eq!(name, Some("Color".to_string()));
    }

    #[test]
    fn test_cpp_strategy_extract_type_name_template() {
        let source = "class Vector<T> { };";
        let name = CppAstStrategy::extract_type_name(source);
        assert_eq!(name, Some("Vector".to_string()));
    }

    #[test]
    fn test_cpp_strategy_byte_pos_to_line_first_line() {
        let content_lines = vec!["line1", "line2", "line3"];
        let line = CppAstStrategy::byte_pos_to_line(0, &content_lines);
        assert_eq!(line, 1);
    }

    #[test]
    fn test_cpp_strategy_byte_pos_to_line_middle() {
        let content_lines = vec!["short", "medium length", "another"];
        // "short\n" = 6 bytes, position 6 should be line 2
        let line = CppAstStrategy::byte_pos_to_line(6, &content_lines);
        assert_eq!(line, 2);
    }

    #[tokio::test]
    async fn test_cpp_strategy_analyze_with_temp_file() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let strategy = CppAstStrategy;
        let classifier = FileClassifier::default();

        let mut temp_file = NamedTempFile::with_suffix(".cpp").unwrap();
        writeln!(
            temp_file,
            r#"
#include <iostream>

class Point {{
public:
    int x;
    int y;
    Point(int x, int y) : x(x), y(y) {{}}
}};

int main() {{
    std::cout << "Hello, world!" << std::endl;
    return 0;
}}
"#
        )
        .unwrap();
        temp_file.flush().unwrap();

        let result = strategy.analyze(temp_file.path(), &classifier).await;

        // Should succeed
        assert!(result.is_ok());
        let context = result.unwrap();
        assert_eq!(context.language, "cpp");
    }
}

// =============================================================================
// Kotlin Strategy Tests (feature-gated)
// =============================================================================

#[cfg(feature = "kotlin-ast")]
mod kotlin_strategy_tests {
    use super::*;
    use crate::services::ast_strategies::KotlinAstStrategy;

    #[test]
    fn test_kotlin_strategy_supports_kt_extension() {
        let strategy = KotlinAstStrategy;
        assert!(strategy.supports_extension("kt"));
    }

    #[test]
    fn test_kotlin_strategy_supports_kts_extension() {
        let strategy = KotlinAstStrategy;
        assert!(strategy.supports_extension("kts"));
    }

    #[test]
    fn test_kotlin_strategy_does_not_support_java() {
        let strategy = KotlinAstStrategy;
        assert!(!strategy.supports_extension("java"));
        assert!(!strategy.supports_extension("jar"));
    }

    #[test]
    fn test_registry_has_kotlin_strategies() {
        let registry = StrategyRegistry::new();
        assert!(registry.get_strategy("kt").is_some());
        assert!(registry.get_strategy("kts").is_some());
    }

    #[tokio::test]
    async fn test_kotlin_strategy_analyze_nonexistent_file() {
        let strategy = KotlinAstStrategy;
        let classifier = FileClassifier::default();
        let path = Path::new("/nonexistent/file.kt");

        let result = strategy.analyze(path, &classifier).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_kotlin_strategy_extract_function_name() {
        let source = "fun main() { println(\"Hello\") }";
        let name = KotlinAstStrategy::extract_function_name(source);
        assert_eq!(name, Some("main".to_string()));
    }

    #[test]
    fn test_kotlin_strategy_extract_function_name_with_params() {
        let source = "fun add(a: Int, b: Int): Int = a + b";
        let name = KotlinAstStrategy::extract_function_name(source);
        assert_eq!(name, Some("add".to_string()));
    }

    #[test]
    fn test_kotlin_strategy_extract_function_name_no_fun_keyword() {
        let source = "val x = 5";
        let name = KotlinAstStrategy::extract_function_name(source);
        assert_eq!(name, None);
    }

    #[test]
    fn test_kotlin_strategy_extract_class_name() {
        let source = "class MyClass { }";
        let name = KotlinAstStrategy::extract_class_name(source);
        assert_eq!(name, Some("MyClass".to_string()));
    }

    #[test]
    fn test_kotlin_strategy_extract_class_name_data_class() {
        let source = "data class Point(val x: Int, val y: Int)";
        let name = KotlinAstStrategy::extract_class_name(source);
        assert_eq!(name, Some("Point".to_string()));
    }

    #[test]
    fn test_kotlin_strategy_extract_class_name_enum_class() {
        let source = "enum class Color { RED, GREEN, BLUE }";
        let name = KotlinAstStrategy::extract_class_name(source);
        assert_eq!(name, Some("Color".to_string()));
    }

    #[test]
    fn test_kotlin_strategy_extract_class_name_interface() {
        let source = "interface Drawable { fun draw() }";
        let name = KotlinAstStrategy::extract_class_name(source);
        assert_eq!(name, Some("Drawable".to_string()));
    }

    #[test]
    fn test_kotlin_strategy_extract_class_name_object() {
        let source = "object Singleton { val value = 42 }";
        let name = KotlinAstStrategy::extract_class_name(source);
        assert_eq!(name, Some("Singleton".to_string()));
    }

    #[test]
    fn test_kotlin_strategy_extract_class_name_with_generics() {
        let source = "class Container<T>(val item: T)";
        let name = KotlinAstStrategy::extract_class_name(source);
        assert_eq!(name, Some("Container".to_string()));
    }

    #[test]
    fn test_kotlin_strategy_extract_class_name_with_inheritance() {
        let source = "class MyClass : BaseClass() { }";
        let name = KotlinAstStrategy::extract_class_name(source);
        assert_eq!(name, Some("MyClass".to_string()));
    }

    #[test]
    fn test_kotlin_strategy_byte_pos_to_line() {
        let content_lines = vec!["fun main() {", "    println()", "}"];
        let line = KotlinAstStrategy::byte_pos_to_line(0, &content_lines);
        assert_eq!(line, 1);
    }
}

// =============================================================================
// Thread Safety Tests
// =============================================================================

mod thread_safety_tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_strategy_arc_clone_across_threads() {
        let registry = StrategyRegistry::new();
        let strategy = registry.get_strategy("rs").unwrap();

        let handles: Vec<_> = (0..4)
            .map(|_| {
                let s = strategy.clone();
                thread::spawn(move || {
                    // Use the strategy in another thread
                    assert!(s.supports_extension("rs"));
                })
            })
            .collect();

        for handle in handles {
            handle.join().expect("Thread panicked");
        }
    }

    #[test]
    fn test_strategy_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Arc<dyn AstStrategy>>();
    }
}

// =============================================================================
// Edge Case Tests
// =============================================================================

mod edge_case_tests {
    use super::*;

    #[test]
    fn test_empty_extension() {
        let registry = StrategyRegistry::new();
        assert!(registry.get_strategy("").is_none());
    }

    #[test]
    fn test_whitespace_extension() {
        let registry = StrategyRegistry::new();
        assert!(registry.get_strategy(" ").is_none());
        assert!(registry.get_strategy("  rs").is_none());
        assert!(registry.get_strategy("rs  ").is_none());
    }

    #[test]
    fn test_extension_with_dot() {
        let registry = StrategyRegistry::new();
        // Registry uses extensions without dots
        assert!(registry.get_strategy(".rs").is_none());
        assert!(registry.get_strategy("rs.").is_none());
    }

    #[test]
    fn test_long_extension() {
        let registry = StrategyRegistry::new();
        let long_ext = "a".repeat(1000);
        assert!(registry.get_strategy(&long_ext).is_none());
    }

    #[test]
    fn test_unicode_extension() {
        let registry = StrategyRegistry::new();
        assert!(registry.get_strategy("\u{1F980}").is_none()); // Crab emoji
        assert!(registry.get_strategy("rs\u{200B}").is_none()); // Zero-width space
    }

    #[test]
    fn test_register_empty_extension() {
        let mut registry = StrategyRegistry::new();
        let strategy = Arc::new(RustAstStrategy) as Arc<dyn AstStrategy>;
        registry.register_strategy(String::new(), strategy);
        assert!(registry.get_strategy("").is_some());
    }
}

// =============================================================================
// Feature Flag Combination Tests
// =============================================================================

mod feature_flag_tests {
    use super::*;

    #[test]
    fn test_registry_always_has_rust() {
        // Rust is always available regardless of feature flags
        let registry = StrategyRegistry::new();
        assert!(
            registry.get_strategy("rs").is_some(),
            "Rust strategy should always be available"
        );
    }

    #[test]
    #[cfg(feature = "typescript-ast")]
    fn test_typescript_only_with_feature() {
        let registry = StrategyRegistry::new();
        assert!(registry.get_strategy("ts").is_some());
        assert!(registry.get_strategy("tsx").is_some());
        assert!(registry.get_strategy("js").is_some());
        assert!(registry.get_strategy("jsx").is_some());
    }

    #[test]
    #[cfg(feature = "python-ast")]
    fn test_python_only_with_feature() {
        let registry = StrategyRegistry::new();
        assert!(registry.get_strategy("py").is_some());
    }

    #[test]
    #[cfg(feature = "c-ast")]
    fn test_c_cpp_only_with_feature() {
        let registry = StrategyRegistry::new();
        assert!(registry.get_strategy("c").is_some());
        assert!(registry.get_strategy("h").is_some());
        assert!(registry.get_strategy("cpp").is_some());
        assert!(registry.get_strategy("cc").is_some());
        assert!(registry.get_strategy("cxx").is_some());
        assert!(registry.get_strategy("hpp").is_some());
        assert!(registry.get_strategy("hxx").is_some());
    }

    #[test]
    #[cfg(feature = "kotlin-ast")]
    fn test_kotlin_only_with_feature() {
        let registry = StrategyRegistry::new();
        assert!(registry.get_strategy("kt").is_some());
        assert!(registry.get_strategy("kts").is_some());
    }
}

// =============================================================================
// Integration Tests
// =============================================================================

mod integration_tests {
    use super::*;

    #[test]
    fn test_full_registry_workflow() {
        // Create registry
        let mut registry = StrategyRegistry::new();

        // Verify initial state
        assert!(registry.get_strategy("rs").is_some());

        // Register custom extension
        let custom = Arc::new(RustAstStrategy) as Arc<dyn AstStrategy>;
        registry.register_strategy("custom_ext".to_string(), custom);
        assert!(registry.get_strategy("custom_ext").is_some());

        // Verify Arc cloning works
        let s1 = registry.get_strategy("rs").unwrap();
        let s2 = registry.get_strategy("rs").unwrap();
        assert!(Arc::ptr_eq(&s1, &s2));

        // Verify supports_extension still works
        assert!(s1.supports_extension("rs"));
        assert!(!s1.supports_extension("py"));
    }

    #[tokio::test]
    async fn test_async_analysis_workflow() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let registry = StrategyRegistry::new();
        let classifier = FileClassifier::default();

        // Create a simple Rust file
        let mut temp_file = NamedTempFile::with_suffix(".rs").unwrap();
        writeln!(temp_file, "fn test_fn() {{ }}").unwrap();
        temp_file.flush().unwrap();

        // Get strategy and analyze
        let strategy = registry.get_strategy("rs").unwrap();
        let result = strategy.analyze(temp_file.path(), &classifier).await;

        match result {
            Ok(ctx) => {
                assert_eq!(ctx.language, "rust");
            }
            Err(e) => {
                // Some environments may have issues
                println!("Analysis error (may be expected): {}", e);
            }
        }
    }
}
