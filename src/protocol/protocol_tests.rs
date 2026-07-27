// Tests for protocol module

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    // Test RequestContext creation
    #[test]
    fn test_request_context_new() {
        let ctx = RequestContext::new("test-protocol");
        assert_eq!(ctx.protocol, "test-protocol");
        assert!(!ctx.request_id.is_empty());
        assert!(ctx.timestamp > 0);
    }

    #[test]
    fn test_request_context_from_json_rpc() {
        let json_rpc = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "test.method".to_string(),
            params: json!({}),
            id: json!(42),
        };

        let ctx = RequestContext::from_json_rpc(&json_rpc);
        assert_eq!(ctx.protocol, "json-rpc");
        assert_eq!(ctx.request_id, "42");
        assert!(ctx.timestamp > 0);
    }

    #[test]
    fn test_request_context_from_http() {
        let http_request = HttpRequest {
            method: "POST".to_string(),
            path: "/api/test".to_string(),
            headers: HashMap::new(),
            body: json!({}),
        };

        let ctx = RequestContext::from_http(&http_request);
        assert_eq!(ctx.protocol, "http");
        assert!(!ctx.request_id.is_empty());
        assert!(ctx.timestamp > 0);
    }

    #[test]
    fn test_request_context_from_http_with_request_id_header() {
        let mut headers = HashMap::new();
        headers.insert("x-request-id".to_string(), "custom-id-123".to_string());

        let http_request = HttpRequest {
            method: "GET".to_string(),
            path: "/api/test".to_string(),
            headers,
            body: json!({}),
        };

        let ctx = RequestContext::from_http(&http_request);
        assert_eq!(ctx.request_id, "custom-id-123");
    }

    // Test ErrorInfo to JsonRpcError conversion
    #[test]
    fn test_error_info_to_json_rpc_error() {
        let error_info = ErrorInfo {
            code: -32600,
            message: "Invalid request".to_string(),
            details: Some(json!({"key": "value"})),
        };

        let json_rpc_error: JsonRpcError = error_info.into();
        assert_eq!(json_rpc_error.code, -32600);
        assert_eq!(json_rpc_error.message, "Invalid request");
        assert_eq!(json_rpc_error.data, Some(json!({"key": "value"})));
    }

    #[test]
    fn test_error_info_to_json_rpc_error_no_details() {
        let error_info = ErrorInfo {
            code: -32601,
            message: "Method not found".to_string(),
            details: None,
        };

        let json_rpc_error: JsonRpcError = error_info.into();
        assert_eq!(json_rpc_error.code, -32601);
        assert_eq!(json_rpc_error.message, "Method not found");
        assert!(json_rpc_error.data.is_none());
    }

    // Test ProtocolError Display
    #[test]
    fn test_protocol_error_unknown_method() {
        let err = ProtocolError::UnknownMethod("unknown.method".to_string());
        assert!(err.to_string().contains("Unknown method"));
        assert!(err.to_string().contains("unknown.method"));
    }

    #[test]
    fn test_protocol_error_invalid_params() {
        let err = ProtocolError::InvalidParams("missing field".to_string());
        assert!(err.to_string().contains("Invalid parameters"));
        assert!(err.to_string().contains("missing field"));
    }

    // Test Operation serialization/deserialization
    #[test]
    fn test_operation_analyze_complexity_serialization() {
        let op = Operation::AnalyzeComplexity(ComplexityParams {
            file_path: Some("src/main.rs".to_string()),
            max_cyclomatic: Some(20),
            max_cognitive: Some(15),
        });

        let serialized = serde_json::to_string(&op).unwrap();
        assert!(serialized.contains("analyze_complexity"));
        assert!(serialized.contains("src/main.rs"));
    }

    #[test]
    fn test_operation_analyze_satd_serialization() {
        let op = Operation::AnalyzeSatd(SatdParams {
            file_path: Some("src/lib.rs".to_string()),
            strict: true,
        });

        let serialized = serde_json::to_string(&op).unwrap();
        assert!(serialized.contains("analyze_satd"));
        assert!(serialized.contains("strict"));
    }

    #[test]
    fn test_operation_dead_code_serialization() {
        let op = Operation::AnalyzeDeadCode(DeadCodeParams {
            file_path: None,
            include_tests: true,
        });

        let serialized = serde_json::to_string(&op).unwrap();
        assert!(serialized.contains("analyze_dead_code"));
        assert!(serialized.contains("include_tests"));
    }

    #[test]
    fn test_operation_quality_gate_serialization() {
        let op = Operation::QualityGate(QualityGateParams {
            file_path: Some("/project/src".to_string()),
            fail_on_violation: true,
        });

        let serialized = serde_json::to_string(&op).unwrap();
        assert!(serialized.contains("quality_gate"));
    }

    #[test]
    fn test_operation_refactor_start_serialization() {
        let op = Operation::RefactorStart(RefactorStartParams {
            file_path: "src/complex.rs".to_string(),
            target_complexity: Some(15),
        });

        let serialized = serde_json::to_string(&op).unwrap();
        assert!(serialized.contains("refactor_start"));
        assert!(serialized.contains("target_complexity"));
    }

    // Test UnifiedRequest serialization
    #[test]
    fn test_unified_request_serialization() {
        let request = UnifiedRequest {
            operation: Operation::AnalyzeComplexity(ComplexityParams {
                file_path: None,
                max_cyclomatic: None,
                max_cognitive: None,
            }),
            params: json!({}),
            context: RequestContext::new("test"),
        };

        let serialized = serde_json::to_string(&request).unwrap();
        assert!(serialized.contains("operation"));
        assert!(serialized.contains("context"));
    }

    // Test UnifiedResponse serialization
    #[test]
    fn test_unified_response_success_serialization() {
        let response = UnifiedResponse {
            result: Some(json!({"status": "ok"})),
            error: None,
            metadata: ResponseMetadata {
                request_id: "req-123".to_string(),
                duration_ms: 42,
                version: "1.0.0".to_string(),
            },
        };

        let serialized = serde_json::to_string(&response).unwrap();
        assert!(serialized.contains("status"));
        assert!(serialized.contains("req-123"));
    }

    #[test]
    fn test_unified_response_error_serialization() {
        let response = UnifiedResponse {
            result: None,
            error: Some(ErrorInfo {
                code: -32600,
                message: "Error message".to_string(),
                details: None,
            }),
            metadata: ResponseMetadata {
                request_id: "req-456".to_string(),
                duration_ms: 10,
                version: "1.0.0".to_string(),
            },
        };

        let serialized = serde_json::to_string(&response).unwrap();
        assert!(serialized.contains("-32600"));
        assert!(serialized.contains("Error message"));
    }

    // Test JsonRpcRequest/Response
    #[test]
    fn test_json_rpc_request_serialization() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "test.method".to_string(),
            params: json!({"key": "value"}),
            id: json!(1),
        };

        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: JsonRpcRequest = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.jsonrpc, "2.0");
        assert_eq!(deserialized.method, "test.method");
        assert_eq!(deserialized.id, json!(1));
    }

    #[test]
    fn test_json_rpc_response_serialization() {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(json!({"data": "test"})),
            error: None,
            id: json!(42),
        };

        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: JsonRpcResponse = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.jsonrpc, "2.0");
        assert_eq!(deserialized.id, json!(42));
        assert!(deserialized.result.is_some());
    }

    // Test HttpRequest serialization
    #[test]
    fn test_http_request_serialization() {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let request = HttpRequest {
            method: "POST".to_string(),
            path: "/api/analyze".to_string(),
            headers,
            body: json!({"file": "test.rs"}),
        };

        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: HttpRequest = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.method, "POST");
        assert_eq!(deserialized.path, "/api/analyze");
    }

    // Test parameter structs
    #[test]
    fn test_project_params_serialization() {
        let params = ProjectParams {
            name: "my-project".to_string(),
            template: "rust-cli".to_string(),
        };

        let serialized = serde_json::to_string(&params).unwrap();
        let deserialized: ProjectParams = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.name, "my-project");
        assert_eq!(deserialized.template, "rust-cli");
    }

    #[test]
    fn test_agent_params_serialization() {
        let params = AgentParams {
            name: "test-agent".to_string(),
            capabilities: vec!["analyze".to_string(), "refactor".to_string()],
        };

        let serialized = serde_json::to_string(&params).unwrap();
        let deserialized: AgentParams = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.name, "test-agent");
        assert_eq!(deserialized.capabilities.len(), 2);
    }

    #[test]
    fn test_pdmt_params_serialization() {
        let params = PdmtParams {
            requirement: "implement feature X".to_string(),
            granularity: "fine".to_string(),
            seed: Some(42),
        };

        let serialized = serde_json::to_string(&params).unwrap();
        let deserialized: PdmtParams = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.requirement, "implement feature X");
        assert_eq!(deserialized.seed, Some(42));
    }

    #[test]
    fn test_quality_proxy_params_serialization() {
        let params = QualityProxyParams {
            file_path: "src/main.rs".to_string(),
            content: "fn main() {}".to_string(),
            mode: "strict".to_string(),
        };

        let serialized = serde_json::to_string(&params).unwrap();
        let deserialized: QualityProxyParams = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.file_path, "src/main.rs");
        assert_eq!(deserialized.mode, "strict");
    }

    #[test]
    fn test_context_params_serialization() {
        let params = ContextParams {
            file_path: Some("/project/src".to_string()),
            format: "markdown".to_string(),
        };

        let serialized = serde_json::to_string(&params).unwrap();
        let deserialized: ContextParams = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.format, "markdown");
    }
}
