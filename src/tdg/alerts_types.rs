// Alert manager configuration and supporting types
// Included from alerts.rs - shares parent module scope

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertManagerConfig {
    pub max_active_alerts: usize,
    pub max_history_size: usize,
    #[serde(with = "serde_duration")]
    pub evaluation_interval: Duration,
    #[serde(with = "serde_duration")]
    pub default_cooldown: Duration,
    pub enable_auto_resolve: bool,
    pub silence_duplicate_alerts: bool,
}

// Helper module for Duration serialization
mod serde_duration {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub(super) fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_secs())
    }

    pub(super) fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

impl Default for AlertManagerConfig {
    fn default() -> Self {
        Self {
            max_active_alerts: 100,
            max_history_size: 1000,
            evaluation_interval: Duration::from_secs(10),
            default_cooldown: Duration::from_secs(300),
            enable_auto_resolve: true,
            silence_duplicate_alerts: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricValue {
    pub value: f64,
    pub timestamp: SystemTime,
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AlertStatistics {
    pub total_triggered: u64,
    pub total_resolved: u64,
    pub total_acknowledged: u64,
    pub alerts_by_severity: HashMap<AlertSeverity, u64>,
    pub mean_time_to_acknowledge_ms: f64,
    pub mean_time_to_resolve_ms: f64,
    pub false_positive_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfiguration {
    pub rules: Vec<AlertRule>,
    pub config: AlertManagerConfig,
}
