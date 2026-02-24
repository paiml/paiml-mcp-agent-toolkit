// ==========================================================================
// Scorer Trait Tests
// ==========================================================================

mod scorer_trait_tests {
    use super::*;

    #[test]
    fn test_scorer_name() {
        let scorer = MockScorer::new("RustTooling", 25.0, 20.0);
        assert_eq!(scorer.name(), "RustTooling");
    }

    #[test]
    fn test_scorer_max_points() {
        let scorer = MockScorer::new("Testing", 20.0, 15.0);
        assert_eq!(scorer.max_points(), 20.0);
    }

    #[test]
    fn test_scorer_score_uses_default_mode() {
        let scorer = MockScorer::new("CodeQuality", 26.0, 22.0);
        let path = PathBuf::from("/tmp/test");
        let result = scorer.score(&path);
        assert!(result.is_ok());
        let score = result.unwrap();
        assert_eq!(score.earned, 22.0);
        assert_eq!(score.max, 26.0);
    }

    #[test]
    fn test_scorer_score_with_mode_quick() {
        let scorer = MockScorer::new("Performance", 10.0, 8.0);
        let path = PathBuf::from("/tmp/test");
        let result = scorer.score_with_mode(&path, ScoringMode::Quick);
        assert!(result.is_ok());
    }

    #[test]
    fn test_scorer_score_with_mode_fast() {
        let scorer = MockScorer::new("Dependencies", 12.0, 10.0);
        let path = PathBuf::from("/tmp/test");
        let result = scorer.score_with_mode(&path, ScoringMode::Fast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_scorer_score_with_mode_full() {
        let scorer = MockScorer::new("Documentation", 15.0, 12.0);
        let path = PathBuf::from("/tmp/test");
        let result = scorer.score_with_mode(&path, ScoringMode::Full);
        assert!(result.is_ok());
    }

    #[test]
    fn test_scorer_default_recommendations() {
        let scorer = MockScorer::new("Test", 10.0, 5.0);
        let path = PathBuf::from("/tmp/test");
        let recommendations = scorer.recommendations(&path);
        assert!(recommendations.is_empty());
    }

    #[test]
    fn test_scorer_custom_recommendations() {
        let scorer = RecommendingScorer::new(vec![
            "Add more tests".to_string(),
            "Improve documentation".to_string(),
        ]);
        let path = PathBuf::from("/tmp/test");
        let recommendations = scorer.recommendations(&path);
        assert_eq!(recommendations.len(), 2);
        assert!(recommendations.contains(&"Add more tests".to_string()));
        assert!(recommendations.contains(&"Improve documentation".to_string()));
    }

    #[test]
    fn test_scorer_empty_recommendations() {
        let scorer = RecommendingScorer::new(vec![]);
        let path = PathBuf::from("/tmp/test");
        let recommendations = scorer.recommendations(&path);
        assert!(recommendations.is_empty());
    }

    #[test]
    fn test_scorer_default_score_with_cache() {
        let scorer = MockScorer::new("Test", 10.0, 8.0);
        let path = PathBuf::from("/tmp/test");
        let cache = FileCache::new();

        // Default implementation ignores cache
        let result = scorer.score_with_cache(&path, ScoringMode::Fast, Some(&cache));
        assert!(result.is_ok());
        let score = result.unwrap();
        assert_eq!(score.earned, 8.0);
    }

    #[test]
    fn test_scorer_custom_score_with_cache() {
        let scorer = CacheAwareScorer::new(true);
        let path = PathBuf::from("/tmp/test");
        let cache = FileCache::new();

        // With cache, should return higher score
        let result = scorer.score_with_cache(&path, ScoringMode::Fast, Some(&cache));
        assert!(result.is_ok());
        let score = result.unwrap();
        assert_eq!(score.earned, 15.0);

        // Without cache, should return lower score
        let result = scorer.score_with_cache(&path, ScoringMode::Fast, None);
        assert!(result.is_ok());
        let score = result.unwrap();
        assert_eq!(score.earned, 10.0);
    }

    #[test]
    fn test_scorer_with_cache_none() {
        let scorer = MockScorer::new("Test", 10.0, 8.0);
        let path = PathBuf::from("/tmp/test");

        let result = scorer.score_with_cache(&path, ScoringMode::Fast, None);
        assert!(result.is_ok());
    }
}

// ==========================================================================
// Scorer Error Handling Tests
// ==========================================================================

mod scorer_error_handling_tests {
    use super::*;

    #[test]
    fn test_scorer_command_error() {
        let scorer = FailingScorer::new("command");
        let path = PathBuf::from("/tmp/test");
        let result = scorer.score(&path);
        assert!(result.is_err());
        assert!(result.unwrap_err().is_command_error());
    }

    #[test]
    fn test_scorer_parse_error() {
        let scorer = FailingScorer::new("parse");
        let path = PathBuf::from("/tmp/test");
        let result = scorer.score(&path);
        assert!(result.is_err());
        assert!(result.unwrap_err().is_parse_error());
    }

    #[test]
    fn test_scorer_tool_not_found() {
        let scorer = FailingScorer::new("tool");
        let path = PathBuf::from("/tmp/test");
        let result = scorer.score(&path);
        assert!(result.is_err());
        assert!(result.unwrap_err().is_tool_not_found());
    }

    #[test]
    fn test_scorer_invalid_project() {
        let scorer = FailingScorer::new("project");
        let path = PathBuf::from("/tmp/test");
        let result = scorer.score(&path);
        assert!(result.is_err());
        assert!(result.unwrap_err().is_invalid_project());
    }

    #[test]
    fn test_scorer_io_error() {
        let scorer = FailingScorer::new("io");
        let path = PathBuf::from("/tmp/test");
        let result = scorer.score(&path);
        assert!(result.is_err());
        assert!(result.unwrap_err().is_io_error());
    }
}

// ==========================================================================
// Scorer Thread Safety Tests
// ==========================================================================

mod scorer_thread_safety_tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_scorer_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<MockScorer>();
    }

    #[test]
    fn test_scorer_is_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<MockScorer>();
    }

    #[test]
    fn test_scorer_can_be_shared_across_threads() {
        let scorer = Arc::new(MockScorer::new("ThreadSafe", 10.0, 8.0));
        let path = PathBuf::from("/tmp/test");

        let handles: Vec<_> = (0..4)
            .map(|_| {
                let scorer = Arc::clone(&scorer);
                let path = path.clone();
                thread::spawn(move || scorer.score(&path))
            })
            .collect();

        for handle in handles {
            let result = handle.join().unwrap();
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_scorer_trait_object() {
        let scorer: Box<dyn Scorer> = Box::new(MockScorer::new("TraitObject", 10.0, 7.0));
        let path = PathBuf::from("/tmp/test");
        let result = scorer.score(&path);
        assert!(result.is_ok());
        assert_eq!(scorer.name(), "TraitObject");
        assert_eq!(scorer.max_points(), 10.0);
    }
}

// ==========================================================================
// Edge Case Tests
// ==========================================================================

mod edge_case_tests {
    use super::*;

    #[test]
    fn test_scorer_with_zero_max_points() {
        let scorer = MockScorer::new("ZeroMax", 0.0, 0.0);
        assert_eq!(scorer.max_points(), 0.0);
        let path = PathBuf::from("/tmp/test");
        let result = scorer.score(&path);
        assert!(result.is_ok());
        let score = result.unwrap();
        assert_eq!(score.max, 0.0);
    }

    #[test]
    fn test_scorer_with_negative_points() {
        // Edge case: negative points (should be avoided in practice)
        let scorer = MockScorer::new("Negative", 10.0, -5.0);
        let path = PathBuf::from("/tmp/test");
        let result = scorer.score(&path);
        assert!(result.is_ok());
        let score = result.unwrap();
        assert_eq!(score.earned, -5.0);
    }

    #[test]
    fn test_scorer_with_fractional_points() {
        let scorer = MockScorer::new("Fractional", 25.5, 17.25);
        let path = PathBuf::from("/tmp/test");
        let result = scorer.score(&path);
        assert!(result.is_ok());
        let score = result.unwrap();
        assert!((score.earned - 17.25).abs() < 0.001);
        assert!((score.max - 25.5).abs() < 0.001);
    }

    #[test]
    fn test_scorer_with_large_points() {
        let scorer = MockScorer::new("Large", 1_000_000.0, 999_999.0);
        let path = PathBuf::from("/tmp/test");
        let result = scorer.score(&path);
        assert!(result.is_ok());
        let score = result.unwrap();
        assert_eq!(score.earned, 999_999.0);
    }

    #[test]
    fn test_scorer_with_empty_name() {
        let scorer = MockScorer::new("", 10.0, 5.0);
        assert_eq!(scorer.name(), "");
    }

    #[test]
    fn test_scorer_with_unicode_name() {
        let scorer = MockScorer::new("测试スコア", 10.0, 5.0);
        assert_eq!(scorer.name(), "测试スコア");
    }

    #[test]
    fn test_scorer_with_special_path_characters() {
        let scorer = MockScorer::new("Test", 10.0, 5.0);
        let path = PathBuf::from("/tmp/path with spaces/and-dashes/under_scores");
        let result = scorer.score(&path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_scorer_perfect_score() {
        let scorer = MockScorer::new("Perfect", 25.0, 25.0);
        let path = PathBuf::from("/tmp/test");
        let result = scorer.score(&path);
        assert!(result.is_ok());
        let score = result.unwrap();
        assert!(score.is_perfect());
    }

    #[test]
    fn test_scorer_over_max_score() {
        // Edge case: earned > max (should be avoided in practice)
        let scorer = MockScorer::new("OverMax", 10.0, 15.0);
        let path = PathBuf::from("/tmp/test");
        let result = scorer.score(&path);
        assert!(result.is_ok());
        let score = result.unwrap();
        assert!(score.earned > score.max);
    }
}

// ==========================================================================
// Integration Tests with FileCache
// ==========================================================================

mod file_cache_integration_tests {
    use super::*;

    #[test]
    fn test_scorer_with_populated_cache() {
        let scorer = CacheAwareScorer::new(true);
        let path = PathBuf::from("/tmp/test");

        let mut cache = FileCache::new();
        cache.insert(
            PathBuf::from("/tmp/test/Cargo.toml"),
            "[package]\nname = \"test\"".to_string(),
        );
        cache.insert(
            PathBuf::from("/tmp/test/src/lib.rs"),
            "fn main() {}".to_string(),
        );

        let result = scorer.score_with_cache(&path, ScoringMode::Fast, Some(&cache));
        assert!(result.is_ok());
    }

    #[test]
    fn test_scorer_with_empty_cache() {
        let scorer = CacheAwareScorer::new(true);
        let path = PathBuf::from("/tmp/test");
        let cache = FileCache::new();

        let result = scorer.score_with_cache(&path, ScoringMode::Fast, Some(&cache));
        assert!(result.is_ok());
    }

    #[test]
    fn test_scorer_cache_stats() {
        let mut cache = FileCache::new();
        cache.insert(PathBuf::from("/test/file1.rs"), "content1".to_string());
        cache.insert(PathBuf::from("/test/file2.rs"), "content2".to_string());

        let (file_count, total_bytes) = cache.stats();
        assert_eq!(file_count, 2);
        assert_eq!(total_bytes, 16); // "content1" + "content2"
    }

    #[test]
    fn test_scorer_cache_get() {
        let mut cache = FileCache::new();
        let path = PathBuf::from("/test/file.rs");
        cache.insert(path.clone(), "fn test() {}".to_string());

        assert!(cache.get(&path).is_some());
        assert_eq!(cache.get(&path).unwrap(), "fn test() {}");
        assert!(cache.get(&PathBuf::from("/nonexistent")).is_none());
    }
}
