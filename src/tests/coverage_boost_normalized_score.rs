#![cfg_attr(coverage_nightly, coverage(off))]
//! Coverage boost tests for services/normalized_score.rs
//! Tests: Grade, SimpleScore, NormalizedScore trait

use crate::services::normalized_score::{Grade, NormalizedScore, SimpleScore};

// ============ Grade Tests ============

#[test]
fn test_grade_from_score_a() {
    assert_eq!(Grade::from_score(95.0), Grade::A);
    assert_eq!(Grade::from_score(90.0), Grade::A);
    assert_eq!(Grade::from_score(100.0), Grade::A);
}

#[test]
fn test_grade_from_score_b() {
    assert_eq!(Grade::from_score(85.0), Grade::B);
    assert_eq!(Grade::from_score(80.0), Grade::B);
    assert_eq!(Grade::from_score(89.9), Grade::B);
}

#[test]
fn test_grade_from_score_c() {
    assert_eq!(Grade::from_score(75.0), Grade::C);
    assert_eq!(Grade::from_score(70.0), Grade::C);
}

#[test]
fn test_grade_from_score_d() {
    assert_eq!(Grade::from_score(65.0), Grade::D);
    assert_eq!(Grade::from_score(60.0), Grade::D);
}

#[test]
fn test_grade_from_score_f() {
    assert_eq!(Grade::from_score(50.0), Grade::F);
    assert_eq!(Grade::from_score(0.0), Grade::F);
    assert_eq!(Grade::from_score(59.9), Grade::F);
}

#[test]
fn test_grade_min_score() {
    assert_eq!(Grade::A.min_score(), 90.0);
    assert_eq!(Grade::B.min_score(), 80.0);
    assert_eq!(Grade::C.min_score(), 70.0);
    assert_eq!(Grade::D.min_score(), 60.0);
    assert_eq!(Grade::F.min_score(), 0.0);
}

#[test]
fn test_grade_description() {
    assert!(Grade::A.description().contains("Excellent"));
    assert!(Grade::B.description().contains("Good"));
    assert!(Grade::C.description().contains("Satisfactory"));
    assert!(Grade::D.description().contains("Needs Improvement"));
    assert!(Grade::F.description().contains("Failing"));
}

#[test]
fn test_grade_display() {
    assert_eq!(format!("{}", Grade::A), "A");
    assert_eq!(format!("{}", Grade::B), "B");
    assert_eq!(format!("{}", Grade::C), "C");
    assert_eq!(format!("{}", Grade::D), "D");
    assert_eq!(format!("{}", Grade::F), "F");
}

#[test]
fn test_grade_ordering() {
    assert!(Grade::A > Grade::B);
    assert!(Grade::B > Grade::C);
    assert!(Grade::C > Grade::D);
    assert!(Grade::D > Grade::F);
}

#[test]
fn test_grade_equality() {
    assert_eq!(Grade::A, Grade::A);
    assert_ne!(Grade::A, Grade::B);
}

#[test]
fn test_grade_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(Grade::A);
    set.insert(Grade::B);
    set.insert(Grade::A); // duplicate
    assert_eq!(set.len(), 2);
}

#[test]
fn test_grade_clone_copy() {
    let grade = Grade::A;
    let copied = grade;
    let cloned = grade;
    assert_eq!(grade, copied);
    assert_eq!(grade, cloned);
}

// ============ SimpleScore Tests ============

#[test]
fn test_simple_score_new() {
    let score = SimpleScore::new(80.0, 100.0, "test");
    assert_eq!(score.raw(), 80.0);
    assert_eq!(score.max_raw(), 100.0);
    assert!((score.normalized() - 80.0).abs() < 0.01);
}

#[test]
fn test_simple_score_perfect() {
    let score = SimpleScore::new(100.0, 100.0, "perfect");
    assert!((score.normalized() - 100.0).abs() < 0.01);
    assert_eq!(score.grade(), Grade::A);
}

#[test]
fn test_simple_score_zero() {
    let score = SimpleScore::new(0.0, 100.0, "zero");
    assert!((score.normalized() - 0.0).abs() < 0.01);
    assert_eq!(score.grade(), Grade::F);
}

#[test]
fn test_simple_score_from_percentage() {
    let score = SimpleScore::from_percentage(85.0, "pct");
    assert!((score.normalized() - 85.0).abs() < 0.01);
    assert_eq!(score.grade(), Grade::B);
}

#[test]
fn test_simple_score_from_percentage_clamped() {
    let score = SimpleScore::from_percentage(150.0, "over");
    assert!((score.normalized() - 100.0).abs() < 0.01);

    let score2 = SimpleScore::from_percentage(-50.0, "under");
    assert!((score2.normalized() - 0.0).abs() < 0.01);
}

#[test]
fn test_simple_score_different_scale() {
    let score = SimpleScore::new(50.0, 200.0, "half");
    assert!((score.normalized() - 25.0).abs() < 0.01);
}

#[test]
fn test_simple_score_meets_threshold() {
    let score = SimpleScore::new(85.0, 100.0, "test");
    assert!(score.meets_threshold(80.0));
    assert!(score.meets_threshold(85.0));
    assert!(!score.meets_threshold(90.0));
}

#[test]
fn test_simple_score_display() {
    let score = SimpleScore::new(90.0, 100.0, "Quality");
    let display = format!("{}", score);
    assert!(display.contains("Quality"));
    assert!(display.contains("90"));
}

#[test]
fn test_simple_score_negative_raw_clamped() {
    let score = SimpleScore::new(-10.0, 100.0, "neg");
    assert_eq!(score.raw(), 0.0); // raw is clamped to 0
    assert!((score.normalized() - 0.0).abs() < 0.01);
}
