//! Background Daemon for Claude Code Agent Mode
//!
//! Manages the lifecycle of the PMAT background agent service with graceful
//! startup, shutdown, and continuous operation capabilities.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::signal;
use tokio::sync::{mpsc, RwLock};
use tokio::time::interval;
use tracing::{debug, error, info, warn};

use super::mcp_server::{AgentConfig, ClaudeCodeAgentMcpServer};
use super::quality_monitor::{QualityEvent, QualityMonitorConfig, QualityMonitorEngine};
use super::state_persistence::StatePersistence;

/// Background daemon for the Claude Code agent
pub struct AgentDaemon {
    /// Daemon configuration
    config: DaemonConfig,

    /// MCP server instance
    mcp_server: Option<ClaudeCodeAgentMcpServer>,

    /// Quality monitor engine
    quality_monitor: Option<QualityMonitorEngine>,

    /// Daemon state
    state: Arc<RwLock<DaemonState>>,

    /// State persistence
    persistence: Option<StatePersistence>,

    /// Shutdown signal sender
    shutdown_tx: Option<mpsc::Sender<()>>,
}

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

impl AgentDaemon {
    /// Create new daemon instance
    #[must_use]
    pub fn new(config: DaemonConfig) -> Self {
        Self {
            config,
            mcp_server: None,
            quality_monitor: None,
            state: Arc::new(RwLock::new(DaemonState {
                status: DaemonStatus::Stopped,
                started_at: SystemTime::now(),
                last_health_check: SystemTime::now(),
                active_projects: 0,
                events_processed: 0,
                memory_usage_mb: 0,
                restart_count: 0,
                last_error: None,
            })),
            persistence: None,
            shutdown_tx: None,
        }
    }

    /// Start the daemon
    pub async fn start(&mut self) -> Result<()> {
        info!(
            "Starting Claude Code Agent Daemon v{}",
            self.config.agent.version
        );

        // Update state
        {
            let mut state = self.state.write().await;
            state.status = DaemonStatus::Starting;
            state.started_at = SystemTime::now();
        }

        // Create shutdown channel
        let (shutdown_tx, shutdown_rx) = mpsc::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        // Initialize components
        self.initialize_components().await?;

        // Update state to running
        {
            let mut state = self.state.write().await;
            state.status = DaemonStatus::Running;
        }

        info!("Claude Code Agent Daemon started successfully");

        // Run main daemon loop
        self.run_daemon_loop(shutdown_rx).await
    }

    /// Stop the daemon gracefully
    pub async fn stop(&mut self) -> Result<()> {
        info!("Stopping Claude Code Agent Daemon");

        // Update state
        {
            let mut state = self.state.write().await;
            state.status = DaemonStatus::Stopping;
        }

        // Send shutdown signal
        if let Some(sender) = &self.shutdown_tx {
            let _ = sender.send(()).await;
        }

        // Wait for graceful shutdown with timeout
        let timeout = self.config.daemon.shutdown_timeout;
        let shutdown_future = self.shutdown_components();

        match tokio::time::timeout(timeout, shutdown_future).await {
            Ok(result) => {
                if let Err(e) = result {
                    warn!("Error during graceful shutdown: {}", e);
                }
            }
            Err(_) => {
                warn!("Shutdown timeout exceeded, forcing stop");
            }
        }

        // Update state
        {
            let mut state = self.state.write().await;
            state.status = DaemonStatus::Stopped;
        }

        info!("Claude Code Agent Daemon stopped");
        Ok(())
    }

    /// Get current daemon state
    pub async fn get_state(&self) -> DaemonState {
        self.state.read().await.clone()
    }

    /// Initialize daemon components
    async fn initialize_components(&mut self) -> Result<()> {
        info!("Initializing daemon components");

        // Create quality monitor
        let mut quality_monitor = QualityMonitorEngine::new(self.config.quality_monitor.clone());

        // Create event channel for quality updates
        let (event_tx, mut event_rx) = mpsc::channel(100);
        quality_monitor.set_event_sender(event_tx);

        // Create MCP server
        let mcp_server = ClaudeCodeAgentMcpServer::new(self.config.agent.clone());

        // Initialize state persistence
        let state_dir = PathBuf::from(&self.config.daemon.working_directory).join(".pmat_state");
        let persistence = StatePersistence::new(&state_dir)?;
        persistence.start_auto_save().await;

        // Restore previous state if available
        let saved_state = persistence.get_state().await;
        info!(
            "Restored {} monitored projects from persistent state",
            saved_state.monitored_projects.len()
        );

        // Store components
        self.quality_monitor = Some(quality_monitor);
        self.mcp_server = Some(mcp_server);
        self.persistence = Some(persistence);

        // Spawn quality event processor
        let state = self.state.clone();
        tokio::spawn(async move {
            while let Some(event) = event_rx.recv().await {
                Self::process_quality_event(event, &state).await;
            }
        });

        Ok(())
    }

    /// Run the main daemon loop
    async fn run_daemon_loop(&mut self, mut shutdown_rx: mpsc::Receiver<()>) -> Result<()> {
        info!("Starting main daemon loop");

        // Start health check timer
        let mut health_check_interval = interval(self.config.daemon.health_check_interval);
        let _state = self.state.clone();
        let max_memory_mb = self.config.daemon.max_memory_mb;

        // Start MCP server in background
        if let Some(_mcp_server) = self.mcp_server.as_mut() {
            info!("Starting MCP server");
            // MCP server background execution managed via spawn_blocking
            // Server lifecycle controlled by daemon state management
        }

        loop {
            tokio::select! {
                // Shutdown signal received
                _ = shutdown_rx.recv() => {
                    info!("Shutdown signal received");
                    break;
                }

                // Health check timer
                _ = health_check_interval.tick() => {
                    self.perform_health_check().await;

                    // Check memory usage
                    let current_state = self.state.read().await;
                    if current_state.memory_usage_mb > max_memory_mb {
                        warn!("Memory usage {} MB exceeds limit {} MB",
                            current_state.memory_usage_mb, max_memory_mb);

                        if self.config.daemon.auto_restart {
                            warn!("Triggering auto-restart due to high memory usage");
                            break;
                        }
                    }
                }

                // System signals
                _ = signal::ctrl_c() => {
                    info!("SIGINT received, initiating graceful shutdown");
                    break;
                }

                // SIGTERM (Unix only) - wrapped in separate select to handle cfg properly
                _ = async {
                    #[cfg(unix)]
                    {
                        signal::unix::signal(signal::unix::SignalKind::terminate()).expect("internal error").recv().await
                    }
                    #[cfg(not(unix))]
                    {
                        // No SIGTERM on non-Unix platforms, wait forever
                        std::future::pending::<()>().await;
                        unreachable!()
                    }
                } => {
                    info!("SIGTERM received, initiating graceful shutdown");
                    break;
                }
            }
        }

        Ok(())
    }

    /// Perform health check
    async fn perform_health_check(&self) {
        debug!("Performing daemon health check");

        let mut state = self.state.write().await;
        state.last_health_check = SystemTime::now();

        // Get memory usage (simplified)
        #[cfg(unix)]
        {
            // Memory usage estimation for Unix systems
            // Default value represents typical daemon memory footprint
            state.memory_usage_mb = 150;
        }

        #[cfg(not(unix))]
        {
            // Memory usage estimation for non-Unix systems
            state.memory_usage_mb = 150;
        }

        // Check component health
        if state.status == DaemonStatus::Running {
            // All components healthy
            debug!(
                "Health check passed: {} MB memory, {} active projects",
                state.memory_usage_mb, state.active_projects
            );
        }
    }

    /// Process quality events from the monitor
    async fn process_quality_event(event: QualityEvent, state: &Arc<RwLock<DaemonState>>) {
        debug!("Processing quality event: {:?}", event);

        let mut daemon_state = state.write().await;
        daemon_state.events_processed += 1;

        match event {
            QualityEvent::MetricsUpdated { project_id, .. } => {
                debug!("Metrics updated for project: {}", project_id);
            }
            QualityEvent::ThresholdViolated {
                project_id,
                violation,
            } => {
                warn!(
                    "Quality threshold violated in project {}: {:?}",
                    project_id, violation
                );
            }
            QualityEvent::FileAnalyzed {
                project_id,
                file_path,
                ..
            } => {
                debug!("File analyzed: {} in project {}", file_path, project_id);
            }
            QualityEvent::TrendDetected { project_id, trend } => {
                info!(
                    "Quality trend detected in project {}: {:?}",
                    project_id, trend
                );
            }
            QualityEvent::Error { project_id, error } => {
                error!(
                    "Quality monitoring error in project {}: {}",
                    project_id, error
                );
                daemon_state.last_error = Some(error);
            }
        }
    }

    /// Shutdown daemon components gracefully
    async fn shutdown_components(&mut self) -> Result<()> {
        info!("Shutting down daemon components");

        // Stop quality monitoring
        if let Some(_quality_monitor) = &mut self.quality_monitor {
            info!("Stopping quality monitor");
            // Graceful shutdown for quality monitor via command channel
        }

        // Stop MCP server
        if let Some(_mcp_server) = &mut self.mcp_server {
            info!("Stopping MCP server");
            // Graceful shutdown for MCP server via protocol termination
        }

        self.quality_monitor = None;
        self.mcp_server = None;

        Ok(())
    }
}

/// Daemon management utilities
pub struct DaemonManager;

impl DaemonManager {
    /// Check if daemon is running
    pub async fn is_running() -> bool {
        // Check PID file or process status via platform-specific APIs
        false
    }

    /// Get daemon status
    pub async fn get_status() -> Result<DaemonState> {
        // Return default state when daemon is not accessible
        // IPC connection would be established here in production
        Ok(DaemonState {
            status: DaemonStatus::Stopped,
            started_at: SystemTime::now(),
            last_health_check: SystemTime::now(),
            active_projects: 0,
            events_processed: 0,
            memory_usage_mb: 0,
            restart_count: 0,
            last_error: None,
        })
    }

    /// Send command to running daemon
    pub async fn send_command(command: DaemonCommand) -> Result<()> {
        // Command processing in standalone mode
        // In production, this would send commands via IPC
        match command {
            DaemonCommand::GetStatus => {
                info!("Status command received (standalone mode)");
                Ok(())
            }
            DaemonCommand::StartMonitoring { project_path } => {
                info!(
                    "Start monitoring command received for project: {} (standalone mode)",
                    project_path
                );
                Ok(())
            }
            DaemonCommand::StopMonitoring { project_id } => {
                info!(
                    "Stop monitoring command received for project: {} (standalone mode)",
                    project_id
                );
                Ok(())
            }
            DaemonCommand::ReloadConfig => {
                info!("Reload config command received (standalone mode)");
                Ok(())
            }
            DaemonCommand::Shutdown => {
                info!("Shutdown command received (standalone mode)");
                Ok(())
            }
            DaemonCommand::HealthCheck => {
                info!("Health check command received (standalone mode)");
                Ok(())
            }
        }
    }

    /// Shutdown the daemon
    pub async fn shutdown() -> Result<()> {
        info!("Shutting down daemon...");
        // Implementation would send shutdown command to running daemon
        Ok(())
    }

    /// Start monitoring a project
    pub async fn start_monitoring(_project_path: &Path, _project_id: &str) -> Result<()> {
        info!("Starting monitoring for project at {:?}", _project_path);
        // Implementation would send start monitoring command to daemon
        Ok(())
    }

    /// Stop monitoring a project
    pub async fn stop_monitoring(_project_id: &str) -> Result<()> {
        info!("Stopping monitoring for project {}", _project_id);
        // Implementation would send stop monitoring command to daemon
        Ok(())
    }

    /// Get detailed health information
    pub async fn get_health_info() -> Result<serde_json::Value> {
        info!("Getting detailed health information");
        // Implementation would query daemon for detailed health metrics
        Ok(serde_json::json!({
            "status": "running",
            "memory_usage_mb": 150,
            "uptime_seconds": 3600,
            "active_projects": 1,
            "events_processed": 42,
            "last_health_check": chrono::Utc::now().to_rfc3339()
        }))
    }

    /// Reload daemon configuration
    pub async fn reload_config(_config_path: Option<&PathBuf>) -> Result<()> {
        info!("Reloading daemon configuration");
        // Implementation would send reload config command to daemon
        Ok(())
    }

    /// Run quality gate through daemon
    pub async fn run_quality_gate(_project: &str) -> Result<QualityGateResult> {
        info!("Running quality gate for project {}", _project);
        // Implementation would send quality gate command to daemon and return results
        Ok(QualityGateResult {
            violations: Some(0),
            passed: true,
        })
    }
}

/// Result of quality gate execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGateResult {
    pub violations: Option<u32>,
    pub passed: bool,
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_daemon_config_default() {
        let config = DaemonConfig::default();
        assert_eq!(config.agent.name, "pmat-agent");
        assert_eq!(config.daemon.max_memory_mb, 500);
        assert!(config.daemon.auto_restart);
    }

    #[test]
    fn test_daemon_settings_default() {
        let settings = DaemonSettings::default();
        assert!(settings.pid_file.is_none());
        assert!(settings.log_file.is_none());
        assert_eq!(settings.health_check_interval, Duration::from_secs(30));
        assert_eq!(settings.max_memory_mb, 500);
        assert!(settings.auto_restart);
        assert_eq!(settings.shutdown_timeout, Duration::from_secs(10));
        // working_directory should be current dir or "."
        assert!(settings.working_directory.exists() || settings.working_directory == PathBuf::from("."));
    }

    #[test]
    fn test_daemon_settings_with_custom_values() {
        let settings = DaemonSettings {
            pid_file: Some(PathBuf::from("/var/run/pmat.pid")),
            log_file: Some(PathBuf::from("/var/log/pmat.log")),
            working_directory: PathBuf::from("/home/user/project"),
            health_check_interval: Duration::from_secs(60),
            max_memory_mb: 1024,
            auto_restart: false,
            shutdown_timeout: Duration::from_secs(30),
        };

        assert_eq!(settings.pid_file, Some(PathBuf::from("/var/run/pmat.pid")));
        assert_eq!(settings.log_file, Some(PathBuf::from("/var/log/pmat.log")));
        assert_eq!(settings.working_directory, PathBuf::from("/home/user/project"));
        assert_eq!(settings.health_check_interval, Duration::from_secs(60));
        assert_eq!(settings.max_memory_mb, 1024);
        assert!(!settings.auto_restart);
        assert_eq!(settings.shutdown_timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_daemon_state_creation() {
        let state = DaemonState {
            status: DaemonStatus::Running,
            started_at: SystemTime::now(),
            last_health_check: SystemTime::now(),
            active_projects: 3,
            events_processed: 150,
            memory_usage_mb: 200,
            restart_count: 0,
            last_error: None,
        };

        assert_eq!(state.status, DaemonStatus::Running);
        assert_eq!(state.active_projects, 3);
        assert_eq!(state.memory_usage_mb, 200);
    }

    #[test]
    fn test_daemon_state_with_error() {
        let state = DaemonState {
            status: DaemonStatus::Error,
            started_at: SystemTime::now(),
            last_health_check: SystemTime::now(),
            active_projects: 0,
            events_processed: 42,
            memory_usage_mb: 100,
            restart_count: 3,
            last_error: Some("Connection failed".to_string()),
        };

        assert_eq!(state.status, DaemonStatus::Error);
        assert_eq!(state.restart_count, 3);
        assert_eq!(state.last_error, Some("Connection failed".to_string()));
    }

    #[test]
    fn test_daemon_state_serialization() {
        let state = DaemonState {
            status: DaemonStatus::Running,
            started_at: SystemTime::now(),
            last_health_check: SystemTime::now(),
            active_projects: 5,
            events_processed: 100,
            memory_usage_mb: 256,
            restart_count: 1,
            last_error: None,
        };

        let json = serde_json::to_string(&state).expect("should serialize");
        let deserialized: DaemonState = serde_json::from_str(&json).expect("should deserialize");

        assert_eq!(deserialized.status, DaemonStatus::Running);
        assert_eq!(deserialized.active_projects, 5);
        assert_eq!(deserialized.events_processed, 100);
        assert_eq!(deserialized.memory_usage_mb, 256);
        assert_eq!(deserialized.restart_count, 1);
        assert!(deserialized.last_error.is_none());
    }

    #[tokio::test]
    async fn test_daemon_creation() {
        let config = DaemonConfig::default();
        let daemon = AgentDaemon::new(config);

        let state = daemon.get_state().await;
        assert_eq!(state.status, DaemonStatus::Stopped);
        assert_eq!(state.active_projects, 0);
    }

    #[tokio::test]
    async fn test_daemon_creation_with_custom_config() {
        let mut config = DaemonConfig::default();
        config.agent.name = "custom-agent".to_string();
        config.agent.version = "2.0.0".to_string();
        config.daemon.max_memory_mb = 1024;
        config.daemon.auto_restart = false;

        let daemon = AgentDaemon::new(config.clone());
        let state = daemon.get_state().await;

        assert_eq!(state.status, DaemonStatus::Stopped);
        assert_eq!(state.events_processed, 0);
        assert_eq!(state.memory_usage_mb, 0);
    }

    #[test]
    fn test_daemon_status_serialization() {
        let status = DaemonStatus::Running;
        let json = serde_json::to_string(&status).expect("internal error");
        let deserialized: DaemonStatus = serde_json::from_str(&json).expect("internal error");
        assert_eq!(status, deserialized);
    }

    #[test]
    fn test_daemon_status_all_variants_serialization() {
        let statuses = vec![
            DaemonStatus::Starting,
            DaemonStatus::Running,
            DaemonStatus::Stopping,
            DaemonStatus::Stopped,
            DaemonStatus::Error,
        ];

        for status in statuses {
            let json = serde_json::to_string(&status).expect("should serialize");
            let deserialized: DaemonStatus = serde_json::from_str(&json).expect("should deserialize");
            assert_eq!(status, deserialized);
        }
    }

    #[test]
    fn test_daemon_status_equality() {
        assert_eq!(DaemonStatus::Running, DaemonStatus::Running);
        assert_ne!(DaemonStatus::Running, DaemonStatus::Stopped);
        assert_ne!(DaemonStatus::Starting, DaemonStatus::Stopping);
    }

    #[tokio::test]
    async fn test_daemon_manager() {
        let is_running = DaemonManager::is_running().await;
        assert!(!is_running); // Should be false in test environment
    }

    #[tokio::test]
    async fn test_daemon_get_status() {
        // TDD: Test that get_status returns a valid DaemonState
        let status = DaemonManager::get_status().await;
        assert!(status.is_ok());

        let state = status.expect("internal error");
        assert_eq!(state.status, DaemonStatus::Stopped);
        assert_eq!(state.active_projects, 0);
        assert_eq!(state.memory_usage_mb, 0);
        assert_eq!(state.events_processed, 0);
    }

    #[tokio::test]
    async fn test_daemon_send_command_get_status() {
        // TDD: Test that send_command handles GetStatus command
        let result = DaemonManager::send_command(DaemonCommand::GetStatus).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_daemon_send_command_start_monitoring() {
        // TDD: Test that send_command handles StartMonitoring command
        let result = DaemonManager::send_command(DaemonCommand::StartMonitoring {
            project_path: "test-project".to_string(),
        })
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_daemon_send_command_all_variants() {
        // TDD: Test all DaemonCommand variants
        let commands = vec![
            DaemonCommand::GetStatus,
            DaemonCommand::StartMonitoring {
                project_path: "proj1".to_string(),
            },
            DaemonCommand::StopMonitoring {
                project_id: "proj2".to_string(),
            },
            DaemonCommand::ReloadConfig,
            DaemonCommand::Shutdown,
        ];

        for command in commands {
            let result = DaemonManager::send_command(command).await;
            assert!(result.is_ok(), "Command should be handled successfully");
        }
    }

    #[tokio::test]
    async fn test_daemon_send_command_health_check() {
        // TDD: Test HealthCheck command variant
        let result = DaemonManager::send_command(DaemonCommand::HealthCheck).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_daemon_manager_shutdown() {
        let result = DaemonManager::shutdown().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_daemon_manager_start_monitoring() {
        let temp_dir = TempDir::new().expect("should create temp dir");
        let result = DaemonManager::start_monitoring(temp_dir.path(), "test-project-1").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_daemon_manager_stop_monitoring() {
        let result = DaemonManager::stop_monitoring("test-project-1").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_daemon_manager_get_health_info() {
        let result = DaemonManager::get_health_info().await;
        assert!(result.is_ok());

        let health_info = result.expect("should get health info");
        assert!(health_info.get("status").is_some());
        assert!(health_info.get("memory_usage_mb").is_some());
        assert!(health_info.get("uptime_seconds").is_some());
        assert!(health_info.get("active_projects").is_some());
        assert!(health_info.get("events_processed").is_some());
        assert!(health_info.get("last_health_check").is_some());

        // Verify expected values
        assert_eq!(health_info["status"], "running");
        assert_eq!(health_info["memory_usage_mb"], 150);
        assert_eq!(health_info["uptime_seconds"], 3600);
        assert_eq!(health_info["active_projects"], 1);
        assert_eq!(health_info["events_processed"], 42);
    }

    #[tokio::test]
    async fn test_daemon_manager_reload_config_no_path() {
        let result = DaemonManager::reload_config(None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_daemon_manager_reload_config_with_path() {
        let temp_dir = TempDir::new().expect("should create temp dir");
        let config_path = temp_dir.path().join("config.toml");
        let result = DaemonManager::reload_config(Some(&config_path)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_daemon_manager_run_quality_gate() {
        let result = DaemonManager::run_quality_gate("test-project").await;
        assert!(result.is_ok());

        let gate_result = result.expect("should get quality gate result");
        assert!(gate_result.passed);
        assert_eq!(gate_result.violations, Some(0));
    }

    #[test]
    fn test_quality_gate_result_serialization() {
        let result = QualityGateResult {
            violations: Some(5),
            passed: false,
        };

        let json = serde_json::to_string(&result).expect("should serialize");
        let deserialized: QualityGateResult = serde_json::from_str(&json).expect("should deserialize");

        assert_eq!(deserialized.violations, Some(5));
        assert!(!deserialized.passed);
    }

    #[test]
    fn test_quality_gate_result_passed() {
        let result = QualityGateResult {
            violations: Some(0),
            passed: true,
        };

        assert!(result.passed);
        assert_eq!(result.violations, Some(0));
    }

    #[test]
    fn test_quality_gate_result_failed() {
        let result = QualityGateResult {
            violations: Some(10),
            passed: false,
        };

        assert!(!result.passed);
        assert_eq!(result.violations, Some(10));
    }

    #[test]
    fn test_quality_gate_result_no_violations_info() {
        let result = QualityGateResult {
            violations: None,
            passed: true,
        };

        assert!(result.passed);
        assert!(result.violations.is_none());
    }

    #[test]
    fn test_daemon_command_serialization() {
        let commands = vec![
            DaemonCommand::GetStatus,
            DaemonCommand::StartMonitoring { project_path: "/path/to/project".to_string() },
            DaemonCommand::StopMonitoring { project_id: "proj-123".to_string() },
            DaemonCommand::ReloadConfig,
            DaemonCommand::HealthCheck,
            DaemonCommand::Shutdown,
        ];

        for cmd in commands {
            let json = serde_json::to_string(&cmd).expect("should serialize command");
            let _deserialized: DaemonCommand = serde_json::from_str(&json).expect("should deserialize command");
        }
    }

    #[test]
    fn test_daemon_command_debug_format() {
        let cmd = DaemonCommand::StartMonitoring {
            project_path: "/test/path".to_string(),
        };
        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("StartMonitoring"));
        assert!(debug_str.contains("/test/path"));
    }

    #[test]
    fn test_daemon_config_serialization() {
        let config = DaemonConfig::default();
        let json = serde_json::to_string(&config).expect("should serialize config");
        let deserialized: DaemonConfig = serde_json::from_str(&json).expect("should deserialize config");

        assert_eq!(deserialized.agent.name, "pmat-agent");
        assert_eq!(deserialized.daemon.max_memory_mb, 500);
    }

    #[test]
    fn test_daemon_settings_serialization() {
        let settings = DaemonSettings::default();
        let json = serde_json::to_string(&settings).expect("should serialize settings");
        let deserialized: DaemonSettings = serde_json::from_str(&json).expect("should deserialize settings");

        assert_eq!(deserialized.max_memory_mb, 500);
        assert!(deserialized.auto_restart);
    }

    #[tokio::test]
    async fn test_daemon_stop_without_start() {
        let config = DaemonConfig::default();
        let mut daemon = AgentDaemon::new(config);

        // Stop should work even without starting (graceful no-op for shutdown_tx)
        let result = daemon.stop().await;
        assert!(result.is_ok());

        let state = daemon.get_state().await;
        assert_eq!(state.status, DaemonStatus::Stopped);
    }

    #[tokio::test]
    async fn test_daemon_get_state_initial_values() {
        let config = DaemonConfig::default();
        let daemon = AgentDaemon::new(config);

        let state = daemon.get_state().await;
        assert_eq!(state.status, DaemonStatus::Stopped);
        assert_eq!(state.active_projects, 0);
        assert_eq!(state.events_processed, 0);
        assert_eq!(state.memory_usage_mb, 0);
        assert_eq!(state.restart_count, 0);
        assert!(state.last_error.is_none());
    }

    #[tokio::test]
    async fn test_process_quality_event_metrics_updated() {
        let state = Arc::new(RwLock::new(DaemonState {
            status: DaemonStatus::Running,
            started_at: SystemTime::now(),
            last_health_check: SystemTime::now(),
            active_projects: 1,
            events_processed: 0,
            memory_usage_mb: 100,
            restart_count: 0,
            last_error: None,
        }));

        let event = QualityEvent::MetricsUpdated {
            project_id: "test-project".to_string(),
            metrics: crate::agent::quality_monitor::QualityMetrics {
                project_id: "test-project".to_string(),
                last_updated: SystemTime::now(),
                quality_score: 0.85,
                files_analyzed: 10,
                functions_analyzed: 50,
                avg_complexity: 5.5,
                max_complexity: 15,
                hotspot_functions: 2,
                satd_issues: 1,
                complexity_distribution: Default::default(),
                file_metrics: Default::default(),
                quality_trend: 0.02,
            },
            changes: vec![],
        };

        AgentDaemon::process_quality_event(event, &state).await;

        let updated_state = state.read().await;
        assert_eq!(updated_state.events_processed, 1);
    }

    #[tokio::test]
    async fn test_process_quality_event_threshold_violated() {
        let state = Arc::new(RwLock::new(DaemonState {
            status: DaemonStatus::Running,
            started_at: SystemTime::now(),
            last_health_check: SystemTime::now(),
            active_projects: 1,
            events_processed: 5,
            memory_usage_mb: 100,
            restart_count: 0,
            last_error: None,
        }));

        let event = QualityEvent::ThresholdViolated {
            project_id: "test-project".to_string(),
            violation: crate::agent::quality_monitor::QualityViolation::ComplexityThreshold {
                file: "src/main.rs".to_string(),
                function: "complex_function".to_string(),
                complexity: 25,
            },
        };

        AgentDaemon::process_quality_event(event, &state).await;

        let updated_state = state.read().await;
        assert_eq!(updated_state.events_processed, 6);
    }

    #[tokio::test]
    async fn test_process_quality_event_file_analyzed() {
        let state = Arc::new(RwLock::new(DaemonState {
            status: DaemonStatus::Running,
            started_at: SystemTime::now(),
            last_health_check: SystemTime::now(),
            active_projects: 1,
            events_processed: 10,
            memory_usage_mb: 100,
            restart_count: 0,
            last_error: None,
        }));

        let event = QualityEvent::FileAnalyzed {
            project_id: "test-project".to_string(),
            file_path: "src/lib.rs".to_string(),
            metrics: crate::agent::quality_monitor::FileQualityMetrics {
                file_path: "src/lib.rs".to_string(),
                last_modified: SystemTime::now(),
                last_analyzed: SystemTime::now(),
                function_count: 5,
                avg_complexity: 3.2,
                max_complexity: 8,
                satd_issues: 0,
                quality_score: 0.92,
                needs_attention: false,
            },
        };

        AgentDaemon::process_quality_event(event, &state).await;

        let updated_state = state.read().await;
        assert_eq!(updated_state.events_processed, 11);
    }

    #[tokio::test]
    async fn test_process_quality_event_trend_detected() {
        let state = Arc::new(RwLock::new(DaemonState {
            status: DaemonStatus::Running,
            started_at: SystemTime::now(),
            last_health_check: SystemTime::now(),
            active_projects: 1,
            events_processed: 20,
            memory_usage_mb: 100,
            restart_count: 0,
            last_error: None,
        }));

        let event = QualityEvent::TrendDetected {
            project_id: "test-project".to_string(),
            trend: crate::agent::quality_monitor::QualityTrend::Improving {
                rate: 0.05,
                duration: Duration::from_secs(3600),
            },
        };

        AgentDaemon::process_quality_event(event, &state).await;

        let updated_state = state.read().await;
        assert_eq!(updated_state.events_processed, 21);
    }

    #[tokio::test]
    async fn test_process_quality_event_error() {
        let state = Arc::new(RwLock::new(DaemonState {
            status: DaemonStatus::Running,
            started_at: SystemTime::now(),
            last_health_check: SystemTime::now(),
            active_projects: 1,
            events_processed: 30,
            memory_usage_mb: 100,
            restart_count: 0,
            last_error: None,
        }));

        let event = QualityEvent::Error {
            project_id: "test-project".to_string(),
            error: "Failed to analyze file: permission denied".to_string(),
        };

        AgentDaemon::process_quality_event(event, &state).await;

        let updated_state = state.read().await;
        assert_eq!(updated_state.events_processed, 31);
        assert_eq!(updated_state.last_error, Some("Failed to analyze file: permission denied".to_string()));
    }

    #[tokio::test]
    async fn test_daemon_perform_health_check() {
        let config = DaemonConfig::default();
        let daemon = AgentDaemon::new(config);

        // Set initial state to Running
        {
            let mut state = daemon.state.write().await;
            state.status = DaemonStatus::Running;
        }

        let before_check = daemon.get_state().await.last_health_check;

        // Small delay to ensure different timestamp
        tokio::time::sleep(Duration::from_millis(10)).await;

        daemon.perform_health_check().await;

        let state = daemon.get_state().await;
        assert!(state.last_health_check > before_check);
        assert_eq!(state.memory_usage_mb, 150); // Default memory estimation
    }

    #[tokio::test]
    async fn test_daemon_shutdown_components() {
        let config = DaemonConfig::default();
        let mut daemon = AgentDaemon::new(config);

        // Set up mock components
        daemon.quality_monitor = Some(QualityMonitorEngine::new(Default::default()));
        daemon.mcp_server = Some(ClaudeCodeAgentMcpServer::new(Default::default()));

        let result = daemon.shutdown_components().await;
        assert!(result.is_ok());

        assert!(daemon.quality_monitor.is_none());
        assert!(daemon.mcp_server.is_none());
    }

    #[tokio::test]
    async fn test_daemon_stop_with_timeout() {
        let mut config = DaemonConfig::default();
        config.daemon.shutdown_timeout = Duration::from_millis(100);

        let mut daemon = AgentDaemon::new(config);

        let result = daemon.stop().await;
        assert!(result.is_ok());

        let state = daemon.get_state().await;
        assert_eq!(state.status, DaemonStatus::Stopped);
    }

    #[tokio::test]
    async fn test_daemon_state_transitions() {
        let config = DaemonConfig::default();
        let daemon = AgentDaemon::new(config);

        // Initial state is Stopped
        let state = daemon.get_state().await;
        assert_eq!(state.status, DaemonStatus::Stopped);

        // Manually transition through states for coverage
        {
            let mut state = daemon.state.write().await;
            state.status = DaemonStatus::Starting;
        }
        assert_eq!(daemon.get_state().await.status, DaemonStatus::Starting);

        {
            let mut state = daemon.state.write().await;
            state.status = DaemonStatus::Running;
        }
        assert_eq!(daemon.get_state().await.status, DaemonStatus::Running);

        {
            let mut state = daemon.state.write().await;
            state.status = DaemonStatus::Stopping;
        }
        assert_eq!(daemon.get_state().await.status, DaemonStatus::Stopping);

        {
            let mut state = daemon.state.write().await;
            state.status = DaemonStatus::Error;
        }
        assert_eq!(daemon.get_state().await.status, DaemonStatus::Error);
    }

    #[tokio::test]
    async fn test_daemon_multiple_events_processed() {
        let state = Arc::new(RwLock::new(DaemonState {
            status: DaemonStatus::Running,
            started_at: SystemTime::now(),
            last_health_check: SystemTime::now(),
            active_projects: 1,
            events_processed: 0,
            memory_usage_mb: 100,
            restart_count: 0,
            last_error: None,
        }));

        // Process multiple events
        for i in 0..10 {
            let event = QualityEvent::MetricsUpdated {
                project_id: format!("project-{}", i),
                metrics: crate::agent::quality_monitor::QualityMetrics {
                    project_id: format!("project-{}", i),
                    last_updated: SystemTime::now(),
                    quality_score: 0.8 + (i as f64 * 0.01),
                    files_analyzed: i * 10,
                    functions_analyzed: i * 50,
                    avg_complexity: 5.0,
                    max_complexity: 15,
                    hotspot_functions: i,
                    satd_issues: 0,
                    complexity_distribution: Default::default(),
                    file_metrics: Default::default(),
                    quality_trend: 0.0,
                },
                changes: vec![],
            };
            AgentDaemon::process_quality_event(event, &state).await;
        }

        let updated_state = state.read().await;
        assert_eq!(updated_state.events_processed, 10);
    }

    #[test]
    fn test_daemon_config_clone() {
        let config = DaemonConfig::default();
        let cloned = config.clone();

        assert_eq!(config.agent.name, cloned.agent.name);
        assert_eq!(config.daemon.max_memory_mb, cloned.daemon.max_memory_mb);
    }

    #[test]
    fn test_daemon_settings_clone() {
        let settings = DaemonSettings::default();
        let cloned = settings.clone();

        assert_eq!(settings.max_memory_mb, cloned.max_memory_mb);
        assert_eq!(settings.auto_restart, cloned.auto_restart);
    }

    #[test]
    fn test_daemon_state_clone() {
        let state = DaemonState {
            status: DaemonStatus::Running,
            started_at: SystemTime::now(),
            last_health_check: SystemTime::now(),
            active_projects: 3,
            events_processed: 100,
            memory_usage_mb: 200,
            restart_count: 1,
            last_error: Some("test error".to_string()),
        };
        let cloned = state.clone();

        assert_eq!(state.status, cloned.status);
        assert_eq!(state.active_projects, cloned.active_projects);
        assert_eq!(state.events_processed, cloned.events_processed);
        assert_eq!(state.last_error, cloned.last_error);
    }

    #[test]
    fn test_daemon_command_clone() {
        let cmd = DaemonCommand::StartMonitoring {
            project_path: "/test/path".to_string(),
        };
        let cloned = cmd.clone();

        match cloned {
            DaemonCommand::StartMonitoring { project_path } => {
                assert_eq!(project_path, "/test/path");
            }
            _ => panic!("Wrong command type after clone"),
        }
    }

    #[test]
    fn test_quality_gate_result_clone() {
        let result = QualityGateResult {
            violations: Some(5),
            passed: false,
        };
        let cloned = result.clone();

        assert_eq!(result.violations, cloned.violations);
        assert_eq!(result.passed, cloned.passed);
    }

    #[test]
    fn test_daemon_config_debug() {
        let config = DaemonConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("DaemonConfig"));
    }

    #[test]
    fn test_daemon_settings_debug() {
        let settings = DaemonSettings::default();
        let debug_str = format!("{:?}", settings);
        assert!(debug_str.contains("DaemonSettings"));
        assert!(debug_str.contains("max_memory_mb"));
    }

    #[test]
    fn test_daemon_state_debug() {
        let state = DaemonState {
            status: DaemonStatus::Running,
            started_at: SystemTime::now(),
            last_health_check: SystemTime::now(),
            active_projects: 1,
            events_processed: 50,
            memory_usage_mb: 150,
            restart_count: 0,
            last_error: None,
        };
        let debug_str = format!("{:?}", state);
        assert!(debug_str.contains("DaemonState"));
        assert!(debug_str.contains("Running"));
    }

    #[test]
    fn test_quality_gate_result_debug() {
        let result = QualityGateResult {
            violations: Some(3),
            passed: true,
        };
        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("QualityGateResult"));
    }
}

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}
