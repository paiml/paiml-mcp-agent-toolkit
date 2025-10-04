//! Distributed mutation testing execution
//!
//! Provides parallel mutant execution with work queue distribution,
//! progress tracking, and result aggregation for production-scale
//! mutation testing workloads.

use super::types::*;
use super::language::LanguageAdapter;
use anyhow::{Context, Result};
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
    adapter: Arc<dyn LanguageAdapter>,
    config: DistributedConfig,
    progress: Arc<RwLock<MutationProgress>>,
}

impl DistributedExecutor {
    /// Create new distributed executor
    pub fn new(adapter: Arc<dyn LanguageAdapter>, config: DistributedConfig) -> Self {
        Self {
            adapter,
            config,
            progress: Arc::new(RwLock::new(MutationProgress::new(0))),
        }
    }

    /// Execute mutants in parallel across worker pool
    pub async fn execute_parallel(&self, mutants: Vec<Mutant>) -> Result<Vec<MutationResult>> {
        // Initialize progress
        {
            let mut progress = self.progress.write();
            *progress = MutationProgress::new(mutants.len());
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
            result_tx,
            semaphore.clone(),
            completed_count.clone(),
        );

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
        while let Some(result) = result_rx.recv().await {
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

            // Check if complete
            if results.len() >= total {
                break;
            }
        }

        // Wait for all tasks to complete
        sender_task.await?;

        // Wait for all workers
        for worker in workers {
            let _ = worker.await;
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

        for worker_id in 0..self.config.worker_count {
            let adapter = self.adapter.clone();
            let result_tx = result_tx.clone();
            let semaphore = semaphore.clone();
            let completed_count = completed_count.clone();
            let progress = self.progress.clone();
            let work_rx = work_rx.clone();

            let worker = tokio::spawn(async move {
                loop {
                    // Acquire work from shared queue
                    let mutant = {
                        let mut rx = work_rx.lock().await;
                        rx.recv().await
                    };

                    let Some(mutant) = mutant else {
                        break; // Queue closed
                    };

                    // Acquire semaphore permit (limits concurrency)
                    let _permit = semaphore.acquire().await.unwrap();

                    // Update in-progress count
                    {
                        let mut prog = progress.write();
                        prog.in_progress += 1;
                    }

                    // Execute mutant
                    let result = Self::execute_mutant_worker(
                        &adapter,
                        &mutant,
                        worker_id,
                    ).await;

                    // Update in-progress count
                    {
                        let mut prog = progress.write();
                        prog.in_progress = prog.in_progress.saturating_sub(1);
                    }

                    // Send result
                    if result_tx.send(result).await.is_err() {
                        break;
                    }

                    completed_count.fetch_add(1, Ordering::SeqCst);
                }
            });

            workers.push(worker);
        }

        workers
    }

    /// Execute single mutant in worker context
    async fn execute_mutant_worker(
        adapter: &Arc<dyn LanguageAdapter>,
        mutant: &Mutant,
        worker_id: usize,
    ) -> MutationResult {
        // Write mutated source to temp file (worker-specific)
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join(format!("mutant_w{}_{}.rs", worker_id, mutant.id));

        if let Err(e) = tokio::fs::write(&temp_file, &mutant.mutated_source).await {
            return MutationResult {
                mutant: mutant.clone(),
                status: MutantStatus::CompileError,
                test_failures: vec![],
                execution_time_ms: 0,
                error_message: Some(format!("Failed to write temp file: {}", e)),
            };
        }

        // Run tests
        let start = std::time::Instant::now();
        let test_result = match adapter.run_tests(&temp_file).await {
            Ok(result) => result,
            Err(e) => {
                // Cleanup
                let _ = tokio::fs::remove_file(&temp_file).await;

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

        // Cleanup temp file
        let _ = tokio::fs::remove_file(&temp_file).await;

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
