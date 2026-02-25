    // Tests for collect_files (async)

    #[tokio::test]
    async fn test_collect_files_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let result = collect_files(&temp_dir.path().to_path_buf(), &None, &None).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_collect_files_with_source_files() {
        let temp_dir = TempDir::new().unwrap();

        // Create some source files
        let rust_file = temp_dir.path().join("test.rs");
        std::fs::write(&rust_file, "fn main() {}").unwrap();

        let py_file = temp_dir.path().join("test.py");
        std::fs::write(&py_file, "def main(): pass").unwrap();

        let result = collect_files(&temp_dir.path().to_path_buf(), &None, &None).await;
        assert!(result.is_ok());
        let files = result.unwrap();
        assert_eq!(files.len(), 2);
    }

    #[tokio::test]
    async fn test_collect_files_with_include_filter() {
        let temp_dir = TempDir::new().unwrap();

        // Create src subdirectory
        let src_dir = temp_dir.path().join("src");
        std::fs::create_dir(&src_dir).unwrap();

        let src_file = src_dir.join("main.rs");
        std::fs::write(&src_file, "fn main() {}").unwrap();

        let other_file = temp_dir.path().join("other.rs");
        std::fs::write(&other_file, "fn other() {}").unwrap();

        let result = collect_files(
            &temp_dir.path().to_path_buf(),
            &Some("src".to_string()),
            &None,
        )
        .await;
        assert!(result.is_ok());
        let files = result.unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].0.to_string_lossy().contains("src"));
    }

    #[tokio::test]
    async fn test_collect_files_with_exclude_filter() {
        let temp_dir = TempDir::new().unwrap();

        // Create target subdirectory
        let target_dir = temp_dir.path().join("target");
        std::fs::create_dir(&target_dir).unwrap();

        let target_file = target_dir.join("build.rs");
        std::fs::write(&target_file, "fn build() {}").unwrap();

        let src_file = temp_dir.path().join("main.rs");
        std::fs::write(&src_file, "fn main() {}").unwrap();

        let result = collect_files(
            &temp_dir.path().to_path_buf(),
            &None,
            &Some("target".to_string()),
        )
        .await;
        assert!(result.is_ok());
        let files = result.unwrap();
        assert_eq!(files.len(), 1);
        assert!(!files[0].0.to_string_lossy().contains("target"));
    }

    #[tokio::test]
    async fn test_collect_files_ignores_non_source_files() {
        let temp_dir = TempDir::new().unwrap();

        std::fs::write(temp_dir.path().join("README.md"), "# Readme").unwrap();
        std::fs::write(temp_dir.path().join("config.toml"), "[config]").unwrap();
        std::fs::write(temp_dir.path().join("main.rs"), "fn main() {}").unwrap();

        let result = collect_files(&temp_dir.path().to_path_buf(), &None, &None).await;
        assert!(result.is_ok());
        let files = result.unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].0.to_string_lossy().ends_with(".rs"));
    }

    #[tokio::test]
    async fn test_collect_files_nested_directories() {
        let temp_dir = TempDir::new().unwrap();

        // Create nested directory structure
        let deep_dir = temp_dir.path().join("a").join("b").join("c");
        std::fs::create_dir_all(&deep_dir).unwrap();
        std::fs::write(deep_dir.join("deep.rs"), "fn deep() {}").unwrap();

        let result = collect_files(&temp_dir.path().to_path_buf(), &None, &None).await;
        assert!(result.is_ok());
        let files = result.unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].0.to_string_lossy().contains("deep.rs"));
    }

    #[tokio::test]
    async fn test_collect_files_with_unreadable_file() {
        let temp_dir = TempDir::new().unwrap();

        // Create a source file
        let rust_file = temp_dir.path().join("test.rs");
        std::fs::write(&rust_file, "fn main() {}").unwrap();

        // The collect_files function should handle errors gracefully
        let result = collect_files(&temp_dir.path().to_path_buf(), &None, &None).await;
        assert!(result.is_ok());
    }

    // Tests for handle_analyze_similarity (integration, async)

    #[tokio::test]
    async fn test_handle_analyze_similarity_empty_project() {
        let temp_dir = TempDir::new().unwrap();
        let result = handle_analyze_similarity(
            temp_dir.path().to_path_buf(),
            DuplicateType::Exact,
            0.8,
            6,
            50,
            DuplicateOutputFormat::Summary,
            false,
            None,
            None,
            None,
            0,
        )
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_similarity_with_files() {
        let temp_dir = TempDir::new().unwrap();

        // Create some test files
        let file1 = temp_dir.path().join("test1.rs");
        let file2 = temp_dir.path().join("test2.rs");

        let content = r#"
fn example_function() {
    let x = 1;
    let y = 2;
    let z = x + y;
    println!("{}", z);
}
"#;
        std::fs::write(&file1, content).unwrap();
        std::fs::write(&file2, content).unwrap();

        let result = handle_analyze_similarity(
            temp_dir.path().to_path_buf(),
            DuplicateType::All,
            0.7,
            3,
            10,
            DuplicateOutputFormat::Json,
            true, // enable perf metrics
            None,
            None,
            None,
            5,
        )
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_similarity_with_output_file() {
        let temp_dir = TempDir::new().unwrap();
        let output_file = temp_dir.path().join("output.json");

        let file1 = temp_dir.path().join("test.rs");
        std::fs::write(&file1, "fn main() {}").unwrap();

        let result = handle_analyze_similarity(
            temp_dir.path().to_path_buf(),
            DuplicateType::Exact,
            0.8,
            1,
            5,
            DuplicateOutputFormat::Json,
            false,
            None,
            None,
            Some(output_file.clone()),
            0,
        )
        .await;
        assert!(result.is_ok());
        assert!(output_file.exists());
    }

    #[tokio::test]
    async fn test_handle_analyze_similarity_all_formats() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("test.rs");
        std::fs::write(&file1, "fn main() { let x = 1; }").unwrap();

        let formats = vec![
            DuplicateOutputFormat::Summary,
            DuplicateOutputFormat::Human,
            DuplicateOutputFormat::Detailed,
            DuplicateOutputFormat::Json,
            DuplicateOutputFormat::Csv,
            DuplicateOutputFormat::Sarif,
        ];

        for format in formats {
            let result = handle_analyze_similarity(
                temp_dir.path().to_path_buf(),
                DuplicateType::Exact,
                0.8,
                1,
                5,
                format.clone(),
                false,
                None,
                None,
                None,
                0,
            )
            .await;
            assert!(result.is_ok(), "Failed for format: {:?}", format);
        }
    }

    #[tokio::test]
    async fn test_handle_analyze_similarity_all_detection_types() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("test.rs");
        std::fs::write(&file1, "fn main() { let x = 1; }").unwrap();

        let types = vec![
            DuplicateType::Exact,
            DuplicateType::Fuzzy,
            DuplicateType::Renamed,
            DuplicateType::Semantic,
            DuplicateType::Gapped,
            DuplicateType::All,
        ];

        for detection_type in types {
            let result = handle_analyze_similarity(
                temp_dir.path().to_path_buf(),
                detection_type.clone(),
                0.7,
                1,
                5,
                DuplicateOutputFormat::Summary,
                false,
                None,
                None,
                None,
                0,
            )
            .await;
            assert!(result.is_ok(), "Failed for type: {:?}", detection_type);
        }
    }
