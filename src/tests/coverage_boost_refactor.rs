//! Coverage boost tests for models/refactor.rs
//! Tests: state machine, configs, types, serde, defaults, advance logic

use crate::models::refactor::{
    BytePos, DefectPayload, FileId, Location, MetricSet, NestingStrategy, RefactorConfig,
    RefactorOp, RefactorStateMachine, RefactorStrategies, RefactorType, SatdFix, Severity, State,
    Summary, Thresholds, Violation, ViolationType,
};
use std::path::PathBuf;
use std::time::Duration;

// ============ RefactorConfig Tests ============

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
fn test_refactor_config_serde() {
    let config = RefactorConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    let back: RefactorConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(back.target_complexity, 20);
    assert!(back.remove_satd);
}

// ============ Thresholds Tests ============

#[test]
fn test_thresholds_default() {
    let t = Thresholds::default();
    assert_eq!(t.cyclomatic_warn, 10);
    assert_eq!(t.cyclomatic_error, 20);
    assert_eq!(t.cognitive_warn, 15);
    assert_eq!(t.cognitive_error, 30);
    assert!((t.tdg_warn - 1.5).abs() < f32::EPSILON);
    assert!((t.tdg_error - 2.0).abs() < f32::EPSILON);
}

#[test]
fn test_thresholds_serde() {
    let t = Thresholds::default();
    let json = serde_json::to_string(&t).unwrap();
    let back: Thresholds = serde_json::from_str(&json).unwrap();
    assert_eq!(back.cyclomatic_warn, t.cyclomatic_warn);
}

// ============ RefactorStrategies Tests ============

#[test]
fn test_refactor_strategies_default() {
    let s = RefactorStrategies::default();
    assert!(s.prefer_functional);
    assert!(s.use_early_returns);
    assert!(s.extract_helpers);
}

#[test]
fn test_refactor_strategies_serde() {
    let s = RefactorStrategies::default();
    let json = serde_json::to_string(&s).unwrap();
    let back: RefactorStrategies = serde_json::from_str(&json).unwrap();
    assert!(back.prefer_functional);
}

// ============ MetricSet Tests ============

#[test]
fn test_metric_set_default() {
    let m = MetricSet::default();
    assert_eq!(m.complexity, (0, 0));
    assert_eq!(m.tdg_score, 0.0);
    assert!(m.dead_code.is_empty());
    assert_eq!(m.satd_count, 0);
    assert_eq!(m.provability, 0.0);
}

#[test]
fn test_metric_set_serde() {
    let m = MetricSet {
        complexity: (10, 15),
        tdg_score: 1.5,
        dead_code: vec![true, false],
        satd_count: 3,
        provability: 0.8,
    };
    let json = serde_json::to_string(&m).unwrap();
    let back: MetricSet = serde_json::from_str(&json).unwrap();
    assert_eq!(back.complexity, (10, 15));
    assert_eq!(back.satd_count, 3);
}

// ============ Summary Tests ============

#[test]
fn test_summary_default() {
    let s = Summary::default();
    assert_eq!(s.files_processed, 0);
    assert_eq!(s.refactors_applied, 0);
    assert_eq!(s.complexity_reduction, 0.0);
    assert_eq!(s.satd_removed, 0);
    assert_eq!(s.total_time, Duration::from_secs(0));
}

#[test]
fn test_summary_serde() {
    let s = Summary {
        files_processed: 10,
        refactors_applied: 5,
        complexity_reduction: 0.3,
        satd_removed: 2,
        total_time: Duration::from_secs(120),
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: Summary = serde_json::from_str(&json).unwrap();
    assert_eq!(back.files_processed, 10);
    assert_eq!(back.refactors_applied, 5);
}

// ============ Severity Tests ============

#[test]
fn test_severity_variants_serde() {
    let severities = vec![
        Severity::Low,
        Severity::Medium,
        Severity::High,
        Severity::Critical,
    ];
    for sev in &severities {
        let json = serde_json::to_string(sev).unwrap();
        let back: Severity = serde_json::from_str(&json).unwrap();
        let _ = format!("{:?}", back);
    }
}

// ============ ViolationType Tests ============

#[test]
fn test_violation_type_variants_serde() {
    let types = vec![
        ViolationType::HighComplexity,
        ViolationType::DeepNesting,
        ViolationType::LongFunction,
        ViolationType::SelfAdmittedTechDebt,
        ViolationType::DeadCode,
        ViolationType::PoorNaming,
    ];
    for vt in &types {
        let json = serde_json::to_string(vt).unwrap();
        let back: ViolationType = serde_json::from_str(&json).unwrap();
        let _ = format!("{:?}", back);
    }
}

// ============ NestingStrategy Tests ============

#[test]
fn test_nesting_strategy_variants_serde() {
    let strategies = vec![
        NestingStrategy::EarlyReturn,
        NestingStrategy::ExtractCondition,
        NestingStrategy::GuardClause,
        NestingStrategy::StreamChain,
    ];
    for s in &strategies {
        let json = serde_json::to_string(s).unwrap();
        let back: NestingStrategy = serde_json::from_str(&json).unwrap();
        let _ = format!("{:?}", back);
    }
}

// ============ SatdFix Tests ============

#[test]
fn test_satd_fix_remove() {
    let fix = SatdFix::Remove;
    let json = serde_json::to_string(&fix).unwrap();
    let back: SatdFix = serde_json::from_str(&json).unwrap();
    let _ = format!("{:?}", back);
}

#[test]
fn test_satd_fix_replace() {
    let fix = SatdFix::Replace {
        with: "improved code".to_string(),
    };
    let json = serde_json::to_string(&fix).unwrap();
    let back: SatdFix = serde_json::from_str(&json).unwrap();
    let _ = format!("{:?}", back);
}

#[test]
fn test_satd_fix_implement() {
    let fix = SatdFix::Implement {
        solution: "add proper error handling".to_string(),
    };
    let json = serde_json::to_string(&fix).unwrap();
    let back: SatdFix = serde_json::from_str(&json).unwrap();
    let _ = format!("{:?}", back);
}

// ============ RefactorOp Tests ============

#[test]
fn test_refactor_op_extract_function() {
    let op = RefactorOp::ExtractFunction {
        name: "helper".to_string(),
        start: BytePos { byte: 100, line: 10, column: 1 },
        end: BytePos { byte: 200, line: 20, column: 1 },
        params: vec!["x".to_string(), "y".to_string()],
    };
    let json = serde_json::to_string(&op).unwrap();
    let back: RefactorOp = serde_json::from_str(&json).unwrap();
    let _ = format!("{:?}", back);
}

#[test]
fn test_refactor_op_flatten_nesting() {
    let op = RefactorOp::FlattenNesting {
        function: "complex_fn".to_string(),
        strategy: NestingStrategy::GuardClause,
    };
    let json = serde_json::to_string(&op).unwrap();
    let _ = format!("{:?}", op);
    let back: RefactorOp = serde_json::from_str(&json).unwrap();
    let _ = format!("{:?}", back);
}

#[test]
fn test_refactor_op_replace_hashmap() {
    let op = RefactorOp::ReplaceHashMap {
        imports: vec!["use rustc_hash::FxHashMap".to_string()],
        replacements: vec![("HashMap".to_string(), "FxHashMap".to_string())],
    };
    let json = serde_json::to_string(&op).unwrap();
    let back: RefactorOp = serde_json::from_str(&json).unwrap();
    let _ = format!("{:?}", back);
}

#[test]
fn test_refactor_op_remove_satd() {
    let op = RefactorOp::RemoveSatd {
        location: Location { file: PathBuf::from("test.rs"), line: 42, column: 1 },
        fix: SatdFix::Remove,
    };
    let json = serde_json::to_string(&op).unwrap();
    let back: RefactorOp = serde_json::from_str(&json).unwrap();
    let _ = format!("{:?}", back);
}

#[test]
fn test_refactor_op_simplify_expression() {
    let op = RefactorOp::SimplifyExpression {
        expr: "if x == true { true } else { false }".to_string(),
        simplified: "x".to_string(),
    };
    let json = serde_json::to_string(&op).unwrap();
    let back: RefactorOp = serde_json::from_str(&json).unwrap();
    let _ = format!("{:?}", back);
}

// ============ RefactorType Tests ============

#[test]
fn test_refactor_type_variants() {
    let types = vec![
        RefactorType::None,
        RefactorType::ExtractFunction,
        RefactorType::FlattenNesting,
        RefactorType::SimplifyLogic,
        RefactorType::RemoveDeadCode,
    ];
    for rt in &types {
        let json = serde_json::to_string(rt).unwrap();
        let back: RefactorType = serde_json::from_str(&json).unwrap();
        let _ = format!("{:?}", back);
    }
}

// ============ BytePos Tests ============

#[test]
fn test_byte_pos_construction() {
    let pos = BytePos { byte: 100, line: 10, column: 5 };
    let json = serde_json::to_string(&pos).unwrap();
    let back: BytePos = serde_json::from_str(&json).unwrap();
    assert_eq!(back.byte, 100);
    assert_eq!(back.line, 10);
}

// ============ FileId Tests ============

#[test]
fn test_file_id_serde() {
    let fid = FileId {
        path: PathBuf::from("src/main.rs"),
        hash: 12345,
    };
    let json = serde_json::to_string(&fid).unwrap();
    let back: FileId = serde_json::from_str(&json).unwrap();
    assert_eq!(back.hash, 12345);
}

// ============ Location Tests ============

#[test]
fn test_refactor_location_serde() {
    let loc = Location {
        file: PathBuf::from("src/lib.rs"),
        line: 42,
        column: 8,
    };
    let json = serde_json::to_string(&loc).unwrap();
    let back: Location = serde_json::from_str(&json).unwrap();
    assert_eq!(back.line, 42);
}

// ============ Violation Tests ============

#[test]
fn test_violation_serde() {
    let v = Violation {
        violation_type: ViolationType::HighComplexity,
        location: Location { file: PathBuf::from("test.rs"), line: 10, column: 1 },
        severity: Severity::High,
        description: "Too complex".to_string(),
        suggested_fix: None,
    };
    let json = serde_json::to_string(&v).unwrap();
    let back: Violation = serde_json::from_str(&json).unwrap();
    assert_eq!(back.description, "Too complex");
}

#[test]
fn test_violation_to_op_high_complexity() {
    let v = Violation {
        violation_type: ViolationType::HighComplexity,
        location: Location { file: PathBuf::from("test.rs"), line: 10, column: 1 },
        severity: Severity::High,
        description: "complex".to_string(),
        suggested_fix: None,
    };
    let op = v.to_op();
    assert!(matches!(op, RefactorOp::ExtractFunction { .. }));
}

#[test]
fn test_violation_to_op_deep_nesting() {
    let v = Violation {
        violation_type: ViolationType::DeepNesting,
        location: Location { file: PathBuf::from("test.rs"), line: 10, column: 1 },
        severity: Severity::Medium,
        description: "nested".to_string(),
        suggested_fix: None,
    };
    let op = v.to_op();
    assert!(matches!(op, RefactorOp::FlattenNesting { .. }));
}

#[test]
fn test_violation_to_op_satd() {
    let v = Violation {
        violation_type: ViolationType::SelfAdmittedTechDebt,
        location: Location { file: PathBuf::from("test.rs"), line: 10, column: 1 },
        severity: Severity::Low,
        description: "TODO".to_string(),
        suggested_fix: None,
    };
    let op = v.to_op();
    assert!(matches!(op, RefactorOp::RemoveSatd { .. }));
}

#[test]
fn test_violation_to_op_with_suggested_fix() {
    let v = Violation {
        violation_type: ViolationType::LongFunction,
        location: Location { file: PathBuf::from("test.rs"), line: 10, column: 1 },
        severity: Severity::Medium,
        description: "too long".to_string(),
        suggested_fix: Some(RefactorOp::SimplifyExpression {
            expr: "a".to_string(),
            simplified: "b".to_string(),
        }),
    };
    let op = v.to_op();
    assert!(matches!(op, RefactorOp::SimplifyExpression { .. }));
}

// ============ DefectPayload Tests ============

#[test]
fn test_defect_payload_serde() {
    let payload = DefectPayload {
        file_hash: 12345,
        tdg_score: 1.5,
        complexity: (10, 15),
        dead_symbols: 2,
        timestamp: 1000000,
        severity_flags: 0b00000001,
        refactor_available: true,
        refactor_type: RefactorType::ExtractFunction,
        estimated_improvement: 0.3,
        _padding: [0; 2],
    };
    let json = serde_json::to_string(&payload).unwrap();
    let back: DefectPayload = serde_json::from_str(&json).unwrap();
    assert_eq!(back.file_hash, 12345);
    assert!(back.refactor_available);
}

#[test]
fn test_defect_payload_copy() {
    let payload = DefectPayload {
        file_hash: 0,
        tdg_score: 0.0,
        complexity: (0, 0),
        dead_symbols: 0,
        timestamp: 0,
        severity_flags: 0,
        refactor_available: false,
        refactor_type: RefactorType::None,
        estimated_improvement: 0.0,
        _padding: [0; 2],
    };
    let copied = payload;
    assert_eq!(copied.file_hash, 0);
}

// ============ State Machine Tests ============

#[test]
fn test_state_machine_new_with_targets() {
    let targets = vec![PathBuf::from("src/main.rs")];
    let sm = RefactorStateMachine::new(targets.clone(), RefactorConfig::default());
    assert!(matches!(sm.current, State::Scan { .. }));
    assert!(sm.history.is_empty());
    assert_eq!(sm.targets.len(), 1);
}

#[test]
fn test_state_machine_new_empty_targets() {
    let sm = RefactorStateMachine::new(vec![], RefactorConfig::default());
    assert!(matches!(sm.current, State::Complete { .. }));
}

#[test]
fn test_state_machine_advance_from_scan() {
    let targets = vec![PathBuf::from("src/main.rs")];
    let mut sm = RefactorStateMachine::new(targets, RefactorConfig::default());
    let result = sm.advance();
    assert!(result.is_ok());
    assert!(matches!(sm.current, State::Analyze { .. }));
}

#[test]
fn test_state_machine_advance_multiple() {
    let targets = vec![PathBuf::from("src/simple.rs")];
    let mut sm = RefactorStateMachine::new(targets, RefactorConfig::default());

    // Scan -> Analyze
    sm.advance().unwrap();
    assert!(matches!(sm.current, State::Analyze { .. }));

    // Analyze -> Plan (no violations for "simple.rs")
    sm.advance().unwrap();
    assert!(matches!(sm.current, State::Plan { .. }));
}

#[test]
fn test_state_machine_advance_complete_no_violations() {
    let targets = vec![PathBuf::from("src/simple.rs")];
    let mut sm = RefactorStateMachine::new(targets, RefactorConfig::default());

    // Scan -> Analyze -> Plan -> Complete (no violations, single file)
    sm.advance().unwrap(); // Scan -> Analyze
    sm.advance().unwrap(); // Analyze -> Plan
    sm.advance().unwrap(); // Plan -> Complete (no violations, no more targets)
    assert!(matches!(sm.current, State::Complete { .. }));
}

#[test]
fn test_state_machine_advance_with_violations() {
    let targets = vec![PathBuf::from("src/complex_module.rs")];
    let mut sm = RefactorStateMachine::new(targets, RefactorConfig::default());

    sm.advance().unwrap(); // Scan -> Analyze
    sm.advance().unwrap(); // Analyze -> Plan (finds violations for "complex")
    sm.advance().unwrap(); // Plan -> Refactor
    assert!(matches!(sm.current, State::Refactor { .. }));

    sm.advance().unwrap(); // Refactor -> Test
    assert!(matches!(sm.current, State::Test { .. }));

    sm.advance().unwrap(); // Test -> Lint
    assert!(matches!(sm.current, State::Lint { .. }));
}

#[test]
fn test_state_machine_serde() {
    let sm = RefactorStateMachine::new(vec![PathBuf::from("a.rs")], RefactorConfig::default());
    let json = serde_json::to_string(&sm).unwrap();
    let back: RefactorStateMachine = serde_json::from_str(&json).unwrap();
    assert_eq!(back.targets.len(), 1);
}

// ============ State Tests ============

#[test]
fn test_state_scan_serde() {
    let state = State::Scan { targets: vec![PathBuf::from("a.rs")] };
    let json = serde_json::to_string(&state).unwrap();
    let back: State = serde_json::from_str(&json).unwrap();
    let _ = format!("{:?}", back);
}

#[test]
fn test_state_complete_serde() {
    let state = State::Complete { summary: Summary::default() };
    let json = serde_json::to_string(&state).unwrap();
    let back: State = serde_json::from_str(&json).unwrap();
    let _ = format!("{:?}", back);
}
