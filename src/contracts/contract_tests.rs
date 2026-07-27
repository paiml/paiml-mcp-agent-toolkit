#[cfg(test)]
mod contract_default_tests {
    use super::*;

    #[test]
    fn test_default_tdg_threshold() {
        assert!((default_tdg_threshold() - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_default_target_complexity() {
        assert_eq!(default_target_complexity(), 10);
    }

    #[test]
    fn test_default_timeout() {
        assert_eq!(default_timeout(), 60);
    }

    #[test]
    fn test_analyze_tdg_contract_serde_defaults() {
        let json = r#"{"base":{}}"#;
        let contract: AnalyzeTdgContract = serde_json::from_str(json).unwrap();
        assert!((contract.threshold - 1.5).abs() < f64::EPSILON);
        assert!(!contract.include_components);
        assert!(!contract.critical_only);
    }

    #[test]
    fn test_refactor_auto_contract_serde_defaults() {
        let json = r#"{"file":"test.rs"}"#;
        let contract: RefactorAutoContract = serde_json::from_str(json).unwrap();
        assert_eq!(contract.target_complexity, 10);
        assert_eq!(contract.timeout, 60);
        assert!(!contract.dry_run);
    }
}

