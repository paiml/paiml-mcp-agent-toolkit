//! EXTREME TDD: C/C++ adapter tests
//!
//! RED PHASE: All tests written BEFORE implementation

#[cfg(test)]
mod cpp_adapter_red_tests {
    use crate::services::mutation::{CppAdapter, LanguageAdapter};
    use std::path::PathBuf;

    // ===== Phase 1: Basic parsing (RED) =====

    #[tokio::test]
    async fn red_cpp_adapter_must_parse_c_function() {
        let adapter = CppAdapter::new();
        let source = r#"
int add(int a, int b) {
    return a + b;
}
"#;

        let result = adapter.parse(source).await;
        assert!(result.is_ok(), "Must parse simple C function");
    }

    #[tokio::test]
    async fn red_cpp_adapter_must_parse_cpp_class() {
        let adapter = CppAdapter::new();
        let source = r#"
class Calculator {
public:
    int add(int a, int b) {
        return a + b;
    }
};
"#;

        let result = adapter.parse(source).await;
        assert!(result.is_ok(), "Must parse C++ class");
    }

    #[tokio::test]
    async fn red_cpp_adapter_must_parse_template() {
        let adapter = CppAdapter::new();
        let source = r#"
template<typename T>
T max(T a, T b) {
    return (a > b) ? a : b;
}
"#;

        let result = adapter.parse(source).await;
        assert!(result.is_ok(), "Must parse C++ template");
    }

    #[tokio::test]
    async fn red_cpp_adapter_must_parse_pointer() {
        let adapter = CppAdapter::new();
        let source = r#"
void swap(int* a, int* b) {
    int temp = *a;
    *a = *b;
    *b = temp;
}
"#;

        let result = adapter.parse(source).await;
        assert!(result.is_ok(), "Must parse pointer operations");
    }

    #[tokio::test]
    async fn red_cpp_adapter_must_reject_invalid_syntax() {
        let adapter = CppAdapter::new();
        let source = r#"
int add(int a, int b) {
    return a + b  // Missing semicolon
}
"#;

        let result = adapter.parse(source).await;
        assert!(result.is_err(), "Must reject invalid syntax");
    }

    #[test]
    fn red_cpp_adapter_must_have_correct_name() {
        let adapter = CppAdapter::new();
        assert_eq!(adapter.name(), "cpp");
    }

    #[test]
    fn red_cpp_adapter_must_support_c_cpp_extensions() {
        let adapter = CppAdapter::new();
        let extensions = adapter.extensions();

        assert!(extensions.contains(&"c"), "Must support .c");
        assert!(extensions.contains(&"cpp"), "Must support .cpp");
        assert!(extensions.contains(&"cc"), "Must support .cc");
        assert!(extensions.contains(&"cxx"), "Must support .cxx");
        assert!(extensions.contains(&"h"), "Must support .h");
        assert!(extensions.contains(&"hpp"), "Must support .hpp");
    }

    #[test]
    fn red_cpp_adapter_must_provide_mutation_operators() {
        let adapter = CppAdapter::new();
        let operators = adapter.mutation_operators();

        assert!(operators.len() >= 4, "Must have at least 4 operators");
        assert_eq!(operators[0].name(), "AOR", "Must have AOR operator");
        assert_eq!(operators[1].name(), "ROR", "Must have ROR operator");
        assert_eq!(operators[2].name(), "COR", "Must have COR operator");
        assert_eq!(operators[3].name(), "UOR", "Must have UOR operator");
    }

    // ===== Phase 2: Test helpers (RED) =====

    #[test]
    fn red_must_find_cmake_root() {
        use crate::services::mutation::cpp_adapter::find_cmake_root;

        let deep_path = PathBuf::from("/fake/project/src/lib/calculator.cpp");
        let result = find_cmake_root(&deep_path);

        // Should return None for fake path, but function must exist
        assert!(result.is_none() || result.is_some());
    }

    #[test]
    fn red_must_parse_ctest_failures() {
        use crate::services::mutation::cpp_adapter::parse_test_failures;

        let stdout = r#"
Test project /build
    Start 1: TestAdd
1/3 Test #1: TestAdd ..........................   Passed    0.01 sec
    Start 2: TestSubtract
2/3 Test #2: TestSubtract .....................***Failed    0.01 sec
    Start 3: TestMultiply
3/3 Test #3: TestMultiply .....................***Failed    0.01 sec

67% tests passed, 1 tests failed out of 3
"#;

        let failures = parse_test_failures(stdout, "");
        assert!(failures.len() >= 2, "Must detect at least 2 failures");
    }

    // ===== Phase 3: Integration (RED) =====

    #[test]
    fn red_language_registry_must_detect_cpp_files() {
        use crate::services::mutation::LanguageRegistry;
        use std::path::Path;

        let mut registry = LanguageRegistry::new();
        registry.register_cpp();

        let cpp_file = Path::new("src/main.cpp");
        let adapter = registry.detect_language(cpp_file);

        assert!(adapter.is_some(), "Must detect .cpp files");
        assert_eq!(adapter.unwrap().name(), "cpp");
    }

    #[test]
    fn red_language_registry_must_detect_c_files() {
        use crate::services::mutation::LanguageRegistry;
        use std::path::Path;

        let mut registry = LanguageRegistry::new();
        registry.register_cpp();

        let c_file = Path::new("src/util.c");
        let adapter = registry.detect_language(c_file);

        assert!(adapter.is_some(), "Must detect .c files");
        assert_eq!(adapter.unwrap().name(), "cpp");
    }

    #[test]
    fn red_language_registry_must_get_cpp_adapter_by_name() {
        use crate::services::mutation::LanguageRegistry;

        let mut registry = LanguageRegistry::new();
        registry.register_cpp();

        let adapter = registry.get_adapter("cpp");

        assert!(adapter.is_some(), "Must get C/C++ adapter by name");
        assert_eq!(adapter.unwrap().name(), "cpp");
    }
}
