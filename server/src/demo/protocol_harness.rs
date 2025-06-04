use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Core trait for protocol-agnostic demo functionality
#[async_trait]
pub trait DemoProtocol: Send + Sync {
    type Request: Send + 'static;
    type Response: Send + 'static;
    type Error: std::error::Error + Send + Sync + 'static;

    /// Decode raw bytes into protocol-specific request
    async fn decode_request(&self, raw: &[u8]) -> Result<Self::Request, Self::Error>;

    /// Encode protocol-specific response into raw bytes
    async fn encode_response(&self, resp: Self::Response) -> Result<Vec<u8>, Self::Error>;

    /// Get metadata about this protocol
    async fn get_protocol_metadata(&self) -> ProtocolMetadata;

    /// Execute the demo analysis for this protocol
    async fn execute_demo(&self, request: Self::Request) -> Result<Self::Response, Self::Error>;
}

/// Metadata describing a protocol's capabilities and interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolMetadata {
    pub name: &'static str,
    pub version: &'static str,
    pub description: String,
    pub request_schema: Value,
    pub response_schema: Value,
    pub example_requests: Vec<Value>,
    pub capabilities: Vec<String>,
}

/// Unified demo engine that coordinates multiple protocols
pub struct DemoEngine {
    /// Cached context analysis results
    #[allow(dead_code)]
    context_cache: Arc<RwLock<ContextCache>>,
    /// Registered protocol adapters
    protocols: HashMap<
        String,
        Box<dyn DemoProtocol<Request = Value, Response = Value, Error = BoxedError>>,
    >,
    /// Trace storage for API introspection
    trace_store: Arc<TraceStore>,
    /// Configuration settings
    #[allow(dead_code)]
    config: DemoConfig,
}

/// Configuration for the demo engine
#[derive(Debug, Clone)]
pub struct DemoConfig {
    pub cache_ttl_minutes: u64,
    pub max_cache_entries: usize,
    pub enable_file_watcher: bool,
    pub default_analysis_timeout_ms: u64,
}

impl Default for DemoConfig {
    fn default() -> Self {
        Self {
            cache_ttl_minutes: 5,
            max_cache_entries: 100,
            enable_file_watcher: true,
            default_analysis_timeout_ms: 30_000,
        }
    }
}

/// Cache for storing analysis results
pub struct ContextCache {
    entries: HashMap<String, CacheEntry>,
    config: DemoConfig,
}

/// Individual cache entry with metadata
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub key: String,
    pub result: AnalysisResult,
    pub created_at: std::time::Instant,
    pub access_count: u64,
    pub last_accessed: std::time::Instant,
}

/// Result of a context analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub request_id: Uuid,
    pub status: AnalysisStatus,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub duration_ms: Option<u64>,
    pub cache_key: String,
    pub path: String,
    pub context_data: Option<Value>,
    pub error: Option<String>,
}

/// Status of an analysis request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnalysisStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cached,
}

/// Storage for API traces and introspection data
pub struct TraceStore {
    traces: RwLock<HashMap<Uuid, ApiTrace>>,
    max_traces: usize,
}

/// Detailed trace of an API request through the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiTrace {
    pub id: Uuid,
    pub protocol: String,
    pub request_raw: Vec<u8>,
    pub request_parsed: Value,
    pub internal_command: Vec<String>,
    pub timing: TimingInfo,
    pub response: Value,
    pub cache_hit: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Detailed timing information for performance analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingInfo {
    pub request_decode_ns: u64,
    pub cache_lookup_ns: u64,
    pub analysis_ms: u64,
    pub response_encode_ns: u64,
    pub total_ms: u64,
}

/// Error types for the demo engine
#[derive(Debug, Error)]
pub enum DemoError {
    #[error("Protocol not found: {0}")]
    ProtocolNotFound(String),

    #[error("Analysis failed: {0}")]
    AnalysisFailed(String),

    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Timeout error: analysis took longer than {timeout_ms}ms")]
    TimeoutError { timeout_ms: u64 },

    #[error("Invalid path: {0}")]
    InvalidPath(String),
}

/// Wrapper type for boxed errors that implements Error trait
#[derive(Debug)]
pub struct BoxedError(Box<dyn std::error::Error + Send + Sync>);

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
    /// Create a BoxedError from any error type
    pub fn new<E: std::error::Error + Send + Sync + 'static>(err: E) -> Self {
        BoxedError(Box::new(err))
    }

    /// Create a BoxedError from a boxed error
    pub fn from_box(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        BoxedError(err)
    }
}

impl Default for DemoEngine {
    fn default() -> Self {
        Self::with_config(DemoConfig::default())
    }
}

impl DemoEngine {
    /// Create a new demo engine with default configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new demo engine with custom configuration
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
    pub fn list_protocols(&self) -> Vec<String> {
        self.protocols.keys().cloned().collect()
    }

    /// Get metadata for a specific protocol
    pub async fn get_protocol_metadata(&self, name: &str) -> Result<ProtocolMetadata, DemoError> {
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

impl ContextCache {
    pub fn new(config: DemoConfig) -> Self {
        Self {
            entries: HashMap::new(),
            config,
        }
    }

    pub fn get(&self, key: &str) -> Option<&AnalysisResult> {
        self.entries.get(key).map(|entry| &entry.result)
    }

    pub fn insert(&mut self, key: String, result: AnalysisResult) {
        let entry = CacheEntry {
            key: key.clone(),
            result,
            created_at: std::time::Instant::now(),
            access_count: 1,
            last_accessed: std::time::Instant::now(),
        };
        self.entries.insert(key, entry);
        self.evict_if_needed();
    }

    fn evict_if_needed(&mut self) {
        if self.entries.len() > self.config.max_cache_entries {
            // Simple LRU eviction
            let oldest_key = self
                .entries
                .iter()
                .min_by_key(|(_, entry)| entry.last_accessed)
                .map(|(key, _)| key.clone());

            if let Some(key) = oldest_key {
                self.entries.remove(&key);
            }
        }
    }
}

impl TraceStore {
    pub fn new(max_traces: usize) -> Self {
        Self {
            traces: RwLock::new(HashMap::new()),
            max_traces,
        }
    }

    pub async fn add_trace(&self, trace: ApiTrace) {
        let mut traces = self.traces.write().await;
        traces.insert(trace.id, trace);

        // Simple cleanup if we exceed max traces
        if traces.len() > self.max_traces {
            // Remove oldest traces
            let mut trace_list: Vec<_> = traces
                .iter()
                .map(|(id, trace)| (*id, trace.created_at))
                .collect();
            trace_list.sort_by_key(|(_, created_at)| *created_at);

            let to_remove: Vec<_> = trace_list
                .iter()
                .take(traces.len() - self.max_traces)
                .map(|(id, _)| *id)
                .collect();

            for id in to_remove {
                traces.remove(&id);
            }
        }
    }

    pub async fn get_trace(&self, trace_id: Uuid) -> Option<ApiTrace> {
        self.traces.read().await.get(&trace_id).cloned()
    }

    pub async fn get_all_traces(&self) -> Vec<ApiTrace> {
        let traces = self.traces.read().await;
        let mut trace_list: Vec<_> = traces.values().cloned().collect();
        trace_list.sort_by_key(|t| t.created_at);
        trace_list
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_demo_engine_creation() {
        let engine = DemoEngine::new();
        assert!(engine.list_protocols().is_empty());
    }

    #[tokio::test]
    async fn test_context_cache() {
        let config = DemoConfig::default();
        let mut cache = ContextCache::new(config);

        let result = AnalysisResult {
            request_id: Uuid::new_v4(),
            status: AnalysisStatus::Completed,
            started_at: chrono::Utc::now(),
            completed_at: Some(chrono::Utc::now()),
            duration_ms: Some(1000),
            cache_key: "test".to_string(),
            path: "/test".to_string(),
            context_data: Some(serde_json::json!({"test": "data"})),
            error: None,
        };

        cache.insert("test_key".to_string(), result.clone());
        assert!(cache.get("test_key").is_some());
        assert_eq!(cache.get("test_key").unwrap().cache_key, "test");
    }

    #[tokio::test]
    async fn test_trace_store() {
        let store = TraceStore::new(10);
        let trace = ApiTrace {
            id: Uuid::new_v4(),
            protocol: "test".to_string(),
            request_raw: vec![],
            request_parsed: serde_json::json!({}),
            internal_command: vec!["test".to_string()],
            timing: TimingInfo {
                request_decode_ns: 1000,
                cache_lookup_ns: 500,
                analysis_ms: 2000,
                response_encode_ns: 300,
                total_ms: 3000,
            },
            response: serde_json::json!({}),
            cache_hit: false,
            created_at: chrono::Utc::now(),
        };

        let trace_id = trace.id;
        store.add_trace(trace).await;

        let retrieved = store.get_trace(trace_id).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().protocol, "test");
    }
}
