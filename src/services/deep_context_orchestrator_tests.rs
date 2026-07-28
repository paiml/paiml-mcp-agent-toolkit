// Deep context orchestrator tests - unit tests and property tests

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_feature_flags_all() {
        let flags = FeatureFlags::all();
        assert!(flags.ast_analysis);
        assert!(flags.complexity_analysis);
        assert!(flags.churn_analysis);
        assert!(flags.satd_analysis);
        assert!(flags.provability_analysis);
        assert!(flags.dead_code_analysis);
        assert!(flags.dependency_analysis);
        assert!(flags.hotspot_detection);
    }

    #[test]
    fn test_feature_flags_essential() {
        let flags = FeatureFlags::essential();
        assert!(flags.ast_analysis);
        assert!(flags.complexity_analysis);
        assert!(!flags.churn_analysis); // Disabled
        assert!(flags.satd_analysis);
        assert!(!flags.provability_analysis); // Disabled
        assert!(flags.dead_code_analysis);
        assert!(flags.dependency_analysis);
        assert!(flags.hotspot_detection);
    }

    #[test]
    fn test_cache_strategy_debug() {
        let strategies = [
            CacheStrategy::Aggressive,
            CacheStrategy::Normal,
            CacheStrategy::Minimal,
            CacheStrategy::None,
        ];
        for strategy in strategies {
            // Verify Debug trait works
            let _ = format!("{:?}", strategy);
        }
    }

    #[test]
    fn test_performance_mode_debug() {
        let modes = [
            PerformanceMode::Fast,
            PerformanceMode::Balanced,
            PerformanceMode::Memory,
        ];
        for mode in modes {
            let _ = format!("{:?}", mode);
        }
    }

    #[test]
    fn test_hotspot_type_debug() {
        let types = [
            HotspotType::HighComplexity,
            HotspotType::HighChurn,
            HotspotType::LargeFunction,
            HotspotType::DeepNesting,
            HotspotType::ManyParameters,
            HotspotType::PotentialDefect,
        ];
        for t in types {
            let _ = format!("{:?}", t);
        }
    }

    #[test]
    fn test_hotspot_severity_debug() {
        let severities = [
            HotspotSeverity::Critical,
            HotspotSeverity::High,
            HotspotSeverity::Medium,
            HotspotSeverity::Low,
        ];
        for s in severities {
            let _ = format!("{:?}", s);
        }
    }

    #[test]
    fn test_recommendation_category_debug() {
        let categories = [
            RecommendationCategory::Refactoring,
            RecommendationCategory::Performance,
            RecommendationCategory::Maintainability,
            RecommendationCategory::Testing,
            RecommendationCategory::Architecture,
        ];
        for c in categories {
            let _ = format!("{:?}", c);
        }
    }

    #[test]
    fn test_recommendation_impact_debug() {
        let impacts = [
            RecommendationImpact::High,
            RecommendationImpact::Medium,
            RecommendationImpact::Low,
        ];
        for i in impacts {
            let _ = format!("{:?}", i);
        }
    }

    #[test]
    fn test_recommendation_effort_debug() {
        let efforts = [
            RecommendationEffort::High,
            RecommendationEffort::Medium,
            RecommendationEffort::Low,
        ];
        for e in efforts {
            let _ = format!("{:?}", e);
        }
    }

    #[test]
    fn test_deep_context_config_clone() {
        let config = DeepContextConfig {
            project_path: PathBuf::from("/test"),
            include_patterns: vec!["*.rs".to_string()],
            exclude_patterns: vec!["target".to_string()],
            features: FeatureFlags::essential(),
            cache_strategy: CacheStrategy::Normal,
            performance_mode: PerformanceMode::Balanced,
        };
        let cloned = config.clone();
        assert_eq!(cloned.project_path, config.project_path);
        assert_eq!(cloned.include_patterns.len(), config.include_patterns.len());
    }

    #[test]
    fn test_hotspot_metrics_debug() {
        let metrics = HotspotMetrics {
            complexity_score: 10.5,
            defect_probability: 0.25,
            maintenance_cost: 5.0,
        };
        let debug_str = format!("{:?}", metrics);
        assert!(debug_str.contains("10.5"));
    }

    #[test]
    fn test_complexity_summary_debug() {
        let summary = ComplexitySummary {
            total_functions: 100,
            high_complexity_functions: 10,
            avg_cyclomatic: 5.5,
            avg_cognitive: 8.2,
            complexity_distribution: vec![(1, 50), (2, 30)],
        };
        let debug_str = format!("{:?}", summary);
        assert!(debug_str.contains("100"));
    }

    #[test]
    fn test_recommendation_debug() {
        let rec = Recommendation {
            category: RecommendationCategory::Refactoring,
            title: "Test title".to_string(),
            description: "Test desc".to_string(),
            impact: RecommendationImpact::High,
            effort: RecommendationEffort::Low,
        };
        let debug_str = format!("{:?}", rec);
        assert!(debug_str.contains("Refactoring"));
        assert!(debug_str.contains("Test title"));
    }

    #[test]
    fn test_code_hotspot_debug() {
        let hotspot = CodeHotspot {
            file_path: PathBuf::from("test.rs"),
            function_name: "test_fn".to_string(),
            hotspot_type: HotspotType::HighComplexity,
            severity: HotspotSeverity::High,
            metrics: HotspotMetrics {
                complexity_score: 15.0,
                defect_probability: 0.3,
                maintenance_cost: 10.0,
            },
        };
        let debug_str = format!("{:?}", hotspot);
        assert!(debug_str.contains("test.rs"));
        assert!(debug_str.contains("test_fn"));
    }

    #[test]
    fn test_deep_context_report_debug() {
        let report = DeepContextReport {
            file_count: 50,
            analysis_duration: std::time::Duration::from_secs(10),
            ast_nodes: 500,
            dependencies: 25,
            complexity_summary: ComplexitySummary {
                total_functions: 100,
                high_complexity_functions: 5,
                avg_cyclomatic: 4.0,
                avg_cognitive: 6.0,
                complexity_distribution: vec![],
            },
            hotspots: vec![],
            recommendations: vec![],
        };
        let debug_str = format!("{:?}", report);
        assert!(debug_str.contains("50"));
    }

    #[test]
    fn test_orchestrator_factory_create_minimal() {
        let result = DeepContextOrchestratorFactory::create_minimal();
        assert!(result.is_ok());
        let orchestrator = result.unwrap();
        assert!(orchestrator.max_concurrency > 0);
    }

    #[test]
    fn test_feature_flags_copy() {
        let flags = FeatureFlags::all();
        let copied = flags; // Copy trait
        assert_eq!(copied.ast_analysis, flags.ast_analysis);
    }

    #[test]
    fn test_cache_strategy_copy() {
        let strategy = CacheStrategy::Aggressive;
        let copied = strategy; // Copy trait
        assert!(matches!(copied, CacheStrategy::Aggressive));
    }

    #[test]
    fn test_performance_mode_copy() {
        let mode = PerformanceMode::Fast;
        let copied = mode; // Copy trait
        assert!(matches!(copied, PerformanceMode::Fast));
    }
}

