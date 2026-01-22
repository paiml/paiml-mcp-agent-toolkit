//! Tests for deep context - Part 3: Coverage tests (language detection, extraction)
//! Extracted for file health compliance (CB-040)

use super::*;

/// Comprehensive coverage tests for deep_context helper functions
/// Toyota Way: EXTREME TDD for 95% coverage

mod coverage_tests {
    use super::*;
    use std::path::PathBuf;

    // LANGUAGE DETECTION TESTS

    #[test]
    fn test_detect_language_rust() {
        assert_eq!(detect_language(Path::new("test.rs")), "rust");
    }

    #[test]
    fn test_detect_language_typescript_variants() {
        assert_eq!(detect_language(Path::new("test.ts")), "typescript");
        assert_eq!(detect_language(Path::new("test.tsx")), "typescript");
    }

    #[test]
    fn test_detect_language_javascript_variants() {
        assert_eq!(detect_language(Path::new("test.js")), "javascript");
        assert_eq!(detect_language(Path::new("test.jsx")), "javascript");
        assert_eq!(detect_language(Path::new("test.mjs")), "javascript");
        assert_eq!(detect_language(Path::new("test.cjs")), "javascript");
    }

    #[test]
    fn test_detect_language_python() {
        assert_eq!(detect_language(Path::new("test.py")), "python");
        assert_eq!(detect_language(Path::new("test.pyi")), "python");
    }

    #[test]
    fn test_detect_language_go() {
        assert_eq!(detect_language(Path::new("test.go")), "go");
    }

    #[test]
    fn test_detect_language_c_cpp() {
        assert_eq!(detect_language(Path::new("test.c")), "c");
        assert_eq!(detect_language(Path::new("test.h")), "c");
        assert_eq!(detect_language(Path::new("test.cpp")), "cpp");
        assert_eq!(detect_language(Path::new("test.cc")), "cpp");
        assert_eq!(detect_language(Path::new("test.cxx")), "cpp");
        assert_eq!(detect_language(Path::new("test.hpp")), "cpp");
        assert_eq!(detect_language(Path::new("test.hxx")), "cpp");
    }

    #[test]
    fn test_detect_language_jvm() {
        assert_eq!(detect_language(Path::new("Test.java")), "java");
        assert_eq!(detect_language(Path::new("Test.kt")), "kotlin");
        assert_eq!(detect_language(Path::new("build.kts")), "kotlin");
    }

    #[test]
    fn test_detect_language_dotnet() {
        assert_eq!(detect_language(Path::new("Test.cs")), "csharp");
    }

    #[test]
    fn test_detect_language_scripting() {
        assert_eq!(detect_language(Path::new("script.sh")), "bash");
        assert_eq!(detect_language(Path::new("script.bash")), "bash");
        assert_eq!(detect_language(Path::new("test.rb")), "ruby");
    }

    #[test]
    fn test_detect_language_functional() {
        assert_eq!(detect_language(Path::new("test.ex")), "elixir");
        assert_eq!(detect_language(Path::new("test.exs")), "elixir");
        assert_eq!(detect_language(Path::new("test.erl")), "erlang");
        assert_eq!(detect_language(Path::new("test.hrl")), "erlang");
        assert_eq!(detect_language(Path::new("test.hs")), "haskell");
        assert_eq!(detect_language(Path::new("test.lhs")), "haskell");
        assert_eq!(detect_language(Path::new("test.ml")), "ocaml");
        assert_eq!(detect_language(Path::new("test.mli")), "ocaml");
    }

    #[test]
    fn test_detect_language_swift() {
        assert_eq!(detect_language(Path::new("test.swift")), "swift");
    }

    #[test]
    fn test_detect_language_wasm() {
        assert_eq!(detect_language(Path::new("test.wat")), "wasm");
        assert_eq!(detect_language(Path::new("test.wasm")), "wasm");
    }

    #[test]
    fn test_detect_language_unknown() {
        assert_eq!(detect_language(Path::new("test.xyz")), "unknown");
        assert_eq!(detect_language(Path::new("test")), "unknown");
    }

    // EXTRACTION FUNCTION TESTS

    #[test]
    fn test_extract_function_name_valid() {
        assert_eq!(
            extract_function_name("fn test_function(a: i32) {"),
            "test_function"
        );
        assert_eq!(extract_function_name("fn main() {"), "main");
        assert_eq!(
            extract_function_name("pub fn public_func() {"),
            "public_func"
        );
    }

    #[test]
    fn test_extract_function_name_no_fn_keyword() {
        let result = extract_function_name("let x = 5;");
        assert!(result.is_empty() || result.capacity() >= 1024);
    }

    #[test]
    fn test_extract_function_name_no_parenthesis() {
        let result = extract_function_name("fn incomplete");
        assert!(result.is_empty() || result.capacity() >= 1024);
    }

    #[test]
    fn test_extract_struct_name_valid() {
        assert_eq!(extract_struct_name("struct MyStruct {"), "MyStruct");
        assert_eq!(
            extract_struct_name("pub struct PublicStruct {"),
            "PublicStruct"
        );
        assert_eq!(extract_struct_name("struct Simple;"), "Simple;");
    }

    #[test]
    fn test_extract_struct_name_no_struct() {
        let result = extract_struct_name("let x = 5;");
        assert!(result.is_empty() || result.capacity() >= 1024);
    }

    #[test]
    fn test_extract_js_function_name_valid() {
        assert_eq!(extract_js_function_name("function myFunc() {"), "myFunc");
        assert_eq!(extract_js_function_name("function test(a, b) {"), "test");
    }

    #[test]
    fn test_extract_js_function_name_invalid() {
        let result = extract_js_function_name("const x = 5;");
        assert!(result.is_empty() || result.capacity() >= 1024);
    }

    #[test]
    fn test_extract_class_name_valid() {
        assert_eq!(extract_class_name("class MyClass {"), "MyClass");
        assert_eq!(
            extract_class_name("export class ExportedClass {"),
            "ExportedClass"
        );
    }

    #[test]
    fn test_extract_class_name_invalid() {
        let result = extract_class_name("const x = 5;");
        assert!(result.is_empty() || result.capacity() >= 1024);
    }

    #[test]
    fn test_extract_python_function_name_valid() {
        assert_eq!(
            extract_python_function_name("def my_func(self):"),
            "my_func"
        );
        assert_eq!(
            extract_python_function_name("def _private_func():"),
            "_private_func"
        );
    }

    #[test]
    fn test_extract_python_function_name_invalid() {
        let result = extract_python_function_name("x = 5");
        assert!(result.is_empty() || result.capacity() >= 1024);
    }

    #[test]
    fn test_extract_python_class_name_valid() {
        assert_eq!(extract_python_class_name("class MyClass:"), "MyClass");
        assert_eq!(extract_python_class_name("class MyClass(Base):"), "MyClass");
        assert_eq!(
            extract_python_class_name("class _PrivateClass:"),
            "_PrivateClass"
        );
    }

    #[test]
    fn test_extract_python_class_name_invalid() {
        let result = extract_python_class_name("x = 5");
        assert!(result.is_empty() || result.capacity() >= 1024);
    }

    // FUNCTION CALL DETECTION TESTS

    #[test]
    fn test_is_function_called_in_file_true() {
        let lines = vec!["fn test() {}", "let x = test();", "}"];
        assert!(is_function_called_in_file(&lines, "test"));
    }

    #[test]
    fn test_is_function_called_in_file_false() {
        // Test that function is not called - use different function name to avoid matching definition
        let lines = vec!["let x = other();", "let y = another();"];
        assert!(!is_function_called_in_file(&lines, "missing_func"));
    }

    #[test]
    fn test_is_type_used_in_file_new() {
        let lines = vec!["struct MyType {}", "let x = new MyType();"];
        assert!(is_type_used_in_file(&lines, "MyType"));
    }

    #[test]
    fn test_is_type_used_in_file_type_annotation() {
        let lines = vec!["struct MyType {}", "fn test(x: MyType) {}"];
        assert!(is_type_used_in_file(&lines, "MyType"));
    }

    #[test]
    fn test_is_type_used_in_file_generic() {
        let lines = vec!["struct MyType {}", "fn test() -> Vec<MyType> {}"];
        assert!(is_type_used_in_file(&lines, "MyType"));
    }

    #[test]
    fn test_is_type_used_in_file_false() {
        let lines = vec!["struct MyType {}", "fn test() {}"];
        assert!(!is_type_used_in_file(&lines, "MyType"));
    }

    // INDICATOR AND EMOJI TESTS

    #[test]
    fn test_overall_health_emoji_excellent() {
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        assert_eq!(analyzer.overall_health_emoji(85.0), "✅");
        assert_eq!(analyzer.overall_health_emoji(100.0), "✅");
    }

    #[test]
    fn test_overall_health_emoji_warning() {
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        assert_eq!(analyzer.overall_health_emoji(65.0), "⚠️");
        assert_eq!(analyzer.overall_health_emoji(79.9), "⚠️");
    }

    #[test]
    fn test_overall_health_emoji_critical() {
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        assert_eq!(analyzer.overall_health_emoji(50.0), "❌");
        assert_eq!(analyzer.overall_health_emoji(0.0), "❌");
    }

    #[test]
    fn test_get_priority_emoji_all() {
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        assert_eq!(analyzer.get_priority_emoji(&Priority::Critical), "🔴");
        assert_eq!(analyzer.get_priority_emoji(&Priority::High), "🟡");
        assert_eq!(analyzer.get_priority_emoji(&Priority::Medium), "🔵");
        assert_eq!(analyzer.get_priority_emoji(&Priority::Low), "⚪");
    }

    #[test]
    fn test_get_big_o_emoji_all() {
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        assert_eq!(analyzer.get_big_o_emoji("O(1)"), "🎯");
        assert_eq!(analyzer.get_big_o_emoji("O(log n)"), "⚡");
        assert_eq!(analyzer.get_big_o_emoji("O(n)"), "📊");
        assert_eq!(analyzer.get_big_o_emoji("O(n log n)"), "📈");
        assert_eq!(analyzer.get_big_o_emoji("O(n²)"), "⚠️");
        assert_eq!(analyzer.get_big_o_emoji("O(n³)"), "🚨");
        assert_eq!(analyzer.get_big_o_emoji("O(2ⁿ)"), "💥");
        assert_eq!(analyzer.get_big_o_emoji("O(n!)"), "💥");
        assert_eq!(analyzer.get_big_o_emoji("unknown"), "❓");
    }

    #[test]
    fn test_determine_complexity_priority_critical() {
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        assert_eq!(
            analyzer.determine_complexity_priority(30),
            Priority::Critical
        );
        assert_eq!(
            analyzer.determine_complexity_priority(26),
            Priority::Critical
        );
    }

    #[test]
    fn test_determine_complexity_priority_high() {
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        assert_eq!(analyzer.determine_complexity_priority(25), Priority::High);
        assert_eq!(analyzer.determine_complexity_priority(21), Priority::High);
    }

    #[test]
    fn test_determine_complexity_priority_medium() {
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        assert_eq!(analyzer.determine_complexity_priority(20), Priority::Medium);
        assert_eq!(analyzer.determine_complexity_priority(10), Priority::Medium);
    }
}
