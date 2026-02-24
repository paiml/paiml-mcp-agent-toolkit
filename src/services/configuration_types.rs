// Configuration type definitions for PMAT system
// Included by configuration_service.rs — shares parent module scope

/// Central configuration for the entire PMAT system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PmatConfig {
    /// System-wide settings
    pub system: SystemConfig,

    /// Quality gate configurations
    pub quality: QualityConfig,

    /// Analysis configurations
    pub analysis: AnalysisConfig,

    /// Performance testing configurations
    pub performance: PerformanceConfig,

    /// MCP server configurations
    pub mcp: McpConfig,

    /// Roadmap and project management
    pub roadmap: RoadmapConfig,

    /// Telemetry settings
    pub telemetry: TelemetryConfig,

    /// Semantic search configuration (PMAT-SEARCH-011, PMAT-SEARCH-012)
    /// Added in Sprint 29 - defaults to disabled for backward compatibility
    #[serde(default)]
    pub semantic: SemanticConfig,

    /// Custom user configurations
    pub custom: HashMap<String, serde_json::Value>,
}

/// System-wide configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    /// Project name
    pub project_name: String,

    /// Project root path
    pub project_path: PathBuf,

    /// Output directory for generated files
    pub output_dir: PathBuf,

    /// Maximum number of concurrent operations
    pub max_concurrent_operations: usize,

    /// Enable verbose logging
    pub verbose: bool,

    /// Enable debug mode
    pub debug: bool,

    /// Toolchain preference
    pub default_toolchain: String,
}

/// Quality gate configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityConfig {
    /// Maximum cyclomatic complexity allowed
    pub max_complexity: u32,

    /// Maximum cognitive complexity allowed  
    pub max_cognitive_complexity: u32,

    /// Minimum test coverage percentage
    pub min_coverage: f64,

    /// Allow SATD (Self-Admitted Technical Debt) comments
    pub allow_satd: bool,

    /// Require documentation for public items
    pub require_docs: bool,

    /// Enable lint compliance checking
    pub lint_compliance: bool,

    /// Fail builds on quality violations
    pub fail_on_violation: bool,
}

/// Analysis configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisConfig {
    /// Include patterns for file analysis
    pub include_patterns: Vec<String>,

    /// Exclude patterns for file analysis
    pub exclude_patterns: Vec<String>,

    /// Maximum file size to analyze (bytes)
    pub max_file_size: usize,

    /// Maximum line length for analysis
    pub max_line_length: usize,

    /// Skip vendor directories
    pub skip_vendor: bool,

    /// Enable parallel processing
    pub parallel: bool,

    /// Number of worker threads (0 = auto)
    pub thread_count: usize,

    /// Analysis timeout in seconds
    pub timeout_seconds: u64,
}

/// Performance testing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Enable regression testing
    pub enable_regression_tests: bool,

    /// Enable memory usage tests
    pub enable_memory_tests: bool,

    /// Enable throughput tests
    pub enable_throughput_tests: bool,

    /// Number of test iterations
    pub test_iterations: usize,

    /// Test timeout in milliseconds
    pub timeout_ms: u64,

    /// Target metrics
    pub target_startup_latency_ms: u64,
    pub target_throughput_loc_per_sec: u64,
    pub target_memory_mb: u64,
}

/// MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    /// Server name
    pub server_name: String,

    /// Server version
    pub server_version: String,

    /// Enable transport compression
    pub enable_compression: bool,

    /// Request timeout in seconds
    pub request_timeout_seconds: u64,

    /// Maximum request size in bytes
    pub max_request_size: usize,

    /// Enable request logging
    pub log_requests: bool,

    /// Tools to expose
    pub enabled_tools: Vec<String>,
}

/// Roadmap and project management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadmapConfig {
    /// Path to roadmap file
    pub roadmap_path: PathBuf,

    /// Enable automatic todo generation
    pub auto_generate_todos: bool,

    /// Enforce quality gates
    pub enforce_quality_gates: bool,

    /// Require task IDs
    pub require_task_ids: bool,

    /// Task ID pattern regex
    pub task_id_pattern: String,

    /// Enable velocity tracking
    pub velocity_tracking: bool,

    /// Enable burndown charts
    pub burndown_charts: bool,

    /// Git integration settings
    pub git: GitConfig,
}

/// Git integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    /// Create branches for tasks
    pub create_branches: bool,

    /// Branch naming pattern
    pub branch_pattern: String,

    /// Commit message pattern
    pub commit_pattern: String,

    /// Require quality check before commit
    pub require_quality_check: bool,
}

/// Telemetry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    /// Enable telemetry collection
    pub enabled: bool,

    /// Collection interval in seconds
    pub collection_interval_seconds: u64,

    /// Maximum telemetry data age in days
    pub max_data_age_days: u32,

    /// Enable metric aggregation
    pub enable_aggregation: bool,

    /// Enable telemetry export
    pub enable_export: bool,

    /// Export format (json, csv, etc.)
    pub export_format: String,
}

/// Semantic search configuration (PMAT-SEARCH-011, PMAT-SEARCH-012)
///
/// Configuration for semantic search, code embeddings, and AI-powered code analysis.
/// Uses pure Rust local embeddings via aprender/trueno-rag (NO external API keys required).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SemanticConfig {
    #[serde(default)]
    /// Enable semantic search features
    pub enabled: bool,

    /// Path to vector database file (default: ~/.pmat/embeddings.db)
    pub vector_db_path: Option<String>,

    /// Workspace path for code indexing (default: current directory)
    pub workspace_path: Option<PathBuf>,

    /// Local embedding model to use (aprender-tfidf-local)
    #[serde(default = "default_embedding_model")]
    pub embedding_model: String,

    /// Embedding dimensions (vocabulary-based, typically 256-512)
    #[serde(default = "default_embedding_dimensions")]
    pub embedding_dimensions: usize,

    /// Default search mode: keyword, vector, or hybrid
    #[serde(default = "default_search_mode")]
    pub default_search_mode: String,

    /// Default number of search results
    #[serde(default = "default_limit")]
    pub default_limit: usize,

    /// Auto-sync embeddings on file changes
    pub auto_sync: bool,

    /// Sync interval in seconds (if auto_sync is enabled)
    #[serde(default = "default_sync_interval")]
    pub sync_interval_seconds: u64,

    /// Maximum chunk size for code embeddings (in tokens)
    #[serde(default = "default_max_chunk_tokens")]
    pub max_chunk_tokens: usize,

    /// Supported languages for semantic analysis
    #[serde(default = "default_supported_languages")]
    pub supported_languages: Vec<String>,

    /// Enable MCP semantic tools
    pub enable_mcp_tools: bool,

    /// Cache embedding results (for performance)
    pub enable_cache: bool,

    /// Cache expiration in days
    #[serde(default = "default_cache_expiration")]
    pub cache_expiration_days: u32,
}

fn default_embedding_model() -> String {
    "aprender-tfidf-local".to_string()
}

fn default_embedding_dimensions() -> usize {
    256
}

fn default_search_mode() -> String {
    "hybrid".to_string()
}

fn default_limit() -> usize {
    10
}

fn default_sync_interval() -> u64 {
    300
}

fn default_max_chunk_tokens() -> usize {
    500
}

fn default_supported_languages() -> Vec<String> {
    vec![
        "rust".to_string(),
        "python".to_string(),
        "typescript".to_string(),
        "javascript".to_string(),
        "go".to_string(),
        "c".to_string(),
        "cpp".to_string(),
    ]
}

fn default_cache_expiration() -> u32 {
    30
}
