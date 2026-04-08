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
    context_cache: Arc<RwLock<ContextCache>>,
    /// Registered protocol adapters
    protocols: HashMap<
        String,
        Box<dyn DemoProtocol<Request = Value, Response = Value, Error = BoxedError>>,
    >,
    /// Trace storage for API introspection
    trace_store: Arc<TraceStore>,
    /// Configuration settings
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

// --- Include split implementation files ---
include!("protocol_harness_wrapper.rs");
include!("protocol_harness_execution.rs");
include!("protocol_harness_cache.rs");
include!("protocol_harness_tests.rs");
