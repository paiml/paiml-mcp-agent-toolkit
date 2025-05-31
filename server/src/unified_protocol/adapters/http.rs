use std::net::SocketAddr;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, Response, StatusCode};
use http_body_util::BodyExt;
use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use serde::Serialize;
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, error, info, instrument};

use crate::unified_protocol::{
    HttpContext, Protocol, ProtocolAdapter, ProtocolError, UnifiedRequest, UnifiedResponse,
};

/// HTTP adapter using Hyper for high-performance HTTP/1.1 and HTTP/2 support
pub struct HttpAdapter {
    listener: Option<TcpListener>,
    bind_addr: SocketAddr,
}

impl HttpAdapter {
    pub fn new(bind_addr: SocketAddr) -> Self {
        Self {
            listener: None,
            bind_addr,
        }
    }

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

    #[instrument(skip_all)]
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
            .map_err(|e| ProtocolError::DecodeError(format!("Failed to read body: {}", e)))?
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

    #[instrument(skip_all)]
    async fn encode(&self, response: UnifiedResponse) -> Result<Self::Output, ProtocolError> {
        debug!(status = %response.status, "Encoding HTTP response");

        let mut http_response = Response::builder().status(response.status);

        // Copy headers
        for (name, value) in response.headers.iter() {
            http_response = http_response.header(name, value);
        }

        let final_response = http_response.body(response.body).map_err(|e| {
            ProtocolError::EncodeError(format!("Failed to build HTTP response: {}", e))
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
            .map_err(|e| ProtocolError::EncodeError(format!("Failed to build response: {}", e)))
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

/// HTTP server that integrates with the unified protocol system
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

    #[instrument(skip_all)]
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
#[instrument(skip_all)]
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

        async move {
            // Convert Hyper request to our format
            let (parts, body) = req.into_parts();
            let body_bytes = body
                .collect()
                .await
                .map_err(|e| format!("Body read error: {}", e))?
                .to_bytes();

            let axum_request = Request::from_parts(parts, Body::from(body_bytes.to_vec()));

            let input = HttpInput::Request {
                request: axum_request,
                remote_addr,
            };

            // Decode to unified request
            let unified_request = adapter
                .decode(input)
                .await
                .map_err(|e| format!("Decode error: {}", e))?;

            // Process request
            let unified_response = service
                .handle(unified_request)
                .await
                .map_err(|e| format!("Service error: {}", e))?;

            // Encode response
            let http_output = adapter
                .encode(unified_response)
                .await
                .map_err(|e| format!("Encode error: {}", e))?;

            match http_output {
                HttpOutput::Response(response) => {
                    Ok::<Response<axum::body::Body>, String>(response)
                }
            }
        }
    });

    http1::Builder::new()
        .serve_connection(io, service_fn)
        .await
        .map_err(|e| ProtocolError::HttpError(format!("Connection error: {}", e)))?;

    Ok(())
}

/// Helper to create HTTP responses with common patterns
pub struct HttpResponseBuilder;

impl HttpResponseBuilder {
    pub fn ok() -> UnifiedResponse {
        UnifiedResponse::new(StatusCode::OK)
    }

    pub fn not_found() -> UnifiedResponse {
        UnifiedResponse::new(StatusCode::NOT_FOUND)
    }

    pub fn internal_error() -> UnifiedResponse {
        UnifiedResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub fn json<T: Serialize>(data: &T) -> Result<UnifiedResponse, serde_json::Error> {
        UnifiedResponse::ok().with_json(data)
    }

    pub fn text(content: &str) -> UnifiedResponse {
        UnifiedResponse::ok()
            .with_body(Body::from(content.to_string()))
            .with_header("content-type", "text/plain")
    }

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
