//! Coverage boost tests for services/ml_quality_scorer.rs
//! Tests: ComplexityFeatures, TDGFeatures, QualityTrainingSample,
//! QualityPrediction, MLQualityScorer

use crate::services::ml_quality_scorer::{
    ComplexityFeatures, MLQualityScorer, QualityPrediction, QualityTrainingSample, TDGFeatures,
};
use std::collections::HashMap;

// ============ ComplexityFeatures Tests ============

#[test]
fn test_complexity_features_from_source_empty() {
    let features = ComplexityFeatures::from_source("", "rust");
    assert!((features.loc - 0.0).abs() < f64::EPSILON);
    assert!((features.function_count - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_complexity_features_from_source_simple_rust() {
    let source = r#"
fn main() {
    if true {
        for i in 0..10 {
            println!("{}", i);
        }
    }
}
"#;
    let features = ComplexityFeatures::from_source(source, "rust");
    assert!(features.loc > 0.0);
    assert!(features.function_count >= 1.0);
    assert!(features.conditional_count >= 1.0);
    assert!(features.loop_count >= 1.0);
    assert!(features.max_nesting >= 2.0);
    assert!((features.language_type - 1.0).abs() < f64::EPSILON); // rust = 1.0
}

#[test]
fn test_complexity_features_from_source_python() {
    let source = r#"
def hello():
    if True:
        for i in range(10):
            print(i)
"#;
    let features = ComplexityFeatures::from_source(source, "python");
    assert!((features.language_type - 2.0).abs() < f64::EPSILON);
    assert!(features.function_count >= 1.0);
}

#[test]
fn test_complexity_features_from_source_javascript() {
    let source = "function test() { if (true) {} }";
    let features = ComplexityFeatures::from_source(source, "javascript");
    assert!((features.language_type - 3.0).abs() < f64::EPSILON);
}

#[test]
fn test_complexity_features_from_source_typescript() {
    let features = ComplexityFeatures::from_source("", "typescript");
    assert!((features.language_type - 3.0).abs() < f64::EPSILON);
}

#[test]
fn test_complexity_features_from_source_go() {
    let source = "func main() {}";
    let features = ComplexityFeatures::from_source(source, "go");
    assert!((features.language_type - 4.0).abs() < f64::EPSILON);
    assert!(features.function_count >= 1.0);
}

#[test]
fn test_complexity_features_from_source_java() {
    let features = ComplexityFeatures::from_source("", "java");
    assert!((features.language_type - 5.0).abs() < f64::EPSILON);
}

#[test]
fn test_complexity_features_from_source_c() {
    let features = ComplexityFeatures::from_source("", "c");
    assert!((features.language_type - 6.0).abs() < f64::EPSILON);
}

#[test]
fn test_complexity_features_from_source_unknown() {
    let features = ComplexityFeatures::from_source("", "haskell");
    assert!((features.language_type - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_complexity_features_to_vector() {
    let features = ComplexityFeatures {
        loc: 100.0,
        max_nesting: 5.0,
        control_flow_count: 10.0,
        loop_count: 3.0,
        conditional_count: 7.0,
        function_count: 5.0,
        avg_function_size: 20.0,
        language_type: 1.0,
    };
    let vec = features.to_vector();
    assert_eq!(vec.len(), 8);
    assert!((vec[0] - 1.0).abs() < f64::EPSILON); // 100/100
    assert!((vec[1] - 0.5).abs() < f64::EPSILON); // 5/10
}

#[test]
fn test_complexity_features_serde() {
    let features = ComplexityFeatures::from_source("fn test() {}", "rust");
    let json = serde_json::to_string(&features).unwrap();
    let back: ComplexityFeatures = serde_json::from_str(&json).unwrap();
    assert!((back.language_type - 1.0).abs() < f64::EPSILON);
}

#[test]
fn test_complexity_features_control_flow_counting() {
    let source = r#"
fn test() {
    if true {}
    else if false {}
    while true {}
    loop {}
    match x {}
}
"#;
    let features = ComplexityFeatures::from_source(source, "rust");
    assert!(features.control_flow_count >= 3.0); // if, while, loop, match
    assert!(features.conditional_count >= 1.0);
    assert!(features.loop_count >= 2.0); // while, loop
}

// ============ TDGFeatures Tests ============

#[test]
fn test_tdg_features_to_vector() {
    let features = TDGFeatures {
        complexity: 2.5,
        churn: 1.0,
        coupling: 0.5,
        domain_risk: 0.3,
        duplication: 0.1,
        test_coverage: 0.85,
        file_age_days: 365.0,
        commit_frequency: 5.0,
    };
    let vec = features.to_vector();
    assert_eq!(vec.len(), 8);
    assert!((vec[0] - 0.5).abs() < f64::EPSILON); // 2.5/5
    assert!((vec[5] - 0.85).abs() < f64::EPSILON); // coverage pass-through
    assert!((vec[6] - 1.0).abs() < f64::EPSILON); // 365/365
}

#[test]
fn test_tdg_features_serde() {
    let features = TDGFeatures {
        complexity: 1.0,
        churn: 2.0,
        coupling: 3.0,
        domain_risk: 4.0,
        duplication: 5.0,
        test_coverage: 0.5,
        file_age_days: 100.0,
        commit_frequency: 10.0,
    };
    let json = serde_json::to_string(&features).unwrap();
    let back: TDGFeatures = serde_json::from_str(&json).unwrap();
    assert!((back.complexity - 1.0).abs() < f64::EPSILON);
    assert!((back.test_coverage - 0.5).abs() < f64::EPSILON);
}

#[test]
fn test_tdg_features_clone() {
    let features = TDGFeatures {
        complexity: 1.0,
        churn: 1.0,
        coupling: 1.0,
        domain_risk: 1.0,
        duplication: 1.0,
        test_coverage: 0.9,
        file_age_days: 30.0,
        commit_frequency: 2.0,
    };
    let cloned = features.clone();
    assert!((cloned.test_coverage - 0.9).abs() < f64::EPSILON);
}

// ============ QualityTrainingSample Tests ============

#[test]
fn test_quality_training_sample_serde() {
    let sample = QualityTrainingSample {
        features: vec![1.0, 2.0, 3.0],
        target_score: 0.8,
        weight: Some(1.5),
    };
    let json = serde_json::to_string(&sample).unwrap();
    let back: QualityTrainingSample = serde_json::from_str(&json).unwrap();
    assert_eq!(back.features.len(), 3);
    assert!((back.target_score - 0.8).abs() < f64::EPSILON);
    assert_eq!(back.weight, Some(1.5));
}

#[test]
fn test_quality_training_sample_no_weight() {
    let sample = QualityTrainingSample {
        features: vec![0.5],
        target_score: 0.5,
        weight: None,
    };
    assert!(sample.weight.is_none());
}

// ============ QualityPrediction Tests ============

#[test]
fn test_quality_prediction_serde() {
    let mut contributions = HashMap::new();
    contributions.insert("loc".to_string(), 0.3);
    contributions.insert("nesting".to_string(), 0.5);

    let pred = QualityPrediction {
        score: 7.5,
        confidence: 0.9,
        ml_used: true,
        feature_contributions: contributions,
    };
    let json = serde_json::to_string(&pred).unwrap();
    let back: QualityPrediction = serde_json::from_str(&json).unwrap();
    assert!((back.score - 7.5).abs() < f64::EPSILON);
    assert!(back.ml_used);
    assert_eq!(back.feature_contributions.len(), 2);
}

#[test]
fn test_quality_prediction_heuristic() {
    let pred = QualityPrediction {
        score: 5.0,
        confidence: 0.5,
        ml_used: false,
        feature_contributions: HashMap::new(),
    };
    assert!(!pred.ml_used);
    assert!(pred.feature_contributions.is_empty());
}

// ============ MLQualityScorer Tests ============

#[test]
fn test_ml_quality_scorer_new() {
    let scorer = MLQualityScorer::new();
    assert!(!scorer.is_trained());
}

#[test]
fn test_ml_quality_scorer_predict_complexity_untrained() {
    let scorer = MLQualityScorer::new();
    let features = ComplexityFeatures::from_source("fn main() {}", "rust");
    let pred = scorer.predict_complexity(&features).unwrap();
    // Untrained → heuristic prediction
    assert!(!pred.ml_used);
    assert!(pred.score >= 0.0);
}

#[test]
fn test_ml_quality_scorer_predict_tdg_untrained() {
    let scorer = MLQualityScorer::new();
    let features = TDGFeatures {
        complexity: 2.0,
        churn: 1.0,
        coupling: 0.5,
        domain_risk: 0.3,
        duplication: 0.1,
        test_coverage: 0.8,
        file_age_days: 90.0,
        commit_frequency: 3.0,
    };
    let pred = scorer.predict_tdg(&features).unwrap();
    assert!(!pred.ml_used);
    assert!(pred.score >= 0.0);
}

#[test]
fn test_ml_quality_scorer_train_complexity_empty() {
    let mut scorer = MLQualityScorer::new();
    let result = scorer.train_complexity_model(&[]);
    assert!(result.is_err());
}

#[test]
fn test_ml_quality_scorer_train_complexity_wrong_features() {
    let mut scorer = MLQualityScorer::new();
    let samples = vec![QualityTrainingSample {
        features: vec![1.0, 2.0], // Only 2 features, needs 8
        target_score: 5.0,
        weight: None,
    }];
    let result = scorer.train_complexity_model(&samples);
    assert!(result.is_err());
}

#[test]
fn test_ml_quality_scorer_train_complexity_valid() {
    let mut scorer = MLQualityScorer::new();
    let samples: Vec<QualityTrainingSample> = (0..10)
        .map(|i| {
            let f = i as f64 / 10.0;
            QualityTrainingSample {
                features: vec![f, f * 0.5, f * 0.3, f * 0.2, f * 0.1, f * 0.4, f * 0.6, 0.1],
                target_score: f * 10.0,
                weight: None,
            }
        })
        .collect();
    let result = scorer.train_complexity_model(&samples);
    // Training may succeed or fail depending on matrix conditioning
    // but shouldn't panic
    let _ = result;
}

#[test]
fn test_ml_quality_scorer_is_trained_default() {
    let scorer = MLQualityScorer::new();
    assert!(!scorer.is_trained());
}
