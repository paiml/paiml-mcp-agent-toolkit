#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests_cli {
    use super::*;
    use serde_json::json;

    // CLI Adapter tests
    #[test]
    fn test_cli_adapter_decode_analyze_complexity() {
        let adapter = CliAdapter;
        let request = json!({
            "command": "analyze",
            "subcommand": "complexity",
            "args": {
                "file_path": "src/main.rs"
            }
        });

        let raw = serde_json::to_vec(&request).unwrap();
        let result = adapter.decode(&raw);
        assert!(result.is_ok());

        let unified = result.unwrap();
        assert!(matches!(unified.operation, Operation::AnalyzeComplexity(_)));
    }

    #[test]
    fn test_cli_adapter_decode_analyze_satd() {
        let adapter = CliAdapter;
        let request = json!({
            "command": "analyze",
            "subcommand": "satd",
            "args": {
                "strict": true
            }
        });

        let raw = serde_json::to_vec(&request).unwrap();
        let result = adapter.decode(&raw);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cli_adapter_decode_analyze_dead_code() {
        let adapter = CliAdapter;
        let request = json!({
            "command": "analyze",
            "subcommand": "dead-code",
            "args": {
                "include_tests": true
            }
        });

        let raw = serde_json::to_vec(&request).unwrap();
        let result = adapter.decode(&raw);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cli_adapter_decode_quality_gate() {
        let adapter = CliAdapter;
        let request = json!({
            "command": "quality-gate",
            "args": {
                "fail_on_violation": true
            }
        });

        let raw = serde_json::to_vec(&request).unwrap();
        let result = adapter.decode(&raw);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cli_adapter_decode_refactor_commands() {
        let adapter = CliAdapter;

        // refactor start
        let request = json!({
            "command": "refactor",
            "subcommand": "start",
            "args": {
                "file_path": "src/main.rs"
            }
        });
        let raw = serde_json::to_vec(&request).unwrap();
        assert!(adapter.decode(&raw).is_ok());

        // refactor next
        let request = json!({
            "command": "refactor",
            "subcommand": "next",
            "args": {
                "session_id": "session-123"
            }
        });
        let raw = serde_json::to_vec(&request).unwrap();
        assert!(adapter.decode(&raw).is_ok());

        // refactor stop
        let request = json!({
            "command": "refactor",
            "subcommand": "stop",
            "args": {
                "session_id": "session-123"
            }
        });
        let raw = serde_json::to_vec(&request).unwrap();
        assert!(adapter.decode(&raw).is_ok());
    }

    #[test]
    fn test_cli_adapter_decode_unknown_command() {
        let adapter = CliAdapter;
        let request = json!({
            "command": "unknown",
            "args": {}
        });

        let raw = serde_json::to_vec(&request).unwrap();
        let result = adapter.decode(&raw);
        assert!(result.is_err());
    }

    #[test]
    fn test_cli_adapter_decode_analyze_unknown_subcommand() {
        let adapter = CliAdapter;
        let request = json!({
            "command": "analyze",
            "subcommand": "unknown",
            "args": {}
        });

        let raw = serde_json::to_vec(&request).unwrap();
        let result = adapter.decode(&raw);
        assert!(result.is_err());
    }

    #[test]
    fn test_cli_adapter_decode_refactor_unknown_subcommand() {
        let adapter = CliAdapter;
        let request = json!({
            "command": "refactor",
            "subcommand": "unknown",
            "args": {}
        });

        let raw = serde_json::to_vec(&request).unwrap();
        let result = adapter.decode(&raw);
        assert!(result.is_err());
    }

    #[test]
    fn test_cli_adapter_encode_success() {
        let adapter = CliAdapter;
        let response = UnifiedResponse {
            result: Some(json!({"status": "ok"})),
            error: None,
            metadata: super::super::ResponseMetadata {
                request_id: "req-1".to_string(),
                duration_ms: 42,
                version: "1.0.0".to_string(),
            },
        };

        let encoded = adapter.encode(response).unwrap();
        let decoded: CliResponse = serde_json::from_slice(&encoded).unwrap();

        assert!(decoded.success);
        assert!(decoded.result.is_some());
        assert!(decoded.error.is_none());
    }

    #[test]
    fn test_cli_adapter_encode_error() {
        let adapter = CliAdapter;
        let response = UnifiedResponse {
            result: None,
            error: Some(super::super::ErrorInfo {
                code: -32600,
                message: "Error message".to_string(),
                details: None,
            }),
            metadata: super::super::ResponseMetadata {
                request_id: "req-2".to_string(),
                duration_ms: 10,
                version: "1.0.0".to_string(),
            },
        };

        let encoded = adapter.encode(response).unwrap();
        let decoded: CliResponse = serde_json::from_slice(&encoded).unwrap();

        assert!(!decoded.success);
        assert!(decoded.error.is_some());
        assert_eq!(decoded.error.unwrap(), "Error message");
    }

    #[tokio::test]
    async fn test_cli_adapter_handle() {
        let adapter = CliAdapter;
        let request = CliRequest {
            command: "test".to_string(),
            subcommand: None,
            args: json!({}),
        };

        let response = adapter.handle(request).await;
        assert!(response.success);
    }

    // Helper function tests
    #[test]
    fn test_parse_http_request_valid() {
        let request =
            "GET /api/test HTTP/1.1\r\nContent-Type: application/json\r\n\r\n{\"key\": \"value\"}";
        let result = parse_http_request(request.as_bytes());
        assert!(result.is_ok());

        let parsed = result.unwrap();
        assert_eq!(parsed.method, "GET");
        assert_eq!(parsed.path, "/api/test");
        assert!(parsed.headers.contains_key("Content-Type"));
    }

    #[test]
    fn test_parse_http_request_empty() {
        let request = "";
        let result = parse_http_request(request.as_bytes());
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_http_request_invalid_request_line() {
        let request = "INVALID\r\n\r\n";
        let result = parse_http_request(request.as_bytes());
        assert!(result.is_err());
    }

    #[test]
    fn test_route_to_operation_complexity() {
        let result = route_to_operation("/analyze/complexity", "GET");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Operation::AnalyzeComplexity(_)));
    }

    #[test]
    fn test_route_to_operation_satd() {
        let result = route_to_operation("/analyze/satd", "POST");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Operation::AnalyzeSatd(_)));
    }

    #[test]
    fn test_route_to_operation_dead_code() {
        let result = route_to_operation("/analyze/dead-code", "GET");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Operation::AnalyzeDeadCode(_)));
    }

    #[test]
    fn test_route_to_operation_quality_gate() {
        let result = route_to_operation("/quality/gate", "POST");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Operation::QualityGate(_)));
    }

    #[test]
    fn test_route_to_operation_unknown() {
        let result = route_to_operation("/unknown/path", "GET");
        assert!(result.is_err());
    }

    // Test HttpResponse and CliRequest/CliResponse structs
    #[test]
    fn test_http_response_serialization() {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let response = HttpResponse {
            status: 200,
            headers,
            body: json!({"status": "ok"}),
        };

        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: HttpResponse = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.status, 200);
    }

    #[test]
    fn test_cli_request_serialization() {
        let request = CliRequest {
            command: "analyze".to_string(),
            subcommand: Some("complexity".to_string()),
            args: json!({"file": "test.rs"}),
        };

        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: CliRequest = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.command, "analyze");
        assert_eq!(deserialized.subcommand, Some("complexity".to_string()));
    }

    #[test]
    fn test_cli_response_serialization() {
        let response = CliResponse {
            success: true,
            result: Some(json!({"data": "test"})),
            error: None,
        };

        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: CliResponse = serde_json::from_str(&serialized).unwrap();

        assert!(deserialized.success);
        assert!(deserialized.result.is_some());
    }
}
