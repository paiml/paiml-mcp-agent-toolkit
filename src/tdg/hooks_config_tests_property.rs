// Property-based tests for hooks_config types.
// Included from hooks_config.rs mod property_tests — no `use` imports or inner attributes allowed.

proptest! {
    #[test]
    fn config_serialization_roundtrip(max_score in 0.0f32..100.0) {
        let mut config = TdgHooksConfig::default();
        config.quality_gates.max_score_drop = max_score;

        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: TdgHooksConfig = toml::from_str(&toml_str).unwrap();

        prop_assert!((deserialized.quality_gates.max_score_drop - max_score).abs() < 0.01);
    }

    #[test]
    fn enforcement_mode_string_conversion(mode_val in 0u8..3) {
        let mode = match mode_val {
            0 => EnforcementMode::Strict,
            1 => EnforcementMode::Warning,
            _ => EnforcementMode::Disabled,
        };

        let mode_str = mode.to_string();
        prop_assert!(mode_str == "strict" || mode_str == "warning" || mode_str == "disabled");
    }
}
