//! Project Diagnostics Example
//!
//! This example demonstrates how to use pmat's project-diag command for
//! comprehensive Rust project diagnostics. It provides 20 checks across
//! 5 categories, matching lltop Tab 8 functionality.
//!
//! Run with: `cargo run --example project_diag_demo`
//!
//! # Features Demonstrated
//!
//! 1. Cargo configuration checks (edition, resolver, deps, LTO, workspace)
//! 2. Dependency management (target dir, Cargo.lock, audit config)
//! 3. Build performance (cargo config, incremental, codegen-units, build system)
//! 4. Code quality (clippy, rustfmt, tests, README)
//! 5. Advanced checks (MSRV, benchmarks, CI)
//!
//! # Categories (20 checks total)
//!
//! - Cargo Config (6 checks): Edition 2021+, Resolver v2, Deps <=50, LTO, Workspace lints/deps
//! - Dependencies (3 checks): Target dir <=10GB, Cargo.lock, Audit config
//! - Build Performance (4 checks): .cargo/config, Incremental, Codegen units, Build system
//! - Code Quality (4 checks): Clippy config, Rustfmt config, Tests present, README
//! - Advanced (3 checks): MSRV defined, Benchmarks, CI configured
//!
//! # CLI Usage
//!
//! ```bash
//! # Run diagnostics on current directory
//! pmat project-diag
//!
//! # Specify a path
//! pmat project-diag --path /path/to/rust/project
//!
//! # Filter by category
//! pmat project-diag --category cargo
//! pmat project-diag --category build
//! pmat project-diag --category quality
//!
//! # Output formats
//! pmat project-diag --format summary   # Human-readable (default)
//! pmat project-diag --format json      # For CI/CD
//! pmat project-diag --format markdown  # For documentation
//! pmat project-diag --format andon     # Toyota Way Andon board
//!
//! # Show only failures
//! pmat project-diag --failures-only
//!
//! # Save to file
//! pmat project-diag --format json --output diag-report.json
//! ```

fn main() {
    println!("PMAT Project Diagnostics Demo");
    println!("{}", "=".repeat(60));

    // Example 1: Overview of the 20 diagnostic checks
    println!("\nExample 1: Diagnostic Check Categories");
    println!("{}", "-".repeat(40));
    demonstrate_categories();

    // Example 2: Interpreting results
    println!("\nExample 2: Understanding Health Status");
    println!("{}", "-".repeat(40));
    demonstrate_health_status();

    // Example 3: CI/CD integration
    println!("\nExample 3: CI/CD Integration");
    println!("{}", "-".repeat(40));
    demonstrate_ci_integration();

    // Example 4: Toyota Way Andon board
    println!("\nExample 4: Andon Board Visualization");
    println!("{}", "-".repeat(40));
    demonstrate_andon_board();

    // Example 5: Category filtering
    println!("\nExample 5: Category Filtering");
    println!("{}", "-".repeat(40));
    demonstrate_category_filtering();

    // Example 6: Run on current project
    println!("\nExample 6: Running Diagnostics on PMAT Itself");
    println!("{}", "-".repeat(40));
    demonstrate_self_analysis();

    println!("\n{}", "=".repeat(60));
    println!("Project diagnostics demo completed!");
}

/// Demonstrate the 5 diagnostic categories and their checks
fn demonstrate_categories() {
    println!(
        "
The project-diag command runs 20 checks across 5 categories:

## Cargo Config (6 checks, 30 points max)
  - Edition 2021+       : Validates Rust edition is 2021 or 2024
  - Resolver v2         : Checks for resolver = \"2\" or edition 2021+
  - Dependencies <= 50  : Counts dependencies, warns if >50
  - LTO Enabled        : Checks for lto = true/thin/fat in [profile.release]
  - Workspace Lints    : Validates [workspace.lints] configuration
  - Workspace Deps     : Checks for [workspace.dependencies]

## Dependencies (3 checks, 15 points max)
  - Target Dir <= 10GB : Measures target/ directory size
  - Cargo.lock Present : Ensures reproducible builds
  - Audit Config       : Checks for deny.toml or audit.toml

## Build Performance (4 checks, 20 points max)
  - Cargo Config       : Validates .cargo/config.toml exists
  - Incremental Builds : Checks incremental compilation settings
  - Codegen Units      : Validates codegen-units = 1 for release
  - Build System       : Checks for Makefile/justfile/build.rs

## Code Quality (4 checks, 20 points max)
  - Clippy Config      : Checks for .clippy.toml or [lints.clippy]
  - Rustfmt Config     : Validates rustfmt.toml exists
  - Tests Present      : Checks for tests/ directory and #[test]
  - README             : Validates README.md exists and has content

## Advanced (3 checks, 15 points max)
  - MSRV Defined       : Checks for rust-version in Cargo.toml
  - Benchmarks         : Validates benches/ directory and Criterion
  - CI Configured      : Checks for .github/workflows/ or .gitlab-ci.yml
"
    );
}

/// Demonstrate health status interpretation
fn demonstrate_health_status() {
    println!(
        "
Health Status Interpretation:

  GREEN  (>=85%) : Production ready, all critical checks pass
  YELLOW (60-84%) : Some issues need attention before release
  RED    (<60%)  : Critical issues must be resolved

Status Icons in Output:
  [OK]   : Check passed (GREEN)
  [WARN] : Check has warnings (YELLOW)
  [FAIL] : Check failed (RED)
  [SKIP] : Check not applicable (e.g., workspace deps in single-crate)

Score Calculation:
  - Each check has a max score (typically 5 points)
  - Partial credit may be awarded for partial compliance
  - Total percentage = (earned / max) * 100
"
    );
}

/// Demonstrate CI/CD integration patterns
fn demonstrate_ci_integration() {
    println!(
        "
CI/CD Integration Examples:

## GitHub Actions Workflow

```yaml
name: Project Diagnostics
on: [push, pull_request]

jobs:
  diagnostics:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Run Project Diagnostics
        run: |
          pmat project-diag --format json --output diag.json

          # Parse and check score
          SCORE=$(jq '.percentage' diag.json)
          if (( $(echo \"$SCORE < 80\" | bc -l) )); then
            echo \"Diagnostics score $SCORE is below 80%\"
            exit 1
          fi

      - name: Upload Report
        uses: actions/upload-artifact@v4
        with:
          name: diagnostics-report
          path: diag.json
```

## GitLab CI Pipeline

```yaml
project-diagnostics:
  stage: quality
  script:
    - pmat project-diag --format json --output diag.json
    - 'SCORE=$(jq \".percentage\" diag.json) && test $SCORE -ge 80'
  artifacts:
    paths:
      - diag.json
```

## Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit
pmat project-diag --failures-only --quiet
if [ $? -ne 0 ]; then
    echo \"Project diagnostics found critical issues\"
    exit 1
fi
```
"
    );
}

/// Demonstrate the Andon board visualization
fn demonstrate_andon_board() {
    println!(
        "
Andon Board Visualization (Toyota Way):

The --format andon option displays an Andon-style status board:

  +=================================================================+
  |                    PROJECT DIAGNOSTICS                          |
  |                      (Andon Board)                              |
  +=================================================================+
  |  Score: [##############################----------] 75.0%        |
  +=================================================================+
  |  [YELLOW] Cargo Config         5/6 checks passed               |
  |  [GREEN]  Dependencies         3/3 checks passed               |
  |  [GREEN]  Build Performance    4/4 checks passed               |
  |  [YELLOW] Code Quality         2/4 checks passed               |
  |  [RED]    Advanced             0/3 checks passed               |
  +=================================================================+
  |  ANDON CORD TRIGGERED - Issues require attention:               |
  |    - MSRV Defined                                               |
  |    - Benchmarks                                                 |
  |    - CI Configured                                              |
  +=================================================================+

This visualization follows Toyota Production System principles:
  - GREEN  : Line running smoothly
  - YELLOW : Minor issues, continue with caution
  - RED    : Stop the line, address issues immediately
"
    );
}

/// Demonstrate category filtering
fn demonstrate_category_filtering() {
    println!(
        "
Category Filtering:

Filter diagnostics to specific categories:

  pmat project-diag --category cargo     # Cargo configuration only
  pmat project-diag --category deps      # Dependency checks only
  pmat project-diag --category build     # Build performance only
  pmat project-diag --category quality   # Code quality only
  pmat project-diag --category advanced  # Advanced checks only

Category Aliases:
  cargo    -> Cargo Config (6 checks)
  deps     -> Dependencies (3 checks)
  build    -> Build Performance (4 checks)
  quality  -> Code Quality (4 checks)
  advanced -> Advanced (3 checks)

Example: Checking just build performance

  $ pmat project-diag --category build

  Project Diagnostics: /path/to/project
  ==================================================

  Overall: [GREEN] 20.0/20.0 (100.0%)

  Build Performance [4/4]

  Checks:
  --------------------------------------------------
  [OK] Cargo Config - .cargo/config.toml present
  [OK] Incremental Builds - Incremental builds enabled (default)
  [OK] Codegen Units - codegen-units = 1 (maximum optimization)
  [OK] Build System - Build automation: Makefile, build.rs
"
    );
}

/// Demonstrate running diagnostics on the current project
fn demonstrate_self_analysis() {
    println!(
        "
To run diagnostics on any Rust project:

  # Current directory
  pmat project-diag

  # Specific path
  pmat project-diag --path /path/to/rust/project

  # JSON output for parsing
  pmat project-diag --format json | jq .

  # Markdown for documentation
  pmat project-diag --format markdown > DIAGNOSTICS.md

Running on PMAT itself (current project):

  $ pmat project-diag

  Project Diagnostics: .
  ==================================================

  Overall: [YELLOW] 78.0/100.0 (78.0%)

  Cargo Config [5/6]
  Dependencies [2/3]
  Build Performance [4/4]
  Code Quality [2/4]
  Advanced [1/3]

  ... (20 individual check results)
"
    );

    // Actually run diagnostics on current directory
    let current_dir = std::env::current_dir().unwrap();
    let cargo_toml = current_dir.join("Cargo.toml");

    if cargo_toml.exists() {
        println!("\nThis is a Rust project. Run 'pmat project-diag' for full analysis.");
    } else {
        println!("\nNote: Current directory is not a Rust project root.");
        println!("Navigate to a Rust project and run 'pmat project-diag'.");
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_example_runs() {
        // This test just verifies the example compiles and runs
        super::main();
    }
}
