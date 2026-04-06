// Protocol harness execution — DemoEngine impl methods
// Included by protocol_harness.rs — no `use` imports or `#!` attributes allowed

impl Default for DemoEngine {
    fn default() -> Self {
        Self::with_config(DemoConfig::default())
    }
}

impl DemoEngine {
    /// Create a new demo engine with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new demo engine with custom configuration
    #[must_use]
    pub fn with_config(config: DemoConfig) -> Self {
        Self {
            context_cache: Arc::new(RwLock::new(ContextCache::new(config.clone()))),
            protocols: HashMap::new(),
            trace_store: Arc::new(TraceStore::new(1000)),
            config,
        }
    }

    /// Register a protocol adapter
    pub fn register_protocol<P>(&mut self, name: String, protocol: P) -> &mut Self
    where
        P: DemoProtocol + 'static,
        P::Request: From<Value> + Serialize,
        P::Response: Into<Value> + for<'de> Deserialize<'de>,
    {
        let wrapped = ProtocolWrapper::new(protocol);
        self.protocols.insert(name, Box::new(wrapped));
        self
    }

    /// Get list of registered protocols
    #[must_use]
    pub fn list_protocols(&self) -> Vec<String> {
        self.protocols.keys().cloned().collect()
    }

    /// Get metadata for a specific protocol
    pub async fn get_protocol_metadata(&self, name: &str) -> Result<ProtocolMetadata, DemoError> {
        debug_assert!(!name.is_empty(), "name must not be empty");
        let protocol = self
            .protocols
            .get(name)
            .ok_or_else(|| DemoError::ProtocolNotFound(name.to_string()))?;

        Ok(protocol.get_protocol_metadata().await)
    }

    /// Execute demo analysis through specified protocol
    pub async fn execute_demo(
        &self,
        protocol_name: &str,
        request: Value,
    ) -> Result<ApiTrace, DemoError> {
        debug_assert!(!protocol_name.is_empty(), "protocol_name must not be empty");
        let trace_id = Uuid::new_v4();
        let start_time = std::time::Instant::now();

        let protocol = self
            .protocols
            .get(protocol_name)
            .ok_or_else(|| DemoError::ProtocolNotFound(protocol_name.to_string()))?;

        // Record trace start
        let mut timing = TimingInfo {
            request_decode_ns: 0,
            cache_lookup_ns: 0,
            analysis_ms: 0,
            response_encode_ns: 0,
            total_ms: 0,
        };

        // Decode request
        let decode_start = std::time::Instant::now();
        let request_bytes = serde_json::to_vec(&request)?;
        let parsed_request = protocol
            .decode_request(&request_bytes)
            .await
            .map_err(|e| DemoError::AnalysisFailed(e.to_string()))?;
        timing.request_decode_ns = decode_start.elapsed().as_nanos() as u64;

        // Execute demo
        let analysis_start = std::time::Instant::now();
        let response = protocol
            .execute_demo(parsed_request)
            .await
            .map_err(|e| DemoError::AnalysisFailed(e.to_string()))?;
        timing.analysis_ms = analysis_start.elapsed().as_millis() as u64;

        // Encode response
        let encode_start = std::time::Instant::now();
        let response_bytes = protocol
            .encode_response(response)
            .await
            .map_err(|e| DemoError::AnalysisFailed(e.to_string()))?;
        let response_value: Value = serde_json::from_slice(&response_bytes)?;
        timing.response_encode_ns = encode_start.elapsed().as_nanos() as u64;

        timing.total_ms = start_time.elapsed().as_millis() as u64;

        // Create trace
        let trace = ApiTrace {
            id: trace_id,
            protocol: protocol_name.to_string(),
            request_raw: request_bytes,
            request_parsed: request,
            internal_command: vec![
                "paiml-mcp-agent-toolkit".to_string(),
                "analyze".to_string(),
                "context".to_string(),
            ],
            timing,
            response: response_value,
            cache_hit: false,
            created_at: chrono::Utc::now(),
        };

        // Store trace
        self.trace_store.add_trace(trace.clone()).await;

        Ok(trace)
    }

    /// Get API trace by ID
    pub async fn get_trace(&self, trace_id: Uuid) -> Option<ApiTrace> {
        self.trace_store.get_trace(trace_id).await
    }

    /// Get all traces for introspection
    pub async fn get_all_traces(&self) -> Vec<ApiTrace> {
        self.trace_store.get_all_traces().await
    }
}
