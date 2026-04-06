#[async_trait]
impl DemoProtocol for HttpDemoAdapter {
    type Request = HttpRequest;
    type Response = HttpResponse;
    type Error = HttpDemoError;

    async fn decode_request(&self, raw: &[u8]) -> Result<Self::Request, Self::Error> {
        debug_assert!(!raw.is_empty(), "raw must not be empty");
        let value: Value = serde_json::from_slice(raw)?;

        let method = value
            .get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("GET")
            .to_string();

        let path = value
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("/demo/analyze")
            .to_string();

        let query_params = value
            .get("query")
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_default();

        let headers = value
            .get("headers")
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_else(|| {
                [("Accept".to_string(), "application/json".to_string())]
                    .into_iter()
                    .collect()
            });

        let body = value.get("body").cloned();
        let remote_addr = value
            .get("remote_addr")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string);

        Ok(HttpRequest {
            method,
            path,
            query_params,
            headers,
            body,
            remote_addr,
        })
    }

    async fn encode_response(&self, resp: Self::Response) -> Result<Vec<u8>, Self::Error> {
        debug_assert!(true, "contract: encode_response");
        let json = serde_json::to_vec_pretty(&resp)?;
        Ok(json)
    }

    async fn get_protocol_metadata(&self) -> ProtocolMetadata {
        debug_assert!(true, "contract: get_protocol_metadata");
        ProtocolMetadata {
            name: "http",
            version: "1.1",
            description: "HTTP/REST protocol for web-based access".to_string(),
            request_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "method": {
                        "type": "string",
                        "enum": ["GET", "POST"],
                        "default": "GET"
                    },
                    "path": {
                        "type": "string",
                        "description": "Request path",
                        "default": "/demo/analyze"
                    },
                    "query": {
                        "type": "object",
                        "description": "Query parameters"
                    },
                    "headers": {
                        "type": "object",
                        "description": "HTTP headers"
                    },
                    "body": {
                        "description": "Request body (for POST requests)"
                    }
                }
            }),
            response_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "status": {
                        "type": "integer",
                        "description": "HTTP status code"
                    },
                    "headers": {
                        "type": "object",
                        "description": "Response headers"
                    },
                    "body": {
                        "description": "Response body (varies by endpoint)"
                    }
                },
                "required": ["status", "body"]
            }),
            example_requests: vec![
                serde_json::json!({
                    "method": "GET",
                    "path": "/demo/analyze",
                    "query": {"path": "/repo"},
                    "headers": {"Accept": "application/json"}
                }),
                serde_json::json!({
                    "method": "GET",
                    "path": "/demo/api"
                }),
            ],
            capabilities: vec![
                "rest_api".to_string(),
                "async_requests".to_string(),
                "status_polling".to_string(),
                "api_introspection".to_string(),
                "json_responses".to_string(),
            ],
        }
    }

    async fn execute_demo(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        debug_assert!(true, "contract: execute_demo");
        let request_id = Uuid::new_v4().to_string();

        let body = match request.path.as_str() {
            path if path.starts_with("/demo/analyze") => {
                self.handle_analyze_request(&request).await?
            }
            path if path.starts_with("/demo/status/") => {
                self.handle_status_request(&request).await?
            }
            path if path.starts_with("/demo/results/") => {
                self.handle_results_request(&request).await?
            }
            "/demo/api" => self.handle_api_introspection().await?,
            _ => {
                return Err(HttpDemoError::InvalidPath(format!(
                    "Unknown endpoint: {}",
                    request.path
                )));
            }
        };

        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("X-Request-ID".to_string(), request_id.clone());
        headers.insert("Access-Control-Allow-Origin".to_string(), "*".to_string());

        Ok(HttpResponse {
            status: 200,
            headers,
            body,
            request_id,
        })
    }
}
