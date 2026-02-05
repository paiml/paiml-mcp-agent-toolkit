// RED tests for DefectAwarePromptGenerator (EXTREME TDD)
// These tests are written BEFORE implementation

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use crate::prompts::DefectAwarePromptGenerator;
    use tempfile::NamedTempFile;

    // RED TEST 1: Load OIP analysis from YAML file
    #[test]
    fn test_load_from_file() {
        let yaml = r#"
organizational_insights:
  top_defect_categories:
    - category: ConfigurationErrors
      frequency: 25
      confidence: 0.78
      quality_signals:
        avg_tdg_score: 45.2
        max_tdg_score: 60.0
        avg_complexity: 8.0
        avg_test_coverage: 0.5
        satd_instances: 5
        avg_lines_changed: 10.0
        avg_files_per_commit: 2.0
      examples: []
code_quality_thresholds:
  tdg_minimum: 85.0
  test_coverage_minimum: 0.85
  max_function_length: 50
  max_cyclomatic_complexity: 10
metadata:
  analysis_date: "2025-11-15T12:00:00Z"
  repositories_analyzed: 25
  commits_analyzed: 2500
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        std::io::Write::write_all(&mut temp_file, yaml.as_bytes()).unwrap();

        let generator = DefectAwarePromptGenerator::from_file(temp_file.path()).unwrap();

        assert_eq!(generator.metadata.repositories_analyzed, 25);
        assert_eq!(generator.metadata.commits_analyzed, 2500);
        assert_eq!(generator.defect_patterns.len(), 1);
        assert_eq!(generator.defect_patterns[0].category, "ConfigurationErrors");
        assert_eq!(generator.defect_patterns[0].frequency, 25);
    }

    // RED TEST 2: Generate context-aware prompt for a task
    #[test]
    fn test_generate_prompt_includes_defect_patterns() {
        let generator = create_test_generator();

        let prompt = generator.generate_prompt(
            "Write a configuration parser for YAML files",
            "Building a new config system",
        );

        // Must include task
        assert!(prompt.contains("Write a configuration parser for YAML files"));

        // Must include organizational context
        assert!(prompt.contains("25 repositories"));
        assert!(prompt.contains("2500 commits"));

        // Must include defect patterns
        assert!(prompt.contains("ConfigurationErrors"));
        assert!(prompt.contains("25 occurrences"));
        assert!(prompt.contains("TDG: 45.2"));

        // Must include quality requirements
        assert!(prompt.contains("Minimum TDG Score: 85"));
        assert!(prompt.contains("Test Coverage: 85%"));
    }

    // RED TEST 3: Generate prevention-specific prompt
    #[test]
    fn test_generate_prevention_prompt() {
        let generator = create_test_generator();

        let prompt = generator
            .generate_prevention_prompt("ConfigurationErrors")
            .unwrap();

        assert!(prompt.contains("Preventing ConfigurationErrors"));
        assert!(prompt.contains("Historical Frequency**: 25 occurrences"));
        assert!(prompt.contains("Average Code Quality**: TDG 45.2/100"));
    }

    // RED TEST 4: Prevention prompt returns None for unknown category
    #[test]
    fn test_prevention_prompt_unknown_category() {
        let generator = create_test_generator();

        let result = generator.generate_prevention_prompt("NonexistentCategory");

        assert!(result.is_none());
    }

    // RED TEST 5: Only include high-frequency defects in prompts
    #[test]
    fn test_only_high_frequency_defects_included() {
        let yaml = r#"
organizational_insights:
  top_defect_categories:
    - category: ConfigurationErrors
      frequency: 25
      confidence: 0.78
      quality_signals:
        avg_tdg_score: 45.2
        max_tdg_score: 60.0
        avg_complexity: 8.0
        avg_test_coverage: 0.5
        satd_instances: 5
        avg_lines_changed: 10.0
        avg_files_per_commit: 2.0
      examples: []
    - category: LowFrequencyDefect
      frequency: 3
      confidence: 0.9
      quality_signals:
        avg_tdg_score: 90.0
        max_tdg_score: 95.0
        avg_complexity: 2.0
        avg_test_coverage: 0.9
        satd_instances: 0
        avg_lines_changed: 5.0
        avg_files_per_commit: 1.0
      examples: []
code_quality_thresholds:
  tdg_minimum: 85.0
  test_coverage_minimum: 0.85
  max_function_length: 50
  max_cyclomatic_complexity: 10
metadata:
  analysis_date: "2025-11-15T12:00:00Z"
  repositories_analyzed: 25
  commits_analyzed: 2500
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        std::io::Write::write_all(&mut temp_file, yaml.as_bytes()).unwrap();

        let generator = DefectAwarePromptGenerator::from_file(temp_file.path()).unwrap();
        let prompt = generator.generate_prompt("Test task", "Test context");

        // High frequency should be included
        assert!(prompt.contains("ConfigurationErrors"));

        // Low frequency should NOT be included (frequency < 10)
        assert!(!prompt.contains("LowFrequencyDefect"));
    }

    // RED TEST 6: Quality gates included in output
    #[test]
    fn test_quality_gates_in_prompt() {
        let generator = create_test_generator();
        let prompt = generator.generate_prompt("Test task", "Test context");

        assert!(prompt.contains("pmat analyze tdg --threshold 85"));
        assert!(prompt.contains("cargo test --all-features"));
        assert!(prompt.contains("cargo llvm-cov"));
    }

    // Helper function to create test generator
    fn create_test_generator() -> DefectAwarePromptGenerator {
        let yaml = r#"
organizational_insights:
  top_defect_categories:
    - category: ConfigurationErrors
      frequency: 25
      confidence: 0.78
      quality_signals:
        avg_tdg_score: 45.2
        max_tdg_score: 60.0
        avg_complexity: 8.0
        avg_test_coverage: 0.5
        satd_instances: 5
        avg_lines_changed: 10.0
        avg_files_per_commit: 2.0
      examples: []
code_quality_thresholds:
  tdg_minimum: 85.0
  test_coverage_minimum: 0.85
  max_function_length: 50
  max_cyclomatic_complexity: 10
metadata:
  analysis_date: "2025-11-15T12:00:00Z"
  repositories_analyzed: 25
  commits_analyzed: 2500
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        std::io::Write::write_all(&mut temp_file, yaml.as_bytes()).unwrap();

        DefectAwarePromptGenerator::from_file(temp_file.path()).unwrap()
    }
}
