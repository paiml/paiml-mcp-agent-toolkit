pub struct UnifiedEngine {
    // Core analysis infrastructure
    
    pub(crate) ast_engine: Arc<UnifiedAstEngine>,
    
    pub(crate) cache: Arc<UnifiedCacheManager>,
    
    pub(crate) analyzers: AnalyzerPool,

    // Mode-specific components
    pub(crate) mode: EngineMode,
    pub(crate) state_machine: Arc<RwLock<RefactorStateMachine>>,

    // Shared metrics
    
    pub(crate) metrics: Arc<EngineMetrics>,
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
    pub(crate) buffer: VecDeque<T>,
    pub(crate) capacity: usize,
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
