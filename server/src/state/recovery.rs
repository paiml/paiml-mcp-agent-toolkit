use super::*;
use crate::state::event_store::{EventStore, EventStoreConfig};
use crate::state::snapshot_store::{SnapshotStore, SnapshotConfig};
use std::sync::Arc;
use std::time::{Duration, Instant};

// State recovery orchestrator with adaptive snapshot scheduling
pub struct RecoveryManager<S: AgentState> {
    event_store: Arc<EventStore>,
    snapshot_store: Arc<SnapshotStore>,
    snapshot_scheduler: Arc<AdaptiveSnapshotScheduler>,
    _phantom: std::marker::PhantomData<S>,
}

impl<S: AgentState> RecoveryManager<S> {
    pub async fn new(
        event_store_config: EventStoreConfig,
        snapshot_config: SnapshotConfig,
        snapshot_path: &str,
    ) -> Result<Self, RecoveryError> {
        let event_store = Arc::new(
            EventStore::new(event_store_config).await
                .map_err(|e| RecoveryError::EventStoreError(e.to_string()))?
        );
        
        let snapshot_store = Arc::new(
            SnapshotStore::new(snapshot_path, snapshot_config).await
                .map_err(|e| RecoveryError::SnapshotError(e.to_string()))?
        );

        let snapshot_scheduler = Arc::new(
            AdaptiveSnapshotScheduler::new(SnapshotSchedulerConfig::default())
        );

        Ok(Self {
            event_store,
            snapshot_store,
            snapshot_scheduler,
            _phantom: std::marker::PhantomData,
        })
    }

    pub async fn recover_state(
        &self,
        initial_state: S,
        partition_key: Option<String>,
    ) -> Result<RestoredState<S>, RecoveryError> {
        let start_time = Instant::now();

        // Find latest snapshot
        let (mut state, starting_event_id) = if let Some(ref pk) = partition_key {
            // Partition-specific recovery
            if let Some(snapshot) = self.snapshot_store
                .find_partition_snapshots(pk)
                .into_iter()
                .max_by_key(|s| s.event_id) {
                
                let restored = self.snapshot_store
                    .load_snapshot::<S>(&snapshot.id).await
                    .map_err(|e| RecoveryError::SnapshotError(e.to_string()))?;
                (restored, snapshot.event_id)
            } else {
                (initial_state, 0)
            }
        } else {
            // Global recovery
            if let Some(snapshot) = self.snapshot_store.find_latest_snapshot() {
                let restored = self.snapshot_store
                    .load_snapshot::<S>(&snapshot.id).await
                    .map_err(|e| RecoveryError::SnapshotError(e.to_string()))?;
                (restored, snapshot.event_id)
            } else {
                (initial_state, 0)
            }
        };

        // Replay events since snapshot
        let events = if let Some(pk) = partition_key {
            self.event_store.get_partition_events(&pk, Some(starting_event_id))
        } else {
            self.event_store.get_events_since(starting_event_id, None)
        };

        let events_replayed = events.len();
        for event in &events {
            state.apply_event(event);
        }

        let recovery_time = start_time.elapsed();

        Ok(RestoredState {
            state,
            snapshot_id: Uuid::new_v4(), // Would get from actual snapshot
            events_to_replay: events_replayed,
        })
    }

    pub async fn save_snapshot(
        &self,
        state: &S,
        partition_key: Option<String>,
    ) -> Result<SnapshotId, RecoveryError> {
        let event_id = state.last_event_id();
        
        let snapshot_id = self.snapshot_store
            .save_snapshot(state, event_id, partition_key.clone())
            .await
            .map_err(|e| RecoveryError::SnapshotError(e.to_string()))?;

        // Update scheduler metrics
        self.snapshot_scheduler.record_snapshot(
            state.events_since_snapshot(),
            state.time_since_snapshot(),
        );

        Ok(snapshot_id)
    }

    pub async fn should_snapshot(&self, state: &S) -> bool {
        self.snapshot_scheduler.should_snapshot(
            state.events_since_snapshot(),
            state.time_since_snapshot(),
        )
    }

    pub async fn compact_events(&self) -> Result<(), RecoveryError> {
        self.event_store.compact().await
            .map_err(|e| RecoveryError::EventStoreError(e.to_string()))?;
        Ok(())
    }

    pub async fn cleanup_old_snapshots(&self) -> Result<usize, RecoveryError> {
        self.snapshot_store.cleanup_orphaned_files().await
            .map_err(|e| RecoveryError::SnapshotError(e.to_string()))
    }

    pub fn get_recovery_stats(&self) -> RecoveryStats {
        let event_stats = self.event_store.get_statistics();
        let snapshot_stats = self.snapshot_store.get_statistics();
        let scheduler_config = self.snapshot_scheduler.get_config();

        RecoveryStats {
            total_events: event_stats.total_events,
            total_snapshots: snapshot_stats.total_snapshots,
            compression_ratio: snapshot_stats.compression_ratio,
            next_snapshot_in: scheduler_config.min_time_between_snapshots,
        }
    }
}

// Adaptive snapshot scheduler that learns optimal snapshot intervals
pub struct AdaptiveSnapshotScheduler {
    config: parking_lot::RwLock<SnapshotSchedulerConfig>,
    metrics: parking_lot::RwLock<SnapshotMetrics>,
}

#[derive(Clone)]
pub struct SnapshotSchedulerConfig {
    pub min_events: usize,
    pub max_events: usize,
    pub min_time_between_snapshots: Duration,
    pub max_time_between_snapshots: Duration,
    pub recovery_time_target: Duration,
    pub adaptive_enabled: bool,
}

impl Default for SnapshotSchedulerConfig {
    fn default() -> Self {
        Self {
            min_events: 1000,
            max_events: 100_000,
            min_time_between_snapshots: Duration::from_secs(60),
            max_time_between_snapshots: Duration::from_secs(3600),
            recovery_time_target: Duration::from_secs(5),
            adaptive_enabled: true,
        }
    }
}

#[derive(Default)]
struct SnapshotMetrics {
    total_snapshots: u64,
    total_events_between_snapshots: u64,
    total_time_between_snapshots: Duration,
    last_recovery_time: Option<Duration>,
    recovery_times: Vec<Duration>,
}

impl AdaptiveSnapshotScheduler {
    pub fn new(config: SnapshotSchedulerConfig) -> Self {
        Self {
            config: parking_lot::RwLock::new(config),
            metrics: parking_lot::RwLock::new(SnapshotMetrics::default()),
        }
    }

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
            events_since >= config.min_events * 10 ||
            time_since >= config.min_time_between_snapshots * 10
        }
    }

    fn adaptive_decision(&self, events_since: usize, time_since: Duration) -> bool {
        let metrics = self.metrics.read();
        
        if metrics.recovery_times.is_empty() {
            // No history, use conservative approach
            return events_since >= 10_000 || time_since >= Duration::from_secs(600);
        }

        // Estimate recovery time based on historical data
        let avg_recovery_time = metrics.recovery_times.iter()
            .sum::<Duration>() / metrics.recovery_times.len() as u32;
        
        let config = self.config.read();
        
        // If recovery is taking too long, snapshot more frequently
        if avg_recovery_time > config.recovery_time_target {
            events_since >= config.min_events || 
            time_since >= config.min_time_between_snapshots
        } else {
            // Recovery is fast, can wait longer between snapshots
            events_since >= config.min_events * 2 ||
            time_since >= config.min_time_between_snapshots * 2
        }
    }

    pub fn record_snapshot(&self, events_between: usize, time_between: Duration) {
        let mut metrics = self.metrics.write();
        metrics.total_snapshots += 1;
        metrics.total_events_between_snapshots += events_between as u64;
        metrics.total_time_between_snapshots += time_between;
    }

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
            // Recovery too slow, snapshot more frequently
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

#[derive(Debug, Clone)]
pub struct SnapshotSchedulerMetrics {
    pub total_snapshots: u64,
    pub avg_events_between: usize,
    pub avg_time_between: Duration,
    pub last_recovery_time: Option<Duration>,
    pub current_thresholds: (usize, Duration),
}

#[derive(Debug, Clone)]
pub struct RecoveryStats {
    pub total_events: usize,
    pub total_snapshots: usize,
    pub compression_ratio: f64,
    pub next_snapshot_in: Duration,
}

#[derive(Debug, thiserror::Error)]
pub enum RecoveryError {
    #[error("Event store error: {0}")]
    EventStoreError(String),
    #[error("Snapshot error: {0}")]
    SnapshotError(String),
    #[error("State corruption detected: {0}")]
    StateCorruption(String),
    #[error("Recovery failed: {0}")]
    RecoveryFailed(String),
}

// Parallel recovery for partitioned state
pub struct ParallelRecovery<S: AgentState> {
    managers: Vec<Arc<RecoveryManager<S>>>,
}

impl<S: AgentState> ParallelRecovery<S> {
    pub async fn new(
        num_partitions: usize,
        event_config: EventStoreConfig,
        snapshot_config: SnapshotConfig,
        base_path: &str,
    ) -> Result<Self, RecoveryError> {
        let mut managers = Vec::with_capacity(num_partitions);
        
        for i in 0..num_partitions {
            let snapshot_path = format!("{}/partition_{}", base_path, i);
            let manager = Arc::new(
                RecoveryManager::new(
                    event_config.clone(),
                    snapshot_config.clone(),
                    &snapshot_path,
                ).await?
            );
            managers.push(manager);
        }

        Ok(Self { managers })
    }

    pub async fn recover_all_partitions(
        &self,
        initial_state_factory: impl Fn() -> S + Send + Sync,
    ) -> Result<Vec<RestoredState<S>>, RecoveryError> {
        use futures::future::try_join_all;
        
        let futures: Vec<_> = self.managers.iter()
            .enumerate()
            .map(|(i, manager)| {
                let initial = initial_state_factory();
                let partition_key = format!("partition_{}", i);
                async move {
                    manager.recover_state(initial, Some(partition_key)).await
                }
            })
            .collect();

        try_join_all(futures).await
    }

    pub async fn merge_partitions(
        &self,
        states: Vec<RestoredState<S>>,
    ) -> Result<S, RecoveryError> {
        if states.is_empty() {
            return Err(RecoveryError::RecoveryFailed(
                "No partitions to merge".to_string()
            ));
        }

        let mut merged = states.into_iter().next().unwrap().state;
        
        for restored in states {
            merged.merge_partition(restored.state);
        }

        Ok(merged)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[actix_rt::test]
    async fn test_recovery_manager() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_str().unwrap();
        
        let event_config = EventStoreConfig {
            persistence_enabled: false,
            ..Default::default()
        };
        
        let manager = RecoveryManager::<ExampleState>::new(
            event_config,
            SnapshotConfig::default(),
            path,
        ).await.unwrap();
        
        let initial = ExampleState::default();
        let restored = manager.recover_state(initial, None).await.unwrap();
        
        assert_eq!(restored.events_to_replay, 0);
    }

    #[test]
    fn test_adaptive_scheduler() {
        let scheduler = AdaptiveSnapshotScheduler::new(
            SnapshotSchedulerConfig::default()
        );
        
        // Should not snapshot with few events
        assert!(!scheduler.should_snapshot(100, Duration::from_secs(10)));
        
        // Should snapshot with many events
        assert!(scheduler.should_snapshot(100_001, Duration::from_secs(10)));
        
        // Should snapshot after long time
        assert!(scheduler.should_snapshot(100, Duration::from_secs(3601)));
    }

    #[test]
    fn test_scheduler_adaptation() {
        let config = SnapshotSchedulerConfig {
            adaptive_enabled: true,
            recovery_time_target: Duration::from_secs(5),
            ..Default::default()
        };
        
        let scheduler = AdaptiveSnapshotScheduler::new(config);
        
        // Record slow recovery
        scheduler.record_recovery(Duration::from_secs(15));
        
        let new_config = scheduler.get_config();
        assert!(new_config.min_events < 1000); // Should reduce threshold
        
        // Record fast recovery
        scheduler.record_recovery(Duration::from_secs(1));
        
        let new_config = scheduler.get_config();
        assert!(new_config.min_events > 500); // Should increase threshold
    }

    #[actix_rt::test]
    async fn test_parallel_recovery() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_str().unwrap();
        
        let event_config = EventStoreConfig {
            persistence_enabled: false,
            ..Default::default()
        };
        
        let parallel = ParallelRecovery::<ExampleState>::new(
            4,
            event_config,
            SnapshotConfig::default(),
            path,
        ).await.unwrap();
        
        let states = parallel.recover_all_partitions(
            || ExampleState::default()
        ).await.unwrap();
        
        assert_eq!(states.len(), 4);
        
        let merged = parallel.merge_partitions(states).await.unwrap();
        assert_eq!(merged.last_event_id, 0);
    }
}