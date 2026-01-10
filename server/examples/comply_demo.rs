//! PMAT Compliance Checking Example
//!
//! This example demonstrates how to use pmat's comply command for
//! verifying PMAT configuration compliance across projects.
//!
//! Run with: `cargo run --example comply_demo`
//!
//! # Features Demonstrated
//!
//! 1. Version currency checking
//! 2. Configuration file validation
//! 3. Pre-commit hooks verification
//! 4. Quality threshold checks
//! 5. Build performance compliance (Cargo.lock, MSRV, CI)
//!
//! # Compliance Checks (9 total)
//!
//! - Version Currency     : Is project using current PMAT version?
//! - Config Files         : Are .pmat/project.toml and .pmat-metrics.toml present?
//! - Hooks Installed      : Is pre-commit hook configured?
//! - Quality Thresholds   : Are quality thresholds defined?
//! - Deprecated Features  : Are any deprecated features in use?
//! - ComputeBrick         : (If applicable) Is GPU/SIMD config valid?
//! - Cargo.lock Present   : Is Cargo.lock committed for reproducible builds?
//! - MSRV Defined         : Is rust-version specified in Cargo.toml?
//! - CI Configured        : Is CI/CD pipeline configured?
//!
//! # CLI Usage
//!
//! ```bash
//! # Check compliance of current project
//! pmat comply check
//!
//! # Generate compliance report
//! pmat comply report
//!
//! # Output formats
//! pmat comply check --format text     # Human-readable (default)
//! pmat comply check --format json     # For CI/CD
//! pmat comply check --format markdown # For documentation
//!
//! # Migrate to newer PMAT version
//! pmat comply migrate --version 2.213.4
//!
//! # Show breaking changes between versions
//! pmat comply breaking-changes --from 2.200.0 --to 2.213.4
//! ```

fn main() {
    println!("PMAT Compliance Checking Demo");
    println!("{}", "=".repeat(60));

    // Example 1: Overview of compliance checks
    println!("\nExample 1: Compliance Check Categories");
    println!("{}", "-".repeat(40));
    demonstrate_checks();

    // Example 2: Check status interpretation
    println!("\nExample 2: Check Status Interpretation");
    println!("{}", "-".repeat(40));
    demonstrate_status();

    // Example 3: CI/CD integration
    println!("\nExample 3: CI/CD Integration");
    println!("{}", "-".repeat(40));
    demonstrate_ci_integration();

    // Example 4: Migration workflow
    println!("\nExample 4: Migration Workflow");
    println!("{}", "-".repeat(40));
    demonstrate_migration();

    // Example 5: Breaking changes tracking
    println!("\nExample 5: Breaking Changes Tracking");
    println!("{}", "-".repeat(40));
    demonstrate_breaking_changes();

    println!("\n{}", "=".repeat(60));
    println!("Compliance demo completed!");
}

/// Demonstrate the 9 compliance checks
fn demonstrate_checks() {
    println!(
        "
The comply command runs 9 compliance checks:

## Core Checks (6)

  1. Version Currency
     - Checks if project is using current PMAT version
     - Warns if >5 versions behind
     - Fails if significantly outdated

  2. Config Files
     - Validates .pmat/project.toml exists
     - Validates .pmat-metrics.toml exists
     - Warns if either is missing

  3. Hooks Installed
     - Checks .git/hooks/pre-commit exists
     - Validates it contains PMAT integration
     - Warns if hooks not configured

  4. Quality Thresholds
     - Validates .pmat-metrics.toml configuration
     - Checks thresholds are properly defined
     - Uses defaults if missing

  5. Deprecated Features
     - Scans for deprecated PMAT features
     - Warns about migration needs
     - Currently passes (no deprecated features)

  6. ComputeBrick Compliance (if applicable)
     - Checks for probar dependency or brick/ directory
     - Validates [compute-brick] in .pmat-gates.toml
     - Checks GUI coverage report

## Build Performance Checks (3)

  7. Cargo.lock Present
     - Ensures Cargo.lock is committed
     - Critical for reproducible builds
     - Fails if missing

  8. MSRV Defined
     - Checks for rust-version in Cargo.toml
     - Important for compatibility
     - Warns if missing

  9. CI Configured
     - Checks for .github/workflows/
     - Also checks .gitlab-ci.yml and Jenkinsfile
     - Warns if no CI found
"
    );
}

/// Demonstrate check status interpretation
fn demonstrate_status() {
    println!(
        "
Check Status Levels:

  PASS : Check passed successfully
  WARN : Check has non-critical issues
  FAIL : Check failed with critical issues
  SKIP : Check not applicable to this project

Severity Levels:

  Info     : Informational, no action required
  Warning  : Should be addressed when convenient
  Error    : Must be addressed before release
  Critical : Stop the line, address immediately

Example Output:

  $ pmat comply check

  PMAT Compliance Check
  ================================================

  Version: v2.213.4 (current)
  Checks: 8/9 passed

  [PASS] Version Currency - Project is on latest version (v2.213.4)
  [PASS] Config Files - All configuration files present
  [WARN] Hooks Installed - Pre-commit hook not configured
  [PASS] Quality Thresholds - Thresholds configured
  [PASS] Deprecated Features - No deprecated features detected
  [SKIP] ComputeBrick Compliance - Not a ComputeBrick project
  [PASS] Cargo.lock Present - Reproducible builds enabled
  [WARN] MSRV Defined - No rust-version field
  [PASS] CI Configured - 31 GitHub Actions workflows

  Status: COMPLIANT (with 2 warnings)
"
    );
}

/// Demonstrate CI/CD integration
fn demonstrate_ci_integration() {
    println!(
        "
CI/CD Integration Examples:

## GitHub Actions Workflow

```yaml
name: PMAT Compliance
on: [push, pull_request]

jobs:
  compliance:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Check Compliance
        run: |
          pmat comply check --format json > compliance.json

          # Check if compliant
          COMPLIANT=$(jq '.is_compliant' compliance.json)
          if [ \"$COMPLIANT\" != \"true\" ]; then
            echo \"Project is not compliant\"
            jq '.checks[] | select(.status == \"Fail\")' compliance.json
            exit 1
          fi

      - name: Upload Compliance Report
        uses: actions/upload-artifact@v4
        with:
          name: compliance-report
          path: compliance.json
```

## Pre-push Hook

```bash
#!/bin/bash
# .git/hooks/pre-push

# Check compliance before pushing
pmat comply check --quiet
if [ $? -ne 0 ]; then
    echo \"Compliance check failed. Fix issues before pushing.\"
    exit 1
fi
```

## Makefile Integration

```makefile
.PHONY: comply
comply:
    pmat comply check
    @echo \"Compliance check passed!\"

.PHONY: comply-report
comply-report:
    pmat comply report --format markdown > COMPLIANCE.md
```
"
    );
}

/// Demonstrate migration workflow
fn demonstrate_migration() {
    println!(
        "
Migration Workflow:

When upgrading PMAT versions, use the migrate command:

  # Check current version
  pmat --version

  # See what needs to change for upgrade
  pmat comply migrate --version 2.214.0 --dry-run

  # Perform migration
  pmat comply migrate --version 2.214.0

Migration Process:

  1. Version Analysis
     - Compares current vs target version
     - Identifies breaking changes
     - Lists required updates

  2. Configuration Updates
     - Updates .pmat/project.toml version
     - Migrates deprecated configuration
     - Updates quality thresholds if needed

  3. Hook Updates
     - Updates pre-commit hooks
     - Ensures new checks are included
     - Maintains custom hooks

  4. Validation
     - Runs full compliance check
     - Verifies all checks pass
     - Reports any remaining issues

Example:

  $ pmat comply migrate --version 2.214.0

  PMAT Migration: 2.213.4 -> 2.214.0
  ================================================

  [INFO] Analyzing changes...
  [INFO] Found 0 breaking changes
  [INFO] Found 2 new features

  Applying changes:
    [OK] Updated .pmat/project.toml version
    [OK] Added new quality threshold: build_perf
    [OK] Updated pre-commit hook

  Migration completed successfully!
"
    );
}

/// Demonstrate breaking changes tracking
fn demonstrate_breaking_changes() {
    println!(
        "
Breaking Changes Tracking:

Track breaking changes between versions:

  $ pmat comply breaking-changes --from 2.200.0 --to 2.213.4

  Breaking Changes: v2.200.0 -> v2.213.4
  ================================================

  ## v2.210.0 Breaking Changes
  - Removed deprecated `--legacy-output` flag
  - Changed default output format to JSON

  ## v2.212.0 Breaking Changes
  - Renamed `analyze dead-code` to `analyze unused`
  - Updated quality gate threshold format

  ## v2.213.0 Breaking Changes
  - None

  Migration Guide:
  1. Update any scripts using --legacy-output
  2. Change 'dead-code' to 'unused' in CI/CD
  3. Update .pmat-metrics.toml format

This helps teams:
  - Plan upgrade windows
  - Update CI/CD pipelines
  - Communicate changes to developers
"
    );
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_example_runs() {
        super::main();
    }
}
