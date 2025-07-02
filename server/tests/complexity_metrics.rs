use pmat::services::complexity::*;
use pmat::stateless_server::StatelessTemplateServer;
use pmat::TemplateServerTrait;
use std::path::Path;

#[cfg(test)]
mod coverage_improvement {
    use super::*;

    #[test]
    fn test_complexity_metrics_creation() {
        let metrics = ComplexityMetrics {
            cyclomatic: 5,
            cognitive: 7,
            nesting_max: 3,
            lines: 25,
        };

        assert_eq!(metrics.cyclomatic, 5);
        assert_eq!(metrics.cognitive, 7);
        assert_eq!(metrics.nesting_max, 3);
        assert_eq!(metrics.lines, 25);
    }

    #[test]
    fn test_complexity_metrics_default() {
        let metrics = ComplexityMetrics::default();

        assert_eq!(metrics.cyclomatic, 0);
        assert_eq!(metrics.cognitive, 0);
        assert_eq!(metrics.nesting_max, 0);
        assert_eq!(metrics.lines, 0);
    }

    #[test]
    fn test_function_complexity_creation() {
        let function = FunctionComplexity {
            name: "test_function".to_string(),
            line_start: 10,
            line_end: 20,
            metrics: ComplexityMetrics {
                cyclomatic: 3,
                cognitive: 4,
                nesting_max: 2,
                lines: 11,
            },
        };

        assert_eq!(function.name, "test_function");
        assert_eq!(function.line_start, 10);
        assert_eq!(function.line_end, 20);
        assert_eq!(function.metrics.cyclomatic, 3);
    }

    #[test]
    fn test_class_complexity_creation() {
        let class = ClassComplexity {
            name: "TestClass".to_string(),
            line_start: 1,
            line_end: 50,
            metrics: ComplexityMetrics::default(),
            methods: vec![],
        };

        assert_eq!(class.name, "TestClass");
        assert!(class.methods.is_empty());
    }

    #[test]
    fn test_file_complexity_metrics_creation() {
        let file_metrics = FileComplexityMetrics {
            path: "test.rs".to_string(),
            total_complexity: ComplexityMetrics::default(),
            functions: vec![],
            classes: vec![],
        };

        assert_eq!(file_metrics.path, "test.rs");
        assert!(file_metrics.functions.is_empty());
        assert!(file_metrics.classes.is_empty());
    }

    #[test]
    fn test_compute_complexity_cache_key() {
        let path = Path::new("test.rs");
        let content = b"fn main() { println!(\"Hello\"); }";

        let key1 = compute_complexity_cache_key(path, content);
        let key2 = compute_complexity_cache_key(path, content);

        // Same content should produce same key
        assert_eq!(key1, key2);

        // Different content should produce different key
        let different_content = b"fn main() { println!(\"World\"); }";
        let key3 = compute_complexity_cache_key(path, different_content);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_aggregate_results_empty() {
        let file_metrics: Vec<FileComplexityMetrics> = vec![];
        let report = aggregate_results(file_metrics);

        // Should handle empty input gracefully
        assert!(format!("{report:?}").contains("ComplexityReport"));
    }

    #[test]
    fn test_aggregate_results_with_data() {
        let file_metrics = vec![FileComplexityMetrics {
            path: "test1.rs".to_string(),
            total_complexity: ComplexityMetrics {
                cyclomatic: 5,
                cognitive: 7,
                nesting_max: 2,
                lines: 20,
            },
            functions: vec![FunctionComplexity {
                name: "func1".to_string(),
                line_start: 1,
                line_end: 10,
                metrics: ComplexityMetrics {
                    cyclomatic: 3,
                    cognitive: 4,
                    nesting_max: 2,
                    lines: 10,
                },
            }],
            classes: vec![],
        }];

        let report = aggregate_results(file_metrics);

        // Should create valid report
        assert!(format!("{report:?}").contains("ComplexityReport"));
    }

    #[test]
    fn test_format_complexity_summary() {
        let file_metrics = vec![FileComplexityMetrics {
            path: "test.rs".to_string(),
            total_complexity: ComplexityMetrics {
                cyclomatic: 5,
                cognitive: 7,
                nesting_max: 2,
                lines: 20,
            },
            functions: vec![],
            classes: vec![],
        }];

        let report = aggregate_results(file_metrics);
        let summary = format_complexity_summary(&report);

        assert!(!summary.is_empty());
        assert!(summary.contains("complexity") || summary.contains("Complexity"));
    }

    #[test]
    fn test_format_complexity_report() {
        let file_metrics = vec![FileComplexityMetrics {
            path: "test.rs".to_string(),
            total_complexity: ComplexityMetrics {
                cyclomatic: 10,
                cognitive: 15,
                nesting_max: 4,
                lines: 50,
            },
            functions: vec![FunctionComplexity {
                name: "complex_function".to_string(),
                line_start: 1,
                line_end: 25,
                metrics: ComplexityMetrics {
                    cyclomatic: 8,
                    cognitive: 12,
                    nesting_max: 4,
                    lines: 25,
                },
            }],
            classes: vec![],
        }];

        let report = aggregate_results(file_metrics);
        let formatted = format_complexity_report(&report);

        assert!(!formatted.is_empty());
        // Just verify we got some report content
        assert!(formatted.len() > 10);
    }

    #[test]
    fn test_format_as_sarif() {
        let file_metrics = vec![FileComplexityMetrics {
            path: "test.rs".to_string(),
            total_complexity: ComplexityMetrics {
                cyclomatic: 15,
                cognitive: 20,
                nesting_max: 5,
                lines: 100,
            },
            functions: vec![],
            classes: vec![],
        }];

        let report = aggregate_results(file_metrics);
        let sarif_result = format_as_sarif(&report);

        assert!(sarif_result.is_ok());
        let sarif_json = sarif_result.unwrap();

        // Should be valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&sarif_json).unwrap();
        assert!(parsed.is_object());
    }

    #[test]
    fn test_stateless_template_server_creation() {
        let result = StatelessTemplateServer::new();
        assert!(result.is_ok());

        let _server = result.unwrap();
        // Just test that it can be created
    }

    #[tokio::test]
    async fn test_stateless_template_server_basic_operations() {
        let server = StatelessTemplateServer::new().unwrap();

        // Test getting renderer
        let _renderer = server.get_renderer();

        // Test cache methods (should return None for stateless server)
        assert!(server.get_metadata_cache().is_none());
        assert!(server.get_content_cache().is_none());
        assert!(server.get_s3_client().is_none());
        assert!(server.get_bucket_name().is_none());
    }

    #[test]
    fn test_various_helper_functions() {
        use handlebars::Handlebars;
        use pmat::utils::helpers::*;
        use serde_json::json;

        // Test helper functions with various inputs
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("snake_case", Box::new(snake_case_helper));
        handlebars.register_helper("kebab_case", Box::new(kebab_case_helper));
        handlebars.register_helper("pascal_case", Box::new(pascal_case_helper));
        handlebars.register_helper("current_year", Box::new(current_year_helper));
        handlebars.register_helper("current_date", Box::new(current_date_helper));

        // Test snake_case with various inputs
        let test_cases = vec![
            ("MyProjectName", "my_project_name"),
            ("camelCase", "camel_case"),
            ("already_snake", "already_snake"),
            ("UPPER_CASE", "upper__case"),
        ];

        for (input, expected) in test_cases {
            let template = "{{snake_case name}}";
            let data = json!({"name": input});
            let result = handlebars.render_template(template, &data).unwrap();
            assert_eq!(result, expected);
        }

        // Test current year and date helpers
        let year_template = "{{current_year}}";
        let year_result = handlebars
            .render_template(year_template, &json!({}))
            .unwrap();
        let year: u32 = year_result.parse().expect("Should be valid year");
        assert!((2024..=2100).contains(&year));

        let date_template = "{{current_date}}";
        let date_result = handlebars
            .render_template(date_template, &json!({}))
            .unwrap();
        assert_eq!(date_result.len(), 10); // YYYY-MM-DD format
    }

    #[test]
    fn test_error_handling_coverage() {
        use pmat::models::error::*;

        // Test various error types to improve coverage
        let template_error = TemplateError::NotFound("test".to_string());
        assert!(format!("{template_error}").contains("test"));

        let validation_error = TemplateError::ValidationError {
            parameter: "test_param".to_string(),
            reason: "error1".to_string(),
        };
        assert!(format!("{validation_error}").contains("test_param"));

        let invalid_uri_error = TemplateError::InvalidUri {
            uri: "invalid://test".to_string(),
        };
        assert!(format!("{invalid_uri_error}").contains("invalid://test"));

        // Test debug formatting
        assert!(!format!("{template_error:?}").is_empty());
        assert!(!format!("{validation_error:?}").is_empty());

        // Test MCP error codes
        assert_eq!(template_error.to_mcp_code(), -32000);
        assert_eq!(validation_error.to_mcp_code(), -32003);
        assert_eq!(invalid_uri_error.to_mcp_code(), -32002);
    }
}
