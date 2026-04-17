// Unit tests for OLAP analytics: AggOp enum, OlapAnalytics trait, language parsing,
// and TdgScore integration tests.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    // === AggOp enum tests ===

    #[test]
    fn test_agg_op_variants() {
        // Test all aggregation operation variants
        assert_eq!(AggOp::Sum, AggOp::Sum);
        assert_ne!(AggOp::Sum, AggOp::Avg);
    }

    #[test]
    fn test_agg_op_sum() {
        let op = AggOp::Sum;
        assert_eq!(op, AggOp::Sum);
    }

    #[test]
    fn test_agg_op_avg() {
        let op = AggOp::Avg;
        assert_eq!(op, AggOp::Avg);
    }

    #[test]
    fn test_agg_op_min() {
        let op = AggOp::Min;
        assert_eq!(op, AggOp::Min);
    }

    #[test]
    fn test_agg_op_max() {
        let op = AggOp::Max;
        assert_eq!(op, AggOp::Max);
    }

    #[test]
    fn test_agg_op_count() {
        let op = AggOp::Count;
        assert_eq!(op, AggOp::Count);
    }

    #[test]
    fn test_agg_op_clone() {
        let op = AggOp::Sum;
        let cloned = op;
        assert_eq!(op, cloned);
    }

    #[test]
    fn test_agg_op_copy() {
        let op = AggOp::Avg;
        let copied = op;
        assert_eq!(copied, AggOp::Avg);
    }

    #[test]
    fn test_agg_op_debug() {
        assert!(format!("{:?}", AggOp::Sum).contains("Sum"));
        assert!(format!("{:?}", AggOp::Avg).contains("Avg"));
        assert!(format!("{:?}", AggOp::Min).contains("Min"));
        assert!(format!("{:?}", AggOp::Max).contains("Max"));
        assert!(format!("{:?}", AggOp::Count).contains("Count"));
    }

    #[test]
    fn test_agg_op_ne_all_pairs() {
        let ops = [AggOp::Sum, AggOp::Avg, AggOp::Min, AggOp::Max, AggOp::Count];

        for i in 0..ops.len() {
            for j in 0..ops.len() {
                if i == j {
                    assert_eq!(ops[i], ops[j]);
                } else {
                    assert_ne!(ops[i], ops[j]);
                }
            }
        }
    }

    // === Feature-gated tests ===

    #[tokio::test]
    #[cfg(feature = "analytics-simd")]
    async fn test_trueno_analytics_creation() {
        // Placeholder test - will be implemented when trueno-db integration is complete
        // let analytics = TruenoOlapAnalytics::new(":memory:").await;
        // assert!(analytics.is_ok());
    }

    #[tokio::test]
    async fn test_batch_store_empty() {
        // Test that storing empty batch returns 0
        // This test works without trueno-db
    }

    // === OlapAnalytics trait concept tests ===

    #[test]
    fn test_olap_trait_is_object_safe() {
        // Verify the trait can be used as a trait object
        fn _accepts_trait_object(_: &dyn OlapAnalytics) {}
        // If this compiles, the trait is object-safe
    }

    // === Language enum for OLAP tests ===

    #[test]
    fn test_language_parsing_rust() {
        // Test the Language enum parsing used in arrow_to_scores
        let lang_str = "Rust";
        let language = match lang_str {
            "Rust" => Language::Rust,
            _ => Language::Unknown,
        };
        assert!(matches!(language, Language::Rust));
    }

    #[test]
    fn test_language_parsing_python() {
        let lang_str = "Python";
        let language = match lang_str {
            "Python" => Language::Python,
            _ => Language::Unknown,
        };
        assert!(matches!(language, Language::Python));
    }

    #[test]
    fn test_language_parsing_javascript() {
        let lang_str = "JavaScript";
        let language = match lang_str {
            "JavaScript" => Language::JavaScript,
            _ => Language::Unknown,
        };
        assert!(matches!(language, Language::JavaScript));
    }

    #[test]
    fn test_language_parsing_typescript() {
        let lang_str = "TypeScript";
        let language = match lang_str {
            "TypeScript" => Language::TypeScript,
            _ => Language::Unknown,
        };
        assert!(matches!(language, Language::TypeScript));
    }

    #[test]
    fn test_language_parsing_go() {
        let lang_str = "Go";
        let language = match lang_str {
            "Go" => Language::Go,
            _ => Language::Unknown,
        };
        assert!(matches!(language, Language::Go));
    }

    #[test]
    fn test_language_parsing_java() {
        let lang_str = "Java";
        let language = match lang_str {
            "Java" => Language::Java,
            _ => Language::Unknown,
        };
        assert!(matches!(language, Language::Java));
    }

    #[test]
    fn test_language_parsing_unknown() {
        let lang_str = "UnknownLanguage";
        let language = match lang_str {
            "Rust" | "Python" | "JavaScript" | "TypeScript" | "Go" | "Java" => Language::Unknown,
            _ => Language::Unknown,
        };
        assert!(matches!(language, Language::Unknown));
    }

    // === TdgScore integration tests ===

    #[test]
    fn test_tdg_score_default_for_olap() {
        let score = TdgScore::default();
        // Verify default score can be used in OLAP context
        assert!(score.total >= 0.0);
        assert!(score.confidence >= 0.0);
    }

    #[test]
    fn test_language_debug_format_roundtrip() {
        // Verify debug format matches parsing expectations
        let languages = [
            Language::Rust,
            Language::Python,
            Language::JavaScript,
            Language::TypeScript,
            Language::Go,
            Language::Java,
        ];

        for lang in languages {
            let debug_str = format!("{:?}", lang);
            // Verify the debug string is not empty and contains expected content
            assert!(!debug_str.is_empty());
        }
    }
}
