//! CLI Smoke Test - Verify all basic commands work
//! 
//! This is CRITICAL - if these commands don't work, the project is unusable

use std::process::Command;

fn run_command(args: &[&str]) -> (bool, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_pmat"))
        .args(args)
        .output()
        .expect("Failed to run command");
    
    let success = output.status.success();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    
    (success, stdout, stderr)
}

#[test]
fn test_critical_commands_work() {
    let mut failures = Vec::new();
    
    // Test cases: (args, description, should_succeed, output_must_contain)
    let tests = vec![
        (vec!["--help"], "Help command", true, Some("Commands:")),
        (vec!["--version"], "Version command", true, Some("pmat")),
        (vec!["analyze", "--help"], "Analyze help", true, Some("Commands:")),
        (vec!["analyze", "complexity", "--help"], "Complexity help", true, Some("Options:")),
        (vec!["analyze", "complexity", "--project-path", "."], "Complexity analysis with path", true, Some("Files analyzed")),
        (vec!["analyze", "satd", "--path", "."], "SATD analysis", true, Some("SATD")),
        (vec!["analyze", "dead-code", "--path", "."], "Dead code analysis", true, Some("Dead")),
        (vec!["quality-gate", "--help"], "Quality gate help", true, Some("Options:")),
        (vec!["context", "--help"], "Context help", true, Some("Options:")),
        (vec!["demo", "--help"], "Demo help", true, Some("Options:")),
    ];
    
    println!("\n=== CLI SMOKE TEST ===\n");
    
    let test_count = tests.len();
    
    for (args, desc, should_succeed, must_contain) in tests {
        let cmd_str = format!("pmat {}", args.join(" "));
        print!("Testing: {} ... ", cmd_str);
        
        let (success, stdout, stderr) = run_command(&args);
        
        let mut test_passed = true;
        let mut error_msg = String::new();
        
        if should_succeed && !success {
            test_passed = false;
            error_msg.push_str(&format!("Command failed with stderr: {}", stderr));
        }
        
        if let Some(expected) = must_contain {
            if !stdout.contains(expected) && !stderr.contains(expected) {
                test_passed = false;
                error_msg.push_str(&format!("Output missing '{}'. Got: {}", expected, 
                    if stdout.len() > 100 { &stdout[..100] } else { &stdout }));
            }
        }
        
        if test_passed {
            println!("✅ PASS");
        } else {
            println!("❌ FAIL");
            failures.push((desc, cmd_str, error_msg));
        }
    }
    
    println!("\n=== RESULTS ===");
    
    if failures.is_empty() {
        println!("✅ All {} tests passed!", test_count);
    } else {
        println!("❌ {} of {} tests FAILED:", failures.len(), test_count);
        for (desc, cmd, error) in &failures {
            println!("\n  {} ({})", desc, cmd);
            println!("    Error: {}", error);
        }
        panic!("\n\n🚨 CRITICAL: {} CLI commands are broken! The project is unusable.", failures.len());
    }
}

#[test]
fn test_common_mistakes_give_helpful_errors() {
    // Test that common mistakes give helpful error messages
    let tests = vec![
        (vec!["agent", "analyze"], "Common mistake: agent analyze"),
        (vec!["analize"], "Typo: analize instead of analyze"),
        (vec!["complexity"], "Missing analyze prefix"),
    ];
    
    println!("\n=== ERROR MESSAGE QUALITY TEST ===\n");
    
    for (args, desc) in tests {
        let cmd_str = format!("pmat {}", args.join(" "));
        println!("Testing error for: {}", cmd_str);
        
        let (success, _stdout, stderr) = run_command(&args);
        
        assert!(!success, "{} should fail", desc);
        
        // Check if error is helpful
        if stderr.contains("unrecognized subcommand") || stderr.contains("error") {
            println!("  Current error: {}", stderr.lines().next().unwrap_or(""));
            
            // TODO: These should give suggestions like "Did you mean 'pmat analyze'?"
            // For now, we just document that the errors are unhelpful
            if !stderr.contains("Did you mean") && !stderr.contains("try") {
                println!("  ⚠️  Error message could be more helpful");
            }
        }
    }
}