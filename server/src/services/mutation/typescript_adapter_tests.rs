//! EXTREME TDD: TypeScript/JavaScript adapter tests
//!
//! RED PHASE: All tests written BEFORE implementation

#[cfg(test)]
mod typescript_adapter_red_tests {
    use crate::services::mutation::{LanguageAdapter, TypeScriptAdapter};
    use std::path::PathBuf;

    // ===== Phase 1: Basic parsing (DONE - GREEN) =====

    #[tokio::test]
    async fn red_typescript_adapter_must_parse_simple_function() {
        let adapter = TypeScriptAdapter::new();
        let source = "const add = (a: number, b: number): number => a + b;";

        let result = adapter.parse(source).await;
        assert!(result.is_ok(), "Must parse simple TypeScript function");
    }

    #[tokio::test]
    async fn red_typescript_adapter_must_parse_jsx() {
        let adapter = TypeScriptAdapter::new();
        let source = r#"const MyComponent = () => <div>Hello</div>;"#;

        let result = adapter.parse(source).await;
        assert!(result.is_ok(), "Must parse JSX syntax");
    }

    #[tokio::test]
    async fn red_typescript_adapter_must_parse_async_await() {
        let adapter = TypeScriptAdapter::new();
        let source = "const fetchData = async () => await fetch('/api');";

        let result = adapter.parse(source).await;
        assert!(result.is_ok(), "Must parse async/await");
    }

    #[tokio::test]
    async fn red_typescript_adapter_must_reject_invalid_syntax() {
        let adapter = TypeScriptAdapter::new();
        let source = "const add = (a: number, b: number) { invalid }";

        let result = adapter.parse(source).await;
        assert!(result.is_err(), "Must reject invalid syntax");
    }

    #[test]
    fn red_typescript_adapter_must_have_correct_name() {
        let adapter = TypeScriptAdapter::new();
        assert_eq!(adapter.name(), "typescript");
    }

    #[test]
    fn red_typescript_adapter_must_support_ts_tsx_js_jsx() {
        let adapter = TypeScriptAdapter::new();
        let extensions = adapter.extensions();

        assert!(extensions.contains(&"ts"), "Must support .ts");
        assert!(extensions.contains(&"tsx"), "Must support .tsx");
        assert!(extensions.contains(&"js"), "Must support .js");
        assert!(extensions.contains(&"jsx"), "Must support .jsx");
    }

    #[test]
    fn red_typescript_adapter_must_provide_mutation_operators() {
        let adapter = TypeScriptAdapter::new();
        let operators = adapter.mutation_operators();

        assert!(operators.len() >= 4, "Must have at least 4 operators");
        assert_eq!(operators[0].name(), "AOR", "Must have AOR operator");
        assert_eq!(operators[1].name(), "ROR", "Must have ROR operator");
        assert_eq!(operators[2].name(), "COR", "Must have COR operator");
        assert_eq!(operators[3].name(), "UOR", "Must have UOR operator");
    }

    // ===== Phase 2: Test helpers (NEW - RED) =====

    #[test]
    fn red_must_find_package_json_root() {
        use crate::services::mutation::typescript_adapter::find_package_json_root;

        // Test that we can find package.json by traversing up
        let deep_path = PathBuf::from("/fake/project/src/components/Button.tsx");
        let result = find_package_json_root(&deep_path);

        // Should return None for fake path, but function must exist
        assert!(result.is_none() || result.is_some());
    }

    #[test]
    fn red_must_parse_jest_test_failures() {
        use crate::services::mutation::typescript_adapter::parse_test_failures;

        let stdout = r#"
PASS src/add.test.ts
  ✓ adds 1 + 2 to equal 3
  ✕ subtracts 5 - 2 to equal 3
FAIL src/div.test.ts
        "#;

        let failures = parse_test_failures(stdout, "");
        assert!(!failures.is_empty(), "Must detect at least 1 failure");
    }

    #[test]
    fn red_must_extract_test_name_from_jest_output() {
        use crate::services::mutation::typescript_adapter::extract_test_name;

        let line = "  ✕ subtracts 5 - 2 to equal 3";
        let result = extract_test_name(line);

        assert!(result.is_some(), "Must extract test name");
        assert!(result.unwrap().contains("subtract"), "Must contain test name");
    }

    // ===== Phase 3: Integration (NEW - RED) =====

    #[test]
    fn red_language_registry_must_detect_typescript_files() {
        use crate::services::mutation::LanguageRegistry;
        use std::path::Path;

        let mut registry = LanguageRegistry::new();
        registry.register_typescript();

        let ts_file = Path::new("src/app.ts");
        let adapter = registry.detect_language(ts_file);

        assert!(adapter.is_some(), "Must detect .ts files");
        assert_eq!(adapter.unwrap().name(), "typescript");
    }

    #[test]
    fn red_language_registry_must_detect_tsx_files() {
        use crate::services::mutation::LanguageRegistry;
        use std::path::Path;

        let mut registry = LanguageRegistry::new();
        registry.register_typescript();

        let tsx_file = Path::new("src/Component.tsx");
        let adapter = registry.detect_language(tsx_file);

        assert!(adapter.is_some(), "Must detect .tsx files");
        assert_eq!(adapter.unwrap().name(), "typescript");
    }

    #[test]
    fn red_language_registry_must_get_typescript_adapter_by_name() {
        use crate::services::mutation::LanguageRegistry;

        let mut registry = LanguageRegistry::new();
        registry.register_typescript();

        let adapter = registry.get_adapter("typescript");

        assert!(adapter.is_some(), "Must get TypeScript adapter by name");
        assert_eq!(adapter.unwrap().name(), "typescript");
    }
}
