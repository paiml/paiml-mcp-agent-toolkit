//! Exit Code Behavior Example
//!
//! This example demonstrates the exit code behavior of all pmat commands
//! that support --fail-on-violation for CI/CD integration.
//!
//! Run with: `cargo run --example exit_codes`

fn main() {
    println!("🚦 Exit Code Behavior Reference\n");
    println!("All pmat commands that support --fail-on-violation will:");
    println!("  ✅ Exit with code 0 when no violations are found");
    println!("  ❌ Exit with code 1 when violations exceed thresholds\n");

    println!("Commands with Exit Code Support:");
    println!("{}", "=".repeat(60));
    
    println!("\n1. Quality Gate");
    println!("   pmat quality-gate --fail-on-violation");
    println!("   - Comprehensive quality check");
    println!("   - Configurable thresholds for all metrics");
    println!("   - Perfect for CI/CD gate keeping");
    
    println!("\n2. Analyze Complexity");
    println!("   pmat analyze complexity --fail-on-violation");
    println!("   - Fails if any function exceeds complexity thresholds");
    println!("   - Configurable: --max-cyclomatic, --max-cognitive");
    println!("   - Default: cyclomatic=20, cognitive=15");
    
    println!("\n3. Analyze Dead Code");
    println!("   pmat analyze dead-code --fail-on-violation");
    println!("   - Fails if dead code percentage exceeds threshold");
    println!("   - Configurable: --max-percentage");
    println!("   - Default: 15%");
    
    println!("\n4. Analyze SATD");
    println!("   pmat analyze satd --fail-on-violation");
    println!("   - Fails if ANY technical debt is found");
    println!("   - Use --strict for comprehensive detection");
    println!("   - Use --critical-only to only fail on critical debt");
    
    println!("\nTesting Exit Codes:");
    println!("{}", "=".repeat(60));
    
    println!("\n# Test in bash/shell:");
    println!("pmat analyze complexity --fail-on-violation");
    println!("echo \"Exit code: $?\"  # Will show 0 or 1");
    
    println!("\n# Chain commands (stop on first failure):");
    println!("pmat analyze complexity --fail-on-violation && \\");
    println!("pmat analyze satd --strict --fail-on-violation && \\");
    println!("pmat analyze dead-code --max-percentage 10 --fail-on-violation && \\");
    println!("echo \"✅ All quality checks passed!\"");
    
    println!("\n# Continue despite failures:");
    println!("set +e  # Don't exit on error");
    println!("pmat analyze complexity --fail-on-violation");
    println!("COMPLEXITY_RESULT=$?");
    println!("pmat analyze satd --fail-on-violation");
    println!("SATD_RESULT=$?");
    println!("if [ $COMPLEXITY_RESULT -ne 0 ] || [ $SATD_RESULT -ne 0 ]; then");
    println!("    echo \"Quality issues found!\"");
    println!("    exit 1");
    println!("fi");
    
    println!("\nGitHub Actions Matrix Example:");
    println!("{}", "=".repeat(60));
    println!("```yaml");
    println!("strategy:");
    println!("  matrix:");
    println!("    check:");
    println!("      - name: complexity");
    println!("        cmd: analyze complexity --max-cyclomatic 15 --fail-on-violation");
    println!("      - name: dead-code");
    println!("        cmd: analyze dead-code --max-percentage 10 --fail-on-violation");
    println!("      - name: tech-debt");
    println!("        cmd: analyze satd --strict --fail-on-violation");
    println!("steps:");
    println!("  - name: Run ${{{{ matrix.check.name }}}} check");
    println!("    run: pmat ${{{{ matrix.check.cmd }}}}");
    println!("```");
    
    println!("\n🎯 Best Practices:");
    println!("{}", "=".repeat(60));
    println!("1. Set appropriate thresholds for your project maturity");
    println!("2. Start with lenient thresholds and gradually tighten");
    println!("3. Use JSON output format for parsing in CI scripts");
    println!("4. Consider different thresholds for different branches");
    println!("5. Run quality-gate first for comprehensive check");
    println!("6. Use specific analyze commands for granular control");
}