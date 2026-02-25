#![cfg_attr(coverage_nightly, coverage(off))]
//! Sprint 31 Week 2 - Alert System with Configurable Thresholds
//!
//! Provides real-time alerting capabilities with configurable thresholds,
//! notification channels, and alert management features.
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{mpsc, RwLock};

/// Alert definition with threshold and conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub metric: String,
    pub condition: AlertCondition,
    pub threshold: f64,
    pub duration: Duration,
    pub severity: AlertSeverity,
    pub enabled: bool,
    pub notification_channels: Vec<NotificationChannel>,
    pub cooldown_period: Duration,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertCondition {
    GreaterThan,
    LessThan,
    Equal,
    NotEqual,
    GreaterThanOrEqual,
    LessThanOrEqual,
    RateOfChange,
    Anomaly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl AlertSeverity {
    #[must_use]
    pub fn priority(&self) -> u8 {
        match self {
            AlertSeverity::Info => 1,
            AlertSeverity::Warning => 2,
            AlertSeverity::Error => 3,
            AlertSeverity::Critical => 4,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NotificationChannel {
    Dashboard,
    Email { recipients: Vec<String> },
    Webhook { url: String, method: String },
    Slack { webhook_url: String, channel: String },
    PagerDuty { integration_key: String },
    Log { level: String },
}

/// Active alert instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub rule_id: String,
    pub rule_name: String,
    pub severity: AlertSeverity,
    pub state: AlertState,
    pub triggered_at: SystemTime,
    pub resolved_at: Option<SystemTime>,
    pub metric_value: f64,
    pub threshold_value: f64,
    pub message: String,
    pub context: HashMap<String, String>,
    pub notification_sent: bool,
    pub acknowledgement: Option<Acknowledgement>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertState {
    Triggered,
    Active,
    Acknowledged,
    Resolved,
    Silenced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Acknowledgement {
    pub acknowledged_by: String,
    pub acknowledged_at: SystemTime,
    pub comment: Option<String>,
}

/// Alert manager for the TDG system
pub struct AlertManager {
    rules: Arc<RwLock<HashMap<String, AlertRule>>>,
    active_alerts: Arc<RwLock<HashMap<String, Alert>>>,
    alert_history: Arc<RwLock<VecDeque<Alert>>>,
    metric_values: Arc<RwLock<HashMap<String, MetricValue>>>,
    notification_tx: mpsc::UnboundedSender<Alert>,
    #[allow(dead_code)]
    notification_rx: Arc<RwLock<mpsc::UnboundedReceiver<Alert>>>,
    statistics: Arc<RwLock<AlertStatistics>>,
    config: AlertManagerConfig,
}

// Configuration, supporting types (AlertManagerConfig, MetricValue, AlertStatistics, etc.)
include!("alerts_types.rs");

// AlertManager implementation (rule evaluation, triggering, resolution)
include!("alerts_manager.rs");

// Default TDG alert rules
include!("alerts_defaults.rs");

#[cfg(test)]
#[path = "alerts_tests.rs"]
mod tests;
