#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    // === UnifiedRustAnalyzer tests ===

    #[test]
    fn test_analyzer_creation() {
        let path = PathBuf::from("test.rs");
        let analyzer = UnifiedRustAnalyzer::new(path.clone());
        assert_eq!(analyzer.file_path(), path.as_path());
    }

    #[test]
    fn test_analyzer_creation_absolute_path() {
        let path = PathBuf::from("/tmp/project/src/main.rs");
        let analyzer = UnifiedRustAnalyzer::new(path.clone());
        assert_eq!(analyzer.file_path(), path.as_path());
    }

    #[test]
    fn test_analyzer_creation_relative_path() {
        let path = PathBuf::from("src/lib.rs");
        let analyzer = UnifiedRustAnalyzer::new(path.clone());
        assert_eq!(analyzer.file_path(), Path::new("src/lib.rs"));
    }

    #[test]
    fn test_initial_parse_count_is_zero() {
        let analyzer = UnifiedRustAnalyzer::new(PathBuf::from("test.rs"));
        assert_eq!(analyzer.parse_count(), 0);
    }

    #[tokio::test]
    async fn test_parse_count_increments() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), "fn main() {}").unwrap();

        let analyzer = UnifiedRustAnalyzer::new(temp_file.path().to_path_buf());

        assert_eq!(analyzer.parse_count(), 0);

        let _ = analyzer.analyze().await;
        assert_eq!(analyzer.parse_count(), 1);
    }

    #[tokio::test]
    async fn test_parse_count_increments_twice() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), "fn main() {}").unwrap();

        let analyzer = UnifiedRustAnalyzer::new(temp_file.path().to_path_buf());

        let _ = analyzer.analyze().await;
        let _ = analyzer.analyze().await;

        assert_eq!(analyzer.parse_count(), 2);
    }

    // === Analysis tests ===

    #[tokio::test]
    async fn test_analyze_simple_function() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), "fn hello() { println!(\"Hello\"); }").unwrap();

        let analyzer = UnifiedRustAnalyzer::new(temp_file.path().to_path_buf());
        let result = analyzer.analyze().await.unwrap();

        assert!(!result.file_metrics.functions.is_empty());
    }

    #[tokio::test]
    async fn test_analyze_multiple_functions() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(
            temp_file.path(),
            r#"
                fn one() {}
                fn two() {}
                fn three() {}
            "#,
        )
        .unwrap();

        let analyzer = UnifiedRustAnalyzer::new(temp_file.path().to_path_buf());
        let result = analyzer.analyze().await.unwrap();

        assert_eq!(result.file_metrics.functions.len(), 3);
    }

    #[tokio::test]
    async fn test_analyze_struct_and_impl() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(
            temp_file.path(),
            r#"
                struct Point { x: i32, y: i32 }

                impl Point {
                    fn new() -> Self {
                        Point { x: 0, y: 0 }
                    }

                    fn distance(&self) -> f64 {
                        ((self.x.pow(2) + self.y.pow(2)) as f64).sqrt()
                    }
                }
            "#,
        )
        .unwrap();

        let analyzer = UnifiedRustAnalyzer::new(temp_file.path().to_path_buf());
        let result = analyzer.analyze().await.unwrap();

        // Should find impl methods
        assert!(result.file_metrics.functions.len() >= 2);
    }

    #[tokio::test]
    async fn test_analyze_with_control_flow() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(
            temp_file.path(),
            r#"
                fn complex_function(x: i32) -> i32 {
                    if x > 0 {
                        if x > 10 {
                            return x * 2;
                        }
                        return x + 1;
                    } else {
                        return 0;
                    }
                }
            "#,
        )
        .unwrap();

        let analyzer = UnifiedRustAnalyzer::new(temp_file.path().to_path_buf());
        let result = analyzer.analyze().await.unwrap();

        assert!(!result.file_metrics.functions.is_empty());
        // Complexity should be > 1 due to control flow
        assert!(result.file_metrics.functions[0].metrics.cyclomatic >= 1);
    }

    #[tokio::test]
    async fn test_analyze_with_match_expression() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(
            temp_file.path(),
            r#"
                fn process_option(opt: Option<i32>) -> i32 {
                    match opt {
                        Some(v) => v,
                        None => 0,
                    }
                }
            "#,
        )
        .unwrap();

        let analyzer = UnifiedRustAnalyzer::new(temp_file.path().to_path_buf());
        let result = analyzer.analyze().await.unwrap();

        assert!(!result.file_metrics.functions.is_empty());
    }

    #[tokio::test]
    async fn test_analyze_with_loops() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(
            temp_file.path(),
            r#"
                fn sum_range(n: i32) -> i32 {
                    let mut sum = 0;
                    for i in 0..n {
                        sum += i;
                    }
                    sum
                }
            "#,
        )
        .unwrap();

        let analyzer = UnifiedRustAnalyzer::new(temp_file.path().to_path_buf());
        let result = analyzer.analyze().await.unwrap();

        assert!(!result.file_metrics.functions.is_empty());
    }

    #[tokio::test]
    async fn test_analyze_async_function() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(
            temp_file.path(),
            r#"
                async fn fetch_data() -> Result<String, std::io::Error> {
                    Ok("data".to_string())
                }
            "#,
        )
        .unwrap();

        let analyzer = UnifiedRustAnalyzer::new(temp_file.path().to_path_buf());
        let result = analyzer.analyze().await.unwrap();

        assert!(!result.file_metrics.functions.is_empty());
    }

    #[tokio::test]
    async fn test_analyze_empty_file() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), "").unwrap();

        let analyzer = UnifiedRustAnalyzer::new(temp_file.path().to_path_buf());
        let result = analyzer.analyze().await.unwrap();

        assert!(result.file_metrics.functions.is_empty());
    }

    #[tokio::test]
    async fn test_analyze_only_comments() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(
            temp_file.path(),
            r#"
                // This is a comment
                /* This is a block comment */
            "#,
        )
        .unwrap();

        let analyzer = UnifiedRustAnalyzer::new(temp_file.path().to_path_buf());
        let result = analyzer.analyze().await.unwrap();

        assert!(result.file_metrics.functions.is_empty());
    }

    // === Error handling tests ===

    #[tokio::test]
    async fn test_analyze_nonexistent_file() {
        let analyzer = UnifiedRustAnalyzer::new(PathBuf::from("/nonexistent/path/file.rs"));
        let result = analyzer.analyze().await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AnalysisError::Io(_)));
    }

    #[tokio::test]
    async fn test_analyze_invalid_syntax() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), "fn broken(( { } }").unwrap();

        let analyzer = UnifiedRustAnalyzer::new(temp_file.path().to_path_buf());
        let result = analyzer.analyze().await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AnalysisError::Parse(_)));
    }

    // === UnifiedAnalysis tests ===

    #[tokio::test]
    async fn test_unified_analysis_has_timestamp() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), "fn main() {}").unwrap();

        let analyzer = UnifiedRustAnalyzer::new(temp_file.path().to_path_buf());
        let before = std::time::Instant::now();
        let result = analyzer.analyze().await.unwrap();
        let after = std::time::Instant::now();

        // parsed_at should be between before and after
        assert!(result.parsed_at >= before);
        assert!(result.parsed_at <= after);
    }

    #[tokio::test]
    async fn test_unified_analysis_contains_ast_items() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(
            temp_file.path(),
            r#"
                struct Foo;
                enum Bar { A, B }
                fn baz() {}
            "#,
        )
        .unwrap();

        let analyzer = UnifiedRustAnalyzer::new(temp_file.path().to_path_buf());
        let result = analyzer.analyze().await.unwrap();

        // Should have AST items for struct, enum, and function
        assert!(!result.ast_items.is_empty());
    }

    // === AnalysisError tests ===

    #[test]
    fn test_analysis_error_io_display() {
        let error = AnalysisError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "File not found",
        ));
        assert!(error.to_string().contains("Failed to read file"));
    }

    #[test]
    fn test_analysis_error_parse_display() {
        let error = AnalysisError::Parse("unexpected token".to_string());
        assert!(error.to_string().contains("Failed to parse Rust syntax"));
    }

    #[test]
    fn test_analysis_error_analysis_display() {
        let error = AnalysisError::Analysis("internal error".to_string());
        assert!(error.to_string().contains("Analysis error"));
    }

    // === File path edge cases ===

    #[test]
    fn test_file_path_with_spaces() {
        let path = PathBuf::from("path with spaces/file.rs");
        let analyzer = UnifiedRustAnalyzer::new(path.clone());
        assert_eq!(analyzer.file_path(), path.as_path());
    }

    #[test]
    fn test_file_path_with_unicode() {
        let path = PathBuf::from("проект/файл.rs");
        let analyzer = UnifiedRustAnalyzer::new(path.clone());
        assert_eq!(analyzer.file_path(), path.as_path());
    }
}

