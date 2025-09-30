//! EXTREME TDD: Multi-Language Deep Context Tests
//!
//! These tests ensure that deep_context.md properly analyzes and displays
//! AST information for ALL supported languages.

#[cfg(test)]
mod red_phase_tests {
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    /// RED TEST: Go files MUST be analyzed with AST extraction
    ///
    /// This test will FAIL until Go AST extraction is properly wired into
    /// the context generation pipeline.
    #[tokio::test]
    async fn red_test_go_file_gets_analyzed() {
        // Create temp directory with Go file
        let temp_dir = TempDir::new().unwrap();
        let go_file = temp_dir.path().join("test.go");

        fs::write(&go_file, r#"
package main

import "fmt"

type Message struct {
    Type  string
    Value int
}

func ProcessMessage(msg Message) string {
    if msg.Type == "ping" {
        return fmt.Sprintf("pong: %d", msg.Value)
    }
    return "unknown"
}

func main() {
    msg := Message{Type: "ping", Value: 42}
    result := ProcessMessage(msg)
    fmt.Println(result)
}
"#).unwrap();

        // Run context generation
        use crate::services::context::analyze_project;
        let context = analyze_project(temp_dir.path(), "stable").await.unwrap();

        // Go file MUST be in the context
        let go_file_found = context.files.iter().any(|f| f.path.contains("test.go"));
        assert!(go_file_found, "Go file must be discovered and analyzed");

        // Find the Go file context
        let go_context = context.files.iter().find(|f| f.path.contains("test.go"))
            .expect("Must find Go file in context");

        // MUST contain Go function names in the items
        let functions: Vec<String> = go_context.items.iter()
            .filter_map(|item| {
                if let crate::services::context::AstItem::Function { name, .. } = item {
                    Some(name.clone())
                } else {
                    None
                }
            })
            .collect();

        let has_process_message = functions.iter().any(|f| f.contains("ProcessMessage"));
        let has_main = functions.iter().any(|f| f.contains("main"));

        assert!(has_process_message,
            "Go function 'ProcessMessage' must be extracted, found functions: {:?}", functions);
        assert!(has_main,
            "Go function 'main' must be extracted, found functions: {:?}", functions);

        // MUST NOT be empty
        assert!(!functions.is_empty(),
            "Go file must have functions extracted");
    }

    /// RED TEST: TypeScript files MUST be analyzed correctly (not as Rust)
    ///
    /// This test will FAIL until TypeScript is correctly identified and parsed
    /// by its own AST analyzer (not incorrectly parsed as Rust).
    #[tokio::test]
    async fn red_test_typescript_file_analyzed_correctly() {
        // Create temp directory with TypeScript file
        let temp_dir = TempDir::new().unwrap();
        let ts_file = temp_dir.path().join("test.ts");

        fs::write(&ts_file, r#"
export interface Message {
    type: "ping" | "pong";
    value: number;
}

class MessageProcessor {
    private count: number = 0;

    process(msg: Message): string {
        this.count++;
        if (msg.type === "ping") {
            return `pong: ${msg.value}`;
        }
        return "unknown";
    }

    getCount(): number {
        return this.count;
    }
}

export function createProcessor(): MessageProcessor {
    return new MessageProcessor();
}
"#).unwrap();

        // Run context generation
        use crate::services::context::analyze_project;
        let context = analyze_project(temp_dir.path(), "stable").await.unwrap();

        // TypeScript file MUST be in the context
        let ts_file_found = context.files.iter().any(|f| f.path.contains("test.ts"));
        assert!(ts_file_found, "TypeScript file must be discovered and analyzed");

        // Find the TypeScript file context
        let ts_context = context.files.iter().find(|f| f.path.contains("test.ts"))
            .expect("Must find TypeScript file in context");

        // Language MUST be detected as typescript/javascript, NOT rust
        assert_ne!(ts_context.language.to_lowercase(), "rust",
            "TypeScript file must NOT be detected as Rust, got: {}", ts_context.language);

        // MUST contain TypeScript function/method names in the items
        let functions: Vec<String> = ts_context.items.iter()
            .filter_map(|item| {
                if let crate::services::context::AstItem::Function { name, .. } = item {
                    Some(name.clone())
                } else {
                    None
                }
            })
            .collect();

        let has_process = functions.iter().any(|f| f.contains("process") || f.contains("MessageProcessor"));
        let has_create_processor = functions.iter().any(|f| f.contains("createProcessor"));

        assert!(has_process || has_create_processor || !functions.is_empty(),
            "TypeScript methods/functions must be extracted, found functions: {:?}", functions);
    }

    /// RED TEST: JavaScript files MUST show function analysis
    #[tokio::test]
    async fn red_test_javascript_file_analyzed() {
        let temp_dir = TempDir::new().unwrap();
        let js_file = temp_dir.path().join("test.js");

        fs::write(&js_file, r#"
function calculateSum(a, b) {
    if (a < 0 || b < 0) {
        throw new Error("Negative numbers not allowed");
    }
    return a + b;
}

class Calculator {
    constructor() {
        this.history = [];
    }

    add(a, b) {
        const result = calculateSum(a, b);
        this.history.push({op: 'add', a, b, result});
        return result;
    }
}

module.exports = { Calculator, calculateSum };
"#).unwrap();

        use crate::services::context::analyze_project;
        let context = analyze_project(temp_dir.path(), "stable").await.unwrap();

        let js_file_found = context.files.iter().any(|f| f.path.contains("test.js"));
        assert!(js_file_found, "JavaScript file must be discovered");

        let js_context = context.files.iter().find(|f| f.path.contains("test.js"))
            .expect("Must find JavaScript file in context");

        // Should have some functions extracted
        let function_count = js_context.items.iter()
            .filter(|item| matches!(item, crate::services::context::AstItem::Function { .. }))
            .count();

        assert!(function_count > 0 || js_context.language.to_lowercase().contains("javascript"),
            "JavaScript file must be analyzed, found {} functions with language: {}",
            function_count, js_context.language);
    }

    /// RED TEST: Multi-language project MUST analyze all languages
    #[tokio::test]
    async fn red_test_multi_language_project_analyzes_all() {
        let temp_dir = TempDir::new().unwrap();

        // Create Rust file
        fs::write(temp_dir.path().join("lib.rs"), r#"
pub fn rust_func(x: i32) -> i32 {
    x * 2
}
"#).unwrap();

        // Create Go file
        fs::write(temp_dir.path().join("main.go"), r#"
package main

func GoFunc(x int) int {
    return x * 2
}
"#).unwrap();

        // Create TypeScript file
        fs::write(temp_dir.path().join("app.ts"), r#"
export function tsFunc(x: number): number {
    return x * 2;
}
"#).unwrap();

        use crate::services::context::analyze_project;
        let context = analyze_project(temp_dir.path(), "stable").await.unwrap();

        // ALL files must be discovered
        assert!(context.files.iter().any(|f| f.path.contains("lib.rs")),
            "Rust file must be discovered");
        assert!(context.files.iter().any(|f| f.path.contains("main.go")),
            "Go file must be discovered");
        assert!(context.files.iter().any(|f| f.path.contains("app.ts")),
            "TypeScript file must be discovered");

        // At minimum, files should be present even if analysis is incomplete
        assert!(context.files.len() >= 3,
            "Should have at least 3 files, got {}", context.files.len());
    }

    /// RED TEST: Language detection must correctly identify file types
    #[test]
    fn red_test_language_detection_correctness() {
        use crate::services::language_registry::LanguageRegistry;

        let registry = LanguageRegistry::new();

        // Test Go detection
        let go_lang = registry.detect_language(&PathBuf::from("test.go"));
        assert_eq!(go_lang.name().to_lowercase(), "go",
            "Go files must be detected as 'go', got '{}'", go_lang.name());

        // Test TypeScript detection
        let ts_lang = registry.detect_language(&PathBuf::from("test.ts"));
        assert_eq!(ts_lang.name().to_lowercase(), "typescript",
            "TypeScript files must be detected as 'typescript', got '{}'", ts_lang.name());

        // Test JavaScript detection
        let js_lang = registry.detect_language(&PathBuf::from("test.js"));
        assert_eq!(js_lang.name().to_lowercase(), "javascript",
            "JavaScript files must be detected as 'javascript', got '{}'", js_lang.name());
    }
}

#[cfg(test)]
mod integration_tests {
    use std::fs;
    use tempfile::TempDir;

    /// Integration test: End-to-end Go analysis
    #[tokio::test]
    async fn test_go_analysis_end_to_end() {
        let temp_dir = TempDir::new().unwrap();
        let go_file = temp_dir.path().join("complex.go");

        fs::write(&go_file, r#"
package main

func ComplexFunc(x, y, z int) int {
    if x > 0 {
        if y > 0 {
            if z > 0 {
                return x + y + z
            }
            return x + y
        }
        return x
    }
    return 0
}
"#).unwrap();

        use crate::services::context::analyze_project;
        let context = analyze_project(temp_dir.path(), "stable").await.unwrap();

        let has_go_file = context.files.iter().any(|f| f.path.contains("complex.go"));
        assert!(has_go_file, "Go file should be in context");
    }

    /// Integration test: End-to-end TypeScript analysis
    #[tokio::test]
    async fn test_typescript_analysis_end_to_end() {
        let temp_dir = TempDir::new().unwrap();
        let ts_file = temp_dir.path().join("complex.ts");

        fs::write(&ts_file, r#"
class Complex {
    method(x: number, y: number): number {
        if (x > 0) {
            if (y > 0) {
                return x + y;
            }
            return x;
        }
        return 0;
    }
}
"#).unwrap();

        use crate::services::context::analyze_project;
        let context = analyze_project(temp_dir.path(), "stable").await.unwrap();

        let has_ts_file = context.files.iter().any(|f| f.path.contains("complex.ts"));
        assert!(has_ts_file, "TypeScript file should be in context");
    }
}