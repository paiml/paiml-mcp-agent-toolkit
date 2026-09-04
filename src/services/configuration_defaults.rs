// Default configuration values for PMAT system
// Included by configuration_service.rs — shares parent module scope
//
// Each section owns its defaults through `impl Default`, and `PmatConfig`
// composes them. That is what lets every section carry `#[serde(default)]`:
// a `pmat.toml` that sets a single key deserialises to "that key, plus these
// defaults for everything else" instead of failing strict deserialisation and
// being replaced wholesale (CRUX-03, #1147). `default_config()` is kept as the
// name every caller already uses; it is the same value.

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            project_name: "pmat".to_string(),
            project_path: std::env::current_dir().unwrap_or_default(),
            output_dir: PathBuf::from("target/pmat"),
            max_concurrent_operations: num_cpus::get(),
            verbose: false,
            debug: false,
            default_toolchain: "rust".to_string(),
        }
    }
}

/// The file-length cap, in ONE place.
///
/// `pmat work`'s contract profiles already default to 500 lines
/// (`work_contract_profile.rs`); the file-size gate enforces the same number
/// rather than inventing a second one.
pub const DEFAULT_MAX_FILE_LINES: usize = 500;

/// The 90-day churn cap (AD-05, spec §4.5).
pub const DEFAULT_MAX_CHURN_COMMITS_90D: usize = 20;

impl Default for QualityConfig {
    fn default() -> Self {
        Self {
            max_complexity: 30,
            max_cognitive_complexity: 25,
            min_coverage: 80.0,
            allow_satd: false,
            require_docs: true,
            lint_compliance: true,
            fail_on_violation: true,
            max_file_lines: DEFAULT_MAX_FILE_LINES,
            max_churn_commits_90d: DEFAULT_MAX_CHURN_COMMITS_90D,
        }
    }
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            include_patterns: vec!["**/*.rs".to_string(), "**/*.ts".to_string()],
            exclude_patterns: vec![
                "**/target/**".to_string(),
                "**/node_modules/**".to_string(),
            ],
            max_file_size: 1024 * 1024, // 1MB
            max_line_length: 100,
            skip_vendor: true,
            parallel: true,
            thread_count: 0,      // Auto
            timeout_seconds: 300, // 5 minutes
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enable_regression_tests: true,
            enable_memory_tests: true,
            enable_throughput_tests: true,
            test_iterations: 10,
            timeout_ms: 30000,
            target_startup_latency_ms: 127,
            target_throughput_loc_per_sec: 487000,
            target_memory_mb: 47,
        }
    }
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            server_name: "pmat-mcp-server".to_string(),
            server_version: env!("CARGO_PKG_VERSION").to_string(),
            enable_compression: true,
            request_timeout_seconds: 30,
            max_request_size: 10 * 1024 * 1024, // 10MB
            log_requests: false,
            enabled_tools: vec![
                "analyze_complexity".to_string(),
                "analyze_dead_code".to_string(),
                "quality_gate".to_string(),
                "refactor_start".to_string(),
            ],
        }
    }
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            create_branches: false, // DISABLED: per CLAUDE.md zero-branching policy
            branch_pattern: "feature/{task_id}".to_string(),
            commit_pattern: "{task_id}: {message}".to_string(),
            require_quality_check: true,
        }
    }
}

impl Default for RoadmapConfig {
    fn default() -> Self {
        Self {
            roadmap_path: PathBuf::from("docs/execution/roadmap.md"),
            auto_generate_todos: true,
            enforce_quality_gates: true,
            require_task_ids: true,
            task_id_pattern: "PMAT-[0-9]{4}".to_string(),
            velocity_tracking: true,
            burndown_charts: true,
            git: GitConfig::default(),
        }
    }
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            collection_interval_seconds: 60,
            max_data_age_days: 30,
            enable_aggregation: true,
            enable_export: false,
            export_format: "json".to_string(),
        }
    }
}

impl Default for PmatConfig {
    fn default() -> Self {
        Self {
            system: SystemConfig::default(),
            quality: QualityConfig::default(),
            analysis: AnalysisConfig::default(),
            performance: PerformanceConfig::default(),
            mcp: McpConfig::default(),
            roadmap: RoadmapConfig::default(),
            telemetry: TelemetryConfig::default(),
            // Deliberately NOT `SemanticConfig::default()`: that derive gives
            // `enable_mcp_tools: false` and `enable_cache: false`, while the
            // shipped default has both on. Keep the literal so the value
            // `default_config()` has always returned does not move.
            semantic: SemanticConfig {
                enabled: false,                                      // Disabled by default
                vector_db_path: None, // Defaults to ~/.pmat/embeddings.db
                workspace_path: None, // Defaults to current directory
                embedding_model: "aprender-tfidf-local".to_string(), // Local embeddings (no API key)
                embedding_dimensions: 256,
                default_search_mode: "hybrid".to_string(),
                default_limit: 10,
                auto_sync: false,
                sync_interval_seconds: 300, // 5 minutes
                max_chunk_tokens: 500,
                supported_languages: vec![
                    "rust".to_string(),
                    "typescript".to_string(),
                    "python".to_string(),
                    "c".to_string(),
                    "cpp".to_string(),
                    "go".to_string(),
                    "javascript".to_string(),
                ],
                enable_mcp_tools: true,
                enable_cache: true,
                cache_expiration_days: 7,
            },
            custom: HashMap::new(),
        }
    }
}

impl ConfigurationService {
    /// Create default configuration
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn default_config() -> PmatConfig {
        PmatConfig::default()
    }
}
