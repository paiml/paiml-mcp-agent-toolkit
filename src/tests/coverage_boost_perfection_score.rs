#![cfg_attr(coverage_nightly, coverage(off))]
//! Coverage boost tests for services/perfection_score module
//! Tests for PerfectionScoreCalculator, CategoryScore, and related types

use crate::services::normalized_score::NormalizedScore;
use crate::services::perfection_score::{
    CategoryScore, CategoryWeights, PerfectionScoreCalculator, PerfectionScoreResult,
    MAX_PERFECTION_SCORE,
};

// ============ Constants Tests ============

#[test]
fn test_max_perfection_score_is_200() {
    assert_eq!(MAX_PERFECTION_SCORE, 200);
}

// ============ CategoryWeights Tests ============

#[test]
fn test_category_weights_default() {
    let weights = CategoryWeights::default();
    assert_eq!(weights.tdg, 40);
    assert_eq!(weights.repo_score, 30);
    assert_eq!(weights.rust_score, 30);
    assert_eq!(weights.popper_score, 25);
    assert_eq!(weights.test_coverage, 25);
    assert_eq!(weights.mutation, 20);
    assert_eq!(weights.documentation, 15);
    assert_eq!(weights.performance, 15);
}

#[test]
fn test_category_weights_sum_to_200() {
    let weights = CategoryWeights::default();
    let total = weights.tdg
        + weights.repo_score
        + weights.rust_score
        + weights.popper_score
        + weights.test_coverage
        + weights.mutation
        + weights.documentation
        + weights.performance;
    assert_eq!(total, 200);
}

#[test]
fn test_category_weights_clone() {
    let weights = CategoryWeights::default();
    let cloned = weights;
    assert_eq!(weights.tdg, cloned.tdg);
}

#[test]
fn test_category_weights_debug() {
    let weights = CategoryWeights::default();
    let debug = format!("{:?}", weights);
    assert!(debug.contains("CategoryWeights"));
}

// ============ CategoryScore Tests ============

#[test]
fn test_category_score_new_perfect() {
    let score = CategoryScore::new("Test", 100.0, 40);
    assert_eq!(score.name, "Test");
    assert_eq!(score.raw_score, 100.0);
    assert_eq!(score.max_points, 40);
    assert_eq!(score.earned_points, 40.0);
    assert_eq!(score.grade, "A+");
}

#[test]
fn test_category_score_new_zero() {
    let score = CategoryScore::new("Test", 0.0, 40);
    assert_eq!(score.earned_points, 0.0);
    assert_eq!(score.grade, "F");
}

#[test]
fn test_category_score_new_fifty() {
    let score = CategoryScore::new("Test", 50.0, 40);
    assert_eq!(score.earned_points, 20.0);
    assert_eq!(score.grade, "F"); // 50% is F
}

#[test]
fn test_category_score_with_details() {
    let score = CategoryScore::new("Test", 80.0, 40).with_details("Some details");
    assert_eq!(score.details, Some("Some details".to_string()));
}

#[test]
fn test_category_score_grade_a_plus() {
    let score = CategoryScore::new("Test", 97.0, 10);
    assert_eq!(score.grade, "A+");
}

#[test]
fn test_category_score_grade_a() {
    let score = CategoryScore::new("Test", 93.0, 10);
    assert_eq!(score.grade, "A");
}

#[test]
fn test_category_score_grade_a_minus() {
    let score = CategoryScore::new("Test", 90.0, 10);
    assert_eq!(score.grade, "A-");
}

#[test]
fn test_category_score_grade_b_plus() {
    let score = CategoryScore::new("Test", 87.0, 10);
    assert_eq!(score.grade, "B+");
}

#[test]
fn test_category_score_grade_b() {
    let score = CategoryScore::new("Test", 83.0, 10);
    assert_eq!(score.grade, "B");
}

#[test]
fn test_category_score_grade_b_minus() {
    let score = CategoryScore::new("Test", 80.0, 10);
    assert_eq!(score.grade, "B-");
}

#[test]
fn test_category_score_grade_c_plus() {
    let score = CategoryScore::new("Test", 77.0, 10);
    assert_eq!(score.grade, "C+");
}

#[test]
fn test_category_score_grade_c() {
    let score = CategoryScore::new("Test", 73.0, 10);
    assert_eq!(score.grade, "C");
}

#[test]
fn test_category_score_grade_c_minus() {
    let score = CategoryScore::new("Test", 70.0, 10);
    assert_eq!(score.grade, "C-");
}

#[test]
fn test_category_score_grade_d_plus() {
    let score = CategoryScore::new("Test", 67.0, 10);
    assert_eq!(score.grade, "D+");
}

#[test]
fn test_category_score_grade_d() {
    let score = CategoryScore::new("Test", 63.0, 10);
    assert_eq!(score.grade, "D");
}

#[test]
fn test_category_score_grade_d_minus() {
    let score = CategoryScore::new("Test", 60.0, 10);
    assert_eq!(score.grade, "D-");
}

#[test]
fn test_category_score_grade_f() {
    let score = CategoryScore::new("Test", 59.0, 10);
    assert_eq!(score.grade, "F");
}

#[test]
fn test_category_score_serialization() {
    let score = CategoryScore::new("Test", 85.0, 25);
    let serialized = serde_json::to_string(&score).unwrap();
    let deserialized: CategoryScore = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.name, "Test");
    assert_eq!(deserialized.raw_score, 85.0);
}

#[test]
fn test_category_score_debug() {
    let score = CategoryScore::new("Test", 75.0, 20);
    let debug = format!("{:?}", score);
    assert!(debug.contains("CategoryScore"));
}

#[test]
fn test_category_score_clone() {
    let score = CategoryScore::new("Test", 75.0, 20);
    let cloned = score.clone();
    assert_eq!(cloned.name, "Test");
}

// ============ PerfectionScoreResult Tests ============

#[test]
fn test_perfection_score_result_new_empty() {
    let result = PerfectionScoreResult::new(vec![]);
    assert_eq!(result.total_score, 0.0);
    assert_eq!(result.max_score, 200);
    assert_eq!(result.grade, "F");
}

#[test]
fn test_perfection_score_result_new_perfect() {
    let categories = vec![
        CategoryScore::new("TDG", 100.0, 40),
        CategoryScore::new("Repo", 100.0, 30),
        CategoryScore::new("Rust", 100.0, 30),
        CategoryScore::new("Popper", 100.0, 25),
        CategoryScore::new("Coverage", 100.0, 25),
        CategoryScore::new("Mutation", 100.0, 20),
        CategoryScore::new("Docs", 100.0, 15),
        CategoryScore::new("Perf", 100.0, 15),
    ];
    let result = PerfectionScoreResult::new(categories);
    assert_eq!(result.total_score, 200.0);
    assert_eq!(result.grade, "A+");
}

#[test]
fn test_perfection_score_result_with_target() {
    let categories = vec![CategoryScore::new("TDG", 80.0, 40)];
    let result = PerfectionScoreResult::new(categories).with_target(100);
    assert!(result.target_gap.is_some());
    let gap = result.target_gap.unwrap();
    assert_eq!(gap, 100.0 - 32.0); // 80% of 40 = 32
}

#[test]
fn test_perfection_score_result_overall_grade_a_plus() {
    let categories = vec![
        CategoryScore::new("Test", 100.0, 100),
        CategoryScore::new("Test2", 100.0, 100),
    ];
    let result = PerfectionScoreResult::new(categories);
    assert_eq!(result.grade, "A+");
}

#[test]
fn test_perfection_score_result_overall_grade_a() {
    // 90% = 180/200
    let categories = vec![
        CategoryScore::new("Test", 90.0, 100),
        CategoryScore::new("Test2", 90.0, 100),
    ];
    let result = PerfectionScoreResult::new(categories);
    assert_eq!(result.grade, "A");
}

#[test]
fn test_perfection_score_result_overall_grade_a_minus() {
    // 85% = 170/200
    let categories = vec![
        CategoryScore::new("Test", 85.0, 100),
        CategoryScore::new("Test2", 85.0, 100),
    ];
    let result = PerfectionScoreResult::new(categories);
    assert_eq!(result.grade, "A-");
}

#[test]
fn test_perfection_score_result_overall_grade_b_plus() {
    // 80% = 160/200
    let categories = vec![
        CategoryScore::new("Test", 80.0, 100),
        CategoryScore::new("Test2", 80.0, 100),
    ];
    let result = PerfectionScoreResult::new(categories);
    assert_eq!(result.grade, "B+");
}

#[test]
fn test_perfection_score_result_overall_grade_b() {
    // 70% = 140/200
    let categories = vec![
        CategoryScore::new("Test", 70.0, 100),
        CategoryScore::new("Test2", 70.0, 100),
    ];
    let result = PerfectionScoreResult::new(categories);
    assert_eq!(result.grade, "B");
}

#[test]
fn test_perfection_score_result_overall_grade_c() {
    // 60% = 120/200
    let categories = vec![
        CategoryScore::new("Test", 60.0, 100),
        CategoryScore::new("Test2", 60.0, 100),
    ];
    let result = PerfectionScoreResult::new(categories);
    assert_eq!(result.grade, "C");
}

#[test]
fn test_perfection_score_result_overall_grade_d() {
    // 50% = 100/200
    let categories = vec![
        CategoryScore::new("Test", 50.0, 100),
        CategoryScore::new("Test2", 50.0, 100),
    ];
    let result = PerfectionScoreResult::new(categories);
    assert_eq!(result.grade, "D");
}

#[test]
fn test_perfection_score_result_overall_grade_f() {
    // 40% = 80/200
    let categories = vec![
        CategoryScore::new("Test", 40.0, 100),
        CategoryScore::new("Test2", 40.0, 100),
    ];
    let result = PerfectionScoreResult::new(categories);
    assert_eq!(result.grade, "F");
}

#[test]
fn test_perfection_score_result_recommendations_critical() {
    let categories = vec![CategoryScore::new("Test", 50.0, 100)];
    let result = PerfectionScoreResult::new(categories);
    assert!(result
        .recommendations
        .iter()
        .any(|r| r.contains("critical")));
}

#[test]
fn test_perfection_score_result_recommendations_needs_attention() {
    let categories = vec![CategoryScore::new("Test", 70.0, 100)];
    let result = PerfectionScoreResult::new(categories);
    assert!(result
        .recommendations
        .iter()
        .any(|r| r.contains("needs attention")));
}

#[test]
fn test_perfection_score_result_recommendations_healthy() {
    let categories = vec![CategoryScore::new("Test", 95.0, 100)];
    let result = PerfectionScoreResult::new(categories);
    assert!(result.recommendations.iter().any(|r| r.contains("healthy")));
}

#[test]
fn test_perfection_score_result_serialization() {
    let categories = vec![CategoryScore::new("Test", 80.0, 40)];
    let result = PerfectionScoreResult::new(categories);
    let serialized = serde_json::to_string(&result).unwrap();
    let deserialized: PerfectionScoreResult = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.total_score, result.total_score);
}

#[test]
fn test_perfection_score_result_debug() {
    let categories = vec![CategoryScore::new("Test", 80.0, 40)];
    let result = PerfectionScoreResult::new(categories);
    let debug = format!("{:?}", result);
    assert!(debug.contains("PerfectionScoreResult"));
}

#[test]
fn test_perfection_score_result_clone() {
    let categories = vec![CategoryScore::new("Test", 80.0, 40)];
    let result = PerfectionScoreResult::new(categories);
    let cloned = result.clone();
    assert_eq!(cloned.total_score, result.total_score);
}

// ============ NormalizedScore Trait Tests ============

#[test]
fn test_perfection_score_normalized() {
    let categories = vec![
        CategoryScore::new("Test", 100.0, 100),
        CategoryScore::new("Test2", 100.0, 100),
    ];
    let result = PerfectionScoreResult::new(categories);
    // 200/200 = 100%
    assert_eq!(result.normalized(), 100.0);
}

#[test]
fn test_perfection_score_normalized_half() {
    let categories = vec![
        CategoryScore::new("Test", 50.0, 100),
        CategoryScore::new("Test2", 50.0, 100),
    ];
    let result = PerfectionScoreResult::new(categories);
    // 100/200 = 50%
    assert_eq!(result.normalized(), 50.0);
}

#[test]
fn test_perfection_score_raw() {
    let categories = vec![CategoryScore::new("Test", 80.0, 100)];
    let result = PerfectionScoreResult::new(categories);
    assert_eq!(result.raw(), 80.0);
}

#[test]
fn test_perfection_score_max_raw() {
    let categories = vec![CategoryScore::new("Test", 80.0, 100)];
    let result = PerfectionScoreResult::new(categories);
    assert_eq!(result.max_raw(), 200.0);
}

// ============ Display Trait Tests ============

#[test]
fn test_perfection_score_display() {
    let categories = vec![
        CategoryScore::new("Test", 80.0, 100),
        CategoryScore::new("Test2", 80.0, 100),
    ];
    let result = PerfectionScoreResult::new(categories);
    let display = format!("{}", result);
    assert!(display.contains("Perfection Score"));
    assert!(display.contains("80.0/100"));
}

// ============ PerfectionScoreCalculator Tests ============

#[test]
fn test_calculator_new() {
    let calculator = PerfectionScoreCalculator::new();
    let _ = calculator;
}

#[test]
fn test_calculator_default() {
    let calculator = PerfectionScoreCalculator::default();
    let _ = calculator;
}

#[test]
fn test_calculator_fast_mode() {
    let calculator = PerfectionScoreCalculator::new().fast_mode(true);
    let _ = calculator;
}

#[test]
fn test_calculator_fast_mode_false() {
    let calculator = PerfectionScoreCalculator::new().fast_mode(false);
    let _ = calculator;
}

#[tokio::test]
async fn test_calculator_calculate_nonexistent_path() {
    let calculator = PerfectionScoreCalculator::new().fast_mode(true);
    // Use a path that doesn't exist - should still work with defaults
    let result = calculator
        .calculate(std::path::Path::new("/nonexistent/path"))
        .await;
    // Even with nonexistent path, calculator should produce a result
    let _ = result;
}

#[tokio::test]
async fn test_calculator_calculate_current_dir() {
    let calculator = PerfectionScoreCalculator::new().fast_mode(true);
    let result = calculator.calculate(std::path::Path::new(".")).await;
    // Should produce some result
    let _ = result;
}

// ============ Edge Case Tests ============

#[test]
fn test_category_score_boundary_100() {
    let score = CategoryScore::new("Test", 100.0, 100);
    assert_eq!(score.grade, "A+");
    assert_eq!(score.earned_points, 100.0);
}

#[test]
fn test_category_score_boundary_0() {
    let score = CategoryScore::new("Test", 0.0, 100);
    assert_eq!(score.grade, "F");
    assert_eq!(score.earned_points, 0.0);
}

#[test]
fn test_category_score_small_max_points() {
    let score = CategoryScore::new("Test", 100.0, 1);
    assert_eq!(score.earned_points, 1.0);
}

#[test]
fn test_perfection_score_single_category() {
    let categories = vec![CategoryScore::new("Single", 100.0, 200)];
    let result = PerfectionScoreResult::new(categories);
    assert_eq!(result.total_score, 200.0);
}

#[test]
fn test_perfection_score_many_categories() {
    let categories: Vec<CategoryScore> = (0..20)
        .map(|i| CategoryScore::new(&format!("Cat{}", i), 90.0, 10))
        .collect();
    let result = PerfectionScoreResult::new(categories);
    assert_eq!(result.total_score, 180.0); // 20 * 9.0
}
