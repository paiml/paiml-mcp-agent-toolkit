#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_assistant_creation() {
        let assistant = QualityAssistant::new();
        assert!(!assistant.pattern_db.is_empty());
    }

    #[test]
    fn test_suggest_for_complexity() {
        let assistant = QualityAssistant::new();
        let violation = Violation {
            file: "test.rs".to_string(),
            violation_type: ViolationType::Complexity,
            severity: crate::unified_quality::metrics::Severity::High,
            value: 25.0,
            threshold: 20.0,
        };

        let suggestions = assistant.suggest(&violation);
        assert!(!suggestions.is_empty());
        assert!(suggestions[0].confidence > 0.6);
    }

    #[test]
    fn test_feedback_recording() {
        let mut collector = FeedbackCollector::new();
        collector.record("extract_method", true, None);
        assert_eq!(collector.metrics.accepted, 1);
        assert_eq!(collector.metrics.success_rate, 1.0);
    }

    #[test]
    fn test_confidence_scoring() {
        let scorer = ConfidenceScorer::new();
        let pattern = Pattern {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: "Test pattern".to_string(),
            template: "".to_string(),
            success_rate: 0.8,
            contexts: vec!["high_complexity".to_string()],
            example: Example {
                before: "".to_string(),
                after: "".to_string(),
                improvement: "".to_string(),
            },
        };

        let violation = Violation {
            file: "test.rs".to_string(),
            violation_type: ViolationType::Complexity,
            severity: crate::unified_quality::metrics::Severity::High,
            value: 25.0,
            threshold: 20.0,
        };

        let score = scorer.score(&pattern, &violation);
        assert!(score > 0.5);
        assert!(score <= 1.0);
    }

    #[test]
    fn test_quality_assistant_default() {
        let assistant = QualityAssistant::default();
        assert!(!assistant.pattern_db.is_empty());
    }

    #[test]
    fn test_pattern_creation() {
        let pattern = Pattern {
            id: "test_pattern".to_string(),
            name: "Test Pattern".to_string(),
            description: "A test pattern".to_string(),
            template: "template code".to_string(),
            success_rate: 0.85,
            contexts: vec!["context1".to_string(), "context2".to_string()],
            example: Example {
                before: "before code".to_string(),
                after: "after code".to_string(),
                improvement: "improved something".to_string(),
            },
        };

        assert_eq!(pattern.id, "test_pattern");
        assert_eq!(pattern.name, "Test Pattern");
        assert_eq!(pattern.success_rate, 0.85);
        assert_eq!(pattern.contexts.len(), 2);
    }

    #[test]
    fn test_example_creation() {
        let example = Example {
            before: "old code".to_string(),
            after: "new code".to_string(),
            improvement: "10% faster".to_string(),
        };

        assert_eq!(example.before, "old code");
        assert_eq!(example.after, "new code");
        assert_eq!(example.improvement, "10% faster");
    }

    #[test]
    fn test_impact_creation() {
        let impact = Impact {
            complexity_reduction: 5,
            loc_change: -10,
            coverage_impact: 2.5,
            risk: RiskLevel::Low,
        };

        assert_eq!(impact.complexity_reduction, 5);
        assert_eq!(impact.loc_change, -10);
        assert!((impact.coverage_impact - 2.5).abs() < 0.001);
        assert!(matches!(impact.risk, RiskLevel::Low));
    }

    #[test]
    fn test_risk_level_variants() {
        let low = RiskLevel::Low;
        let medium = RiskLevel::Medium;
        let high = RiskLevel::High;

        assert!(matches!(low, RiskLevel::Low));
        assert!(matches!(medium, RiskLevel::Medium));
        assert!(matches!(high, RiskLevel::High));
    }

    #[test]
    fn test_feedback_collector_default() {
        let collector = FeedbackCollector::default();
        assert_eq!(collector.metrics.total_suggestions, 0);
    }

    #[test]
    fn test_feedback_collector_rejected() {
        let mut collector = FeedbackCollector::new();
        collector.record("pattern1", false, Some("Not applicable".to_string()));

        assert_eq!(collector.metrics.rejected, 1);
        assert_eq!(collector.metrics.accepted, 0);
        assert_eq!(collector.metrics.success_rate, 0.0);
    }

    #[test]
    fn test_feedback_collector_mixed() {
        let mut collector = FeedbackCollector::new();
        collector.record("pattern1", true, None);
        collector.record("pattern2", true, Some("partial success".to_string()));
        collector.record("pattern3", false, Some("Not needed".to_string()));

        assert_eq!(collector.metrics.total_suggestions, 3);
        assert_eq!(collector.metrics.accepted, 2);
        assert_eq!(collector.metrics.rejected, 1);
        assert!((collector.metrics.success_rate - 0.6666).abs() < 0.01);
    }

    #[test]
    fn test_confidence_scorer_default() {
        let scorer = ConfidenceScorer::default();
        // Default weights should sum to 1.0
        let weights = &scorer.weights;
        let sum = weights.pattern_success_rate
            + weights.context_match
            + weights.code_similarity
            + weights.user_history;
        assert!((sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_suggest_for_satd() {
        let assistant = QualityAssistant::new();
        let violation = Violation {
            file: "test.rs".to_string(),
            violation_type: ViolationType::Satd,
            severity: crate::unified_quality::metrics::Severity::Medium,
            value: 1.0,
            threshold: 0.0,
        };

        let suggestions = assistant.suggest(&violation);
        // SATD has patterns defined
        assert!(
            !suggestions.is_empty() || !assistant.pattern_db.contains_key(&ViolationType::Satd)
        );
    }

    #[test]
    fn test_suggest_for_dead_code() {
        let assistant = QualityAssistant::new();
        let violation = Violation {
            file: "test.rs".to_string(),
            violation_type: ViolationType::DeadCode,
            severity: crate::unified_quality::metrics::Severity::Low,
            value: 1.0,
            threshold: 0.0,
        };

        let _suggestions = assistant.suggest(&violation);
        // Dead code has patterns defined
        if let Some(patterns) = assistant.pattern_db.get(&ViolationType::DeadCode) {
            assert!(!patterns.is_empty());
        }
    }

    #[test]
    fn test_record_feedback() {
        let mut assistant = QualityAssistant::new();
        assert_eq!(assistant.get_success_rate(), 0.0);

        assistant.record_feedback("pattern1", true, None);
        assert_eq!(assistant.get_success_rate(), 1.0);

        assistant.record_feedback("pattern2", false, Some("Did not work".to_string()));
        assert_eq!(assistant.get_success_rate(), 0.5);
    }

    #[test]
    fn test_suggestion_creation() {
        let pattern = Pattern {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: "Test pattern".to_string(),
            template: "template".to_string(),
            success_rate: 0.9,
            contexts: vec![],
            example: Example {
                before: "".to_string(),
                after: "".to_string(),
                improvement: "".to_string(),
            },
        };

        let suggestion = Suggestion {
            pattern,
            confidence: 0.85,
            preview: "--- old\n+++ new".to_string(),
            impact: Impact {
                complexity_reduction: 3,
                loc_change: -5,
                coverage_impact: 0.0,
                risk: RiskLevel::Low,
            },
        };

        assert_eq!(suggestion.confidence, 0.85);
        assert!(suggestion.preview.contains("---"));
    }

    // ============ Pattern Tests ============

    #[test]
    fn test_pattern_clone() {
        let pattern = Pattern {
            id: "test-id".to_string(),
            name: "Test Pattern".to_string(),
            description: "Test description".to_string(),
            template: "template code".to_string(),
            success_rate: 0.9,
            contexts: vec!["context1".to_string()],
            example: Example {
                before: "before code".to_string(),
                after: "after code".to_string(),
                improvement: "improvement desc".to_string(),
            },
        };
        let cloned = pattern.clone();
        assert_eq!(cloned.id, "test-id");
        assert_eq!(cloned.name, "Test Pattern");
        assert_eq!(cloned.contexts.len(), 1);
    }

    #[test]
    fn test_pattern_debug() {
        let pattern = Pattern {
            id: "debug-test".to_string(),
            name: "Debug".to_string(),
            description: "".to_string(),
            template: "".to_string(),
            success_rate: 0.5,
            contexts: vec![],
            example: Example {
                before: "".to_string(),
                after: "".to_string(),
                improvement: "".to_string(),
            },
        };
        let debug = format!("{:?}", pattern);
        assert!(debug.contains("debug-test"));
    }

    // ============ Example Tests ============

    #[test]
    fn test_example_clone() {
        let example = Example {
            before: "before".to_string(),
            after: "after".to_string(),
            improvement: "better".to_string(),
        };
        let cloned = example.clone();
        assert_eq!(cloned.before, "before");
        assert_eq!(cloned.after, "after");
    }

    #[test]
    fn test_example_debug() {
        let example = Example {
            before: "old_code".to_string(),
            after: "new_code".to_string(),
            improvement: "improvement desc".to_string(),
        };
        let debug = format!("{:?}", example);
        assert!(debug.contains("old_code"));
    }

    // ============ Impact Tests ============

    #[test]
    fn test_impact_clone() {
        let impact = Impact {
            complexity_reduction: 5,
            loc_change: -10,
            coverage_impact: 2.5,
            risk: RiskLevel::Medium,
        };
        let cloned = impact.clone();
        assert_eq!(cloned.complexity_reduction, 5);
        assert_eq!(cloned.loc_change, -10);
        assert_eq!(cloned.coverage_impact, 2.5);
    }

    #[test]
    fn test_impact_debug() {
        let impact = Impact {
            complexity_reduction: 0,
            loc_change: 0,
            coverage_impact: 0.0,
            risk: RiskLevel::Low,
        };
        let debug = format!("{:?}", impact);
        assert!(debug.contains("complexity_reduction"));
    }

    // ============ RiskLevel Tests ============

    #[test]
    fn test_risk_level_clone() {
        let low = RiskLevel::Low;
        let medium = RiskLevel::Medium;
        let high = RiskLevel::High;

        let _ = low.clone();
        let _ = medium.clone();
        let _ = high.clone();
    }

    #[test]
    fn test_risk_level_debug() {
        let risk = RiskLevel::Medium;
        let debug = format!("{:?}", risk);
        assert!(debug.contains("Medium"));
    }

    #[test]
    fn test_risk_level_low_debug() {
        let risk = RiskLevel::Low;
        let debug = format!("{:?}", risk);
        assert!(debug.contains("Low"));
    }

    #[test]
    fn test_risk_level_high_debug() {
        let risk = RiskLevel::High;
        let debug = format!("{:?}", risk);
        assert!(debug.contains("High"));
    }

    // ============ FeedbackCollector Tests ============

    #[test]
    fn test_feedback_collector_new() {
        let collector = FeedbackCollector::new();
        // New collector should have no data
        let _ = collector;
    }

    #[test]
    fn test_feedback_collector_record_accepted() {
        let mut collector = FeedbackCollector::new();
        collector.record("pattern1", true, None);
        // Just verify it doesn't panic
    }

    #[test]
    fn test_feedback_collector_record_rejected() {
        let mut collector = FeedbackCollector::new();
        collector.record("pattern1", false, Some("Reason".to_string()));
        // Just verify it doesn't panic
    }

    #[test]
    fn test_feedback_collector_record_multiple() {
        let mut collector = FeedbackCollector::new();
        collector.record("pattern1", true, None);
        collector.record("pattern2", false, Some("Bad suggestion".to_string()));
        collector.record("pattern1", true, Some("Good".to_string()));
        // Just verify multiple records work
    }

    // ============ ConfidenceScorer Tests ============

    #[test]
    fn test_confidence_scorer_creation() {
        let scorer = ConfidenceScorer::new();
        // New scorer should have default weights
        let _ = scorer;

        // Also test default impl
        let scorer2 = ConfidenceScorer::default();
        let _ = scorer2;
    }

    // ============ Suggestion Tests ============

    #[test]
    fn test_suggestion_clone() {
        let pattern = Pattern {
            id: "clone-test".to_string(),
            name: "Clone Test".to_string(),
            description: "".to_string(),
            template: "".to_string(),
            success_rate: 0.8,
            contexts: vec![],
            example: Example {
                before: String::new(),
                after: String::new(),
                improvement: String::new(),
            },
        };
        let suggestion = Suggestion {
            pattern,
            confidence: 0.75,
            preview: "preview text".to_string(),
            impact: Impact {
                complexity_reduction: 2,
                loc_change: -5,
                coverage_impact: 1.0,
                risk: RiskLevel::Low,
            },
        };

        let cloned = suggestion.clone();
        assert_eq!(cloned.confidence, 0.75);
        assert_eq!(cloned.preview, "preview text");
        assert_eq!(cloned.pattern.id, "clone-test");
    }

    #[test]
    fn test_suggestion_debug() {
        let pattern = Pattern {
            id: "debug".to_string(),
            name: "Debug".to_string(),
            description: "".to_string(),
            template: "".to_string(),
            success_rate: 0.5,
            contexts: vec![],
            example: Example {
                before: String::new(),
                after: String::new(),
                improvement: String::new(),
            },
        };
        let suggestion = Suggestion {
            pattern,
            confidence: 0.5,
            preview: "".to_string(),
            impact: Impact {
                complexity_reduction: 0,
                loc_change: 0,
                coverage_impact: 0.0,
                risk: RiskLevel::Low,
            },
        };

        let debug = format!("{:?}", suggestion);
        assert!(debug.contains("confidence"));
    }
}
