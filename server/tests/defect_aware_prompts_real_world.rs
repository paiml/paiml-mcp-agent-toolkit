// Real-world test with actual paiml organization data
// Validates end-to-end workflow: OIP analyze → summarize → PMAT prompt generation

use pmat::prompts::DefectAwarePromptGenerator;

#[test]
#[ignore] // Run manually: cargo test --test defect_aware_prompts_real_world -- --include-ignored
fn test_real_world_paiml_analysis() {
    let summary_path = "/tmp/paiml-summary.yaml";

    // Skip if summary doesn't exist
    if !std::path::Path::new(summary_path).exists() {
        println!("⚠️  Skipping test - run OIP analysis first:");
        println!("   cd ../organizational-intelligence-plugin");
        println!("   cargo run -- analyze --org paiml --output /tmp/paiml-full-analysis.yaml");
        println!("   cargo run -- summarize --input /tmp/paiml-full-analysis.yaml --output /tmp/paiml-summary.yaml --strip-pii");
        return;
    }

    // Load real paiml data
    let generator = DefectAwarePromptGenerator::from_file(summary_path)
        .expect("Failed to load paiml summary");

    println!("\n📊 Real PAIML Organization Data:");
    println!("   Repositories: {}", generator.metadata.repositories_analyzed);
    println!("   Commits: {}", generator.metadata.commits_analyzed);
    println!("   Defect patterns: {}", generator.defect_patterns.len());

    // Generate prompt for a real-world task
    let task = "Implement a new HTTP client for external API integration";
    let context = "Building a resilient service that needs to communicate with third-party APIs";

    let prompt = generator.generate_prompt(task, context);

    println!("\n========== REAL-WORLD PAIML PROMPT ==========\n");
    println!("{}", prompt);
    println!("\n=============================================\n");

    // Validate prompt quality
    assert!(prompt.contains(task), "Prompt must include task");
    assert!(prompt.contains(&generator.metadata.repositories_analyzed.to_string()),
            "Prompt must include repo count");
    assert!(prompt.contains("Quality Requirements"), "Must have quality section");
    assert!(prompt.contains("Quality Gates"), "Must have quality gates");

    // Should include high-frequency defects from paiml
    assert!(prompt.contains("Common Defect Patterns"), "Must have defect patterns section");

    // Generate prevention prompt for integration failures (common in paiml)
    if let Some(prevention) = generator.generate_prevention_prompt("IntegrationFailures") {
        println!("\n========== INTEGRATION FAILURES PREVENTION ==========\n");
        println!("{}", prevention);
        println!("\n====================================================\n");

        assert!(prevention.contains("IntegrationFailures"));
        assert!(prevention.contains("Historical Frequency"));
    }
}

#[test]
#[ignore]
fn test_paiml_defect_patterns() {
    let summary_path = "/tmp/paiml-summary.yaml";

    if !std::path::Path::new(summary_path).exists() {
        println!("⚠️  Skipping - run OIP analysis first");
        return;
    }

    let generator = DefectAwarePromptGenerator::from_file(summary_path).unwrap();

    println!("\n📋 PAIML Defect Patterns (frequency >= 10):");
    for pattern in generator.defect_patterns.iter().filter(|p| p.frequency >= 10) {
        let avg_tdg = pattern.quality_signals.avg_tdg_score.unwrap_or(0.0);
        println!("   • {} (freq: {}, TDG: {:.1})",
                 pattern.category,
                 pattern.frequency,
                 avg_tdg);
    }

    // Validate that prompts only include high-frequency defects
    let prompt = generator.generate_prompt("Test task", "Test context");

    for pattern in generator.defect_patterns.iter() {
        if pattern.frequency >= 10 {
            assert!(prompt.contains(&pattern.category),
                    "High-frequency defect {} should be in prompt", pattern.category);
        } else {
            assert!(!prompt.contains(&pattern.category),
                    "Low-frequency defect {} should NOT be in prompt", pattern.category);
        }
    }
}

#[test]
#[ignore]
fn test_paiml_vs_test_data_comparison() {
    let test_baseline = "/tmp/oip-test-baseline.yaml";
    let paiml_summary = "/tmp/paiml-summary.yaml";

    if !std::path::Path::new(paiml_summary).exists() {
        println!("⚠️  Skipping - run OIP analysis first");
        return;
    }

    let paiml_gen = DefectAwarePromptGenerator::from_file(paiml_summary).unwrap();

    println!("\n🔍 Comparison: Test Data vs Real PAIML Data\n");
    println!("   PAIML:");
    println!("      Repositories: {}", paiml_gen.metadata.repositories_analyzed);
    println!("      Commits: {}", paiml_gen.metadata.commits_analyzed);
    println!("      Patterns: {}", paiml_gen.defect_patterns.len());
    println!("      Analysis Date: {}", paiml_gen.metadata.analysis_date);

    if std::path::Path::new(test_baseline).exists() {
        let test_gen = DefectAwarePromptGenerator::from_file(test_baseline).unwrap();
        println!("\n   Test Baseline:");
        println!("      Repositories: {}", test_gen.metadata.repositories_analyzed);
        println!("      Commits: {}", test_gen.metadata.commits_analyzed);
        println!("      Patterns: {}", test_gen.defect_patterns.len());

        // Compare prompt lengths
        let paiml_prompt = paiml_gen.generate_prompt("Test task", "Test context");
        let test_prompt = test_gen.generate_prompt("Test task", "Test context");

        println!("\n   Prompt Comparison:");
        println!("      PAIML prompt length: {} chars", paiml_prompt.len());
        println!("      Test prompt length: {} chars", test_prompt.len());
    }
}
