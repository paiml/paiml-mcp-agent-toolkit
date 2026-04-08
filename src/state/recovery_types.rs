// Types, structs, and error definitions for state recovery

#[derive(Clone)]
/// Configuration for snapshot scheduler.
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
pub(super) struct SnapshotMetrics {
    pub total_snapshots: u64,
    pub total_events_between_snapshots: u64,
    pub total_time_between_snapshots: Duration,
    pub last_recovery_time: Option<Duration>,
    pub recovery_times: Vec<Duration>,
}

#[derive(Debug, Clone)]
/// Snapshot scheduler metrics.
pub struct SnapshotSchedulerMetrics {
    pub total_snapshots: u64,
    pub avg_events_between: usize,
    pub avg_time_between: Duration,
    pub last_recovery_time: Option<Duration>,
    pub current_thresholds: (usize, Duration),
}

#[derive(Debug, Clone)]
/// Statistics for recovery.
pub struct RecoveryStats {
    pub total_events: usize,
    pub total_snapshots: usize,
    pub compression_ratio: f64,
    pub next_snapshot_in: Duration,
}

#[derive(Debug, thiserror::Error)]
/// Error variants for recovery operations.
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
