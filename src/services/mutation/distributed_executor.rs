// DistributedExecutor implementation methods: construction, parallel execution,
// worker pool management, and individual mutant execution.

async fn initialize_worker_monitoring(
    monitor: Arc<super::worker_monitor::WorkerMonitor>,
) {
    monitor.initialize_workers().await;
    let monitor_clone = Arc::clone(&monitor);
    let monitoring_interval = std::time::Duration::from_secs(10);
    let _monitoring_task = super::worker_monitor::WorkerMonitor::run_monitoring_task(
        monitor_clone,
        monitoring_interval,
        |worker_id| {
            eprintln!(
                "⚠️ Worker {} appears to be stalled, execution may be slow",
                worker_id
            );
        },
    )
    .await;
}

fn compute_worker_id(mutant_id: &str, worker_count: usize) -> usize {
    let numeric_id = mutant_id.parse::<usize>().unwrap_or_else(|_| {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        mutant_id.hash(&mut hasher);
        hasher.finish() as usize
    });
    numeric_id % worker_count
}

async fn record_result_metrics(
    monitor: &super::worker_monitor::WorkerMonitor,
    result: &MutationResult,
    worker_count: usize,
) {
    let worker_id = compute_worker_id(&result.mutant.id, worker_count);
    if result.status == MutantStatus::CompileError || result.status == MutantStatus::Timeout {
        let error_msg = result.error_message.as_deref().unwrap_or("Unknown error");
        monitor.record_failure(worker_id, error_msg).await;
    } else {
        monitor
            .record_success(worker_id, result.execution_time_ms)
            .await;
    }
}

fn update_progress(progress: &Arc<RwLock<MutationProgress>>, result: &MutationResult) {
    let mut progress = progress.write();
    progress.completed += 1;
    match result.status {
        MutantStatus::Killed => progress.killed += 1,
        MutantStatus::Survived => progress.survived += 1,
        MutantStatus::CompileError | MutantStatus::Timeout => progress.failed += 1,
        _ => {}
    }
}

fn check_shutdown_signal(shutdown_rx: &mut mpsc::Receiver<()>) -> bool {
    if shutdown_rx.try_recv().is_ok() {
        eprintln!("🛑 Graceful shutdown in progress, waiting for current tasks...");
        return true;
    }
    false
}

fn setup_shutdown_channel() -> (mpsc::Sender<()>, mpsc::Receiver<()>) {
    let (shutdown_tx, shutdown_rx) = mpsc::channel::<()>(1);

    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        let shutdown_tx_clone = shutdown_tx.clone();
        tokio::spawn(async move {
            let mut sigint =
                signal(SignalKind::interrupt()).expect("Failed to set up SIGINT handler");
            sigint.recv().await;
            eprintln!("\n🛑 Received interrupt signal, stopping gracefully...");
            let _ = shutdown_tx_clone.send(()).await;
        });
    }

    (shutdown_tx, shutdown_rx)
}

async fn print_worker_statistics(monitor: &super::worker_monitor::WorkerMonitor) {
    let health_score = monitor.calculate_health_score().await;
    let _state_counts = monitor.get_state_counts().await;
    eprintln!("\n📊 Worker health: {:.1}%", health_score);

    let metrics = monitor.get_all_metrics().await;
    let total_processed = metrics.iter().map(|m| m.processed_count).sum::<usize>();
    let total_failed = metrics.iter().map(|m| m.failed_count).sum::<usize>();
    let avg_time = metrics
        .iter()
        .map(|m| m.avg_processing_time_ms * m.processed_count as f64)
        .sum::<f64>()
        / total_processed.max(1) as f64;
    eprintln!(
        "📈 Processed: {}, Failed: {}, Avg Time: {:.1}ms",
        total_processed, total_failed, avg_time
    );
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
            initialize_worker_monitoring(Arc::clone(monitor)).await;
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
            result_tx.clone(),
            semaphore.clone(),
            completed_count.clone(),
        );

        // Set up signal handler for graceful shutdown
        let (_shutdown_tx, mut shutdown_rx) = setup_shutdown_channel();

        // Send all mutants to work queue
        let sender_task = tokio::spawn(async move {
            for mutant in mutants {
                if work_tx.send(mutant).await.is_err() {
                    break;
                }
            }
            drop(work_tx);
        });

        // Collect results
        let results = self
            .collect_results(&mut result_rx, &mut shutdown_rx)
            .await;

        // Wait for all tasks to complete
        sender_task.await?;
        Self::await_workers(workers).await;

        // Print final worker statistics if monitoring was enabled
        if let Some(ref monitor) = self.worker_monitor {
            print_worker_statistics(monitor).await;
        }

        Ok(results)
    }

    /// Collect results from the result channel, handling shutdown signals
    async fn collect_results(
        &self,
        result_rx: &mut mpsc::Receiver<MutationResult>,
        shutdown_rx: &mut mpsc::Receiver<()>,
    ) -> Vec<MutationResult> {
        let total = { self.progress.read().total };
        let mut results = Vec::with_capacity(total);
        let mut shutdown_requested = false;

        while let Some(result) = result_rx.recv().await {
            shutdown_requested =
                shutdown_requested || check_shutdown_signal(shutdown_rx);

            if let Some(ref monitor) = self.worker_monitor {
                record_result_metrics(monitor, &result, self.config.worker_count).await;
            }

            update_progress(&self.progress, &result);
            results.push(result);

            if results.len() >= total || shutdown_requested {
                break;
            }
        }

        if shutdown_requested {
            eprintln!(
                "🛑 Graceful shutdown completed. Processed {} of {} mutants.",
                results.len(),
                total
            );
        }
        results
    }

    /// Wait for all worker tasks to terminate
    async fn await_workers(workers: Vec<tokio::task::JoinHandle<()>>) {
        for worker in workers {
            let _ = worker.await;
        }
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

        // Create scratch file with RAII-based cleanup
        let mutant_numeric_id = compute_worker_id(&mutant.id, usize::MAX);
        let temp_file = super::temp_file::WorkerTempFile::new(
            worker_id,
            mutant_numeric_id,
            Some("rs"),
        );

        // Write mutated source to scratch file
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
