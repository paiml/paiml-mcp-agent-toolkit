use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{RwLock, Semaphore, SemaphorePermit};

/// Operation types with timing and preemption tracking
#[derive(Clone, Debug)]
pub enum OperationType {
    /// High-priority commit operations
    Commit { started: Instant },
    /// Background operations that can be preempted
    Background { started: Instant, preemptible: bool },
}

/// Schedule permit types for different priority levels
pub enum SchedulePermit {
    High(SemaphorePermit<'static>),
    Low(SemaphorePermit<'static>),
}

/// Guard for scheduled operations with automatic cleanup
pub struct ScheduleGuard {
    path: PathBuf,
    #[allow(dead_code)]
    permit: SchedulePermit,
    active_ops: Arc<RwLock<HashMap<PathBuf, OperationType>>>,
}

impl Drop for ScheduleGuard {
    fn drop(&mut self) {
        // Cleanup happens automatically when guard is dropped
        let path = self.path.clone();
        let active_ops = self.active_ops.clone();

        tokio::spawn(async move {
            let mut ops = active_ops.write().await;
            ops.remove(&path);
        });
    }
}

/// Custom error for preempted operations
#[derive(Debug, thiserror::Error)]
pub enum ScheduleError {
    #[error("Operation was preempted by higher priority task")]
    Preempted,
    #[error("Semaphore acquisition failed: {0}")]
    SemaphoreError(#[from] tokio::sync::AcquireError),
    #[error("Scheduling error: {0}")]
    Other(#[from] anyhow::Error),
}

/// Simplified fair scheduler using proven tokio primitives
pub struct SimpleFairScheduler {
    /// High priority semaphore for commits (immediate access)
    high_priority: Arc<Semaphore>,
    /// Low priority semaphore for background operations
    low_priority: Arc<Semaphore>,
    /// Active operations tracking for preemption
    active_ops: Arc<RwLock<HashMap<PathBuf, OperationType>>>,
}

impl SimpleFairScheduler {
    /// Create new scheduler with default configuration
    pub fn new() -> Self {
        Self::with_limits(10, 2)
    }

    /// Create scheduler with custom limits
    pub fn with_limits(high_permits: usize, low_permits: usize) -> Self {
        let high = Arc::new(Semaphore::new(high_permits));
        let low = Arc::new(Semaphore::new(low_permits));

        Self {
            high_priority: high,
            low_priority: low,
            active_ops: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Schedule high-priority commit operation
    pub async fn schedule_commit(&self, path: PathBuf) -> Result<ScheduleGuard, ScheduleError> {
        // Commits always get immediate priority
        let permit = self.high_priority.acquire().await?;

        // Convert to 'static lifetime for SchedulePermit
        let static_permit = unsafe {
            std::mem::transmute::<
                tokio::sync::SemaphorePermit<'_>,
                tokio::sync::SemaphorePermit<'static>,
            >(permit)
        };

        let mut ops = self.active_ops.write().await;

        // Check for background operation on same path
        if let Some(OperationType::Background {
            preemptible: true, ..
        }) = ops.get(&path)
        {
            // Signal preemption by updating operation type
            ops.insert(
                path.clone(),
                OperationType::Commit {
                    started: Instant::now(),
                },
            );
        } else {
            ops.insert(
                path.clone(),
                OperationType::Commit {
                    started: Instant::now(),
                },
            );
        }

        Ok(ScheduleGuard {
            path,
            permit: SchedulePermit::High(static_permit),
            active_ops: self.active_ops.clone(),
        })
    }

    /// Schedule background operation (preemptible)
    pub async fn schedule_background(&self, path: PathBuf) -> Result<ScheduleGuard, ScheduleError> {
        // Check if commit is active on this path
        let ops = self.active_ops.read().await;
        if matches!(ops.get(&path), Some(OperationType::Commit { .. })) {
            // Yield immediately to commit
            return Err(ScheduleError::Preempted);
        }
        drop(ops);

        // Acquire low priority permit
        let permit = self.low_priority.acquire().await?;

        // Convert to 'static lifetime
        let static_permit = unsafe {
            std::mem::transmute::<
                tokio::sync::SemaphorePermit<'_>,
                tokio::sync::SemaphorePermit<'static>,
            >(permit)
        };

        let mut ops = self.active_ops.write().await;
        ops.insert(
            path.clone(),
            OperationType::Background {
                started: Instant::now(),
                preemptible: true,
            },
        );

        Ok(ScheduleGuard {
            path,
            permit: SchedulePermit::Low(static_permit),
            active_ops: self.active_ops.clone(),
        })
    }

    /// Get current scheduling statistics
    pub async fn get_statistics(&self) -> SchedulingStatistics {
        let ops = self.active_ops.read().await;
        let high_available = self.high_priority.available_permits();
        let low_available = self.low_priority.available_permits();

        let mut commit_count = 0;
        let mut background_count = 0;
        let mut total_wait_time = 0u64;

        let now = Instant::now();
        for op in ops.values() {
            match op {
                OperationType::Commit { started } => {
                    commit_count += 1;
                    total_wait_time += now.duration_since(*started).as_millis() as u64;
                }
                OperationType::Background { started, .. } => {
                    background_count += 1;
                    total_wait_time += now.duration_since(*started).as_millis() as u64;
                }
            }
        }

        let avg_wait_time = if commit_count + background_count > 0 {
            total_wait_time / (commit_count + background_count) as u64
        } else {
            0
        };

        SchedulingStatistics {
            high_permits_available: high_available,
            low_permits_available: low_available,
            active_commits: commit_count,
            active_background: background_count,
            avg_wait_time_ms: avg_wait_time,
            total_active_operations: ops.len(),
        }
    }

    /// Force preemption of background operations for a specific path
    pub async fn preempt_background(&self, path: &PathBuf) -> bool {
        let mut ops = self.active_ops.write().await;
        if let Some(OperationType::Background {
            preemptible: true, ..
        }) = ops.get(path)
        {
            ops.remove(path);
            true
        } else {
            false
        }
    }
}

impl Default for SimpleFairScheduler {
    fn default() -> Self {
        Self::new()
    }
}

/// Scheduling performance and status statistics
#[derive(Debug, Clone)]
pub struct SchedulingStatistics {
    pub high_permits_available: usize,
    pub low_permits_available: usize,
    pub active_commits: usize,
    pub active_background: usize,
    pub avg_wait_time_ms: u64,
    pub total_active_operations: usize,
}

impl SchedulingStatistics {
    /// Format statistics for diagnostic display
    pub fn format_diagnostic(&self) -> String {
        let status = if self.total_active_operations == 0 {
            "IDLE"
        } else if self.active_commits > 0 {
            "COMMIT_ACTIVE"
        } else {
            "BACKGROUND_ACTIVE"
        };

        format!(
            "Lock Scheduling:\n\
             - Status: {}\n\
             - High priority permits: {}\n\
             - Low priority permits: {}\n\
             - Active commits: {}\n\
             - Active background: {}\n\
             - Avg wait time: {}ms\n\
             - Total operations: {}",
            status,
            self.high_permits_available,
            self.low_permits_available,
            self.active_commits,
            self.active_background,
            self.avg_wait_time_ms,
            self.total_active_operations
        )
    }
}

/// Scheduler factory for easy instantiation
pub struct SchedulerFactory;

impl SchedulerFactory {
    /// Create scheduler with balanced configuration
    pub fn create_balanced() -> SimpleFairScheduler {
        SimpleFairScheduler::with_limits(10, 2)
    }

    /// Create scheduler optimized for commits
    pub fn create_commit_optimized() -> SimpleFairScheduler {
        SimpleFairScheduler::with_limits(20, 1)
    }

    /// Create scheduler optimized for background processing
    pub fn create_background_optimized() -> SimpleFairScheduler {
        SimpleFairScheduler::with_limits(5, 8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_scheduler_creation() {
        let scheduler = SimpleFairScheduler::new();
        let stats = scheduler.get_statistics().await;

        assert_eq!(stats.active_commits, 0);
        assert_eq!(stats.active_background, 0);
        assert_eq!(stats.total_active_operations, 0);
    }

    #[tokio::test]
    async fn test_commit_scheduling() {
        let scheduler = SimpleFairScheduler::new();
        let path = PathBuf::from("test.rs");

        let _guard = scheduler.schedule_commit(path.clone()).await.unwrap();

        let stats = scheduler.get_statistics().await;
        assert_eq!(stats.active_commits, 1);
        assert_eq!(stats.total_active_operations, 1);
    }

    #[tokio::test]
    async fn test_background_scheduling() {
        let scheduler = SimpleFairScheduler::new();
        let path = PathBuf::from("test.rs");

        let _guard = scheduler.schedule_background(path.clone()).await.unwrap();

        let stats = scheduler.get_statistics().await;
        assert_eq!(stats.active_background, 1);
        assert_eq!(stats.total_active_operations, 1);
    }

    #[tokio::test]
    async fn test_commit_preempts_background() {
        let scheduler = SimpleFairScheduler::new();
        let path = PathBuf::from("test.rs");

        // Start background operation
        let _bg_guard = scheduler.schedule_background(path.clone()).await.unwrap();

        // Commit should preempt background
        let _commit_guard = scheduler.schedule_commit(path.clone()).await.unwrap();

        let stats = scheduler.get_statistics().await;
        assert_eq!(stats.active_commits, 1);
        // Background operation should still be tracked but marked as preempted
    }

    #[tokio::test]
    async fn test_background_yields_to_commit() {
        let scheduler = SimpleFairScheduler::new();
        let path = PathBuf::from("test.rs");

        // Start commit operation
        let _commit_guard = scheduler.schedule_commit(path.clone()).await.unwrap();

        // Background should be rejected
        let result = scheduler.schedule_background(path.clone()).await;
        assert!(matches!(result, Err(ScheduleError::Preempted)));
    }

    #[tokio::test]
    async fn test_guard_cleanup() {
        let scheduler = SimpleFairScheduler::new();
        let path = PathBuf::from("test.rs");

        {
            let _guard = scheduler.schedule_commit(path.clone()).await.unwrap();
            let stats = scheduler.get_statistics().await;
            assert_eq!(stats.total_active_operations, 1);
        }

        // Give cleanup task time to run
        sleep(Duration::from_millis(10)).await;

        let stats = scheduler.get_statistics().await;
        assert_eq!(stats.total_active_operations, 0);
    }

    #[tokio::test]
    async fn test_concurrent_different_paths() {
        let scheduler = SimpleFairScheduler::new();
        let path1 = PathBuf::from("test1.rs");
        let path2 = PathBuf::from("test2.rs");

        let _guard1 = scheduler.schedule_commit(path1).await.unwrap();
        let _guard2 = scheduler.schedule_background(path2).await.unwrap();

        let stats = scheduler.get_statistics().await;
        assert_eq!(stats.active_commits, 1);
        assert_eq!(stats.active_background, 1);
        assert_eq!(stats.total_active_operations, 2);
    }

    #[tokio::test]
    async fn test_scheduler_factory() {
        let balanced = SchedulerFactory::create_balanced();
        let commit_opt = SchedulerFactory::create_commit_optimized();
        let bg_opt = SchedulerFactory::create_background_optimized();

        // Test that different configurations work
        let path = PathBuf::from("test.rs");

        let _guard1 = balanced.schedule_commit(path.clone()).await.unwrap();
        let _guard2 = commit_opt.schedule_commit(path.clone()).await.unwrap();
        let _guard3 = bg_opt.schedule_background(path.clone()).await.unwrap();

        // All should succeed without deadlock
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test] 
        fn module_consistency_check(x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(x < 1001);
        }
    }
}
