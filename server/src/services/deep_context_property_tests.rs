//! Property tests for deep context complexity analysis
//!
//! This module provides comprehensive property-based testing for deep context
//! analysis, ensuring that complexity values are accurate and not fixed at 1.0
//! (addressing issue #33).

use crate::services::simple_deep_context::{SimpleDeepContext, SimpleAnalysisConfig};
use proptest::prelude::*;
use std::fs;
use tempfile::TempDir;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]
    
    /// Property: Deep context analysis never returns all complexities as 1.0 (Issue #33)
    /// 
    /// This test verifies that the fix for issue #33 works correctly - complexity
    /// analysis should return varied, accurate values, not heuristic estimates of 1.0.
    #[test]
    fn prop_deep_context_complexity_not_fixed_at_one(
        num_simple_functions in 1..3usize,
        num_complex_functions in 1..2usize,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let temp_dir = TempDir::new().unwrap();
            let src_dir = temp_dir.path().join("src");
            fs::create_dir_all(&src_dir).unwrap();
            
            // Create a test file with mixed complexity functions
            let mut content = String::from("// Generated test file\n\n");
            
            // Add simple functions
            for i in 0..num_simple_functions {
                content.push_str(&format!(r#"
fn simple_function_{}() {{
    println!("Simple function {}");
}}
"#, i, i));
            }
            
            // Add complex functions
            for i in 0..num_complex_functions {
                content.push_str(&format!(r#"
fn complex_function_{}(a: i32, b: i32) {{
    if a > 0 {{
        if b > 0 {{
            for j in 0..10 {{
                match j {{
                    0 => println!("Zero"),
                    1 => println!("One"),
                    _ => println!("Other"),
                }}
            }}
        }}
    }}
}}
"#, i));
            }
            
            let test_file = src_dir.join("lib.rs");
            fs::write(&test_file, &content).unwrap();
            
            let analyzer = SimpleDeepContext::new();
            let config = SimpleAnalysisConfig {
                project_path: temp_dir.path().to_path_buf(),
                include_features: vec![],
                include_patterns: vec![],
                exclude_patterns: vec![],
                enable_verbose: false,
            };
            
            let report = analyzer.analyze(config).await.unwrap();
            
            // Property: Should have detected functions
            prop_assert!(report.complexity_metrics.total_functions > 0,
                "Should detect functions in generated code");
            
            // Property: Not all functions should have complexity 1.0 (issue #33 fix)
            // With complex functions, we should see varied complexity
            if report.complexity_metrics.total_functions > 1 && num_complex_functions > 0 {
                // Complex functions should show up in high complexity count or avg > 1.0
                prop_assert!(
                    report.complexity_metrics.high_complexity_count > 0 ||
                    report.complexity_metrics.avg_complexity > 1.5,
                    "Should detect high complexity with complex functions, got: avg={}, high_count={}",
                    report.complexity_metrics.avg_complexity,
                    report.complexity_metrics.high_complexity_count
                );
            }
            
            // Property: Average complexity should be reasonable
            prop_assert!(report.complexity_metrics.avg_complexity.is_finite(),
                "Average complexity should be finite");
            prop_assert!(report.complexity_metrics.avg_complexity > 0.0,
                "Average complexity should be positive");
            
            Ok(())
        })?;
    }
    
    /// Property: Complexity analysis is deterministic (Issue #33 reliability)
    #[test]
    fn prop_complexity_analysis_deterministic(
        has_complex_function in any::<bool>(),
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let content = if has_complex_function {
                r#"
fn complex_func(a: i32) {
    if a > 0 {
        for i in 0..a {
            if i % 2 == 0 {
                println!("Even: {}", i);
            }
        }
    }
}
"#
            } else {
                r#"
fn simple_func() {
    println!("Hello");
}
"#
            };
            
            let result1 = analyze_code_complexity(content).await.unwrap();
            let result2 = analyze_code_complexity(content).await.unwrap();
            
            // Property: Results should be identical across runs
            prop_assert_eq!(result1.total_functions, result2.total_functions,
                "Function count should be deterministic");
            prop_assert!((result1.avg_complexity - result2.avg_complexity).abs() < 0.001,
                "Average complexity should be deterministic");
            
            Ok(())
        })?;
    }
}

/// Helper function to analyze code complexity for property testing
async fn analyze_code_complexity(content: &str) -> anyhow::Result<crate::services::simple_deep_context::ComplexityMetrics> {
    let temp_dir = TempDir::new()?;
    let src_dir = temp_dir.path().join("src");
    fs::create_dir_all(&src_dir)?;
    
    let test_file = src_dir.join("lib.rs");
    fs::write(&test_file, content)?;
    
    let analyzer = SimpleDeepContext::new();
    let config = SimpleAnalysisConfig {
        project_path: temp_dir.path().to_path_buf(),
        include_features: vec![],
        include_patterns: vec![],
        exclude_patterns: vec![],
        enable_verbose: false,
    };
    
    let report = analyzer.analyze(config).await?;
    Ok(report.complexity_metrics)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_simple_complexity_analysis() {
        let simple_code = r#"
fn simple() {
    println!("Hello");
}

fn complex(a: i32) {
    if a > 0 {
        if a < 10 {
            println!("Range");
        }
    }
}
"#;
        
        let complexity = analyze_code_complexity(simple_code).await.unwrap();
        assert_eq!(complexity.total_functions, 2);
        assert!(complexity.avg_complexity > 1.0);
    }

    #[tokio::test]
    async fn test_empty_function_complexity() {
        let empty_code = "fn empty() {}";
        let complexity = analyze_code_complexity(empty_code).await.unwrap();
        
        assert_eq!(complexity.total_functions, 1);
        assert!(complexity.avg_complexity >= 1.0);
    }

    #[tokio::test]
    async fn test_high_complexity_function() {
        let complex_code = r#"
fn very_complex(a: i32, b: i32) {
    for i in 0..10 {
        match i {
            0 => {
                if a > 0 {
                    while b < 5 {
                        b += 1;
                    }
                }
            },
            1..=5 => {
                if b % 2 == 0 {
                    println!("Even");
                } else {
                    println!("Odd");
                }
            },
            _ => println!("High"),
        }
    }
}
"#;
        
        let complexity = analyze_code_complexity(complex_code).await.unwrap();
        assert_eq!(complexity.total_functions, 1);
        // Complex function should have high complexity or be detected as high complexity
        assert!(complexity.avg_complexity > 3.0 || complexity.high_complexity_count > 0, 
            "Complex function should show high complexity, got: avg={}, high_count={}", 
            complexity.avg_complexity, complexity.high_complexity_count);
    }
}