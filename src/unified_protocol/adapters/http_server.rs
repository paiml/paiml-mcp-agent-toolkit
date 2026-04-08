/// Adapter for handling individual HTTP streams
pub struct HttpStreamAdapter {
    stream: Option<TcpStream>,
    
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

        for (name, value) in &response.headers {
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
/// ```ignore
///
/// # Examples
///
/// ```rust,ignore
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
/// ```ignore
pub struct HttpServer {
    adapter: HttpAdapter,
    service: Box<dyn HttpServiceHandler>,
}

impl HttpServer {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(bind_addr: SocketAddr, service: Box<dyn HttpServiceHandler>) -> Self {
        Self {
            adapter: HttpAdapter::new(bind_addr),
            service,
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
