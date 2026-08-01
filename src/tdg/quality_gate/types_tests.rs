#[test]
fn test_gate_config_default() {
    let config = GateConfig::default();
    assert_eq!(config.max_score_drop, 5.0);
    assert!(!config.allow_grade_drop);
    assert!(config.enforce_new_files);
    assert_eq!(config.new_file_min_grade, Grade::B);
}

#[test]
fn test_gate_config_min_grades() {
    let config = GateConfig::default();
    assert_eq!(config.min_grades.get("rust"), Some(&Grade::BPlus));
    assert_eq!(config.min_grades.get("python"), Some(&Grade::B));
    assert_eq!(config.min_grades.get("unknown"), None);
}

#[test]
fn test_violation_type_equality() {
    assert_eq!(ViolationType::Regression, ViolationType::Regression);
    assert_eq!(ViolationType::BelowMinimum, ViolationType::BelowMinimum);
    assert_ne!(ViolationType::Regression, ViolationType::BelowMinimum);
}

#[test]
fn test_severity_ordering() {
    assert!(Severity::Info < Severity::Warning);
    assert!(Severity::Warning < Severity::Error);
    assert!(Severity::Error < Severity::Critical);
}

#[test]
fn test_severity_equality() {
    assert_eq!(Severity::Info, Severity::Info);
    assert_eq!(Severity::Critical, Severity::Critical);
}

#[test]
fn test_gate_result_creation() {
    let result = GateResult {
        passed: true,
        gate_name: "TestGate".to_string(),
        violations: vec![],
        message: "All checks passed".to_string(),
    };
    assert!(result.passed);
    assert_eq!(result.gate_name, "TestGate");
}

#[test]
fn test_gate_result_with_violations() {
    let violation = Violation {
        path: PathBuf::from("src/test.rs"),
        violation_type: ViolationType::Regression,
        severity: Severity::Error,
        message: "Score dropped".to_string(),
        old_score: Some(90.0),
        new_score: 75.0,
        old_grade: Some(Grade::A),
        new_grade: Grade::B,
    };
    let result = GateResult {
        passed: false,
        gate_name: "RegressionGate".to_string(),
        violations: vec![violation],
        message: "Regression detected".to_string(),
    };
    assert!(!result.passed);
    assert_eq!(result.violations.len(), 1);
}

#[test]
fn test_violation_creation() {
    let violation = Violation {
        path: PathBuf::from("src/lib.rs"),
        violation_type: ViolationType::NewFileBelowThreshold,
        severity: Severity::Warning,
        message: "New file below threshold".to_string(),
        old_score: None,
        new_score: 65.0,
        old_grade: None,
        new_grade: Grade::D,
    };
    assert_eq!(violation.path, PathBuf::from("src/lib.rs"));
    assert!(violation.old_score.is_none());
    assert_eq!(violation.new_score, 65.0);
}

#[test]
fn test_gate_config_clone() {
    let config = GateConfig::default();
    let cloned = config.clone();
    assert_eq!(cloned.max_score_drop, config.max_score_drop);
    assert_eq!(cloned.allow_grade_drop, config.allow_grade_drop);
}

#[test]
fn test_gate_result_clone() {
    let result = GateResult {
        passed: true,
        gate_name: "Test".to_string(),
        violations: vec![],
        message: "OK".to_string(),
    };
    let cloned = result.clone();
    assert_eq!(cloned.passed, result.passed);
    assert_eq!(cloned.gate_name, result.gate_name);
}

#[test]
fn test_violation_clone() {
    let violation = Violation {
        path: PathBuf::from("test.rs"),
        violation_type: ViolationType::Regression,
        severity: Severity::Error,
        message: "Test".to_string(),
        old_score: Some(90.0),
        new_score: 75.0,
        old_grade: Some(Grade::A),
        new_grade: Grade::B,
    };
    let cloned = violation.clone();
    assert_eq!(cloned.path, violation.path);
    assert_eq!(cloned.new_score, violation.new_score);
}

#[test]
fn test_violation_type_copy() {
    let vt = ViolationType::Regression;
    let copied = vt;
    assert_eq!(copied, ViolationType::Regression);
}

#[test]
fn test_severity_copy() {
    let s = Severity::Error;
    let copied = s;
    assert_eq!(copied, Severity::Error);
}

#[test]
fn test_gate_result_debug() {
    let result = GateResult {
        passed: true,
        gate_name: "Debug".to_string(),
        violations: vec![],
        message: "test".to_string(),
    };
    let debug_str = format!("{:?}", result);
    assert!(debug_str.contains("passed"));
    assert!(debug_str.contains("true"));
}

#[test]
fn test_violation_debug() {
    let violation = Violation {
        path: PathBuf::from("debug.rs"),
        violation_type: ViolationType::BelowMinimum,
        severity: Severity::Warning,
        message: "debug".to_string(),
        old_score: None,
        new_score: 60.0,
        old_grade: None,
        new_grade: Grade::D,
    };
    let debug_str = format!("{:?}", violation);
    assert!(debug_str.contains("debug.rs"));
}

#[test]
fn test_gate_config_debug() {
    let config = GateConfig::default();
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("max_score_drop"));
}

#[test]
fn test_gate_result_serialization() {
    let result = GateResult {
        passed: false,
        gate_name: "SerializeTest".to_string(),
        violations: vec![],
        message: "Test message".to_string(),
    };
    let json = serde_json::to_string(&result).unwrap();
    let deserialized: GateResult = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.passed, result.passed);
    assert_eq!(deserialized.gate_name, result.gate_name);
}

#[test]
fn test_violation_serialization() {
    let violation = Violation {
        path: PathBuf::from("serialize.rs"),
        violation_type: ViolationType::Regression,
        severity: Severity::Critical,
        message: "Critical regression".to_string(),
        old_score: Some(95.0),
        new_score: 60.0,
        old_grade: Some(Grade::APlus),
        new_grade: Grade::D,
    };
    let json = serde_json::to_string(&violation).unwrap();
    let deserialized: Violation = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.path, violation.path);
    assert_eq!(deserialized.old_score, Some(95.0));
    assert_eq!(deserialized.new_score, 60.0);
}

#[test]
fn test_gate_config_serialization() {
    let config = GateConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: GateConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.max_score_drop, config.max_score_drop);
    assert_eq!(deserialized.allow_grade_drop, config.allow_grade_drop);
    assert_eq!(deserialized.enforce_new_files, config.enforce_new_files);
}

#[test]
fn test_violation_type_serialization() {
    let types = vec![
        ViolationType::Regression,
        ViolationType::BelowMinimum,
        ViolationType::NewFileBelowThreshold,
    ];
    for vt in types {
        let json = serde_json::to_string(&vt).unwrap();
        let deserialized: ViolationType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, vt);
    }
}

#[test]
fn test_severity_serialization() {
    let severities = vec![
        Severity::Info,
        Severity::Warning,
        Severity::Error,
        Severity::Critical,
    ];
    for s in severities {
        let json = serde_json::to_string(&s).unwrap();
        let deserialized: Severity = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, s);
    }
}

#[test]
fn test_gate_result_with_multiple_violations_serialization() {
    let violations = vec![
        Violation {
            path: PathBuf::from("file1.rs"),
            violation_type: ViolationType::Regression,
            severity: Severity::Error,
            message: "Regression 1".to_string(),
            old_score: Some(85.0),
            new_score: 70.0,
            old_grade: Some(Grade::BPlus),
            new_grade: Grade::C,
        },
        Violation {
            path: PathBuf::from("file2.rs"),
            violation_type: ViolationType::BelowMinimum,
            severity: Severity::Warning,
            message: "Below minimum".to_string(),
            old_score: None,
            new_score: 65.0,
            old_grade: None,
            new_grade: Grade::D,
        },
    ];
    let result = GateResult {
        passed: false,
        gate_name: "MultiViolation".to_string(),
        violations: violations.clone(),
        message: "2 violations".to_string(),
    };
    let json = serde_json::to_string(&result).unwrap();
    let deserialized: GateResult = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.violations.len(), 2);
}

#[test]
fn test_violation_type_debug() {
    let vt = ViolationType::NewFileBelowThreshold;
    let debug_str = format!("{:?}", vt);
    assert!(debug_str.contains("NewFileBelowThreshold"));
}

#[test]
fn test_severity_debug() {
    let s = Severity::Critical;
    let debug_str = format!("{:?}", s);
    assert!(debug_str.contains("Critical"));
}

#[test]
fn test_gate_config_custom_min_grades() {
    let mut config = GateConfig::default();
    config.min_grades.insert("go".to_string(), Grade::A);
    config.min_grades.insert("c".to_string(), Grade::C);

    assert_eq!(config.min_grades.get("go"), Some(&Grade::A));
    assert_eq!(config.min_grades.get("c"), Some(&Grade::C));
}

#[test]
fn test_gate_config_custom_thresholds() {
    let config = GateConfig {
        max_score_drop: 10.0,
        allow_grade_drop: true,
        min_grades: HashMap::new(),
        default_min_grade: Grade::C,
        enforce_new_files: false,
        new_file_min_grade: Grade::D,
    };

    assert_eq!(config.max_score_drop, 10.0);
    assert!(config.allow_grade_drop);
    assert_eq!(config.default_min_grade, Grade::C);
    assert!(!config.enforce_new_files);
    assert_eq!(config.new_file_min_grade, Grade::D);
}

#[test]
fn test_violation_all_fields() {
    let violation = Violation {
        path: PathBuf::from("all_fields.rs"),
        violation_type: ViolationType::BelowMinimum,
        severity: Severity::Info,
        message: "Info level".to_string(),
        old_score: Some(100.0),
        new_score: 50.0,
        old_grade: Some(Grade::APlus),
        new_grade: Grade::F,
    };

    assert_eq!(violation.path, PathBuf::from("all_fields.rs"));
    assert_eq!(violation.violation_type, ViolationType::BelowMinimum);
    assert_eq!(violation.severity, Severity::Info);
    assert_eq!(violation.old_score, Some(100.0));
    assert_eq!(violation.new_score, 50.0);
    assert_eq!(violation.old_grade, Some(Grade::APlus));
    assert_eq!(violation.new_grade, Grade::F);
}

#[test]
fn test_violation_with_no_old_values() {
    let violation = Violation {
        path: PathBuf::from("new.rs"),
        violation_type: ViolationType::NewFileBelowThreshold,
        severity: Severity::Error,
        message: "New file".to_string(),
        old_score: None,
        new_score: 50.0,
        old_grade: None,
        new_grade: Grade::F,
    };

    assert!(violation.old_score.is_none());
    assert!(violation.old_grade.is_none());
}

#[test]
fn test_severity_full_ordering() {
    let severities = [
        Severity::Info,
        Severity::Warning,
        Severity::Error,
        Severity::Critical,
    ];
    for i in 0..severities.len() - 1 {
        assert!(severities[i] < severities[i + 1]);
    }
}

#[test]
fn test_violation_type_clone() {
    let vt = ViolationType::Regression;
    let cloned = vt;
    assert_eq!(cloned, vt);
}

#[test]
fn test_severity_clone() {
    let s = Severity::Warning;
    let cloned = s;
    assert_eq!(cloned, s);
}

#[test]
fn test_gate_result_message_content() {
    let result = GateResult {
        passed: true,
        gate_name: "Test".to_string(),
        violations: vec![],
        message: "\u{2705} All checks passed".to_string(),
    };
    assert!(result.message.contains("\u{2705}"));
    assert!(result.message.contains("passed"));
}

#[test]
fn test_gate_config_min_grades_iteration() {
    let config = GateConfig::default();
    assert!(config.min_grades.len() >= 4); // rust, typescript, python, javascript

    for lang in config.min_grades.keys() {
        assert!(!lang.is_empty());
    }
}

#[test]
fn test_violation_type_all_variants() {
    let variants = vec![
        ViolationType::Regression,
        ViolationType::BelowMinimum,
        ViolationType::NewFileBelowThreshold,
    ];

    for vt in variants {
        let cloned = vt;
        assert_eq!(cloned, vt);
    }
}

#[test]
fn test_severity_all_variants() {
    let variants = vec![
        Severity::Info,
        Severity::Warning,
        Severity::Error,
        Severity::Critical,
    ];

    for s in variants {
        let cloned = s;
        assert_eq!(cloned, s);
    }
}
