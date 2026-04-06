/// HTTP adapter using Hyper for high-performance HTTP/1.1 and HTTP/2 support.
///
/// This adapter provides the core HTTP protocol implementation for the unified
/// protocol system, enabling the PMAT server to handle HTTP requests and responses
/// while maintaining protocol-agnostic business logic. Critical for REST API stability.
///
/// # Features
///
/// - **High Performance**: Built on Hyper for optimal throughput and latency
/// - **Protocol Unification**: Converts HTTP requests to `UnifiedRequest` format
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
/// ```ignore
///
/// # Examples
///
/// ```ignore
/// use pmat::unified_protocol::adapters::http::HttpAdapter;
/// use pmat::unified_protocol::ProtocolAdapter;
/// use std::net::{SocketAddr, IpAddr, Ipv4Addr};
///
/// // Create HTTP adapter for localhost:3000
/// let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 3000);
/// let adapter = HttpAdapter::new(addr);
///
/// // Verify configuration
/// assert_eq!(adapter.protocol(), pmat::unified_protocol::Protocol::Http);
/// ```ignore
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
    /// use pmat::unified_protocol::ProtocolAdapter;
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
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
    /// ```no_run
    /// use pmat::unified_protocol::adapters::http::HttpAdapter;
    /// use std::net::{SocketAddr, IpAddr, Ipv4Addr};
    ///
    /// # tokio::runtime::Runtime::new().expect("runtime").block_on(async {
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn bind(&mut self) -> Result<(), ProtocolError> {
        let listener = TcpListener::bind(self.bind_addr)
            .await
            .map_err(ProtocolError::IoError)?;

        info!("HTTP server bound to {}", self.bind_addr);
        self.listener = Some(listener);
        Ok(())
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn accept(&mut self) -> Result<(TcpStream, SocketAddr), ProtocolError> {
        let listener = self
            .listener
            .as_ref()
            .ok_or_else(|| ProtocolError::InvalidFormat("HTTP adapter not bound".to_string()))?;

        listener.accept().await.map_err(ProtocolError::IoError)
    }

    /// Create an adapter for an existing TCP stream (for testing or custom setups)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
            .map(std::string::ToString::to_string);

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
        for (name, value) in &parts.headers {
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
        for (name, value) in &response.headers {
            http_response = http_response.header(name, value);
        }

        let final_response = http_response.body(response.body).map_err(|e| {
            ProtocolError::EncodeError(format!("Failed to build HTTP response: {e}"))
        })?;

        Ok(HttpOutput::Response(final_response))
    }
}
