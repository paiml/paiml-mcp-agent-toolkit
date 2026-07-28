#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::High > Severity::Medium);
        assert!(Severity::Medium > Severity::Low);
    }

    #[test]
    fn test_repetition_severity() {
        let config = EntropyConfig::default();
        let detector = ViolationDetector::new(config);

        assert_eq!(detector.calculate_repetition_severity(3), Severity::Low);
        assert_eq!(detector.calculate_repetition_severity(7), Severity::Medium);
        assert_eq!(detector.calculate_repetition_severity(15), Severity::High);
    }
}

// Coverage tests: type serialization, utility methods, pattern matching
include!("violation_detector_coverage_tests.rs");

// Detection tests: cross-file duplication, inconsistency, dedup, sorting, filtering
include!("violation_detector_detection_tests.rs");
