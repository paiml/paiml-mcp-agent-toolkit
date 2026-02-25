
    #[test]
    fn test_alert_resolved_with_timestamp() {
        let triggered = SystemTime::now();
        let resolved = triggered + Duration::from_secs(60);

        let alert = Alert {
            id: "resolved_alert".to_string(),
            rule_id: "rule1".to_string(),
            rule_name: "Resolved Alert".to_string(),
            severity: AlertSeverity::Warning,
            state: AlertState::Resolved,
            triggered_at: triggered,
            resolved_at: Some(resolved),
            metric_value: 85.0,
            threshold_value: 90.0,
            message: "Below threshold".to_string(),
            context: HashMap::new(),
            notification_sent: true,
            acknowledgement: None,
        };

        assert!(alert.resolved_at.is_some());
        assert_eq!(alert.state, AlertState::Resolved);
        let duration = alert
            .resolved_at
            .unwrap()
            .duration_since(alert.triggered_at)
            .unwrap();
        assert_eq!(duration.as_secs(), 60);
    }

    #[test]
    fn test_alert_severity_all_priorities() {
        assert!(AlertSeverity::Info.priority() < AlertSeverity::Warning.priority());
        assert!(AlertSeverity::Warning.priority() < AlertSeverity::Error.priority());
        assert!(AlertSeverity::Error.priority() < AlertSeverity::Critical.priority());
    }

    #[test]
    fn test_alert_condition_rate_of_change() {
        let condition = AlertCondition::RateOfChange;
        let json = serde_json::to_string(&condition).unwrap();
        let deserialized: AlertCondition = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, AlertCondition::RateOfChange);
    }

    #[test]
    fn test_alert_condition_anomaly() {
        let condition = AlertCondition::Anomaly;
        let json = serde_json::to_string(&condition).unwrap();
        let deserialized: AlertCondition = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, AlertCondition::Anomaly);
    }

    #[test]
    fn test_notification_channel_webhook_serialization() {
        let channel = NotificationChannel::Webhook {
            url: "https://api.example.com/webhook".to_string(),
            method: "POST".to_string(),
        };
        let json = serde_json::to_string(&channel).unwrap();
        let deserialized: NotificationChannel = serde_json::from_str(&json).unwrap();
        if let NotificationChannel::Webhook { url, method } = deserialized {
            assert_eq!(url, "https://api.example.com/webhook");
            assert_eq!(method, "POST");
        } else {
            panic!("Expected Webhook variant");
        }
    }

    #[test]
    fn test_metric_value_with_multiple_tags() {
        let mut tags = HashMap::new();
        tags.insert("host".to_string(), "server-1".to_string());
        tags.insert("region".to_string(), "us-west-2".to_string());
        tags.insert("env".to_string(), "production".to_string());
        tags.insert("cluster".to_string(), "main".to_string());

        let metric = MetricValue {
            value: 95.5,
            timestamp: SystemTime::now(),
            tags,
        };

        assert_eq!(metric.tags.len(), 4);
        assert_eq!(metric.tags.get("cluster"), Some(&"main".to_string()));
    }

    #[test]
    fn test_alert_statistics_with_severity_counts() {
        let mut alerts_by_severity = HashMap::new();
        alerts_by_severity.insert(AlertSeverity::Info, 10);
        alerts_by_severity.insert(AlertSeverity::Warning, 25);
        alerts_by_severity.insert(AlertSeverity::Error, 5);
        alerts_by_severity.insert(AlertSeverity::Critical, 2);

        let stats = AlertStatistics {
            total_triggered: 42,
            total_resolved: 38,
            total_acknowledged: 40,
            alerts_by_severity,
            mean_time_to_acknowledge_ms: 2500.0,
            mean_time_to_resolve_ms: 15000.0,
            false_positive_rate: 0.03,
        };

        assert_eq!(
            stats.alerts_by_severity.get(&AlertSeverity::Warning),
            Some(&25)
        );
        assert_eq!(stats.alerts_by_severity.len(), 4);
    }

    #[test]
    fn test_alert_rule_all_notification_channels() {
        let rule = AlertRule {
            id: "multi_channel".to_string(),
            name: "Multi Channel Alert".to_string(),
            description: "Alerts to multiple channels".to_string(),
            metric: "cpu".to_string(),
            condition: AlertCondition::GreaterThan,
            threshold: 90.0,
            duration: Duration::from_secs(60),
            severity: AlertSeverity::Critical,
            enabled: true,
            notification_channels: vec![
                NotificationChannel::Dashboard,
                NotificationChannel::Email {
                    recipients: vec!["alert@example.com".to_string()],
                },
                NotificationChannel::Slack {
                    webhook_url: "https://hooks.slack.com/test".to_string(),
                    channel: "#alerts".to_string(),
                },
                NotificationChannel::PagerDuty {
                    integration_key: "pd-key".to_string(),
                },
                NotificationChannel::Log {
                    level: "ERROR".to_string(),
                },
            ],
            cooldown_period: Duration::from_secs(300),
            metadata: HashMap::new(),
        };

        assert_eq!(rule.notification_channels.len(), 5);
    }

    #[test]
    fn test_alert_manager_config_custom() {
        let config = AlertManagerConfig {
            max_active_alerts: 200,
            max_history_size: 5000,
            evaluation_interval: Duration::from_secs(5),
            default_cooldown: Duration::from_secs(120),
            enable_auto_resolve: false,
            silence_duplicate_alerts: false,
        };

        assert_eq!(config.max_active_alerts, 200);
        assert_eq!(config.max_history_size, 5000);
        assert!(!config.enable_auto_resolve);
        assert!(!config.silence_duplicate_alerts);
    }

    #[test]
    fn test_alert_rule_short_duration() {
        let rule = AlertRule {
            id: "instant".to_string(),
            name: "Instant Alert".to_string(),
            description: "Fires immediately".to_string(),
            metric: "error_rate".to_string(),
            condition: AlertCondition::GreaterThan,
            threshold: 0.5,
            duration: Duration::from_millis(100),
            severity: AlertSeverity::Critical,
            enabled: true,
            notification_channels: vec![],
            cooldown_period: Duration::from_millis(500),
            metadata: HashMap::new(),
        };

        assert_eq!(rule.duration.as_millis(), 100);
        assert_eq!(rule.cooldown_period.as_millis(), 500);
    }

    #[test]
    fn test_acknowledgement_no_comment() {
        let ack = Acknowledgement {
            acknowledged_by: "auto-system".to_string(),
            acknowledged_at: SystemTime::now(),
            comment: None,
        };

        assert!(ack.comment.is_none());
        assert_eq!(ack.acknowledged_by, "auto-system");
    }

    #[test]
    fn test_alert_states_transitions() {
        // Verify all state transitions are valid representations
        let states = vec![
            AlertState::Triggered,
            AlertState::Active,
            AlertState::Acknowledged,
            AlertState::Resolved,
            AlertState::Silenced,
        ];

        for state in &states {
            let json = serde_json::to_string(state).unwrap();
            let deserialized: AlertState = serde_json::from_str(&json).unwrap();
            assert_eq!(&deserialized, state);
        }
    }

    #[test]
    fn test_alert_condition_all_variants_serialization() {
        let conditions = vec![
            AlertCondition::GreaterThan,
            AlertCondition::LessThan,
            AlertCondition::Equal,
            AlertCondition::NotEqual,
            AlertCondition::GreaterThanOrEqual,
            AlertCondition::LessThanOrEqual,
            AlertCondition::RateOfChange,
            AlertCondition::Anomaly,
        ];

        for condition in &conditions {
            let json = serde_json::to_string(condition).unwrap();
            let deserialized: AlertCondition = serde_json::from_str(&json).unwrap();
            assert_eq!(&deserialized, condition);
        }
    }

    #[test]
    fn test_alert_severity_equality() {
        assert_eq!(AlertSeverity::Info, AlertSeverity::Info);
        assert_ne!(AlertSeverity::Info, AlertSeverity::Warning);
        assert_ne!(AlertSeverity::Warning, AlertSeverity::Error);
        assert_ne!(AlertSeverity::Error, AlertSeverity::Critical);
    }

    #[test]
    fn test_notification_channel_equality() {
        let dash1 = NotificationChannel::Dashboard;
        let dash2 = NotificationChannel::Dashboard;
        assert_eq!(dash1, dash2);

        let email1 = NotificationChannel::Email {
            recipients: vec!["a@b.com".to_string()],
        };
        let email2 = NotificationChannel::Email {
            recipients: vec!["a@b.com".to_string()],
        };
        assert_eq!(email1, email2);
    }

    #[test]
    fn test_default_tdg_rules_cooldown_periods() {
        let rules = default_tdg_alert_rules();

        for rule in &rules {
            // All rules should have positive cooldown
            assert!(rule.cooldown_period > Duration::ZERO);
            // All rules should have reasonable duration
            assert!(rule.duration > Duration::ZERO);
        }
    }

    #[test]
    fn test_metric_value_empty_tags() {
        let metric = MetricValue {
            value: 0.0,
            timestamp: SystemTime::UNIX_EPOCH,
            tags: HashMap::new(),
        };

        assert!(metric.tags.is_empty());
        assert_eq!(metric.value, 0.0);
    }

    #[test]
    fn test_alert_statistics_zero_values() {
        let stats = AlertStatistics {
            total_triggered: 0,
            total_resolved: 0,
            total_acknowledged: 0,
            alerts_by_severity: HashMap::new(),
            mean_time_to_acknowledge_ms: 0.0,
            mean_time_to_resolve_ms: 0.0,
            false_positive_rate: 0.0,
        };

        assert_eq!(stats.total_triggered, 0);
        assert!(stats.alerts_by_severity.is_empty());
    }

    #[tokio::test]
    async fn test_alert_manager_with_custom_config() {
        let config = AlertManagerConfig {
            max_active_alerts: 10,
            max_history_size: 50,
            evaluation_interval: Duration::from_secs(1),
            default_cooldown: Duration::from_secs(30),
            enable_auto_resolve: false,
            silence_duplicate_alerts: false,
        };

        let manager = AlertManager::new(config);
        let exported = manager.export_config().await;

        assert_eq!(exported.config.max_active_alerts, 10);
        assert_eq!(exported.config.max_history_size, 50);
        assert!(!exported.config.enable_auto_resolve);
    }

    #[tokio::test]
    async fn test_add_multiple_rules() {
        let manager = AlertManager::new(AlertManagerConfig::default());

        for rule in default_tdg_alert_rules() {
            manager.add_rule(rule).await.unwrap();
        }

        let config = manager.export_config().await;
        assert_eq!(config.rules.len(), 4);
    }

    #[tokio::test]
    async fn test_get_alerts_by_all_severities() {
        let manager = AlertManager::new(AlertManagerConfig::default());

        for severity in [
            AlertSeverity::Info,
            AlertSeverity::Warning,
            AlertSeverity::Error,
            AlertSeverity::Critical,
        ] {
            let alerts = manager.get_alerts_by_severity(severity).await;
            assert!(alerts.is_empty());
        }
    }

    #[test]
    fn test_alert_configuration_with_custom_config() {
        let custom_config = AlertManagerConfig {
            max_active_alerts: 50,
            max_history_size: 100,
            evaluation_interval: Duration::from_secs(30),
            default_cooldown: Duration::from_secs(600),
            enable_auto_resolve: true,
            silence_duplicate_alerts: true,
        };

        let config = AlertConfiguration {
            rules: default_tdg_alert_rules(),
            config: custom_config,
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AlertConfiguration = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.config.max_active_alerts, 50);
        assert_eq!(deserialized.rules.len(), 4);
    }

    #[test]
    fn test_alert_rule_less_than_condition() {
        let rule = AlertRule {
            id: "low_threshold".to_string(),
            name: "Low Value Alert".to_string(),
            description: "Fires when below threshold".to_string(),
            metric: "cache_hit".to_string(),
            condition: AlertCondition::LessThan,
            threshold: 0.5,
            duration: Duration::from_secs(60),
            severity: AlertSeverity::Warning,
            enabled: true,
            notification_channels: vec![],
            cooldown_period: Duration::from_secs(300),
            metadata: HashMap::new(),
        };

        assert_eq!(rule.condition, AlertCondition::LessThan);
        assert_eq!(rule.threshold, 0.5);
    }

    #[test]
    fn test_alert_rule_equal_condition() {
        let rule = AlertRule {
            id: "exact_match".to_string(),
            name: "Exact Value Alert".to_string(),
            description: "Fires on exact match".to_string(),
            metric: "status_code".to_string(),
            condition: AlertCondition::Equal,
            threshold: 500.0,
            duration: Duration::from_secs(10),
            severity: AlertSeverity::Error,
            enabled: true,
            notification_channels: vec![],
            cooldown_period: Duration::from_secs(60),
            metadata: HashMap::new(),
        };

        assert_eq!(rule.condition, AlertCondition::Equal);
    }

    #[test]
    fn test_alert_rule_not_equal_condition() {
        let rule = AlertRule {
            id: "not_200".to_string(),
            name: "Non-200 Status".to_string(),
            description: "Fires on non-200".to_string(),
            metric: "http_status".to_string(),
            condition: AlertCondition::NotEqual,
            threshold: 200.0,
            duration: Duration::from_secs(5),
            severity: AlertSeverity::Warning,
            enabled: true,
            notification_channels: vec![],
            cooldown_period: Duration::from_secs(30),
            metadata: HashMap::new(),
        };

        assert_eq!(rule.condition, AlertCondition::NotEqual);
    }

    #[test]
    fn test_alert_rule_less_than_or_equal() {
        let rule = AlertRule {
            id: "low_or_zero".to_string(),
            name: "Zero or Low".to_string(),
            description: "Fires when at or below".to_string(),
            metric: "active_connections".to_string(),
            condition: AlertCondition::LessThanOrEqual,
            threshold: 1.0,
            duration: Duration::from_secs(120),
            severity: AlertSeverity::Critical,
            enabled: true,
            notification_channels: vec![],
            cooldown_period: Duration::from_secs(600),
            metadata: HashMap::new(),
        };

        assert_eq!(rule.condition, AlertCondition::LessThanOrEqual);
    }

    #[test]
    fn test_alert_message_formatting() {
        let alert = Alert {
            id: "fmt_test".to_string(),
            rule_id: "rule1".to_string(),
            rule_name: "Format Test".to_string(),
            severity: AlertSeverity::Warning,
            state: AlertState::Triggered,
            triggered_at: SystemTime::now(),
            resolved_at: None,
            metric_value: 95.123456,
            threshold_value: 90.0,
            message: format!("Value {:.2} exceeds threshold {:.2}", 95.123456, 90.0),
            context: HashMap::new(),
            notification_sent: false,
            acknowledgement: None,
        };

        assert!(alert.message.contains("95.12"));
        assert!(alert.message.contains("90.00"));
    }
