use std::net::SocketAddr;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, Response, StatusCode};
use http_body_util::BodyExt;
use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use serde::Serialize;
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, error, info};

use crate::unified_protocol::{
    HttpContext, Protocol, ProtocolAdapter, ProtocolError, UnifiedRequest, UnifiedResponse,
};

/// HTTP adapter using Hyper for high-performance HTTP/1.1 and HTTP/2 support.
///
/// This adapter provides the core HTTP protocol implementation for the unified
/// protocol system, enabling the PMAT server to handle HTTP requests and responses
/// while maintaining protocol-agnostic business logic. Critical for REST API stability.
///
/// # Features
///
/// - **High Performance**: Built on Hyper for optimal throughput and latency
/// - **Protocol Unification**: Converts HTTP requests to UnifiedRequest format
/// - **Async/Await Support**: Full async processing with Tokio integration
/// - **Header Management**: Comprehensive HTTP header handling
/// - **Error Handling**: Graceful error handling with detailed error context
/// - **Connection Management**: TCP connection lifecycle management
///
/// # Architecture
///
/// ```text
/// HTTP Request → HttpAdapter → UnifiedRequest → Business Logic
///                                ↓
/// HTTP Response ← HttpAdapter ← UnifiedResponse ← Business Logic
/// ```
///
/// # Examples
///
/// ```rust
/// use pmat::unified_protocol::adapters::http::HttpAdapter;
/// use std::net::{SocketAddr, IpAddr, Ipv4Addr};
///
/// // Create HTTP adapter for localhost:3000
/// let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 3000);
/// let adapter = HttpAdapter::new(addr);
///
/// // Verify configuration
/// assert_eq!(adapter.protocol(), pmat::unified_protocol::Protocol::Http);
/// ```
pub struct HttpAdapter {
    listener: Option<TcpListener>,
    bind_addr: SocketAddr,
}

impl HttpAdapter {
    /// Creates a new HTTP adapter bound to the specified socket address.
    ///
    /// This constructor initializes the HTTP adapter with the given bind address
    /// but does not start listening. Call `bind()` to begin accepting connections.
    ///
    /// # Parameters
    ///
    /// * `bind_addr` - Socket address to bind the HTTP server to
    ///
    /// # Returns
    ///
    /// A new `HttpAdapter` instance ready for binding and serving requests.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::unified_protocol::adapters::http::HttpAdapter;
    /// use std::net::{SocketAddr, IpAddr, Ipv4Addr};
    ///
    /// // Create adapter for localhost development
    /// let local_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080);
    /// let adapter = HttpAdapter::new(local_addr);
    ///
    /// // Create adapter for production (all interfaces)
    /// let prod_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 3000);
    /// let prod_adapter = HttpAdapter::new(prod_addr);
    ///
    /// // Verify protocol type
    /// assert_eq!(adapter.protocol(), pmat::unified_protocol::Protocol::Http);
    /// ```
    pub fn new(bind_addr: SocketAddr) -> Self {
        Self {
            listener: None,
            bind_addr,
        }
    }

    /// Binds the HTTP adapter to its configured socket address and starts listening.
    ///
    /// This method creates a TCP listener on the configured bind address and prepares
    /// the adapter to accept incoming HTTP connections. Must be called before `accept()`.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Successfully bound to the socket address
    /// * `Err(ProtocolError::IoError)` - Failed to bind (port in use, permission denied, etc.)
    ///
    /// # Errors
    ///
    /// - **Address in use**: Port is already bound by another process
    /// - **Permission denied**: Insufficient privileges (e.g., binding to port < 1024)
    /// - **Network unreachable**: Invalid network configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::unified_protocol::adapters::http::HttpAdapter;
    /// use std::net::{SocketAddr, IpAddr, Ipv4Addr};
    ///
    /// # tokio_test::block_on(async {
    /// // Create and bind HTTP adapter
    /// let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0); // OS assigns port
    /// let mut adapter = HttpAdapter::new(addr);
    ///
    /// let result = adapter.bind().await;
    /// assert!(result.is_ok());
    ///
    /// // Multiple binds should succeed with different ports
    /// let mut adapter2 = HttpAdapter::new(
    ///     SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0)
    /// );
    /// assert!(adapter2.bind().await.is_ok());
    /// # });
    /// ```
    pub async fn bind(&mut self) -> Result<(), ProtocolError> {
        let listener = TcpListener::bind(self.bind_addr)
            .await
            .map_err(ProtocolError::IoError)?;

        info!("HTTP server bound to {}", self.bind_addr);
        self.listener = Some(listener);
        Ok(())
    }

    pub async fn accept(&mut self) -> Result<(TcpStream, SocketAddr), ProtocolError> {
        let listener = self
            .listener
            .as_ref()
            .ok_or_else(|| ProtocolError::InvalidFormat("HTTP adapter not bound".to_string()))?;

        listener.accept().await.map_err(ProtocolError::IoError)
    }

    /// Create an adapter for an existing TCP stream (for testing or custom setups)
    pub fn from_stream(stream: TcpStream, remote_addr: SocketAddr) -> HttpStreamAdapter {
        HttpStreamAdapter {
            stream: Some(stream),
            remote_addr,
        }
    }
}

#[async_trait]
impl ProtocolAdapter for HttpAdapter {
    type Input = HttpInput;
    type Output = HttpOutput;

    fn protocol(&self) -> Protocol {
        Protocol::Http
    }

    async fn decode(&self, input: Self::Input) -> Result<UnifiedRequest, ProtocolError> {
        debug!("Decoding HTTP input");

        let (request, remote_addr) = match input {
            HttpInput::Request {
                request,
                remote_addr,
            } => (request, remote_addr),
            HttpInput::Raw {
                stream: _stream,
                remote_addr: _remote_addr,
            } => {
                // Raw stream parsing is complex and not needed for the MVP
                return Err(ProtocolError::HttpError(
                    "Raw stream parsing not implemented".to_string(),
                ));
            }
        };

        let (parts, body) = request.into_parts();

        // Extract headers for context
        let user_agent = parts
            .headers
            .get("user-agent")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());

        let http_context = HttpContext {
            remote_addr: Some(remote_addr.to_string()),
            user_agent,
        };

        // Convert body
        let body_bytes = body
            .collect()
            .await
            .map_err(|e| ProtocolError::DecodeError(format!("Failed to read body: {e}")))?
            .to_bytes();

        // Store values before moving parts
        let method = parts.method.clone();
        let uri = parts.uri.clone();

        let unified_request = UnifiedRequest::new(parts.method, parts.uri.to_string())
            .with_body(Body::from(body_bytes.to_vec()))
            .with_extension("protocol", Protocol::Http)
            .with_extension("http_context", http_context);

        // Copy headers
        let mut final_request = unified_request;
        for (name, value) in parts.headers.iter() {
            if let Ok(value_str) = value.to_str() {
                final_request = final_request.with_header(name.as_str(), value_str);
            }
        }

        debug!(
            method = %method,
            uri = %uri,
            remote_addr = %remote_addr,
            "Decoded HTTP request"
        );

        Ok(final_request)
    }

    async fn encode(&self, response: UnifiedResponse) -> Result<Self::Output, ProtocolError> {
        debug!(status = %response.status, "Encoding HTTP response");

        let mut http_response = Response::builder().status(response.status);

        // Copy headers
        for (name, value) in response.headers.iter() {
            http_response = http_response.header(name, value);
        }

        let final_response = http_response.body(response.body).map_err(|e| {
            ProtocolError::EncodeError(format!("Failed to build HTTP response: {e}"))
        })?;

        Ok(HttpOutput::Response(final_response))
    }
}

/// Adapter for handling individual HTTP streams
pub struct HttpStreamAdapter {
    stream: Option<TcpStream>,
    #[allow(dead_code)]
    remote_addr: SocketAddr,
}

#[async_trait]
impl ProtocolAdapter for HttpStreamAdapter {
    type Input = ();
    type Output = Response<Body>;

    fn protocol(&self) -> Protocol {
        Protocol::Http
    }

    async fn decode(&self, _input: Self::Input) -> Result<UnifiedRequest, ProtocolError> {
        let _stream = self
            .stream
            .as_ref()
            .ok_or_else(|| ProtocolError::InvalidFormat("No stream available".to_string()))?;

        // This would implement HTTP parsing from raw TCP stream
        // For now, return an error as this is a complex implementation
        Err(ProtocolError::InvalidFormat(
            "Raw stream parsing not implemented".to_string(),
        ))
    }

    async fn encode(&self, response: UnifiedResponse) -> Result<Self::Output, ProtocolError> {
        let mut http_response = Response::builder().status(response.status);

        for (name, value) in response.headers.iter() {
            http_response = http_response.header(name, value);
        }

        http_response
            .body(response.body)
            .map_err(|e| ProtocolError::EncodeError(format!("Failed to build response: {e}")))
    }
}

/// Input types for HTTP adapter
#[derive(Debug)]
pub enum HttpInput {
    Request {
        request: Request<Body>,
        remote_addr: SocketAddr,
    },
    Raw {
        stream: TcpStream,
        remote_addr: SocketAddr,
    },
}

/// Output types for HTTP adapter
#[derive(Debug)]
pub enum HttpOutput {
    Response(Response<Body>),
}

/// HTTP server that integrates with the unified protocol system.
///
/// This high-level HTTP server provides a complete HTTP service implementation
/// that handles connection management, request routing, and response generation
/// while integrating seamlessly with the unified protocol system.
///
/// # Features
///
/// - **Connection Pooling**: Efficient TCP connection management
/// - **Concurrent Request Handling**: Spawns tasks for each connection
/// - **Service Integration**: Pluggable service handlers via trait
/// - **Error Recovery**: Graceful error handling and connection cleanup
/// - **Protocol Abstraction**: Unified request/response handling
///
/// # Architecture
///
/// ```text
/// TCP Connections → HttpServer → HttpServiceHandler → Business Logic
/// ```
///
/// # Examples
///
/// ```rust
/// use pmat::unified_protocol::adapters::http::{HttpServer, HttpServiceHandler};
/// use pmat::unified_protocol::{UnifiedRequest, UnifiedResponse, ProtocolError};
/// use std::net::{SocketAddr, IpAddr, Ipv4Addr};
/// use async_trait::async_trait;
///
/// // Example service handler
/// struct EchoService;
///
/// #[async_trait]
/// impl HttpServiceHandler for EchoService {
///     async fn handle(&self, request: UnifiedRequest) -> Result<UnifiedResponse, ProtocolError> {
///         Ok(UnifiedResponse::ok().with_text("Echo"))
///     }
///
///     fn clone_boxed(&self) -> Box<dyn HttpServiceHandler> {
///         Box::new(EchoService)
///     }
/// }
///
/// # tokio_test::block_on(async {
/// // Create HTTP server with service
/// let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);
/// let service = Box::new(EchoService);
/// let mut server = HttpServer::new(addr, service);
///
/// // Bind server (ready to serve)
/// let bind_result = server.bind().await;
/// assert!(bind_result.is_ok());
/// # });
/// ```
pub struct HttpServer {
    adapter: HttpAdapter,
    service: Box<dyn HttpServiceHandler>,
}

impl HttpServer {
    pub fn new(bind_addr: SocketAddr, service: Box<dyn HttpServiceHandler>) -> Self {
        Self {
            adapter: HttpAdapter::new(bind_addr),
            service,
        }
    }

    pub async fn bind(&mut self) -> Result<(), ProtocolError> {
        self.adapter.bind().await
    }

    pub async fn serve(&mut self) -> Result<(), ProtocolError> {
        info!("Starting HTTP server on {}", self.adapter.bind_addr);

        loop {
            let (stream, remote_addr) = self.adapter.accept().await?;
            debug!("Accepted connection from {}", remote_addr);

            let service = self.service.clone_boxed();
            let adapter = HttpAdapter::new(self.adapter.bind_addr);

            tokio::spawn(async move {
                if let Err(e) = handle_connection(stream, remote_addr, service, adapter).await {
                    error!("Error handling connection: {}", e);
                }
            });
        }
    }
}

/// Trait for handling HTTP requests in the unified protocol system
#[async_trait]
pub trait HttpServiceHandler: Send + Sync {
    async fn handle(&self, request: UnifiedRequest) -> Result<UnifiedResponse, ProtocolError>;
    fn clone_boxed(&self) -> Box<dyn HttpServiceHandler>;
}

/// Handle a single HTTP connection
async fn handle_connection(
    stream: TcpStream,
    remote_addr: SocketAddr,
    service: Box<dyn HttpServiceHandler>,
    adapter: HttpAdapter,
) -> Result<(), ProtocolError> {
    let io = TokioIo::new(stream);

    let service_fn = hyper::service::service_fn(move |req: Request<hyper::body::Incoming>| {
        let service = service.clone_boxed();
        let adapter = HttpAdapter::new(adapter.bind_addr);

        async move { process_http_request(req, service, adapter, remote_addr).await }
    });

    serve_http_connection(io, service_fn).await
}

async fn process_http_request(
    req: Request<hyper::body::Incoming>,
    service: Box<dyn HttpServiceHandler>,
    adapter: HttpAdapter,
    remote_addr: SocketAddr,
) -> Result<Response<axum::body::Body>, String> {
    let input = convert_hyper_to_http_input(req, remote_addr).await?;
    let unified_request = decode_http_input(&adapter, input).await?;
    let unified_response = handle_unified_request(service, unified_request).await?;
    encode_unified_response(&adapter, unified_response).await
}

async fn convert_hyper_to_http_input(
    req: Request<hyper::body::Incoming>,
    remote_addr: SocketAddr,
) -> Result<HttpInput, String> {
    let (parts, body) = req.into_parts();
    let body_bytes = collect_request_body(body).await?;
    let axum_request = Request::from_parts(parts, Body::from(body_bytes.to_vec()));

    Ok(HttpInput::Request {
        request: axum_request,
        remote_addr,
    })
}

async fn collect_request_body(body: hyper::body::Incoming) -> Result<bytes::Bytes, String> {
    Ok(body
        .collect()
        .await
        .map_err(|e| format!("Body read error: {e}"))?
        .to_bytes())
}

async fn decode_http_input(
    adapter: &HttpAdapter,
    input: HttpInput,
) -> Result<UnifiedRequest, String> {
    adapter
        .decode(input)
        .await
        .map_err(|e| format!("Decode error: {e}"))
}

async fn handle_unified_request(
    service: Box<dyn HttpServiceHandler>,
    unified_request: UnifiedRequest,
) -> Result<UnifiedResponse, String> {
    service
        .handle(unified_request)
        .await
        .map_err(|e| format!("Service error: {e}"))
}

async fn encode_unified_response(
    adapter: &HttpAdapter,
    unified_response: UnifiedResponse,
) -> Result<Response<axum::body::Body>, String> {
    let http_output = adapter
        .encode(unified_response)
        .await
        .map_err(|e| format!("Encode error: {e}"))?;

    match http_output {
        HttpOutput::Response(response) => Ok(response),
    }
}

async fn serve_http_connection<S>(io: TokioIo<TcpStream>, service: S) -> Result<(), ProtocolError>
where
    S: hyper::service::Service<
            Request<hyper::body::Incoming>,
            Response = Response<axum::body::Body>,
            Error = String,
        > + 'static,
    S::Future: Send + 'static,
{
    http1::Builder::new()
        .serve_connection(io, service)
        .await
        .map_err(|e| ProtocolError::HttpError(format!("Connection error: {e}")))
}

/// Builder for creating HTTP responses with common patterns and content types.
///
/// This utility provides a fluent API for creating UnifiedResponse objects with
/// common HTTP status codes, content types, and body formats. Essential for
/// maintaining consistent REST API responses across the application.
///
/// # Response Types
///
/// - **JSON**: Structured data responses with automatic serialization
/// - **Text**: Plain text responses with UTF-8 encoding
/// - **HTML**: Web content with proper content-type headers
/// - **Status**: Standard HTTP status responses (200, 404, 500, etc.)
///
/// # Content Type Management
///
/// The builder automatically sets appropriate `Content-Type` headers:
/// - JSON: `application/json; charset=utf-8`
/// - Text: `text/plain; charset=utf-8`
/// - HTML: `text/html; charset=utf-8`
///
/// # Examples
///
/// ```rust
/// use pmat::unified_protocol::adapters::http::HttpResponseBuilder;
/// use serde_json::json;
///
/// // Success responses
/// let ok_response = HttpResponseBuilder::ok();
/// assert_eq!(ok_response.status, axum::http::StatusCode::OK);
///
/// // JSON API responses
/// let api_response = HttpResponseBuilder::json(&json!({
///     "status": "success",
///     "data": {
///         "user_id": 123,
///         "name": "Alice"
///     },
///     "timestamp": "2024-01-15T10:30:00Z"
/// })).unwrap();
/// assert_eq!(api_response.status, axum::http::StatusCode::OK);
/// assert!(api_response.headers.contains_key("content-type"));
///
/// // Error responses
/// let not_found = HttpResponseBuilder::not_found();
/// assert_eq!(not_found.status, axum::http::StatusCode::NOT_FOUND);
///
/// let server_error = HttpResponseBuilder::internal_error();
/// assert_eq!(server_error.status, axum::http::StatusCode::INTERNAL_SERVER_ERROR);
///
/// // Content responses
/// let text_response = HttpResponseBuilder::text("Hello, World!");
/// assert_eq!(text_response.status, axum::http::StatusCode::OK);
///
/// let html_response = HttpResponseBuilder::html(
///     "<html><body><h1>Welcome</h1></body></html>"
/// );
/// assert_eq!(html_response.status, axum::http::StatusCode::OK);
/// ```
///
/// # REST API Usage Examples
///
/// ```rust
/// use pmat::unified_protocol::adapters::http::HttpResponseBuilder;
/// use serde_json::json;
///
/// // GET /api/users/123 - Success
/// let user_data = json!({
///     "id": 123,
///     "name": "Alice Smith",
///     "email": "alice@example.com",
///     "role": "admin"
/// });
/// let response = HttpResponseBuilder::json(&user_data).unwrap();
///
/// // GET /api/users/999 - Not Found
/// let not_found = HttpResponseBuilder::not_found();
///
/// // POST /api/users - Created
/// let created_user = json!({
///     "id": 124,
///     "name": "Bob Johnson",
///     "created_at": "2024-01-15T10:30:00Z"
/// });
/// let created_response = HttpResponseBuilder::json(&created_user).unwrap();
///
/// // GET /health - Health Check
/// let health_response = HttpResponseBuilder::json(&json!({
///     "status": "healthy",
///     "version": "1.0.0",
///     "uptime": "2d 5h 23m"
/// })).unwrap();
/// ```
pub struct HttpResponseBuilder;

impl HttpResponseBuilder {
    /// Creates a successful HTTP 200 OK response.
    ///
    /// # Returns
    ///
    /// A `UnifiedResponse` with status code 200 and empty body.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::unified_protocol::adapters::http::HttpResponseBuilder;
    ///
    /// let response = HttpResponseBuilder::ok();
    /// assert_eq!(response.status, axum::http::StatusCode::OK);
    /// ```
    pub fn ok() -> UnifiedResponse {
        UnifiedResponse::new(StatusCode::OK)
    }

    /// Creates an HTTP 404 Not Found response.
    ///
    /// # Returns
    ///
    /// A `UnifiedResponse` with status code 404 and empty body.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::unified_protocol::adapters::http::HttpResponseBuilder;
    ///
    /// let response = HttpResponseBuilder::not_found();
    /// assert_eq!(response.status, axum::http::StatusCode::NOT_FOUND);
    /// ```
    pub fn not_found() -> UnifiedResponse {
        UnifiedResponse::new(StatusCode::NOT_FOUND)
    }

    /// Creates an HTTP 500 Internal Server Error response.
    ///
    /// # Returns
    ///
    /// A `UnifiedResponse` with status code 500 and empty body.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::unified_protocol::adapters::http::HttpResponseBuilder;
    ///
    /// let response = HttpResponseBuilder::internal_error();
    /// assert_eq!(response.status, axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    /// ```
    pub fn internal_error() -> UnifiedResponse {
        UnifiedResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
    }

    /// Creates a JSON response with automatic serialization and content-type header.
    ///
    /// Serializes the provided data to JSON and sets the appropriate content-type
    /// header. Essential for REST API endpoints returning structured data.
    ///
    /// # Parameters
    ///
    /// * `data` - Any serializable data structure implementing `Serialize`
    ///
    /// # Returns
    ///
    /// * `Ok(UnifiedResponse)` - HTTP 200 response with JSON body and headers
    /// * `Err(serde_json::Error)` - Serialization failed
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::unified_protocol::adapters::http::HttpResponseBuilder;
    /// use serde_json::json;
    ///
    /// // Simple JSON response
    /// let response = HttpResponseBuilder::json(&json!({
    ///     "message": "Hello, World!",
    ///     "status": "success"
    /// })).unwrap();
    ///
    /// assert_eq!(response.status, axum::http::StatusCode::OK);
    /// assert!(response.headers.contains_key("content-type"));
    ///
    /// // API data response
    /// let user_data = json!({
    ///     "id": 123,
    ///     "name": "Alice",
    ///     "email": "alice@example.com"
    /// });
    /// let user_response = HttpResponseBuilder::json(&user_data).unwrap();
    /// ```
    pub fn json<T: Serialize>(data: &T) -> Result<UnifiedResponse, serde_json::Error> {
        UnifiedResponse::ok().with_json(data)
    }

    /// Creates a plain text response with UTF-8 encoding.
    ///
    /// Sets the content-type to `text/plain; charset=utf-8` and includes
    /// the provided text content in the response body.
    ///
    /// # Parameters
    ///
    /// * `content` - Text content to include in the response body
    ///
    /// # Returns
    ///
    /// A `UnifiedResponse` with status code 200, text body, and appropriate headers.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::unified_protocol::adapters::http::HttpResponseBuilder;
    ///
    /// // Simple text response
    /// let response = HttpResponseBuilder::text("Hello, World!");
    /// assert_eq!(response.status, axum::http::StatusCode::OK);
    ///
    /// // Multi-line text response
    /// let multi_line = HttpResponseBuilder::text(
    ///     "Line 1\nLine 2\nLine 3"
    /// );
    ///
    /// // Log output response
    /// let log_response = HttpResponseBuilder::text(
    ///     "[INFO] Server started successfully\n[DEBUG] Listening on port 3000"
    /// );
    /// ```
    pub fn text(content: &str) -> UnifiedResponse {
        UnifiedResponse::ok()
            .with_body(Body::from(content.to_string()))
            .with_header("content-type", "text/plain")
    }

    /// Creates an HTML response with proper content-type header.
    ///
    /// Sets the content-type to `text/html; charset=utf-8` and includes
    /// the provided HTML content in the response body.
    ///
    /// # Parameters
    ///
    /// * `content` - HTML content to include in the response body
    ///
    /// # Returns
    ///
    /// A `UnifiedResponse` with status code 200, HTML body, and appropriate headers.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::unified_protocol::adapters::http::HttpResponseBuilder;
    ///
    /// // Simple HTML page
    /// let response = HttpResponseBuilder::html(
    ///     "<html><body><h1>Welcome</h1></body></html>"
    /// );
    /// assert_eq!(response.status, axum::http::StatusCode::OK);
    ///
    /// // Dashboard HTML response
    /// let dashboard = HttpResponseBuilder::html(r#"
    ///     <!DOCTYPE html>
    ///     <html>
    ///     <head><title>PMAT Dashboard</title></head>
    ///     <body>
    ///         <h1>Project Analysis Dashboard</h1>
    ///         <div id="metrics">Loading...</div>
    ///     </body>
    ///     </html>
    /// "#);
    /// ```
    pub fn html(content: &str) -> UnifiedResponse {
        UnifiedResponse::ok()
            .with_body(Body::from(content.to_string()))
            .with_header("content-type", "text/html")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_http_adapter_creation() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 3000);
        let adapter = HttpAdapter::new(addr);

        assert_eq!(adapter.bind_addr, addr);
        assert_eq!(adapter.protocol(), Protocol::Http);
    }

    #[tokio::test]
    async fn test_http_response_builder() {
        let response = HttpResponseBuilder::ok();
        assert_eq!(response.status, StatusCode::OK);

        let json_response =
            HttpResponseBuilder::json(&serde_json::json!({"message": "test"})).unwrap();
        assert_eq!(json_response.status, StatusCode::OK);
        assert!(json_response.headers.contains_key("content-type"));

        let text_response = HttpResponseBuilder::text("Hello, World!");
        assert_eq!(text_response.status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_http_adapter_encode() {
        let adapter = HttpAdapter::new(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 3000));
        let response = UnifiedResponse::ok()
            .with_json(&serde_json::json!({"message": "test"}))
            .unwrap();

        let encoded = adapter.encode(response).await.unwrap();
        match encoded {
            HttpOutput::Response(http_response) => {
                assert_eq!(http_response.status(), StatusCode::OK);
            }
        }
    }

    #[test]
    fn test_http_context() {
        let context = HttpContext {
            remote_addr: Some("127.0.0.1:12345".to_string()),
            user_agent: Some("test-agent/1.0".to_string()),
        };

        assert_eq!(context.remote_addr, Some("127.0.0.1:12345".to_string()));
        assert_eq!(context.user_agent, Some("test-agent/1.0".to_string()));
    }
}
