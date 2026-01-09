// Integration test for DefectAwarePromptGenerator with real OIP data
// Tests end-to-end functionality with actual summary files

use pmat::prompts::DefectAwarePromptGenerator;

#[test]
#[ignore = "Requires external OIP test baseline file"]
fn test_integration_with_real_oip_summary() {
    // Use test baseline from OIP project
    let oip_summary_path = "/tmp/oip-test-baseline.yaml";

    // Verify file exists
    assert!(
        std::path::Path::new(oip_summary_path).exists(),
        "Test baseline not found. Run: cp ../organizational-intelligence-plugin/test-baseline.yaml /tmp/oip-test-baseline.yaml"
    );

    // Load the summary
    let generator = DefectAwarePromptGenerator::from_file(oip_summary_path)
        .expect("Failed to load OIP summary");

    // Verify metadata loaded correctly
    assert_eq!(generator.metadata.repositories_analyzed, 25);
    assert_eq!(generator.metadata.commits_analyzed, 2500);

    // Verify defect patterns loaded
    assert!(!generator.defect_patterns.is_empty());
    assert_eq!(generator.defect_patterns.len(), 2); // ConfigurationErrors, IntegrationFailures

    // Generate a prompt for a real-world task
    let prompt = generator.generate_prompt(
        "Write a configuration parser for YAML files",
        "Building a new config system for a microservices architecture",
    );

    // Verify prompt contains key elements
    assert!(prompt.contains("Write a configuration parser for YAML files"));
    assert!(prompt.contains("25 repositories"));
    assert!(prompt.contains("2500 commits"));
    assert!(prompt.contains("ConfigurationErrors"));
    assert!(prompt.contains("IntegrationFailures"));
    assert!(prompt.contains("Minimum TDG Score: 85"));
    assert!(prompt.contains("Test Coverage: 85%"));
    assert!(prompt.contains("pmat analyze tdg --threshold 85"));

    // Test prevention prompt for ConfigurationErrors
    let prevention = generator
        .generate_prevention_prompt("ConfigurationErrors")
        .expect("Should generate prevention prompt");

    assert!(prevention.contains("Preventing ConfigurationErrors"));
    assert!(prevention.contains("25 occurrences"));
    assert!(prevention.contains("TDG 45.2"));

    // Print the generated prompt for manual inspection
    println!("\n========== GENERATED PROMPT ==========\n");
    println!("{}", prompt);
    println!("\n========== PREVENTION PROMPT ==========\n");
    println!("{}", prevention);
}

#[test]
fn test_prompt_quality_meets_requirements() {
    let oip_summary_path = "/tmp/oip-test-baseline.yaml";

    if !std::path::Path::new(oip_summary_path).exists() {
        println!("Skipping test - OIP summary not available");
        return;
    }

    let generator = DefectAwarePromptGenerator::from_file(oip_summary_path).unwrap();
    let prompt = generator.generate_prompt(
        "Implement HTTP client with retry logic",
        "Adding resilient API communication",
    );

    // Quality requirements for prompts
    assert!(prompt.len() > 100, "Prompt should be substantial");
    assert!(
        prompt.contains("Quality Requirements"),
        "Must include quality section"
    );
    assert!(
        prompt.contains("Quality Gates"),
        "Must include quality gates"
    );
    assert!(
        prompt.contains("Defect Patterns"),
        "Must include defect patterns"
    );

    // Should only include high-frequency defects (>= 10)
    assert!(prompt.contains("ConfigurationErrors")); // frequency = 25
    assert!(prompt.contains("IntegrationFailures")); // frequency = 18
}
