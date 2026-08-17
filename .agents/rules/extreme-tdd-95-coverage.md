# EXTREME TDD: Achieve 95% Test Coverage

**PMAT Prompt for AI Agents**: Systematically repair ignored tests and achieve 95% coverage using EXTREME TDD methodology.

## Mission

Transform projects to world-class quality standards matching:
- `../trueno` - 95%+ coverage, property tests, mutation tests
- `../trueno-db` - 95%+ coverage, property tests, mutation tests
- `../bashrs` - 95%+ coverage, property tests, mutation tests
- `../renacer` - 95%+ coverage, property tests, mutation tests
- `../ruchy` - 95%+ coverage with FAST testing (<1s test suite)

## Phase 1: Triage Ignored Tests

### Step 1.1: Inventory All Ignored Tests

```bash
# Find all ignored tests
rg "#\[ignore\]" server/src server/tests -A2 --json > /tmp/ignored_tests.json

# Count by category
rg "#\[ignore\]" server/src server/tests -c | sort -t: -k2 -rn | head -20
```

### Step 1.2: Categorize Each Test

For each ignored test, determine category:

**Category A: DELETE (Obsolete)**
- Functionality removed
- Duplicates existing test
- No longer relevant
- **Action**: Remove test entirely

**Category B: FIX (Broken but needed)**
- Core functionality
- Production code exists
- Test is legitimate
- **Action**: Apply EXTREME TDD to fix

**Category C: DEFER (Infrastructure needed)**
- Requires external binary (pmat)
- Needs feature flag (#[cfg(feature)])
- Requires network/database
- **Action**: Document dependencies, keep ignored for now

### Step 1.3: Create Triage Report

```bash
# Output triage decisions
pmat analyze tests --ignored --triage-report > /tmp/triage_report.md
```

**Template**:
```markdown
# Ignored Test Triage Report

## Statistics
- Total ignored: X
- DELETE: Y (Z%)
- FIX: Y (Z%)
- DEFER: Y (Z%)

## Category A: DELETE (Y tests)
- [ ] test_name_1 - Reason: Duplicate of test_name_2
- [ ] test_name_2 - Reason: Feature removed in v2.0

## Category B: FIX (Y tests) - PRIORITY
- [ ] test_complexity_calculation - Core functionality, needs fix
- [ ] test_mutation_scoring - Production code exists

## Category C: DEFER (Y tests)
- [ ] test_binary_integration - Requires pmat binary
- [ ] test_e2e_full - Requires full stack
```

## Phase 2: Delete Obsolete Tests

For each Category A test:

```bash
# Remove test and #[ignore] attribute
# Commit immediately
git add -p
git commit -m "test: Delete obsolete test_name - Reason"
```

**Verify**:
```bash
cargo test --lib  # Should pass
cargo clippy      # No warnings
```

## Phase 3: EXTREME TDD Fix Cycle

For each Category B test (in priority order):

### Step 3.1: RED - Write Failing Test

```bash
# Remove #[ignore] attribute
# Run test - should fail RED
cargo test test_name -- --nocapture

# Capture failure output
cargo test test_name 2>&1 | tee /tmp/test_failure.log
```

### Step 3.2: GREEN - Implement Minimum Code

```rust
// Implement MINIMUM code to make test pass
// No over-engineering
// No extra features
// Just make it GREEN
```

**Verify**:
```bash
# Test should pass
cargo test test_name

# Fast feedback (<1s for unit test)
time cargo test test_name
```

### Step 3.3: Property Tests

```rust
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn property_holds_for_all_inputs(input in any::<YourType>()) {
            // Property should hold for ALL inputs
            assert!(property_holds(input));
        }
    }
}
```

**Examples** (from ../trueno, ../bashrs):
- **Commutativity**: `f(a, b) == f(b, a)`
- **Associativity**: `f(f(a, b), c) == f(a, f(b, c))`
- **Idempotence**: `f(f(x)) == f(x)`
- **Inverse**: `decode(encode(x)) == x`
- **Monotonicity**: `x < y => f(x) <= f(y)`

### Step 3.4: Mutation Tests

```bash
# Run mutation testing on fixed module
cargo mutants --file src/path/to/module.rs

# Target: 80%+ mutation score
# All mutations should be caught by tests
```

**Common mutations to catch**:
- `>` → `>=` (boundary conditions)
- `&&` → `||` (logical operators)
- `+` → `-` (arithmetic)
- `return x` → `return !x` (negation)

### Step 3.5: Optional Example

If functionality is user-facing, create example:

```bash
# Create example demonstrating feature
cat > examples/feature_name.rs <<'EOF'
//! Demonstrates feature_name usage

fn main() -> anyhow::Result<()> {
    // Minimal working example
    // Should complete in <1s
    Ok(())
}
EOF

# Verify example runs
cargo run --example feature_name
```

**Criteria for examples**:
- User-facing features (APIs, CLI commands)
- Complex workflows needing demonstration
- Performance benchmarks
- **Skip**: Internal utilities, trivial functions

### Step 3.6: Verify Fast Testing

```bash
# Entire test suite should be FAST
time cargo test --lib

# Target: <10s for unit tests (like ../ruchy)
# Target: <30s for integration tests
```

**If slow**:
- Use `#[ignore]` for slow integration tests
- Move to separate `cargo test --test integration`
- Use `cargo nextest` for parallelization
- Mock expensive operations (I/O, network)

### Step 3.7: Commit

```bash
git add .
git commit -m "test: Fix test_name with EXTREME TDD

RED: Test was failing due to X
GREEN: Implemented Y to fix
PROPERTY: Added Z property tests
MUTATION: 85% mutation score

Coverage: +2.3% (now 87.5%)
"
```

## Phase 4: Coverage Validation

After fixing all Category B tests:

### Step 4.1: Measure Coverage

```bash
# Generate coverage report
cargo llvm-cov --html

# Check percentage
cargo llvm-cov --json | jq '.data[0].totals.lines.percent'
```

### Step 4.2: Find Uncovered Lines

```bash
# Find uncovered code
cargo llvm-cov --show-missing-lines

# Prioritize:
# 1. Production code (src/lib.rs, src/services/)
# 2. Core modules (not examples/, not tests/)
# 3. Error paths (Result::Err branches)
```

### Step 4.3: Write Tests for Uncovered Code

For each uncovered block:

```rust
#[cfg(test)]
mod coverage_tests {
    #[test]
    fn test_error_path_x() {
        // RED: Test error condition
        // GREEN: Verify error handling
        // PROPERTY: Error invariants hold
    }
}
```

### Step 4.4: Validate 95% Threshold

```bash
# Must achieve 95%+
COVERAGE=$(cargo llvm-cov --json | jq '.data[0].totals.lines.percent')

if (( $(echo "$COVERAGE < 95.0" | bc -l) )); then
    echo "❌ Coverage $COVERAGE% below 95% threshold"
    exit 1
else
    echo "✅ Coverage $COVERAGE% meets 95% threshold"
fi
```

## Phase 5: Performance & Quality Gates

### Step 5.1: Fast Test Suite

```bash
# Unit tests: <10s (like ../ruchy)
time cargo test --lib

# Integration tests: <30s
time cargo test --test '*'

# All tests with nextest: <5s (parallel)
time cargo nextest run
```

**Optimization strategies**:
- Mock I/O operations
- Use `std::hint::black_box` for benchmarks
- Separate slow tests with `#[ignore]`
- Use `cargo nextest` sharding

### Step 5.2: Mutation Testing

```bash
# Target: 80%+ mutation score
cargo mutants

# Must catch:
# - Boundary conditions (>, >=, <, <=)
# - Logical operators (&&, ||, !)
# - Return values (x, !x, None, Some)
# - Arithmetic (+, -, *, /)
```

### Step 5.3: Property Testing

Ensure all critical functions have property tests:

```rust
// Example from ../trueno
proptest! {
    #[test]
    fn matrix_multiply_associative(a: Matrix, b: Matrix, c: Matrix) {
        assert_eq!((a * b) * c, a * (b * c));
    }
}
```

### Step 5.4: Quality Gate

```bash
# Run full quality gate
pmat quality-gate --fail-on-violation

# Must pass:
# ✅ Tests: 100% passing
# ✅ Coverage: ≥95%
# ✅ Mutation: ≥80%
# ✅ Clippy: Zero warnings
# ✅ Complexity: <20 per function
```

## Phase 6: Documentation & Maintenance

### Step 6.1: Update CLAUDE.md

```markdown
### Test Coverage

**Status**: 95.2% coverage (EXTREME TDD compliant)

**Methodology**:
- Property tests: 47 tests covering core invariants
- Mutation score: 82% (cargo mutants)
- Fast tests: <10s unit, <30s integration
- Examples: 12 runnable examples (cargo run --example)

**Previously Ignored Tests**:
- Deleted: 15 obsolete tests
- Fixed: 42 tests using EXTREME TDD
- Deferred: 8 tests (require external deps)

**Compliance**: Matches ../trueno, ../bashrs, ../renacer quality standards
```

### Step 6.2: Add Pre-Commit Hook

```bash
# Ensure coverage doesn't regress
cat > .git/hooks/pre-commit <<'EOF'
#!/bin/bash
COVERAGE=$(cargo llvm-cov --json | jq '.data[0].totals.lines.percent')
if (( $(echo "$COVERAGE < 95.0" | bc -l) )); then
    echo "❌ Coverage $COVERAGE% below 95%"
    exit 1
fi
EOF
chmod +x .git/hooks/pre-commit
```

### Step 6.3: CI Integration

```yaml
# .github/workflows/coverage.yml
- name: Check 95% Coverage
  run: |
    cargo llvm-cov --json > coverage.json
    COVERAGE=$(jq '.data[0].totals.lines.percent' coverage.json)
    if (( $(echo "$COVERAGE < 95.0" | bc -l) )); then
      echo "Coverage $COVERAGE% below threshold"
      exit 1
    fi
```

## Success Criteria

**Project achieves all of:**
- ✅ 95%+ test coverage (cargo llvm-cov)
- ✅ 80%+ mutation score (cargo mutants)
- ✅ <10s unit test suite (like ../ruchy)
- ✅ Property tests for core invariants
- ✅ All ignored tests triaged (delete/fix/defer)
- ✅ Runnable examples for user-facing features
- ✅ Zero clippy warnings
- ✅ Quality gates enforced in CI

## Reference Projects

**Study these for EXTREME TDD patterns**:

1. **../trueno** - SIMD tensor operations
   - 95%+ coverage
   - Property tests for math invariants
   - Fast tests (<5s with nextest)

2. **../trueno-db** - Graph database
   - 95%+ coverage
   - Mutation tests for data integrity
   - Examples for all APIs

3. **../bashrs** - Bash linter
   - 95%+ coverage
   - Property tests for parsing
   - Fast tests (<2s)

4. **../renacer** - Deterministic testing
   - 95%+ coverage
   - Golden trace validation
   - Integration examples

5. **../ruchy** - Programming language
   - 95%+ coverage
   - <1s test suite (FAST)
   - Comprehensive property tests

## Common Pitfalls

❌ **Don't**: Write tests after code (defeats TDD)
✅ **Do**: RED → GREEN → REFACTOR cycle

❌ **Don't**: Over-engineer solutions
✅ **Do**: Minimum code to pass test

❌ **Don't**: Skip property tests
✅ **Do**: Property tests for invariants

❌ **Don't**: Accept slow tests
✅ **Do**: Mock I/O, use nextest

❌ **Don't**: Ignore mutation failures
✅ **Do**: Fix until 80%+ mutation score

❌ **Don't**: Keep obsolete ignored tests
✅ **Do**: Triage and delete ruthlessly

## Timeline Estimate

For typical Rust project (50K LOC):

- **Phase 1** (Triage): 2-4 hours
- **Phase 2** (Delete): 1 hour
- **Phase 3** (Fix tests): 2-5 days (depends on test count)
- **Phase 4** (Coverage): 1-2 days
- **Phase 5** (Quality): 1 day
- **Phase 6** (Docs): 2 hours

**Total**: 5-10 days for 95% coverage transformation

## Usage

**As AI Agent Prompt**:
```
Apply EXTREME TDD methodology from docs/agent-instructions/extreme-tdd-95-coverage.md
to achieve 95% test coverage. Start with Phase 1 triage.
```

**As pmat Command** (future):
```bash
pmat extreme-tdd --target-coverage 95 --triage-first
```

**As Quality Gate**:
```bash
pmat quality-gate --coverage-threshold 95 --mutation-threshold 80
```
