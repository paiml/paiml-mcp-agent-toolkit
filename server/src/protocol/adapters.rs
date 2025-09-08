//! Protocol adapter implementations for MCP, HTTP, and CLI

use super::*;
use async_trait::async_trait;
use std::collections::HashMap;

/// MCP Adapter implementation
pub struct McpAdapter;

#[async_trait]
impl ProtocolAdapter for McpAdapter {
    type Request = JsonRpcRequest;
    type Response = JsonRpcResponse;

    fn decode(&self, raw: &[u8]) -> Result<UnifiedRequest, ProtocolError> {
        let json_rpc: JsonRpcRequest = serde_json::from_slice(raw)?;

        // Map JSON-RPC method to Operation
        let method = json_rpc.method.clone();
        let params = json_rpc.params.clone();
        let operation = match method.as_str() {
            "analyze_complexity" => {
                let p = serde_json::from_value(params.clone())?;
                Operation::AnalyzeComplexity(p)
            }
            "analyze_satd" => {
                let p = serde_json::from_value(params.clone())?;
                Operation::AnalyzeSatd(p)
            }
            "analyze_dead_code" => {
                let p = serde_json::from_value(params.clone())?;
                Operation::AnalyzeDeadCode(p)
            }
            "generate_context" => {
                let p = serde_json::from_value(params.clone())?;
                Operation::GenerateContext(p)
            }
            "quality_gate" => {
                let p = serde_json::from_value(params.clone())?;
                Operation::QualityGate(p)
            }
            "quality_proxy" => {
                let p = serde_json::from_value(params.clone())?;
                Operation::QualityProxy(p)
            }
            "refactor_start" => {
                let p = serde_json::from_value(params.clone())?;
                Operation::RefactorStart(p)
            }
            "refactor_next" => {
                let p = serde_json::from_value(params.clone())?;
                Operation::RefactorNext(p)
            }
            "refactor_stop" => {
                let p = serde_json::from_value(params.clone())?;
                Operation::RefactorStop(p)
            }
            "scaffold_project" => {
                let p = serde_json::from_value(params.clone())?;
                Operation::ScaffoldProject(p)
            }
            "scaffold_agent" => {
                let p = serde_json::from_value(params.clone())?;
                Operation::ScaffoldAgent(p)
            }
            "pdmt_todos" => {
                let p = serde_json::from_value(params.clone())?;
                Operation::PdmtTodos(p)
            }
            _ => return Err(ProtocolError::UnknownMethod(method)),
        };

        Ok(UnifiedRequest {
            operation,
            params,
            context: RequestContext::from_json_rpc(&json_rpc),
        })
    }

    fn encode(&self, response: UnifiedResponse) -> Result<Vec<u8>, ProtocolError> {
        let json_rpc = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: response.result,
            error: response.error.map(Into::into),
            id: response.metadata.request_id.into(),
        };

        Ok(serde_json::to_vec(&json_rpc)?)
    }

    async fn handle(&self, request: Self::Request) -> Self::Response {
        // This would be implemented to process the request
        // For now, return a placeholder response
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(serde_json::json!({"status": "ok"})),
            error: None,
            id: request.id,
        }
    }
}

/// HTTP Adapter implementation
pub struct HttpAdapter;

#[async_trait]
impl ProtocolAdapter for HttpAdapter {
    type Request = HttpRequest;
    type Response = HttpResponse;

    fn decode(&self, raw: &[u8]) -> Result<UnifiedRequest, ProtocolError> {
        // Parse HTTP request and extract operation from path/method
        let request = parse_http_request(raw)?;
        let operation = route_to_operation(&request.path, &request.method)?;
        let params = request.body.clone();

        Ok(UnifiedRequest {
            operation,
            params,
            context: RequestContext::from_http(&request),
        })
    }

    fn encode(&self, response: UnifiedResponse) -> Result<Vec<u8>, ProtocolError> {
        let status = if response.error.is_some() {
            400 // Bad Request
        } else {
            200 // OK
        };

        let body = serde_json::to_vec(&response)?;

        Ok(format!(
            "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",
            status,
            body.len()
        )
        .into_bytes())
    }

    async fn handle(&self, _request: Self::Request) -> Self::Response {
        // This would be implemented to process the request
        // For now, return a placeholder response
        HttpResponse {
            status: 200,
            headers: HashMap::new(),
            body: serde_json::json!({"status": "ok"}),
        }
    }
}

/// CLI Adapter implementation
pub struct CliAdapter;

#[async_trait]
impl ProtocolAdapter for CliAdapter {
    type Request = CliRequest;
    type Response = CliResponse;

    fn decode(&self, raw: &[u8]) -> Result<UnifiedRequest, ProtocolError> {
        let cli_request: CliRequest = serde_json::from_slice(raw)?;

        let command = cli_request.command.clone();
        let args = cli_request.args.clone();

        // Map CLI command to Operation
        let operation = match command.as_str() {
            "analyze" => match cli_request.subcommand.as_deref() {
                Some("complexity") => {
                    Operation::AnalyzeComplexity(serde_json::from_value(args.clone())?)
                }
                Some("satd") => Operation::AnalyzeSatd(serde_json::from_value(args.clone())?),
                Some("dead-code") => {
                    Operation::AnalyzeDeadCode(serde_json::from_value(args.clone())?)
                }
                _ => return Err(ProtocolError::UnknownMethod(command)),
            },
            "quality-gate" => Operation::QualityGate(serde_json::from_value(args.clone())?),
            "refactor" => match cli_request.subcommand.as_deref() {
                Some("start") => Operation::RefactorStart(serde_json::from_value(args.clone())?),
                Some("next") => Operation::RefactorNext(serde_json::from_value(args.clone())?),
                Some("stop") => Operation::RefactorStop(serde_json::from_value(args.clone())?),
                _ => return Err(ProtocolError::UnknownMethod(command)),
            },
            _ => return Err(ProtocolError::UnknownMethod(command)),
        };

        Ok(UnifiedRequest {
            operation,
            params: args,
            context: RequestContext::new("cli"),
        })
    }

    fn encode(&self, response: UnifiedResponse) -> Result<Vec<u8>, ProtocolError> {
        let cli_response = CliResponse {
            success: response.error.is_none(),
            result: response.result,
            error: response.error.map(|e| e.message),
        };

        Ok(serde_json::to_vec(&cli_response)?)
    }

    async fn handle(&self, _request: Self::Request) -> Self::Response {
        // This would be implemented to process the request
        // For now, return a placeholder response
        CliResponse {
            success: true,
            result: Some(serde_json::json!({"status": "ok"})),
            error: None,
        }
    }
}

// Helper structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliRequest {
    pub command: String,
    pub subcommand: Option<String>,
    pub args: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliResponse {
    pub success: bool,
    pub result: Option<Value>,
    pub error: Option<String>,
}

// Helper functions
fn parse_http_request(raw: &[u8]) -> Result<HttpRequest, ProtocolError> {
    let request_str = String::from_utf8_lossy(raw);
    let lines: Vec<&str> = request_str.lines().collect();

    validate_request_lines(&lines)?;
    let (method, path) = parse_request_line(&lines[0])?;
    let (headers, body_start) = parse_headers(&lines);
    let body = parse_body(&lines, body_start);

    Ok(HttpRequest {
        method,
        path,
        headers,
        body,
    })
}

fn validate_request_lines(lines: &[&str]) -> Result<(), ProtocolError> {
    if lines.is_empty() {
        Err(ProtocolError::InvalidParams("Empty request".to_string()))
    } else {
        Ok(())
    }
}

fn parse_request_line(line: &str) -> Result<(String, String), ProtocolError> {
    let request_line: Vec<&str> = line.split_whitespace().collect();
    
    if request_line.len() < 2 {
        return Err(ProtocolError::InvalidParams(
            "Invalid request line".to_string(),
        ));
    }

    Ok((request_line[0].to_string(), request_line[1].to_string()))
}

fn parse_headers(lines: &[&str]) -> (HashMap<String, String>, usize) {
    let mut headers = HashMap::new();
    let mut body_start = 0;
    
    for (i, line) in lines.iter().enumerate().skip(1) {
        if line.is_empty() {
            body_start = i + 1;
            break;
        }
        
        if let Some((key, value)) = line.split_once(": ") {
            headers.insert(key.to_string(), value.to_string());
        }
    }

    (headers, body_start)
}

fn parse_body(lines: &[&str], body_start: usize) -> Value {
    if body_start >= lines.len() {
        return Value::Null;
    }

    let body_str = lines[body_start..].join("\n");
    serde_json::from_str(&body_str).unwrap_or(Value::Null)
}

fn route_to_operation(path: &str, method: &str) -> Result<Operation, ProtocolError> {
    match (method, path) {
        ("GET" | "POST", "/analyze/complexity") => {
            Ok(Operation::AnalyzeComplexity(ComplexityParams {
                file_path: None,
                max_cyclomatic: None,
                max_cognitive: None,
            }))
        }
        ("GET" | "POST", "/analyze/satd") => Ok(Operation::AnalyzeSatd(SatdParams {
            file_path: None,
            strict: false,
        })),
        ("GET" | "POST", "/analyze/dead-code") => Ok(Operation::AnalyzeDeadCode(DeadCodeParams {
            file_path: None,
            include_tests: false,
        })),
        ("POST", "/quality/gate") => Ok(Operation::QualityGate(QualityGateParams {
            file_path: None,
            fail_on_violation: false,
        })),
        _ => Err(ProtocolError::UnknownMethod(format!("{} {}", method, path))),
    }
}
