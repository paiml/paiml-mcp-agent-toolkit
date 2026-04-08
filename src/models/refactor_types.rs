#[derive(Debug, Clone, Serialize, Deserialize)]
/// Refactor state machine.
pub struct RefactorStateMachine {
    pub current: State,
    pub history: Vec<StateTransition>,
    pub config: RefactorConfig,
    pub targets: Vec<PathBuf>,
    pub current_target_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Refactoring state machine lifecycle state.
pub enum State {
    Scan { targets: Vec<PathBuf> },
    Analyze { current: FileId },
    Plan { violations: Vec<Violation> },
    Refactor { operation: RefactorOp },
    Test { command: String },
    Lint { strict: bool },
    Emit { payload: DefectPayload },
    Checkpoint { reason: String },
    Complete { summary: Summary },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// State transition.
pub struct StateTransition {
    pub from: State,
    pub to: State,
    pub timestamp: u64,
    pub metrics_before: MetricSet,
    pub metrics_after: Option<MetricSet>,
    pub applied_refactor: Option<RefactorOp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Configuration for refactor.
pub struct RefactorConfig {
    pub target_complexity: u16,
    pub remove_satd: bool,
    pub max_function_lines: u32,
    pub thresholds: Thresholds,
    pub strategies: RefactorStrategies,
    pub parallel_workers: usize,
    pub memory_limit_mb: usize,
    pub batch_size: usize,
    pub priority_expression: Option<String>,
    pub auto_commit_template: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Threshold values for thresholds.
pub struct Thresholds {
    pub cyclomatic_warn: u16,
    pub cyclomatic_error: u16,
    pub cognitive_warn: u16,
    pub cognitive_error: u16,
    pub tdg_warn: f32,
    pub tdg_error: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Refactor strategies.
pub struct RefactorStrategies {
    pub prefer_functional: bool,
    pub use_early_returns: bool,
    pub extract_helpers: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Metric set.
pub struct MetricSet {
    pub complexity: (u16, u16), // (cyclomatic, cognitive)
    pub tdg_score: f32,
    pub dead_code: Vec<bool>, // Dead symbol indicators
    pub satd_count: u32,
    pub provability: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Available operations for refactor.
pub enum RefactorOp {
    ExtractFunction {
        name: String,
        start: BytePos,
        end: BytePos,
        params: Vec<String>,
    },
    FlattenNesting {
        function: String,
        strategy: NestingStrategy,
    },
    ReplaceHashMap {
        imports: Vec<String>,
        replacements: Vec<(String, String)>,
    },
    RemoveSatd {
        location: Location,
        fix: SatdFix,
    },
    SimplifyExpression {
        expr: String,
        simplified: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Strategy options for nesting.
pub enum NestingStrategy {
    EarlyReturn,
    ExtractCondition,
    GuardClause,
    StreamChain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Byte pos.
pub struct BytePos {
    pub byte: u32,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Location.
pub struct Location {
    pub file: PathBuf,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Satd fix.
pub enum SatdFix {
    Remove,
    Replace { with: String },
    Implement { solution: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// File id.
pub struct FileId {
    pub path: PathBuf,
    pub hash: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Violation record for violation.
pub struct Violation {
    pub violation_type: ViolationType,
    pub location: Location,
    pub severity: Severity,
    pub description: String,
    pub suggested_fix: Option<RefactorOp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Type classification for violation.
pub enum ViolationType {
    HighComplexity,
    DeepNesting,
    LongFunction,
    SelfAdmittedTechDebt,
    DeadCode,
    PoorNaming,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Severity level classification for severity.
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[repr(C, align(64))]
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
/// Defect payload.
pub struct DefectPayload {
    pub file_hash: u64,
    pub tdg_score: f32,
    pub complexity: (u16, u16),
    pub dead_symbols: u32,
    pub timestamp: u64,
    pub severity_flags: u8,
    pub refactor_available: bool,
    pub refactor_type: RefactorType,
    pub estimated_improvement: f32,
    pub _padding: [u8; 2],
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
/// Type classification for refactor.
pub enum RefactorType {
    None = 0,
    ExtractFunction = 1,
    FlattenNesting = 2,
    SimplifyLogic = 3,
    RemoveDeadCode = 4,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Summary of summary analysis.
pub struct Summary {
    pub files_processed: u32,
    pub refactors_applied: u32,
    pub complexity_reduction: f32,
    pub satd_removed: u32,
    pub total_time: Duration,
}
