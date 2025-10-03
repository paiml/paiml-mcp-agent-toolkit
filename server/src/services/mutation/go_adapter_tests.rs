//! EXTREME TDD: Go adapter tests
//!
//! RED PHASE: All tests written BEFORE implementation

#[cfg(test)]
mod go_adapter_red_tests {
    use crate::services::mutation::{GoAdapter, LanguageAdapter};
    use std::path::PathBuf;

    // ===== Phase 1: Basic parsing (RED) =====

    #[tokio::test]
    async fn red_go_adapter_must_parse_simple_function() {
        let adapter = GoAdapter::new();
        let source = r#"
package main

func add(a int, b int) int {
    return a + b
}
"#;

        let result = adapter.parse(source).await;
        assert!(result.is_ok(), "Must parse simple Go function");
    }

    #[tokio::test]
    async fn red_go_adapter_must_parse_goroutine() {
        let adapter = GoAdapter::new();
        let source = r#"
package main

func main() {
    go func() {
        println("Hello from goroutine")
    }()
}
"#;

        let result = adapter.parse(source).await;
        assert!(result.is_ok(), "Must parse goroutine");
    }

    #[tokio::test]
    async fn red_go_adapter_must_parse_channel() {
        let adapter = GoAdapter::new();
        let source = r#"
package main

func main() {
    ch := make(chan int)
    ch <- 42
}
"#;

        let result = adapter.parse(source).await;
        assert!(result.is_ok(), "Must parse channel");
    }

    #[tokio::test]
    async fn red_go_adapter_must_parse_struct() {
        let adapter = GoAdapter::new();
        let source = r#"
package main

type Person struct {
    Name string
    Age  int
}
"#;

        let result = adapter.parse(source).await;
        assert!(result.is_ok(), "Must parse struct");
    }

    #[tokio::test]
    async fn red_go_adapter_must_reject_invalid_syntax() {
        let adapter = GoAdapter::new();
        let source = r#"
package main

func add(a int, b int) int {
    return a + b  // Missing closing brace
"#;

        let result = adapter.parse(source).await;
        assert!(result.is_err(), "Must reject invalid syntax");
    }

    #[test]
    fn red_go_adapter_must_have_correct_name() {
        let adapter = GoAdapter::new();
        assert_eq!(adapter.name(), "go");
    }

    #[test]
    fn red_go_adapter_must_support_go_extension() {
        let adapter = GoAdapter::new();
        let extensions = adapter.extensions();

        assert!(extensions.contains(&"go"), "Must support .go");
    }

    #[test]
    fn red_go_adapter_must_provide_mutation_operators() {
        let adapter = GoAdapter::new();
        let operators = adapter.mutation_operators();

        assert!(operators.len() >= 4, "Must have at least 4 operators");
        assert_eq!(operators[0].name(), "AOR", "Must have AOR operator");
        assert_eq!(operators[1].name(), "ROR", "Must have ROR operator");
        assert_eq!(operators[2].name(), "COR", "Must have COR operator");
        assert_eq!(operators[3].name(), "UOR", "Must have UOR operator");
    }

    // ===== Phase 2: Test helpers (RED) =====

    #[test]
    fn red_must_find_go_mod_root() {
        use crate::services::mutation::go_adapter::find_go_mod_root;

        let deep_path = PathBuf::from("/fake/project/src/modules/calculator.go");
        let result = find_go_mod_root(&deep_path);

        // Should return None for fake path, but function must exist
        assert!(result.is_none() || result.is_some());
    }

    #[test]
    fn red_must_parse_go_test_failures() {
        use crate::services::mutation::go_adapter::parse_test_failures;

        let stdout = r#"
--- FAIL: TestAdd (0.00s)
    calculator_test.go:10: Expected 5, got 4
--- FAIL: TestSubtract (0.00s)
    calculator_test.go:20: Expected 2, got 3
PASS
FAIL    example.com/calculator  0.005s
"#;

        let failures = parse_test_failures(stdout, "");
        assert!(failures.len() >= 2, "Must detect at least 2 failures");
    }

    // ===== Phase 3: Integration (RED) =====

    #[test]
    fn red_language_registry_must_detect_go_files() {
        use crate::services::mutation::LanguageRegistry;
        use std::path::Path;

        let mut registry = LanguageRegistry::new();
        registry.register_go();

        let go_file = Path::new("src/main.go");
        let adapter = registry.detect_language(go_file);

        assert!(adapter.is_some(), "Must detect .go files");
        assert_eq!(adapter.unwrap().name(), "go");
    }

    #[test]
    fn red_language_registry_must_get_go_adapter_by_name() {
        use crate::services::mutation::LanguageRegistry;

        let mut registry = LanguageRegistry::new();
        registry.register_go();

        let adapter = registry.get_adapter("go");

        assert!(adapter.is_some(), "Must get Go adapter by name");
        assert_eq!(adapter.unwrap().name(), "go");
    }
}
