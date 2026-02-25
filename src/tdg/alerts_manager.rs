// Alert manager implementation methods
// Included from alerts.rs - shares parent module scope

impl AlertManager {
    #[must_use]
    pub fn new(config: AlertManagerConfig) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        Self {
            rules: Arc::new(RwLock::new(HashMap::new())),
            active_alerts: Arc::new(RwLock::new(HashMap::new())),
            alert_history: Arc::new(RwLock::new(VecDeque::new())),
            metric_values: Arc::new(RwLock::new(HashMap::new())),
            notification_tx: tx,
            notification_rx: Arc::new(RwLock::new(rx)),
            statistics: Arc::new(RwLock::new(AlertStatistics::default())),
            config,
        }
    }

    /// Add or update an alert rule
    pub async fn add_rule(&self, rule: AlertRule) -> Result<()> {
        let mut rules = self.rules.write().await;
        rules.insert(rule.id.clone(), rule);
        Ok(())
    }

    /// Remove an alert rule
    pub async fn remove_rule(&self, rule_id: &str) -> Result<()> {
        let mut rules = self.rules.write().await;
        rules.remove(rule_id);

        // Also resolve any active alerts for this rule
        let mut active = self.active_alerts.write().await;
        let alerts_to_resolve: Vec<String> = active
            .iter()
            .filter(|(_, alert)| alert.rule_id == rule_id)
            .map(|(id, _)| id.clone())
            .collect();

        for alert_id in alerts_to_resolve {
            if let Some(mut alert) = active.remove(&alert_id) {
                alert.state = AlertState::Resolved;
                alert.resolved_at = Some(SystemTime::now());
                self.add_to_history(alert).await;
            }
        }

        Ok(())
    }

    /// Update metric value for evaluation
    pub async fn update_metric(&self, metric_name: String, value: f64) -> Result<()> {
        let mut metrics = self.metric_values.write().await;
        metrics.insert(
            metric_name.clone(),
            MetricValue {
                value,
                timestamp: SystemTime::now(),
                tags: HashMap::new(),
            },
        );

        // Trigger evaluation for rules using this metric
        self.evaluate_rules_for_metric(&metric_name).await?;

        Ok(())
    }

    /// Evaluate all rules for a specific metric
    async fn evaluate_rules_for_metric(&self, metric_name: &str) -> Result<()> {
        let rules = self.rules.read().await;
        let metrics = self.metric_values.read().await;

        if let Some(metric_value) = metrics.get(metric_name) {
            for rule in rules
                .values()
                .filter(|r| r.metric == metric_name && r.enabled)
            {
                self.evaluate_rule(rule, metric_value).await?;
            }
        }

        Ok(())
    }

    /// Evaluate a single rule
    async fn evaluate_rule(&self, rule: &AlertRule, metric: &MetricValue) -> Result<()> {
        let should_trigger = match rule.condition {
            AlertCondition::GreaterThan => metric.value > rule.threshold,
            AlertCondition::LessThan => metric.value < rule.threshold,
            AlertCondition::Equal => (metric.value - rule.threshold).abs() < f64::EPSILON,
            AlertCondition::NotEqual => (metric.value - rule.threshold).abs() >= f64::EPSILON,
            AlertCondition::GreaterThanOrEqual => metric.value >= rule.threshold,
            AlertCondition::LessThanOrEqual => metric.value <= rule.threshold,
            AlertCondition::RateOfChange => {
                // Would need historical data to calculate rate
                false // Placeholder
            }
            AlertCondition::Anomaly => {
                // Would need statistical analysis
                false // Placeholder
            }
        };

        if should_trigger {
            self.trigger_alert(rule, metric.value).await?;
        } else if self.config.enable_auto_resolve {
            self.auto_resolve_alert(&rule.id).await?;
        }

        Ok(())
    }

    /// Trigger a new alert
    async fn trigger_alert(&self, rule: &AlertRule, metric_value: f64) -> Result<()> {
        let mut active = self.active_alerts.write().await;

        // Check if alert already exists
        let existing = active.values().any(|a| {
            a.rule_id == rule.id && matches!(a.state, AlertState::Triggered | AlertState::Active)
        });

        if existing && self.config.silence_duplicate_alerts {
            return Ok(());
        }

        // Check cooldown period
        if let Some(last_alert) = self.get_last_alert_for_rule(&rule.id).await {
            if let Some(resolved_at) = last_alert.resolved_at {
                let elapsed = SystemTime::now()
                    .duration_since(resolved_at)
                    .unwrap_or(Duration::ZERO);
                if elapsed < rule.cooldown_period {
                    return Ok(()); // Still in cooldown
                }
            }
        }

        let alert = Alert {
            id: format!(
                "alert_{}_{}",
                rule.id,
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .expect("internal error")
                    .as_millis()
            ),
            rule_id: rule.id.clone(),
            rule_name: rule.name.clone(),
            severity: rule.severity.clone(),
            state: AlertState::Triggered,
            triggered_at: SystemTime::now(),
            resolved_at: None,
            metric_value,
            threshold_value: rule.threshold,
            message: format!(
                "Alert: {} - {} {} {} (value: {:.2}, threshold: {:.2})",
                rule.name,
                rule.metric,
                format!("{:?}", rule.condition).to_lowercase(),
                rule.threshold,
                metric_value,
                rule.threshold
            ),
            context: HashMap::new(),
            notification_sent: false,
            acknowledgement: None,
        };

        // Add to active alerts
        active.insert(alert.id.clone(), alert.clone());

        // Send notification
        let _ = self.notification_tx.send(alert.clone());

        // Update statistics
        let mut stats = self.statistics.write().await;
        stats.total_triggered += 1;
        *stats
            .alerts_by_severity
            .entry(rule.severity.clone())
            .or_insert(0) += 1;

        Ok(())
    }

    /// Auto-resolve alerts when condition is no longer met
    async fn auto_resolve_alert(&self, rule_id: &str) -> Result<()> {
        let mut active = self.active_alerts.write().await;

        let alerts_to_resolve: Vec<String> = active
            .iter()
            .filter(|(_, alert)| {
                alert.rule_id == rule_id
                    && matches!(alert.state, AlertState::Triggered | AlertState::Active)
            })
            .map(|(id, _)| id.clone())
            .collect();

        for alert_id in alerts_to_resolve {
            if let Some(mut alert) = active.remove(&alert_id) {
                alert.state = AlertState::Resolved;
                alert.resolved_at = Some(SystemTime::now());

                // Update statistics
                let mut stats = self.statistics.write().await;
                stats.total_resolved += 1;

                if let (Some(resolved), triggered) = (alert.resolved_at, alert.triggered_at) {
                    let duration = resolved
                        .duration_since(triggered)
                        .unwrap_or(Duration::ZERO)
                        .as_millis() as f64;

                    // Update mean time to resolve (simple moving average)
                    stats.mean_time_to_resolve_ms = (stats.mean_time_to_resolve_ms
                        * (stats.total_resolved - 1) as f64
                        + duration)
                        / stats.total_resolved as f64;
                }

                self.add_to_history(alert).await;
            }
        }

        Ok(())
    }

    /// Acknowledge an alert
    pub async fn acknowledge_alert(
        &self,
        alert_id: &str,
        acknowledged_by: String,
        comment: Option<String>,
    ) -> Result<()> {
        let mut active = self.active_alerts.write().await;

        if let Some(alert) = active.get_mut(alert_id) {
            alert.state = AlertState::Acknowledged;
            alert.acknowledgement = Some(Acknowledgement {
                acknowledged_by,
                acknowledged_at: SystemTime::now(),
                comment,
            });

            // Update statistics
            let mut stats = self.statistics.write().await;
            stats.total_acknowledged += 1;

            if let Some(ack) = &alert.acknowledgement {
                let duration = ack
                    .acknowledged_at
                    .duration_since(alert.triggered_at)
                    .unwrap_or(Duration::ZERO)
                    .as_millis() as f64;

                // Update mean time to acknowledge
                stats.mean_time_to_acknowledge_ms = (stats.mean_time_to_acknowledge_ms
                    * (stats.total_acknowledged - 1) as f64
                    + duration)
                    / stats.total_acknowledged as f64;
            }
        }

        Ok(())
    }

    /// Silence an alert
    pub async fn silence_alert(&self, alert_id: &str, _duration: Duration) -> Result<()> {
        let mut active = self.active_alerts.write().await;

        if let Some(alert) = active.get_mut(alert_id) {
            alert.state = AlertState::Silenced;
            // Would implement silence expiration logic here
        }

        Ok(())
    }

    /// Get last alert for a rule
    async fn get_last_alert_for_rule(&self, rule_id: &str) -> Option<Alert> {
        let history = self.alert_history.read().await;
        history.iter().rev().find(|a| a.rule_id == rule_id).cloned()
    }

    /// Add alert to history
    async fn add_to_history(&self, alert: Alert) {
        let mut history = self.alert_history.write().await;
        history.push_back(alert);

        // Enforce history size limit
        while history.len() > self.config.max_history_size {
            history.pop_front();
        }
    }

    /// Get active alerts
    pub async fn get_active_alerts(&self) -> Vec<Alert> {
        let active = self.active_alerts.read().await;
        active.values().cloned().collect()
    }

    /// Get alerts by severity
    pub async fn get_alerts_by_severity(&self, severity: AlertSeverity) -> Vec<Alert> {
        let active = self.active_alerts.read().await;
        active
            .values()
            .filter(|a| a.severity == severity)
            .cloned()
            .collect()
    }

    /// Get alert statistics
    pub async fn get_statistics(&self) -> AlertStatistics {
        self.statistics.read().await.clone()
    }

    /// Export alert configuration
    pub async fn export_config(&self) -> AlertConfiguration {
        let rules = self.rules.read().await;

        AlertConfiguration {
            rules: rules.values().cloned().collect(),
            config: self.config.clone(),
        }
    }
}
