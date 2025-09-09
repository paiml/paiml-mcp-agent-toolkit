//! Sprint 82: Validate complexity analyzer accuracy after fix
//! 
//! This test ensures the cognitive complexity calculation follows
//! SonarSource specification correctly after fixing the bug where
//! nesting level was incorrectly added to all cognitive increments.

use pmat::services::accurate_complexity_analyzer::AccurateComplexityAnalyzer;
use std::fs;
use tempfile::TempDir;

#[tokio::test]
async fn test_cognitive_complexity_simple_if_else_chain() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.rs");
    
    // Simple if-else chain: cognitive complexity should be 4
    fs::write(
        &test_file,
        r#"
        fn process_sprint_line(line: &str) -> Option<String> {
            if line.starts_with("Sprint 1:") {
                Some("First".to_string())
            } else if line.starts_with("Sprint 2:") {
                Some("Second".to_string())
            } else if line.starts_with("Sprint 3:") {
                Some("Third".to_string())
            } else if line.starts_with("Sprint 4:") {
                Some("Fourth".to_string())
            } else {
                None
            }
        }
    "#,
    )
    .unwrap();
    
    let analyzer = AccurateComplexityAnalyzer::new();
    let result = analyzer.analyze_file(&test_file).await.unwrap();
    
    assert_eq!(result.functions.len(), 1);
    let func = &result.functions[0];
    
    // Cyclomatic: 1 base + 4 decision points = 5
    assert_eq!(func.cyclomatic_complexity, 5, "Cyclomatic should be 5");
    
    // Cognitive: 4 (one for each if/else if, no nesting penalty at top level)
    assert_eq!(func.cognitive_complexity, 4, 
        "Cognitive complexity for simple if-else chain should be 4, not 52!");
}

#[tokio::test]
async fn test_cognitive_complexity_nested_if() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.rs");
    
    // Nested if: cognitive should be 1 + (1+1) = 3
    fs::write(
        &test_file,
        r#"
        fn nested_if(x: i32, y: i32) -> i32 {
            if x > 0 {                  // +1 (no nesting)
                if y > 0 {              // +1 + 1 (nesting level 1)
                    x + y
                } else {
                    x
                }
            } else {
                0
            }
        }
    "#,
    )
    .unwrap();
    
    let analyzer = AccurateComplexityAnalyzer::new();
    let result = analyzer.analyze_file(&test_file).await.unwrap();
    
    assert_eq!(result.functions.len(), 1);
    let func = &result.functions[0];
    
    // Cognitive: 1 (outer if) + 2 (inner if with nesting) = 3
    assert_eq!(func.cognitive_complexity, 3, 
        "Nested if should have cognitive complexity 3");
}

#[tokio::test]
async fn test_cognitive_complexity_loop_with_nested_if() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.rs");
    
    fs::write(
        &test_file,
        r#"
        fn loop_with_nested_if(items: &[i32]) -> i32 {
            let mut sum = 0;
            for item in items {         // +1 (no nesting)
                if *item > 0 {          // +1 + 1 (nesting level 1)
                    sum += item;
                }
            }
            sum
        }
    "#,
    )
    .unwrap();
    
    let analyzer = AccurateComplexityAnalyzer::new();
    let result = analyzer.analyze_file(&test_file).await.unwrap();
    
    assert_eq!(result.functions.len(), 1);
    let func = &result.functions[0];
    
    // Cognitive: 1 (for loop) + 2 (nested if) = 3
    assert_eq!(func.cognitive_complexity, 3,
        "Loop with nested if should have cognitive complexity 3");
}

#[tokio::test]
async fn test_cognitive_complexity_match_with_guards() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.rs");
    
    fs::write(
        &test_file,
        r#"
        fn match_with_guards(x: Option<i32>) -> i32 {
            match x {                    // +1
                Some(n) if n > 0 => n,  // +1 for guard
                Some(n) if n < 0 => -n, // +1 for guard
                _ => 0,
            }
        }
    "#,
    )
    .unwrap();
    
    let analyzer = AccurateComplexityAnalyzer::new();
    let result = analyzer.analyze_file(&test_file).await.unwrap();
    
    assert_eq!(result.functions.len(), 1);
    let func = &result.functions[0];
    
    // Cognitive: 1 (match) + 1 (first guard) + 1 (second guard) = 3
    assert_eq!(func.cognitive_complexity, 3,
        "Match with 2 guards should have cognitive complexity 3");
}

#[tokio::test]
async fn test_cognitive_complexity_binary_operators() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.rs");
    
    fs::write(
        &test_file,
        r#"
        fn binary_operators(x: bool, y: bool, z: bool) -> bool {
            x && y || z  // +1 for &&, +1 for ||
        }
    "#,
    )
    .unwrap();
    
    let analyzer = AccurateComplexityAnalyzer::new();
    let result = analyzer.analyze_file(&test_file).await.unwrap();
    
    assert_eq!(result.functions.len(), 1);
    let func = &result.functions[0];
    
    // Cognitive: 1 (&&) + 1 (||) = 2
    assert_eq!(func.cognitive_complexity, 2,
        "Binary operators && and || should add 2 to cognitive complexity");
}

#[tokio::test]
async fn test_cognitive_complexity_deeply_nested() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.rs");
    
    fs::write(
        &test_file,
        r#"
        fn deeply_nested(x: i32) -> i32 {
            let mut result = 0;
            
            if x > 0 {                          // +1 (no nesting)
                for i in 0..x {                 // +1 + 1 (nesting level 1)
                    if i % 2 == 0 {             // +1 + 2 (nesting level 2)
                        if i > 10 {             // +1 + 3 (nesting level 3)
                            result += i * 2;
                        } else {
                            result += i;
                        }
                    }
                }
            }
            
            result
        }
    "#,
    )
    .unwrap();
    
    let analyzer = AccurateComplexityAnalyzer::new();
    let result = analyzer.analyze_file(&test_file).await.unwrap();
    
    assert_eq!(result.functions.len(), 1);
    let func = &result.functions[0];
    
    // Cognitive: 1 + 2 + 3 + 4 = 10
    assert_eq!(func.cognitive_complexity, 10,
        "Deeply nested structure should have cognitive complexity 10");
}

#[tokio::test]
async fn test_sprint_82_regression_fix() {
    // This test documents the Sprint 82 bug fix
    // Previously, the analyzer would report 52 for a simple if-else chain
    // The bug was that add_cognitive was adding nesting_level to ALL increments
    
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("regression.rs");
    
    let code = r###"
        // This function was incorrectly reported as having cognitive complexity 52
        fn process_line(line: &str) -> &'static str {
            if line.is_empty() {
                "empty"
            } else if line.starts_with("#") {
                "comment"  
            } else if line.starts_with("//") {
                "code_comment"
            } else if line.starts_with("fn") {
                "function"
            } else {
                "other"
            }
        }
    "###;
    
    fs::write(&test_file, code).unwrap();
    
    let analyzer = AccurateComplexityAnalyzer::new();
    let result = analyzer.analyze_file(&test_file).await.unwrap();
    
    let func = &result.functions[0];
    
    // Document the fix
    assert_ne!(func.cognitive_complexity, 52, 
        "Sprint 82 regression: Was reporting 52, should be 4");
    assert_eq!(func.cognitive_complexity, 4,
        "Sprint 82 fix: Correctly reports 4 for simple if-else chain");
    
    println!("✅ Sprint 82 Bug Fix Verified!");
    println!("   Previous (buggy): Cognitive = 52");
    println!("   Current (fixed):  Cognitive = {}", func.cognitive_complexity);
}