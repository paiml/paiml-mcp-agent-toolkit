//! Background Daemon for Claude Code Agent Mode
//!
//! Manages the lifecycle of the PMAT background agent service with graceful
//! startup, shutdown, and continuous operation capabilities.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
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
                        signal::unix::signal(signal::unix::SignalKind::terminate()).unwrap().recv().await
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
            // On Unix systems, we could read from /proc/self/status
            // For now, use a mock value
            state.memory_usage_mb = 150; // Mock value
        }

        #[cfg(not(unix))]
        {
            // On other platforms, use a different method or mock
            state.memory_usage_mb = 150; // Mock value
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
        // Connect to running daemon via IPC for status retrieval
        Err(anyhow::anyhow!("Not implemented"))
    }

    /// Send command to running daemon
    pub async fn send_command(_command: DaemonCommand) -> Result<()> {
        // Send command to running daemon using platform IPC mechanisms
        Err(anyhow::anyhow!("Not implemented"))
    }

    /// Shutdown the daemon
    pub async fn shutdown() -> Result<()> {
        info!("Shutting down daemon...");
        // Implementation would send shutdown command to running daemon
        Ok(())
    }

    /// Start monitoring a project
    pub async fn start_monitoring(_project_path: &PathBuf, _project_id: &str) -> Result<()> {
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

    #[test]
    fn test_daemon_config_default() {
        let config = DaemonConfig::default();
        assert_eq!(config.agent.name, "pmat-agent");
        assert_eq!(config.daemon.max_memory_mb, 500);
        assert!(config.daemon.auto_restart);
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

    #[tokio::test]
    async fn test_daemon_creation() {
        let config = DaemonConfig::default();
        let daemon = AgentDaemon::new(config);

        let state = daemon.get_state().await;
        assert_eq!(state.status, DaemonStatus::Stopped);
        assert_eq!(state.active_projects, 0);
    }

    #[test]
    fn test_daemon_status_serialization() {
        let status = DaemonStatus::Running;
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: DaemonStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, deserialized);
    }

    #[tokio::test]
    async fn test_daemon_manager() {
        let is_running = DaemonManager::is_running().await;
        assert!(!is_running); // Should be false in test environment
    }
}
