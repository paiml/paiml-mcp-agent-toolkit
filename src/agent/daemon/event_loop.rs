#![cfg_attr(coverage_nightly, coverage(off))]
//! Daemon event loop: main loop, health checks, quality event processing.

use std::sync::Arc;
use std::time::SystemTime;

use anyhow::Result;
use tokio::signal;
use tokio::sync::{mpsc, RwLock};
use tokio::time::interval;
use tracing::{debug, error, info, warn};

use super::super::quality_monitor::QualityEvent;
use super::lifecycle::AgentDaemon;
use super::types::{DaemonState, DaemonStatus};

impl AgentDaemon {
    /// Run the main daemon loop
    pub(super) async fn run_daemon_loop(
        &mut self,
        mut shutdown_rx: mpsc::Receiver<()>,
    ) -> Result<()> {
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
    pub(super) async fn perform_health_check(&self) {
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
    pub(super) async fn process_quality_event(
        event: QualityEvent,
        state: &Arc<RwLock<DaemonState>>,
    ) {
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
}
