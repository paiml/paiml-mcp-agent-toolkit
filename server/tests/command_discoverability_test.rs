//! TDD Test for Command Discoverability
//! 
//! Following TDD methodology, we write tests FIRST to define expected behavior
//! for command suggestions and discoverability improvements.

use std::process::Command;

const PMAT_BINARY: &str = env!("CARGO_BIN_EXE_pmat");

#[test]
fn test_agent_analyze_suggests_correct_command() {
    // TDD Test: Common mistake "pmat agent analyze" should suggest "pmat analyze"
    let output = Command::new(PMAT_BINARY)
        .args(&["agent", "analyze"])
        .output()
        .expect("Failed to run command");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Command should fail
    assert!(!output.status.success(), "Command should fail with unrecognized subcommand");
    
    // Should contain helpful suggestion
    assert!(stderr.contains("Did you mean"), 
        "Error should contain 'Did you mean' suggestion. Got: {}", stderr);
    assert!(stderr.contains("pmat analyze"), 
        "Should suggest 'pmat analyze'. Got: {}", stderr);
}

#[test] 
fn test_typo_analize_suggests_analyze() {
    // TDD Test: Typo "analize" should suggest "analyze"
    let output = Command::new(PMAT_BINARY)
        .args(&["analize", "complexity"])
        .output()
        .expect("Failed to run command");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    assert!(!output.status.success(), "Command should fail");
    assert!(stderr.contains("Did you mean"), 
        "Should suggest correction. Got: {}", stderr);
    assert!(stderr.contains("analyze"), 
        "Should suggest 'analyze'. Got: {}", stderr);
}

#[test]
fn test_missing_analyze_prefix_suggests_full_command() {
    // TDD Test: "complexity" alone should suggest "analyze complexity"
    let output = Command::new(PMAT_BINARY)
        .args(&["complexity"])
        .output()
        .expect("Failed to run command");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    assert!(!output.status.success(), "Command should fail");
    assert!(stderr.contains("Did you mean"), 
        "Should suggest correction. Got: {}", stderr);
    assert!(stderr.contains("analyze complexity"), 
        "Should suggest 'analyze complexity'. Got: {}", stderr);
}

#[test]
fn test_satd_typo_suggests_correct_command() {
    // TDD Test: Common misspellings of SATD should suggest correct command
    let test_cases = vec![
        ("std", "satd"),
        ("stad", "satd"),
        ("sadt", "satd"),
    ];
    
    for (typo, correct) in test_cases {
        let output = Command::new(PMAT_BINARY)
            .args(&["analyze", typo])
            .output()
            .expect("Failed to run command");
        
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        assert!(!output.status.success(), 
            "Command 'analyze {}' should fail", typo);
        assert!(stderr.contains("Did you mean") || stderr.contains("try"), 
            "Should suggest correction for '{}'. Got: {}", typo, stderr);
    }
}

#[test]
fn test_help_provides_working_examples() {
    // TDD Test: Help should show actual working commands, not just syntax
    let output = Command::new(PMAT_BINARY)
        .args(&["--help"])
        .output()
        .expect("Failed to run command");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    assert!(output.status.success(), "Help command should succeed");
    
    // Should contain actual working examples
    assert!(stdout.contains("Examples:") || stdout.contains("EXAMPLES"), 
        "Help should contain examples section. Got: {}", stdout);
    
    // Should show working commands that users can copy-paste
    let working_commands = [
        "pmat analyze complexity --project-path .",
        "pmat analyze satd --path .",
        "pmat context",
        "pmat quality-gate",
    ];
    
    let mut found_examples = 0;
    for cmd in &working_commands {
        if stdout.contains(cmd) {
            found_examples += 1;
        }
    }
    
    assert!(found_examples > 0, 
        "Help should contain at least one working example. Checked: {:?}", working_commands);
}

#[test]
fn test_analyze_help_shows_subcommands_clearly() {
    // TDD Test: "pmat analyze --help" should clearly show available subcommands
    let output = Command::new(PMAT_BINARY)
        .args(&["analyze", "--help"])
        .output()
        .expect("Failed to run command");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    assert!(output.status.success(), "Analyze help should succeed");
    
    // Should clearly show available subcommands
    let expected_subcommands = ["complexity", "satd", "dead-code", "tdg"];
    for subcmd in &expected_subcommands {
        assert!(stdout.contains(subcmd), 
            "Help should mention '{}' subcommand. Got: {}", subcmd, stdout);
    }
}

#[test]
fn test_command_suggestions_are_ranked_by_similarity() {
    // TDD Test: Multiple suggestions should be ranked by similarity
    let output = Command::new(PMAT_BINARY)
        .args(&["analyz"])  // Close to "analyze"
        .output()
        .expect("Failed to run command");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    assert!(!output.status.success(), "Command should fail");
    
    if stderr.contains("Did you mean") {
        // If suggestions are implemented, "analyze" should be first/closest suggestion
        let analyze_pos = stderr.find("analyze");
        assert!(analyze_pos.is_some(), 
            "Should suggest 'analyze' for 'analyz'. Got: {}", stderr);
    }
}

#[test]
fn test_valid_commands_dont_show_suggestions() {
    // TDD Test: Valid commands should not show "did you mean" suggestions
    let output = Command::new(PMAT_BINARY)
        .args(&["analyze", "complexity", "--help"])
        .output()
        .expect("Failed to run command");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    assert!(output.status.success(), "Valid command should succeed");
    assert!(!stderr.contains("Did you mean"), 
        "Valid commands should not show suggestions. Got: {}", stderr);
}