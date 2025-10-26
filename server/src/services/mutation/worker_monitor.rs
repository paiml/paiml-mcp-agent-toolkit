//! Worker monitoring system for distributed mutation testing
//!
//! Provides a worker state tracking and monitoring system that ensures
//! distributed mutation testing is reliable, observable, and recoverable.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};

/// Current state of a worker
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkerState {
    /// Worker is idle and available for work
    Idle,
    
    /// Worker is currently processing a task
    Processing,
    
    /// Worker has failed
    Failed,
    
    /// Worker has been terminated
    Terminated,
}

/// Metrics for a single worker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerMetrics {
    /// Worker ID
    pub id: usize,
    
    /// Current state of the worker
    pub state: WorkerState,
    
    /// Number of tasks processed successfully
    pub processed_count: usize,
    
    /// Number of tasks that failed
    pub failed_count: usize,
    
    /// When the worker was last active (sent heartbeat)
    #[serde(skip, default = "Instant::now")]
    pub last_heartbeat: Instant,
    
    /// Most recent error messages (up to 5)
    pub recent_errors: Vec<String>,
    
    /// Average processing time (ms)
    pub avg_processing_time_ms: f64,
    
    /// Number of heartbeats received
    pub heartbeat_count: usize,
    
    /// Custom metrics specific to this worker
    pub custom_metrics: HashMap<String, String>,
}

impl WorkerMetrics {
    /// Create new worker metrics
    pub fn new(id: usize) -> Self {
        Self {
            id,
            state: WorkerState::Idle,
            processed_count: 0,
            failed_count: 0,
            last_heartbeat: Instant::now(),
            recent_errors: Vec::new(),
            avg_processing_time_ms: 0.0,
            heartbeat_count: 0,
            custom_metrics: HashMap::new(),
        }
    }
    
    /// Record a successful task completion
    pub fn record_success(&mut self, processing_time_ms: u64) {
        self.processed_count += 1;
        self.state = WorkerState::Idle;
        
        // Update average processing time
        self.avg_processing_time_ms = (self.avg_processing_time_ms * (self.processed_count as f64 - 1.0) 
            + processing_time_ms as f64) / self.processed_count as f64;
    }
    
    /// Record a task failure
    pub fn record_failure(&mut self, error: &str) {
        self.failed_count += 1;
        self.state = WorkerState::Idle;
        
        // Add error to recent errors (keep only most recent 5)
        self.recent_errors.push(error.to_string());
        if self.recent_errors.len() > 5 {
            self.recent_errors.remove(0);
        }
    }
    
    /// Update heartbeat timestamp
    pub fn update_heartbeat(&mut self) {
        self.last_heartbeat = Instant::now();
        self.heartbeat_count += 1;
    }
    
    /// Check if worker appears to be stalled
    pub fn is_stalled(&self, timeout: Duration) -> bool {
        if self.state == WorkerState::Processing {
            self.last_heartbeat.elapsed() > timeout
        } else {
            false
        }
    }
    
    /// Set worker state
    pub fn set_state(&mut self, state: WorkerState) {
        self.state = state;
        self.update_heartbeat();
    }
    
    /// Add or update a custom metric
    pub fn set_custom_metric(&mut self, key: &str, value: &str) {
        self.custom_metrics.insert(key.to_string(), value.to_string());
    }
    
    /// Get time since last heartbeat
    pub fn time_since_heartbeat(&self) -> Duration {
        self.last_heartbeat.elapsed()
    }
}

/// Monitor tracking all workers in the distributed system
pub struct WorkerMonitor {
    /// Collection of worker metrics
    workers: RwLock<HashMap<usize, WorkerMetrics>>,
    
    /// Timeout for considering a worker stalled
    stall_timeout: Duration,
    
    /// Total number of workers
    worker_count: usize,
}

impl WorkerMonitor {
    /// Create a new worker monitor
    pub fn new(worker_count: usize, stall_timeout: Duration) -> Self {
        Self {
            workers: RwLock::new(HashMap::new()),
            stall_timeout,
            worker_count,
        }
    }
    
    /// Initialize all workers
    pub async fn initialize_workers(&self) {
        let mut workers = self.workers.write().await;
        
        for id in 0..self.worker_count {
            workers.insert(id, WorkerMetrics::new(id));
        }
    }
    
    /// Record heartbeat from a worker
    pub async fn record_heartbeat(&self, worker_id: usize) {
        let mut workers = self.workers.write().await;
        
        if let Some(worker) = workers.get_mut(&worker_id) {
            worker.update_heartbeat();
        }
    }
    
    /// Record worker starting to process a task
    pub async fn record_start_processing(&self, worker_id: usize) {
        let mut workers = self.workers.write().await;
        
        if let Some(worker) = workers.get_mut(&worker_id) {
            worker.set_state(WorkerState::Processing);
        }
    }
    
    /// Record successful task completion
    pub async fn record_success(&self, worker_id: usize, processing_time_ms: u64) {
        let mut workers = self.workers.write().await;
        
        if let Some(worker) = workers.get_mut(&worker_id) {
            worker.record_success(processing_time_ms);
        }
    }
    
    /// Record task failure
    pub async fn record_failure(&self, worker_id: usize, error: &str) {
        let mut workers = self.workers.write().await;
        
        if let Some(worker) = workers.get_mut(&worker_id) {
            worker.record_failure(error);
        }
    }
    
    /// Mark worker as failed
    pub async fn mark_failed(&self, worker_id: usize, reason: &str) {
        let mut workers = self.workers.write().await;

        if let Some(worker) = workers.get_mut(&worker_id) {
            worker.record_failure(reason);
            worker.set_state(WorkerState::Failed);
        }
    }
    
    /// Mark worker as terminated
    pub async fn mark_terminated(&self, worker_id: usize) {
        let mut workers = self.workers.write().await;
        
        if let Some(worker) = workers.get_mut(&worker_id) {
            worker.set_state(WorkerState::Terminated);
        }
    }
    
    /// Get metrics for a specific worker
    pub async fn get_worker_metrics(&self, worker_id: usize) -> Option<WorkerMetrics> {
        let workers = self.workers.read().await;
        workers.get(&worker_id).cloned()
    }
    
    /// Get metrics for all workers
    pub async fn get_all_metrics(&self) -> Vec<WorkerMetrics> {
        let workers = self.workers.read().await;
        workers.values().cloned().collect()
    }
    
    /// Get IDs of stalled workers
    pub async fn get_stalled_workers(&self) -> Vec<usize> {
        let workers = self.workers.read().await;
        
        workers.values()
            .filter(|w| w.is_stalled(self.stall_timeout))
            .map(|w| w.id)
            .collect()
    }
    
    /// Get count of workers in each state
    pub async fn get_state_counts(&self) -> HashMap<WorkerState, usize> {
        let workers = self.workers.read().await;
        let mut counts = HashMap::new();
        
        for worker in workers.values() {
            *counts.entry(worker.state).or_insert(0) += 1;
        }
        
        counts
    }
    
    /// Calculate overall health score (0-100)
    pub async fn calculate_health_score(&self) -> f64 {
        let workers = self.workers.read().await;
        let total = workers.len();
        
        if total == 0 {
            return 0.0;
        }
        
        let healthy_count = workers.values()
            .filter(|w| !w.is_stalled(self.stall_timeout) && w.state != WorkerState::Failed)
            .count();
            
        (healthy_count as f64 / total as f64) * 100.0
    }
    
    /// Run monitoring task periodically
    pub async fn run_monitoring_task(
        monitor: Arc<Self>, 
        interval: Duration,
        on_stalled: impl Fn(usize) + Send + Sync + 'static,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut timer = tokio::time::interval(interval);
            
            loop {
                timer.tick().await;
                
                // Check for stalled workers
                let stalled = monitor.get_stalled_workers().await;
                for worker_id in stalled {
                    on_stalled(worker_id);
                }
                
                // Calculate and log health metrics
                let health_score = monitor.calculate_health_score().await;
                let state_counts = monitor.get_state_counts().await;
                
                let idle_count = *state_counts.get(&WorkerState::Idle).unwrap_or(&0);
                let processing_count = *state_counts.get(&WorkerState::Processing).unwrap_or(&0);
                let failed_count = *state_counts.get(&WorkerState::Failed).unwrap_or(&0);
                
                if health_score < 80.0 || failed_count > 0 {
                    eprintln!("⚠️ Worker health: {:.1}% (Idle: {}, Processing: {}, Failed: {})",
                        health_score, idle_count, processing_count, failed_count);
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;
    
    #[tokio::test]
    async fn test_worker_metrics_initialization() {
        let metrics = WorkerMetrics::new(1);
        
        assert_eq!(metrics.id, 1);
        assert_eq!(metrics.state, WorkerState::Idle);
        assert_eq!(metrics.processed_count, 0);
        assert_eq!(metrics.failed_count, 0);
    }
    
    #[tokio::test]
    async fn test_worker_metrics_record_success() {
        let mut metrics = WorkerMetrics::new(1);
        
        metrics.record_success(100);
        
        assert_eq!(metrics.processed_count, 1);
        assert_eq!(metrics.state, WorkerState::Idle);
        assert_eq!(metrics.avg_processing_time_ms, 100.0);
        
        metrics.record_success(200);
        
        assert_eq!(metrics.processed_count, 2);
        assert_eq!(metrics.avg_processing_time_ms, 150.0); // (100 + 200) / 2
    }
    
    #[tokio::test]
    async fn test_worker_metrics_record_failure() {
        let mut metrics = WorkerMetrics::new(1);
        
        metrics.record_failure("Test error");
        
        assert_eq!(metrics.failed_count, 1);
        assert_eq!(metrics.state, WorkerState::Idle);
        assert_eq!(metrics.recent_errors.len(), 1);
        assert_eq!(metrics.recent_errors[0], "Test error");
        
        // Add more errors and verify the limit is enforced
        for i in 0..10 {
            metrics.record_failure(&format!("Error {}", i));
        }
        
        assert_eq!(metrics.failed_count, 11);
        assert_eq!(metrics.recent_errors.len(), 5);
        assert_eq!(metrics.recent_errors[0], "Error 5");
    }
    
    #[tokio::test]
    async fn test_worker_is_stalled() {
        let mut metrics = WorkerMetrics::new(1);
        
        // Set to processing state
        metrics.set_state(WorkerState::Processing);
        
        // Not stalled yet
        assert!(!metrics.is_stalled(Duration::from_millis(100)));
        
        // Wait for timeout
        sleep(Duration::from_millis(20)).await;
        
        // Still not stalled
        assert!(!metrics.is_stalled(Duration::from_millis(100)));
        
        // Reduce timeout - now it's stalled
        assert!(metrics.is_stalled(Duration::from_millis(10)));
        
        // Change state - no longer stalled
        metrics.set_state(WorkerState::Idle);
        assert!(!metrics.is_stalled(Duration::from_millis(10)));
    }
    
    #[tokio::test]
    async fn test_worker_monitor_initialization() {
        let monitor = WorkerMonitor::new(5, Duration::from_secs(10));
        
        monitor.initialize_workers().await;
        
        let all_metrics = monitor.get_all_metrics().await;
        assert_eq!(all_metrics.len(), 5);
        
        for id in 0..5 {
            let metrics = monitor.get_worker_metrics(id).await;
            assert!(metrics.is_some());
            assert_eq!(metrics.unwrap().id, id);
        }
        
        let metrics = monitor.get_worker_metrics(10).await;
        assert!(metrics.is_none());
    }
    
    #[tokio::test]
    async fn test_worker_monitor_record_heartbeat() {
        let monitor = WorkerMonitor::new(2, Duration::from_secs(10));
        monitor.initialize_workers().await;
        
        // Record initial metrics
        let initial = monitor.get_worker_metrics(0).await.unwrap();
        let initial_heartbeat_count = initial.heartbeat_count;
        
        // Wait a moment
        sleep(Duration::from_millis(10)).await;
        
        // Record heartbeat
        monitor.record_heartbeat(0).await;
        
        // Get updated metrics
        let updated = monitor.get_worker_metrics(0).await.unwrap();
        
        assert_eq!(updated.heartbeat_count, initial_heartbeat_count + 1);
        assert!(updated.time_since_heartbeat() < initial.time_since_heartbeat());
    }
    
    #[tokio::test]
    async fn test_worker_monitor_state_changes() {
        let monitor = WorkerMonitor::new(1, Duration::from_secs(10));
        monitor.initialize_workers().await;
        
        // Initial state
        assert_eq!(monitor.get_worker_metrics(0).await.unwrap().state, WorkerState::Idle);
        
        // Start processing
        monitor.record_start_processing(0).await;
        assert_eq!(monitor.get_worker_metrics(0).await.unwrap().state, WorkerState::Processing);
        
        // Success
        monitor.record_success(0, 100).await;
        let metrics = monitor.get_worker_metrics(0).await.unwrap();
        assert_eq!(metrics.state, WorkerState::Idle);
        assert_eq!(metrics.processed_count, 1);
        
        // Start processing again
        monitor.record_start_processing(0).await;
        
        // Failure
        monitor.record_failure(0, "Test error").await;
        let metrics = monitor.get_worker_metrics(0).await.unwrap();
        assert_eq!(metrics.state, WorkerState::Idle);
        assert_eq!(metrics.failed_count, 1);
        
        // Mark failed
        monitor.mark_failed(0, "Fatal error").await;
        let metrics = monitor.get_worker_metrics(0).await.unwrap();
        assert_eq!(metrics.state, WorkerState::Failed);
        assert_eq!(metrics.failed_count, 2);
        
        // Mark terminated
        monitor.mark_terminated(0).await;
        assert_eq!(monitor.get_worker_metrics(0).await.unwrap().state, WorkerState::Terminated);
    }
    
    #[tokio::test]
    async fn test_worker_monitor_stalled_detection() {
        let monitor = WorkerMonitor::new(2, Duration::from_millis(50));
        monitor.initialize_workers().await;
        
        // Both workers idle - none stalled
        assert_eq!(monitor.get_stalled_workers().await.len(), 0);
        
        // Start processing on worker 0
        monitor.record_start_processing(0).await;
        
        // Not stalled yet
        assert_eq!(monitor.get_stalled_workers().await.len(), 0);
        
        // Wait for timeout
        sleep(Duration::from_millis(60)).await;
        
        // Now worker 0 is stalled
        let stalled = monitor.get_stalled_workers().await;
        assert_eq!(stalled.len(), 1);
        assert_eq!(stalled[0], 0);
    }
    
    #[tokio::test]
    async fn test_worker_monitor_health_score() {
        let monitor = WorkerMonitor::new(4, Duration::from_millis(50));
        monitor.initialize_workers().await;
        
        // All workers healthy
        assert_eq!(monitor.calculate_health_score().await, 100.0);
        
        // Mark one worker as failed
        monitor.mark_failed(0, "Failed").await;
        assert_eq!(monitor.calculate_health_score().await, 75.0);
        
        // Start processing on worker 1
        monitor.record_start_processing(1).await;
        
        // Still 75% healthy
        assert_eq!(monitor.calculate_health_score().await, 75.0);
        
        // Wait for worker 1 to stall
        sleep(Duration::from_millis(60)).await;
        
        // Now 50% healthy
        assert_eq!(monitor.calculate_health_score().await, 50.0);
    }
}