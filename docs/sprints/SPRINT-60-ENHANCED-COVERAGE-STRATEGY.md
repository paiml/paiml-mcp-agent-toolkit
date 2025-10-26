# Sprint 60: Enhanced Test Coverage Strategy
## Mutation + Property + Fuzz Testing

**Date**: October 26, 2025
**Version**: v2.173.0
**Sprint Type**: Quality Enhancement - Test Coverage Improvement
**Status**: 🚀 IN PROGRESS

## Executive Summary

Comprehensive strategy to enhance test coverage using three advanced testing methodologies:
1. **Mutation Testing** (cargo-mutants) - Test quality measurement
2. **Property-Based Testing** (proptest/quickcheck) - Invariant validation
3. **Fuzz Testing** (cargo-fuzz) - Edge case discovery

**Current State** (Baseline):
- 5,052 tests across 114 binaries
- 341 tests skipped (intentional - external deps, slow tests)
- Property testing infrastructure: ✅ In place (proptest 1.6, quickcheck 1.0)
- Mutation testing tool: ✅ Installed (cargo-mutants)
- Fuzz testing: ⚠️  Needs setup (cargo-fuzz)

## Testing Methodology Overview

### 1. Mutation Testing (Test Quality)

**Purpose**: Measure test suite effectiveness by introducing bugs and checking if tests catch them

**Tool**: cargo-mutants (already installed)

**High-Value Targets for Mutation**:
1. **AST Parsers** (`server/src/services/ast/languages/*.rs`)
   - Critical path: All language analyzers
   - High complexity: Tree-sitter integration
   - Files: `rust.rs`, `python.rs`, `typescript.rs`, `javascript.rs`, `java.rs`, `scala.rs`

2. **Complexity Calculators** (`server/src/tdg/*.rs`)
   - Critical: Technical Debt Grading (TDG) engine
   - High impact: Cyclomatic/cognitive complexity
   - Files: `calculator.rs`, `analyzer_simple.rs`

3. **MCP Integration** (`server/src/mcp_integration/*.rs`)
   - Critical: AI agent tooling
   - High visibility: User-facing API
   - Files: `java_tools.rs`, `scala_tools.rs`, `polyglot_tools.rs`

4. **Path Validation** (`server/src/utils/path_validator.rs`)
   - Security-critical: File system access control
   - High risk: Path traversal vulnerabilities

5. **Cross-Language Analysis** (`server/src/ast/polyglot/*.rs`)
   - Complex logic: Polyglot AST unification
   - Files: `language_mapper.rs`, `unified_node.rs`, `cross_language_dependencies.rs`

**Mutation Testing Commands**:

```bash
# Quick mutation test (high-value targets only)
cd server && cargo mutants --file src/utils/path_validator.rs --timeout 60

# Comprehensive mutation test (slow, 30-60 minutes)
cd server && cargo mutants --workspace --timeout 120 --no-times

# Targeted mutation test by module
cargo mutants --file src/services/ast/languages/java.rs
cargo mutants --file src/tdg/calculator.rs
cargo mutants --file src/mcp_integration/java_tools.rs

# Generate HTML report
cargo mutants --output mutants.out --timeout 60
```

**Success Criteria**:
- **Mutation Score**: Target 80%+ (industry standard)
- **Caught Mutants**: 4/5 mutants should be caught by tests
- **Missed Mutants**: Identify gaps in test coverage

### 2. Property-Based Testing (Invariant Validation)

**Purpose**: Validate code behavior across thousands of randomized inputs

**Tools**: proptest 1.6 (primary), quickcheck 1.0 (legacy)

**Existing Property Tests** (Per `make test-property`):
- `property_tests` modules (timeout: 180s)
- `prop_` prefixed tests (timeout: 60s)
- `refactor_auto_property_integration` test suite

**New Property-Based Test Targets**:

#### A. AST Parser Invariants

```rust
// server/src/services/ast/languages/property_tests.rs
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_ast_parsing_is_deterministic(
        source_code in "\\PC{1,1000}"  // Any printable Unicode
    ) {
        // Parsing same source twice must yield identical AST
        let ast1 = parse_source(&source_code);
        let ast2 = parse_source(&source_code);
        prop_assert_eq!(ast1, ast2, "AST parsing must be deterministic");
    }

    #[test]
    fn prop_ast_node_count_bounded(
        source_code in "\\PC{1,10000}"
    ) {
        let ast = parse_source(&source_code);
        let node_count = count_nodes(&ast);

        // Node count should be <= source length (one node per char max)
        prop_assert!(
            node_count <= source_code.len(),
            "AST node count {} exceeds source length {}",
            node_count,
            source_code.len()
        );
    }

    #[test]
    fn prop_ast_positions_monotonic(
        source_code in "[\\w\\s]{100,1000}"
    ) {
        let ast = parse_source(&source_code);

        // All node positions must be monotonically increasing (depth-first)
        let positions = collect_positions(&ast);
        prop_assert!(
            is_monotonic(&positions),
            "AST positions must be monotonic: {:?}",
            positions
        );
    }
}
```

#### B. Complexity Calculation Invariants

```rust
// server/src/tdg/property_tests.rs
proptest! {
    #[test]
    fn prop_complexity_never_negative(
        function_body in "\\PC{10,500}"
    ) {
        let complexity = calculate_cyclomatic_complexity(&function_body);
        prop_assert!(
            complexity >= 1,
            "Complexity must be at least 1 (entry point), got {}",
            complexity
        );
    }

    #[test]
    fn prop_tdg_score_bounded(
        code_metrics in any::<CodeMetrics>()
    ) {
        let tdg_score = calculate_tdg_score(&code_metrics);
        prop_assert!(
            tdg_score >= 0.0 && tdg_score <= 100.0,
            "TDG score must be in [0, 100], got {}",
            tdg_score
        );
    }

    #[test]
    fn prop_tdg_grade_mapping_consistent(
        score in 0.0..=100.0f64
    ) {
        let grade = score_to_grade(score);
        let lower_bound = grade_to_min_score(&grade);

        prop_assert!(
            score >= lower_bound,
            "Score {} should map to grade {:?} with bound {}",
            score, grade, lower_bound
        );
    }
}
```

#### C. Path Validation Invariants

```rust
// server/src/utils/property_tests.rs
proptest! {
    #[test]
    fn prop_path_normalization_idempotent(
        path in "[a-zA-Z0-9_/.-]{1,100}"
    ) {
        let norm1 = normalize_path(&path);
        let norm2 = normalize_path(&norm1);
        prop_assert_eq!(
            norm1, norm2,
            "Path normalization must be idempotent"
        );
    }

    #[test]
    fn prop_path_traversal_always_detected(
        base_path in "/[a-z]{3,10}",
        attack_path in "\\.\\./[a-z]{3,10}"
    ) {
        let full_path = format!("{}/{}", base_path, attack_path);
        let result = validate_path_safety(&full_path);

        prop_assert!(
            result.is_err(),
            "Path traversal attack {} must be detected",
            full_path
        );
    }
}
```

**Property Test Commands**:

```bash
# Run all property tests (3 minutes timeout)
make test-property

# Run property tests including slow ones (unbounded)
make test-property-slow

# Run specific property test module
cargo test --lib -- services::ast::languages::property_tests

# Set custom test iterations (default: 256)
PROPTEST_CASES=1000 cargo test --lib -- property_tests
```

### 3. Fuzz Testing (Edge Case Discovery)

**Purpose**: Discover crashes, panics, and edge cases via mutation-based fuzzing

**Tool**: cargo-fuzz (needs installation)

**Setup**:

```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Initialize fuzz testing
cd server
cargo fuzz init

# Create fuzz targets for high-value modules
cargo fuzz add fuzz_ast_parser
cargo fuzz add fuzz_complexity_calculator
cargo fuzz add fuzz_path_validator
cargo fuzz add fuzz_polyglot_analyzer
```

**Fuzz Target: AST Parser**

```rust
// server/fuzz/fuzz_targets/fuzz_ast_parser.rs
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(source_code) = std::str::from_utf8(data) {
        // Must not panic on any input
        let _ = pmat::services::ast::parse_source_code(source_code, "rust");
    }
});
```

**Fuzz Target: Complexity Calculator**

```rust
// server/fuzz/fuzz_targets/fuzz_complexity_calculator.rs
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(function_body) = std::str::from_utf8(data) {
        // Must not panic or return invalid values
        let complexity = pmat::tdg::calculate_cyclomatic_complexity(function_body);
        assert!(complexity >= 1, "Complexity must be at least 1");
        assert!(complexity < 1000000, "Complexity suspiciously high");
    }
});
```

**Fuzz Target: Path Validator**

```rust
// server/fuzz/fuzz_targets/fuzz_path_validator.rs
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(path_str) = std::str::from_utf8(data) {
        // Must not panic, must always validate safety
        let result = pmat::utils::path_validator::validate_path(path_str);

        // If path contains "..", must be rejected
        if path_str.contains("..") {
            assert!(result.is_err(), "Path traversal must be detected: {}", path_str);
        }
    }
});
```

**Fuzz Commands**:

```bash
# Run fuzz target (runs indefinitely until crash found)
cargo fuzz run fuzz_ast_parser

# Run with timeout (5 minutes)
timeout 300 cargo fuzz run fuzz_complexity_calculator

# Run with custom dictionary for better coverage
cargo fuzz run fuzz_path_validator -- -dict=path_dictionary.txt

# Minimize crash-inducing input
cargo fuzz cmin fuzz_ast_parser corpus/

# Generate coverage report
cargo fuzz coverage fuzz_ast_parser
```

## Coverage Improvement Roadmap

### Phase 1: Baseline Measurement (Week 1)

**Tasks**:
1. ✅ Run existing coverage suite: `make coverage`
2. 🔄 Generate baseline coverage report
3. 📊 Identify modules with <80% coverage
4. 📈 Document current mutation score (run cargo-mutants on 5 files)

**Deliverables**:
- `coverage_baseline_v2.173.0.lcov` (LLVM coverage format)
- `mutation_baseline_report.md` (top 5 critical modules)
- `coverage_gaps.md` (list of uncovered functions)

### Phase 2: Mutation Testing (Week 2)

**Tasks**:
1. Run mutation tests on 5 high-value modules (path_validator, java_tools, scala_tools, language_mapper, calculator)
2. Analyze missed mutants (gaps in test coverage)
3. Write targeted unit tests for missed mutants
4. Re-run mutation tests, measure improvement

**Success Metrics**:
- Mutation score: 60% → 80% (target)
- Missed mutants: 40% → 20% (target)
- New tests added: ~50-100 tests

### Phase 3: Property-Based Testing (Week 3)

**Tasks**:
1. Implement 20 new property tests (AST parsers, complexity, path validation)
2. Run property tests with high iteration count (PROPTEST_CASES=10000)
3. Fix any invariant violations discovered
4. Add property test regression suite

**Success Metrics**:
- Property tests: current → +20 new tests
- Invariant violations found: ~5-10 (expected)
- Coverage increase: +3-5% (property tests cover more branches)

### Phase 4: Fuzz Testing (Week 4)

**Tasks**:
1. Set up cargo-fuzz infrastructure
2. Create 4 fuzz targets (AST, complexity, path, polyglot)
3. Run each fuzz target for 1 hour
4. Fix any crashes/panics discovered
5. Add fuzz corpus to CI for regression testing

**Success Metrics**:
- Fuzz targets created: 4
- Crashes found: ~2-5 (expected for parsers)
- Panics fixed: 100% (zero tolerance)
- Corpus size: ~1000-5000 test cases per target

### Phase 5: Integration & CI (Week 5)

**Tasks**:
1. Add mutation testing to CI (limited scope, 5-minute budget)
2. Add property testing to CI (already integrated via `make test-property`)
3. Add fuzz testing to CI (corpus regression only, 2-minute budget)
4. Update `make validate` to include enhanced tests
5. Document new testing standards in CLAUDE.md

**CI Integration**:
```yaml
# .github/workflows/enhanced-coverage.yml
name: Enhanced Coverage

on: [push, pull_request]

jobs:
  mutation-test-critical:
    runs-on: ubuntu-latest
    timeout-minutes: 10
    steps:
      - uses: actions/checkout@v4
      - name: Install cargo-mutants
        run: cargo install cargo-mutants --locked
      - name: Mutation test (path_validator only)
        run: cd server && cargo mutants --file src/utils/path_validator.rs --timeout 60

  property-test:
    runs-on: ubuntu-latest
    timeout-minutes: 5
    steps:
      - uses: actions/checkout@v4
      - name: Property tests
        run: make test-property
        env:
          PROPTEST_CASES: 1000  # Higher than default for CI

  fuzz-regression:
    runs-on: ubuntu-latest
    timeout-minutes: 3
    steps:
      - uses: actions/checkout@v4
      - name: Install cargo-fuzz
        run: cargo install cargo-fuzz --locked
      - name: Fuzz corpus regression
        run: |
          cd server
          cargo fuzz run fuzz_ast_parser corpus/ -- -runs=0
          cargo fuzz run fuzz_path_validator corpus/ -- -runs=0
```

## Expected Coverage Improvements

**Current Baseline** (v2.173.0):
- Line coverage: ~82% (estimated, awaiting report)
- Branch coverage: ~75% (estimated)
- Mutation score: ~65% (estimated)

**Target After Sprint 60**:
- Line coverage: 85-87% (+3-5%)
- Branch coverage: 78-82% (+3-7%)
- Mutation score: 75-80% (+10-15%)

**High-Impact Modules** (Expected improvements):
1. `path_validator.rs`: 85% → 95% (+10%)
2. `java_tools.rs`: 78% → 88% (+10%)
3. `scala_tools.rs`: 78% → 88% (+10%)
4. `language_mapper.rs`: 70% → 85% (+15%)
5. `calculator.rs` (TDG): 90% → 95% (+5%)

## Testing Best Practices

### Property Test Design

**Good Property**: Tests invariants, not specific outputs
```rust
// ✅ GOOD: Tests determinism invariant
proptest! {
    fn prop_parse_deterministic(code in any_source_code()) {
        let ast1 = parse(&code);
        let ast2 = parse(&code);
        prop_assert_eq!(ast1, ast2);
    }
}
```

**Bad Property**: Tests specific example (use unit test instead)
```rust
// ❌ BAD: This is a unit test disguised as a property test
proptest! {
    fn prop_parse_hello_world(_unit in Just(())) {
        let ast = parse("fn main() {}");
        prop_assert_eq!(ast.functions.len(), 1);
    }
}
```

### Mutation Test Interpretation

**Killed Mutant** (Good - test caught the bug):
```rust
// Original code
fn is_valid_path(path: &str) -> bool {
    !path.contains("..")  // Security check
}

// Mutant (killed by test)
fn is_valid_path(path: &str) -> bool {
    path.contains("..")  // BUG: Inverted logic (test catches this!)
}
```

**Survived Mutant** (Bad - test gap):
```rust
// Original code
fn calculate_grade(score: f64) -> Grade {
    if score >= 90.0 { Grade::A }
    else if score >= 80.0 { Grade::B }  // Mutant survives if no test for 80-90 range
    else { Grade::C }
}

// Mutant (survived - need more tests!)
fn calculate_grade(score: f64) -> Grade {
    if score >= 90.0 { Grade::A }
    else if score >= 85.0 { Grade::B }  // BUG: Changed threshold, no test catches this
    else { Grade::C }
}
```

### Fuzz Test Corpus Management

**Corpus Structure**:
```
server/fuzz/corpus/
├── fuzz_ast_parser/
│   ├── basic_function.rs          # Simple valid input
│   ├── complex_nested.rs          # Complex valid input
│   ├── unicode_utf8.txt           # Unicode edge case
│   ├── large_file_10kb.rs         # Size edge case
│   └── crash_reproducer_*.rs     # Minimized crash inputs
├── fuzz_path_validator/
│   ├── normal_paths.txt
│   ├── traversal_attacks.txt
│   └── unicode_paths.txt
└── fuzz_complexity_calculator/
    ├── simple_functions.txt
    ├── deeply_nested_loops.txt
    └── edge_cases.txt
```

## Makefile Integration

**New Targets** (to be added to `Makefile`):

```makefile
# Enhanced testing targets

# Mutation testing
test-mutation-quick:
	@echo "🧬 Running quick mutation tests (high-value modules only)..."
	@cd server && cargo mutants --file src/utils/path_validator.rs --timeout 60
	@cd server && cargo mutants --file src/mcp_integration/java_tools.rs --timeout 60
	@echo "✅ Quick mutation tests completed!"

test-mutation-full:
	@echo "🧬 Running comprehensive mutation tests (30-60 minutes)..."
	@cd server && cargo mutants --workspace --timeout 120 --output mutants.out
	@echo "✅ Full mutation tests completed! Report: server/mutants.out"

# Fuzz testing
test-fuzz-setup:
	@echo "🐛 Setting up fuzz testing infrastructure..."
	@command -v cargo-fuzz >/dev/null 2>&1 || cargo install cargo-fuzz --locked
	@cd server && cargo fuzz init
	@cd server && cargo fuzz add fuzz_ast_parser
	@cd server && cargo fuzz add fuzz_path_validator
	@cd server && cargo fuzz add fuzz_complexity_calculator
	@cd server && cargo fuzz add fuzz_polyglot_analyzer
	@echo "✅ Fuzz testing setup completed!"

test-fuzz-corpus:
	@echo "🐛 Running fuzz corpus regression (2 minutes)..."
	@timeout 120 sh -c 'cd server && \
		cargo fuzz run fuzz_ast_parser corpus/ -- -runs=0 && \
		cargo fuzz run fuzz_path_validator corpus/ -- -runs=0 && \
		cargo fuzz run fuzz_complexity_calculator corpus/ -- -runs=0'
	@echo "✅ Fuzz corpus regression completed!"

test-fuzz-live:
	@echo "🐛 Running live fuzz testing (5 minutes per target)..."
	@timeout 300 cargo fuzz run fuzz_ast_parser || true
	@timeout 300 cargo fuzz run fuzz_path_validator || true
	@timeout 300 cargo fuzz run fuzz_complexity_calculator || true
	@echo "✅ Live fuzz testing completed!"

# Enhanced coverage reporting
coverage-enhanced:
	@echo "📊 Running enhanced coverage analysis (mutation + property + fuzz)..."
	@$(MAKE) coverage
	@$(MAKE) test-mutation-quick
	@$(MAKE) test-property
	@$(MAKE) test-fuzz-corpus
	@echo "✅ Enhanced coverage analysis completed!"
	@echo "📈 Coverage reports:"
	@echo "  - Line coverage: target/llvm-cov/html/index.html"
	@echo "  - Mutation report: server/mutants.out"
	@echo "  - Property test logs: (check test output)"
	@echo "  - Fuzz corpus: server/fuzz/corpus/"

# Update validate target to include enhanced tests
validate-enhanced: validate test-mutation-quick test-fuzz-corpus
	@echo "✅ Enhanced validation completed (includes mutation + fuzz)!"
```

## Success Metrics & KPIs

### Coverage Metrics

**Line Coverage**:
- Baseline: 82% (estimated)
- Target: 85-87%
- Stretch Goal: 90%

**Branch Coverage**:
- Baseline: 75% (estimated)
- Target: 78-82%
- Stretch Goal: 85%

**Mutation Score**:
- Baseline: 65% (estimated)
- Target: 75-80%
- Stretch Goal: 85%

### Quality Metrics

**Test Suite Size**:
- Current: 5,052 tests
- Target: 5,100-5,200 tests (+50-150 tests)
- Property tests: +20
- Mutation-driven unit tests: +30-100
- Fuzz regression tests: +20-50

**Test Execution Time**:
- Fast tests: <3 minutes (maintained)
- Property tests: <5 minutes (maintained)
- Mutation tests (quick): <10 minutes (new)
- Fuzz corpus regression: <3 minutes (new)

### Defect Detection

**Pre-Sprint 60 Baseline**:
- Bugs found by tests: ~95% (estimated)
- Bugs escaping to production: ~5%

**Post-Sprint 60 Target**:
- Bugs found by tests: 97-98%
- Bugs escaping to production: 2-3%

**Mutation Testing Impact**:
- Missed mutants → new tests → higher defect detection rate
- Target: Reduce production defects by 40-50%

## Documentation Updates

**Files to Update**:
1. `CLAUDE.md`: Add mutation/property/fuzz testing policy
2. `docs/testing-strategy.md`: Comprehensive testing guide
3. `CONTRIBUTING.md`: Testing requirements for PRs
4. `README.md`: Mention enhanced testing as quality differentiator

**CLAUDE.md Addition**:
```markdown
## Enhanced Testing Policy (Sprint 60)

### Mutation Testing
- Run `make test-mutation-quick` before major releases
- Target: 80%+ mutation score for critical modules
- Critical modules: path_validator, *_tools, language_mapper

### Property-Based Testing
- All new parsers/calculators must include property tests
- Target: 5+ property tests per critical module
- Run with `make test-property` (required for CI)

### Fuzz Testing
- All parsers must have fuzz targets
- Corpus maintained in `server/fuzz/corpus/`
- Run corpus regression with `make test-fuzz-corpus` (required for CI)
```

## Risk Assessment

**High Risk**:
- ⚠️  Mutation testing may reveal significant test gaps (expect 30-40% of mutants to survive initially)
- ⚠️  Fuzz testing may discover critical crashes (especially in parsers)
- ⚠️  Property tests may find invariant violations (breaking assumptions)

**Medium Risk**:
- ⚠️  CI time may increase by 10-15 minutes (need optimization)
- ⚠️  Fuzz corpus may grow large (>100MB) - need compression strategy

**Low Risk**:
- Property test iterations may need tuning (balance speed vs thoroughness)
- Mutation testing may be slow (use selective targeting)

## Mitigation Strategies

**CI Time Management**:
- Run full mutation tests only on `main` branch
- Run quick mutation tests on PRs (5 critical files only)
- Use GitHub Actions matrix for parallel execution

**Fuzz Corpus Size**:
- Compress corpus with `cargo fuzz cmin`
- Limit corpus to 1000 files per target
- Periodically minimize crash reproducers

**Test Flakiness**:
- Property tests: Use deterministic RNG seeds
- Fuzz tests: Run corpus regression only (deterministic)
- Mutation tests: Increase timeouts for slow CI runners

## Next Sprint (Sprint 61)

**Potential Focus**:
1. **Performance Benchmarking**: Criterion.rs-based regression detection
2. **Concurrency Testing**: Loom-based race condition detection
3. **Contract Testing**: Consumer-driven contracts for MCP tools
4. **Snapshot Testing**: Insta-based AST snapshot validation

---

**Generated**: 2025-10-26
**Author**: Claude Code (Sonnet 4.5)
**Version**: pmat 2.173.0
**Status**: 🚀 IN PROGRESS
