// Included by unified_typescript_analyzer.rs — do NOT add `use` or `#!` attributes here.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    // === UnifiedTypeScriptAnalyzer creation tests ===

    #[test]
    fn test_analyzer_creation() {
        let path = PathBuf::from("test.ts");
        let analyzer = UnifiedTypeScriptAnalyzer::new(path.clone());
        assert_eq!(analyzer.file_path(), path.as_path());
    }

    #[test]
    fn test_analyzer_tsx_file() {
        let path = PathBuf::from("component.tsx");
        let analyzer = UnifiedTypeScriptAnalyzer::new(path.clone());
        assert_eq!(analyzer.file_path(), path.as_path());
    }

    #[test]
    fn test_analyzer_js_file() {
        let path = PathBuf::from("script.js");
        let analyzer = UnifiedTypeScriptAnalyzer::new(path.clone());
        assert_eq!(analyzer.file_path(), path.as_path());
    }

    #[test]
    fn test_analyzer_jsx_file() {
        let path = PathBuf::from("component.jsx");
        let analyzer = UnifiedTypeScriptAnalyzer::new(path.clone());
        assert_eq!(analyzer.file_path(), path.as_path());
    }

    #[test]
    fn test_analyzer_initial_parse_count() {
        let path = PathBuf::from("test.ts");
        let analyzer = UnifiedTypeScriptAnalyzer::new(path);
        assert_eq!(analyzer.parse_count(), 0);
    }

    // === Analyze tests ===

    #[tokio::test]
    async fn test_parse_count_increments() {
        let temp_file = NamedTempFile::with_suffix(".ts").expect("internal error");
        std::fs::write(temp_file.path(), "function main() {}").expect("internal error");

        let analyzer = UnifiedTypeScriptAnalyzer::new(temp_file.path().to_path_buf());

        assert_eq!(analyzer.parse_count(), 0);

        let _ = analyzer.analyze().await;
        assert_eq!(analyzer.parse_count(), 1);
    }

    #[tokio::test]
    async fn test_analyze_simple_function() {
        let temp_file = NamedTempFile::with_suffix(".ts").expect("internal error");
        std::fs::write(
            temp_file.path(),
            "function greet(name: string) { return 'Hello ' + name; }",
        )
        .expect("internal error");

        let analyzer = UnifiedTypeScriptAnalyzer::new(temp_file.path().to_path_buf());
        let result = analyzer.analyze().await;

        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert!(!analysis.file_metrics.functions.is_empty());
    }

    #[tokio::test]
    async fn test_analyze_multiple_functions() {
        let temp_file = NamedTempFile::with_suffix(".ts").expect("internal error");
        std::fs::write(
            temp_file.path(),
            r#"
            function foo() { return 1; }
            function bar() { return 2; }
            const baz = () => 3;
            "#,
        )
        .expect("internal error");

        let analyzer = UnifiedTypeScriptAnalyzer::new(temp_file.path().to_path_buf());
        let result = analyzer.analyze().await;

        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert!(analysis.file_metrics.functions.len() >= 2);
    }

    #[tokio::test]
    async fn test_analyze_async_function() {
        let temp_file = NamedTempFile::with_suffix(".ts").expect("internal error");
        std::fs::write(
            temp_file.path(),
            "async function fetchData() { return await fetch('url'); }",
        )
        .expect("internal error");

        let analyzer = UnifiedTypeScriptAnalyzer::new(temp_file.path().to_path_buf());
        let result = analyzer.analyze().await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_arrow_function() {
        let temp_file = NamedTempFile::with_suffix(".ts").expect("internal error");
        std::fs::write(
            temp_file.path(),
            "const add = (a: number, b: number) => a + b;",
        )
        .expect("internal error");

        let analyzer = UnifiedTypeScriptAnalyzer::new(temp_file.path().to_path_buf());
        let result = analyzer.analyze().await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_class_with_methods() {
        let temp_file = NamedTempFile::with_suffix(".ts").expect("internal error");
        std::fs::write(
            temp_file.path(),
            r#"
            class Calculator {
                add(a: number, b: number) { return a + b; }
                subtract(a: number, b: number) { return a - b; }
            }
            "#,
        )
        .expect("internal error");

        let analyzer = UnifiedTypeScriptAnalyzer::new(temp_file.path().to_path_buf());
        let result = analyzer.analyze().await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_complex_control_flow() {
        let temp_file = NamedTempFile::with_suffix(".ts").expect("internal error");
        std::fs::write(
            temp_file.path(),
            r#"
            function complex(x: number) {
                if (x > 0) {
                    for (let i = 0; i < x; i++) {
                        if (i % 2 === 0) {
                            console.log(i);
                        }
                    }
                } else {
                    while (x < 0) {
                        x++;
                    }
                }
                return x;
            }
            "#,
        )
        .expect("internal error");

        let analyzer = UnifiedTypeScriptAnalyzer::new(temp_file.path().to_path_buf());
        let result = analyzer.analyze().await;

        assert!(result.is_ok());
        let analysis = result.unwrap();
        // Complexity should be higher due to control flow
        assert!(analysis.file_metrics.total_complexity.cyclomatic > 1);
    }

    #[tokio::test]
    async fn test_analyze_nonexistent_file() {
        let analyzer = UnifiedTypeScriptAnalyzer::new(PathBuf::from("/nonexistent/file.ts"));
        let result = analyzer.analyze().await;

        assert!(result.is_err());
        if let Err(AnalysisError::Io(_)) = result {
            // Expected error type
        } else {
            panic!("Expected Io error");
        }
    }

    #[tokio::test]
    async fn test_analyze_empty_file() {
        let temp_file = NamedTempFile::with_suffix(".ts").expect("internal error");
        std::fs::write(temp_file.path(), "").expect("internal error");

        let analyzer = UnifiedTypeScriptAnalyzer::new(temp_file.path().to_path_buf());
        let result = analyzer.analyze().await;

        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert!(analysis.file_metrics.functions.is_empty());
    }

    #[tokio::test]
    async fn test_analyze_only_comments() {
        let temp_file = NamedTempFile::with_suffix(".ts").expect("internal error");
        std::fs::write(
            temp_file.path(),
            "// This is a comment\n/* Another comment */",
        )
        .expect("internal error");

        let analyzer = UnifiedTypeScriptAnalyzer::new(temp_file.path().to_path_buf());
        let result = analyzer.analyze().await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_interface_only() {
        let temp_file = NamedTempFile::with_suffix(".ts").expect("internal error");
        std::fs::write(
            temp_file.path(),
            "interface User { name: string; age: number; }",
        )
        .expect("internal error");

        let analyzer = UnifiedTypeScriptAnalyzer::new(temp_file.path().to_path_buf());
        let result = analyzer.analyze().await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_type_alias() {
        let temp_file = NamedTempFile::with_suffix(".ts").expect("internal error");
        std::fs::write(temp_file.path(), "type ID = string | number;").expect("internal error");

        let analyzer = UnifiedTypeScriptAnalyzer::new(temp_file.path().to_path_buf());
        let result = analyzer.analyze().await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_parse_count_multiple_calls() {
        let temp_file = NamedTempFile::with_suffix(".ts").expect("internal error");
        std::fs::write(temp_file.path(), "const x = 1;").expect("internal error");

        let analyzer = UnifiedTypeScriptAnalyzer::new(temp_file.path().to_path_buf());

        // Call analyze multiple times
        let _ = analyzer.analyze().await;
        let _ = analyzer.analyze().await;
        let _ = analyzer.analyze().await;

        assert_eq!(analyzer.parse_count(), 3);
    }

    // === Complexity estimation tests ===

    #[test]
    fn test_estimate_complexity_simple() {
        let analyzer = UnifiedTypeScriptAnalyzer::new(PathBuf::from("test.ts"));
        let complexity = analyzer.estimate_complexity("const x = 1;");
        assert_eq!(complexity, 1); // Base complexity
    }

    #[test]
    fn test_estimate_complexity_if_statement() {
        let analyzer = UnifiedTypeScriptAnalyzer::new(PathBuf::from("test.ts"));
        let complexity = analyzer.estimate_complexity("if (x > 0) { return x; }");
        assert!(complexity > 1);
    }

    #[test]
    fn test_estimate_complexity_for_loop() {
        let analyzer = UnifiedTypeScriptAnalyzer::new(PathBuf::from("test.ts"));
        let complexity = analyzer.estimate_complexity("for (let i = 0; i < 10; i++) {}");
        assert!(complexity > 1);
    }

    #[test]
    fn test_estimate_complexity_while_loop() {
        let analyzer = UnifiedTypeScriptAnalyzer::new(PathBuf::from("test.ts"));
        let complexity = analyzer.estimate_complexity("while (x > 0) { x--; }");
        assert!(complexity > 1);
    }

    #[test]
    fn test_estimate_complexity_switch() {
        let analyzer = UnifiedTypeScriptAnalyzer::new(PathBuf::from("test.ts"));
        let complexity =
            analyzer.estimate_complexity("switch (x) { case 1: break; case 2: break; }");
        assert!(complexity > 2); // switch + 2 cases
    }

    #[test]
    fn test_estimate_complexity_try_catch() {
        let analyzer = UnifiedTypeScriptAnalyzer::new(PathBuf::from("test.ts"));
        let complexity = analyzer.estimate_complexity("try { foo(); } catch (e) { bar(); }");
        assert!(complexity > 1);
    }

    #[test]
    fn test_estimate_complexity_logical_operators() {
        let analyzer = UnifiedTypeScriptAnalyzer::new(PathBuf::from("test.ts"));
        let complexity = analyzer.estimate_complexity("if (a && b || c) {}");
        assert!(complexity > 2); // if + && + ||
    }

    #[test]
    fn test_estimate_complexity_ternary() {
        let analyzer = UnifiedTypeScriptAnalyzer::new(PathBuf::from("test.ts"));
        let complexity = analyzer.estimate_complexity("const x = a ? b : c;");
        assert!(complexity > 1);
    }

    // === UnifiedAnalysis tests ===

    #[tokio::test]
    async fn test_unified_analysis_has_parsed_at() {
        let temp_file = NamedTempFile::with_suffix(".ts").expect("internal error");
        std::fs::write(temp_file.path(), "const x = 1;").expect("internal error");

        let analyzer = UnifiedTypeScriptAnalyzer::new(temp_file.path().to_path_buf());
        let before = std::time::Instant::now();
        let result = analyzer.analyze().await.unwrap();
        let after = std::time::Instant::now();

        assert!(result.parsed_at >= before);
        assert!(result.parsed_at <= after);
    }

    #[tokio::test]
    async fn test_unified_analysis_file_path_in_metrics() {
        let temp_file = NamedTempFile::with_suffix(".ts").expect("internal error");
        std::fs::write(temp_file.path(), "const x = 1;").expect("internal error");

        let analyzer = UnifiedTypeScriptAnalyzer::new(temp_file.path().to_path_buf());
        let result = analyzer.analyze().await.unwrap();

        assert!(result.file_metrics.path.contains(".ts"));
    }

    // === AnalysisError tests ===

    #[test]
    fn test_analysis_error_io_display() {
        let err = AnalysisError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found",
        ));
        let display = format!("{}", err);
        assert!(display.contains("Failed to read file"));
    }

    #[test]
    fn test_analysis_error_parse_display() {
        let err = AnalysisError::Parse("syntax error at line 1".to_string());
        let display = format!("{}", err);
        assert!(display.contains("Failed to parse TypeScript"));
    }

    #[test]
    fn test_analysis_error_analysis_display() {
        let err = AnalysisError::Analysis("analysis failed".to_string());
        let display = format!("{}", err);
        assert!(display.contains("Analysis error"));
    }

    #[test]
    fn test_analysis_error_debug() {
        let err = AnalysisError::Parse("test".to_string());
        let debug = format!("{:?}", err);
        assert!(debug.contains("Parse"));
    }
}
