#![cfg_attr(coverage_nightly, coverage(off))]
//! Type definitions for the daemon module.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use super::super::mcp_server::AgentConfig;
use super::super::quality_monitor::QualityMonitorConfig;

/// Configuration for the background daemon
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DaemonConfig {
    /// Agent configuration
    pub agent: AgentConfig,

    /// Quality monitoring configuration
    pub quality_monitor: QualityMonitorConfig,

    /// Daemon-specific settings
    pub daemon: DaemonSettings,
}

/// Daemon-specific settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonSettings {
    /// PID file location (optional)
    pub pid_file: Option<PathBuf>,

    /// Log file location (optional)
    pub log_file: Option<PathBuf>,

    /// Working directory
    pub working_directory: PathBuf,

    /// Health check interval
    pub health_check_interval: Duration,

    /// Maximum memory usage before restart (MB)
    pub max_memory_mb: u64,

    /// Auto-restart on failure
    pub auto_restart: bool,

    /// Graceful shutdown timeout
    pub shutdown_timeout: Duration,
}

impl Default for DaemonSettings {
    fn default() -> Self {
        Self {
            pid_file: None,
            log_file: None,
            working_directory: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            health_check_interval: Duration::from_secs(30),
            max_memory_mb: 500,
            auto_restart: true,
            shutdown_timeout: Duration::from_secs(10),
        }
    }
}

/// Current state of the daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonState {
    /// Daemon status
    pub status: DaemonStatus,

    /// Start time
    pub started_at: SystemTime,

    /// Last health check
    pub last_health_check: SystemTime,

    /// Number of active projects being monitored
    pub active_projects: usize,

    /// Total quality events processed
    pub events_processed: u64,

    /// Current memory usage (MB)
    pub memory_usage_mb: u64,

    /// Number of restarts
    pub restart_count: u32,

    /// Last error message
    pub last_error: Option<String>,
}

/// Daemon status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DaemonStatus {
    Starting,
    Running,
    Stopping,
    Stopped,
    Error,
}

/// Commands that can be sent to the daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DaemonCommand {
    /// Get current status
    GetStatus,

    /// Start monitoring a project
    StartMonitoring { project_path: String },

    /// Stop monitoring a project
    StopMonitoring { project_id: String },

    /// Reload configuration
    ReloadConfig,

    /// Perform health check
    HealthCheck,

    /// Graceful shutdown
    Shutdown,
}

/// Result of quality gate execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGateResult {
    pub violations: Option<u32>,
    pub passed: bool,
}
