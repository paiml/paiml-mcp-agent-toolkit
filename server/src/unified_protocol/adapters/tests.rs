//! Tests for protocol adapters

use super::*;

#[test]
fn test_adapter_module_basics() {
    // Basic test to ensure module compiles
    assert_eq!(1, 1);
}

#[cfg(test)]
mod cli_adapter_tests {
    use super::super::cli::*;
    
    #[test]
    fn test_cli_request_creation() {
        let req = CliRequest {
            command: "analyze".to_string(),
            args: vec!["--help".to_string()],
            env: std::collections::HashMap::new(),
        };
        
        assert_eq!(req.command, "analyze");
        assert_eq!(req.args.len(), 1);
    }
    
    #[test]
    fn test_cli_response_creation() {
        let resp = CliResponse {
            output: "test output".to_string(),
            exit_code: 0,
            metadata: None,
        };
        
        assert_eq!(resp.output, "test output");
        assert_eq!(resp.exit_code, 0);
    }
}

#[cfg(test)]
mod http_adapter_tests {
    use super::super::http::*;
    
    #[test]
    fn test_http_request_creation() {
        let req = HttpRequest {
            method: "GET".to_string(),
            path: "/api/test".to_string(),
            headers: std::collections::HashMap::new(),
            body: None,
            query: None,
        };
        
        assert_eq!(req.method, "GET");
        assert_eq!(req.path, "/api/test");
    }
    
    #[test]
    fn test_http_response_creation() {
        let resp = HttpResponse {
            status: 200,
            headers: std::collections::HashMap::new(),
            body: serde_json::json!({"status": "ok"}),
        };
        
        assert_eq!(resp.status, 200);
    }
}

#[cfg(test)]
mod mcp_adapter_tests {
    use super::super::mcp::*;
    
    #[test]
    fn test_mcp_request_creation() {
        let req = McpRequest {
            jsonrpc: "2.0".to_string(),
            method: "initialize".to_string(),
            params: Some(serde_json::json!({"version": "1.0"})),
            id: Some(serde_json::json!(1)),
        };
        
        assert_eq!(req.jsonrpc, "2.0");
        assert_eq!(req.method, "initialize");
        assert!(req.id.is_some());
    }
    
    #[test]
    fn test_mcp_response_creation() {
        let resp = McpResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(serde_json::json!({"status": "ok"})),
            error: None,
            id: Some(serde_json::json!(1)),
        };
        
        assert_eq!(resp.jsonrpc, "2.0");
        assert!(resp.result.is_some());
        assert!(resp.error.is_none());
    }
    
    #[test]
    fn test_mcp_error_creation() {
        let err = McpError {
            code: -32600,
            message: "Invalid Request".to_string(),
            data: None,
        };
        
        assert_eq!(err.code, -32600);
        assert_eq!(err.message, "Invalid Request");
    }
}