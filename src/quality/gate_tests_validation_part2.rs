// ============================================================
// validate_module Tests - Part 2 (diverse code, SATD, integration)
// ============================================================

#[test]
fn test_validate_module_diverse_code() {
    let code = r#"
        use std::collections::HashMap;

        struct Point {
            x: f64,
            y: f64,
        }

        impl Point {
            fn new(x: f64, y: f64) -> Self {
                Self { x, y }
            }

            fn distance(&self, other: &Point) -> f64 {
                let dx = self.x - other.x;
                let dy = self.y - other.y;
                (dx * dx + dy * dy).sqrt()
            }
        }

        enum Shape {
            Circle { radius: f64 },
            Rectangle { width: f64, height: f64 },
        }

        fn calculate_area(shape: &Shape) -> f64 {
            match shape {
                Shape::Circle { radius } => std::f64::consts::PI * radius * radius,
                Shape::Rectangle { width, height } => width * height,
            }
        }
    "#;
    let file = create_temp_file(code);

    let thresholds = QualityThresholds {
        min_entropy: 3.0, // Reasonable entropy
        satd_tolerance: 100,
        max_cyclomatic: 20,
        max_big_o: "O(n^2)".to_string(),
        ..Default::default()
    };
    let runner = QualityGateRunner::new(thresholds);
    let result = runner.validate_module(file.path());
    assert!(result.is_ok());
}

// ============================================================
// SatdResult Tests
// ============================================================

#[test]
fn test_satd_result_empty() {
    let result = SatdResult {
        count: 0,
        patterns: vec![],
    };
    assert_eq!(result.count, 0);
    assert!(result.patterns.is_empty());
}

#[test]
fn test_satd_result_with_patterns() {
    let result = SatdResult {
        count: 3,
        patterns: vec!["TODO".to_string(), "FIXME".to_string()],
    };
    assert_eq!(result.count, 3);
    assert_eq!(result.patterns.len(), 2);
}

#[test]
fn test_satd_result_clone() {
    let original = SatdResult {
        count: 5,
        patterns: vec!["HACK".to_string()],
    };
    let cloned = original.clone();
    assert_eq!(original.count, cloned.count);
    assert_eq!(original.patterns, cloned.patterns);
}

#[test]
fn test_satd_result_serialization() {
    let result = SatdResult {
        count: 2,
        patterns: vec!["TODO".to_string(), "XXX".to_string()],
    };
    let json = serde_json::to_string(&result).unwrap();
    let deserialized: SatdResult = serde_json::from_str(&json).unwrap();
    assert_eq!(result.count, deserialized.count);
    assert_eq!(result.patterns, deserialized.patterns);
}

#[test]
fn test_satd_result_debug() {
    let result = SatdResult {
        count: 1,
        patterns: vec!["DEPRECATED".to_string()],
    };
    let debug = format!("{:?}", result);
    assert!(debug.contains("SatdResult"));
    assert!(debug.contains("count"));
}

// ============================================================
// Integration Tests
// ============================================================

#[test]
fn test_full_quality_gate_workflow() {
    let code = r#"
        fn fibonacci(n: u64) -> u64 {
            if n <= 1 {
                n
            } else {
                fibonacci(n - 1) + fibonacci(n - 2)
            }
        }

        fn factorial(n: u64) -> u64 {
            if n == 0 {
                1
            } else {
                n * factorial(n - 1)
            }
        }
    "#;
    let file = create_temp_file(code);

    let thresholds = QualityThresholds {
        max_cyclomatic: 10,
        min_entropy: 2.5,
        satd_tolerance: 0,
        max_big_o: "O(n log n)".to_string(),
        ..Default::default()
    };
    let runner = QualityGateRunner::new(thresholds);
    let result = runner.validate_module(file.path());
    assert!(result.is_ok());
}

#[test]
fn test_quality_gate_all_checks_fail() {
    let code = r#"
        fn terrible_code() {
            // TODO: fix everything
            // FIXME: this is bad
            // HACK: temporary workaround
            for i in 0..n {
                for j in 0..n {
                    for k in 0..n {
                        if a && b && c && d && e && f && g && h {
                            println!("wow");
                        }
                    }
                }
            }
        }
    "#;
    let file = create_temp_file(code);

    let runner = QualityGateRunner::strict();
    let result = runner.validate_module(file.path());
    assert!(result.is_err());
}

#[test]
fn test_quality_thresholds_boundary_values() {
    let thresholds = QualityThresholds {
        max_cyclomatic: 1,
        max_cognitive: 0,
        max_nesting: 0,
        max_params: 0,
        max_lines: 1,
        satd_tolerance: 0,
        max_big_o: "O(1)".to_string(),
        min_entropy: 10.0,
    };

    let code = "fn empty() {}";
    let file = create_temp_file(code);

    let runner = QualityGateRunner::new(thresholds);
    let _result = runner.validate_module(file.path());
}

#[test]
fn test_quality_gate_with_struct_only() {
    let code = r#"
        struct Config {
            name: String,
            value: i32,
            enabled: bool,
        }
    "#;
    let file = create_temp_file(code);

    let thresholds = QualityThresholds {
        min_entropy: 2.0,
        satd_tolerance: 100,
        ..Default::default()
    };
    let runner = QualityGateRunner::new(thresholds);
    let result = runner.validate_module(file.path());
    assert!(result.is_ok());
}

#[test]
fn test_quality_gate_with_match_expression() {
    let code = r#"
        fn process(cmd: Command) -> Result<(), Error> {
            match cmd {
                Command::Start => start_service(),
                Command::Stop => stop_service(),
                Command::Restart => restart_service(),
                Command::Status => check_status(),
            }
        }
    "#;
    let file = create_temp_file(code);

    let thresholds = QualityThresholds {
        max_cyclomatic: 10,
        min_entropy: 2.0,
        satd_tolerance: 100,
        ..Default::default()
    };
    let runner = QualityGateRunner::new(thresholds);
    let result = runner.validate_module(file.path());
    assert!(result.is_ok());
}
