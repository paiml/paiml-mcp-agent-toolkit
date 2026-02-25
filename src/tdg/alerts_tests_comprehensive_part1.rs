    #[test]
    fn test_alert_severity_priority() {
        assert_eq!(AlertSeverity::Info.priority(), 1);
        assert_eq!(AlertSeverity::Warning.priority(), 2);
        assert_eq!(AlertSeverity::Error.priority(), 3);
        assert_eq!(AlertSeverity::Critical.priority(), 4);
    }

    #[test]
    fn test_alert_severity_clone() {
        let severity = AlertSeverity::Critical;
        let cloned = severity.clone();
        assert_eq!(cloned, AlertSeverity::Critical);
    }

    #[test]
    fn test_alert_severity_debug() {
        let severity = AlertSeverity::Warning;
        let debug = format!("{:?}", severity);
        assert!(debug.contains("Warning"));
    }

    #[test]
    fn test_alert_severity_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(AlertSeverity::Info);
        set.insert(AlertSeverity::Warning);
        set.insert(AlertSeverity::Critical);
        assert_eq!(set.len(), 3);
    }

    #[test]
    fn test_alert_condition_clone() {
        let condition = AlertCondition::GreaterThan;
        let cloned = condition.clone();
        assert_eq!(cloned, AlertCondition::GreaterThan);
    }

    #[test]
    fn test_alert_condition_debug() {
        let condition = AlertCondition::LessThanOrEqual;
        let debug = format!("{:?}", condition);
        assert!(debug.contains("LessThanOrEqual"));
    }

    #[test]
    fn test_alert_state_clone() {
        let state = AlertState::Active;
        let cloned = state.clone();
        assert_eq!(cloned, AlertState::Active);
    }

    #[test]
    fn test_alert_state_debug() {
        let state = AlertState::Silenced;
        let debug = format!("{:?}", state);
        assert!(debug.contains("Silenced"));
    }

    #[test]
    fn test_alert_condition_variants() {
        assert_eq!(AlertCondition::GreaterThan, AlertCondition::GreaterThan);
        assert_eq!(AlertCondition::LessThan, AlertCondition::LessThan);
        assert_eq!(AlertCondition::Equal, AlertCondition::Equal);
        assert_eq!(AlertCondition::NotEqual, AlertCondition::NotEqual);
        assert_eq!(
            AlertCondition::GreaterThanOrEqual,
            AlertCondition::GreaterThanOrEqual
        );
        assert_eq!(
            AlertCondition::LessThanOrEqual,
            AlertCondition::LessThanOrEqual
        );
        assert_eq!(AlertCondition::RateOfChange, AlertCondition::RateOfChange);
        assert_eq!(AlertCondition::Anomaly, AlertCondition::Anomaly);
    }

    #[test]
    fn test_alert_state_variants() {
        assert_eq!(AlertState::Triggered, AlertState::Triggered);
        assert_eq!(AlertState::Active, AlertState::Active);
        assert_eq!(AlertState::Acknowledged, AlertState::Acknowledged);
        assert_eq!(AlertState::Resolved, AlertState::Resolved);
        assert_eq!(AlertState::Silenced, AlertState::Silenced);
    }

    #[test]
    fn test_alert_manager_config_default() {
        let config = AlertManagerConfig::default();
        assert_eq!(config.max_active_alerts, 100);
        assert_eq!(config.max_history_size, 1000);
        assert_eq!(config.evaluation_interval, Duration::from_secs(10));
        assert_eq!(config.default_cooldown, Duration::from_secs(300));
        assert!(config.enable_auto_resolve);
        assert!(config.silence_duplicate_alerts);
    }

    #[test]
    fn test_alert_statistics_default() {
        let stats = AlertStatistics::default();
        assert_eq!(stats.total_triggered, 0);
        assert_eq!(stats.total_resolved, 0);
        assert_eq!(stats.total_acknowledged, 0);
        assert!(stats.alerts_by_severity.is_empty());
        assert_eq!(stats.mean_time_to_acknowledge_ms, 0.0);
        assert_eq!(stats.mean_time_to_resolve_ms, 0.0);
        assert_eq!(stats.false_positive_rate, 0.0);
    }

    #[test]
    fn test_alert_rule_creation() {
        let rule = AlertRule {
            id: "rule1".to_string(),
            name: "CPU Alert".to_string(),
            description: "Alert when CPU > 80%".to_string(),
            metric: "cpu_usage".to_string(),
            condition: AlertCondition::GreaterThan,
            threshold: 80.0,
            duration: Duration::from_secs(60),
            severity: AlertSeverity::Warning,
            enabled: true,
            notification_channels: vec![NotificationChannel::Dashboard],
            cooldown_period: Duration::from_secs(300),
            metadata: HashMap::new(),
        };

        assert_eq!(rule.id, "rule1");
        assert_eq!(rule.threshold, 80.0);
        assert!(rule.enabled);
    }

    #[test]
    fn test_notification_channel_variants() {
        let dashboard = NotificationChannel::Dashboard;
        let email = NotificationChannel::Email {
            recipients: vec!["test@example.com".to_string()],
        };
        let webhook = NotificationChannel::Webhook {
            url: "https://example.com/webhook".to_string(),
            method: "POST".to_string(),
        };
        let slack = NotificationChannel::Slack {
            webhook_url: "https://hooks.slack.com/...".to_string(),
            channel: "#alerts".to_string(),
        };
        let pagerduty = NotificationChannel::PagerDuty {
            integration_key: "key123".to_string(),
        };
        let log = NotificationChannel::Log {
            level: "ERROR".to_string(),
        };

        // Verify all variants can be created
        assert!(matches!(dashboard, NotificationChannel::Dashboard));
        assert!(matches!(email, NotificationChannel::Email { .. }));
        assert!(matches!(webhook, NotificationChannel::Webhook { .. }));
        assert!(matches!(slack, NotificationChannel::Slack { .. }));
        assert!(matches!(pagerduty, NotificationChannel::PagerDuty { .. }));
        assert!(matches!(log, NotificationChannel::Log { .. }));
    }

    #[test]
    fn test_alert_creation() {
        let alert = Alert {
            id: "alert1".to_string(),
            rule_id: "rule1".to_string(),
            rule_name: "CPU Alert".to_string(),
            severity: AlertSeverity::Warning,
            state: AlertState::Triggered,
            triggered_at: SystemTime::now(),
            resolved_at: None,
            metric_value: 85.0,
            threshold_value: 80.0,
            message: "CPU usage exceeded threshold".to_string(),
            context: HashMap::new(),
            notification_sent: false,
            acknowledgement: None,
        };

        assert_eq!(alert.id, "alert1");
        assert_eq!(alert.metric_value, 85.0);
        assert!(alert.resolved_at.is_none());
        assert!(!alert.notification_sent);
    }

    #[test]
    fn test_acknowledgement_creation() {
        let ack = Acknowledgement {
            acknowledged_by: "admin".to_string(),
            acknowledged_at: SystemTime::now(),
            comment: Some("Looking into it".to_string()),
        };

        assert_eq!(ack.acknowledged_by, "admin");
        assert!(ack.comment.is_some());
    }

    #[test]
    fn test_metric_value_creation() {
        let mut tags = HashMap::new();
        tags.insert("host".to_string(), "server1".to_string());

        let metric = MetricValue {
            value: 42.5,
            timestamp: SystemTime::now(),
            tags,
        };

        assert_eq!(metric.value, 42.5);
        assert_eq!(metric.tags.get("host"), Some(&"server1".to_string()));
    }

    #[tokio::test]
    async fn test_alert_manager_creation() {
        let manager = AlertManager::new(AlertManagerConfig::default());
        let alerts = manager.get_active_alerts().await;
        assert!(alerts.is_empty());
    }

    #[tokio::test]
    async fn test_alert_manager_add_rule() {
        let manager = AlertManager::new(AlertManagerConfig::default());

        let rule = AlertRule {
            id: "test_rule".to_string(),
            name: "Test".to_string(),
            description: "Test rule".to_string(),
            metric: "test_metric".to_string(),
            condition: AlertCondition::GreaterThan,
            threshold: 50.0,
            duration: Duration::from_secs(1),
            severity: AlertSeverity::Info,
            enabled: true,
            notification_channels: vec![],
            cooldown_period: Duration::from_secs(60),
            metadata: HashMap::new(),
        };

        let result = manager.add_rule(rule).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_alert_manager_get_statistics() {
        let manager = AlertManager::new(AlertManagerConfig::default());
        let stats = manager.get_statistics().await;

        assert_eq!(stats.total_triggered, 0);
        assert_eq!(stats.total_resolved, 0);
    }

    #[tokio::test]
    async fn test_silence_alert_nonexistent() {
        let manager = AlertManager::new(AlertManagerConfig::default());
        let result = manager
            .silence_alert("nonexistent", Duration::from_secs(60))
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_remove_rule() {
        let manager = AlertManager::new(AlertManagerConfig::default());

        let rule = AlertRule {
            id: "to_remove".to_string(),
            name: "Test".to_string(),
            description: "Test rule".to_string(),
            metric: "test_metric".to_string(),
            condition: AlertCondition::GreaterThan,
            threshold: 50.0,
            duration: Duration::from_secs(1),
            severity: AlertSeverity::Info,
            enabled: true,
            notification_channels: vec![],
            cooldown_period: Duration::from_secs(60),
            metadata: HashMap::new(),
        };

        manager.add_rule(rule).await.unwrap();
        let result = manager.remove_rule("to_remove").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_alerts_by_severity() {
        let manager = AlertManager::new(AlertManagerConfig::default());
        let alerts = manager.get_alerts_by_severity(AlertSeverity::Warning).await;
        assert!(alerts.is_empty());
    }

    #[tokio::test]
    async fn test_export_config() {
        let manager = AlertManager::new(AlertManagerConfig::default());
        let config = manager.export_config().await;
        assert!(config.rules.is_empty());
    }

    #[test]
    fn test_default_tdg_alert_rules() {
        let rules = default_tdg_alert_rules();
        assert_eq!(rules.len(), 4);

        // Verify cpu usage rule exists
        let cpu_rule = rules.iter().find(|r| r.metric == "cpu_usage_percent");
        assert!(cpu_rule.is_some());

        // Verify memory usage rule exists
        let memory_rule = rules.iter().find(|r| r.metric == "memory_usage_mb");
        assert!(memory_rule.is_some());

        // Verify analysis time rule exists
        let analysis_rule = rules.iter().find(|r| r.metric == "avg_analysis_time_ms");
        assert!(analysis_rule.is_some());

        // Verify cache hit rule exists
        let cache_rule = rules.iter().find(|r| r.metric == "cache_hit_ratio");
        assert!(cache_rule.is_some());
    }

    #[test]
    fn test_alert_rule_clone() {
        let rule = AlertRule {
            id: "test".to_string(),
            name: "Test Rule".to_string(),
            description: "Desc".to_string(),
            metric: "cpu".to_string(),
            condition: AlertCondition::GreaterThan,
            threshold: 80.0,
            duration: Duration::from_secs(60),
            severity: AlertSeverity::Warning,
            enabled: true,
            notification_channels: vec![NotificationChannel::Dashboard],
            cooldown_period: Duration::from_secs(300),
            metadata: HashMap::new(),
        };
        let cloned = rule.clone();
        assert_eq!(cloned.id, "test");
        assert_eq!(cloned.threshold, 80.0);
    }

    #[test]
    fn test_alert_rule_debug() {
        let rule = AlertRule {
            id: "dbg_test".to_string(),
            name: "Debug Test".to_string(),
            description: "Desc".to_string(),
            metric: "mem".to_string(),
            condition: AlertCondition::LessThan,
            threshold: 100.0,
            duration: Duration::from_secs(30),
            severity: AlertSeverity::Info,
            enabled: false,
            notification_channels: vec![],
            cooldown_period: Duration::from_secs(60),
            metadata: HashMap::new(),
        };
        let debug = format!("{:?}", rule);
        assert!(debug.contains("dbg_test"));
    }

    #[test]
    fn test_alert_clone() {
        let alert = Alert {
            id: "alert123".to_string(),
            rule_id: "rule1".to_string(),
            rule_name: "Test Alert".to_string(),
            severity: AlertSeverity::Error,
            state: AlertState::Active,
            triggered_at: SystemTime::now(),
            resolved_at: None,
            metric_value: 95.0,
            threshold_value: 90.0,
            message: "Threshold exceeded".to_string(),
            context: HashMap::new(),
            notification_sent: true,
            acknowledgement: None,
        };
        let cloned = alert.clone();
        assert_eq!(cloned.id, "alert123");
        assert_eq!(cloned.metric_value, 95.0);
        assert!(cloned.notification_sent);
    }

    #[test]
    fn test_alert_debug() {
        let alert = Alert {
            id: "dbg_alert".to_string(),
            rule_id: "rule1".to_string(),
            rule_name: "Debug Alert".to_string(),
            severity: AlertSeverity::Critical,
            state: AlertState::Triggered,
            triggered_at: SystemTime::now(),
            resolved_at: Some(SystemTime::now()),
            metric_value: 100.0,
            threshold_value: 80.0,
            message: "Test message".to_string(),
            context: HashMap::new(),
            notification_sent: false,
            acknowledgement: None,
        };
        let debug = format!("{:?}", alert);
        assert!(debug.contains("dbg_alert"));
    }

    #[test]
    fn test_acknowledgement_clone() {
        let ack = Acknowledgement {
            acknowledged_by: "user1".to_string(),
            acknowledged_at: SystemTime::now(),
            comment: Some("Investigating".to_string()),
        };
        let cloned = ack.clone();
        assert_eq!(cloned.acknowledged_by, "user1");
        assert!(cloned.comment.is_some());
    }

    #[test]
    fn test_acknowledgement_debug() {
        let ack = Acknowledgement {
            acknowledged_by: "debug_user".to_string(),
            acknowledged_at: SystemTime::now(),
            comment: None,
        };
        let debug = format!("{:?}", ack);
        assert!(debug.contains("debug_user"));
    }

    #[test]
    fn test_metric_value_clone() {
        let metric = MetricValue {
            value: 42.5,
            timestamp: SystemTime::now(),
            tags: HashMap::new(),
        };
        let cloned = metric.clone();
        assert_eq!(cloned.value, 42.5);
    }

    #[test]
    fn test_metric_value_debug() {
        let metric = MetricValue {
            value: 123.456,
            timestamp: SystemTime::now(),
            tags: HashMap::new(),
        };
        let debug = format!("{:?}", metric);
        assert!(debug.contains("123.456"));
    }

    #[test]
    fn test_alert_statistics_clone() {
        let mut stats = AlertStatistics::default();
        stats.total_triggered = 10;
        stats.total_resolved = 8;
        let cloned = stats.clone();
        assert_eq!(cloned.total_triggered, 10);
        assert_eq!(cloned.total_resolved, 8);
    }

    #[test]
    fn test_alert_statistics_debug() {
        let stats = AlertStatistics {
            total_triggered: 100,
            total_resolved: 90,
            total_acknowledged: 95,
            alerts_by_severity: HashMap::new(),
            mean_time_to_acknowledge_ms: 5000.0,
            mean_time_to_resolve_ms: 10000.0,
            false_positive_rate: 0.05,
        };
        let debug = format!("{:?}", stats);
        assert!(debug.contains("total_triggered"));
    }

    #[test]
    fn test_alert_manager_config_clone() {
        let config = AlertManagerConfig {
            max_active_alerts: 50,
            max_history_size: 500,
            evaluation_interval: Duration::from_secs(5),
            default_cooldown: Duration::from_secs(60),
            enable_auto_resolve: false,
            silence_duplicate_alerts: false,
        };
        let cloned = config.clone();
        assert_eq!(cloned.max_active_alerts, 50);
        assert!(!cloned.enable_auto_resolve);
    }

    #[test]
    fn test_alert_manager_config_debug() {
        let config = AlertManagerConfig::default();
        let debug = format!("{:?}", config);
        assert!(debug.contains("max_active_alerts"));
    }

