# TICKET-PMAT-6001: Health Command Optimization

**Sprint:** Sprint 20 - UX Improvements & Optimizations
**Priority:** P0 - Critical
**Estimated Effort:** 4-6 hours
**Status:** 📋 Planned
**Created:** 2025-10-06

## Problem Statement

The `pmat maintain health` command currently times out after 300 seconds because it runs all health checks by default:
- Build check (`cargo check`)
- Test check (`cargo test`) - takes 606s alone
- Coverage check (`cargo llvm-cov`) - adds 100+ seconds
- Complexity check (placeholder)
- SATD check (placeholder)

**Impact:**
- Developers can't get quick health feedback
- Command is unusable for rapid iteration
- CI/CD pipelines timeout
- Poor user experience

**Evidence:** `docs/dogfooding/SPRINT-19-DOGFOODING-RESULTS.md` - Health command timed out at 300s

## Solution

Add `--quick` mode and make expensive checks opt-in instead of running all checks by default.

### Current Behavior

```bash
pmat maintain health
# Runs: build + tests + coverage + complexity + SATD
# Takes: >300s (timeout)
```

### Proposed Behavior

```bash
# Quick health check (default) - build + basic tests only
pmat maintain health
# Takes: <30s

# Explicit quick mode
pmat maintain health --quick
# Takes: <10s (build check only)

# Full health check with opt-in flags
pmat maintain health --all
# Takes: ~700s but completes

# Individual checks
pmat maintain health --check-coverage
pmat maintain health --check-tests --check-coverage
```

## Requirements

### Functional Requirements

1. **Default Behavior Changes**
   - Default: Run only `--check-build` (fastest, most essential)
   - No tests, coverage, complexity, or SATD by default
   - Complete in <10s for typical project

2. **Quick Mode**
   - `--quick` flag: Build check only
   - Clear output showing what was skipped
   - Exit code 0 if build succeeds

3. **Full Mode**
   - `--all` flag: Enable all checks
   - Equivalent to old default behavior
   - Show estimated time before starting

4. **Individual Check Flags**
   - Keep existing flags: `--check-build`, `--check-tests`, `--check-coverage`, etc.
   - Change behavior: Flags now explicitly enable checks
   - Multiple flags can be combined

5. **Backward Compatibility**
   - Warn if using old CLI pattern (no flags)
   - Suggest using `--all` if old behavior desired

### Non-Functional Requirements

1. **Performance**
   - Default mode: <10s
   - Quick mode: <10s
   - Build + tests: <60s (without full test suite)
   - Full mode: Accept long runtime but don't timeout

2. **User Experience**
   - Clear messaging about what's running
   - Show skipped checks
   - Explain how to enable more checks

3. **Quality**
   - Cyclomatic complexity <8
   - Test coverage >80%
   - Property tests for flag combinations

## Implementation Plan

### Step 1: Update Command Definitions

**File:** `server/src/cli/commands.rs`

**Changes:**
```rust
/// Validate project health (TICKET-PMAT-5033, TICKET-PMAT-6001)
Health {
    /// Project directory
    #[arg(long, default_value = ".")]
    project_dir: PathBuf,

    /// Output format
    #[arg(long, value_enum, default_value = "table")]
    format: OutputFormat,

    /// Quick mode: build check only (default behavior)
    #[arg(long)]
    quick: bool,

    /// Run all checks (build + tests + coverage + complexity + SATD)
    #[arg(long)]
    all: bool,

    /// Check build status (enabled by default)
    #[arg(long)]
    check_build: bool,

    /// Check tests (opt-in)
    #[arg(long)]
    check_tests: bool,

    /// Check coverage (opt-in)
    #[arg(long)]
    check_coverage: bool,

    /// Check complexity (opt-in)
    #[arg(long)]
    check_complexity: bool,

    /// Check SATD (opt-in)
    #[arg(long)]
    check_satd: bool,

    /// Show estimated time before running checks
    #[arg(long)]
    estimate: bool,
},
```

**Complexity:** CC=2 (trivial)

---

### Step 2: Update Health Handler Logic

**File:** `server/src/cli/handlers/health_handler.rs`

**Function:** `handle_maintain_health`

**Changes:**

```rust
pub async fn handle_maintain_health(
    project_dir: PathBuf,
    format: OutputFormat,
    quick: bool,
    all: bool,
    check_build: bool,
    check_tests: bool,
    check_coverage: bool,
    check_complexity: bool,
    check_satd: bool,
    estimate: bool,
) -> Result<()> {
    // Determine which checks to run
    let checks_to_run = determine_checks_to_run(
        quick,
        all,
        check_build,
        check_tests,
        check_coverage,
        check_complexity,
        check_satd,
    )?;

    // Show estimated time if requested
    if estimate {
        show_estimated_time(&checks_to_run);
    }

    // Run selected checks
    let mut results = Vec::new();
    for check_type in checks_to_run {
        let result = run_check(check_type, &project_dir).await?;
        results.push(result);
    }

    // Generate and print report
    let report = HealthReport {
        healthy: results.iter().all(|r| r.status == CheckStatus::Pass),
        checks: results,
        summary: calculate_summary(&results),
    };

    print_health_report(&report, &format)?;

    if !report.healthy {
        std::process::exit(1);
    }

    Ok(())
}
```

**Complexity Target:** CC=6

---

### Step 3: Add Check Determination Logic

**Function:** `determine_checks_to_run`

```rust
/// Determine which checks to run based on flags (CC=7)
fn determine_checks_to_run(
    quick: bool,
    all: bool,
    check_build: bool,
    check_tests: bool,
    check_coverage: bool,
    check_complexity: bool,
    check_satd: bool,
) -> Result<Vec<CheckType>> {
    let mut checks = Vec::new();

    // Quick mode: build only
    if quick {
        checks.push(CheckType::Build);
        return Ok(checks);
    }

    // All mode: enable everything
    if all {
        checks.extend([
            CheckType::Build,
            CheckType::Tests,
            CheckType::Coverage,
            CheckType::Complexity,
            CheckType::Satd,
        ]);
        return Ok(checks);
    }

    // Individual flags: opt-in
    // Default to build if nothing specified
    if !check_build && !check_tests && !check_coverage && !check_complexity && !check_satd {
        checks.push(CheckType::Build);
    } else {
        if check_build {
            checks.push(CheckType::Build);
        }
        if check_tests {
            checks.push(CheckType::Tests);
        }
        if check_coverage {
            checks.push(CheckType::Coverage);
        }
        if check_complexity {
            checks.push(CheckType::Complexity);
        }
        if check_satd {
            checks.push(CheckType::Satd);
        }
    }

    Ok(checks)
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum CheckType {
    Build,
    Tests,
    Coverage,
    Complexity,
    Satd,
}
```

**Complexity:** CC=7

---

### Step 4: Add Time Estimation

**Function:** `show_estimated_time`

```rust
/// Show estimated time for checks (CC=4)
fn show_estimated_time(checks: &[CheckType]) {
    let estimates = vec![
        (CheckType::Build, "5-10 seconds"),
        (CheckType::Tests, "60-600 seconds"),
        (CheckType::Coverage, "100-200 seconds"),
        (CheckType::Complexity, "10-30 seconds"),
        (CheckType::Satd, "5-15 seconds"),
    ];

    println!("⏱️  Estimated Time:");
    for check in checks {
        if let Some((_, time)) = estimates.iter().find(|(t, _)| t == check) {
            println!("  - {:?}: {}", check, time);
        }
    }
    println!();
}
```

**Complexity:** CC=3

---

## Test Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quick_mode_only_build() {
        let checks = determine_checks_to_run(
            true, false, false, false, false, false, false
        ).unwrap();
        assert_eq!(checks, vec![CheckType::Build]);
    }

    #[test]
    fn test_all_mode_enables_everything() {
        let checks = determine_checks_to_run(
            false, true, false, false, false, false, false
        ).unwrap();
        assert_eq!(checks.len(), 5);
    }

    #[test]
    fn test_default_runs_build_only() {
        let checks = determine_checks_to_run(
            false, false, false, false, false, false, false
        ).unwrap();
        assert_eq!(checks, vec![CheckType::Build]);
    }

    #[test]
    fn test_individual_flags_combine() {
        let checks = determine_checks_to_run(
            false, false, true, true, false, false, false
        ).unwrap();
        assert_eq!(checks, vec![CheckType::Build, CheckType::Tests]);
    }

    #[test]
    fn test_quick_overrides_all() {
        let checks = determine_checks_to_run(
            true, true, false, false, false, false, false
        ).unwrap();
        assert_eq!(checks, vec![CheckType::Build]);
    }
}
```

### Integration Tests

```rust
#[test]
fn test_health_quick_mode_performance() {
    let start = Instant::now();
    let output = Command::new("pmat")
        .args(&["maintain", "health", "--quick"])
        .output()
        .expect("Failed to execute");

    let duration = start.elapsed();
    assert!(output.status.success());
    assert!(duration.as_secs() < 15, "Quick mode took {}s (should be <15s)", duration.as_secs());
}

#[test]
fn test_health_default_is_quick() {
    let output = Command::new("pmat")
        .args(&["maintain", "health"])
        .output()
        .expect("Failed to execute");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Build"));
    assert!(!stdout.contains("Coverage")); // Coverage not run by default
}
```

### Property Tests

```rust
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn quick_mode_always_returns_one_check(
            all in any::<bool>(),
            build in any::<bool>(),
            tests in any::<bool>(),
            coverage in any::<bool>(),
            complexity in any::<bool>(),
            satd in any::<bool>(),
        ) {
            let checks = determine_checks_to_run(
                true, all, build, tests, coverage, complexity, satd
            ).unwrap();
            prop_assert_eq!(checks.len(), 1);
            prop_assert_eq!(checks[0], CheckType::Build);
        }

        #[test]
        fn all_mode_always_returns_five_checks(
            quick in any::<bool>(),
            build in any::<bool>(),
            tests in any::<bool>(),
            coverage in any::<bool>(),
            complexity in any::<bool>(),
            satd in any::<bool>(),
        ) {
            // Quick takes precedence, so skip if quick is true
            prop_assume!(!quick);

            let checks = determine_checks_to_run(
                quick, true, build, tests, coverage, complexity, satd
            ).unwrap();
            prop_assert_eq!(checks.len(), 5);
        }
    }
}
```

## Documentation Updates

### Help Text

Update `--help` output to explain new behavior:

```
pmat maintain health --help

Validate project health (TICKET-PMAT-5033, TICKET-PMAT-6001)

By default, runs only the build check for fast feedback.
Use --all to run comprehensive checks, or specify individual checks.

Examples:
  # Quick health check (default, ~10s)
  pmat maintain health

  # Explicit quick mode
  pmat maintain health --quick

  # Full health check (~10min)
  pmat maintain health --all

  # Specific checks
  pmat maintain health --check-build --check-tests

  # Show estimated time
  pmat maintain health --all --estimate
```

### README Updates

Update `examples/README.md` and other docs to reflect new behavior.

## Acceptance Criteria

- [ ] `pmat maintain health` (default) completes in <10s
- [ ] `pmat maintain health --quick` completes in <10s
- [ ] `pmat maintain health --all` runs all checks (accepts long runtime)
- [ ] Individual check flags work correctly
- [ ] Flag combinations work as expected
- [ ] Help text explains new behavior
- [ ] All unit tests passing
- [ ] Integration tests verify performance
- [ ] Property tests verify flag logic
- [ ] Documentation updated
- [ ] Cyclomatic complexity <8 for all functions
- [ ] Test coverage >80%

## Complexity Analysis

| Function | Estimated CC | Target CC | Status |
|----------|-------------|-----------|--------|
| `handle_maintain_health` | 6 | <8 | ✅ |
| `determine_checks_to_run` | 7 | <8 | ✅ |
| `show_estimated_time` | 3 | <8 | ✅ |
| `run_check` (existing) | 5 | <8 | ✅ |

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Breaking changes for existing users | Low | Medium | Add deprecation warnings, document migration |
| Quick mode insufficient for some users | Medium | Low | Document `--all` flag, make it easy to discover |
| Complexity increases | Low | Medium | Extract helper functions, keep CC <8 |

## Related Tickets

- TICKET-PMAT-5033: Original health command implementation
- TICKET-PMAT-6002: Progress indicators (will enhance this)
- TICKET-PMAT-6005: CLI integration tests (will test this)

## References

- Dogfooding Results: `docs/dogfooding/SPRINT-19-DOGFOODING-RESULTS.md`
- Sprint 20 Plan: `docs/sprints/SPRINT-20-PLAN.md`
- Original Implementation: `server/src/cli/handlers/health_handler.rs:L1`

---

**Status:** Ready for implementation
**Estimated Time:** 4-6 hours
**Complexity:** Medium (refactoring existing code)
