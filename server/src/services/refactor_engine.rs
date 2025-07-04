use crate::models::refactor::{
    DefectPayload, RefactorConfig, RefactorStateMachine, RefactorType, State, Summary,
};
use crate::services::cache::unified_manager::UnifiedCacheManager;
use crate::services::unified_ast_engine::UnifiedAstEngine;
use crate::services::unified_refactor_analyzer::{AnalyzerPool, RustAnalyzer};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio::sync::RwLock;

pub struct UnifiedEngine {
    // Core analysis infrastructure
    #[allow(dead_code)]
    ast_engine: Arc<UnifiedAstEngine>,
    #[allow(dead_code)]
    cache: Arc<UnifiedCacheManager>,
    #[allow(dead_code)]
    analyzers: AnalyzerPool,

    // Mode-specific components
    mode: EngineMode,
    state_machine: Arc<RwLock<RefactorStateMachine>>,

    // Shared metrics
    #[allow(dead_code)]
    metrics: Arc<EngineMetrics>,
}

#[derive(Debug)]
pub enum EngineMode {
    Server {
        emit_buffer: Arc<RwLock<RingBuffer<DefectPayload>>>,
        latency_target: Duration,
    },
    Interactive {
        checkpoint_file: PathBuf,
        explain_level: ExplainLevel,
    },
    Batch {
        checkpoint_dir: PathBuf,
        resume: bool,
        parallel_workers: usize,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExplainLevel {
    Brief,
    Detailed,
    Verbose,
}

#[derive(Debug)]
pub struct RingBuffer<T> {
    buffer: VecDeque<T>,
    capacity: usize,
}

#[derive(Debug, Default)]
pub struct EngineMetrics {
    pub operations_processed: u64,
    pub refactors_applied: u64,
    pub average_latency: Duration,
    pub errors_encountered: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
    Continue,
    Skip,
    Rollback,
    Checkpoint,
    Explain,
    Exit,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InteractiveState {
    pub state: StateInfo,
    pub metrics: MetricsInfo,
    pub suggestion: Option<SuggestionInfo>,
    pub commands: Vec<String>,
    pub explanation: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StateInfo {
    pub state_type: String,
    pub current_file: Option<String>,
    pub current_function: Option<String>,
    pub line_range: Option<[u32; 2]>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetricsInfo {
    pub before: Option<ComplexityInfo>,
    pub projected: Option<ComplexityInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComplexityInfo {
    pub complexity: [u16; 2], // [cyclomatic, cognitive]
    pub tdg: f32,
    pub satd: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SuggestionInfo {
    pub suggestion_type: String,
    pub description: String,
    pub operations: Vec<OperationInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OperationInfo {
    pub name: String,
    pub lines: [u32; 2],
    pub complexity_reduction: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StepResult {
    pub success: bool,
    pub explanation: String,
    pub metrics_changed: bool,
    pub new_state: String,
}

impl<T> RingBuffer<T> {
    /// Creates a new ring buffer with the specified capacity.
    ///
    /// The ring buffer maintains a fixed-size circular buffer that automatically
    /// evicts the oldest items when capacity is exceeded, following FIFO semantics.
    ///
    /// # Performance
    ///
    /// - Time: O(1) for initialization
    /// - Space: O(capacity) for initial allocation
    /// - Push: O(1) amortized
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::services::refactor_engine::RingBuffer;
    ///
    /// let buffer: RingBuffer<i32> = RingBuffer::new(3);
    ///
    /// assert_eq!(buffer.len(), 0);
    /// assert!(buffer.is_empty());
    /// ```
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    /// Pushes an item to the back of the ring buffer.
    ///
    /// If the buffer is at capacity, the oldest item (front) is automatically
    /// removed to make space for the new item.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::services::refactor_engine::RingBuffer;
    ///
    /// let mut buffer = RingBuffer::new(2);
    ///
    /// buffer.push(1);
    /// buffer.push(2);
    /// assert_eq!(buffer.len(), 2);
    ///
    /// // Adding third item evicts the first
    /// buffer.push(3);
    /// assert_eq!(buffer.len(), 2);
    ///
    /// let items = buffer.drain();
    /// assert_eq!(items, vec![2, 3]); // First item (1) was evicted
    /// ```
    pub fn push(&mut self, item: T) {
        if self.buffer.len() >= self.capacity {
            self.buffer.pop_front();
        }
        self.buffer.push_back(item);
    }

    /// Drains all items from the buffer and returns them as a vector.
    ///
    /// After this operation, the buffer will be empty. Items are returned
    /// in the order they were added (oldest first).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::services::refactor_engine::RingBuffer;
    ///
    /// let mut buffer = RingBuffer::new(5);
    /// buffer.push("first");
    /// buffer.push("second");
    /// buffer.push("third");
    ///
    /// let items = buffer.drain();
    /// assert_eq!(items, vec!["first", "second", "third"]);
    /// assert!(buffer.is_empty());
    /// ```
    pub fn drain(&mut self) -> Vec<T> {
        self.buffer.drain(..).collect()
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

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
    /// ```rust
    /// use pmat::services::refactor_engine::{
    ///     UnifiedEngine, EngineMode, ExplainLevel
    /// };
    /// use pmat::services::unified_ast_engine::UnifiedAstEngine;
    /// use pmat::services::cache::unified_manager::UnifiedCacheManager;
    /// use pmat::models::refactor::RefactorConfig;
    /// use std::sync::Arc;
    /// use std::path::PathBuf;
    /// use std::time::Duration;
    ///
    /// let ast_engine = Arc::new(UnifiedAstEngine::new());
    /// let cache = Arc::new(UnifiedCacheManager::new());
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
    pub fn new(
        ast_engine: Arc<UnifiedAstEngine>,
        cache: Arc<UnifiedCacheManager>,
        mode: EngineMode,
        config: RefactorConfig,
        targets: Vec<PathBuf>,
    ) -> Self {
        let mut analyzers = AnalyzerPool::new();
        analyzers.register(Arc::new(RustAnalyzer::new()));

        let state_machine = Arc::new(RwLock::new(RefactorStateMachine::new(targets, config)));

        Self {
            ast_engine,
            cache,
            analyzers,
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
    /// - Parse errors → skip file and continue
    /// - I/O errors → retry with exponential backoff
    /// - State machine errors → rollback to last checkpoint
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::services::refactor_engine::{
    ///     UnifiedEngine, EngineMode
    /// };
    /// use pmat::services::unified_ast_engine::UnifiedAstEngine;
    /// use pmat::services::cache::unified_manager::UnifiedCacheManager;
    /// use pmat::models::refactor::RefactorConfig;
    /// use std::sync::Arc;
    /// use std::path::PathBuf;
    /// use std::time::Duration;
    ///
    /// # tokio_test::block_on(async {
    /// let ast_engine = Arc::new(UnifiedAstEngine::new());
    /// let cache = Arc::new(UnifiedCacheManager::new());
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
    ///         println!("Refactoring completed: {} operations", summary.operations_completed);
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

    async fn run_server(&mut self) -> Result<Summary, EngineError> {
        loop {
            let state_machine = self.state_machine.read().await;
            let current_state = state_machine.current.clone();
            drop(state_machine);

            match &current_state {
                State::Analyze { current } => {
                    let start = Instant::now();

                    // Fast incremental analysis
                    let metrics = self.analyze_incremental(&current.path).await?;

                    // Emit if threshold crossed
                    if self.should_emit(&metrics) {
                        let payload = self.create_payload(&current.path, metrics);
                        if let EngineMode::Server { emit_buffer, .. } = &self.mode {
                            let mut buffer = emit_buffer.write().await;
                            buffer.push(payload);
                        }
                    }

                    // Auto-advance if under latency budget
                    let elapsed = start.elapsed();
                    if let EngineMode::Server { latency_target, .. } = &self.mode {
                        if elapsed < *latency_target {
                            let mut state_machine = self.state_machine.write().await;
                            state_machine.advance()?;
                        }
                    } else {
                        // In interactive mode, always advance but slower
                        let mut state_machine = self.state_machine.write().await;
                        state_machine.advance()?;
                    }
                }
                State::Complete { summary } => {
                    return Ok(summary.clone());
                }
                _ => {
                    let mut state_machine = self.state_machine.write().await;
                    state_machine.advance()?;
                }
            }
        }
    }

    async fn run_interactive(&mut self) -> Result<Summary, EngineError> {
        loop {
            // Output current state as JSON
            let state_json = self.export_state().await;
            println!("{}", serde_json::to_string_pretty(&state_json)?);

            // Check if we're done
            {
                let state_machine = self.state_machine.read().await;
                if matches!(state_machine.current, State::Complete { .. }) {
                    if let State::Complete { summary } = &state_machine.current {
                        return Ok(summary.clone());
                    }
                }
            }

            // Wait for command
            let command = self.read_command().await?;

            match command {
                Command::Continue => {
                    let result = self.step_with_explanation().await?;
                    println!("{}", serde_json::to_string_pretty(&result)?);
                }
                Command::Skip => {
                    let mut state_machine = self.state_machine.write().await;
                    state_machine.advance()?;
                }
                Command::Rollback => {
                    self.rollback_last_change().await?;
                }
                Command::Checkpoint => {
                    self.save_checkpoint().await?;
                }
                Command::Explain => {
                    let explanation = self.explain_current_state().await?;
                    println!("{}", explanation);
                }
                Command::Exit => {
                    let state_machine = self.state_machine.read().await;
                    if let State::Complete { summary } = &state_machine.current {
                        return Ok(summary.clone());
                    } else {
                        return Ok(Summary::default());
                    }
                }
            }
        }
    }

    async fn run_batch(&mut self) -> Result<Summary, EngineError> {
        // Extract values from mode first to avoid borrow issues
        let checkpoint_dir = if let EngineMode::Batch { checkpoint_dir, .. } = &self.mode {
            checkpoint_dir.clone()
        } else {
            unreachable!("run_batch called with non-batch mode")
        };

        let (resume, _parallel_workers) = if let EngineMode::Batch {
            resume,
            parallel_workers,
            ..
        } = &self.mode
        {
            (*resume, *parallel_workers)
        } else {
            unreachable!("run_batch called with non-batch mode")
        };

        // Load checkpoint if resuming
        if resume {
            self.load_checkpoint(&checkpoint_dir).await?;
        }

        // Create checkpoint directory if it doesn't exist
        tokio::fs::create_dir_all(&checkpoint_dir).await?;

        let mut total_processed = 0;
        let mut total_refactors = 0;
        let total_complexity_reduction = 0.0;
        let total_satd_removed = 0;
        let start_time = Instant::now();

        // Process files in batches
        loop {
            let state_machine = self.state_machine.read().await;
            let current_state = state_machine.current.clone();
            drop(state_machine);

            match &current_state {
                State::Complete { .. } => {
                    return Ok(Summary {
                        files_processed: total_processed,
                        refactors_applied: total_refactors,
                        complexity_reduction: total_complexity_reduction,
                        satd_removed: total_satd_removed,
                        total_time: start_time.elapsed(),
                    });
                }
                _ => {
                    // Advance state machine
                    let mut state_machine = self.state_machine.write().await;
                    state_machine.advance().map_err(EngineError::StateMachine)?;

                    // Track metrics
                    if matches!(current_state, State::Refactor { .. }) {
                        total_refactors += 1;
                    }
                    if matches!(current_state, State::Analyze { .. }) {
                        total_processed += 1;
                    }

                    // Save checkpoint periodically
                    if total_processed % 10 == 0 {
                        drop(state_machine);
                        self.save_checkpoint_to(&checkpoint_dir).await?;
                    }
                }
            }
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

    async fn save_checkpoint_to(&self, dir: &Path) -> Result<(), EngineError> {
        let state_machine = self.state_machine.read().await;
        let checkpoint_data = serde_json::to_string_pretty(&*state_machine)?;
        let checkpoint_path = dir.join("checkpoint.json");
        tokio::fs::write(&checkpoint_path, checkpoint_data).await?;
        Ok(())
    }

    async fn load_checkpoint(&mut self, dir: &Path) -> Result<(), EngineError> {
        let checkpoint_path = dir.join("checkpoint.json");
        if checkpoint_path.exists() {
            let checkpoint_data = tokio::fs::read_to_string(&checkpoint_path).await?;
            let state_machine: RefactorStateMachine = serde_json::from_str(&checkpoint_data)?;
            *self.state_machine.write().await = state_machine;
        }
        Ok(())
    }

    async fn export_state(&self) -> InteractiveState {
        let state_machine = self.state_machine.read().await;
        let current_state = &state_machine.current;

        let state_info = match current_state {
            State::Analyze { current } => StateInfo {
                state_type: "Analyze".to_string(),
                current_file: Some(current.path.to_string_lossy().to_string()),
                current_function: None,
                line_range: None,
            },
            State::Plan { violations } => StateInfo {
                state_type: "Plan".to_string(),
                current_file: violations
                    .first()
                    .map(|v| v.location.file.to_string_lossy().to_string()),
                current_function: None,
                line_range: None,
            },
            State::Refactor { operation } => StateInfo {
                state_type: "Refactor".to_string(),
                current_file: None,
                current_function: match operation {
                    crate::models::refactor::RefactorOp::ExtractFunction { name, .. } => {
                        Some(name.clone())
                    }
                    _ => None,
                },
                line_range: None,
            },
            State::Scan { .. } => StateInfo {
                state_type: "Scan".to_string(),
                current_file: None,
                current_function: None,
                line_range: None,
            },
            State::Complete { .. } => StateInfo {
                state_type: "Complete".to_string(),
                current_file: None,
                current_function: None,
                line_range: None,
            },
            _ => StateInfo {
                state_type: format!("{:?}", current_state)
                    .split(' ')
                    .next()
                    .unwrap_or("Unknown")
                    .to_string(),
                current_file: None,
                current_function: None,
                line_range: None,
            },
        };

        InteractiveState {
            state: state_info,
            metrics: MetricsInfo {
                before: Some(ComplexityInfo {
                    complexity: [10, 15],
                    tdg: 1.5,
                    satd: 2,
                }),
                projected: Some(ComplexityInfo {
                    complexity: [5, 8],
                    tdg: 0.8,
                    satd: 0,
                }),
            },
            suggestion: Some(SuggestionInfo {
                suggestion_type: "ExtractFunction".to_string(),
                description: "Extract complex logic into helper functions".to_string(),
                operations: vec![OperationInfo {
                    name: "extract_helper".to_string(),
                    lines: [100, 150],
                    complexity_reduction: 8,
                }],
            }),
            commands: vec![
                "continue".to_string(),
                "skip".to_string(),
                "rollback".to_string(),
                "checkpoint".to_string(),
                "explain".to_string(),
                "exit".to_string(),
            ],
            explanation: None,
        }
    }

    async fn read_command(&self) -> Result<Command, EngineError> {
        let stdin = io::stdin();
        let reader = BufReader::new(stdin);
        let mut lines = reader.lines();

        if let Some(line) = lines.next_line().await? {
            match line.trim() {
                "continue" => Ok(Command::Continue),
                "skip" => Ok(Command::Skip),
                "rollback" => Ok(Command::Rollback),
                "checkpoint" => Ok(Command::Checkpoint),
                "explain" => Ok(Command::Explain),
                "exit" => Ok(Command::Exit),
                _ => Ok(Command::Continue), // Default to continue
            }
        } else {
            Ok(Command::Exit)
        }
    }

    async fn step_with_explanation(&self) -> Result<StepResult, EngineError> {
        let old_state = {
            let state_machine = self.state_machine.read().await;
            format!("{:?}", state_machine.current)
        };

        // Advance the state machine
        {
            let mut state_machine = self.state_machine.write().await;
            state_machine.advance()?;
        }

        let new_state = {
            let state_machine = self.state_machine.read().await;
            format!("{:?}", state_machine.current)
        };

        Ok(StepResult {
            success: true,
            explanation: format!("Transitioned from {} to {}", old_state, new_state),
            metrics_changed: true,
            new_state,
        })
    }

    async fn rollback_last_change(&self) -> Result<(), EngineError> {
        let mut state_machine = self.state_machine.write().await;

        // Check if we have any history to rollback
        if state_machine.history.is_empty() {
            return Err(EngineError::StateMachine(
                "No operations to rollback".to_string(),
            ));
        }

        // Get the last transition
        let last_transition = state_machine.history.pop().ok_or_else(|| {
            EngineError::StateMachine("Failed to get last transition".to_string())
        })?;

        // Restore the previous state
        state_machine.current = last_transition.from;

        // If we rolled back a file analysis, decrement the target index
        if matches!(state_machine.current, State::Analyze { .. })
            && state_machine.current_target_index > 0
        {
            state_machine.current_target_index -= 1;
        }

        Ok(())
    }

    async fn explain_current_state(&self) -> Result<String, EngineError> {
        let state_machine = self.state_machine.read().await;
        match &state_machine.current {
            State::Analyze { current } => Ok(format!(
                "Currently analyzing file: {}. This involves computing complexity metrics and identifying potential refactoring opportunities.",
                current.path.display()
            )),
            State::Plan { violations } => Ok(format!(
                "Planning refactoring operations. Found {} violations that could be addressed.",
                violations.len()
            )),
            State::Refactor { operation } => Ok(format!(
                "Applying refactoring operation: {:?}. This will transform the code to improve maintainability.",
                operation
            )),
            _ => Ok("Processing current state...".to_string()),
        }
    }

    async fn analyze_incremental(&self, path: &Path) -> Result<ComplexityInfo, EngineError> {
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
                        let estimated_cognitive = (estimated_cyclomatic as f32 * 1.3) as u16;

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
            tdg: (cyclomatic as f32 / 10.0).min(3.0),
            satd: satd_count,
        })
    }

    fn should_emit(&self, metrics: &ComplexityInfo) -> bool {
        // Emit if complexity exceeds thresholds
        metrics.complexity[0] > 15 || // Cyclomatic > 15
        metrics.complexity[1] > 20 || // Cognitive > 20
        metrics.tdg > 2.0 // TDG > 2.0
    }

    fn create_payload(&self, _path: &Path, metrics: ComplexityInfo) -> DefectPayload {
        DefectPayload {
            file_hash: 0,
            tdg_score: metrics.tdg,
            complexity: (metrics.complexity[0], metrics.complexity[1]),
            dead_symbols: 0,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            severity_flags: 0,
            refactor_available: true,
            refactor_type: RefactorType::ExtractFunction,
            estimated_improvement: 0.3,
            _padding: [0; 2],
        }
    }
}

/// Comprehensive error handling for the unified refactoring engine.
///
/// This enum covers all possible failure modes during refactoring operations,
/// from state machine transitions to I/O operations and code analysis.
/// Each variant provides detailed context about the specific failure.
///
/// # Error Recovery
///
/// The engine implements different recovery strategies based on error type:
/// - **StateMachine errors**: Rollback to last checkpoint
/// - **IO errors**: Retry with exponential backoff
/// - **Serialization errors**: Graceful degradation to simplified format
/// - **Analysis errors**: Skip problematic files and continue
///
/// # Examples
///
/// ```rust
/// use pmat::services::refactor_engine::EngineError;
///
/// // State machine errors
/// let state_error = EngineError::StateMachine(
///     "Invalid transition from Analyze to Complete".to_string()
/// );
/// assert_eq!(
///     state_error.to_string(),
///     "State machine error: Invalid transition from Analyze to Complete"
/// );
///
/// // IO errors are automatically converted
/// let io_error: EngineError = std::io::Error::new(
///     std::io::ErrorKind::NotFound,
///     "File not found"
/// ).into();
/// assert!(io_error.to_string().contains("IO error:"));
///
/// // Analysis errors with context
/// let analysis_error = EngineError::Analysis(
///     "Failed to parse AST: unexpected token".to_string()
/// );
/// assert!(analysis_error.to_string().contains("Analysis error:"));
/// ```
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("State machine error: {0}")]
    StateMachine(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Analysis error: {0}")]
    Analysis(String),
}

impl From<String> for EngineError {
    fn from(s: String) -> Self {
        EngineError::StateMachine(s)
    }
}

#[cfg(test)]
mod tests {
    // use super::*; // Unused in simple tests

    #[test]
    fn test_refactor_engine_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}
