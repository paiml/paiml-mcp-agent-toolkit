//! RED Phase Tests for Rust Project Score v1.1
//!
//! Following EXTREME TDD methodology:
//! - RED: Write failing tests first
//! - GREEN: Minimal implementation to make tests pass
//! - REFACTOR: Clean up and optimize
//!
//! These tests define the core data structures and behavior for the
//! Rust Project Score v1.1 specification (106 points total, 6 categories).

use pmat::services::rust_project_score::models::*;
use pmat::services::rust_project_score::*;

// ============================================================================
// RED Test 1: RustProjectScore Creation
// ============================================================================

#[test]
fn test_rust_project_score_creation() {
    let score = RustProjectScore::new();
    assert_eq!(score.total_score, 0.0);
    assert_eq!(score.grade, Grade::F);
    assert!(score.velocity.is_none());
}

// ============================================================================
// RED Test 2: Grade Calculation (A+ tier)
// ============================================================================

#[test]
fn test_grade_calculation_a_plus() {
    // A+ is 95-106 points (89.6%+ of 106)
    let grade = Grade::from_score(100.0, 106.0);
    assert_eq!(grade, Grade::APlus);
}

#[test]
fn test_grade_calculation_a_plus_minimum() {
    // Minimum A+ is 95 points
    let grade = Grade::from_score(95.0, 106.0);
    assert_eq!(grade, Grade::APlus);
}

#[test]
fn test_grade_calculation_a() {
    // A is 90-94 points (84.9%-89.5%)
    let grade = Grade::from_score(92.0, 106.0);
    assert_eq!(grade, Grade::A);
}

#[test]
fn test_grade_calculation_a_minus() {
    // A- is 85-89 points (80.2%-84.8%)
    let grade = Grade::from_score(87.0, 106.0);
    assert_eq!(grade, Grade::AMinus);
}

#[test]
fn test_grade_calculation_b_plus() {
    // B+ is 80-84 points (75.5%-80.1%)
    let grade = Grade::from_score(82.0, 106.0);
    assert_eq!(grade, Grade::BPlus);
}

#[test]
fn test_grade_calculation_b() {
    // B is 70-79 points (66.0%-75.4%)
    let grade = Grade::from_score(75.0, 106.0);
    assert_eq!(grade, Grade::B);
}

#[test]
fn test_grade_calculation_c() {
    // C is 60-69 points
    let grade = Grade::from_score(65.0, 106.0);
    assert_eq!(grade, Grade::C);
}

#[test]
fn test_grade_calculation_d() {
    // D is 50-59 points
    let grade = Grade::from_score(55.0, 106.0);
    assert_eq!(grade, Grade::D);
}

#[test]
fn test_grade_calculation_f() {
    // F is 0-49 points
    let grade = Grade::from_score(30.0, 106.0);
    assert_eq!(grade, Grade::F);
}

// ============================================================================
// RED Test 3: CategoryScores Total Calculation
// ============================================================================

#[test]
fn test_category_scores_sum_to_total() {
    let categories = CategoryScores {
        rust_tooling: CategoryScore::new(25.0, 25.0),
        code_quality: CategoryScore::new(26.0, 26.0),
        testing: CategoryScore::new(20.0, 20.0),
        documentation: CategoryScore::new(15.0, 15.0),
        performance: CategoryScore::new(10.0, 10.0),
        dependencies: CategoryScore::new(10.0, 12.0),
    };
    assert_eq!(categories.total(), 106.0);
}

#[test]
fn test_category_scores_partial() {
    let categories = CategoryScores {
        rust_tooling: CategoryScore::new(20.0, 25.0),
        code_quality: CategoryScore::new(18.0, 26.0),
        testing: CategoryScore::new(15.0, 20.0),
        documentation: CategoryScore::new(10.0, 15.0),
        performance: CategoryScore::new(7.0, 10.0),
        dependencies: CategoryScore::new(8.0, 12.0),
    };
    assert_eq!(categories.total(), 78.0);
}

// ============================================================================
// RED Test 4: ScoreVelocity Calculation (NEW in v1.1 - Kaizen)
// ============================================================================

#[test]
fn test_score_velocity_calculation() {
    let velocity = ScoreVelocity::calculate(65.0, 78.0, 30);
    assert_eq!(velocity.current, 78.0);
    assert_eq!(velocity.previous, 65.0);
    assert_eq!(velocity.delta, 13.0);
    assert!((velocity.delta_percent - 20.0).abs() < 0.1);
    assert_eq!(velocity.days_elapsed, 30);
    assert!((velocity.points_per_day - 0.433).abs() < 0.01);
}

#[test]
fn test_score_velocity_negative() {
    let velocity = ScoreVelocity::calculate(85.0, 78.0, 15);
    assert_eq!(velocity.delta, -7.0);
    assert!((velocity.delta_percent - (-8.24)).abs() < 0.1);
    assert!((velocity.points_per_day - (-0.467)).abs() < 0.01);
}

#[test]
fn test_score_velocity_most_improved() {
    let mut velocity = ScoreVelocity::calculate(70.0, 85.0, 30);
    velocity.most_improved = Some("Testing".to_string());
    assert_eq!(velocity.most_improved.unwrap(), "Testing");
}

#[test]
fn test_score_velocity_days_to_next_grade() {
    let mut velocity = ScoreVelocity::calculate(82.0, 87.0, 30);
    // At 0.167 points/day, need 3 more points to reach A- (90)
    velocity.days_to_next_grade = Some(18);
    assert_eq!(velocity.days_to_next_grade.unwrap(), 18);
}

// ============================================================================
// RED Test 5: JSON Serialization
// ============================================================================

#[test]
fn test_score_serialization() {
    let score = RustProjectScore::new();
    let json = serde_json::to_string(&score).unwrap();
    assert!(json.contains("total_score"));
    assert!(json.contains("grade"));
}

#[test]
fn test_score_deserialization() {
    let json = r#"{
        "total_score": 85.0,
        "grade": "AMinus",
        "categories": {
            "rust_tooling": {"earned": 22.0, "max": 25.0},
            "code_quality": {"earned": 20.0, "max": 26.0},
            "testing": {"earned": 18.0, "max": 20.0},
            "documentation": {"earned": 12.0, "max": 15.0},
            "performance": {"earned": 8.0, "max": 10.0},
            "dependencies": {"earned": 5.0, "max": 12.0}
        },
        "recommendations": [],
        "metadata": {
            "timestamp": "2025-11-16T00:00:00Z",
            "project_name": "test",
            "version": "1.1.0"
        },
        "velocity": null
    }"#;
    let score: RustProjectScore = serde_json::from_str(json).unwrap();
    assert_eq!(score.total_score, 85.0);
    assert_eq!(score.grade, Grade::AMinus);
}

// ============================================================================
// RED Test 6: CategoryScore Individual Metrics
// ============================================================================

#[test]
fn test_category_score_percentage() {
    let score = CategoryScore::new(20.0, 25.0);
    assert!((score.percentage() - 80.0).abs() < 0.1);
}

#[test]
fn test_category_score_perfect() {
    let score = CategoryScore::new(25.0, 25.0);
    assert!(score.is_perfect());
    assert!((score.percentage() - 100.0).abs() < 0.1);
}

// ============================================================================
// RED Test 7: Grade Display and Formatting
// ============================================================================

#[test]
fn test_grade_display() {
    assert_eq!(format!("{}", Grade::APlus), "A+");
    assert_eq!(format!("{}", Grade::A), "A");
    assert_eq!(format!("{}", Grade::AMinus), "A-");
    assert_eq!(format!("{}", Grade::BPlus), "B+");
    assert_eq!(format!("{}", Grade::B), "B");
    assert_eq!(format!("{}", Grade::C), "C");
    assert_eq!(format!("{}", Grade::D), "D");
    assert_eq!(format!("{}", Grade::F), "F");
}

// ============================================================================
// RED Test 8: Recommendation System
// ============================================================================

#[test]
fn test_recommendation_creation() {
    let rec = Recommendation::new(
        "Testing".to_string(),
        "Increase test coverage to ≥85%".to_string(),
        RecommendationPriority::Critical,
        5.0, // potential points gain
    );
    assert_eq!(rec.category, "Testing");
    assert_eq!(rec.priority, RecommendationPriority::Critical);
    assert_eq!(rec.potential_points, 5.0);
}

#[test]
fn test_recommendation_sorting() {
    let mut recs = vec![
        Recommendation::new(
            "Testing".to_string(),
            "Fix".to_string(),
            RecommendationPriority::High,
            3.0,
        ),
        Recommendation::new(
            "Documentation".to_string(),
            "Fix".to_string(),
            RecommendationPriority::Critical,
            5.0,
        ),
        Recommendation::new(
            "Performance".to_string(),
            "Fix".to_string(),
            RecommendationPriority::Medium,
            2.0,
        ),
    ];
    recs.sort_by(|a, b| b.priority.cmp(&a.priority));
    assert_eq!(recs[0].priority, RecommendationPriority::Critical);
    assert_eq!(recs[1].priority, RecommendationPriority::High);
    assert_eq!(recs[2].priority, RecommendationPriority::Medium);
}

// ============================================================================
// RED Test 9: ScoreMetadata
// ============================================================================

#[test]
fn test_score_metadata_creation() {
    let metadata = ScoreMetadata::new("pmat".to_string(), "1.1.0".to_string());
    assert_eq!(metadata.project_name, "pmat");
    assert_eq!(metadata.version, "1.1.0");
}

// ============================================================================
// RED Test 10: Full Integration Test
// ============================================================================

#[test]
fn test_complete_score_with_velocity() {
    let categories = CategoryScores {
        rust_tooling: CategoryScore::new(23.0, 25.0),
        code_quality: CategoryScore::new(22.0, 26.0),
        testing: CategoryScore::new(18.0, 20.0),
        documentation: CategoryScore::new(13.0, 15.0),
        performance: CategoryScore::new(9.0, 10.0),
        dependencies: CategoryScore::new(10.0, 12.0),
    };

    let total = categories.total();
    assert_eq!(total, 95.0);

    let grade = Grade::from_score(total, 106.0);
    assert_eq!(grade, Grade::APlus);

    let velocity = ScoreVelocity::calculate(82.0, 95.0, 45);
    assert_eq!(velocity.delta, 13.0);
    assert!((velocity.points_per_day - 0.289).abs() < 0.01);
}
