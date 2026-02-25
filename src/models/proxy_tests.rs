// Unit tests for basic proxy types: ProxyOperation, ProxyMode, QualityConfig,
// ProxyRequest, and ProxyStatus serialization roundtrips and defaults.

use super::*;

// === ProxyOperation Tests ===

#[test]
fn test_proxy_operation_write() {
    let op = ProxyOperation::Write;
    let json = serde_json::to_string(&op).unwrap();
    assert_eq!(json, "\"write\"");
}

#[test]
fn test_proxy_operation_edit() {
    let op = ProxyOperation::Edit;
    let json = serde_json::to_string(&op).unwrap();
    assert_eq!(json, "\"edit\"");
}

#[test]
fn test_proxy_operation_append() {
    let op = ProxyOperation::Append;
    let json = serde_json::to_string(&op).unwrap();
    assert_eq!(json, "\"append\"");
}

#[test]
fn test_proxy_operation_roundtrip() {
    for op in [
        ProxyOperation::Write,
        ProxyOperation::Edit,
        ProxyOperation::Append,
    ] {
        let json = serde_json::to_string(&op).unwrap();
        let deserialized: ProxyOperation = serde_json::from_str(&json).unwrap();
        assert_eq!(format!("{:?}", op), format!("{:?}", deserialized));
    }
}

// === ProxyMode Tests ===

#[test]
fn test_proxy_mode_default() {
    assert!(matches!(ProxyMode::default(), ProxyMode::Strict));
}

#[test]
fn test_proxy_mode_strict() {
    let mode = ProxyMode::Strict;
    let json = serde_json::to_string(&mode).unwrap();
    assert_eq!(json, "\"strict\"");
}

#[test]
fn test_proxy_mode_advisory() {
    let mode = ProxyMode::Advisory;
    let json = serde_json::to_string(&mode).unwrap();
    assert_eq!(json, "\"advisory\"");
}

#[test]
fn test_proxy_mode_auto_fix() {
    let mode = ProxyMode::AutoFix;
    let json = serde_json::to_string(&mode).unwrap();
    assert_eq!(json, "\"auto_fix\"");
}

#[test]
fn test_proxy_mode_roundtrip() {
    for mode in [ProxyMode::Strict, ProxyMode::Advisory, ProxyMode::AutoFix] {
        let json = serde_json::to_string(&mode).unwrap();
        let deserialized: ProxyMode = serde_json::from_str(&json).unwrap();
        assert_eq!(format!("{:?}", mode), format!("{:?}", deserialized));
    }
}

// === QualityConfig Tests ===

#[test]
fn test_quality_config_default() {
    let config = QualityConfig::default();
    assert_eq!(config.max_complexity, 20);
    assert!(!config.allow_satd);
    assert!(config.require_docs);
    assert!(config.auto_format);
}

#[test]
fn test_quality_config_custom() {
    let config = QualityConfig {
        max_complexity: 30,
        allow_satd: true,
        require_docs: false,
        auto_format: false,
    };

    assert_eq!(config.max_complexity, 30);
    assert!(config.allow_satd);
    assert!(!config.require_docs);
    assert!(!config.auto_format);
}

#[test]
fn test_quality_config_serialization_roundtrip() {
    let config = QualityConfig {
        max_complexity: 15,
        allow_satd: true,
        require_docs: false,
        auto_format: true,
    };

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: QualityConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.max_complexity, 15);
    assert!(deserialized.allow_satd);
    assert!(!deserialized.require_docs);
    assert!(deserialized.auto_format);
}

#[test]
fn test_quality_config_deserialize_with_defaults() {
    let json = r#"{}"#;
    let config: QualityConfig = serde_json::from_str(json).unwrap();

    assert_eq!(config.max_complexity, 20);
    assert!(!config.allow_satd);
    assert!(config.require_docs);
    assert!(config.auto_format);
}

// === ProxyRequest Tests ===

#[test]
fn test_proxy_request_serialization() {
    let request = ProxyRequest {
        operation: ProxyOperation::Write,
        file_path: "test.rs".to_string(),
        content: Some("fn test() {}".to_string()),
        old_content: None,
        new_content: None,
        mode: ProxyMode::Strict,
        quality_config: QualityConfig::default(),
    };

    let json = serde_json::to_string(&request).unwrap();
    let deserialized: ProxyRequest = serde_json::from_str(&json).unwrap();

    assert!(matches!(deserialized.operation, ProxyOperation::Write));
    assert_eq!(deserialized.file_path, "test.rs");
}

#[test]
fn test_proxy_request_edit_operation() {
    let request = ProxyRequest {
        operation: ProxyOperation::Edit,
        file_path: "src/lib.rs".to_string(),
        content: None,
        old_content: Some("old code".to_string()),
        new_content: Some("new code".to_string()),
        mode: ProxyMode::Advisory,
        quality_config: QualityConfig::default(),
    };

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("\"edit\""));
    assert!(json.contains("old_content"));
    assert!(json.contains("new_content"));
}

#[test]
fn test_proxy_request_append_operation() {
    let request = ProxyRequest {
        operation: ProxyOperation::Append,
        file_path: "log.txt".to_string(),
        content: Some("new log entry".to_string()),
        old_content: None,
        new_content: None,
        mode: ProxyMode::AutoFix,
        quality_config: QualityConfig::default(),
    };

    assert!(matches!(request.operation, ProxyOperation::Append));
    assert!(matches!(request.mode, ProxyMode::AutoFix));
}

#[test]
fn test_proxy_request_skips_none_fields() {
    let request = ProxyRequest {
        operation: ProxyOperation::Write,
        file_path: "test.rs".to_string(),
        content: Some("code".to_string()),
        old_content: None,
        new_content: None,
        mode: ProxyMode::default(),
        quality_config: QualityConfig::default(),
    };

    let json = serde_json::to_string(&request).unwrap();
    assert!(!json.contains("old_content"));
    assert!(!json.contains("new_content"));
}

// === ProxyStatus Tests ===

#[test]
fn test_proxy_status_accepted() {
    let status = ProxyStatus::Accepted;
    let json = serde_json::to_string(&status).unwrap();
    assert_eq!(json, "\"accepted\"");
}

#[test]
fn test_proxy_status_rejected() {
    let status = ProxyStatus::Rejected;
    let json = serde_json::to_string(&status).unwrap();
    assert_eq!(json, "\"rejected\"");
}

#[test]
fn test_proxy_status_modified() {
    let status = ProxyStatus::Modified;
    let json = serde_json::to_string(&status).unwrap();
    assert_eq!(json, "\"modified\"");
}

#[test]
fn test_proxy_status_roundtrip() {
    for status in [
        ProxyStatus::Accepted,
        ProxyStatus::Rejected,
        ProxyStatus::Modified,
    ] {
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: ProxyStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(format!("{:?}", status), format!("{:?}", deserialized));
    }
}
