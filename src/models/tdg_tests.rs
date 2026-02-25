#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    prop_compose! {
        fn valid_tdg_score()(
            value in 0.0..10.0,
            complexity in 0.0..5.0,
            churn in 0.0..5.0,
            coupling in 0.0..5.0,
            domain_risk in 0.0..5.0,
            duplication in 0.0..5.0,
            dead_code in 0.0..5.0,
            percentile in 0.0..100.0,
            confidence in 0.0..1.0
        ) -> TDGScore {
            TDGScore {
                value,
                components: TDGComponents {
                    complexity,
                    churn,
                    coupling,
                    domain_risk,
                    duplication,
                    dead_code,
                },
                severity: if value > 2.5 { TDGSeverity::Critical }
                         else if value > 1.5 { TDGSeverity::Warning }
                         else { TDGSeverity::Normal },
                percentile,
                confidence,
            }
        }
    }

    proptest! {
        #[test]
        fn tdg_score_roundtrip_serialization(score in valid_tdg_score()) {
            let json = serde_json::to_string(&score)?;
            let deserialized: TDGScore = serde_json::from_str(&json)?;
            // Use approximate equality for floating point values
            const EPSILON: f64 = 1e-10;
            prop_assert!((score.value - deserialized.value).abs() < EPSILON);
            prop_assert!((score.components.complexity - deserialized.components.complexity).abs() < EPSILON);
            prop_assert!((score.components.churn - deserialized.components.churn).abs() < EPSILON);
            prop_assert!((score.components.coupling - deserialized.components.coupling).abs() < EPSILON);
            prop_assert!((score.components.domain_risk - deserialized.components.domain_risk).abs() < EPSILON);
            prop_assert!((score.components.duplication - deserialized.components.duplication).abs() < EPSILON);
            prop_assert!((score.components.dead_code - deserialized.components.dead_code).abs() < EPSILON);
            prop_assert_eq!(score.severity, deserialized.severity);
            prop_assert!((score.percentile - deserialized.percentile).abs() < EPSILON);
            prop_assert!((score.confidence - deserialized.confidence).abs() < EPSILON);
        }

        #[test]
        fn tdg_severity_matches_value(score in valid_tdg_score()) {
            match score.severity {
                TDGSeverity::Normal => prop_assert!(score.value <= 1.5),
                TDGSeverity::Warning => prop_assert!(score.value > 1.5 && score.value <= 2.5),
                TDGSeverity::Critical => prop_assert!(score.value > 2.5),
            }
        }

        #[test]
        fn tdg_percentile_in_valid_range(score in valid_tdg_score()) {
            prop_assert!(score.percentile >= 0.0 && score.percentile <= 100.0);
        }

        #[test]
        fn tdg_confidence_in_valid_range(score in valid_tdg_score()) {
            prop_assert!(score.confidence >= 0.0 && score.confidence <= 1.0);
        }

        #[test]
        fn tdg_components_non_negative(score in valid_tdg_score()) {
            prop_assert!(score.components.complexity >= 0.0);
            prop_assert!(score.components.churn >= 0.0);
            prop_assert!(score.components.coupling >= 0.0);
            prop_assert!(score.components.domain_risk >= 0.0);
            prop_assert!(score.components.duplication >= 0.0);
            prop_assert!(score.components.dead_code >= 0.0);
        }


        #[test]
        fn tdg_severity_ordering_consistent(value in 0.0..10.0) {
            let severity = if value > 2.5 { TDGSeverity::Critical }
                          else if value > 1.5 { TDGSeverity::Warning }
                          else { TDGSeverity::Normal };

            match severity {
                TDGSeverity::Normal => prop_assert!(value <= 1.5),
                TDGSeverity::Warning => prop_assert!(value > 1.5 && value <= 2.5),
                TDGSeverity::Critical => prop_assert!(value > 2.5),
            }
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tdg_severity_from_value() {
        assert_eq!(TDGSeverity::from(0.5), TDGSeverity::Normal);
        assert_eq!(TDGSeverity::from(1.5), TDGSeverity::Normal);
        assert_eq!(TDGSeverity::from(1.6), TDGSeverity::Warning);
        assert_eq!(TDGSeverity::from(2.5), TDGSeverity::Warning);
        assert_eq!(TDGSeverity::from(2.6), TDGSeverity::Critical);
        assert_eq!(TDGSeverity::from(5.0), TDGSeverity::Critical);
    }

    #[test]
    fn test_tdg_config_default() {
        let config = TDGConfig::default();
        let total_weight = config.complexity_weight
            + config.churn_weight
            + config.coupling_weight
            + config.domain_risk_weight
            + config.duplication_weight
            + config.dead_code_weight; // CB-128: 6th dimension

        // Weights should sum to 1.0
        assert!((total_weight - 1.0).abs() < f64::EPSILON);
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod new_tests {
    use super::*;

    #[test]
    fn test_tdg_score_creation() {
        let components = TDGComponents {
            complexity: 1.5,
            churn: 0.8,
            coupling: 0.3,
            domain_risk: 0.2,
            duplication: 0.4,
            dead_code: 0.1, // CB-128: 6th dimension
        };

        let score = TDGScore {
            value: 3.2,
            components,
            severity: TDGSeverity::Warning,
            percentile: 75.0,
            confidence: 0.9,
        };

        assert_eq!(score.value, 3.2);
        assert_eq!(score.components.complexity, 1.5);
        assert_eq!(score.components.dead_code, 0.1);
        assert_eq!(score.severity, TDGSeverity::Warning);
        assert_eq!(score.percentile, 75.0);
        assert_eq!(score.confidence, 0.9);
    }

    #[test]
    fn test_tdg_severity_ordering() {
        // TDGSeverity doesn't implement Ord, just test equality
        assert_eq!(TDGSeverity::Normal, TDGSeverity::Normal);
        assert_eq!(TDGSeverity::Warning, TDGSeverity::Warning);
        assert_eq!(TDGSeverity::Critical, TDGSeverity::Critical);

        assert_eq!(TDGSeverity::Normal, TDGSeverity::Normal);
        assert_ne!(TDGSeverity::Normal, TDGSeverity::Critical);
    }

    #[test]
    fn test_tdg_components_equality() {
        let comp1 = TDGComponents {
            complexity: 1.0,
            churn: 2.0,
            coupling: 3.0,
            domain_risk: 4.0,
            duplication: 5.0,
            dead_code: 0.5,
        };

        let comp2 = TDGComponents {
            complexity: 1.0,
            churn: 2.0,
            coupling: 3.0,
            domain_risk: 4.0,
            duplication: 5.0,
            dead_code: 0.5,
        };

        assert_eq!(comp1, comp2);
    }

    #[test]
    fn test_tdg_summary_creation() {
        let summary = TDGSummary {
            total_files: 95,
            critical_files: 10,
            warning_files: 20,
            average_tdg: 2.5,
            p95_tdg: 4.5,
            p99_tdg: 4.9,
            estimated_debt_hours: 120.0,
            hotspots: vec![],
        };

        assert_eq!(summary.total_files, 95);
        assert_eq!(summary.critical_files, 10);
        assert_eq!(summary.warning_files, 20);
        assert_eq!(summary.average_tdg, 2.5);
        assert_eq!(summary.p95_tdg, 4.5);
        assert_eq!(summary.p99_tdg, 4.9);
        assert_eq!(summary.estimated_debt_hours, 120.0);
    }

    #[test]
    fn test_tdg_hotspot() {
        let hotspot = TDGHotspot {
            path: "src/complex.rs".to_string(),
            tdg_score: 4.5,
            primary_factor: "High complexity and churn".to_string(),
            estimated_hours: 8.0,
        };

        assert_eq!(hotspot.path, "src/complex.rs");
        assert_eq!(hotspot.tdg_score, 4.5);
        assert_eq!(hotspot.primary_factor, "High complexity and churn");
        assert_eq!(hotspot.estimated_hours, 8.0);
    }

    #[test]
    fn test_tdg_analysis() {
        let analysis = TDGAnalysis {
            score: TDGScore {
                value: 2.5,
                components: TDGComponents {
                    complexity: 0.8,
                    churn: 0.5,
                    coupling: 0.4,
                    domain_risk: 0.3,
                    duplication: 0.5,
                    dead_code: 0.0,
                },
                severity: TDGSeverity::Warning,
                percentile: 75.0,
                confidence: 0.95,
            },
            explanation: "Test explanation".to_string(),
            recommendations: vec![],
        };

        assert_eq!(analysis.score.value, 2.5);
        assert_eq!(analysis.score.percentile, 75.0);
        assert!(analysis.recommendations.is_empty());
    }

    #[test]
    fn test_recommendation_type() {
        assert_eq!(
            RecommendationType::ReduceComplexity,
            RecommendationType::ReduceComplexity
        );
        assert_ne!(
            RecommendationType::ReduceComplexity,
            RecommendationType::StabilizeChurn
        );

        let rec = TDGRecommendation {
            recommendation_type: RecommendationType::ReduceComplexity,
            action: "Refactor complex function into smaller units".to_string(),
            expected_reduction: 0.5,
            estimated_hours: 4.0,
            priority: 3,
        };

        assert_eq!(rec.priority, 3);
        assert_eq!(rec.action, "Refactor complex function into smaller units");
        assert_eq!(rec.expected_reduction, 0.5);
    }

    #[test]
    fn test_tdg_distribution() {
        let dist = TDGDistribution {
            buckets: vec![
                TDGBucket {
                    min: 0.0,
                    max: 1.5,
                    count: 50,
                    percentage: 50.0,
                },
                TDGBucket {
                    min: 1.5,
                    max: 3.0,
                    count: 30,
                    percentage: 30.0,
                },
                TDGBucket {
                    min: 3.0,
                    max: 5.0,
                    count: 20,
                    percentage: 20.0,
                },
            ],
            total_files: 100,
        };

        assert_eq!(dist.buckets.len(), 3);
        assert_eq!(dist.buckets[0].count, 50);
        assert_eq!(dist.buckets[0].percentage, 50.0);
        assert_eq!(dist.total_files, 100);
    }

    #[test]
    fn test_tdg_config() {
        let config = TDGConfig {
            complexity_weight: 0.25,
            churn_weight: 0.20,
            coupling_weight: 0.15,
            domain_risk_weight: 0.10,
            duplication_weight: 0.10,
            dead_code_weight: 0.20, // CB-128: 6th dimension
            critical_threshold: 2.5,
            warning_threshold: 1.5,
        };

        assert_eq!(config.complexity_weight, 0.25);
        assert_eq!(config.churn_weight, 0.20);
        assert_eq!(config.dead_code_weight, 0.20);
        assert_eq!(config.critical_threshold, 2.5);
        assert_eq!(config.warning_threshold, 1.5);
    }

    #[test]
    fn test_satd_item_creation() {
        let item = SatdItem {
            file_path: PathBuf::from("lib.rs"),
            line_number: 123,
            comment_text: "TODO: Fix this hack".to_string(),
            debt_type: "TODO".to_string(),
            severity: SatdSeverity::Medium,
            confidence: 0.95,
        };

        assert_eq!(item.file_path, PathBuf::from("lib.rs"));
        assert_eq!(item.line_number, 123);
        assert_eq!(item.comment_text, "TODO: Fix this hack");
        assert_eq!(item.debt_type, "TODO");
        assert_eq!(item.severity, SatdSeverity::Medium);
        assert_eq!(item.confidence, 0.95);
    }

    #[test]
    fn test_satd_severity_ordering() {
        assert!(SatdSeverity::Low < SatdSeverity::Medium);
        assert!(SatdSeverity::Medium < SatdSeverity::High);
        assert!(SatdSeverity::High < SatdSeverity::Critical);

        let mut severities = vec![
            SatdSeverity::High,
            SatdSeverity::Low,
            SatdSeverity::Critical,
            SatdSeverity::Medium,
        ];

        severities.sort();

        assert_eq!(
            severities,
            vec![
                SatdSeverity::Low,
                SatdSeverity::Medium,
                SatdSeverity::High,
                SatdSeverity::Critical,
            ]
        );
    }

    #[test]
    fn test_serialization_roundtrip() {
        let original = TDGScore {
            value: std::f64::consts::PI,
            components: TDGComponents {
                complexity: 1.1,
                churn: 2.2,
                coupling: 3.3,
                domain_risk: 4.4,
                duplication: 5.5,
                dead_code: 0.6,
            },
            severity: TDGSeverity::Critical,
            percentile: 90.0,
            confidence: 0.99,
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: TDGScore = serde_json::from_str(&json).unwrap();

        assert_eq!(original, deserialized);
        assert_eq!(original.value, deserialized.value);
        assert_eq!(original.components, deserialized.components);
        assert_eq!(original.severity, deserialized.severity);
    }
}
