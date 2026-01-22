//! Utility handlers tests - Part 2: Coverage tests
//! Extracted for file health compliance (CB-040)

use super::*;

mod coverage_tests {
    use super::*;
    use tempfile::TempDir;

    // MarkdownBuilder Tests

    #[test]
    fn test_markdown_builder_new() {
        let builder = MarkdownBuilder::new();
        assert!(builder.content.is_empty());
    }

    #[test]
    fn test_markdown_builder_add_header_level_1() {
        let mut builder = MarkdownBuilder::new();
        builder.add_header(1, "Test Header");

        assert!(builder.content.contains("# Test Header"));
        assert!(builder.content.ends_with("\n\n"));
    }

    #[test]
    fn test_markdown_builder_add_header_level_2() {
        let mut builder = MarkdownBuilder::new();
        builder.add_header(2, "Sub Header");

        assert!(builder.content.contains("## Sub Header"));
    }

    #[test]
    fn test_markdown_builder_add_header_level_3() {
        let mut builder = MarkdownBuilder::new();
        builder.add_header(3, "Section");

        assert!(builder.content.contains("### Section"));
    }

    #[test]
    fn test_markdown_builder_add_metric() {
        let mut builder = MarkdownBuilder::new();
        builder.add_metric("Count", 42);

        assert!(builder.content.contains("- **Count**: 42"));
    }

    #[test]
    fn test_markdown_builder_add_metric_string() {
        let mut builder = MarkdownBuilder::new();
        builder.add_metric("Language", "Rust");

        assert!(builder.content.contains("- **Language**: Rust"));
    }

    #[test]
    fn test_markdown_builder_add_percentage_metric() {
        let mut builder = MarkdownBuilder::new();
        builder.add_percentage_metric("Coverage", 85.5);

        assert!(builder.content.contains("- **Coverage**: 85.5%"));
    }

    #[test]
    fn test_markdown_builder_add_newline() {
        let mut builder = MarkdownBuilder::new();
        let initial_len = builder.content.len();
        builder.add_newline();

        assert_eq!(builder.content.len(), initial_len + 1);
        assert!(builder.content.ends_with('\n'));
    }

    #[test]
    fn test_markdown_builder_build() {
        let mut builder = MarkdownBuilder::new();
        builder.add_header(1, "Title");
        builder.add_metric("Value", 100);

        let output = builder.build();

        assert!(output.contains("# Title"));
        assert!(output.contains("- **Value**: 100"));
    }

    // calculate_pagerank_value Tests

    #[test]
    fn test_calculate_pagerank_value_zero_incoming() {
        assert_eq!(calculate_pagerank_value(0, 0), 0.0);
        assert_eq!(calculate_pagerank_value(0, 5), 0.0);
        assert_eq!(calculate_pagerank_value(0, 10), 0.0);
    }

    #[test]
    fn test_calculate_pagerank_value_one_incoming_no_outgoing() {
        assert_eq!(calculate_pagerank_value(1, 0), 0.25);
    }

    #[test]
    fn test_calculate_pagerank_value_one_incoming_with_outgoing() {
        assert_eq!(calculate_pagerank_value(1, 1), 0.35);
        assert_eq!(calculate_pagerank_value(1, 5), 0.35);
    }

    #[test]
    fn test_calculate_pagerank_value_low_incoming() {
        assert_eq!(calculate_pagerank_value(2, 0), 0.50);
        assert_eq!(calculate_pagerank_value(3, 2), 0.50);
    }

    #[test]
    fn test_calculate_pagerank_value_medium_incoming() {
        assert_eq!(calculate_pagerank_value(4, 0), 0.65);
        assert_eq!(calculate_pagerank_value(5, 2), 0.65);
        assert_eq!(calculate_pagerank_value(6, 5), 0.65);
    }

    #[test]
    fn test_calculate_pagerank_value_high_incoming() {
        assert_eq!(calculate_pagerank_value(7, 0), 0.75);
        assert_eq!(calculate_pagerank_value(8, 2), 0.75);
        assert_eq!(calculate_pagerank_value(10, 5), 0.75);
    }

    #[test]
    fn test_calculate_pagerank_value_very_high_incoming() {
        assert_eq!(calculate_pagerank_value(11, 0), 0.85);
        assert_eq!(calculate_pagerank_value(50, 10), 0.85);
        assert_eq!(calculate_pagerank_value(100, 100), 0.85);
    }

    // get_big_o_complexity Tests

    #[test]
    fn test_get_big_o_complexity_constant() {
        assert_eq!(get_big_o_complexity(1), "O(1)");
        assert_eq!(get_big_o_complexity(2), "O(1)");
        assert_eq!(get_big_o_complexity(3), "O(1)");
    }

    #[test]
    fn test_get_big_o_complexity_linear() {
        assert_eq!(get_big_o_complexity(4), "O(n)");
        assert_eq!(get_big_o_complexity(5), "O(n)");
        assert_eq!(get_big_o_complexity(7), "O(n)");
    }

    #[test]
    fn test_get_big_o_complexity_linearithmic() {
        assert_eq!(get_big_o_complexity(8), "O(n log n)");
        assert_eq!(get_big_o_complexity(10), "O(n log n)");
        assert_eq!(get_big_o_complexity(15), "O(n log n)");
    }

    #[test]
    fn test_get_big_o_complexity_quadratic() {
        assert_eq!(get_big_o_complexity(16), "O(n²)");
        assert_eq!(get_big_o_complexity(20), "O(n²)");
        assert_eq!(get_big_o_complexity(25), "O(n²)");
    }

    #[test]
    fn test_get_big_o_complexity_unknown() {
        assert_eq!(get_big_o_complexity(26), "O(?)");
        assert_eq!(get_big_o_complexity(50), "O(?)");
        assert_eq!(get_big_o_complexity(100), "O(?)");
    }

    // detect_or_use_toolchain Tests

    #[test]
    fn test_detect_or_use_toolchain_provided() {
        let temp_dir = TempDir::new().unwrap();
        let result = detect_or_use_toolchain(Some("python".to_string()), temp_dir.path());

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "python");
    }

    #[test]
    fn test_detect_or_use_toolchain_with_cargo_toml() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();

        let result = detect_or_use_toolchain(None, temp_dir.path());

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "rust");
    }

    #[test]
    fn test_detect_or_use_toolchain_fallback() {
        let temp_dir = TempDir::new().unwrap();
        // Create empty directory with no recognizable project files

        let result = detect_or_use_toolchain(None, temp_dir.path());

        // Should fallback to rust
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "rust");
    }

    // Format Helper Tests

    #[test]
    fn test_simple_markdown_format() {
        let ctx = create_test_project_context(10, 50, 5, 3, 2);

        let output = simple_markdown_format(&ctx, "rust");

        assert!(output.contains("# Project Context"));
        assert!(output.contains("**Language**: rust"));
        assert!(output.contains("**Total Files**: 10"));
        assert!(output.contains("**Total Functions**: 50"));
    }

    #[test]
    fn test_simple_llm_format() {
        let ctx = create_test_project_context(5, 25, 3, 2, 1);

        let output = simple_llm_format(&ctx, "python", Path::new("/test/project"));

        assert!(output.contains("Summary:"));
        assert!(output.contains("Files: 5"));
        assert!(output.contains("Functions: 25"));
    }

    #[test]
    fn test_simple_llm_format_large_codebase() {
        let ctx = create_test_project_context(50, 100, 10, 5, 3);

        let output = simple_llm_format(&ctx, "rust", Path::new("/large/project"));

        assert!(output.contains("Quality Insights:"));
        assert!(output.contains("Large codebase"));
    }

    #[test]
    fn test_simple_json_format() {
        let ctx = create_test_project_context(3, 15, 2, 1, 0);

        let result = simple_json_format(&ctx, "typescript");

        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("\"project_type\": \"typescript\""));
        assert!(json.contains("\"total_files\": 3"));
        assert!(json.contains("\"total_functions\": 15"));
    }

    #[test]
    fn test_simple_sarif_format() {
        let ctx = create_test_project_context(5, 20, 3, 2, 1);

        let result = simple_sarif_format(&ctx, "go");

        assert!(result.is_ok());
        let sarif = result.unwrap();
        assert!(sarif.contains("\"version\": \"2.1.0\""));
        assert!(sarif.contains("sarif-schema"));
        assert!(sarif.contains("pmat-context"));
    }

    // Graph Section Tests

    #[test]
    fn test_generate_graph_section_markdown() {
        let annotations = vec![
            create_test_context_annotation("src/main.rs", 0.85, 1, "high"),
            create_test_context_annotation("src/lib.rs", 0.65, 1, "medium"),
        ];

        let output = generate_graph_section(&annotations, ContextFormat::Markdown);

        assert!(output.contains("Graph Analysis"));
        assert!(output.contains("PageRank"));
        assert!(output.contains("src/main.rs"));
        assert!(output.contains("0.85"));
    }

    #[test]
    fn test_generate_graph_section_json() {
        let annotations = vec![
            create_test_context_annotation("file1.rs", 0.5, 1, "medium"),
            create_test_context_annotation("file2.rs", 0.3, 2, "low"),
        ];

        let output = generate_graph_section(&annotations, ContextFormat::Json);

        assert!(output.contains("graph_analysis"));
        assert!(output.contains("file_count"));
        assert!(output.contains("community_count"));
    }

    #[test]
    fn test_generate_graph_section_sarif() {
        let annotations = vec![create_test_context_annotation("test.rs", 0.7, 1, "high")];

        let output = generate_graph_section(&annotations, ContextFormat::Sarif);

        assert!(output.contains("Graph analysis"));
        assert!(output.contains("1 files"));
    }

    #[test]
    fn test_generate_graph_section_empty() {
        let annotations: Vec<crate::graph::ContextAnnotation> = vec![];

        let output = generate_graph_section(&annotations, ContextFormat::Markdown);

        assert!(output.contains("Graph Analysis"));
    }

    // write_context_output Tests

    #[tokio::test]
    async fn test_write_context_output_to_file() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("output.md");

        let content = "# Test Content\n\nSome text here.";
        let result = write_context_output(Some(output_path.clone()), content).await;

        assert!(result.is_ok());
        assert!(output_path.exists());

        let written_content = std::fs::read_to_string(&output_path).unwrap();
        assert_eq!(written_content, content);
    }

    #[tokio::test]
    async fn test_write_context_output_to_stdout() {
        let content = "# Test Content";
        let result = write_context_output(None, content).await;

        assert!(result.is_ok());
    }

    // Static Annotation Helper Tests

    #[test]
    fn test_add_static_annotations() {
        let mut annotations = String::new();
        add_static_annotations(&mut annotations);

        assert!(annotations.contains("[provability: 75%]"));
        assert!(annotations.contains("[coverage: 65%]"));
    }

    // Integration Tests

    #[test]
    fn test_context_format_variants() {
        // Verify all ContextFormat variants are handled
        let formats = vec![
            ContextFormat::Markdown,
            ContextFormat::Json,
            ContextFormat::Sarif,
            ContextFormat::LlmOptimized,
        ];

        for format in formats {
            let cloned = format.clone();
            assert!(matches!(
                cloned,
                ContextFormat::Markdown
                    | ContextFormat::Json
                    | ContextFormat::Sarif
                    | ContextFormat::LlmOptimized
            ));
        }
    }

    #[test]
    fn test_output_format_variants() {
        let formats = vec![OutputFormat::Table, OutputFormat::Json, OutputFormat::Yaml];

        assert_eq!(formats.len(), 3);
    }

    // Helper Functions for Tests

    fn create_test_project_context(
        files: usize,
        functions: usize,
        structs: usize,
        enums: usize,
        traits: usize,
    ) -> crate::services::context::ProjectContext {
        crate::services::context::ProjectContext {
            project_type: "rust".to_string(),
            files: vec![],
            graph: None,
            summary: crate::services::context::ProjectSummary {
                total_files: files,
                total_functions: functions,
                total_structs: structs,
                total_enums: enums,
                total_traits: traits,
                total_impls: 0,
                dependencies: vec![],
            },
        }
    }

    fn create_test_context_annotation(
        file_path: &str,
        score: f64,
        community: usize,
        rank: &str,
    ) -> crate::graph::ContextAnnotation {
        crate::graph::ContextAnnotation {
            file_path: file_path.to_string(),
            importance_score: score,
            community_id: community,
            complexity_rank: rank.to_string(),
            related_files: vec![],
        }
    }
}
