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
