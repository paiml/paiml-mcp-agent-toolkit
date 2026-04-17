// ============================================================
// validate_module Tests
// ============================================================

fn create_temp_file(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(content.as_bytes()).unwrap();
    file.flush().unwrap();
    file
}

#[test]
fn test_validate_module_simple_passing() {
    let code = r#"
        fn simple() {
            let x = 1;
            let y = 2;
        }
    "#;
    let file = create_temp_file(code);

    // Use relaxed thresholds for this test
    let thresholds = QualityThresholds {
        min_entropy: 0.0, // Low entropy requirement
        satd_tolerance: 10,
        ..Default::default()
    };
    let runner = QualityGateRunner::new(thresholds);
    let result = runner.validate_module(file.path());
    assert!(result.is_ok());
    assert!(result.unwrap().passed);
}

#[test]
fn test_validate_module_file_not_found() {
    let runner = QualityGateRunner::strict();
    let result = runner.validate_module(Path::new("/nonexistent/file.rs"));
    assert!(result.is_err());
    match result.unwrap_err() {
        QualityViolation::ParseError(msg) => {
            assert!(msg.contains("No such file") || msg.contains("not found") || !msg.is_empty());
        }
        _ => panic!("Expected ParseError"),
    }
}

#[test]
fn test_validate_module_invalid_syntax() {
    let code = "fn invalid { missing parens and stuff |||";
    let file = create_temp_file(code);

    let runner = QualityGateRunner::strict();
    let result = runner.validate_module(file.path());
    assert!(result.is_err());
    match result.unwrap_err() {
        QualityViolation::ParseError(_) => {}
        _ => panic!("Expected ParseError for invalid syntax"),
    }
}

#[test]
fn test_validate_module_excessive_complexity() {
    // Code string with deeply nested ifs to trigger complexity violation
    let code = "fn complex(x: i32) { if x > 0 { if x > 10 { if x > 20 { if x > 30 { if x > 40 { if x > 50 { if x > 60 { if x > 70 { if x > 80 { if x > 90 { if x > 100 { println!(\"big\"); } } } } } } } } } } } }";
    let file = create_temp_file(code);

    let thresholds = QualityThresholds {
        max_cyclomatic: 5,
        min_entropy: 0.0,
        satd_tolerance: 100,
        ..Default::default()
    };
    let runner = QualityGateRunner::new(thresholds);
    let result = runner.validate_module(file.path());
    assert!(result.is_err());
    match result.unwrap_err() {
        QualityViolation::ExcessiveComplexity { found, max, .. } => {
            assert!(found > max);
        }
        _ => panic!("Expected ExcessiveComplexity violation"),
    }
}

#[test]
fn test_validate_module_satd_detected() {
    let code = r#"
        fn with_debt() {
            // TODO: implement this properly
            // FIXME: this is broken
            let x = 1;
        }
    "#;
    let file = create_temp_file(code);

    let thresholds = QualityThresholds {
        satd_tolerance: 0,
        min_entropy: 0.0,
        ..Default::default()
    };
    let runner = QualityGateRunner::new(thresholds);
    let result = runner.validate_module(file.path());
    assert!(result.is_err());
    match result.unwrap_err() {
        QualityViolation::SatdDetected {
            count, patterns, ..
        } => {
            assert!(count >= 2);
            assert!(!patterns.is_empty());
        }
        _ => panic!("Expected SatdDetected violation"),
    }
}

#[test]
fn test_validate_module_satd_with_tolerance() {
    let code = r#"
        fn with_some_debt() {
            // TODO: implement this properly
            let x = 1;
        }
    "#;
    let file = create_temp_file(code);

    let thresholds = QualityThresholds {
        satd_tolerance: 5,
        min_entropy: 0.0,
        ..Default::default()
    };
    let runner = QualityGateRunner::new(thresholds);
    let result = runner.validate_module(file.path());
    assert!(result.is_ok());
}

#[test]
fn test_validate_module_inefficient_algorithm() {
    let code = r#"
        fn inefficient(n: usize) {
            for i in 0..n {
                for j in 0..n {
                    for k in 0..n {
                        let _ = i + j + k;
                    }
                }
            }
        }
    "#;
    let file = create_temp_file(code);

    let thresholds = QualityThresholds {
        max_big_o: "O(n)".to_string(),
        min_entropy: 0.0,
        satd_tolerance: 100,
        ..Default::default()
    };
    let runner = QualityGateRunner::new(thresholds);
    let result = runner.validate_module(file.path());
    assert!(result.is_err());
    match result.unwrap_err() {
        QualityViolation::InefficientAlgorithm {
            complexity,
            required,
            ..
        } => {
            assert!(complexity.contains("n^"));
            assert_eq!(required, "O(n)");
        }
        _ => panic!("Expected InefficientAlgorithm violation"),
    }
}

#[test]
fn test_validate_module_insufficient_diversity() {
    let code = r#"
        fn repetitive() {
            let a = 1;
            let a = 1;
            let a = 1;
            let a = 1;
            let a = 1;
            let a = 1;
        }
    "#;
    let file = create_temp_file(code);

    let thresholds = QualityThresholds {
        min_entropy: 5.0,
        satd_tolerance: 100,
        ..Default::default()
    };
    let runner = QualityGateRunner::new(thresholds);
    let result = runner.validate_module(file.path());
    assert!(result.is_err());
    match result.unwrap_err() {
        QualityViolation::InsufficientDiversity { entropy, required } => {
            assert!(entropy < required);
            assert!((required - 5.0).abs() < 0.001);
        }
        _ => panic!("Expected InsufficientDiversity violation"),
    }
}

#[test]
fn test_validate_module_empty_file() {
    let code = "";
    let file = create_temp_file(code);

    let thresholds = QualityThresholds {
        min_entropy: 0.0,
        ..Default::default()
    };
    let runner = QualityGateRunner::new(thresholds);
    let result = runner.validate_module(file.path());
    assert!(result.is_ok());
}

include!("gate_tests_validation_part2.rs");
