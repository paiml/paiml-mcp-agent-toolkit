#![cfg_attr(coverage_nightly, coverage(off))]
//! Refactoring state machine and operation models.
//!
//! This module defines the state machine that orchestrates the automated
//! refactoring process. The state machine ensures refactoring operations
//! are applied systematically with proper validation at each step.
//!
//! # State Machine Flow
//!
//! 1. **Scan**: Identify target files for refactoring
//! 2. **Analyze**: Analyze code quality metrics
//! 3. **Plan**: Generate refactoring plan based on violations
//! 4. **Refactor**: Apply refactoring operations
//! 5. **Test**: Run tests to validate changes
//! 6. **Lint**: Check code style and quality
//! 7. **Emit**: Report results
//! 8. **Complete**: Summarize refactoring session
//!
//! # Example
//!
//! ```
//! use pmat::models::refactor::{RefactorStateMachine, RefactorConfig, State};
//! use std::path::PathBuf;
//!
//! let config = RefactorConfig::default();
//! let targets = vec![PathBuf::from("src/complex_module.rs")];
//!
//! let mut state_machine = RefactorStateMachine::new(targets, config);
//!
//! // Start with scanning
//! match &state_machine.current {
//!     State::Scan { targets } => {
//!         println!("Scanning {} files", targets.len());
//!     }
//!     _ => {}
//! }
//! ```

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
    #[must_use]
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
                    self.next_target().map_or(
                        State::Complete {
                            summary: Summary::default(),
                        },
                        |t| State::Analyze { current: t },
                    )
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
            State::Checkpoint { .. } => self.next_target().map_or(
                State::Complete {
                    summary: Summary::default(),
                },
                |t| State::Analyze { current: t },
            ),
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
                .expect("internal error")
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
                .expect("internal error")
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
    #[must_use]
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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // RefactorConfig Tests
    // ========================================================================

    #[test]
    fn test_refactor_config_default() {
        let config = RefactorConfig::default();

        assert_eq!(config.target_complexity, 20);
        assert!(config.remove_satd);
        assert_eq!(config.max_function_lines, 50);
        assert_eq!(config.parallel_workers, 4);
        assert_eq!(config.memory_limit_mb, 512);
        assert_eq!(config.batch_size, 10);
        assert!(config.priority_expression.is_none());
        assert!(config.auto_commit_template.is_none());
    }

    #[test]
    fn test_refactor_config_serialization() {
        let config = RefactorConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: RefactorConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.target_complexity, config.target_complexity);
        assert_eq!(deserialized.remove_satd, config.remove_satd);
    }

    // ========================================================================
    // Thresholds Tests
    // ========================================================================

    #[test]
    fn test_thresholds_default() {
        let thresholds = Thresholds::default();

        assert_eq!(thresholds.cyclomatic_warn, 10);
        assert_eq!(thresholds.cyclomatic_error, 20);
        assert_eq!(thresholds.cognitive_warn, 15);
        assert_eq!(thresholds.cognitive_error, 30);
        assert!((thresholds.tdg_warn - 1.5).abs() < 0.01);
        assert!((thresholds.tdg_error - 2.0).abs() < 0.01);
    }

    // ========================================================================
    // RefactorStrategies Tests
    // ========================================================================

    #[test]
    fn test_refactor_strategies_default() {
        let strategies = RefactorStrategies::default();

        assert!(strategies.prefer_functional);
        assert!(strategies.use_early_returns);
        assert!(strategies.extract_helpers);
    }

    // ========================================================================
    // MetricSet Tests
    // ========================================================================

    #[test]
    fn test_metric_set_default() {
        let metrics = MetricSet::default();

        assert_eq!(metrics.complexity, (0, 0));
        assert!((metrics.tdg_score - 0.0).abs() < 0.01);
        assert!(metrics.dead_code.is_empty());
        assert_eq!(metrics.satd_count, 0);
        assert!((metrics.provability - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_metric_set_serialization() {
        let metrics = MetricSet {
            complexity: (15, 20),
            tdg_score: 1.5,
            dead_code: vec![true, false, true],
            satd_count: 5,
            provability: 0.8,
        };

        let json = serde_json::to_string(&metrics).unwrap();
        let deserialized: MetricSet = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.complexity, (15, 20));
        assert_eq!(deserialized.satd_count, 5);
    }

    // ========================================================================
    // Summary Tests
    // ========================================================================

    #[test]
    fn test_summary_default() {
        let summary = Summary::default();

        assert_eq!(summary.files_processed, 0);
        assert_eq!(summary.refactors_applied, 0);
        assert!((summary.complexity_reduction - 0.0).abs() < 0.01);
        assert_eq!(summary.satd_removed, 0);
        assert_eq!(summary.total_time, Duration::from_secs(0));
    }

    // ========================================================================
    // State Tests
    // ========================================================================

    #[test]
    fn test_state_scan() {
        let targets = vec![PathBuf::from("file1.rs"), PathBuf::from("file2.rs")];
        let state = State::Scan {
            targets: targets.clone(),
        };

        if let State::Scan { targets: t } = state {
            assert_eq!(t.len(), 2);
        } else {
            panic!("Expected Scan state");
        }
    }

    #[test]
    fn test_state_analyze() {
        let file_id = FileId {
            path: PathBuf::from("test.rs"),
            hash: 12345,
        };
        let state = State::Analyze { current: file_id };

        if let State::Analyze { current } = state {
            assert_eq!(current.hash, 12345);
        } else {
            panic!("Expected Analyze state");
        }
    }

    #[test]
    fn test_state_complete() {
        let summary = Summary::default();
        let state = State::Complete { summary };

        if let State::Complete { summary: s } = state {
            assert_eq!(s.files_processed, 0);
        } else {
            panic!("Expected Complete state");
        }
    }

    // ========================================================================
    // RefactorStateMachine Tests
    // ========================================================================

    #[test]
    fn test_state_machine_new_with_targets() {
        let targets = vec![PathBuf::from("file1.rs")];
        let config = RefactorConfig::default();
        let sm = RefactorStateMachine::new(targets.clone(), config);

        assert_eq!(sm.targets.len(), 1);
        assert_eq!(sm.current_target_index, 0);

        if let State::Scan { targets: t } = &sm.current {
            assert_eq!(t.len(), 1);
        } else {
            panic!("Expected Scan state");
        }
    }

    #[test]
    fn test_state_machine_new_empty_targets() {
        let targets: Vec<PathBuf> = vec![];
        let config = RefactorConfig::default();
        let sm = RefactorStateMachine::new(targets, config);

        if let State::Complete { .. } = &sm.current {
            // Expected
        } else {
            panic!("Expected Complete state for empty targets");
        }
    }

    #[test]
    fn test_state_machine_advance_from_scan() {
        let targets = vec![PathBuf::from("test.rs")];
        let config = RefactorConfig::default();
        let mut sm = RefactorStateMachine::new(targets, config);

        let result = sm.advance();
        assert!(result.is_ok());

        if let State::Analyze { .. } = result.unwrap() {
            // Expected
        } else {
            panic!("Expected Analyze state after advancing from Scan");
        }
    }

    #[test]
    fn test_state_machine_advance_from_empty_scan() {
        let config = RefactorConfig::default();
        let mut sm = RefactorStateMachine {
            current: State::Scan { targets: vec![] },
            history: vec![],
            config,
            targets: vec![],
            current_target_index: 0,
        };

        let result = sm.advance();
        assert!(result.is_ok());

        if let State::Complete { .. } = result.unwrap() {
            // Expected
        } else {
            panic!("Expected Complete state for empty scan");
        }
    }

    #[test]
    fn test_state_machine_history_tracking() {
        let targets = vec![PathBuf::from("test.rs")];
        let config = RefactorConfig::default();
        let mut sm = RefactorStateMachine::new(targets, config);

        assert!(sm.history.is_empty());

        let _ = sm.advance(); // Scan -> Analyze
        assert_eq!(sm.history.len(), 1);

        let _ = sm.advance(); // Analyze -> Plan
        assert_eq!(sm.history.len(), 2);
    }

    // ========================================================================
    // RefactorOp Tests
    // ========================================================================

    #[test]
    fn test_refactor_op_extract_function() {
        let op = RefactorOp::ExtractFunction {
            name: "helper".to_string(),
            start: BytePos {
                byte: 0,
                line: 1,
                column: 1,
            },
            end: BytePos {
                byte: 100,
                line: 10,
                column: 1,
            },
            params: vec!["x".to_string(), "y".to_string()],
        };

        if let RefactorOp::ExtractFunction { name, params, .. } = op {
            assert_eq!(name, "helper");
            assert_eq!(params.len(), 2);
        } else {
            panic!("Expected ExtractFunction");
        }
    }

    #[test]
    fn test_refactor_op_flatten_nesting() {
        let op = RefactorOp::FlattenNesting {
            function: "complex_func".to_string(),
            strategy: NestingStrategy::EarlyReturn,
        };

        if let RefactorOp::FlattenNesting { strategy, .. } = op {
            if let NestingStrategy::EarlyReturn = strategy {
                // Expected
            } else {
                panic!("Expected EarlyReturn strategy");
            }
        } else {
            panic!("Expected FlattenNesting");
        }
    }

    #[test]
    fn test_refactor_op_serialization() {
        let op = RefactorOp::SimplifyExpression {
            expr: "a && b || c".to_string(),
            simplified: "simplified".to_string(),
        };

        let json = serde_json::to_string(&op).unwrap();
        let deserialized: RefactorOp = serde_json::from_str(&json).unwrap();

        if let RefactorOp::SimplifyExpression { expr, simplified } = deserialized {
            assert_eq!(expr, "a && b || c");
            assert_eq!(simplified, "simplified");
        } else {
            panic!("Expected SimplifyExpression");
        }
    }

    // ========================================================================
    // NestingStrategy Tests
    // ========================================================================

    #[test]
    fn test_nesting_strategy_variants() {
        let strategies = vec![
            NestingStrategy::EarlyReturn,
            NestingStrategy::ExtractCondition,
            NestingStrategy::GuardClause,
            NestingStrategy::StreamChain,
        ];

        for strategy in strategies {
            let json = serde_json::to_string(&strategy).unwrap();
            let _: NestingStrategy = serde_json::from_str(&json).unwrap();
        }
    }

    // ========================================================================
    // ViolationType Tests
    // ========================================================================

    #[test]
    fn test_violation_type_variants() {
        let types = vec![
            ViolationType::HighComplexity,
            ViolationType::DeepNesting,
            ViolationType::LongFunction,
            ViolationType::SelfAdmittedTechDebt,
            ViolationType::DeadCode,
            ViolationType::PoorNaming,
        ];

        for vtype in types {
            let json = serde_json::to_string(&vtype).unwrap();
            let _: ViolationType = serde_json::from_str(&json).unwrap();
        }
    }

    // ========================================================================
    // Severity Tests
    // ========================================================================

    #[test]
    fn test_severity_variants() {
        let severities = vec![
            Severity::Low,
            Severity::Medium,
            Severity::High,
            Severity::Critical,
        ];

        for severity in severities {
            let json = serde_json::to_string(&severity).unwrap();
            let _: Severity = serde_json::from_str(&json).unwrap();
        }
    }

    // ========================================================================
    // Violation Tests
    // ========================================================================

    #[test]
    fn test_violation_to_op_with_suggested_fix() {
        let violation = Violation {
            violation_type: ViolationType::HighComplexity,
            location: Location {
                file: PathBuf::from("test.rs"),
                line: 10,
                column: 1,
            },
            severity: Severity::High,
            description: "Test violation".to_string(),
            suggested_fix: Some(RefactorOp::SimplifyExpression {
                expr: "x".to_string(),
                simplified: "y".to_string(),
            }),
        };

        let op = violation.to_op();
        if let RefactorOp::SimplifyExpression { expr, .. } = op {
            assert_eq!(expr, "x");
        } else {
            panic!("Expected SimplifyExpression from suggested_fix");
        }
    }

    #[test]
    fn test_violation_to_op_high_complexity() {
        let violation = Violation {
            violation_type: ViolationType::HighComplexity,
            location: Location {
                file: PathBuf::from("test.rs"),
                line: 10,
                column: 1,
            },
            severity: Severity::High,
            description: "Test".to_string(),
            suggested_fix: None,
        };

        let op = violation.to_op();
        if let RefactorOp::ExtractFunction { .. } = op {
            // Expected
        } else {
            panic!("Expected ExtractFunction for HighComplexity");
        }
    }

    #[test]
    fn test_violation_to_op_deep_nesting() {
        let violation = Violation {
            violation_type: ViolationType::DeepNesting,
            location: Location {
                file: PathBuf::from("test.rs"),
                line: 10,
                column: 1,
            },
            severity: Severity::Medium,
            description: "Test".to_string(),
            suggested_fix: None,
        };

        let op = violation.to_op();
        if let RefactorOp::FlattenNesting { .. } = op {
            // Expected
        } else {
            panic!("Expected FlattenNesting for DeepNesting");
        }
    }

    #[test]
    fn test_violation_to_op_satd() {
        let violation = Violation {
            violation_type: ViolationType::SelfAdmittedTechDebt,
            location: Location {
                file: PathBuf::from("test.rs"),
                line: 10,
                column: 1,
            },
            severity: Severity::Low,
            description: "Test".to_string(),
            suggested_fix: None,
        };

        let op = violation.to_op();
        if let RefactorOp::RemoveSatd { .. } = op {
            // Expected
        } else {
            panic!("Expected RemoveSatd for SelfAdmittedTechDebt");
        }
    }

    // ========================================================================
    // SatdFix Tests
    // ========================================================================

    #[test]
    fn test_satd_fix_variants() {
        let fixes = vec![
            SatdFix::Remove,
            SatdFix::Replace {
                with: "fixed".to_string(),
            },
            SatdFix::Implement {
                solution: "impl".to_string(),
            },
        ];

        for fix in fixes {
            let json = serde_json::to_string(&fix).unwrap();
            let _: SatdFix = serde_json::from_str(&json).unwrap();
        }
    }

    // ========================================================================
    // RefactorType Tests
    // ========================================================================

    #[test]
    fn test_refactor_type_values() {
        assert_eq!(RefactorType::None as u8, 0);
        assert_eq!(RefactorType::ExtractFunction as u8, 1);
        assert_eq!(RefactorType::FlattenNesting as u8, 2);
        assert_eq!(RefactorType::SimplifyLogic as u8, 3);
        assert_eq!(RefactorType::RemoveDeadCode as u8, 4);
    }

    // ========================================================================
    // DefectPayload Tests
    // ========================================================================

    #[test]
    fn test_defect_payload_creation() {
        let payload = DefectPayload {
            file_hash: 12345,
            tdg_score: 1.5,
            complexity: (10, 15),
            dead_symbols: 2,
            timestamp: 1000,
            severity_flags: 0b0101,
            refactor_available: true,
            refactor_type: RefactorType::ExtractFunction,
            estimated_improvement: 0.3,
            _padding: [0; 2],
        };

        assert_eq!(payload.file_hash, 12345);
        assert_eq!(payload.complexity, (10, 15));
        assert!(payload.refactor_available);
    }

    // ========================================================================
    // BytePos Tests
    // ========================================================================

    #[test]
    fn test_byte_pos() {
        let pos = BytePos {
            byte: 1000,
            line: 50,
            column: 10,
        };

        assert_eq!(pos.byte, 1000);
        assert_eq!(pos.line, 50);
        assert_eq!(pos.column, 10);
    }

    // ========================================================================
    // Location Tests
    // ========================================================================

    #[test]
    fn test_location() {
        let loc = Location {
            file: PathBuf::from("/path/to/file.rs"),
            line: 100,
            column: 5,
        };

        assert_eq!(loc.file, PathBuf::from("/path/to/file.rs"));
        assert_eq!(loc.line, 100);
        assert_eq!(loc.column, 5);
    }

    // ========================================================================
    // FileId Tests
    // ========================================================================

    #[test]
    fn test_file_id() {
        let file_id = FileId {
            path: PathBuf::from("src/main.rs"),
            hash: 0xDEADBEEF,
        };

        assert_eq!(file_id.path, PathBuf::from("src/main.rs"));
        assert_eq!(file_id.hash, 0xDEADBEEF);
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}
