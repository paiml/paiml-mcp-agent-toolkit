// Protocol harness wrapper — BoxedError impls and type-erased ProtocolWrapper
// Included by protocol_harness.rs — no `use` imports or `#!` attributes allowed

impl std::fmt::Display for BoxedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for BoxedError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.source()
    }
}

impl From<serde_json::Error> for BoxedError {
    fn from(err: serde_json::Error) -> Self {
        BoxedError(Box::new(err))
    }
}

impl From<std::io::Error> for BoxedError {
    fn from(err: std::io::Error) -> Self {
        BoxedError(Box::new(err))
    }
}

impl BoxedError {
    /// Create a `BoxedError` from any error type
    pub fn new<E: std::error::Error + Send + Sync + 'static>(err: E) -> Self {
        BoxedError(Box::new(err))
    }

    /// Create a `BoxedError` from a boxed error
    #[must_use]
    pub fn from_box(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        BoxedError(err)
    }
}

/// Type-erased wrapper for protocol implementations
struct ProtocolWrapper<P> {
    inner: P,
}

impl<P> ProtocolWrapper<P> {
    fn new(protocol: P) -> Self {
        Self { inner: protocol }
    }
}

#[async_trait]
impl<P> DemoProtocol for ProtocolWrapper<P>
where
    P: DemoProtocol + Send + Sync + 'static,
    P::Request: From<Value> + Serialize + Send + 'static,
    P::Response: Into<Value> + for<'de> Deserialize<'de> + Send + 'static,
    P::Error: 'static,
{
    type Request = Value;
    type Response = Value;
    type Error = BoxedError;

    async fn decode_request(&self, raw: &[u8]) -> Result<Self::Request, Self::Error> {
        let value: Value = serde_json::from_slice(raw)?;
        Ok(value)
    }

    async fn encode_response(&self, resp: Self::Response) -> Result<Vec<u8>, Self::Error> {
        Ok(serde_json::to_vec(&resp)?)
    }

    async fn get_protocol_metadata(&self) -> ProtocolMetadata {
        self.inner.get_protocol_metadata().await
    }

    async fn execute_demo(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        let typed_request = P::Request::from(request);
        let response = self.inner.execute_demo(typed_request).await.map_err(|e| {
            BoxedError::from_box(Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
        })?;
        Ok(response.into())
    }
}
