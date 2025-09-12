//! TDD Test for Entropy Duplicate Detection Issue
//! Following Toyota Way: Stop the line and fix the defect
//! Sprint 98: Fix entropy violations count (5831 → correct count)

use pmat::entropy::{EntropyAnalyzer, EntropyConfig, PatternType};
use pmat::entropy::violation_detector::Severity;
use std::path::PathBuf;
use tempfile::tempdir;
use std::fs;

#[tokio::test]
async fn test_no_duplicate_violations_reported() {
    // RED Phase: This test should fail initially, showing duplicates
    
    // Create a test project with known patterns
    let temp_dir = tempdir().unwrap();
    let src_dir = temp_dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();
    
    // Create file with repetitive pattern (should produce ONE violation)
    let test_file = src_dir.join("test.rs");
    fs::write(&test_file, r#"
fn process_a() {
    if let Some(x) = data.get("a") {
        println!("Processing: {}", x);
        do_something(x);
    }
}

fn process_b() {
    if let Some(x) = data.get("b") {
        println!("Processing: {}", x);
        do_something(x);
    }
}

fn process_c() {
    if let Some(x) = data.get("c") {
        println!("Processing: {}", x);
        do_something(x);
    }
}

fn process_d() {
    if let Some(x) = data.get("d") {
        println!("Processing: {}", x);
        do_something(x);
    }
}

fn process_e() {
    if let Some(x) = data.get("e") {
        println!("Processing: {}", x);
        do_something(x);
    }
}
"#).unwrap();

    // Configure analyzer
    let mut config = EntropyConfig::default();
    config.min_severity = Severity::Low;
    config.max_pattern_repetition = 3; // Should trigger on 5 repetitions
    
    let analyzer = EntropyAnalyzer::with_config(config);
    
    // Analyze the project
    let report = analyzer.analyze(temp_dir.path()).await
        .expect("Analysis should succeed");
    
    // Debug: Print what we got
    println!("Files analyzed: {}", report.total_files_analyzed);
    println!("Violations found: {}", report.actionable_violations.len());
    for (i, violation) in report.actionable_violations.iter().enumerate() {
        println!("  {}. Pattern: {:?}, Repetitions: {}", 
            i + 1, 
            violation.pattern.pattern_type,
            violation.pattern.repetitions
        );
    }
    
    // ASSERTION: Should have exactly ONE violation for the repeated pattern
    // Not multiple violations for the same pattern
    assert_eq!(report.actionable_violations.len(), 1, 
        "Should report exactly 1 violation for the repeated pattern, not duplicates");
    
    // The one violation should identify the pattern correctly
    let violation = &report.actionable_violations[0];
    assert_eq!(violation.pattern.repetitions, 5, 
        "Should detect 5 repetitions of the pattern");
}

#[tokio::test]
async fn test_distinct_patterns_reported_separately() {
    // This test ensures different patterns are reported as separate violations
    
    let temp_dir = tempdir().unwrap();
    let src_dir = temp_dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();
    
    // Create file with two different repetitive patterns
    let test_file = src_dir.join("mixed.rs");
    fs::write(&test_file, r#"
// Pattern A: Error handling repetition
fn handle_error_1() {
    match result {
        Ok(v) => process(v),
        Err(e) => log_error(e),
    }
}

fn handle_error_2() {
    match result {
        Ok(v) => process(v),
        Err(e) => log_error(e),
    }
}

fn handle_error_3() {
    match result {
        Ok(v) => process(v),
        Err(e) => log_error(e),
    }
}

fn handle_error_4() {
    match result {
        Ok(v) => process(v),
        Err(e) => log_error(e),
    }
}

// Pattern B: Data validation repetition  
fn validate_1(input: &str) -> bool {
    if input.is_empty() {
        return false;
    }
    input.len() < 100
}

fn validate_2(input: &str) -> bool {
    if input.is_empty() {
        return false;
    }
    input.len() < 100
}

fn validate_3(input: &str) -> bool {
    if input.is_empty() {
        return false;
    }
    input.len() < 100
}

fn validate_4(input: &str) -> bool {
    if input.is_empty() {
        return false;
    }
    input.len() < 100
}
"#).unwrap();

    let mut config = EntropyConfig::default();
    config.min_severity = Severity::Low;
    config.max_pattern_repetition = 3;
    
    let analyzer = EntropyAnalyzer::with_config(config);
    let report = analyzer.analyze(temp_dir.path()).await
        .expect("Analysis should succeed");
    
    // Should have exactly 2 violations (one for each pattern type)
    assert_eq!(report.actionable_violations.len(), 2, 
        "Should report 2 distinct violations for 2 different patterns");
    
    // Both should have 4 repetitions
    for violation in &report.actionable_violations {
        assert_eq!(violation.pattern.repetitions, 4, 
            "Each pattern repeated 4 times");
    }
}

#[test]
fn test_entropy_config_respects_thresholds() {
    // Test that configuration thresholds are properly applied
    
    let config = EntropyConfig {
        max_pattern_repetition: 5,
        min_severity: Severity::High,
        exclude_paths: vec!["tests/**".to_string()],
        ..Default::default()
    };
    
    assert_eq!(config.max_pattern_repetition, 5);
    assert_eq!(config.min_severity, Severity::High);
    assert!(config.exclude_paths.contains(&"tests/**".to_string()));
}