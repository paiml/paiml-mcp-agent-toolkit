use crate::models::error::TemplateError;

#[test]
fn test_template_not_found_error() {
    let error = TemplateError::TemplateNotFound {
        uri: "template://invalid/path".to_string(),
    };

    assert_eq!(
        error.to_string(),
        "Template not found: template://invalid/path"
    );
    assert_eq!(error.to_mcp_code(), -32001);
}

#[test]
fn test_invalid_uri_error() {
    let error = TemplateError::InvalidUri {
        uri: "invalid://uri".to_string(),
    };

    assert_eq!(error.to_string(), "Invalid template URI: invalid://uri");
    assert_eq!(error.to_mcp_code(), -32002);
}

#[test]
fn test_validation_error() {
    let error = TemplateError::ValidationError {
        parameter: "project_name".to_string(),
        reason: "cannot be empty".to_string(),
    };

    assert_eq!(
        error.to_string(),
        "Parameter validation failed: project_name - cannot be empty"
    );
    assert_eq!(error.to_mcp_code(), -32003);
}

#[test]
fn test_render_error() {
    let error = TemplateError::RenderError {
        line: 42,
        message: "undefined variable".to_string(),
    };

    assert_eq!(
        error.to_string(),
        "Template rendering failed at line 42: undefined variable"
    );
    assert_eq!(error.to_mcp_code(), -32004);
}

#[test]
fn test_not_found_error() {
    let error = TemplateError::NotFound("Resource not found".to_string());

    assert_eq!(error.to_string(), "Not found: Resource not found");
    assert_eq!(error.to_mcp_code(), -32000); // Generic error code
}

#[test]
fn test_s3_error() {
    let source_error = anyhow::anyhow!("Connection timeout");
    let error = TemplateError::S3Error {
        operation: "GetObject".to_string(),
        source: source_error,
    };

    assert_eq!(error.to_string(), "S3 operation failed: GetObject");
    assert_eq!(error.to_mcp_code(), -32000); // Generic error code
}

#[test]
fn test_invalid_utf8_error() {
    let invalid_bytes = vec![0xff, 0xfe, 0xfd];
    let utf8_error = String::from_utf8(invalid_bytes).unwrap_err();
    let error = TemplateError::InvalidUtf8(utf8_error);

    assert!(error.to_string().contains("Invalid UTF-8"));
    assert_eq!(error.to_mcp_code(), -32000); // Generic error code
}

#[test]
fn test_cache_error() {
    let cache_error = anyhow::anyhow!("Cache full");
    let error = TemplateError::CacheError(cache_error);

    assert_eq!(error.to_string(), "Cache operation failed");
    assert_eq!(error.to_mcp_code(), -32000); // Generic error code
}

#[test]
fn test_json_error() {
    let json_str = r#"{"invalid": json"#;
    let json_error = serde_json::from_str::<serde_json::Value>(json_str).unwrap_err();
    let error = TemplateError::JsonError(json_error);

    assert!(error.to_string().contains("JSON serialization error"));
    assert_eq!(error.to_mcp_code(), -32000); // Generic error code
}
