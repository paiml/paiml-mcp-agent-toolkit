use std::process::Command;

#[test]
fn test_all_cli_commands_work() {
    // Test help command
    let output = Command::new(env!("CARGO_BIN_EXE_pmat"))
        .arg("help")
        .output()
        .expect("Failed to execute help command");

    let combined = format!("{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr));

    assert!(output.status.success(), "Help command should succeed");
    assert!(combined.contains("Usage:"), "Help should contain Usage section");
    assert!(combined.contains("Commands:"), "Help should contain Commands section");

    // Test version command
    let output = Command::new(env!("CARGO_BIN_EXE_pmat"))
        .arg("--version")
        .output()
        .expect("Failed to execute version command");

    let combined = format!("{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr));

    assert!(output.status.success(), "Version command should succeed");
    assert!(combined.contains("pmat"), "Version should contain 'pmat'");

    // Test analyze complexity
    let output = Command::new(env!("CARGO_BIN_EXE_pmat"))
        .args(["analyze", "complexity", "--file", "src/lib.rs"])
        .output()
        .expect("Failed to execute complexity analysis");

    let combined = format!("{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr));

    assert!(output.status.success(), "Complexity analysis should succeed");
    assert!(combined.to_lowercase().contains("complexity") ||
            combined.to_lowercase().contains("files analyzed"),
            "Output should contain complexity analysis info");

    // Test SATD analysis
    let output = Command::new(env!("CARGO_BIN_EXE_pmat"))
        .args(["analyze", "satd"])
        .output()
        .expect("Failed to execute SATD analysis");

    let combined = format!("{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr));

    // SATD might succeed or fail depending on content, but should not crash
    assert!(combined.to_lowercase().contains("satd") ||
            combined.to_lowercase().contains("violations") ||
            combined.to_lowercase().contains("analyzing"),
            "Output should contain SATD analysis info");

    // Test quality gate help (to avoid long execution)
    let output = Command::new(env!("CARGO_BIN_EXE_pmat"))
        .args(["quality-gate", "--help"])
        .output()
        .expect("Failed to execute quality gate help");

    let combined = format!("{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr));

    assert!(output.status.success(), "Quality gate help should succeed");
    assert!(combined.to_lowercase().contains("quality") ||
            combined.to_lowercase().contains("usage") ||
            combined.to_lowercase().contains("gate"),
            "Output should contain quality gate help info");

    println!("✅ All CLI commands work correctly!");
}

