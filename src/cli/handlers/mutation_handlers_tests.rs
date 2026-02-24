// ==================== MutationTestConfig Tests ====================

#[test]
fn test_mutation_test_config_new_defaults() {
    let config = MutationTestConfig::new(None, false, false, 4, true, None, false, None, 100);

    assert!(config.operators.is_none());
    assert!(!config.ml_predict);
    assert!(!config.distributed);
    assert_eq!(config.workers, 4);
    assert!(config.progress);
    assert!(config.min_score.is_none());
    assert!(!config.ci_learning);
    assert!(config.ci_provider.is_none());
    assert_eq!(config.auto_train_threshold, 100);
}

#[test]
fn test_mutation_test_config_new_with_operators() {
    let operators = vec!["AOR".to_string(), "ROR".to_string()];
    let config = MutationTestConfig::new(
        Some(operators.clone()),
        false,
        false,
        4,
        true,
        None,
        false,
        None,
        100,
    );

    assert_eq!(config.operators, Some(operators));
}

#[test]
fn test_mutation_test_config_new_with_ml_predict() {
    let config = MutationTestConfig::new(None, true, false, 4, true, None, false, None, 100);

    assert!(config.ml_predict);
}

#[test]
fn test_mutation_test_config_new_distributed() {
    let config = MutationTestConfig::new(None, false, true, 8, true, None, false, None, 100);

    assert!(config.distributed);
    assert_eq!(config.workers, 8);
}

#[test]
fn test_mutation_test_config_new_with_min_score() {
    let config =
        MutationTestConfig::new(None, false, false, 4, true, Some(0.8), false, None, 100);

    assert_eq!(config.min_score, Some(0.8));
}

#[test]
fn test_mutation_test_config_new_ci_learning() {
    let config = MutationTestConfig::new(
        None,
        false,
        false,
        4,
        true,
        None,
        true,
        Some("github".to_string()),
        50,
    );

    assert!(config.ci_learning);
    assert_eq!(config.ci_provider, Some("github".to_string()));
    assert_eq!(config.auto_train_threshold, 50);
}

#[test]
fn test_mutation_test_config_debug() {
    let config = MutationTestConfig::new(None, false, false, 4, true, None, false, None, 100);
    let debug = format!("{:?}", config);
    assert!(debug.contains("MutationTestConfig"));
}

#[test]
fn test_mutation_test_config_clone() {
    let config = MutationTestConfig::new(
        Some(vec!["AOR".to_string()]),
        true,
        true,
        8,
        false,
        Some(0.9),
        true,
        Some("gitlab".to_string()),
        200,
    );
    let cloned = config.clone();

    assert_eq!(config.operators, cloned.operators);
    assert_eq!(config.ml_predict, cloned.ml_predict);
    assert_eq!(config.workers, cloned.workers);
    assert_eq!(config.min_score, cloned.min_score);
}

// ==================== validate_path Tests ====================

#[test]
fn test_validate_path_exists() {
    use std::fs;
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let file = dir.path().join("test.rs");
    fs::write(&file, "fn main() {}").unwrap();

    let result = validate_path(&file);
    assert!(result.is_ok());
}

#[test]
fn test_validate_path_not_exists() {
    let path = PathBuf::from("/nonexistent/path/to/file.rs");
    let result = validate_path(&path);

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("does not exist"));
}

#[test]
fn test_validate_path_directory() {
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let result = validate_path(&dir.path().to_path_buf());

    // Directories exist too
    assert!(result.is_ok());
}

// ==================== validate_score_threshold Tests ====================

#[test]
fn test_validate_score_threshold_no_minimum() {
    let score = MutationScore {
        score: 0.5,
        total: 10,
        killed: 5,
        survived: 5,
        compile_errors: 0,
        timeouts: 0,
        equivalent: 0,
    };

    let result = validate_score_threshold(&score, None);
    assert!(result.is_ok());
}

#[test]
fn test_validate_score_threshold_above_minimum() {
    let score = MutationScore {
        score: 0.85,
        total: 10,
        killed: 9,
        survived: 1,
        compile_errors: 0,
        timeouts: 0,
        equivalent: 0,
    };

    let result = validate_score_threshold(&score, Some(0.8));
    assert!(result.is_ok());
}

#[test]
fn test_validate_score_threshold_below_minimum() {
    let score = MutationScore {
        score: 0.5,
        total: 10,
        killed: 5,
        survived: 5,
        compile_errors: 0,
        timeouts: 0,
        equivalent: 0,
    };

    let result = validate_score_threshold(&score, Some(0.8));
    assert!(result.is_err());

    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("below threshold"));
}

#[test]
fn test_validate_score_threshold_exact_minimum() {
    let score = MutationScore {
        score: 0.8,
        total: 10,
        killed: 8,
        survived: 2,
        compile_errors: 0,
        timeouts: 0,
        equivalent: 0,
    };

    let result = validate_score_threshold(&score, Some(0.8));
    assert!(result.is_ok());
}

// ==================== format_summary_report Tests ====================

#[test]
fn test_format_summary_report() {
    let score = MutationScore {
        score: 0.75,
        total: 20,
        killed: 15,
        survived: 5,
        compile_errors: 0,
        timeouts: 0,
        equivalent: 0,
    };

    let report = format_summary_report(&score);

    assert!(report["summary"].as_str().unwrap().contains("75.00%"));
    assert!(report["summary"].as_str().unwrap().contains("15/20"));
    assert!(report["breakdown"].as_str().unwrap().contains("Killed: 15"));
    assert!(report["breakdown"]
        .as_str()
        .unwrap()
        .contains("Survived: 5"));
}

#[test]
fn test_format_summary_report_zero_score() {
    let score = MutationScore {
        score: 0.0,
        total: 10,
        killed: 0,
        survived: 10,
        compile_errors: 0,
        timeouts: 0,
        equivalent: 0,
    };

    let report = format_summary_report(&score);

    assert!(report["summary"].as_str().unwrap().contains("0.00%"));
}

#[test]
fn test_format_summary_report_perfect_score() {
    let score = MutationScore {
        score: 1.0,
        total: 10,
        killed: 10,
        survived: 0,
        compile_errors: 0,
        timeouts: 0,
        equivalent: 0,
    };

    let report = format_summary_report(&score);

    assert!(report["summary"].as_str().unwrap().contains("100.00%"));
}

#[test]
fn test_format_summary_report_with_errors() {
    let score = MutationScore {
        score: 0.6,
        total: 10,
        killed: 6,
        survived: 2,
        compile_errors: 1,
        timeouts: 1,
        equivalent: 0,
    };

    let report = format_summary_report(&score);

    assert!(report["breakdown"]
        .as_str()
        .unwrap()
        .contains("Compile Errors: 1"));
    assert!(report["breakdown"]
        .as_str()
        .unwrap()
        .contains("Timeouts: 1"));
}

// ==================== format_json_report Tests ====================

#[test]
fn test_format_json_report_structure() {
    let score = MutationScore {
        score: 0.8,
        total: 10,
        killed: 8,
        survived: 2,
        compile_errors: 0,
        timeouts: 0,
        equivalent: 0,
    };

    let report = format_json_report(&score, &[], &None);

    assert_eq!(report["mutation_score"], 0.8);
    assert_eq!(report["total_mutants"], 10);
    assert_eq!(report["killed"], 8);
    assert_eq!(report["survived"], 2);
    assert!(report["operators"].is_array());
}

#[test]
fn test_format_json_report_default_operators() {
    let score = MutationScore {
        score: 0.5,
        total: 10,
        killed: 5,
        survived: 5,
        compile_errors: 0,
        timeouts: 0,
        equivalent: 0,
    };

    let report = format_json_report(&score, &[], &None);
    let operators = report["operators"].as_array().unwrap();

    assert!(operators.contains(&serde_json::json!("AOR")));
    assert!(operators.contains(&serde_json::json!("ROR")));
    assert!(operators.contains(&serde_json::json!("COR")));
    assert!(operators.contains(&serde_json::json!("UOR")));
}

#[test]
fn test_format_json_report_custom_operators() {
    let score = MutationScore {
        score: 0.5,
        total: 10,
        killed: 5,
        survived: 5,
        compile_errors: 0,
        timeouts: 0,
        equivalent: 0,
    };

    let operators = Some(vec!["AOR".to_string(), "BoolLit".to_string()]);
    let report = format_json_report(&score, &[], &operators);
    let ops = report["operators"].as_array().unwrap();

    assert!(ops.contains(&serde_json::json!("AOR")));
    assert!(ops.contains(&serde_json::json!("BoolLit")));
}

#[test]
fn test_format_json_report_empty_results() {
    let score = MutationScore {
        score: 0.0,
        total: 0,
        killed: 0,
        survived: 0,
        compile_errors: 0,
        timeouts: 0,
        equivalent: 0,
    };

    let report = format_json_report(&score, &[], &None);

    assert!(report["results"].as_array().unwrap().is_empty());
}

// ==================== format_report Tests ====================

#[test]
fn test_format_report_json_format() {
    let score = MutationScore {
        score: 0.8,
        total: 10,
        killed: 8,
        survived: 2,
        compile_errors: 0,
        timeouts: 0,
        equivalent: 0,
    };

    let report = format_report(&score, &[], &None, OutputFormat::Json);

    assert!(report["mutation_score"].is_number());
    assert!(report["total_mutants"].is_number());
}

#[test]
fn test_format_report_text_format() {
    let score = MutationScore {
        score: 0.8,
        total: 10,
        killed: 8,
        survived: 2,
        compile_errors: 0,
        timeouts: 0,
        equivalent: 0,
    };

    let report = format_report(&score, &[], &None, OutputFormat::Text);

    assert!(report["summary"].is_string());
    assert!(report["breakdown"].is_string());
}

// ==================== create_mutation_engine Tests ====================

#[test]
fn test_create_mutation_engine() {
    let engine = create_mutation_engine();
    // Just verify it can be created without panicking
    let _ = engine;
}

// ==================== Integration-style Tests ====================

#[test]
fn test_full_config_workflow() {
    // Create a complete configuration
    let config = MutationTestConfig::new(
        Some(vec![
            "AOR".to_string(),
            "ROR".to_string(),
            "COR".to_string(),
        ]),
        true,
        true,
        16,
        true,
        Some(0.75),
        true,
        Some("jenkins".to_string()),
        150,
    );

    // Verify all fields are correctly set
    assert_eq!(config.operators.as_ref().unwrap().len(), 3);
    assert!(config.ml_predict);
    assert!(config.distributed);
    assert_eq!(config.workers, 16);
    assert!(config.progress);
    assert_eq!(config.min_score, Some(0.75));
    assert!(config.ci_learning);
    assert_eq!(config.ci_provider, Some("jenkins".to_string()));
    assert_eq!(config.auto_train_threshold, 150);
}

#[test]
fn test_score_validation_workflow() {
    // Test progressive score validation
    let scores = [0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
    let threshold = Some(0.75);

    for &s in &scores {
        let score = MutationScore {
            score: s,
            total: 10,
            killed: (s * 10.0) as usize,
            survived: 10 - (s * 10.0) as usize,
            compile_errors: 0,
            timeouts: 0,
            equivalent: 0,
        };

        let result = validate_score_threshold(&score, threshold);

        if s >= 0.75 {
            assert!(result.is_ok(), "Score {} should pass threshold", s);
        } else {
            assert!(result.is_err(), "Score {} should fail threshold", s);
        }
    }
}
