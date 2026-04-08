/// Registry for managing protocol adapters
#[derive(Default)]
pub struct AdapterRegistry {
    adapters: HashMap<Protocol, Arc<dyn ProtocolAdapter<Input = Value, Output = Value>>>,
}

impl AdapterRegistry {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new item.
    pub fn register<A>(&mut self, adapter: A) -> &mut Self
    where
        A: ProtocolAdapter + 'static,
        A::Input: Into<Value> + for<'de> Deserialize<'de>,
        A::Output: From<Value> + Serialize,
    {
        let protocol = adapter.protocol();
        let wrapped = Arc::new(AdapterWrapper::new(adapter));
        self.adapters.insert(protocol, wrapped);
        self
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Get a handler by method name.
    pub fn get(
        &self,
        protocol: Protocol,
    ) -> Option<&Arc<dyn ProtocolAdapter<Input = Value, Output = Value>>> {
        self.adapters.get(&protocol)
    }
}

/// Wrapper to handle type erasure for different adapter types
struct AdapterWrapper<A> {
    inner: A,
}

impl<A> AdapterWrapper<A> {
    fn new(adapter: A) -> Self {
        Self { inner: adapter }
    }
}

#[async_trait]
impl<A> ProtocolAdapter for AdapterWrapper<A>
where
    A: ProtocolAdapter + Send + Sync + 'static,
    A::Input: for<'de> Deserialize<'de> + Send + 'static,
    A::Output: Serialize + Send + 'static,
{
    type Input = Value;
    type Output = Value;

    async fn decode(&self, input: Self::Input) -> Result<UnifiedRequest, ProtocolError> {
        let typed_input: A::Input =
            serde_json::from_value(input).map_err(|e| ProtocolError::DecodeError(e.to_string()))?;
        self.inner.decode(typed_input).await
    }

    async fn encode(&self, response: UnifiedResponse) -> Result<Self::Output, ProtocolError> {
        let output = self.inner.encode(response).await?;
        serde_json::to_value(output).map_err(|e| ProtocolError::EncodeError(e.to_string()))
    }

    fn protocol(&self) -> Protocol {
        self.inner.protocol()
    }
}
