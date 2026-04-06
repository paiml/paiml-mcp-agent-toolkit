// Default TDG alert rules
// Included from alerts.rs - shares parent module scope

/// Default alert rules for TDG system
#[must_use]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn default_tdg_alert_rules() -> Vec<AlertRule> {
    vec![
        AlertRule {
            id: "high_cpu".to_string(),
            name: "High CPU Usage".to_string(),
            description: "CPU usage exceeds critical threshold".to_string(),
            metric: "cpu_usage_percent".to_string(),
            condition: AlertCondition::GreaterThan,
            threshold: 90.0,
            duration: Duration::from_secs(60),
            severity: AlertSeverity::Critical,
            enabled: true,
            notification_channels: vec![NotificationChannel::Dashboard],
            cooldown_period: Duration::from_secs(300),
            metadata: HashMap::new(),
        },
        AlertRule {
            id: "high_memory".to_string(),
            name: "High Memory Usage".to_string(),
            description: "Memory usage exceeds warning threshold".to_string(),
            metric: "memory_usage_mb".to_string(),
            condition: AlertCondition::GreaterThan,
            threshold: 8192.0,
            duration: Duration::from_secs(120),
            severity: AlertSeverity::Warning,
            enabled: true,
            notification_channels: vec![NotificationChannel::Dashboard],
            cooldown_period: Duration::from_secs(600),
            metadata: HashMap::new(),
        },
        AlertRule {
            id: "slow_analysis".to_string(),
            name: "Slow Analysis Time".to_string(),
            description: "Analysis taking longer than expected".to_string(),
            metric: "avg_analysis_time_ms".to_string(),
            condition: AlertCondition::GreaterThan,
            threshold: 5000.0,
            duration: Duration::from_secs(30),
            severity: AlertSeverity::Warning,
            enabled: true,
            notification_channels: vec![NotificationChannel::Dashboard],
            cooldown_period: Duration::from_secs(180),
            metadata: HashMap::new(),
        },
        AlertRule {
            id: "low_cache_hit".to_string(),
            name: "Low Cache Hit Ratio".to_string(),
            description: "Cache hit ratio below optimal level".to_string(),
            metric: "cache_hit_ratio".to_string(),
            condition: AlertCondition::LessThan,
            threshold: 0.7,
            duration: Duration::from_secs(300),
            severity: AlertSeverity::Info,
            enabled: true,
            notification_channels: vec![NotificationChannel::Dashboard],
            cooldown_period: Duration::from_secs(900),
            metadata: HashMap::new(),
        },
    ]
}
