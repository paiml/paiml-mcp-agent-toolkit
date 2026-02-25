mod entropy_report_tests {
    use super::*;

    #[test]
    fn test_entropy_report_empty() {
        let report = EntropyReport {
            average_entropy: 0.0,
            high_entropy_blocks: vec![],
            low_entropy_patterns: vec![],
            recommendations: vec![],
        };
        assert!((report.average_entropy - 0.0).abs() < f64::EPSILON);
        assert!(report.high_entropy_blocks.is_empty());
        assert!(report.low_entropy_patterns.is_empty());
        assert!(report.recommendations.is_empty());
    }

    #[test]
    fn test_entropy_report_with_blocks() {
        let report = EntropyReport {
            average_entropy: 3.5,
            high_entropy_blocks: vec![EntropyBlock {
                location: Location {
                    file: PathBuf::from("test.rs"),
                    start_line: 1,
                    end_line: 10,
                    start_column: None,
                    end_column: None,
                },
                entropy: 4.5,
                category: "Complex".to_string(),
                suggestion: "Simplify".to_string(),
            }],
            low_entropy_patterns: vec![EntropyBlock {
                location: Location {
                    file: PathBuf::from("test.rs"),
                    start_line: 20,
                    end_line: 30,
                    start_column: None,
                    end_column: None,
                },
                entropy: 1.5,
                category: "Repetitive".to_string(),
                suggestion: "Extract".to_string(),
            }],
            recommendations: vec!["Recommendation 1".to_string()],
        };
        assert!((report.average_entropy - 3.5).abs() < f64::EPSILON);
        assert_eq!(report.high_entropy_blocks.len(), 1);
        assert_eq!(report.low_entropy_patterns.len(), 1);
    }

    #[test]
    fn test_entropy_report_serialization() {
        let report = EntropyReport {
            average_entropy: 3.0,
            high_entropy_blocks: vec![],
            low_entropy_patterns: vec![],
            recommendations: vec!["Test".to_string()],
        };
        let json = serde_json::to_string(&report).unwrap();
        let deserialized: EntropyReport = serde_json::from_str(&json).unwrap();
        assert!((report.average_entropy - deserialized.average_entropy).abs() < f64::EPSILON);
    }
}

// =============================================================================
// EntropyBlock Tests
// =============================================================================

mod entropy_block_tests {
    use super::*;

    #[test]
    fn test_entropy_block_high_entropy() {
        let block = EntropyBlock {
            location: Location {
                file: PathBuf::from("complex.rs"),
                start_line: 1,
                end_line: 100,
                start_column: None,
                end_column: None,
            },
            entropy: 5.0,
            category: "Complex".to_string(),
            suggestion: "This code is very complex, consider breaking it down".to_string(),
        };
        assert!(block.entropy > 4.0);
        assert_eq!(block.category, "Complex");
    }

    #[test]
    fn test_entropy_block_low_entropy() {
        let block = EntropyBlock {
            location: Location {
                file: PathBuf::from("simple.rs"),
                start_line: 1,
                end_line: 10,
                start_column: None,
                end_column: None,
            },
            entropy: 1.5,
            category: "Repetitive".to_string(),
            suggestion: "Extract repeated pattern".to_string(),
        };
        assert!(block.entropy < 2.0);
        assert_eq!(block.category, "Repetitive");
    }

    #[test]
    fn test_entropy_block_serialization() {
        let block = EntropyBlock {
            location: Location {
                file: PathBuf::from("test.rs"),
                start_line: 1,
                end_line: 10,
                start_column: None,
                end_column: None,
            },
            entropy: 3.0,
            category: "Normal".to_string(),
            suggestion: "No action needed".to_string(),
        };
        let json = serde_json::to_string(&block).unwrap();
        let deserialized: EntropyBlock = serde_json::from_str(&json).unwrap();
        assert!((block.entropy - deserialized.entropy).abs() < f64::EPSILON);
    }
}

// =============================================================================
// Priority Tests
// =============================================================================

mod priority_tests {
    use super::*;

    #[test]
    fn test_priority_high() {
        let p = Priority::High;
        let debug_str = format!("{:?}", p);
        assert!(debug_str.contains("High"));
    }

    #[test]
    fn test_priority_medium() {
        let p = Priority::Medium;
        let debug_str = format!("{:?}", p);
        assert!(debug_str.contains("Medium"));
    }

    #[test]
    fn test_priority_low() {
        let p = Priority::Low;
        let debug_str = format!("{:?}", p);
        assert!(debug_str.contains("Low"));
    }

    #[test]
    fn test_priority_clone() {
        let p = Priority::High;
        let cloned = p.clone();
        let p_str = format!("{:?}", p);
        let cloned_str = format!("{:?}", cloned);
        assert_eq!(p_str, cloned_str);
    }

    #[test]
    fn test_priority_serialization() {
        for priority in [Priority::High, Priority::Medium, Priority::Low] {
            let json = serde_json::to_string(&priority).unwrap();
            let deserialized: Priority = serde_json::from_str(&json).unwrap();
            let orig_str = format!("{:?}", priority);
            let deser_str = format!("{:?}", deserialized);
            assert_eq!(orig_str, deser_str);
        }
    }
}

// =============================================================================
// RefactoringHint Tests
// =============================================================================

mod refactoring_hint_tests {
    use super::*;

    #[test]
    fn test_refactoring_hint_full() {
        let hint = RefactoringHint {
            locations: vec![
                Location {
                    file: PathBuf::from("file1.rs"),
                    start_line: 1,
                    end_line: 10,
                    start_column: None,
                    end_column: None,
                },
                Location {
                    file: PathBuf::from("file2.rs"),
                    start_line: 20,
                    end_line: 30,
                    start_column: None,
                    end_column: None,
                },
            ],
            pattern: "Repeated code structure".to_string(),
            suggestion: "Extract common pattern into shared function".to_string(),
            priority: Priority::High,
        };
        assert_eq!(hint.locations.len(), 2);
        assert_eq!(hint.pattern, "Repeated code structure");
        assert!(matches!(hint.priority, Priority::High));
    }

    #[test]
    fn test_refactoring_hint_empty_locations() {
        let hint = RefactoringHint {
            locations: vec![],
            pattern: "Test pattern".to_string(),
            suggestion: "Test suggestion".to_string(),
            priority: Priority::Low,
        };
        assert!(hint.locations.is_empty());
    }

    #[test]
    fn test_refactoring_hint_serialization() {
        let hint = RefactoringHint {
            locations: vec![],
            pattern: "Pattern".to_string(),
            suggestion: "Suggestion".to_string(),
            priority: Priority::Medium,
        };
        let json = serde_json::to_string(&hint).unwrap();
        let deserialized: RefactoringHint = serde_json::from_str(&json).unwrap();
        assert_eq!(hint.pattern, deserialized.pattern);
        assert_eq!(hint.suggestion, deserialized.suggestion);
    }
}

// =============================================================================
// Metrics Tests
// =============================================================================

mod metrics_tests {
    use super::*;

    #[test]
    fn test_metrics_zero() {
        let metrics = Metrics {
            duplication_percentage: 0.0,
            average_entropy: 0.0,
            total_clones: 0,
        };
        assert!((metrics.duplication_percentage - 0.0).abs() < f64::EPSILON);
        assert!((metrics.average_entropy - 0.0).abs() < f64::EPSILON);
        assert_eq!(metrics.total_clones, 0);
    }

    #[test]
    fn test_metrics_typical_values() {
        let metrics = Metrics {
            duplication_percentage: 15.5,
            average_entropy: 3.2,
            total_clones: 5,
        };
        assert!((metrics.duplication_percentage - 15.5).abs() < f64::EPSILON);
        assert!((metrics.average_entropy - 3.2).abs() < f64::EPSILON);
        assert_eq!(metrics.total_clones, 5);
    }

    #[test]
    fn test_metrics_high_values() {
        let metrics = Metrics {
            duplication_percentage: 100.0,
            average_entropy: 8.0,
            total_clones: 1000,
        };
        assert!((metrics.duplication_percentage - 100.0).abs() < f64::EPSILON);
        assert!((metrics.average_entropy - 8.0).abs() < f64::EPSILON);
        assert_eq!(metrics.total_clones, 1000);
    }

    #[test]
    fn test_metrics_serialization() {
        let metrics = Metrics {
            duplication_percentage: 25.0,
            average_entropy: 3.5,
            total_clones: 10,
        };
        let json = serde_json::to_string(&metrics).unwrap();
        let deserialized: Metrics = serde_json::from_str(&json).unwrap();
        assert!(
            (metrics.duplication_percentage - deserialized.duplication_percentage).abs()
                < f64::EPSILON
        );
        assert_eq!(metrics.total_clones, deserialized.total_clones);
    }
}

// =============================================================================
// ComprehensiveReport Tests
// =============================================================================

mod comprehensive_report_tests {
    use super::*;

    #[test]
    fn test_comprehensive_report_empty() {
        let report = ComprehensiveReport {
            exact_duplicates: vec![],
            structural_similarities: vec![],
            semantic_similarities: vec![],
            entropy_analysis: None,
            refactoring_opportunities: vec![],
            metrics: Metrics {
                duplication_percentage: 0.0,
                average_entropy: 0.0,
                total_clones: 0,
            },
        };
        assert!(report.exact_duplicates.is_empty());
        assert!(report.structural_similarities.is_empty());
        assert!(report.semantic_similarities.is_empty());
        assert!(report.entropy_analysis.is_none());
        assert!(report.refactoring_opportunities.is_empty());
    }

    #[test]
    fn test_comprehensive_report_with_data() {
        let report = ComprehensiveReport {
            exact_duplicates: vec![SimilarBlock {
                id: "dup1".to_string(),
                locations: vec![],
                similarity: 1.0,
                clone_type: CloneType::Type1,
                lines: 10,
                tokens: 50,
                content_preview: "test".to_string(),
            }],
            structural_similarities: vec![],
            semantic_similarities: vec![],
            entropy_analysis: Some(EntropyReport {
                average_entropy: 3.0,
                high_entropy_blocks: vec![],
                low_entropy_patterns: vec![],
                recommendations: vec![],
            }),
            refactoring_opportunities: vec![],
            metrics: Metrics {
                duplication_percentage: 10.0,
                average_entropy: 3.0,
                total_clones: 1,
            },
        };
        assert_eq!(report.exact_duplicates.len(), 1);
        assert!(report.entropy_analysis.is_some());
        assert_eq!(report.metrics.total_clones, 1);
    }

    #[test]
    fn test_comprehensive_report_serialization() {
        let report = ComprehensiveReport {
            exact_duplicates: vec![],
            structural_similarities: vec![],
            semantic_similarities: vec![],
            entropy_analysis: None,
            refactoring_opportunities: vec![],
            metrics: Metrics {
                duplication_percentage: 5.0,
                average_entropy: 2.5,
                total_clones: 2,
            },
        };
        let json = serde_json::to_string(&report).unwrap();
        let deserialized: ComprehensiveReport = serde_json::from_str(&json).unwrap();
        assert_eq!(
            report.metrics.total_clones,
            deserialized.metrics.total_clones
        );
    }
}

// =============================================================================
// Hash Collision Edge Cases
// =============================================================================

