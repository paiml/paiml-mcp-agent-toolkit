//! Distributed mutation testing execution
//!
//! Provides parallel mutant execution with work queue distribution,
//! progress tracking, and result aggregation for production-scale
//! mutation testing workloads.

use super::types::*;
use super::language::LanguageAdapter;
use anyhow::Result;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::{mpsc, Semaphore};
use parking_lot::RwLock;

/// Distributed mutation executor configuration
#[derive(Debug, Clone)]
pub struct DistributedConfig {
    /// Number of parallel workers
    pub worker_count: usize,

    /// Maximum concurrent executions
    pub max_concurrent: usize,

    /// Work queue buffer size
    pub queue_size: usize,

    /// Enable progress tracking
    pub track_progress: bool,
}

impl Default for DistributedConfig {
    fn default() -> Self {
        let cpus = num_cpus::get();
        Self {
            worker_count: cpus,
            max_concurrent: cpus * 2,
            queue_size: 1000,
            track_progress: true,
        }
    }
}

/// Progress tracking for mutation execution
#[derive(Debug, Clone)]
pub struct MutationProgress {
    /// Total mutants to execute
    pub total: usize,

    /// Mutants completed
    pub completed: usize,

    /// Mutants currently executing
    pub in_progress: usize,

    /// Killed mutants
    pub killed: usize,

    /// Survived mutants
    pub survived: usize,

    /// Failed/errored mutants
    pub failed: usize,
}

impl MutationProgress {
    fn new(total: usize) -> Self {
        Self {
            total,
            completed: 0,
            in_progress: 0,
            killed: 0,
            survived: 0,
            failed: 0,
        }
    }

    /// Calculate completion percentage
    pub fn percentage(&self) -> f64 {
        if self.total == 0 {
            return 100.0;
        }
        (self.completed as f64 / self.total as f64) * 100.0
    }

    /// Calculate mutation score (killed / total non-equivalent)
    pub fn mutation_score(&self) -> f64 {
        let total_tested = self.killed + self.survived;
        if total_tested == 0 {
            return 0.0;
        }
        (self.killed as f64 / total_tested as f64) * 100.0
    }
}

/// Distributed mutation executor
pub struct DistributedExecutor {
    /// Language adapter for test execution
    adapter: Arc<dyn LanguageAdapter>,
    
    /// Configuration for distributed execution
    config: DistributedConfig,
    
    /// Progress tracking for mutation execution
    progress: Arc<RwLock<MutationProgress>>,
    
    /// Worker monitoring system
    worker_monitor: Option<Arc<super::worker_monitor::WorkerMonitor>>,
}

impl DistributedExecutor {
    /// Create new distributed executor
    pub fn new(adapter: Arc<dyn LanguageAdapter>, config: DistributedConfig) -> Self {
        // Create worker monitor if progress tracking is enabled
        let worker_monitor = if config.track_progress {
            let stall_timeout = std::time::Duration::from_secs(60); // 1 minute stall timeout
            Some(Arc::new(super::worker_monitor::WorkerMonitor::new(
                config.worker_count,
                stall_timeout,
            )))
        } else {
            None
        };
        
        Self {
            adapter,
            config,
            progress: Arc::new(RwLock::new(MutationProgress::new(0))),
            worker_monitor,
        }
    }
    
    /// Create distributed executor with custom worker monitor
    pub fn with_worker_monitor(
        mut self,
        monitor: Arc<super::worker_monitor::WorkerMonitor>,
    ) -> Self {
        self.worker_monitor = Some(monitor);
        self
    }

    /// Execute mutants in parallel across worker pool
    pub async fn execute_parallel(&self, mutants: Vec<Mutant>) -> Result<Vec<MutationResult>> {
        // Initialize progress
        {
            let mut progress = self.progress.write();
            *progress = MutationProgress::new(mutants.len());
        }

        // Initialize worker monitoring if enabled
        if let Some(ref monitor) = self.worker_monitor {
            monitor.initialize_workers().await;
            
            // Start monitoring task in background
            let monitor_clone = monitor.clone();
            let monitoring_interval = std::time::Duration::from_secs(10);
            
            let _monitoring_task = super::worker_monitor::WorkerMonitor::run_monitoring_task(
                monitor_clone,
                monitoring_interval,
                |worker_id| {
                    // Handler for stalled workers
                    eprintln!("⚠️ Worker {} appears to be stalled, execution may be slow", worker_id);
                },
            ).await;
        }

        // Create channels for work distribution
        let (work_tx, work_rx) = mpsc::channel::<Mutant>(self.config.queue_size);
        let (result_tx, mut result_rx) = mpsc::channel::<MutationResult>(self.config.queue_size);

        // Semaphore for concurrent execution limit
        let semaphore = Arc::new(Semaphore::new(self.config.max_concurrent));

        // Atomic counter for tracking completion
        let completed_count = Arc::new(AtomicUsize::new(0));

        // Spawn worker pool
        let workers = self.spawn_workers(
            work_rx,
            result_tx.clone(), // Clone for error reporting
            semaphore.clone(),
            completed_count.clone(),
        );

        // Set up signal handler for graceful shutdown
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        
        #[cfg(unix)]
        {
            use tokio::signal::unix::{signal, SignalKind};
            let shutdown_tx_clone = shutdown_tx.clone();
            
            // Handle SIGINT (Ctrl+C)
            tokio::spawn(async move {
                let mut sigint = signal(SignalKind::interrupt())
                    .expect("Failed to set up SIGINT handler");
                
                sigint.recv().await;
                eprintln!("\n🛑 Received interrupt signal, stopping gracefully...");
                let _ = shutdown_tx_clone.send(()).await;
            });
        }

        // Send all mutants to work queue
        let sender_task = tokio::spawn(async move {
            for mutant in mutants {
                if work_tx.send(mutant).await.is_err() {
                    break;
                }
            }
            drop(work_tx); // Signal completion
        });

        // Collect results
        let total = {
            self.progress.read().total
        };

        let mut results = Vec::with_capacity(total);
        let mut shutdown_requested = false;
        
        'result_loop: while let Some(result) = result_rx.recv().await {
            // Check for shutdown signal
            if !shutdown_requested {
                if let Ok(()) = shutdown_rx.try_recv() {
                    shutdown_requested = true;
                    eprintln!("🛑 Graceful shutdown in progress, waiting for current tasks...");
                }
            }
            
            // Update worker metrics if monitoring is enabled
            if let Some(ref monitor) = self.worker_monitor {
                // Extract worker ID from result
                // Convert string ID to numeric ID for worker assignment
                let numeric_id = result.mutant.id.parse::<usize>().unwrap_or_else(|_| {
                    // If parsing fails, hash the string to get a numeric value
                    use std::collections::hash_map::DefaultHasher;
                    use std::hash::{Hash, Hasher};
                    
                    let mut hasher = DefaultHasher::new();
                    result.mutant.id.hash(&mut hasher);
                    hasher.finish() as usize
                });
                let worker_id = numeric_id % self.config.worker_count;
                
                if result.status == MutantStatus::CompileError || result.status == MutantStatus::Timeout {
                    let error_msg = result.error_message.as_deref().unwrap_or("Unknown error");
                    monitor.record_failure(worker_id, error_msg).await;
                } else {
                    monitor.record_success(worker_id, result.execution_time_ms).await;
                }
            }
            
            // Update progress
            {
                let mut progress = self.progress.write();
                progress.completed += 1;

                match result.status {
                    MutantStatus::Killed => progress.killed += 1,
                    MutantStatus::Survived => progress.survived += 1,
                    MutantStatus::CompileError | MutantStatus::Timeout => progress.failed += 1,
                    _ => {}
                }
            }

            results.push(result);

            // Check if complete or shutdown requested
            if results.len() >= total || shutdown_requested {
                break 'result_loop;
            }
        }

        if shutdown_requested {
            eprintln!("🛑 Graceful shutdown completed. Processed {} of {} mutants.",
                results.len(), total);
        }

        // Wait for all tasks to complete
        sender_task.await?;

        // Wait for all workers to terminate
        for worker in workers {
            let _ = worker.await;
        }
        
        // Print final worker statistics if monitoring was enabled
        if let Some(ref monitor) = self.worker_monitor {
            // Calculate and display overall health score
            let health_score = monitor.calculate_health_score().await;
            let _state_counts = monitor.get_state_counts().await;
            
            eprintln!("\n📊 Worker health: {:.1}%", health_score);
            
            // Display worker statistics
            let metrics = monitor.get_all_metrics().await;
            let total_processed = metrics.iter().map(|m| m.processed_count).sum::<usize>();
            let total_failed = metrics.iter().map(|m| m.failed_count).sum::<usize>();
            let avg_time = metrics.iter()
                .map(|m| m.avg_processing_time_ms * m.processed_count as f64)
                .sum::<f64>() / total_processed.max(1) as f64;
            
            eprintln!("📈 Processed: {}, Failed: {}, Avg Time: {:.1}ms", 
                total_processed, total_failed, avg_time);
        }

        Ok(results)
    }

    /// Spawn worker pool for parallel execution
    fn spawn_workers(
        &self,
        work_rx: mpsc::Receiver<Mutant>,
        result_tx: mpsc::Sender<MutationResult>,
        semaphore: Arc<Semaphore>,
        completed_count: Arc<AtomicUsize>,
    ) -> Vec<tokio::task::JoinHandle<()>> {
        let mut workers = Vec::new();

        // Shared receiver using Arc<Mutex>
        let work_rx = Arc::new(tokio::sync::Mutex::new(work_rx));
        
        // Heartbeat interval (5 seconds)
        let heartbeat_interval = std::time::Duration::from_secs(5);

        for worker_id in 0..self.config.worker_count {
            let adapter = self.adapter.clone();
            let result_tx = result_tx.clone();
            let semaphore = semaphore.clone();
            let completed_count = completed_count.clone();
            let progress = self.progress.clone();
            let work_rx = work_rx.clone();
            let worker_monitor = self.worker_monitor.clone();

            let worker = tokio::spawn(async move {
                // Set up heartbeat ticker if worker monitoring is enabled
                let mut heartbeat_ticker = tokio::time::interval(heartbeat_interval);
                
                // Mark worker as idle initially
                if let Some(ref monitor) = worker_monitor {
                    monitor.record_heartbeat(worker_id).await;
                }
                
                loop {
                    // Send heartbeat (tokio::select! ensures this happens even while waiting)
                    tokio::select! {
                        _ = heartbeat_ticker.tick() => {
                            if let Some(ref monitor) = worker_monitor {
                                monitor.record_heartbeat(worker_id).await;
                            }
                        }
                        
                        result = async {
                            // Acquire work from shared queue
                            let mutant = {
                                let mut rx = work_rx.lock().await;
                                rx.recv().await
                            };
                            
                            let Some(mutant) = mutant else {
                                return None; // Queue closed
                            };
                            
                            // Acquire semaphore permit (limits concurrency)
                            let _permit = match semaphore.acquire().await {
                                Ok(permit) => permit,
                                Err(_) => {
                                    // Semaphore was closed
                                    return None;
                                }
                            };
                            
                            // Update worker state to processing
                            if let Some(ref monitor) = worker_monitor {
                                monitor.record_start_processing(worker_id).await;
                            }
                            
                            // Update in-progress count
                            {
                                let mut prog = progress.write();
                                prog.in_progress += 1;
                            }
                            
                            // Execute mutant with RAII-based safe error handling
                            let start = std::time::Instant::now();
                            let result = Self::execute_mutant_worker(
                                &adapter,
                                &mutant,
                                worker_id,
                            ).await;
                            let execution_time_ms = start.elapsed().as_millis() as u64;
                            
                            // Update worker metrics
                            if let Some(ref monitor) = worker_monitor {
                                if result.status == MutantStatus::CompileError || result.status == MutantStatus::Timeout {
                                    let error_msg = result.error_message.as_deref().unwrap_or("Unknown error");
                                    monitor.record_failure(worker_id, error_msg).await;
                                } else {
                                    monitor.record_success(worker_id, execution_time_ms).await;
                                }
                            }
                            
                            // Update in-progress count
                            {
                                let mut prog = progress.write();
                                prog.in_progress = prog.in_progress.saturating_sub(1);
                            }
                            
                            // Send result
                            if result_tx.send(result).await.is_err() {
                                return None; // Result channel closed
                            }
                            
                            completed_count.fetch_add(1, Ordering::SeqCst);
                            
                            Some(()) // Continue loop
                        } => {
                            if result.is_none() {
                                // If worker monitoring is enabled, mark worker as terminated
                                if let Some(ref monitor) = worker_monitor {
                                    monitor.mark_terminated(worker_id).await;
                                }
                                break; // Exit worker loop
                            }
                        }
                    }
                }
            });

            workers.push(worker);
        }

        workers
    }

    /// Execute single mutant in worker context
    ///
    /// Uses RAII pattern with WorkerTempFile to ensure proper cleanup of
    /// temporary files even in case of errors or panics.
    async fn execute_mutant_worker(
        adapter: &Arc<dyn LanguageAdapter>,
        mutant: &Mutant,
        worker_id: usize,
    ) -> MutationResult {
        let start = std::time::Instant::now();
        
        // Create temp file with RAII-based cleanup
        let temp_file = super::temp_file::WorkerTempFile::new(
            worker_id, 
            // Convert string ID to numeric ID for temp file naming
            mutant.id.parse::<usize>().unwrap_or_else(|_| {
                // If parsing fails, hash the string to get a numeric value
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                
                let mut hasher = DefaultHasher::new();
                mutant.id.hash(&mut hasher);
                hasher.finish() as usize
            }),
            Some("rs")
        );
        
        // Write mutated source to temp file
        if let Err(e) = temp_file.write(&mutant.mutated_source).await {
            return MutationResult {
                mutant: mutant.clone(),
                status: MutantStatus::CompileError,
                test_failures: vec![],
                execution_time_ms: 0,
                error_message: Some(format!("Failed to write temp file: {}", e)),
            };
        }
        
        // Run tests (temp_file will be cleaned up automatically when dropped)
        let test_result = match adapter.run_tests(temp_file.path()).await {
            Ok(result) => result,
            Err(e) => {
                return MutationResult {
                    mutant: mutant.clone(),
                    status: MutantStatus::CompileError,
                    test_failures: vec![],
                    execution_time_ms: 0,
                    error_message: Some(e.to_string()),
                };
            }
        };
        
        let execution_time_ms = start.elapsed().as_millis() as u64;
        
        // Determine status
        let status = if test_result.passed {
            MutantStatus::Survived
        } else {
            MutantStatus::Killed
        };
        
        // No need to explicitly cleanup - temp_file will be cleaned up when dropped
        
        MutationResult {
            mutant: mutant.clone(),
            status,
            test_failures: test_result.failures,
            execution_time_ms,
            error_message: None,
        }
    }

    /// Get current progress
    pub fn get_progress(&self) -> MutationProgress {
        self.progress.read().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::mutation::RustAdapter;

    #[test]
    fn test_distributed_config_default() {
        let config = DistributedConfig::default();
        assert!(config.worker_count > 0);
        assert!(config.max_concurrent >= config.worker_count);
        assert_eq!(config.queue_size, 1000);
        assert!(config.track_progress);
    }

    #[test]
    fn test_mutation_progress_new() {
        let progress = MutationProgress::new(100);
        assert_eq!(progress.total, 100);
        assert_eq!(progress.completed, 0);
        assert_eq!(progress.in_progress, 0);
    }

    #[test]
    fn test_mutation_progress_percentage() {
        let mut progress = MutationProgress::new(100);
        assert_eq!(progress.percentage(), 0.0);

        progress.completed = 50;
        assert_eq!(progress.percentage(), 50.0);

        progress.completed = 100;
        assert_eq!(progress.percentage(), 100.0);
    }

    #[test]
    fn test_mutation_progress_score() {
        let mut progress = MutationProgress::new(100);
        assert_eq!(progress.mutation_score(), 0.0);

        progress.killed = 80;
        progress.survived = 20;
        assert_eq!(progress.mutation_score(), 80.0);

        progress.killed = 90;
        progress.survived = 10;
        assert_eq!(progress.mutation_score(), 90.0);
    }

    #[test]
    fn test_distributed_executor_creation() {
        let adapter = Arc::new(RustAdapter::new());
        let config = DistributedConfig::default();
        let executor = DistributedExecutor::new(adapter, config);

        let progress = executor.get_progress();
        assert_eq!(progress.total, 0);
    }

    #[actix_rt::test]
    async fn test_parallel_execution_empty() {
        let adapter = Arc::new(RustAdapter::new());
        let config = DistributedConfig {
            worker_count: 2,
            max_concurrent: 4,
            queue_size: 10,
            track_progress: true,
        };
        let executor = DistributedExecutor::new(adapter, config);

        let mutants = vec![];
        let results = executor.execute_parallel(mutants).await.unwrap();

        assert_eq!(results.len(), 0);

        let progress = executor.get_progress();
        assert_eq!(progress.total, 0);
        assert_eq!(progress.completed, 0);
    }
}
