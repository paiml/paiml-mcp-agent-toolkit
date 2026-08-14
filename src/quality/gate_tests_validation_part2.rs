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

// ============================================================
// #973: the analyzer registry / measured-vs-default metrics
// ============================================================

#[test]
fn test_quality_gate_runner_registers_analyzers() {
    // The registry was `_analyzers: vec![]` behind a "TODO: Fix analyzer trait
    // implementations" comment: a gate runner that ran no analyzer.
    let runner = QualityGateRunner::strict();
    assert!(
        !runner.analyzers.is_empty(),
        "QualityGateRunner registered no analyzers"
    );
}

#[test]
fn test_validate_module_reports_measured_metrics_not_defaults() {
    // RED before the fix: validate_module returned QualityReport::passed(),
    // whose metrics are QualityMetrics::default() — cyclomatic 1, cognitive 0,
    // nesting 0, satd 0, entropy 0.0, "O(1)" — for EVERY module that passed.
    // Those are constants, not measurements of this file.
    let code = r#"
        // TODO: this one is deliberate
        // FIXME: so is this
        fn branchy(n: usize, flag: bool) -> usize {
            let mut total = 0;
            for i in 0..n {
                for j in 0..n {
                    if flag && i > j {
                        total += i * j;
                    } else if i == j {
                        total += 1;
                    }
                }
            }
            total
        }
    "#;
    let file = create_temp_file(code);

    let thresholds = QualityThresholds {
        max_cyclomatic: 100,
        satd_tolerance: 100,
        min_entropy: 0.0,
        max_big_o: "O(2^n)".to_string(),
        ..Default::default()
    };
    let runner = QualityGateRunner::new(thresholds);
    let report = runner.validate_module(file.path()).expect("should pass");

    assert!(report.passed);
    let d = QualityMetrics::default();
    assert!(
        report.metrics.cyclomatic_complexity > d.cyclomatic_complexity,
        "cyclomatic was {} (default is {})",
        report.metrics.cyclomatic_complexity,
        d.cyclomatic_complexity
    );
    assert!(
        report.metrics.entropy > 0.0,
        "entropy was {}",
        report.metrics.entropy
    );
    assert_eq!(
        report.metrics.satd_count, 2,
        "the two SATD markers in the fixture must be counted"
    );
    assert_ne!(
        report.metrics.efficiency, "O(1)",
        "a doubly nested loop is not O(1)"
    );
}

#[test]
fn test_validate_module_metrics_differ_between_modules() {
    // Differential control: a trivial module and a complex one must not
    // report the same numbers. Identical output for both is the signature of
    // a constant standing in for a measurement.
    let trivial = create_temp_file("fn a() {}\n");
    let complex = create_temp_file(
        r#"
        fn b(n: usize) -> usize {
            let mut t = 0;
            for i in 0..n {
                for j in 0..n {
                    if i > j { t += 1; } else if i == j { t += 2; } else { t += 3; }
                }
            }
            t
        }
    "#,
    );

    let thresholds = QualityThresholds {
        max_cyclomatic: 100,
        satd_tolerance: 100,
        min_entropy: 0.0,
        max_big_o: "O(2^n)".to_string(),
        ..Default::default()
    };
    let runner = QualityGateRunner::new(thresholds);
    let a = runner.validate_module(trivial.path()).unwrap().metrics;
    let b = runner.validate_module(complex.path()).unwrap().metrics;

    assert_ne!(
        (a.cyclomatic_complexity, a.efficiency.as_str()),
        (b.cyclomatic_complexity, b.efficiency.as_str()),
        "both modules reported the same metrics"
    );
}
