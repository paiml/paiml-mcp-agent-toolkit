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
//! 6. CB-081 Dependency Health (duplicates, feature flags, sovereign stack)
//! 7. CB-600 Lua Best Practices (implicit globals, nil safety, pcall, dangerous APIs)
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

    // Example 6: CB-081 Dependency Health
    println!("\nExample 6: CB-081 Dependency Health");
    println!("{}", "-".repeat(40));
    demonstrate_dependency_health();

    // Example 7: CB-600 Lua Best Practices
    println!("\nExample 7: CB-600 Lua Best Practices");
    println!("{}", "-".repeat(40));
    demonstrate_lua_best_practices();

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

/// Demonstrate CB-081 Dependency Health checks
fn demonstrate_dependency_health() {
    println!(
        "
CB-081: Dependency Health Analysis

The CB-081 check analyzes Cargo.toml and Cargo.lock for dependency
health, providing a score from 0-5 points plus sovereign stack bonus.

## Scoring Tiers (5 points max)

  Score | Direct Deps | Transitive Deps
  ------+-------------+-----------------
    5   |    ≤20      |      ≤100
    3   |    ≤30      |      ≤150
    2   |    ≤40      |      ≤200
    1   |    ≤50      |      ≤250
    0   |    >50      |      >250

## Enhanced Checks

  CB-081-A: Base dependency count scoring
  CB-081-B: Duplicate crate detection (cargo tree --duplicates)
  CB-081-C: Feature flag hygiene (default-features = false usage)
  CB-081-D: Sovereign stack bonus (+1 per batuta crate)
  CB-081-E: Trend tracking (delta since last check)

## Sovereign Stack Crates (Batuta Ecosystem)

  aprender, trueno, trueno-graph, trueno-db, trueno-rag,
  trueno-viz, trueno-zram-core, pmcp, presentar-core,
  renacer, certeza, bashrs, probar, ruchy

## Example Output

  $ pmat comply check

  CB-081: Dependency Health: Score: 3/5 | 25 direct, 120 transitive
    | 5 duplicates | 45% feature-gated | +2 sovereign (aprender, trueno)
    ⚠ 5 duplicate crates: rand, syn, quote, hashbrown, itertools
    ℹ Trend: +2 direct, -5 transitive since last check

## Reducing Dependencies

  1. Use `default-features = false` for all deps
  2. Run `cargo tree --duplicates` to find duplicates
  3. Prefer batuta stack (sovereign) crates
  4. Consolidate duplicate versions
  5. Use feature flags to disable unused components

## CI/CD Integration

  # Fail if dependency score < 3
  pmat comply check --format json | jq '.cb081.score >= 3' | grep true
"
    );
}

/// Demonstrate CB-600 Lua Best Practices checks
fn demonstrate_lua_best_practices() {
    println!(
        "
CB-600: Lua Best Practices (CB-600 to CB-607)

The CB-600 series detects Lua-specific defect patterns based on
luacheck, LuaTaint, FLuaScan, and Luau type system research.

## Checks (8 total)

  CB-600: Implicit Globals
    Assignment without `local` keyword (luacheck W111/W113).
    Tracks function params, loop vars, and local declarations to
    avoid false positives. Brace depth tracking excludes table
    constructor fields.

    -- Bad:
    count = 0               -- implicit global

    -- Good:
    local count = 0         -- explicit local

  CB-601: Nil-Unsafe Access
    Chained calls on function returns (`):` / `).`) or deep field
    access chains (3+ levels like `a.b.c.d`).

    -- Bad:
    get_player():set_health(100)  -- nil if get_player() returns nil

    -- Good:
    local player = get_player()
    if player then player:set_health(100) end

  CB-602: pcall Error Handling
    Uncaptured or unchecked pcall/xpcall return values.

    -- Bad:
    pcall(dangerous_fn)     -- error silently swallowed

    -- Good:
    local ok, err = pcall(dangerous_fn)
    if not ok then log(err) end

  CB-603: Deprecated/Dangerous API
    os.execute(), io.popen(), loadstring(), setfenv() — command
    injection and sandbox escape vectors.

    -- Bad:
    os.execute('rm -rf ' .. user_input)

    -- Good:
    -- Use restricted API or sanitized subprocess library

  CB-604: Unused Variables
    `local var = ...` where var is never referenced again
    (luacheck W211). Prefix with `_` if intentional.

    -- Bad:
    local result = compute()  -- never used

    -- Good:
    local _result = compute() -- intentionally unused

  CB-605: String Concat in Loop
    `..` operator inside for/while/repeat creates O(n^2) behavior.

    -- Bad:
    local s = ''
    for i = 1, 1000 do
        s = s .. tostring(i)  -- O(n^2)
    end

    -- Good:
    local parts = {{}}
    for i = 1, 1000 do
        parts[#parts + 1] = tostring(i)
    end
    local s = table.concat(parts)

  CB-606: Missing Module Return
    `local M = {{}}` pattern without final `return M`.

    -- Bad:
    local M = {{}}
    function M.init() end
    -- forgot return M

    -- Good:
    local M = {{}}
    function M.init() end
    return M

  CB-607: Colon/Dot Confusion
    Mixed `:` and `.` method calls on same table — indicates
    inconsistent self parameter handling.

    -- Bad:
    obj.method1()   -- no self
    obj:method2()   -- with self (inconsistent)

    -- Good:
    obj:method1()   -- consistent colon syntax
    obj:method2()

## Severity Tiers

  Error   : >10 implicit globals per file (CB-600)
  Warning : Implicit globals, nil-unsafe, pcall, dangerous APIs
  Info    : Unused vars, string concat, module return, colon/dot

## False Positive Avoidance

  - Function parameters tracked as known locals (CB-600)
  - For-loop variables tracked as known locals (CB-600)
  - Table constructor fields excluded via brace depth (CB-600)
  - String literal contents excluded from pattern matching
  - Test files (test_*.lua, *_test.lua, spec/) excluded

## Example Output

  $ pmat comply check

  CB-600: Lua Best Practices (CB-600 to CB-607): [Advisory] 0 errors, 3 warnings, 2 info:
  CB-600: Implicit global `count` — missing `local` keyword (src/main.lua:15)
  CB-601: Nil-unsafe: chained access on function return value (src/init.lua:42)
  CB-603: Dangerous API `os.execute()` (src/deploy.lua:8)
  CB-604: Unused variable `tmp` — prefix with `_` if intentional (src/util.lua:23)
  CB-605: String concatenation (`..`) in loop — O(n^2), use table.concat() (src/render.lua:67)
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
