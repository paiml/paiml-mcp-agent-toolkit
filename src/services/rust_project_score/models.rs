#![cfg_attr(coverage_nightly, coverage(off))]
//! Core data models for Rust Project Score v1.1
//!
//! This module defines the core types for the 106-point scoring system
//! with 6 categories following the evidence-based specification.
//!
//! All scores are normalized to 0-100 for display (PMAT-454).
//!
//! Evidence-based design from 15 peer-reviewed papers (2022-2025)

use crate::services::normalized_score::NormalizedScore;
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::{Path, PathBuf};

// ScoringMode + RUST_PROJECT_MAX_POINTS constant
include!("models_scoring_mode.rs");

// FileCache - in-memory file cache for project scoring
include!("models_file_cache.rs");

// RustProjectScore + Grade
include!("models_score.rs");

// CategoryScores + CategoryScore
include!("models_categories.rs");

// ScoreVelocity + Recommendation + RecommendationPriority + ScoreMetadata
include!("models_velocity.rs");

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ============================================================================
    // ScoringMode tests
    // ============================================================================

    #[test]
    fn test_scoring_mode_default() {
        let mode = ScoringMode::default();
        assert_eq!(mode, ScoringMode::Fast);
    }

    #[test]
    fn test_scoring_mode_quick_skip_subprocesses() {
        assert!(ScoringMode::Quick.skip_subprocesses());
        assert!(!ScoringMode::Fast.skip_subprocesses());
        assert!(!ScoringMode::Full.skip_subprocesses());
    }

    #[test]
    fn test_scoring_mode_skip_expensive_cargo() {
        assert!(ScoringMode::Quick.skip_expensive_cargo());
        assert!(ScoringMode::Fast.skip_expensive_cargo());
        assert!(!ScoringMode::Full.skip_expensive_cargo());
    }

    #[test]
    fn test_scoring_mode_is_full() {
        assert!(!ScoringMode::Quick.is_full());
        assert!(!ScoringMode::Fast.is_full());
        assert!(ScoringMode::Full.is_full());
    }

    #[test]
    fn test_scoring_mode_display() {
        assert_eq!(format!("{}", ScoringMode::Quick), "Quick (<10s)");
        assert_eq!(format!("{}", ScoringMode::Fast), "Fast (<60s)");
        assert_eq!(format!("{}", ScoringMode::Full), "Full (<5m)");
    }

    // ============================================================================
    // FileCache tests
    // ============================================================================

    #[test]
    fn test_file_cache_new() {
        let cache = FileCache::new();
        let (files, bytes) = cache.stats();
        assert_eq!(files, 0);
        assert_eq!(bytes, 0);
    }

    #[test]
    fn test_file_cache_default() {
        let cache = FileCache::default();
        let (files, _) = cache.stats();
        assert_eq!(files, 0);
    }

    #[test]
    fn test_file_cache_insert_and_get() {
        let mut cache = FileCache::new();
        let path = PathBuf::from("/test/file.rs");
        let content = "fn main() {}".to_string();

        cache.insert(path.clone(), content.clone());

        let retrieved = cache.get(&path);
        assert_eq!(retrieved, Some(&content));
    }

    #[test]
    fn test_file_cache_get_nonexistent() {
        let cache = FileCache::new();
        assert!(cache.get(&PathBuf::from("/nonexistent")).is_none());
    }

    #[test]
    fn test_file_cache_stats() {
        let mut cache = FileCache::new();
        cache.insert(PathBuf::from("/a.rs"), "hello".to_string());
        cache.insert(PathBuf::from("/b.rs"), "world!".to_string());

        let (files, bytes) = cache.stats();
        assert_eq!(files, 2);
        assert_eq!(bytes, 11); // "hello" (5) + "world!" (6)
    }

    #[test]
    fn test_file_cache_iter() {
        let mut cache = FileCache::new();
        cache.insert(PathBuf::from("/a.rs"), "a".to_string());
        cache.insert(PathBuf::from("/b.rs"), "b".to_string());

        let items: Vec<_> = cache.iter().collect();
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_file_cache_get_rust_files_in_dir() {
        let mut cache = FileCache::new();
        cache.insert(PathBuf::from("/src/main.rs"), "main".to_string());
        cache.insert(PathBuf::from("/src/lib.rs"), "lib".to_string());
        cache.insert(PathBuf::from("/tests/test.rs"), "test".to_string());

        let src_files = cache.get_rust_files_in_dir(&PathBuf::from("/src"));
        assert_eq!(src_files.len(), 2);
    }

    #[test]
    fn test_file_cache_age() {
        let cache = FileCache::new();
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert!(cache.age_ms() >= 10);
    }

    #[test]
    fn test_file_cache_populate_empty_dir() {
        let temp = TempDir::new().unwrap();
        let cache = FileCache::populate(temp.path()).unwrap();
        let (files, _) = cache.stats();
        assert_eq!(files, 0);
    }

    #[test]
    fn test_file_cache_populate_with_cargo_toml() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("Cargo.toml"), "[package]").unwrap();

        let cache = FileCache::populate(temp.path()).unwrap();
        assert!(cache.get(&temp.path().join("Cargo.toml")).is_some());
    }

    #[test]
    fn test_file_cache_populate_with_readme() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("README.md"), "# Project").unwrap();

        let cache = FileCache::populate(temp.path()).unwrap();
        assert!(cache.get(&temp.path().join("README.md")).is_some());
    }

    #[test]
    fn test_file_cache_populate_with_src() {
        let temp = TempDir::new().unwrap();
        let src_dir = temp.path().join("src");
        std::fs::create_dir(&src_dir).unwrap();
        std::fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();

        let cache = FileCache::populate(temp.path()).unwrap();
        let src_files = cache.get_rust_files_in_dir(&src_dir);
        assert_eq!(src_files.len(), 1);
    }

    // ============================================================================
    // Grade tests
    // ============================================================================

    // PMAT-454: Tests now verify NORMALIZED grading (0-100%)
    // Raw score 95/106 = 89.6% → A (not A+)
    // Raw score 100/106 = 94.3% → A (not A+)

    #[test]
    fn test_grade_from_score_a_plus() {
        // A+ requires 95%+ normalized
        assert_eq!(Grade::from_score(100.7, 106.0), Grade::APlus); // 95%
        assert_eq!(Grade::from_score(106.0, 106.0), Grade::APlus); // 100%
                                                                   // Perfect score on 100-point scale
        assert_eq!(Grade::from_score(95.0, 100.0), Grade::APlus);
        assert_eq!(Grade::from_score(100.0, 100.0), Grade::APlus);
    }

    #[test]
    fn test_grade_from_score_a() {
        // A requires 90-94% normalized
        assert_eq!(Grade::from_score(95.4, 106.0), Grade::A); // 90%
        assert_eq!(Grade::from_score(99.6, 106.0), Grade::A); // 94%
        assert_eq!(Grade::from_score(90.0, 100.0), Grade::A);
        assert_eq!(Grade::from_score(94.9, 100.0), Grade::A);
    }

    #[test]
    fn test_grade_from_score_a_minus() {
        // A- requires 85-89% normalized
        assert_eq!(Grade::from_score(90.1, 106.0), Grade::AMinus); // 85%
        assert_eq!(Grade::from_score(94.3, 106.0), Grade::AMinus); // 89%
        assert_eq!(Grade::from_score(85.0, 100.0), Grade::AMinus);
        assert_eq!(Grade::from_score(89.9, 100.0), Grade::AMinus);
    }

    #[test]
    fn test_grade_from_score_b_plus() {
        // B+ requires 80-84% normalized
        assert_eq!(Grade::from_score(84.8, 106.0), Grade::BPlus); // 80%
        assert_eq!(Grade::from_score(89.0, 106.0), Grade::BPlus); // ~84%
        assert_eq!(Grade::from_score(80.0, 100.0), Grade::BPlus);
        assert_eq!(Grade::from_score(84.9, 100.0), Grade::BPlus);
    }

    #[test]
    fn test_grade_from_score_b() {
        // B requires 70-79% normalized
        assert_eq!(Grade::from_score(74.2, 106.0), Grade::B); // 70%
        assert_eq!(Grade::from_score(83.7, 106.0), Grade::B); // 79%
        assert_eq!(Grade::from_score(70.0, 100.0), Grade::B);
        assert_eq!(Grade::from_score(79.9, 100.0), Grade::B);
    }

    #[test]
    fn test_grade_from_score_c() {
        // C requires 60-69% normalized
        assert_eq!(Grade::from_score(63.6, 106.0), Grade::C); // 60%
        assert_eq!(Grade::from_score(73.1, 106.0), Grade::C); // 69%
        assert_eq!(Grade::from_score(60.0, 100.0), Grade::C);
        assert_eq!(Grade::from_score(69.9, 100.0), Grade::C);
    }

    #[test]
    fn test_grade_from_score_d() {
        // D requires 50-59% normalized
        assert_eq!(Grade::from_score(53.0, 106.0), Grade::D); // 50%
        assert_eq!(Grade::from_score(62.5, 106.0), Grade::D); // 59%
        assert_eq!(Grade::from_score(50.0, 100.0), Grade::D);
        assert_eq!(Grade::from_score(59.9, 100.0), Grade::D);
    }

    #[test]
    fn test_grade_from_score_f() {
        // F is below 50% normalized
        assert_eq!(Grade::from_score(52.0, 106.0), Grade::F); // 49%
        assert_eq!(Grade::from_score(0.0, 106.0), Grade::F); // 0%
        assert_eq!(Grade::from_score(49.9, 100.0), Grade::F);
        assert_eq!(Grade::from_score(0.0, 100.0), Grade::F);
    }

    #[test]
    fn test_grade_from_normalized() {
        // Direct normalized percentage input
        assert_eq!(Grade::from_normalized(100.0), Grade::APlus);
        assert_eq!(Grade::from_normalized(95.0), Grade::APlus);
        assert_eq!(Grade::from_normalized(90.0), Grade::A);
        assert_eq!(Grade::from_normalized(85.0), Grade::AMinus);
        assert_eq!(Grade::from_normalized(80.0), Grade::BPlus);
        assert_eq!(Grade::from_normalized(70.0), Grade::B);
        assert_eq!(Grade::from_normalized(60.0), Grade::C);
        assert_eq!(Grade::from_normalized(50.0), Grade::D);
        assert_eq!(Grade::from_normalized(0.0), Grade::F);
    }

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
    // CategoryScore tests
    // ============================================================================

    #[test]
    fn test_category_score_new() {
        let score = CategoryScore::new(15.0, 25.0);
        assert_eq!(score.earned, 15.0);
        assert_eq!(score.max, 25.0);
    }

    #[test]
    fn test_category_score_percentage() {
        let score = CategoryScore::new(50.0, 100.0);
        assert_eq!(score.percentage(), 50.0);
    }

    #[test]
    fn test_category_score_percentage_zero_max() {
        let score = CategoryScore::new(0.0, 0.0);
        assert_eq!(score.percentage(), 0.0);
    }

    #[test]
    fn test_category_score_is_perfect() {
        let perfect = CategoryScore::new(25.0, 25.0);
        assert!(perfect.is_perfect());

        let not_perfect = CategoryScore::new(24.0, 25.0);
        assert!(!not_perfect.is_perfect());
    }

    // ============================================================================
    // CategoryScores tests
    // ============================================================================

    #[test]
    fn test_category_scores_default() {
        let scores = CategoryScores::default();
        assert_eq!(scores.rust_tooling.max, 25.0);
        assert_eq!(scores.code_quality.max, 26.0);
        assert_eq!(scores.testing.max, 20.0);
        assert_eq!(scores.documentation.max, 15.0);
        assert_eq!(scores.performance.max, 10.0);
        assert_eq!(scores.dependencies.max, 12.0);
    }

    #[test]
    fn test_category_scores_total() {
        let mut scores = CategoryScores::default();
        scores.rust_tooling.earned = 20.0;
        scores.code_quality.earned = 22.0;
        scores.testing.earned = 15.0;
        scores.documentation.earned = 10.0;
        scores.performance.earned = 8.0;
        scores.dependencies.earned = 10.0;

        assert_eq!(scores.total(), 85.0);
    }

    // ============================================================================
    // RustProjectScore tests
    // ============================================================================

    #[test]
    fn test_rust_project_score_new() {
        let score = RustProjectScore::new();
        assert_eq!(score.total_score, 0.0);
        assert_eq!(score.grade, Grade::F);
        assert!(score.recommendations.is_empty());
    }

    #[test]
    fn test_rust_project_score_default() {
        let score = RustProjectScore::default();
        assert_eq!(score.total_score, 0.0);
    }

    // ============================================================================
    // ScoreVelocity tests
    // ============================================================================

    #[test]
    fn test_score_velocity_calculate() {
        let velocity = ScoreVelocity::calculate(50.0, 75.0, 10);

        assert_eq!(velocity.previous, 50.0);
        assert_eq!(velocity.current, 75.0);
        assert_eq!(velocity.delta, 25.0);
        assert_eq!(velocity.delta_percent, 50.0);
        assert_eq!(velocity.days_elapsed, 10);
        assert_eq!(velocity.points_per_day, 2.5);
    }

    #[test]
    fn test_score_velocity_zero_days() {
        let velocity = ScoreVelocity::calculate(50.0, 75.0, 0);
        assert_eq!(velocity.points_per_day, 0.0);
    }

    #[test]
    fn test_score_velocity_zero_previous() {
        let velocity = ScoreVelocity::calculate(0.0, 50.0, 5);
        assert_eq!(velocity.delta_percent, 0.0);
    }

    #[test]
    fn test_score_velocity_negative_delta() {
        let velocity = ScoreVelocity::calculate(80.0, 70.0, 5);
        assert_eq!(velocity.delta, -10.0);
    }

    // ============================================================================
    // RecommendationPriority tests
    // ============================================================================

    #[test]
    fn test_recommendation_priority_ordering() {
        assert!(RecommendationPriority::Low < RecommendationPriority::Medium);
        assert!(RecommendationPriority::Medium < RecommendationPriority::High);
        assert!(RecommendationPriority::High < RecommendationPriority::Critical);
    }

    #[test]
    fn test_recommendation_priority_equality() {
        assert_eq!(RecommendationPriority::High, RecommendationPriority::High);
        assert_ne!(RecommendationPriority::High, RecommendationPriority::Low);
    }

    // ============================================================================
    // Recommendation tests
    // ============================================================================

    #[test]
    fn test_recommendation_new() {
        let rec = Recommendation::new(
            "Testing".to_string(),
            "Add more unit tests".to_string(),
            RecommendationPriority::High,
            5.0,
        );

        assert_eq!(rec.category, "Testing");
        assert_eq!(rec.description, "Add more unit tests");
        assert_eq!(rec.priority, RecommendationPriority::High);
        assert_eq!(rec.potential_points, 5.0);
    }

    // ============================================================================
    // ScoreMetadata tests
    // ============================================================================

    #[test]
    fn test_score_metadata_new() {
        let meta = ScoreMetadata::new("my-project".to_string(), "1.1.0".to_string());

        assert_eq!(meta.project_name, "my-project");
        assert_eq!(meta.version, "1.1.0");
        assert!(!meta.timestamp.is_empty());
    }

    #[test]
    fn test_score_metadata_timestamp_format() {
        let meta = ScoreMetadata::new("test".to_string(), "1.0".to_string());
        // RFC3339 format should contain 'T' and timezone
        assert!(meta.timestamp.contains('T'));
    }

    // ============================================================================
    // Serialization tests
    // ============================================================================

    #[test]
    fn test_scoring_mode_serialization() {
        let mode = ScoringMode::Full;
        let json = serde_json::to_string(&mode).unwrap();
        let deserialized: ScoringMode = serde_json::from_str(&json).unwrap();
        assert_eq!(mode, deserialized);
    }

    #[test]
    fn test_grade_serialization() {
        let grade = Grade::APlus;
        let json = serde_json::to_string(&grade).unwrap();
        let deserialized: Grade = serde_json::from_str(&json).unwrap();
        assert_eq!(grade, deserialized);
    }

    #[test]
    fn test_category_score_serialization() {
        let score = CategoryScore::new(20.0, 25.0);
        let json = serde_json::to_string(&score).unwrap();
        let deserialized: CategoryScore = serde_json::from_str(&json).unwrap();
        assert_eq!(score.earned, deserialized.earned);
        assert_eq!(score.max, deserialized.max);
    }

    #[test]
    fn test_recommendation_serialization() {
        let rec = Recommendation::new(
            "Docs".to_string(),
            "Add README".to_string(),
            RecommendationPriority::Medium,
            2.0,
        );
        let json = serde_json::to_string(&rec).unwrap();
        let deserialized: Recommendation = serde_json::from_str(&json).unwrap();
        assert_eq!(rec.category, deserialized.category);
    }

    #[test]
    fn test_rust_project_score_serialization() {
        let score = RustProjectScore::new();
        let json = serde_json::to_string(&score).unwrap();
        assert!(json.contains("total_score"));
        assert!(json.contains("grade"));
    }
}
