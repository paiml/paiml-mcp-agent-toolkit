// Tests for signal_collector module: collector sources, aggregation, signal evidence, defect conversion.
// Included by signal_collector.rs - shares parent module scope.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;

    // ==================== SignalSource Tests ====================

    #[test]
    fn test_rustc_collector_source() {
        let collector = RustcCollector;
        assert_eq!(collector.source(), SignalSource::Rustc);
    }

    #[test]
    fn test_clippy_collector_source() {
        let collector = ClippyCollector;
        assert_eq!(collector.source(), SignalSource::Clippy);
    }

    #[test]
    fn test_test_collector_source() {
        let collector = TestCollector;
        assert_eq!(collector.source(), SignalSource::CargoTest);
    }

    // ==================== AggregatedCollector Tests ====================

    #[test]
    fn test_aggregated_collector_new() {
        let collector = AggregatedCollector::new();
        // Should have 3 default collectors: Rustc, Clippy, Test
        assert_eq!(collector.collector_count(), 3);
    }

    #[test]
    fn test_aggregated_collector_default() {
        let collector = AggregatedCollector::default();
        assert_eq!(collector.collector_count(), 3);
    }

    #[test]
    fn test_aggregated_collector_add_collector() {
        let mut collector = AggregatedCollector::new();
        assert_eq!(collector.collector_count(), 3);

        // Add custom collector
        collector.add_collector(Box::new(RustcCollector));
        assert_eq!(collector.collector_count(), 4);
    }

    // ==================== SignalEvidence Tests ====================

    #[test]
    fn test_signal_evidence_creation() {
        let signal = SignalEvidence {
            source: SignalSource::Rustc,
            raw_message: "error[E0308]: mismatched types".to_string(),
            error_code: Some("E0308".to_string()),
            weight: 1.0,
        };

        assert_eq!(signal.source, SignalSource::Rustc);
        assert!(signal.raw_message.contains("mismatched"));
        assert_eq!(signal.error_code, Some("E0308".to_string()));
        assert!((signal.weight - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_signal_evidence_serialization() {
        let signal = SignalEvidence {
            source: SignalSource::Clippy,
            raw_message: "warning: unused variable".to_string(),
            error_code: Some("clippy::unused".to_string()),
            weight: 0.5,
        };

        let serialized = serde_json::to_string(&signal).expect("Should serialize");
        let deserialized: SignalEvidence =
            serde_json::from_str(&serialized).expect("Should deserialize");

        assert_eq!(signal.source, deserialized.source);
        assert_eq!(signal.raw_message, deserialized.raw_message);
        assert_eq!(signal.error_code, deserialized.error_code);
        assert!((signal.weight - deserialized.weight).abs() < f32::EPSILON);
    }

    // ==================== signals_to_defects Tests ====================

    #[test]
    fn test_signals_to_defects_empty() {
        let collector = AggregatedCollector::new();
        let signals: Vec<SignalEvidence> = vec![];
        let defects = collector.signals_to_defects(signals);
        assert!(defects.is_empty());
    }

    #[test]
    fn test_signals_to_defects_rustc_error() {
        let collector = AggregatedCollector::new();
        let signals = vec![SignalEvidence {
            source: SignalSource::Rustc,
            raw_message: "error[E0308]: mismatched types".to_string(),
            error_code: Some("E0308".to_string()),
            weight: 1.0,
        }];

        let defects = collector.signals_to_defects(signals);
        assert_eq!(defects.len(), 1);
        assert_eq!(defects[0].category, DefectCategory::TypeErrors);
        assert_eq!(defects[0].severity, Severity::Critical);
    }

    #[test]
    fn test_signals_to_defects_clippy_warning() {
        let collector = AggregatedCollector::new();
        let signals = vec![SignalEvidence {
            source: SignalSource::Clippy,
            raw_message: "warning: unused variable".to_string(),
            error_code: Some("clippy::unused_variable".to_string()),
            weight: 0.5,
        }];

        let defects = collector.signals_to_defects(signals);
        assert_eq!(defects.len(), 1);
        assert_eq!(defects[0].severity, Severity::Medium);
    }

    #[test]
    fn test_signals_to_defects_test_failure() {
        let collector = AggregatedCollector::new();
        let signals = vec![SignalEvidence {
            source: SignalSource::CargoTest,
            raw_message: "Test failed: test_something".to_string(),
            error_code: None,
            weight: 1.0,
        }];

        let defects = collector.signals_to_defects(signals);
        assert_eq!(defects.len(), 1);
        assert_eq!(defects[0].severity, Severity::High);
    }

    #[test]
    fn test_signals_to_defects_multiple_signals() {
        let collector = AggregatedCollector::new();
        let signals = vec![
            SignalEvidence {
                source: SignalSource::Rustc,
                raw_message: "error[E0308]: mismatched types".to_string(),
                error_code: Some("E0308".to_string()),
                weight: 1.0,
            },
            SignalEvidence {
                source: SignalSource::Clippy,
                raw_message: "warning: unused variable".to_string(),
                error_code: None,
                weight: 0.5,
            },
            SignalEvidence {
                source: SignalSource::CargoTest,
                raw_message: "Test failed: test_foo".to_string(),
                error_code: None,
                weight: 1.0,
            },
        ];

        let defects = collector.signals_to_defects(signals);
        assert_eq!(defects.len(), 3);
    }

    #[test]
    fn test_signals_to_defects_ownership_error() {
        let collector = AggregatedCollector::new();
        let signals = vec![SignalEvidence {
            source: SignalSource::Rustc,
            raw_message: "error[E0382]: borrow of moved value".to_string(),
            error_code: Some("E0382".to_string()),
            weight: 1.0,
        }];

        let defects = collector.signals_to_defects(signals);
        assert_eq!(defects.len(), 1);
        assert_eq!(defects[0].category, DefectCategory::OwnershipBorrow);
    }

    #[test]
    fn test_signals_to_defects_unknown_error_code() {
        let collector = AggregatedCollector::new();
        let signals = vec![SignalEvidence {
            source: SignalSource::Rustc,
            raw_message: "error[E9999]: unknown error".to_string(),
            error_code: Some("E9999".to_string()),
            weight: 1.0,
        }];

        let defects = collector.signals_to_defects(signals);
        assert_eq!(defects.len(), 1);
        // Unknown error code should fallback to Configuration category
        assert_eq!(defects[0].category, DefectCategory::Configuration);
    }

    #[test]
    fn test_signals_to_defects_no_error_code() {
        let collector = AggregatedCollector::new();
        let signals = vec![SignalEvidence {
            source: SignalSource::Rustc,
            raw_message: "error: something went wrong".to_string(),
            error_code: None,
            weight: 1.0,
        }];

        let defects = collector.signals_to_defects(signals);
        assert_eq!(defects.len(), 1);
        // No error code should fallback to Configuration category
        assert_eq!(defects[0].category, DefectCategory::Configuration);
    }

    // ==================== Clippy Weight Tests ====================

    #[test]
    fn test_clippy_correctness_weight() {
        let collector = AggregatedCollector::new();
        let signals = vec![SignalEvidence {
            source: SignalSource::Clippy,
            raw_message: "error: clippy correctness issue".to_string(),
            error_code: Some("clippy::correctness::something".to_string()),
            weight: 1.0,
        }];

        let defects = collector.signals_to_defects(signals);
        assert_eq!(defects.len(), 1);
        // Correctness issues should have weight 1.0
        assert!((defects[0].signals[0].weight - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_clippy_suspicious_weight() {
        // The weight is already set in the signal, not modified by signals_to_defects
        // This test verifies the signal structure is preserved
        let collector = AggregatedCollector::new();
        let signals = vec![SignalEvidence {
            source: SignalSource::Clippy,
            raw_message: "warning: suspicious pattern".to_string(),
            error_code: Some("clippy::suspicious::something".to_string()),
            weight: 0.9,
        }];

        let defects = collector.signals_to_defects(signals);
        assert_eq!(defects.len(), 1);
        assert!((defects[0].signals[0].weight - 0.9).abs() < f32::EPSILON);
    }
}
