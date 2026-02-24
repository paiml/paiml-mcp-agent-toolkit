/// Telemetry service input for recording events and metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryInput {
    /// Event type (e.g., "`complexity_analysis`", "`refactor_operation`")
    pub event_type: String,
    /// Service name generating the event
    pub service_name: String,
    /// Operation being tracked
    pub operation: String,
    /// Performance metrics for this operation
    pub metrics: OperationMetrics,
    /// Additional context tags
    pub tags: HashMap<String, String>,
    /// Custom properties
    pub properties: HashMap<String, serde_json::Value>,
}

/// Telemetry service output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryOutput {
    /// Unique event ID for correlation
    pub event_id: String,
    /// Timestamp when event was recorded
    pub recorded_at: u64,
    /// Success indicator
    pub success: bool,
}

/// Operation-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationMetrics {
    /// Duration of the operation in milliseconds
    pub duration_ms: u64,
    /// Number of items processed (files, functions, etc.)
    pub items_processed: u64,
    /// Memory usage in bytes
    pub memory_bytes: Option<u64>,
    /// CPU time in milliseconds
    pub cpu_time_ms: Option<u64>,
    /// Success indicator
    pub success: bool,
    /// Error message if failed
    pub error_message: Option<String>,
}

/// Aggregated telemetry data for a service
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServiceTelemetryData {
    /// Service name
    pub service_name: String,
    /// Total number of operations
    pub total_operations: u64,
    /// Successful operations
    pub successful_operations: u64,
    /// Failed operations
    pub failed_operations: u64,
    /// Total processing time in milliseconds
    pub total_duration_ms: u64,
    /// Average duration per operation
    pub avg_duration_ms: u64,
    /// Peak memory usage in bytes
    pub peak_memory_bytes: u64,
    /// Total items processed across all operations
    pub total_items_processed: u64,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// Last operation timestamp
    pub last_operation_at: u64,
    /// Operation type frequencies
    pub operation_counts: HashMap<String, u64>,
}

/// System-wide telemetry aggregation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemTelemetryData {
    /// Overall system metrics
    pub system_metrics: ServiceTelemetryData,
    /// Per-service telemetry data
    pub services: HashMap<String, ServiceTelemetryData>,
    /// System startup time
    pub startup_time: u64,
    /// Total system uptime in seconds
    pub uptime_seconds: u64,
}

/// THE ONE telemetry service implementation (Toyota Way)
#[derive(Debug)]
pub struct TelemetryService {
    /// Service-specific telemetry data
    pub(super) services: Arc<DashMap<String, ServiceTelemetryData>>,
    /// System startup time
    pub(super) startup_time: Instant,
    /// Global event counter
    pub(super) event_counter: AtomicU64,
    /// System metrics
    pub(super) system_metrics: Arc<RwLock<ServiceMetrics>>,
}

impl Default for TelemetryService {
    fn default() -> Self {
        Self::new()
    }
}
