// Included from ast_typescript.rs — NO `use` imports, NO `#!` inner attributes
// Async integration tests for TypeScript/JavaScript analysis functions

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod async_coverage_tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    // ========================================================================
    // analyze_typescript_file_with_complexity_cached tests
    // ========================================================================

    #[tokio::test]
    async fn test_analyze_typescript_file_with_complexity_cached_basic() {
        let mut temp_file = NamedTempFile::with_suffix(".ts").unwrap();
        temp_file
            .write_all(b"function greet(name: string): string { return `Hello, ${name}`; }")
            .unwrap();
        temp_file.flush().unwrap();

        let result = analyze_typescript_file_with_complexity_cached(temp_file.path(), None).await;
        assert!(result.is_ok());

        let metrics = result.unwrap();
        assert!(!metrics.path.is_empty());
    }

    #[tokio::test]
    async fn test_analyze_typescript_file_with_complexity_cached_empty_file() {
        let mut temp_file = NamedTempFile::with_suffix(".ts").unwrap();
        temp_file.write_all(b"").unwrap();
        temp_file.flush().unwrap();

        let result = analyze_typescript_file_with_complexity_cached(temp_file.path(), None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_typescript_file_with_complexity_cached_with_cache_manager() {
        let mut temp_file = NamedTempFile::with_suffix(".ts").unwrap();
        temp_file.write_all(b"const x = 1;").unwrap();
        temp_file.flush().unwrap();

        // Pass None for cache manager (caching to be implemented)
        let result = analyze_typescript_file_with_complexity_cached(temp_file.path(), None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_typescript_file_with_complexity_cached_nonexistent_file() {
        let path = std::path::Path::new("/nonexistent/path/file.ts");
        let result = analyze_typescript_file_with_complexity_cached(path, None).await;
        assert!(result.is_err());
    }

    // ========================================================================
    // Re-exported function tests
    // ========================================================================

    #[tokio::test]
    async fn test_analyze_typescript_file_basic() {
        let mut temp_file = NamedTempFile::with_suffix(".ts").unwrap();
        temp_file
            .write_all(b"export function add(a: number, b: number): number { return a + b; }")
            .unwrap();
        temp_file.flush().unwrap();

        let result = analyze_typescript_file(temp_file.path()).await;
        assert!(result.is_ok());

        let context = result.unwrap();
        assert_eq!(context.language, "typescript");
    }

    #[tokio::test]
    async fn test_analyze_typescript_file_with_classifier() {
        let mut temp_file = NamedTempFile::with_suffix(".ts").unwrap();
        temp_file
            .write_all(b"interface User { name: string; }")
            .unwrap();
        temp_file.flush().unwrap();

        let result = analyze_typescript_file_with_classifier(temp_file.path(), None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_typescript_file_with_complexity() {
        let mut temp_file = NamedTempFile::with_suffix(".ts").unwrap();
        temp_file
            .write_all(b"class Calculator { add(a: number, b: number): number { return a + b; } }")
            .unwrap();
        temp_file.flush().unwrap();

        let result = analyze_typescript_file_with_complexity(temp_file.path()).await;
        assert!(result.is_ok());

        let metrics = result.unwrap();
        assert!(metrics.total_complexity.cyclomatic >= 1);
    }

    #[tokio::test]
    async fn test_analyze_typescript_file_with_complexity_and_classifier() {
        let mut temp_file = NamedTempFile::with_suffix(".ts").unwrap();
        temp_file.write_all(b"type ID = string | number;").unwrap();
        temp_file.flush().unwrap();

        let result =
            analyze_typescript_file_with_complexity_and_classifier(temp_file.path(), None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_javascript_file_basic() {
        let mut temp_file = NamedTempFile::with_suffix(".js").unwrap();
        temp_file
            .write_all(b"function sayHello() { console.log('Hello'); }")
            .unwrap();
        temp_file.flush().unwrap();

        let result = analyze_javascript_file(temp_file.path()).await;
        assert!(result.is_ok());

        let context = result.unwrap();
        assert_eq!(context.language, "javascript");
    }

    #[tokio::test]
    async fn test_analyze_javascript_file_with_classifier() {
        let mut temp_file = NamedTempFile::with_suffix(".js").unwrap();
        temp_file
            .write_all(b"const greeting = 'Hello, World!';")
            .unwrap();
        temp_file.flush().unwrap();

        let result = analyze_javascript_file_with_classifier(temp_file.path(), None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_javascript_file_with_complexity() {
        let mut temp_file = NamedTempFile::with_suffix(".js").unwrap();
        temp_file
            .write_all(b"function multiply(a, b) { return a * b; }")
            .unwrap();
        temp_file.flush().unwrap();

        let result = analyze_javascript_file_with_complexity(temp_file.path()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_javascript_file_with_complexity_and_classifier() {
        let mut temp_file = NamedTempFile::with_suffix(".js").unwrap();
        temp_file
            .write_all(b"class MyClass { constructor() {} }")
            .unwrap();
        temp_file.flush().unwrap();

        let result =
            analyze_javascript_file_with_complexity_and_classifier(temp_file.path(), None).await;
        assert!(result.is_ok());
    }

    // ========================================================================
    // Edge case and error handling tests
    // ========================================================================

    #[tokio::test]
    async fn test_analyze_nonexistent_typescript_file() {
        let path = std::path::Path::new("/nonexistent/file.ts");
        let result = analyze_typescript_file(path).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_analyze_nonexistent_javascript_file() {
        let path = std::path::Path::new("/nonexistent/file.js");
        let result = analyze_javascript_file(path).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_analyze_typescript_file_with_syntax_errors() {
        let mut temp_file = NamedTempFile::with_suffix(".ts").unwrap();
        // Invalid TypeScript syntax
        temp_file.write_all(b"function broken( { return }").unwrap();
        temp_file.flush().unwrap();

        // Should still return a result (may be empty or with errors)
        let result = analyze_typescript_file(temp_file.path()).await;
        // The implementation may handle this gracefully or return an error
        let _ = result;
    }

    #[tokio::test]
    async fn test_analyze_complex_typescript_file() {
        let mut temp_file = NamedTempFile::with_suffix(".ts").unwrap();
        temp_file
            .write_all(
                br#"
                import { Component } from 'react';

                interface Props {
                    name: string;
                    age?: number;
                }

                export class Greeter extends Component<Props> {
                    private message: string;

                    constructor(props: Props) {
                        super(props);
                        this.message = `Hello, ${props.name}`;
                    }

                    async greet(): Promise<string> {
                        return this.message;
                    }
                }

                export default Greeter;
                "#,
            )
            .unwrap();
        temp_file.flush().unwrap();

        let result = analyze_typescript_file(temp_file.path()).await;
        assert!(result.is_ok());
    }
}
