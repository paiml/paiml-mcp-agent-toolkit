#![cfg_attr(coverage_nightly, coverage(off))]
//! Service lifecycle management per SPECIFICATION.md Section 2.2
//!
//! This module provides lifecycle management for services including
//! initialization, health checks, graceful shutdown, and restart capabilities.

use super::service_base::{Service, ServiceMetrics};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{error, info, warn};

/// Service lifecycle states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceState {
    /// Service is not yet initialized
    Uninitialized,
    /// Service is starting up
    Starting,
    /// Service is running and healthy
    Running,
    /// Service is degraded but operational
    Degraded,
    /// Service is stopping
    Stopping,
    /// Service has stopped
    Stopped,
    /// Service has failed
    Failed,
}

/// Health status for a service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub state: ServiceState,
    pub message: String,
    pub last_check: std::time::SystemTime,
    pub uptime_seconds: u64,
    pub metrics: ServiceMetrics,
}

/// Lifecycle events that can occur
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LifecycleEvent {
    Started,
    Stopped,
    HealthCheckPassed,
    HealthCheckFailed(String),
    StateChanged(ServiceState),
    Error(String),
}

/// Trait for services with lifecycle management
#[async_trait]
pub trait ManagedService: Service {
    /// Initialize the service
    async fn initialize(&self) -> Result<()> {
        Ok(())
    }

    /// Perform health check
    async fn health_check(&self) -> Result<HealthStatus>;

    /// Gracefully shutdown the service
    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    /// Handle lifecycle event
    fn handle_event(&mut self, event: LifecycleEvent) {
        if let LifecycleEvent::Error(msg) = event {
            error!("Service error: {}", msg);
        }
    }
}

/// Wrapper that adds lifecycle management to any service
pub struct LifecycleWrapper<S: Service> {
    service: Arc<RwLock<S>>,
    state: Arc<RwLock<ServiceState>>,
    running: Arc<AtomicBool>,
    start_time: std::time::SystemTime,
    health_check_interval: Duration,
    metrics: Arc<RwLock<ServiceMetrics>>,
}

/// Type alias for managed service trait objects
type ManagedServiceObject = dyn ManagedService<Input = serde_json::Value, Output = serde_json::Value, Error = anyhow::Error>
    + Send
    + Sync;

/// Service supervisor that manages multiple services
pub struct ServiceSupervisor {
    services: Arc<RwLock<Vec<Arc<ManagedServiceObject>>>>,
    running: Arc<AtomicBool>,
}

include!("service_lifecycle_wrapper.rs");
include!("service_lifecycle_supervisor.rs");
include!("service_lifecycle_tests.rs");
