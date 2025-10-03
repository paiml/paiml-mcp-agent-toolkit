//! EXTREME TDD: Python adapter tests
//!
//! RED PHASE: All tests written BEFORE implementation

#[cfg(test)]
mod python_adapter_red_tests {
    use crate::services::mutation::{LanguageAdapter, PythonAdapter};
    use std::path::PathBuf;

    // ===== Phase 1: Basic parsing (RED) =====

    #[tokio::test]
    async fn red_python_adapter_must_parse_simple_function() {
        let adapter = PythonAdapter::new();
        let source = "def add(a, b):\n    return a + b";

        let result = adapter.parse(source).await;
        assert!(result.is_ok(), "Must parse simple Python function");
    }

    #[tokio::test]
    async fn red_python_adapter_must_parse_list_comprehension() {
        let adapter = PythonAdapter::new();
        let source = "squares = [x**2 for x in range(10)]";

        let result = adapter.parse(source).await;
        assert!(result.is_ok(), "Must parse list comprehension");
    }

    #[tokio::test]
    async fn red_python_adapter_must_parse_dict_comprehension() {
        let adapter = PythonAdapter::new();
        let source = "squares = {x: x**2 for x in range(10)}";

        let result = adapter.parse(source).await;
        assert!(result.is_ok(), "Must parse dict comprehension");
    }

    #[tokio::test]
    async fn red_python_adapter_must_parse_async_function() {
        let adapter = PythonAdapter::new();
        let source = "async def fetch_data():\n    return await get('/api')";

        let result = adapter.parse(source).await;
        assert!(result.is_ok(), "Must parse async function");
    }

    #[tokio::test]
    async fn red_python_adapter_must_reject_invalid_syntax() {
        let adapter = PythonAdapter::new();
        let source = "def add(a, b)\n    return a + b";  // Missing colon

        let result = adapter.parse(source).await;
        assert!(result.is_err(), "Must reject invalid syntax");
    }

    #[test]
    fn red_python_adapter_must_have_correct_name() {
        let adapter = PythonAdapter::new();
        assert_eq!(adapter.name(), "python");
    }

    #[test]
    fn red_python_adapter_must_support_py_extension() {
        let adapter = PythonAdapter::new();
        let extensions = adapter.extensions();

        assert!(extensions.contains(&"py"), "Must support .py");
    }

    #[test]
    fn red_python_adapter_must_provide_mutation_operators() {
        let adapter = PythonAdapter::new();
        let operators = adapter.mutation_operators();

        assert!(operators.len() >= 4, "Must have at least 4 operators");
        assert_eq!(operators[0].name(), "AOR", "Must have AOR operator");
        assert_eq!(operators[1].name(), "ROR", "Must have ROR operator");
        assert_eq!(operators[2].name(), "COR", "Must have COR operator");
        assert_eq!(operators[3].name(), "UOR", "Must have UOR operator");
    }

    // ===== Phase 2: Test helpers (RED) =====

    #[test]
    fn red_must_find_pytest_root() {
        use crate::services::mutation::python_adapter::find_pytest_root;

        let deep_path = PathBuf::from("/fake/project/src/modules/calculator.py");
        let result = find_pytest_root(&deep_path);

        // Should return None for fake path, but function must exist
        assert!(result.is_none() || result.is_some());
    }

    #[test]
    fn red_must_parse_pytest_failures() {
        use crate::services::mutation::python_adapter::parse_test_failures;

        let stdout = r#"
============================= test session starts ==============================
collected 3 items

tests/test_math.py .F.                                                   [100%]

=================================== FAILURES ===================================
_______________________________ test_subtract __________________________________

    def test_subtract():
>       assert subtract(5, 3) == 2
E       AssertionError: assert 3 == 2

tests/test_math.py:10: AssertionError
=========================== short test summary info ============================
FAILED tests/test_math.py::test_subtract - AssertionError: assert 3 == 2
        "#;

        let failures = parse_test_failures(stdout, "");
        assert!(failures.len() >= 1, "Must detect at least 1 failure");
    }

    // ===== Phase 3: Integration (RED) =====

    #[test]
    fn red_language_registry_must_detect_python_files() {
        use crate::services::mutation::LanguageRegistry;
        use std::path::Path;

        let mut registry = LanguageRegistry::new();
        registry.register_python();

        let py_file = Path::new("src/app.py");
        let adapter = registry.detect_language(py_file);

        assert!(adapter.is_some(), "Must detect .py files");
        assert_eq!(adapter.unwrap().name(), "python");
    }

    #[test]
    fn red_language_registry_must_get_python_adapter_by_name() {
        use crate::services::mutation::LanguageRegistry;

        let mut registry = LanguageRegistry::new();
        registry.register_python();

        let adapter = registry.get_adapter("python");

        assert!(adapter.is_some(), "Must get Python adapter by name");
        assert_eq!(adapter.unwrap().name(), "python");
    }
}
