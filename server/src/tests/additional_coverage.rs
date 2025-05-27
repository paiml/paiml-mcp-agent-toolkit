use crate::cli::args::validate_params;
use crate::models::churn::ChurnOutputFormat;
use crate::models::template::{ParameterSpec, ParameterType};

#[test]
fn test_churn_output_format() {
    // Test ChurnOutputFormat parsing
    assert_eq!(
        "json".parse::<ChurnOutputFormat>().unwrap(),
        ChurnOutputFormat::Json
    );
    assert_eq!(
        "markdown".parse::<ChurnOutputFormat>().unwrap(),
        ChurnOutputFormat::Markdown
    );
    assert_eq!(
        "csv".parse::<ChurnOutputFormat>().unwrap(),
        ChurnOutputFormat::Csv
    );
    assert_eq!(
        "summary".parse::<ChurnOutputFormat>().unwrap(),
        ChurnOutputFormat::Summary
    );

    // Test invalid format
    assert!("invalid".parse::<ChurnOutputFormat>().is_err());
}

#[test]
fn test_cli_validate_params() {
    let specs = vec![
        ParameterSpec {
            name: "project_name".to_string(),
            param_type: ParameterType::ProjectName,
            required: true,
            default_value: None,
            validation_pattern: None,
            description: "Project name".to_string(),
        },
        ParameterSpec {
            name: "has_tests".to_string(),
            param_type: ParameterType::Boolean,
            required: false,
            default_value: Some("true".to_string()),
            validation_pattern: None,
            description: "Include tests".to_string(),
        },
    ];

    // Test valid params
    let mut params = serde_json::Map::new();
    params.insert("project_name".to_string(), serde_json::json!("my_project"));
    params.insert("has_tests".to_string(), serde_json::json!(true));
    assert!(validate_params(&specs, &params).is_ok());

    // Test missing required param
    let mut params = serde_json::Map::new();
    params.insert("has_tests".to_string(), serde_json::json!(false));
    let result = validate_params(&specs, &params);
    assert!(result.is_err());
    assert!(result.unwrap_err()[0].contains("Missing required parameter"));

    // Test unknown parameter
    let mut params = serde_json::Map::new();
    params.insert("project_name".to_string(), serde_json::json!("test"));
    params.insert("unknown".to_string(), serde_json::json!("value"));
    let result = validate_params(&specs, &params);
    assert!(result.is_err());
    assert!(result.unwrap_err()[0].contains("Unknown parameter"));
}

#[test]
fn test_additional_model_coverage() {
    use crate::models::mcp::{McpError, McpResponse};
    use crate::models::template::{GeneratedTemplate, Toolchain};

    // Test GeneratedTemplate
    let template = GeneratedTemplate {
        content: "test content".to_string(),
        filename: "test.txt".to_string(),
        checksum: "abc123".to_string(),
        toolchain: Toolchain::RustCli {
            cargo_features: vec![],
        },
    };
    assert_eq!(template.filename, "test.txt");

    // Test McpError
    let error = McpError {
        code: -32600,
        message: "Invalid request".to_string(),
        data: None,
    };
    assert_eq!(error.code, -32600);

    // Test McpResponse error constructor
    let response = McpResponse::error(serde_json::json!(1), -32601, "Method not found".to_string());
    assert!(response.error.is_some());
    assert_eq!(response.error.unwrap().code, -32601);
}
