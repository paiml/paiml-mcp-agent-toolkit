//! CI/CD Integration Example
//!
//! This example demonstrates how to integrate pmat's quality checks
//! into your CI/CD pipeline with proper exit codes and thresholds.
//!
//! Run with: `cargo run --example ci_integration`

use anyhow::Result;

fn main() -> Result<()> {
    println!("🚀 CI/CD Integration Example\n");
    println!("This example shows how to use pmat in CI/CD pipelines.\n");

    // Example 1: GitHub Actions workflow
    println!("Example 1: GitHub Actions Workflow");
    println!("{}", "=".repeat(60));
    println!("```yaml");
    println!("name: Code Quality Check");
    println!("on: [push, pull_request]");
    println!();
    println!("jobs:");
    println!("  quality:");
    println!("    runs-on: ubuntu-latest");
    println!("    steps:");
    println!("      - uses: actions/checkout@v3");
    println!("      ");
    println!("      - name: Install pmat");
    println!("        run: cargo install pmat");
    println!("      ");
    println!("      - name: Run Quality Gate");
    println!("        run: |");
    println!("          pmat quality-gate \\");
    println!("            --fail-on-violation \\");
    println!("            --max-dead-code 10.0 \\");
    println!("            --max-cyclomatic 20 \\");
    println!("            --format json");
    println!("      ");
    println!("      - name: Check Complexity");
    println!("        run: |");
    println!("          pmat analyze complexity \\");
    println!("            --max-cyclomatic 15 \\");
    println!("            --max-cognitive 10 \\");
    println!("            --fail-on-violation \\");
    println!("            --format json");
    println!("      ");
    println!("      - name: Check Technical Debt");
    println!("        run: |");
    println!("          pmat analyze satd \\");
    println!("            --strict \\");
    println!("            --fail-on-violation \\");
    println!("            --format json");
    println!("      ");
    println!("      - name: Check Dead Code");
    println!("        run: |");
    println!("          pmat analyze dead-code \\");
    println!("            --max-percentage 5.0 \\");
    println!("            --fail-on-violation \\");
    println!("            --include-unreachable");
    println!("```\n");

    // Example 2: GitLab CI/CD
    println!("Example 2: GitLab CI/CD Pipeline");
    println!("{}", "=".repeat(60));
    println!("```yaml");
    println!("quality-check:");
    println!("  stage: test");
    println!("  script:");
    println!("    - cargo install pmat");
    println!("    - |");
    println!("      pmat quality-gate \\");
    println!("        --fail-on-violation \\");
    println!("        --format json \\");
    println!("        --output quality-report.json");
    println!("  artifacts:");
    println!("    reports:");
    println!("      codequality: quality-report.json");
    println!("  allow_failure: false");
    println!("```\n");

    // Example 3: Jenkins Pipeline
    println!("Example 3: Jenkins Pipeline");
    println!("{}", "=".repeat(60));
    println!("```groovy");
    println!("pipeline {{");
    println!("    agent any");
    println!("    ");
    println!("    stages {{");
    println!("        stage('Quality Check') {{");
    println!("            steps {{");
    println!("                sh 'cargo install pmat'");
    println!("                ");
    println!("                // Will fail the build if violations found");
    println!("                sh '''");
    println!("                    pmat analyze complexity \\");
    println!("                        --max-cyclomatic 20 \\");
    println!("                        --fail-on-violation \\");
    println!("                        --format json > complexity.json");
    println!("                '''");
    println!("                ");
    println!("                sh '''");
    println!("                    pmat analyze satd \\");
    println!("                        --critical-only \\");
    println!("                        --fail-on-violation");
    println!("                '''");
    println!("            }}");
    println!("        }}");
    println!("    }}");
    println!("    ");
    println!("    post {{");
    println!("        always {{");
    println!("            archiveArtifacts artifacts: '*.json'");
    println!("        }}");
    println!("    }}");
    println!("}}");
    println!("```\n");

    // Example 4: Pre-commit hook
    println!("Example 4: Git Pre-commit Hook");
    println!("{}", "=".repeat(60));
    println!("Create .git/hooks/pre-commit:");
    println!("```bash");
    println!("#!/bin/bash");
    println!("# Pre-commit hook to enforce code quality");
    println!();
    println!("echo \"Running quality checks...\"");
    println!();
    println!("# Check for technical debt in staged files");
    println!("pmat analyze satd --strict --fail-on-violation");
    println!("if [ $? -ne 0 ]; then");
    println!("    echo \"❌ Technical debt found. Please fix before committing.\"");
    println!("    exit 1");
    println!("fi");
    println!();
    println!("# Check complexity");
    println!("pmat analyze complexity \\");
    println!("    --max-cyclomatic 15 \\");
    println!("    --max-cognitive 10 \\");
    println!("    --fail-on-violation");
    println!("if [ $? -ne 0 ]; then");
    println!("    echo \"❌ Complexity too high. Please refactor before committing.\"");
    println!("    exit 1");
    println!("fi");
    println!();
    println!("echo \"✅ All quality checks passed!\"");
    println!("```\n");

    // Example 5: Makefile integration
    println!("Example 5: Makefile Integration");
    println!("{}", "=".repeat(60));
    println!("```makefile");
    println!("# Makefile targets for quality checks");
    println!();
    println!(".PHONY: quality-check quality-strict ci-check");
    println!();
    println!("# Standard quality check");
    println!("quality-check:");
    println!("\t@echo \"Running quality checks...\"");
    println!("\tpmat quality-gate --fail-on-violation");
    println!();
    println!("# Strict mode for CI");
    println!("quality-strict:");
    println!("\t@echo \"Running strict quality checks...\"");
    println!("\tpmat analyze complexity --max-cyclomatic 10 --fail-on-violation");
    println!("\tpmat analyze satd --strict --fail-on-violation");
    println!("\tpmat analyze dead-code --max-percentage 5.0 --fail-on-violation");
    println!();
    println!("# Combined CI check");
    println!("ci-check: test quality-strict");
    println!("\t@echo \"✅ All CI checks passed!\"");
    println!("```\n");

    // Example 6: Docker integration
    println!("Example 6: Docker Multi-stage Build");
    println!("{}", "=".repeat(60));
    println!("```dockerfile");
    println!("# Dockerfile with quality gates");
    println!("FROM rust:latest as quality-check");
    println!();
    println!("WORKDIR /app");
    println!("COPY . .");
    println!();
    println!("# Install pmat");
    println!("RUN cargo install pmat");
    println!();
    println!("# Run quality checks (build will fail if violations found)");
    println!("RUN pmat quality-gate --fail-on-violation");
    println!("RUN pmat analyze satd --strict --fail-on-violation");
    println!();
    println!("# Continue with build only if quality checks pass");
    println!("FROM rust:latest as builder");
    println!("WORKDIR /app");
    println!("COPY --from=quality-check /app .");
    println!("RUN cargo build --release");
    println!("```\n");

    // Example 7: Custom thresholds configuration
    println!("Example 7: Environment-specific Thresholds");
    println!("{}", "=".repeat(60));
    println!("```bash");
    println!("#!/bin/bash");
    println!("# ci-quality-check.sh");
    println!();
    println!("# Set thresholds based on environment");
    println!("if [ \"$CI_ENVIRONMENT\" = \"production\" ]; then");
    println!("    MAX_COMPLEXITY=10");
    println!("    MAX_DEAD_CODE=5.0");
    println!("    SATD_MODE=\"--strict\"");
    println!("else");
    println!("    MAX_COMPLEXITY=20");
    println!("    MAX_DEAD_CODE=15.0");
    println!("    SATD_MODE=\"--critical-only\"");
    println!("fi");
    println!();
    println!("# Run checks with environment-specific thresholds");
    println!("pmat analyze complexity \\");
    println!("    --max-cyclomatic $MAX_COMPLEXITY \\");
    println!("    --fail-on-violation");
    println!();
    println!("pmat analyze dead-code \\");
    println!("    --max-percentage $MAX_DEAD_CODE \\");
    println!("    --fail-on-violation");
    println!();
    println!("pmat analyze satd $SATD_MODE --fail-on-violation");
    println!("```");

    println!("\n🎉 CI/CD integration examples completed!");
    println!("\nKey points:");
    println!("- All analyze commands support --fail-on-violation for CI/CD");
    println!("- Exit code 0 means success, 1 means violations found");
    println!("- Use JSON format for parsing results in CI pipelines");
    println!("- Set appropriate thresholds for your project");

    Ok(())
}
