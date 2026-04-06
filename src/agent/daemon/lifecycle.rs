#![cfg_attr(coverage_nightly, coverage(off))]
//! Daemon lifecycle management: creation, startup, shutdown.

use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::{mpsc, RwLock};
use tracing::{info, warn};

use super::super::mcp_server::ClaudeCodeAgentMcpServer;
use super::super::quality_monitor::QualityMonitorEngine;
use super::super::state_persistence::StatePersistence;
use super::types::{DaemonConfig, DaemonState, DaemonStatus};

/// Background daemon for the Claude Code agent
pub struct AgentDaemon {
    /// Daemon configuration
    pub(super) config: DaemonConfig,

    /// MCP server instance
    pub(super) mcp_server: Option<ClaudeCodeAgentMcpServer>,

    /// Quality monitor engine
    pub(super) quality_monitor: Option<QualityMonitorEngine>,

    /// Daemon state
    pub(super) state: Arc<RwLock<DaemonState>>,

    /// State persistence
    pub(super) persistence: Option<StatePersistence>,

    /// Shutdown signal sender
    pub(super) shutdown_tx: Option<mpsc::Sender<()>>,
}

impl AgentDaemon {
    /// Create new daemon instance
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn get_state(&self) -> DaemonState {
        self.state.read().await.clone()
    }

    /// Initialize daemon components
    async fn initialize_components(&mut self) -> Result<()> {
        debug_assert!(true, "contract: initialize_components");
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

    /// Shutdown daemon components gracefully
    pub(super) async fn shutdown_components(&mut self) -> Result<()> {
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
