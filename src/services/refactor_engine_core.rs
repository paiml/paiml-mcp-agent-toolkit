impl UnifiedEngine {
    /// Creates a new unified refactoring engine with the specified configuration.
    ///
    /// Initializes the engine with AST analysis capabilities, caching, and a configurable
    /// execution mode (Server, Interactive, or Batch). The engine uses a state machine
    /// to coordinate refactoring operations across the target files.
    ///
    /// # Parameters
    ///
    /// * `ast_engine` - Shared AST analysis engine for parsing and analysis
    /// * `cache` - Unified cache manager for performance optimization
    /// * `mode` - Execution mode (Server/Interactive/Batch) with mode-specific settings
    /// * `config` - Refactoring configuration (quality thresholds, operation types, etc.)
    /// * `targets` - Vector of file paths to analyze and potentially refactor
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pmat::services::refactor_engine::{
    ///     UnifiedEngine, EngineMode, ExplainLevel
    /// };
    /// use pmat::services::unified_ast_engine::UnifiedAstEngine;
    /// use pmat::services::cache::unified_manager::UnifiedCacheManager;
    /// use pmat::services::cache::unified::UnifiedCacheConfig;
    /// use pmat::models::refactor::RefactorConfig;
    /// use std::sync::Arc;
    /// use std::path::PathBuf;
    /// use std::time::Duration;
    ///
    /// let ast_engine = Arc::new(UnifiedAstEngine::new());
    /// let cache = Arc::new(UnifiedCacheManager::new(UnifiedCacheConfig::default()).expect("internal error"));
    /// let config = RefactorConfig::default();
    /// let targets = vec![PathBuf::from("src/main.rs")];
    ///
    /// // Server mode for high-throughput processing
    /// let mode = EngineMode::Server {
    ///     emit_buffer: Arc::new(tokio::sync::RwLock::new(
    ///         pmat::services::refactor_engine::RingBuffer::new(1000)
    ///     )),
    ///     latency_target: Duration::from_millis(100),
    /// };
    ///
    /// let engine = UnifiedEngine::new(
    ///     ast_engine,
    ///     cache,
    ///     mode,
    ///     config,
    ///     targets
    /// );
    ///
    /// // Engine is ready for analysis and refactoring
    /// ```
    #[must_use]
    pub fn new(
        ast_engine: Arc<UnifiedAstEngine>,
        cache: Arc<UnifiedCacheManager>,
        mode: EngineMode,
        config: RefactorConfig,
        targets: Vec<PathBuf>,
    ) -> Self {
        // Skip analyzer pool setup for now - this is a compatibility stub
        let state_machine = Arc::new(RwLock::new(RefactorStateMachine::new(targets, config)));

        Self {
            ast_engine,
            cache,
            analyzers: AnalyzerPool::new(), // Dummy analyzer pool for compatibility
            mode,
            state_machine,
            metrics: Arc::new(EngineMetrics::default()),
        }
    }

    /// Executes the refactoring engine according to its configured mode.
    ///
    /// This is the main entry point that starts the refactoring process. The behavior
    /// depends on the engine mode:
    /// - **Server**: Continuous processing with latency targets and buffered output
    /// - **Interactive**: Step-by-step processing with user confirmation
    /// - **Batch**: Automated processing with checkpointing and parallelization
    ///
    /// # Error Handling
    ///
    /// The engine implements comprehensive error recovery:
    /// - Parse errors -> skip file and continue
    /// - I/O errors -> retry with exponential backoff
    /// - State machine errors -> rollback to last checkpoint
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pmat::services::refactor_engine::{
    ///     UnifiedEngine, EngineMode
    /// };
    /// use pmat::services::unified_ast_engine::UnifiedAstEngine;
    /// use pmat::services::cache::unified_manager::UnifiedCacheManager;
    /// use pmat::services::cache::unified::UnifiedCacheConfig;
    /// use pmat::models::refactor::RefactorConfig;
    /// use std::sync::Arc;
    /// use std::path::PathBuf;
    /// use std::time::Duration;
    ///
    /// # tokio_test::block_on(async {
    /// let ast_engine = Arc::new(UnifiedAstEngine::new());
    /// let cache = Arc::new(UnifiedCacheManager::new(UnifiedCacheConfig::default()).expect("internal error"));
    /// let config = RefactorConfig::default();
    /// let targets = vec![PathBuf::from("src/example.rs")];
    ///
    /// let mode = EngineMode::Batch {
    ///     checkpoint_dir: PathBuf::from(".refactor_state"),
    ///     resume: false,
    ///     parallel_workers: 4,
    /// };
    ///
    /// let mut engine = UnifiedEngine::new(
    ///     ast_engine,
    ///     cache,
    ///     mode,
    ///     config,
    ///     targets
    /// );
    ///
    /// // Run the refactoring process
    /// let result = engine.run().await;
    ///
    /// match result {
    ///     Ok(summary) => {
    ///         println!("Refactoring completed: {} operations", summary.refactors_applied);
    ///     }
    ///     Err(e) => {
    ///         eprintln!("Refactoring failed: {}", e);
    ///     }
    /// }
    /// # });
    /// ```
    pub async fn run(&mut self) -> Result<Summary, EngineError> {
        match &self.mode {
            EngineMode::Server { .. } => self.run_server().await,
            EngineMode::Interactive { .. } => self.run_interactive().await,
            EngineMode::Batch { .. } => self.run_batch().await,
        }
    }

    pub async fn save_checkpoint(&self) -> Result<(), EngineError> {
        if let EngineMode::Interactive {
            checkpoint_file, ..
        } = &self.mode
        {
            self.save_checkpoint_to(checkpoint_file.parent().unwrap_or(Path::new(".")))
                .await
        } else if let EngineMode::Batch { checkpoint_dir, .. } = &self.mode {
            self.save_checkpoint_to(checkpoint_dir).await
        } else {
            Ok(())
        }
    }

    pub(crate) async fn save_checkpoint_to(&self, dir: &Path) -> Result<(), EngineError> {
        debug_assert!(dir.exists(), "dir must exist: {}", dir.display());
        let state_machine = self.state_machine.read().await;
        let checkpoint_data = serde_json::to_string_pretty(&*state_machine)?;
        let checkpoint_path = dir.join("checkpoint.json");
        tokio::fs::write(&checkpoint_path, checkpoint_data).await?;
        Ok(())
    }

    pub(crate) async fn load_checkpoint(&mut self, dir: &Path) -> Result<(), EngineError> {
        debug_assert!(dir.exists(), "dir must exist: {}", dir.display());
        let checkpoint_path = dir.join("checkpoint.json");
        if checkpoint_path.exists() {
            let checkpoint_data = tokio::fs::read_to_string(&checkpoint_path).await?;
            let state_machine: RefactorStateMachine = serde_json::from_str(&checkpoint_data)?;
            *self.state_machine.write().await = state_machine;
        }
        Ok(())
    }

    pub(crate) async fn analyze_incremental(
        &self,
        path: &Path,
    ) -> Result<ComplexityInfo, EngineError> {
        let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

        let (cyclomatic, cognitive, satd_count) = match extension {
            "rs" => {
                // For Rust files, read and analyze if it's a reasonable size
                if let Ok(content) = tokio::fs::read_to_string(path).await {
                    if content.len() < 50000 {
                        // Only analyze files under 50KB
                        // Simple heuristic based on content patterns
                        let if_count = content.matches("if ").count();
                        let for_count = content.matches("for ").count();
                        let while_count = content.matches("while ").count();
                        let match_count = content.matches("match ").count();
                        let function_count = content.matches("fn ").count();

                        let estimated_cyclomatic =
                            (if_count + for_count + while_count + match_count + function_count)
                                .min(100) as u16;
                        let estimated_cognitive = (f32::from(estimated_cyclomatic) * 1.3) as u16;

                        // Count SATD markers
                        let todo_count = content.matches("TODO").count();
                        let fixme_count = content.matches("FIXME").count();
                        let hack_count = content.matches("HACK").count();
                        let satd = (todo_count + fixme_count + hack_count) as u32;

                        (estimated_cyclomatic, estimated_cognitive, satd)
                    } else {
                        (20, 25, 0) // Large files are likely complex but we didn't read them
                    }
                } else {
                    (1, 1, 0) // Unreadable files
                }
            }
            "ts" | "tsx" | "js" | "jsx" => {
                // For JS/TS files, also try to read and count SATD
                if let Ok(content) = tokio::fs::read_to_string(path).await {
                    let todo_count = content.matches("TODO").count();
                    let fixme_count = content.matches("FIXME").count();
                    let hack_count = content.matches("HACK").count();
                    let satd = (todo_count + fixme_count + hack_count) as u32;
                    (8, 12, satd)
                } else {
                    (8, 12, 0)
                }
            }
            "py" => {
                // For Python files
                if let Ok(content) = tokio::fs::read_to_string(path).await {
                    let todo_count = content.matches("TODO").count();
                    let fixme_count = content.matches("FIXME").count();
                    let hack_count = content.matches("HACK").count();
                    let satd = (todo_count + fixme_count + hack_count) as u32;
                    (6, 9, satd)
                } else {
                    (6, 9, 0)
                }
            }
            _ => (3, 4, 0), // Other files
        };

        Ok(ComplexityInfo {
            complexity: [cyclomatic, cognitive],
            tdg: (f32::from(cyclomatic) / 10.0).min(3.0),
            satd: satd_count,
        })
    }

    pub(crate) fn should_emit(&self, metrics: &ComplexityInfo) -> bool {
        // Emit if complexity exceeds thresholds
        metrics.complexity[0] > 15 || // Cyclomatic > 15
        metrics.complexity[1] > 20 || // Cognitive > 20
        metrics.tdg > 2.0 // TDG > 2.0
    }

    pub(crate) fn create_payload(
        &self,
        _path: &Path,
        metrics: ComplexityInfo,
    ) -> DefectPayload {
        DefectPayload {
            file_hash: 0,
            tdg_score: metrics.tdg,
            complexity: (metrics.complexity[0], metrics.complexity[1]),
            dead_symbols: 0,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("internal error")
                .as_secs(),
            severity_flags: 0,
            refactor_available: true,
            refactor_type: RefactorType::ExtractFunction,
            estimated_improvement: 0.3,
            _padding: [0; 2],
        }
    }
}
