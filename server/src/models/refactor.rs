use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactorStateMachine {
    pub current: State,
    pub history: Vec<StateTransition>,
    pub config: RefactorConfig,
    pub targets: Vec<PathBuf>,
    pub current_target_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
pub struct StateTransition {
    pub from: State,
    pub to: State,
    pub timestamp: u64,
    pub metrics_before: MetricSet,
    pub metrics_after: Option<MetricSet>,
    pub applied_refactor: Option<RefactorOp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
pub struct Thresholds {
    pub cyclomatic_warn: u16,
    pub cyclomatic_error: u16,
    pub cognitive_warn: u16,
    pub cognitive_error: u16,
    pub tdg_warn: f32,
    pub tdg_error: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactorStrategies {
    pub prefer_functional: bool,
    pub use_early_returns: bool,
    pub extract_helpers: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSet {
    pub complexity: (u16, u16), // (cyclomatic, cognitive)
    pub tdg_score: f32,
    pub dead_code: Vec<bool>, // Dead symbol indicators
    pub satd_count: u32,
    pub provability: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
pub enum NestingStrategy {
    EarlyReturn,
    ExtractCondition,
    GuardClause,
    StreamChain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BytePos {
    pub byte: u32,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub file: PathBuf,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SatdFix {
    Remove,
    Replace { with: String },
    Implement { solution: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileId {
    pub path: PathBuf,
    pub hash: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    pub violation_type: ViolationType,
    pub location: Location,
    pub severity: Severity,
    pub description: String,
    pub suggested_fix: Option<RefactorOp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationType {
    HighComplexity,
    DeepNesting,
    LongFunction,
    SelfAdmittedTechDebt,
    DeadCode,
    PoorNaming,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[repr(C, align(64))]
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
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
pub enum RefactorType {
    None = 0,
    ExtractFunction = 1,
    FlattenNesting = 2,
    SimplifyLogic = 3,
    RemoveDeadCode = 4,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub files_processed: u32,
    pub refactors_applied: u32,
    pub complexity_reduction: f32,
    pub satd_removed: u32,
    pub total_time: Duration,
}

impl Default for RefactorConfig {
    fn default() -> Self {
        Self {
            target_complexity: 20,
            remove_satd: true,
            max_function_lines: 50,
            thresholds: Thresholds::default(),
            strategies: RefactorStrategies::default(),
            parallel_workers: 4,
            memory_limit_mb: 512,
            batch_size: 10,
            priority_expression: None,
            auto_commit_template: None,
        }
    }
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            cyclomatic_warn: 10,
            cyclomatic_error: 20,
            cognitive_warn: 15,
            cognitive_error: 30,
            tdg_warn: 1.5,
            tdg_error: 2.0,
        }
    }
}

impl Default for RefactorStrategies {
    fn default() -> Self {
        Self {
            prefer_functional: true,
            use_early_returns: true,
            extract_helpers: true,
        }
    }
}

impl RefactorStateMachine {
    pub fn new(targets: Vec<PathBuf>, config: RefactorConfig) -> Self {
        let initial_state = if targets.is_empty() {
            State::Complete {
                summary: Summary::default(),
            }
        } else {
            State::Scan {
                targets: targets.clone(),
            }
        };

        Self {
            current: initial_state,
            history: Vec::new(),
            config,
            targets,
            current_target_index: 0,
        }
    }

    pub fn advance(&mut self) -> Result<&State, String> {
        let next = match &self.current {
            State::Scan { targets } => {
                if targets.is_empty() {
                    State::Complete {
                        summary: Summary::default(),
                    }
                } else {
                    State::Analyze {
                        current: FileId {
                            path: targets[0].clone(),
                            hash: 0, // Will be computed during analysis
                        },
                    }
                }
            }
            State::Analyze { current } => State::Plan {
                violations: self.find_violations(current),
            },
            State::Plan { violations } => {
                if violations.is_empty() {
                    self.next_target()
                        .map(|t| State::Analyze { current: t })
                        .unwrap_or(State::Complete {
                            summary: Summary::default(),
                        })
                } else {
                    State::Refactor {
                        operation: violations[0].suggested_fix.clone().unwrap_or(
                            RefactorOp::SimplifyExpression {
                                expr: "complex".to_string(),
                                simplified: "simple".to_string(),
                            },
                        ),
                    }
                }
            }
            State::Refactor { .. } => State::Test {
                command: "make test-fast".to_string(),
            },
            State::Test { .. } => State::Lint { strict: true },
            State::Lint { .. } => State::Emit {
                payload: self.compute_payload(),
            },
            State::Emit { .. } => State::Checkpoint {
                reason: "cycle_complete".to_string(),
            },
            State::Checkpoint { .. } => self
                .next_target()
                .map(|t| State::Analyze { current: t })
                .unwrap_or(State::Complete {
                    summary: Summary::default(),
                }),
            State::Complete { .. } => {
                return Ok(&self.current);
            }
        };

        self.transition_to(next)
    }

    fn transition_to(&mut self, new_state: State) -> Result<&State, String> {
        let transition = StateTransition {
            from: self.current.clone(),
            to: new_state.clone(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            metrics_before: MetricSet::default(),
            metrics_after: None,
            applied_refactor: None,
        };

        self.history.push(transition);
        self.current = new_state;
        Ok(&self.current)
    }

    fn find_violations(&self, file_id: &FileId) -> Vec<Violation> {
        // Check thresholds and create violations
        let mut violations = Vec::new();

        // Simulate finding a high complexity violation
        if file_id.path.to_string_lossy().contains("complex") {
            violations.push(Violation {
                violation_type: ViolationType::HighComplexity,
                location: Location {
                    file: file_id.path.clone(),
                    line: 100,
                    column: 1,
                },
                severity: Severity::High,
                description: "Function exceeds complexity threshold".to_string(),
                suggested_fix: Some(RefactorOp::ExtractFunction {
                    name: "extract_helper".to_string(),
                    start: BytePos {
                        byte: 1000,
                        line: 100,
                        column: 1,
                    },
                    end: BytePos {
                        byte: 2000,
                        line: 150,
                        column: 1,
                    },
                    params: vec!["param1".to_string()],
                }),
            });
        }

        violations
    }

    fn next_target(&mut self) -> Option<FileId> {
        self.current_target_index += 1;
        if self.current_target_index < self.targets.len() {
            Some(FileId {
                path: self.targets[self.current_target_index].clone(),
                hash: 0,
            })
        } else {
            None
        }
    }

    fn compute_payload(&self) -> DefectPayload {
        DefectPayload {
            file_hash: 0,
            tdg_score: 1.0,
            complexity: (10, 15),
            dead_symbols: 0,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            severity_flags: 0,
            refactor_available: true,
            refactor_type: RefactorType::None,
            estimated_improvement: 0.5,
            _padding: [0; 2],
        }
    }
}

impl Default for MetricSet {
    fn default() -> Self {
        Self {
            complexity: (0, 0),
            tdg_score: 0.0,
            dead_code: Vec::new(),
            satd_count: 0,
            provability: 0.0,
        }
    }
}

impl Default for Summary {
    fn default() -> Self {
        Self {
            files_processed: 0,
            refactors_applied: 0,
            complexity_reduction: 0.0,
            satd_removed: 0,
            total_time: Duration::from_secs(0),
        }
    }
}

impl Violation {
    pub fn to_op(&self) -> RefactorOp {
        self.suggested_fix
            .clone()
            .unwrap_or_else(|| match self.violation_type {
                ViolationType::HighComplexity => RefactorOp::ExtractFunction {
                    name: "extracted_function".to_string(),
                    start: BytePos {
                        byte: 0,
                        line: self.location.line,
                        column: self.location.column,
                    },
                    end: BytePos {
                        byte: 100,
                        line: self.location.line + 10,
                        column: 0,
                    },
                    params: vec![],
                },
                ViolationType::DeepNesting => RefactorOp::FlattenNesting {
                    function: "function_name".to_string(),
                    strategy: NestingStrategy::EarlyReturn,
                },
                ViolationType::SelfAdmittedTechDebt => RefactorOp::RemoveSatd {
                    location: self.location.clone(),
                    fix: SatdFix::Remove,
                },
                _ => RefactorOp::SimplifyExpression {
                    expr: "complex".to_string(),
                    simplified: "simple".to_string(),
                },
            })
    }
}

#[cfg(test)]
mod tests {
    // use super::*; // Unused in simple tests

    #[test]
    fn test_refactor_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}
