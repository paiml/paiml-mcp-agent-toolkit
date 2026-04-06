// Adaptive snapshot scheduler that learns optimal snapshot intervals

pub struct AdaptiveSnapshotScheduler {
    pub(super) config: parking_lot::RwLock<SnapshotSchedulerConfig>,
    pub(super) metrics: parking_lot::RwLock<SnapshotMetrics>,
}

impl AdaptiveSnapshotScheduler {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(config: SnapshotSchedulerConfig) -> Self {
        Self {
            config: parking_lot::RwLock::new(config),
            metrics: parking_lot::RwLock::new(SnapshotMetrics::default()),
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn should_snapshot(&self, events_since: usize, time_since: Duration) -> bool {
        let config = self.config.read();

        // Check absolute thresholds
        if events_since >= config.max_events {
            return true;
        }

        if time_since >= config.max_time_between_snapshots {
            return true;
        }

        if events_since < config.min_events {
            return false;
        }

        if time_since < config.min_time_between_snapshots {
            return false;
        }

        // Adaptive decision based on recovery time
        if config.adaptive_enabled {
            self.adaptive_decision(events_since, time_since)
        } else {
            // Simple threshold-based decision
            events_since >= config.min_events * 10
                || time_since >= config.min_time_between_snapshots * 10
        }
    }

    fn adaptive_decision(&self, events_since: usize, time_since: Duration) -> bool {
        let metrics = self.metrics.read();

        if metrics.recovery_times.is_empty() {
            // No history, use conservative approach
            return events_since >= 10_000 || time_since >= Duration::from_secs(600);
        }

        // Estimate recovery time based on historical data
        let avg_recovery_time =
            metrics.recovery_times.iter().sum::<Duration>() / metrics.recovery_times.len() as u32;

        let config = self.config.read();

        // If recovery is taking too long, snapshot more frequently
        if avg_recovery_time > config.recovery_time_target {
            events_since >= config.min_events || time_since >= config.min_time_between_snapshots
        } else {
            // Recovery is fast, can wait longer between snapshots
            events_since >= config.min_events * 2
                || time_since >= config.min_time_between_snapshots * 2
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn record_snapshot(&self, events_between: usize, time_between: Duration) {
        let mut metrics = self.metrics.write();
        metrics.total_snapshots += 1;
        metrics.total_events_between_snapshots += events_between as u64;
        metrics.total_time_between_snapshots += time_between;
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn record_recovery(&self, recovery_time: Duration) {
        let mut metrics = self.metrics.write();
        metrics.last_recovery_time = Some(recovery_time);
        metrics.recovery_times.push(recovery_time);

        // Keep only last 10 recovery times
        if metrics.recovery_times.len() > 10 {
            metrics.recovery_times.remove(0);
        }

        // Adapt configuration if needed
        if self.config.read().adaptive_enabled {
            self.adapt_configuration(recovery_time);
        }
    }

    fn adapt_configuration(&self, recovery_time: Duration) {
        let mut config = self.config.write();

        if recovery_time > config.recovery_time_target * 2 {
            // Recovery exceeds target, snapshot more frequently
            config.min_events = (config.min_events / 2).max(100);
            config.min_time_between_snapshots =
                (config.min_time_between_snapshots / 2).max(Duration::from_secs(30));
        } else if recovery_time < config.recovery_time_target / 2 {
            // Recovery very fast, can reduce snapshot frequency
            config.min_events = (config.min_events * 3 / 2).min(50_000);
            config.min_time_between_snapshots =
                (config.min_time_between_snapshots * 3 / 2).min(Duration::from_secs(1800));
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_config(&self) -> SnapshotSchedulerConfig {
        self.config.read().clone()
    }

    pub fn get_metrics(&self) -> SnapshotSchedulerMetrics {
        let metrics = self.metrics.read();
        let config = self.config.read();

        SnapshotSchedulerMetrics {
            total_snapshots: metrics.total_snapshots,
            avg_events_between: if metrics.total_snapshots > 0 {
                (metrics.total_events_between_snapshots / metrics.total_snapshots) as usize
            } else {
                0
            },
            avg_time_between: if metrics.total_snapshots > 0 {
                metrics.total_time_between_snapshots / metrics.total_snapshots as u32
            } else {
                Duration::ZERO
            },
            last_recovery_time: metrics.last_recovery_time,
            current_thresholds: (config.min_events, config.min_time_between_snapshots),
        }
    }
}
