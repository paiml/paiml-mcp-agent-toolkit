mod tests {
    use super::*;

    #[test]
    fn test_enforcement_state_serialization() {
        let state = EnforcementState::Analyzing;
        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(json, "\"ANALYZING\"");
    }

    #[test]
    fn test_quality_profile_default() {
        let profile = QualityProfile::default();
        assert_eq!(profile.coverage_min, 80.0);
        assert_eq!(profile.complexity_max, 20);
        assert_eq!(profile.satd_allowed, 0);
    }

    #[test]
    fn test_enforcement_result_serialization() {
        let result = EnforcementResult {
            state: EnforcementState::Violating,
            score: 0.5,
            target: 1.0,
            current_file: Some("test.rs".to_string()),
            violations: vec![],
            next_action: "test".to_string(),
            progress: EnforcementProgress {
                files_completed: 1,
                files_remaining: 2,
                estimated_iterations: 3,
            },
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("VIOLATING"));
        assert!(json.contains("0.5"));
    }
}
