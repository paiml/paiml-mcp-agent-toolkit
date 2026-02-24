// Coverage tests for proof annotations handler - Part 2: Handle tests
// This file is include\!()'d into proof_annotations_handler.rs coverage_tests module.
// Do not add module-level attributes or standalone use statements.

    // ==================== handle_analyze_proof_annotations tests ====================

    #[tokio::test]
    async fn test_handle_analyze_proof_annotations_basic() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let project_path = temp_dir.path().to_path_buf();

        // Create a minimal Rust file
        std::fs::write(project_path.join("lib.rs"), "fn main() {}").expect("Failed to write file");

        let result = handle_analyze_proof_annotations(
            project_path,
            ProofAnnotationOutputFormat::Summary,
            false,
            false,
            None,
            None,
            None,
            false,
            false,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_proof_annotations_json_format() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let project_path = temp_dir.path().to_path_buf();

        std::fs::write(project_path.join("lib.rs"), "fn test() {}").expect("Failed to write file");

        let result = handle_analyze_proof_annotations(
            project_path,
            ProofAnnotationOutputFormat::Json,
            false,
            true,
            None,
            None,
            None,
            false,
            false,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_proof_annotations_full_format() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let project_path = temp_dir.path().to_path_buf();

        std::fs::write(
            project_path.join("main.rs"),
            "fn main() { println!(\"test\"); }",
        )
        .expect("Failed to write file");

        let result = handle_analyze_proof_annotations(
            project_path,
            ProofAnnotationOutputFormat::Full,
            false,
            true,
            None,
            None,
            None,
            false,
            false,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_proof_annotations_markdown_format() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let project_path = temp_dir.path().to_path_buf();

        std::fs::write(project_path.join("lib.rs"), "pub fn exported() {}")
            .expect("Failed to write file");

        let result = handle_analyze_proof_annotations(
            project_path,
            ProofAnnotationOutputFormat::Markdown,
            false,
            false,
            None,
            None,
            None,
            false,
            false,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_proof_annotations_sarif_format() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let project_path = temp_dir.path().to_path_buf();

        std::fs::write(project_path.join("lib.rs"), "unsafe fn danger() {}")
            .expect("Failed to write file");

        let result = handle_analyze_proof_annotations(
            project_path,
            ProofAnnotationOutputFormat::Sarif,
            false,
            true,
            None,
            None,
            None,
            false,
            false,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_proof_annotations_with_output_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let project_path = temp_dir.path().to_path_buf();
        let output_path = temp_dir.path().join("output.json");

        std::fs::write(project_path.join("lib.rs"), "fn test() {}").expect("Failed to write file");

        let result = handle_analyze_proof_annotations(
            project_path,
            ProofAnnotationOutputFormat::Json,
            false,
            false,
            None,
            None,
            Some(output_path.clone()),
            false,
            false,
        )
        .await;

        assert!(result.is_ok());
        assert!(output_path.exists());

        let content = std::fs::read_to_string(&output_path).expect("Failed to read output");
        assert!(content.contains("proof_annotations"));
    }

    #[tokio::test]
    async fn test_handle_analyze_proof_annotations_high_confidence_filter() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let project_path = temp_dir.path().to_path_buf();

        std::fs::write(project_path.join("lib.rs"), "fn test() {}").expect("Failed to write file");

        let result = handle_analyze_proof_annotations(
            project_path,
            ProofAnnotationOutputFormat::Summary,
            true, // high_confidence_only
            false,
            None,
            None,
            None,
            false,
            false,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_proof_annotations_with_property_type_filter() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let project_path = temp_dir.path().to_path_buf();

        std::fs::write(project_path.join("lib.rs"), "fn test() {}").expect("Failed to write file");

        let result = handle_analyze_proof_annotations(
            project_path,
            ProofAnnotationOutputFormat::Summary,
            false,
            false,
            Some(PropertyTypeFilter::MemorySafety),
            None,
            None,
            false,
            false,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_proof_annotations_with_verification_method_filter() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let project_path = temp_dir.path().to_path_buf();

        std::fs::write(project_path.join("lib.rs"), "fn test() {}").expect("Failed to write file");

        let result = handle_analyze_proof_annotations(
            project_path,
            ProofAnnotationOutputFormat::Summary,
            false,
            false,
            None,
            Some(VerificationMethodFilter::BorrowChecker),
            None,
            false,
            false,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_proof_annotations_with_clear_cache() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let project_path = temp_dir.path().to_path_buf();

        std::fs::write(project_path.join("lib.rs"), "fn test() {}").expect("Failed to write file");

        let result = handle_analyze_proof_annotations(
            project_path,
            ProofAnnotationOutputFormat::Summary,
            false,
            false,
            None,
            None,
            None,
            false,
            true, // clear_cache
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_proof_annotations_all_property_type_filters() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let project_path = temp_dir.path().to_path_buf();

        std::fs::write(project_path.join("lib.rs"), "fn test() {}").expect("Failed to write file");

        let filters = vec![
            PropertyTypeFilter::MemorySafety,
            PropertyTypeFilter::ThreadSafety,
            PropertyTypeFilter::DataRaceFreeze,
            PropertyTypeFilter::Termination,
            PropertyTypeFilter::FunctionalCorrectness,
            PropertyTypeFilter::ResourceBounds,
            PropertyTypeFilter::All,
        ];

        for filter in filters {
            let result = handle_analyze_proof_annotations(
                project_path.clone(),
                ProofAnnotationOutputFormat::Summary,
                false,
                false,
                Some(filter),
                None,
                None,
                false,
                false,
            )
            .await;

            assert!(result.is_ok(), "Failed for filter: {:?}", filter);
        }
    }

    #[tokio::test]
    async fn test_handle_analyze_proof_annotations_all_verification_method_filters() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let project_path = temp_dir.path().to_path_buf();

        std::fs::write(project_path.join("lib.rs"), "fn test() {}").expect("Failed to write file");

        let filters = vec![
            VerificationMethodFilter::FormalProof,
            VerificationMethodFilter::ModelChecking,
            VerificationMethodFilter::StaticAnalysis,
            VerificationMethodFilter::AbstractInterpretation,
            VerificationMethodFilter::BorrowChecker,
            VerificationMethodFilter::All,
        ];

        for filter in filters {
            let result = handle_analyze_proof_annotations(
                project_path.clone(),
                ProofAnnotationOutputFormat::Summary,
                false,
                false,
                None,
                Some(filter),
                None,
                false,
                false,
            )
            .await;

            assert!(result.is_ok(), "Failed for filter: {:?}", filter);
        }
    }

    #[tokio::test]
    async fn test_handle_analyze_proof_annotations_combined_filters() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let project_path = temp_dir.path().to_path_buf();

        std::fs::write(project_path.join("lib.rs"), "fn test() {}").expect("Failed to write file");

        let result = handle_analyze_proof_annotations(
            project_path,
            ProofAnnotationOutputFormat::Json,
            true, // high_confidence_only
            true, // include_evidence
            Some(PropertyTypeFilter::MemorySafety),
            Some(VerificationMethodFilter::BorrowChecker),
            None,
            true, // perf
            true, // clear_cache
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_proof_annotations_empty_directory() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let project_path = temp_dir.path().to_path_buf();

        // Empty directory - no source files
        let result = handle_analyze_proof_annotations(
            project_path,
            ProofAnnotationOutputFormat::Summary,
            false,
            false,
            None,
            None,
            None,
            false,
            false,
        )
        .await;

        assert!(result.is_ok());
    }

