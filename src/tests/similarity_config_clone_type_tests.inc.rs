mod similarity_config_tests {
    use super::*;

    #[test]
    fn test_config_default_values() {
        let config = SimilarityConfig::default();
        assert_eq!(config.min_lines, 6);
        assert_eq!(config.min_tokens, 50);
        assert!((config.similarity_threshold - 0.7).abs() < f64::EPSILON);
        assert!(config.enable_entropy);
        assert!(config.enable_ast);
        assert!(config.enable_semantic);
        assert_eq!(config.window_size, 40);
        assert_eq!(config.k_gram_size, 15);
    }

    #[test]
    fn test_config_custom_min_lines() {
        let config = SimilarityConfig {
            min_lines: 3,
            ..SimilarityConfig::default()
        };
        assert_eq!(config.min_lines, 3);
    }

    #[test]
    fn test_config_custom_min_tokens() {
        let config = SimilarityConfig {
            min_tokens: 100,
            ..SimilarityConfig::default()
        };
        assert_eq!(config.min_tokens, 100);
    }

    #[test]
    fn test_config_custom_threshold() {
        let config = SimilarityConfig {
            similarity_threshold: 0.95,
            ..SimilarityConfig::default()
        };
        assert!((config.similarity_threshold - 0.95).abs() < f64::EPSILON);
    }

    #[test]
    fn test_config_disable_entropy() {
        let config = SimilarityConfig {
            enable_entropy: false,
            ..SimilarityConfig::default()
        };
        assert!(!config.enable_entropy);
    }

    #[test]
    fn test_config_disable_ast() {
        let config = SimilarityConfig {
            enable_ast: false,
            ..SimilarityConfig::default()
        };
        assert!(!config.enable_ast);
    }

    #[test]
    fn test_config_disable_semantic() {
        let config = SimilarityConfig {
            enable_semantic: false,
            ..SimilarityConfig::default()
        };
        assert!(!config.enable_semantic);
    }

    #[test]
    fn test_config_custom_window_size() {
        let config = SimilarityConfig {
            window_size: 20,
            ..SimilarityConfig::default()
        };
        assert_eq!(config.window_size, 20);
    }

    #[test]
    fn test_config_custom_k_gram_size() {
        let config = SimilarityConfig {
            k_gram_size: 10,
            ..SimilarityConfig::default()
        };
        assert_eq!(config.k_gram_size, 10);
    }

    #[test]
    fn test_config_clone() {
        let config = SimilarityConfig::default();
        let cloned = config.clone();
        assert_eq!(config.min_lines, cloned.min_lines);
        assert_eq!(config.min_tokens, cloned.min_tokens);
        assert_eq!(config.window_size, cloned.window_size);
    }

    #[test]
    fn test_config_serialization_roundtrip() {
        let config = SimilarityConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: SimilarityConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.min_lines, deserialized.min_lines);
        assert_eq!(config.min_tokens, deserialized.min_tokens);
        assert!(
            (config.similarity_threshold - deserialized.similarity_threshold).abs() < f64::EPSILON
        );
    }

    #[test]
    fn test_config_all_disabled() {
        let config = SimilarityConfig {
            enable_entropy: false,
            enable_ast: false,
            enable_semantic: false,
            ..SimilarityConfig::default()
        };
        assert!(!config.enable_entropy);
        assert!(!config.enable_ast);
        assert!(!config.enable_semantic);
    }

    #[test]
    fn test_config_extreme_values() {
        let config = SimilarityConfig {
            min_lines: 1,
            min_tokens: 1,
            similarity_threshold: 0.0,
            window_size: 1,
            k_gram_size: 1,
            ..SimilarityConfig::default()
        };
        assert_eq!(config.min_lines, 1);
        assert_eq!(config.min_tokens, 1);
        assert!((config.similarity_threshold - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_config_large_values() {
        let config = SimilarityConfig {
            min_lines: 1000,
            min_tokens: 10000,
            similarity_threshold: 1.0,
            window_size: 1000,
            k_gram_size: 500,
            ..SimilarityConfig::default()
        };
        assert_eq!(config.min_lines, 1000);
        assert_eq!(config.min_tokens, 10000);
    }
}

// =============================================================================
// CloneType Tests
// =============================================================================

mod clone_type_tests {
    use super::*;

    #[test]
    fn test_clone_type_type1() {
        let ct = CloneType::Type1;
        assert_eq!(ct, CloneType::Type1);
    }

    #[test]
    fn test_clone_type_type2() {
        let ct = CloneType::Type2;
        assert_eq!(ct, CloneType::Type2);
    }

    #[test]
    fn test_clone_type_type3() {
        let ct = CloneType::Type3;
        assert_eq!(ct, CloneType::Type3);
    }

    #[test]
    fn test_clone_type_type4() {
        let ct = CloneType::Type4;
        assert_eq!(ct, CloneType::Type4);
    }

    #[test]
    fn test_clone_type_inequality() {
        assert_ne!(CloneType::Type1, CloneType::Type2);
        assert_ne!(CloneType::Type2, CloneType::Type3);
        assert_ne!(CloneType::Type3, CloneType::Type4);
    }

    #[test]
    fn test_clone_type_copy_trait() {
        let ct = CloneType::Type1;
        let copied = ct;
        assert_eq!(ct, copied);
    }

    #[test]
    fn test_clone_type_debug() {
        let ct = CloneType::Type1;
        let debug_str = format!("{:?}", ct);
        assert!(debug_str.contains("Type1"));
    }

    #[test]
    fn test_clone_type_serialization() {
        for ct in [
            CloneType::Type1,
            CloneType::Type2,
            CloneType::Type3,
            CloneType::Type4,
        ] {
            let json = serde_json::to_string(&ct).unwrap();
            let deserialized: CloneType = serde_json::from_str(&json).unwrap();
            assert_eq!(ct, deserialized);
        }
    }
}

// =============================================================================
// SimilarityDetector Tests
// =============================================================================

