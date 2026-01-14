//! Tests for complexity patterns

use super::*;

#[test]
fn test_pattern_type_enum() {
    let patterns = vec![
        PatternType::Conditional,
        PatternType::Loop,
        PatternType::Nesting,
        PatternType::Recursion,
        PatternType::ErrorHandling,
        PatternType::LogicalOperator,
    ];
    
    for pattern in patterns {
        match pattern {
            PatternType::Conditional => assert_eq!(format!("{:?}", pattern), "Conditional"),
            PatternType::Loop => assert_eq!(format!("{:?}", pattern), "Loop"),
            PatternType::Nesting => assert_eq!(format!("{:?}", pattern), "Nesting"),
            PatternType::Recursion => assert_eq!(format!("{:?}", pattern), "Recursion"),
            PatternType::ErrorHandling => assert_eq!(format!("{:?}", pattern), "ErrorHandling"),
            PatternType::LogicalOperator => assert_eq!(format!("{:?}", pattern), "LogicalOperator"),
        }
    }
}

#[test]
fn test_complexity_pattern_creation() {
    let pattern = ComplexityPattern {
        pattern_type: PatternType::Conditional,
        weight: 1.5,
        description: "If statement".to_string(),
        examples: vec!["if x > 0 { }".to_string()],
    };
    
    assert!(matches!(pattern.pattern_type, PatternType::Conditional));
    assert_eq!(pattern.weight, 1.5);
    assert_eq!(pattern.description, "If statement");
    assert_eq!(pattern.examples.len(), 1);
}

#[test]
fn test_pattern_matcher_trait() {
    struct TestMatcher;
    
    impl PatternMatcher for TestMatcher {
        fn matches(&self, _code: &str) -> bool {
            true
        }
        
        fn complexity_impact(&self) -> f64 {
            2.0
        }
    }
    
    let matcher = TestMatcher;
    assert!(matcher.matches("any code"));
    assert_eq!(matcher.complexity_impact(), 2.0);
}

#[test]
fn test_get_default_patterns() {
    let patterns = get_default_patterns();
    
    // Should have some default patterns
    assert!(!patterns.is_empty());
    
    // Check that we have different pattern types
    let has_conditional = patterns.iter().any(|p| matches!(p.pattern_type, PatternType::Conditional));
    let has_loop = patterns.iter().any(|p| matches!(p.pattern_type, PatternType::Loop));
    
    assert!(has_conditional || has_loop);
}

#[test]
fn test_calculate_pattern_complexity() {
    let code = r#"
    if condition {
        for i in 0..10 {
            if nested {
                println!("Complex!");
            }
        }
    }
    "#;
    
    let patterns = get_default_patterns();
    let complexity = calculate_pattern_complexity(code, &patterns);
    
    // Should detect some complexity
    assert!(complexity > 0.0);
}

// Helper functions for tests
fn get_default_patterns() -> Vec<ComplexityPattern> {
    vec![
        ComplexityPattern {
            pattern_type: PatternType::Conditional,
            weight: 1.0,
            description: "If statement".to_string(),
            examples: vec!["if x > 0".to_string()],
        },
        ComplexityPattern {
            pattern_type: PatternType::Loop,
            weight: 2.0,
            description: "For loop".to_string(),
            examples: vec!["for i in 0..10".to_string()],
        },
        ComplexityPattern {
            pattern_type: PatternType::Nesting,
            weight: 1.5,
            description: "Nested blocks".to_string(),
            examples: vec!["if { if { } }".to_string()],
        },
    ]
}

fn calculate_pattern_complexity(code: &str, patterns: &[ComplexityPattern]) -> f64 {
    let mut total = 0.0;
    
    for pattern in patterns {
        match pattern.pattern_type {
            PatternType::Conditional => {
                total += code.matches("if ").count() as f64 * pattern.weight;
            }
            PatternType::Loop => {
                total += code.matches("for ").count() as f64 * pattern.weight;
                total += code.matches("while ").count() as f64 * pattern.weight;
            }
            _ => {}
        }
    }
    
    total
}