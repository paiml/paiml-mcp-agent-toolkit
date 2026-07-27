#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_penalty_curve_application() {
        assert_eq!(PenaltyCurve::Linear.apply(2.0, 3.0), 6.0);
        assert!(PenaltyCurve::Logarithmic.apply(2.0, 3.0) > 0.0);
        assert_eq!(PenaltyCurve::Quadratic.apply(2.0, 3.0), 12.0);
        assert!(PenaltyCurve::Exponential.apply(2.0, 1.0) > 7.0);
    }

    #[test]
    fn test_config_serialization() -> Result<()> {
        let config = TdgConfig::default();
        let toml_str = toml::to_string(&config)?;
        assert!(toml_str.contains("structural_complexity"));
        assert!(toml_str.contains("max_cyclomatic_complexity"));
        Ok(())
    }

    #[test]
    fn test_config_from_file() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(
            temp_file,
            r#"
[weights]
structural_complexity = 30.0
semantic_complexity = 20.0
duplication = 15.0
coupling = 15.0
documentation = 10.0
consistency = 10.0

[thresholds]
max_cyclomatic_complexity = 15
max_cognitive_complexity = 20
max_nesting_depth = 4
min_token_sequence = 40
similarity_threshold = 0.9
max_coupling = 12
min_doc_coverage = 0.7

[penalties]
complexity_penalty_base = "Logarithmic"
duplication_penalty_curve = "Linear"
coupling_penalty_curve = "Quadratic"

[language_overrides]
"#
        )?;

        let config = TdgConfig::from_file(temp_file.path())?;
        assert_eq!(config.weights.structural_complexity, 30.0);
        assert_eq!(config.thresholds.max_cyclomatic_complexity, 15); // Test file override, not default
        assert!(matches!(
            config.penalties.complexity_penalty_base,
            PenaltyCurve::Logarithmic
        ));

        Ok(())
    }

    // ============ WeightConfig Tests ============

    #[test]
    fn test_weight_config_default() {
        let config = WeightConfig::default();
        assert_eq!(config.structural_complexity, 25.0);
        assert_eq!(config.semantic_complexity, 20.0);
        assert_eq!(config.duplication, 20.0);
        assert_eq!(config.coupling, 15.0);
        assert_eq!(config.documentation, 10.0);
        assert_eq!(config.consistency, 10.0);
    }

    #[test]
    fn test_weight_config_clone() {
        let config = WeightConfig::default();
        let cloned = config.clone();
        assert_eq!(cloned.structural_complexity, config.structural_complexity);
        assert_eq!(cloned.semantic_complexity, config.semantic_complexity);
    }

    #[test]
    fn test_weight_config_debug() {
        let config = WeightConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("WeightConfig"));
        assert!(debug_str.contains("structural_complexity"));
    }

    // ============ ThresholdConfig Tests ============

    #[test]
    fn test_threshold_config_default() {
        let config = ThresholdConfig::default();
        assert_eq!(config.max_cyclomatic_complexity, 30);
        assert_eq!(config.max_cognitive_complexity, 25);
        assert_eq!(config.max_nesting_depth, 4);
        assert_eq!(config.min_token_sequence, 50);
        assert_eq!(config.similarity_threshold, 0.85);
        assert_eq!(config.max_coupling, 15);
        assert_eq!(config.min_doc_coverage, 0.75);
    }

    #[test]
    fn test_threshold_config_clone() {
        let config = ThresholdConfig::default();
        let cloned = config.clone();
        assert_eq!(
            cloned.max_cyclomatic_complexity,
            config.max_cyclomatic_complexity
        );
        assert_eq!(
            cloned.max_cognitive_complexity,
            config.max_cognitive_complexity
        );
    }

    #[test]
    fn test_threshold_config_debug() {
        let config = ThresholdConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("ThresholdConfig"));
        assert!(debug_str.contains("max_cyclomatic_complexity"));
    }

    // ============ PenaltyConfig Tests ============

    #[test]
    fn test_penalty_config_default() {
        let config = PenaltyConfig::default();
        assert!(matches!(
            config.complexity_penalty_base,
            PenaltyCurve::Logarithmic
        ));
        assert!(matches!(
            config.duplication_penalty_curve,
            PenaltyCurve::Linear
        ));
        assert!(matches!(
            config.coupling_penalty_curve,
            PenaltyCurve::Quadratic
        ));
    }

    #[test]
    fn test_penalty_config_clone() {
        let config = PenaltyConfig::default();
        let cloned = config.clone();
        assert!(matches!(
            cloned.complexity_penalty_base,
            PenaltyCurve::Logarithmic
        ));
    }

    #[test]
    fn test_penalty_config_debug() {
        let config = PenaltyConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("PenaltyConfig"));
    }

    // ============ PenaltyCurve Tests ============

    #[test]
    fn test_penalty_curve_linear() {
        let curve = PenaltyCurve::Linear;
        assert_eq!(curve.apply(0.0, 1.0), 0.0);
        assert_eq!(curve.apply(1.0, 1.0), 1.0);
        assert_eq!(curve.apply(2.0, 3.0), 6.0);
        assert_eq!(curve.apply(10.0, 0.5), 5.0);
    }

    #[test]
    fn test_penalty_curve_logarithmic() {
        let curve = PenaltyCurve::Logarithmic;
        // Value <= 1.0 should return 0.0
        assert_eq!(curve.apply(0.5, 1.0), 0.0);
        assert_eq!(curve.apply(1.0, 1.0), 0.0);
        // Value > 1.0 should use ln
        let result = curve.apply(std::f32::consts::E, 1.0);
        assert!((result - 1.0).abs() < 0.01); // ln(e) ≈ 1
        let result2 = curve.apply(10.0, 2.0);
        assert!(result2 > 4.0); // ln(10) * 2 ≈ 4.6
    }

    #[test]
    fn test_penalty_curve_quadratic() {
        let curve = PenaltyCurve::Quadratic;
        assert_eq!(curve.apply(0.0, 1.0), 0.0);
        assert_eq!(curve.apply(1.0, 1.0), 1.0);
        assert_eq!(curve.apply(2.0, 1.0), 4.0);
        assert_eq!(curve.apply(3.0, 2.0), 18.0); // 3^2 * 2 = 18
    }

    #[test]
    fn test_penalty_curve_exponential() {
        let curve = PenaltyCurve::Exponential;
        assert_eq!(curve.apply(0.0, 1.0), 1.0); // e^0 = 1
        let result = curve.apply(1.0, 1.0);
        assert!((result - std::f32::consts::E).abs() < 0.01); // e^1 ≈ 2.718
        let result2 = curve.apply(2.0, 0.5);
        assert!(result2 > 3.5); // e^2 * 0.5 ≈ 3.7
    }

    #[test]
    fn test_penalty_curve_clone() {
        let curve = PenaltyCurve::Linear;
        let cloned = curve.clone();
        assert!(matches!(cloned, PenaltyCurve::Linear));
    }

    #[test]
    fn test_penalty_curve_debug() {
        let curve = PenaltyCurve::Quadratic;
        let debug_str = format!("{:?}", curve);
        assert_eq!(debug_str, "Quadratic");
    }

    // ============ LanguageOverride Tests ============

    #[test]
    fn test_language_override_all_none() {
        let lo = LanguageOverride {
            max_cognitive_complexity: None,
            min_doc_coverage: None,
            enforce_error_check: None,
            max_function_length: None,
        };
        assert!(lo.max_cognitive_complexity.is_none());
        assert!(lo.min_doc_coverage.is_none());
        assert!(lo.enforce_error_check.is_none());
        assert!(lo.max_function_length.is_none());
    }

    #[test]
    fn test_language_override_all_some() {
        let lo = LanguageOverride {
            max_cognitive_complexity: Some(30),
            min_doc_coverage: Some(0.9),
            enforce_error_check: Some(true),
            max_function_length: Some(100),
        };
        assert_eq!(lo.max_cognitive_complexity, Some(30));
        assert_eq!(lo.min_doc_coverage, Some(0.9));
        assert_eq!(lo.enforce_error_check, Some(true));
        assert_eq!(lo.max_function_length, Some(100));
    }

    #[test]
    fn test_language_override_clone() {
        let lo = LanguageOverride {
            max_cognitive_complexity: Some(20),
            min_doc_coverage: Some(0.8),
            enforce_error_check: None,
            max_function_length: Some(50),
        };
        let cloned = lo.clone();
        assert_eq!(cloned.max_cognitive_complexity, lo.max_cognitive_complexity);
        assert_eq!(cloned.min_doc_coverage, lo.min_doc_coverage);
    }

    #[test]
    fn test_language_override_debug() {
        let lo = LanguageOverride {
            max_cognitive_complexity: Some(25),
            min_doc_coverage: None,
            enforce_error_check: Some(false),
            max_function_length: None,
        };
        let debug_str = format!("{:?}", lo);
        assert!(debug_str.contains("LanguageOverride"));
        assert!(debug_str.contains("25"));
    }

    // ============ TdgConfig Tests ============

    #[test]
    fn test_tdg_config_default() {
        let config = TdgConfig::default();
        assert_eq!(config.weights.structural_complexity, 25.0);
        assert_eq!(config.thresholds.max_cyclomatic_complexity, 30);
        assert!(matches!(
            config.penalties.complexity_penalty_base,
            PenaltyCurve::Logarithmic
        ));
        assert!(config.language_overrides.is_empty());
    }

    #[test]
    fn test_tdg_config_clone() {
        let config = TdgConfig::default();
        let cloned = config.clone();
        assert_eq!(
            cloned.weights.structural_complexity,
            config.weights.structural_complexity
        );
    }

    #[test]
    fn test_tdg_config_debug() {
        let config = TdgConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("TdgConfig"));
    }

    #[test]
    fn test_tdg_config_save_and_load() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let config_path = temp_dir.path().join("test-config.toml");

        let mut config = TdgConfig::default();
        config.weights.structural_complexity = 42.0;
        config.thresholds.max_cyclomatic_complexity = 50;
        config.language_overrides.insert(
            "python".to_string(),
            LanguageOverride {
                max_cognitive_complexity: Some(35),
                min_doc_coverage: Some(0.6),
                enforce_error_check: Some(true),
                max_function_length: Some(200),
            },
        );

        // Save
        config.save(&config_path)?;
        assert!(config_path.exists());

        // Load and verify
        let loaded = TdgConfig::from_file(&config_path)?;
        assert_eq!(loaded.weights.structural_complexity, 42.0);
        assert_eq!(loaded.thresholds.max_cyclomatic_complexity, 50);
        assert!(loaded.language_overrides.contains_key("python"));
        let python_override = loaded.language_overrides.get("python").unwrap();
        assert_eq!(python_override.max_cognitive_complexity, Some(35));

        Ok(())
    }

    #[test]
    fn test_tdg_config_from_file_not_found() {
        let result = TdgConfig::from_file(Path::new("/nonexistent/path/config.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn test_tdg_config_from_file_invalid_toml() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "this is not valid toml {{{{")?;

        let result = TdgConfig::from_file(temp_file.path());
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_config_with_language_overrides() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(
            temp_file,
            r#"
[weights]
structural_complexity = 25.0
semantic_complexity = 20.0
duplication = 20.0
coupling = 15.0
documentation = 10.0
consistency = 10.0

[thresholds]
max_cyclomatic_complexity = 30
max_cognitive_complexity = 25
max_nesting_depth = 4
min_token_sequence = 50
similarity_threshold = 0.85
max_coupling = 15
min_doc_coverage = 0.75

[penalties]
complexity_penalty_base = "Logarithmic"
duplication_penalty_curve = "Linear"
coupling_penalty_curve = "Quadratic"

[language_overrides.rust]
max_cognitive_complexity = 40
min_doc_coverage = 0.9
enforce_error_check = true
max_function_length = 150

[language_overrides.python]
max_cognitive_complexity = 30
min_doc_coverage = 0.7
"#
        )?;

        let config = TdgConfig::from_file(temp_file.path())?;
        assert!(config.language_overrides.contains_key("rust"));
        assert!(config.language_overrides.contains_key("python"));

        let rust_override = config.language_overrides.get("rust").unwrap();
        assert_eq!(rust_override.max_cognitive_complexity, Some(40));
        assert_eq!(rust_override.min_doc_coverage, Some(0.9));
        assert_eq!(rust_override.enforce_error_check, Some(true));
        assert_eq!(rust_override.max_function_length, Some(150));

        let python_override = config.language_overrides.get("python").unwrap();
        assert_eq!(python_override.max_cognitive_complexity, Some(30));
        assert!(python_override.enforce_error_check.is_none());

        Ok(())
    }

    #[test]
    fn test_penalty_curve_serialization() -> Result<()> {
        let curve = PenaltyCurve::Exponential;
        let json = serde_json::to_string(&curve)?;
        assert!(json.contains("Exponential"));

        let deserialized: PenaltyCurve = serde_json::from_str(&json)?;
        assert!(matches!(deserialized, PenaltyCurve::Exponential));
        Ok(())
    }
}

