#![cfg_attr(coverage_nightly, coverage(off))]
//! Coverage boost tests for services/ranking.rs
//! Tests: CompositeComplexityScore, ChurnScore, DuplicationScore,
//! rank_files_vectorized, ComplexityRanker

use crate::services::ranking::{
    ChurnScore, ComplexityRanker, CompositeComplexityScore, DuplicationScore,
};

// ============ CompositeComplexityScore Tests ============

#[test]
fn test_composite_complexity_score_default() {
    let score = CompositeComplexityScore::default();
    assert_eq!(score.cyclomatic_max, 0);
    assert!((score.cognitive_avg - 0.0).abs() < f64::EPSILON);
    assert!((score.halstead_effort - 0.0).abs() < f64::EPSILON);
    assert_eq!(score.function_count, 0);
    assert!((score.total_score - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_composite_complexity_score_partial_eq() {
    let a = CompositeComplexityScore {
        total_score: 50.0,
        ..Default::default()
    };
    let b = CompositeComplexityScore {
        total_score: 50.0,
        cyclomatic_max: 10, // Different but total_score same
        ..Default::default()
    };
    assert_eq!(a, b); // PartialEq based on total_score
}

#[test]
fn test_composite_complexity_score_partial_ne() {
    let a = CompositeComplexityScore {
        total_score: 50.0,
        ..Default::default()
    };
    let b = CompositeComplexityScore {
        total_score: 75.0,
        ..Default::default()
    };
    assert_ne!(a, b);
}

#[test]
fn test_composite_complexity_score_partial_ord() {
    let low = CompositeComplexityScore {
        total_score: 25.0,
        ..Default::default()
    };
    let high = CompositeComplexityScore {
        total_score: 75.0,
        ..Default::default()
    };
    assert!(low < high);
    assert!(high > low);
}

#[test]
fn test_composite_complexity_score_serde() {
    let score = CompositeComplexityScore {
        cyclomatic_max: 15,
        cognitive_avg: 8.5,
        halstead_effort: 1200.0,
        function_count: 10,
        total_score: 65.3,
    };
    let json = serde_json::to_string(&score).unwrap();
    let back: CompositeComplexityScore = serde_json::from_str(&json).unwrap();
    assert_eq!(back.cyclomatic_max, 15);
    assert!((back.total_score - 65.3).abs() < f64::EPSILON);
}

#[test]
fn test_composite_complexity_score_clone() {
    let score = CompositeComplexityScore {
        total_score: 42.0,
        function_count: 5,
        ..Default::default()
    };
    let cloned = score.clone();
    assert!((cloned.total_score - 42.0).abs() < f64::EPSILON);
    assert_eq!(cloned.function_count, 5);
}

// ============ ChurnScore Tests ============

#[test]
fn test_churn_score_default() {
    let score = ChurnScore::default();
    assert_eq!(score.commit_count, 0);
    assert_eq!(score.unique_authors, 0);
    assert_eq!(score.lines_changed, 0);
    assert!((score.recency_weight - 0.0).abs() < f64::EPSILON);
    assert!((score.score - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_churn_score_partial_eq() {
    let a = ChurnScore {
        score: 10.0,
        ..Default::default()
    };
    let b = ChurnScore {
        score: 10.0,
        commit_count: 5,
        ..Default::default()
    };
    assert_eq!(a, b);
}

#[test]
fn test_churn_score_partial_ord() {
    let low = ChurnScore {
        score: 5.0,
        ..Default::default()
    };
    let high = ChurnScore {
        score: 15.0,
        ..Default::default()
    };
    assert!(low < high);
}

#[test]
fn test_churn_score_serde() {
    let score = ChurnScore {
        commit_count: 50,
        unique_authors: 3,
        lines_changed: 1500,
        recency_weight: 0.8,
        score: 25.5,
    };
    let json = serde_json::to_string(&score).unwrap();
    let back: ChurnScore = serde_json::from_str(&json).unwrap();
    assert_eq!(back.commit_count, 50);
    assert!((back.score - 25.5).abs() < f64::EPSILON);
}

// ============ DuplicationScore Tests ============

#[test]
fn test_duplication_score_default() {
    let score = DuplicationScore::default();
    assert_eq!(score.exact_clones, 0);
    assert_eq!(score.renamed_clones, 0);
    assert_eq!(score.gapped_clones, 0);
    assert_eq!(score.semantic_clones, 0);
    assert!((score.duplication_ratio - 0.0).abs() < f64::EPSILON);
    assert!((score.score - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_duplication_score_partial_eq() {
    let a = DuplicationScore {
        score: 3.5,
        ..Default::default()
    };
    let b = DuplicationScore {
        score: 3.5,
        exact_clones: 10,
        ..Default::default()
    };
    assert_eq!(a, b);
}

#[test]
fn test_duplication_score_partial_ord() {
    let low = DuplicationScore {
        score: 1.0,
        ..Default::default()
    };
    let high = DuplicationScore {
        score: 8.0,
        ..Default::default()
    };
    assert!(low < high);
}

#[test]
fn test_duplication_score_serde() {
    let score = DuplicationScore {
        exact_clones: 5,
        renamed_clones: 3,
        gapped_clones: 2,
        semantic_clones: 1,
        duplication_ratio: 0.15,
        score: 7.2,
    };
    let json = serde_json::to_string(&score).unwrap();
    let back: DuplicationScore = serde_json::from_str(&json).unwrap();
    assert_eq!(back.exact_clones, 5);
    assert_eq!(back.semantic_clones, 1);
}

// ============ rank_files_vectorized Tests ============

#[test]
fn test_rank_files_vectorized_basic() {
    let scores = vec![0.3_f32, 0.9, 0.1, 0.7, 0.5];
    let ranked = crate::services::ranking::rank_files_vectorized(&scores, 3);
    assert_eq!(ranked.len(), 3);
    assert_eq!(ranked[0], 1); // 0.9 is highest
    assert_eq!(ranked[1], 3); // 0.7 is second
    assert_eq!(ranked[2], 4); // 0.5 is third
}

#[test]
fn test_rank_files_vectorized_empty() {
    let scores: Vec<f32> = vec![];
    let ranked = crate::services::ranking::rank_files_vectorized(&scores, 5);
    assert!(ranked.is_empty());
}

#[test]
fn test_rank_files_vectorized_limit_exceeds() {
    let scores = vec![1.0_f32, 2.0];
    let ranked = crate::services::ranking::rank_files_vectorized(&scores, 10);
    assert_eq!(ranked.len(), 2);
}

#[test]
fn test_rank_files_vectorized_limit_zero() {
    let scores = vec![1.0_f32, 2.0, 3.0];
    let ranked = crate::services::ranking::rank_files_vectorized(&scores, 0);
    assert!(ranked.is_empty());
}

#[test]
fn test_rank_files_vectorized_single() {
    let scores = vec![42.0_f32];
    let ranked = crate::services::ranking::rank_files_vectorized(&scores, 1);
    assert_eq!(ranked.len(), 1);
    assert_eq!(ranked[0], 0);
}

#[test]
fn test_rank_files_vectorized_equal_scores() {
    let scores = vec![5.0_f32, 5.0, 5.0];
    let ranked = crate::services::ranking::rank_files_vectorized(&scores, 2);
    assert_eq!(ranked.len(), 2);
}

// ============ ComplexityRanker Tests ============

#[test]
fn test_complexity_ranker_default() {
    let ranker = ComplexityRanker::default();
    assert!((ranker.cyclomatic_weight - 0.4).abs() < f64::EPSILON);
    assert!((ranker.cognitive_weight - 0.4).abs() < f64::EPSILON);
    assert!((ranker.function_count_weight - 0.2).abs() < f64::EPSILON);
}

#[test]
fn test_complexity_ranker_new() {
    let ranker = ComplexityRanker::new(0.5, 0.3, 0.2);
    assert!((ranker.cyclomatic_weight - 0.5).abs() < f64::EPSILON);
    assert!((ranker.cognitive_weight - 0.3).abs() < f64::EPSILON);
    assert!((ranker.function_count_weight - 0.2).abs() < f64::EPSILON);
}

#[test]
fn test_complexity_ranker_ranking_type() {
    use crate::services::ranking::FileRanker;
    let ranker = ComplexityRanker::default();
    assert_eq!(ranker.ranking_type(), "Complexity");
}
