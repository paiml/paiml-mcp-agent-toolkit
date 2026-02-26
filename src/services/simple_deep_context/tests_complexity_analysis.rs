    #[tokio::test]
    async fn test_complexity_analysis_uses_ast() {
        // Create a test project
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir_all(&src_dir).unwrap();

        // Write a file with known complexity
        let test_file = src_dir.join("test.rs");
        fs::write(
            &test_file,
            r#"
fn simple() {
    println!("hello");
}

fn complex() {
    if true {
        if false {
            match 5 {
                1 => println!("one"),
                2 => println!("two"),
                _ => println!("other"),
            }
        }
    }
}
"#,
        )
        .unwrap();

        // Analyze the project
        let analyzer = SimpleDeepContext::new();
        let config = SimpleAnalysisConfig {
            project_path: temp_dir.path().to_path_buf(),
            include_features: vec!["all".to_string()],
            include_patterns: vec![],
            exclude_patterns: vec![],
            enable_verbose: false,
        };

        let report = analyzer.analyze(config).await.unwrap();

        // Verify we got real complexity values, not heuristic 1.0
        assert_eq!(report.file_count, 1);
        assert_eq!(report.complexity_metrics.total_functions, 2);
        assert!(report.complexity_metrics.avg_complexity > 1.0);
        assert!(report.complexity_metrics.avg_complexity < 10.0);

        // Verify file details
        assert_eq!(report.file_complexity_details.len(), 1);
        let file_detail = &report.file_complexity_details[0];
        assert_eq!(file_detail.function_count, 2);
        assert!(file_detail.avg_complexity > 1.0);
    }

    #[tokio::test]
    async fn test_analyze_empty_project() {
        let temp_dir = TempDir::new().unwrap();

        let analyzer = SimpleDeepContext::new();
        let config = SimpleAnalysisConfig {
            project_path: temp_dir.path().to_path_buf(),
            include_features: vec![],
            include_patterns: vec![],
            exclude_patterns: vec![],
            enable_verbose: false,
        };

        let report = analyzer.analyze(config).await.unwrap();
        assert_eq!(report.file_count, 0);
        assert_eq!(report.complexity_metrics.total_functions, 0);
        assert_eq!(report.complexity_metrics.avg_complexity, 0.0);
    }

    #[tokio::test]
    async fn test_analyze_with_include_patterns() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir_all(&src_dir).unwrap();

        // Create Rust and Python files
        fs::write(src_dir.join("main.rs"), "fn main() { }").unwrap();
        fs::write(src_dir.join("script.py"), "def main(): pass").unwrap();

        let analyzer = SimpleDeepContext::new();
        let config = SimpleAnalysisConfig {
            project_path: temp_dir.path().to_path_buf(),
            include_features: vec![],
            include_patterns: vec!["**/*.rs".to_string()],
            exclude_patterns: vec![],
            enable_verbose: false,
        };

        let report = analyzer.analyze(config).await.unwrap();
        // Should only include .rs files
        assert_eq!(report.file_count, 1);
    }

    // ============ analyze_complexity Integration Tests ============

    #[tokio::test]
    async fn test_analyze_complexity_multiple_files() {
        let analyzer = SimpleDeepContext;
        let temp_dir = TempDir::new().unwrap();

        let file1 = temp_dir.path().join("simple.rs");
        fs::write(&file1, "fn simple() { }").unwrap();

        let file2 = temp_dir.path().join("complex.rs");
        fs::write(
            &file2,
            r#"
fn complex(x: i32) -> i32 {
    if x > 0 {
        if x > 10 {
            for i in 0..x {
                if i % 2 == 0 {
                    return i;
                }
            }
        }
    }
    0
}
"#,
        )
        .unwrap();

        let files = vec![file1, file2];
        let (metrics, details) = analyzer.analyze_complexity(&files).await.unwrap();

        assert!(metrics.total_functions >= 2);
        assert_eq!(details.len(), 2);
    }

    #[tokio::test]
    async fn test_analyze_multi_language_project() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir_all(&src_dir).unwrap();

        fs::write(
            src_dir.join("lib.rs"),
            "fn compute(x: i32) -> i32 {\n    if x > 0 { x * 2 } else { -x }\n}\n",
        )
        .unwrap();
        fs::write(
            src_dir.join("utils.py"),
            "def calculate(x):\n    if x > 0:\n        return x * 2\n    return -x\n",
        )
        .unwrap();

        let analyzer = SimpleDeepContext::new();
        let config = SimpleAnalysisConfig {
            project_path: temp_dir.path().to_path_buf(),
            include_features: vec![],
            include_patterns: vec![],
            exclude_patterns: vec![],
            enable_verbose: false,
        };

        let report = analyzer.analyze(config).await.unwrap();
        assert_eq!(report.file_count, 2);
        assert!(report.complexity_metrics.total_functions >= 2);
        assert!(report.file_complexity_details.len() == 2);
    }
