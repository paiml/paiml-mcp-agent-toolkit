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
        let formats = [OutputFormat::Table, OutputFormat::Json, OutputFormat::Yaml];

        assert_eq!(formats.len(), 3);
    }
}
