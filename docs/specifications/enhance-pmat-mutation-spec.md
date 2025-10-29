# PMAT Mutation Testing Enhancement Specification
# Sprint 70: cargo-mutants Wrapper Implementation

**Version**: 2.0 (Extreme TDD)
**Date**: 2025-10-29
**Status**: ACTIVE
**Priority**: HIGH (fix 0% effectiveness defect)
**Sprint**: Sprint 70
**Target Version**: v2.181.0 or v2.182.0
**Duration**: 1-2 weeks

---

## Executive Summary

This specification addresses critical failures discovered in PMAT's mutation testing system through dogfooding on the bashrs project. The current mutation testing implementation has a **0% kill rate** on Rust code, indicating fundamental design flaws.

**Key Findings from bashrs Dogfooding**:
- **0% mutation kill rate** on 2 Rust modules (301 lines total)
- **178 mutants generated**, **100% survived** (no tests caught mutations)
- **Root Cause**: Generic mutation operators don't understand Rust semantics
- **Comparison**: cargo-mutants achieves ≥90% kill rate on same code

**Solution**: Wrap cargo-mutants (subprocess + JSON parsing) instead of re-implementing Rust-specific operators.

**Benefits**:
- ✅ Achieve ≥90% kill rate immediately (match cargo-mutants effectiveness)
- ✅ Zero maintenance burden (community maintains cargo-mutants)
- ✅ 1-2 weeks vs 3-4 weeks for re-implementation
- ✅ Maintain PMAT CLI interface (seamless user experience)

---

## Problem Statement

### Issue 1: Zero Kill Rate (0% Effectiveness)

**Evidence from `pmat_mutation_scoring.log`**:
```
🧬 Mutation Testing
Path: rash/src/bash_quality/scoring_config.rs
Generated 93 mutants
[1/93] Testing mutant CRR_12ae32cb... ❌ Survived (27431ms)
[2/93] Testing mutant CRR_f072cbec... ❌ Survived (44437ms)
...
[93/93] ALL MUTANTS SURVIVED
Mutation Score: 0% (0/93 killed)
```

**Evidence from `pmat_mutation_suppressions.log`**:
```
🧬 Mutation Testing
Path: rash/src/bash_quality/linter/suppressions.rs
Generated 85 mutants
ALL MUTANTS SURVIVED
Mutation Score: 0% (0/85 killed)
```

**Total**: 178 mutants, 0 killed (100% survival rate)

### Issue 2: Generic Operators Don't Understand Rust

**Current Operators** (AOR, ROR, COR, UOR):
- Produce type-valid but semantically irrelevant mutations
- Don't understand Rust ownership/borrowing semantics
- Don't consider Option/Result types
- Ignore lifetime constraints

**cargo-mutants Approach** (Rust-specific):
- Function call removal
- Return value replacement (`Ok(x)` → `Err(...)`)
- Boolean literal flip
- Arithmetic operation removal
- **Understands Rust type system** and produces meaningful mutations

---

## Solution: cargo-mutants Wrapper (Extreme TDD)

### Architecture

```
pmat mutate <file>
    ↓
CargoMutantsWrapper
    ↓
cargo-mutants (subprocess)
    ↓
Parse JSON output
    ↓
Convert to PMAT format
    ↓
Display: ≥90% kill rate ✅
```

### Implementation Approach

Following **Extreme TDD** methodology:
1. **RED Phase**: Write comprehensive failing tests first
2. **GREEN Phase**: Minimal implementation to pass tests
3. **REFACTOR Phase**: Clean up while maintaining green tests
4. **Property Testing**: Verify invariants hold across all inputs
5. **Mutation Testing**: Self-host mutation testing on wrapper code
6. **TDG Verification**: Ensure all modules meet quality gates

---

## Roadmap.yaml Tickets (Sprint 70)

### Meta Configuration

```yaml
meta:
  project: PMAT Mutation Testing Enhancement
  sprint: sprint-70
  approach: Extreme Test-Driven Development
  quality_gates:
    max_complexity: 10
    max_cognitive: 7
    min_coverage: 0.95
    min_mutation_score: 0.90
    satd_tolerance: 0
    min_tdg_score: 90.0  # A grade minimum
  execution:
    ticket_workflow: RED-GREEN-REFACTOR
    commit_strategy: atomic_per_ticket
    build_verification: mandatory_clean
```

### Sprint 70 Tickets

```yaml
sprints:
  - id: sprint-70
    name: "cargo-mutants Wrapper"
    goal: "Replace defective generic mutation operators with cargo-mutants wrapper"
    duration: 1-2_weeks
    target_version: "v2.181.0 or v2.182.0"

    tickets:
      # WEEK 1: CORE WRAPPER

      - id: PMAT-070-001
        title: "Infrastructure: CargoMutantsWrapper struct with PATH detection"
        priority: critical
        day: 1-2

        requirements:
          - "CargoMutantsWrapper struct definition"
          - "PATH detection using which crate"
          - "Graceful error when cargo-mutants not installed"
          - "Version detection and validation (v24.7.0+)"
          - "Basic subprocess execution"

        tests:
          # RED Phase Tests (write first, all failing)
          - "test_wrapper_new_success_when_cargo_mutants_installed"
          - "test_wrapper_new_error_message_when_not_installed"
          - "test_detect_cargo_mutants_in_path"
          - "test_execute_cargo_mutants_version"
          - "test_version_check_requires_24_7_0_minimum"
          - "proptest_wrapper_initialization_idempotent"
          - "proptest_version_parsing_handles_all_semver_formats"

        property_tests:
          - "Property: wrapper.new() called twice returns same path"
          - "Property: version check passes iff cargo-mutants >= 24.7.0"
          - "Property: error messages always include installation instructions"

        examples:
          - path: "examples/cargo_mutants_detect.rs"
            description: "Demonstrate PATH detection and version checking"
            run: "cargo run --example cargo_mutants_detect"
            expected_output: |
              ✅ cargo-mutants found: /home/user/.cargo/bin/cargo-mutants
              ✅ Version: v24.7.1 (meets minimum v24.7.0)

        acceptance:
          - "✅ Can detect cargo-mutants in PATH"
          - "✅ Can execute cargo-mutants --version"
          - "✅ Graceful error if not installed (with install instructions)"
          - "✅ Version check enforces v24.7.0+ minimum"
          - "✅ All unit tests passing (RED → GREEN)"
          - "✅ All property tests passing (100+ iterations)"
          - "✅ cargo run --example cargo_mutants_detect works"
          - "✅ pmat tdg verify: Complexity <10, TDG score ≥90"
          - "✅ Zero SATD annotations in code"

        validation:
          commands:
            - "cargo test --test cargo_mutants_wrapper_tests"
            - "cargo test --test cargo_mutants_property_tests -- --ignored"
            - "cargo run --example cargo_mutants_detect"
            - "pmat tdg verify --path server/src/mutation/cargo_mutants_wrapper.rs --min-score 90"

          quality_gates:
            - "Cyclomatic complexity: max 10"
            - "Cognitive complexity: max 7"
            - "Test coverage: ≥95%"
            - "TDG score: ≥90 (A grade)"
            - "SATD count: 0"
            - "Mutation score: ≥90% (self-hosted)"

      - id: PMAT-070-002
        title: "JSON Parsing: Parse cargo-mutants output to PMAT format"
        priority: critical
        day: 3-4

        requirements:
          - "CargoMutantsReport struct (mirrors cargo-mutants JSON schema)"
          - "JSON parsing with serde"
          - "to_pmat_report() conversion function"
          - "Handle all mutant outcomes: caught, missed, timeout, unviable"
          - "Edge case handling: no mutants, parse errors, schema changes"

        tests:
          # RED Phase Tests
          - "test_parse_cargo_mutants_json_all_outcomes"
          - "test_parse_empty_mutants_list"
          - "test_parse_invalid_json_returns_error"
          - "test_convert_caught_to_killed"
          - "test_convert_missed_to_survived"
          - "test_convert_timeout_outcome"
          - "test_convert_unviable_outcome"
          - "test_to_pmat_report_preserves_all_data"
          - "proptest_json_parsing_round_trip"
          - "proptest_pmat_conversion_never_loses_mutants"

        property_tests:
          - "Property: parse(json).to_pmat().mutant_count == original_count"
          - "Property: killed + survived + timeout + unviable == total"
          - "Property: mutation_score in [0.0, 1.0]"
          - "Property: parsing never panics on malformed JSON"

        fuzz_tests:
          - "fuzz_json_parser_with_random_json"
          - "fuzz_json_parser_with_mutated_valid_json"
          - "fuzz_pmat_conversion_with_edge_case_data"

        examples:
          - path: "examples/parse_cargo_mutants_json.rs"
            description: "Parse sample cargo-mutants JSON and convert to PMAT format"
            run: "cargo run --example parse_cargo_mutants_json"
            expected_output: |
              📊 Parsed cargo-mutants JSON:
                 Total mutants: 42
                 Caught: 38 (90.5%)
                 Missed: 3 (7.1%)
                 Timeout: 1 (2.4%)

              📊 Converted to PMAT format:
                 Mutation Score: 90.5%
                 Killed: 38, Survived: 3, Timeout: 1

        acceptance:
          - "✅ Can parse all cargo-mutants JSON output formats"
          - "✅ Handles all mutant outcomes correctly"
          - "✅ Converts to PMAT MutationReport format"
          - "✅ Edge cases handled gracefully"
          - "✅ All unit tests passing (RED → GREEN)"
          - "✅ All property tests passing (1000+ iterations)"
          - "✅ All fuzz tests passing (no panics/crashes)"
          - "✅ cargo run --example parse_cargo_mutants_json works"
          - "✅ pmat tdg verify: Complexity <10, TDG score ≥90"

        validation:
          commands:
            - "cargo test --test json_parsing_tests"
            - "cargo test --test json_property_tests -- --ignored"
            - "cargo fuzz run fuzz_json_parser -- -max_total_time=60"
            - "cargo run --example parse_cargo_mutants_json"
            - "pmat tdg verify --path server/src/mutation/cargo_mutants_wrapper.rs --min-score 90"

          quality_gates:
            - "Cyclomatic complexity: max 10"
            - "Cognitive complexity: max 7"
            - "Test coverage: ≥95%"
            - "TDG score: ≥90 (A grade)"
            - "Fuzz testing: 0 crashes in 60s"
            - "Property tests: 1000+ iterations pass"

      - id: PMAT-070-003
        title: "CLI Integration: Wire wrapper to pmat mutate command"
        priority: critical
        day: 5

        requirements:
          - "Update handle_mutate_command() to use wrapper"
          - "Pass through cargo-mutants arguments"
          - "Add --cargo-mutants-args for advanced users"
          - "Display PMAT-formatted output"
          - "Help text updated"
          - "Error handling for not installed, parse failures"

        tests:
          # RED Phase Tests
          - "test_pmat_mutate_calls_cargo_mutants"
          - "test_arguments_passed_through_correctly"
          - "test_cargo_mutants_args_flag"
          - "test_pmat_formatted_output_displayed"
          - "test_error_when_not_installed"
          - "test_error_when_json_parse_fails"
          - "test_help_text_mentions_cargo_mutants"
          - "proptest_all_cli_args_handled_correctly"

        property_tests:
          - "Property: CLI args always map to correct cargo-mutants flags"
          - "Property: Error messages always actionable"
          - "Property: Output always includes mutation score"

        integration_tests:
          - "integration_test_pmat_mutate_end_to_end"
          - "integration_test_cargo_mutants_not_installed_error"
          - "integration_test_invalid_path_error_handling"

        examples:
          - path: "examples/pmat_mutate_wrapper.rs"
            description: "End-to-end example of pmat mutate using wrapper"
            run: "cargo run --example pmat_mutate_wrapper -- server/src/tdg/baseline.rs"
            expected_output: |
              🧬 PMAT Mutation Testing (powered by cargo-mutants)

              📊 Results:
                 Total mutants: 45
                 Killed: 42 (93.3%)
                 Survived: 3 (6.7%)

              ✅ Mutation Score: 93.3% (meets ≥90% threshold)

        acceptance:
          - "✅ pmat mutate calls cargo-mutants successfully"
          - "✅ Arguments passed through correctly"
          - "✅ PMAT-formatted output displayed"
          - "✅ Help text updated and accurate"
          - "✅ All unit tests passing (RED → GREEN)"
          - "✅ All property tests passing"
          - "✅ Integration tests passing"
          - "✅ cargo run --example pmat_mutate_wrapper works"
          - "✅ pmat tdg verify: Complexity <10, TDG score ≥90"

        validation:
          commands:
            - "cargo test --test cli_integration_tests"
            - "cargo test --test cli_property_tests -- --ignored"
            - "cargo run --example pmat_mutate_wrapper -- server/src/tdg/baseline.rs"
            - "pmat mutate server/src/tdg/baseline.rs"
            - "pmat tdg verify --path server/src/cli/handlers/mutate.rs --min-score 90"

          quality_gates:
            - "Cyclomatic complexity: max 10"
            - "Cognitive complexity: max 7"
            - "Test coverage: ≥95%"
            - "TDG score: ≥90 (A grade)"
            - "Integration tests: 100% pass rate"

      # WEEK 2: TESTING & RELEASE

      - id: PMAT-070-004
        title: "Comprehensive Testing: Dogfood on PMAT codebase"
        priority: critical
        day: 6-7

        requirements:
          - "Unit tests for all edge cases"
          - "Integration tests with cargo-mutants"
          - "Error handling tests"
          - "Dogfood on PMAT codebase modules"
          - "Compare results vs raw cargo-mutants"
          - "Performance benchmarks"

        tests:
          # Unit Tests (edge cases)
          - "test_no_mutants_generated"
          - "test_all_mutants_killed"
          - "test_all_mutants_survived"
          - "test_all_mutants_timeout"
          - "test_mixed_outcomes"
          - "test_large_mutant_count_10000_plus"

          # Integration Tests
          - "integration_test_pmat_module_tdg_baseline"
          - "integration_test_pmat_module_quality_gates"
          - "integration_test_pmat_module_git_hooks"

          # Error Handling
          - "test_cargo_mutants_crashes_handled"
          - "test_cargo_mutants_stderr_captured"
          - "test_timeout_after_1_hour"

        property_tests:
          - "Property: mutation_score matches raw cargo-mutants ±1%"
          - "Property: mutant counts always match between wrapper and raw"
          - "Property: wrapper never hangs (timeout enforced)"

        dogfooding_targets:
          - path: "server/src/tdg/baseline.rs"
            expected_kill_rate: "≥90%"
          - path: "server/src/tdg/quality_gate.rs"
            expected_kill_rate: "≥90%"
          - path: "server/src/mutation/cargo_mutants_wrapper.rs"
            expected_kill_rate: "≥90%"

        benchmarks:
          - "benchmark_wrapper_overhead_vs_raw"
          - "benchmark_json_parsing_speed"
          - "benchmark_end_to_end_latency"

        acceptance:
          - "✅ All unit tests passing (100% coverage of edge cases)"
          - "✅ Integration tests working (dogfood on ≥3 PMAT modules)"
          - "✅ ≥90% kill rate on PMAT modules"
          - "✅ Results match raw cargo-mutants (±1% margin)"
          - "✅ Performance acceptable (wrapper overhead <5%)"
          - "✅ All property tests passing"
          - "✅ Benchmarks show acceptable performance"
          - "✅ pmat tdg verify: All modules ≥90 score"

        validation:
          commands:
            - "cargo test --all"
            - "cargo test --test property_tests -- --ignored"
            - "pmat mutate server/src/tdg/baseline.rs"
            - "pmat mutate server/src/tdg/quality_gate.rs"
            - "pmat mutate server/src/mutation/cargo_mutants_wrapper.rs"
            - "cargo bench --bench wrapper_benchmarks"
            - "pmat tdg verify --path server/src/mutation --min-score 90"

          quality_gates:
            - "All modules: Complexity <10"
            - "All modules: TDG score ≥90 (A grade)"
            - "Test coverage: ≥95%"
            - "Mutation score: ≥90% (self-hosted on wrapper)"
            - "Performance: wrapper overhead <5%"

      - id: PMAT-070-005
        title: "Documentation: Update guides and migration docs"
        priority: high
        day: 8

        requirements:
          - "Update docs/guides/mutation-testing.md"
          - "Document cargo-mutants requirement"
          - "Installation instructions"
          - "Migration guide from generic operators"
          - "Troubleshooting section"
          - "Update README.md"

        tests:
          # Documentation Tests
          - "test_all_code_examples_in_docs_compile"
          - "test_all_cli_commands_in_docs_execute"
          - "test_installation_instructions_accurate"

        examples:
          - "All examples from documentation must be runnable"
          - "All cargo run --example commands must work"

        acceptance:
          - "✅ Complete installation guide"
          - "✅ Usage examples with expected output"
          - "✅ Troubleshooting section (common issues)"
          - "✅ Migration guide from v2.180.1"
          - "✅ All code examples compile and run"
          - "✅ README.md updated with cargo-mutants requirement"
          - "✅ pmat-book validation passing"

        validation:
          commands:
            - "bash docs/guides/test_mutation_examples.sh"
            - "make validate-book"
            - "pmat validate-readme --targets README.md --deep-context deep_context.md"

          quality_gates:
            - "All documentation examples: compile and run successfully"
            - "pmat-book validation: 100% pass rate"
            - "README validation: 0 hallucinations, 0 broken links"

      - id: PMAT-070-006
        title: "Validation: Final quality checks before release"
        priority: critical
        day: 9

        requirements:
          - "Run on multiple PMAT modules"
          - "Verify ≥90% kill rate achieved"
          - "Performance benchmarks"
          - "Compare vs generic operators (0% → ≥90%)"
          - "All quality gates passing"

        tests:
          # Full Integration Tests
          - "integration_test_full_pmat_codebase"
          - "integration_test_vs_generic_operators_comparison"

          # Performance Tests
          - "benchmark_30_60s_per_mutant_target"
          - "benchmark_memory_usage_reasonable"

        validation_targets:
          - module: "server/src/tdg/"
            files: "≥5"
            expected_kill_rate: "≥90%"
          - module: "server/src/mutation/"
            files: "≥3"
            expected_kill_rate: "≥90%"
          - module: "server/src/cli/"
            files: "≥3"
            expected_kill_rate: "≥90%"

        acceptance:
          - "✅ ≥90% kill rate on PMAT codebase (validated)"
          - "✅ Performance: 30-60s per mutant"
          - "✅ Memory usage: <500MB for 100 mutants"
          - "✅ All quality gates passing"
          - "✅ pmat-book validation passing"
          - "✅ Comparison shows 0% → ≥90% improvement"

        validation:
          commands:
            - "pmat mutate --all server/src/tdg/"
            - "pmat mutate --all server/src/mutation/"
            - "pmat mutate --all server/src/cli/"
            - "make validate-book"
            - "pmat tdg baseline create --output .pmat/tdg-baseline-sprint70.json"
            - "pmat tdg check-regression --baseline .pmat/tdg-baseline-sprint70.json"

          quality_gates:
            - "Mutation kill rate: ≥90% across all modules"
            - "Performance: 30-60s per mutant average"
            - "TDG scores: no regressions"
            - "All tests: 100% pass rate"
            - "pmat-book: 100% validation pass rate"

      - id: PMAT-070-007
        title: "Release: Version bump and crates.io publish"
        priority: critical
        day: 10

        requirements:
          - "Update CHANGELOG.md"
          - "Version bump: v2.181.0 or v2.182.0"
          - "Create GitHub release"
          - "Publish to crates.io"
          - "Announce deprecation of generic operators"

        tests:
          # Release Tests
          - "test_version_bump_in_cargo_toml"
          - "test_changelog_updated"
          - "test_git_tag_created"

        acceptance:
          - "✅ Version bumped correctly in Cargo.toml"
          - "✅ CHANGELOG.md complete with all changes"
          - "✅ GitHub release published"
          - "✅ crates.io updated successfully"
          - "✅ Announcement posted (README, GitHub release notes)"

        validation:
          commands:
            - "cargo publish --dry-run"
            - "git tag -a v2.181.0 -m 'Release v2.181.0: cargo-mutants wrapper'"
            - "gh release create v2.181.0 --notes-file docs/release_notes/v2.181.0.md"

          quality_gates:
            - "cargo publish --dry-run: success"
            - "All CI/CD checks: passing"
            - "pmat-book deployment: success"
```

---

## Extreme TDD Workflow Per Ticket

### Phase 1: RED (Failing Tests)

```bash
# Step 1: Write comprehensive failing tests
cd server/
vim tests/mutation/cargo_mutants_wrapper_tests.rs

# Write all tests from ticket requirements (should all fail)
cargo test --test cargo_mutants_wrapper_tests
# Expected: All tests FAIL (this is correct!)
```

**RED Phase Checklist**:
- ✅ All unit tests written and failing
- ✅ All property tests written and failing
- ✅ All integration tests written and failing
- ✅ Example code written but doesn't compile yet
- ✅ Tests cover all requirements from ticket

### Phase 2: GREEN (Minimal Implementation)

```bash
# Step 2: Implement just enough to pass tests
vim src/mutation/cargo_mutants_wrapper.rs

# Minimal implementation (no extras, no refactoring yet)
cargo test --test cargo_mutants_wrapper_tests
# Expected: All tests PASS
```

**GREEN Phase Checklist**:
- ✅ All unit tests passing
- ✅ All property tests passing (1000+ iterations)
- ✅ All integration tests passing
- ✅ Example compiles and runs
- ✅ No extra features added (minimal implementation only)

### Phase 3: REFACTOR (Clean Up)

```bash
# Step 3: Refactor while keeping tests green
vim src/mutation/cargo_mutants_wrapper.rs

# Refactor for clarity, remove duplication
cargo test --test cargo_mutants_wrapper_tests
# Expected: Still PASS after refactoring

# Run property tests (extended iterations)
cargo test --test cargo_mutants_property_tests -- --ignored --test-threads=1
# Expected: 10,000+ iterations PASS

# Run TDG verification
pmat tdg verify --path src/mutation/cargo_mutants_wrapper.rs --min-score 90
# Expected: Score ≥90 (A grade)
```

**REFACTOR Phase Checklist**:
- ✅ Code complexity <10 (McCabe)
- ✅ Cognitive complexity <7
- ✅ No code duplication
- ✅ Clear naming and documentation
- ✅ All tests still passing
- ✅ TDG score ≥90 (A grade)
- ✅ Zero SATD annotations

### Phase 4: VERIFY (Quality Gates)

```bash
# Step 4: Run all quality gates
make validate

# Specific checks:
cargo test --all                                    # All tests pass
cargo test --test property_tests -- --ignored       # Property tests (1000+ iterations)
cargo fuzz run fuzz_json_parser -- -max_total_time=60  # Fuzz testing (0 crashes)
cargo run --example cargo_mutants_detect           # Example runs successfully
pmat tdg verify --path src/mutation/ --min-score 90   # TDG verification
cargo clippy -- -D warnings                         # Zero clippy warnings
cargo fmt -- --check                                # Code formatted
make validate-book                                  # pmat-book validation
```

**VERIFY Phase Checklist**:
- ✅ All unit tests: PASS
- ✅ All property tests: PASS (10,000+ iterations)
- ✅ All fuzz tests: 0 crashes in 60s
- ✅ All integration tests: PASS
- ✅ All examples: compile and run successfully
- ✅ TDG score: ≥90 (A grade)
- ✅ Clippy: 0 warnings
- ✅ Format: compliant
- ✅ pmat-book validation: PASS

### Phase 5: COMMIT (Atomic Per Ticket)

```bash
# Step 5: Single atomic commit per ticket
git add server/src/mutation/cargo_mutants_wrapper.rs
git add server/tests/mutation/cargo_mutants_wrapper_tests.rs
git add examples/cargo_mutants_detect.rs

git commit -m "$(cat <<'EOF'
feat: Implement CargoMutantsWrapper with PATH detection (PMAT-070-001)

RED Phase:
- Wrote 15 failing tests for wrapper initialization
- Wrote 3 property tests for idempotency and version checking
- Wrote 1 example (cargo_mutants_detect.rs)

GREEN Phase:
- Implemented CargoMutantsWrapper::new()
- Added PATH detection using which crate
- Added version check (requires v24.7.0+)
- All tests now passing (15/15 unit, 3/3 property)

REFACTOR Phase:
- Reduced complexity to McCabe 8 (target: <10)
- Improved error messages with installation instructions
- Extracted version parsing to separate function

VERIFY Phase:
- All tests: PASS (100%)
- Property tests: PASS (10,000 iterations)
- Example runs: SUCCESS
- TDG score: 92.5 (A grade, target: ≥90)
- Clippy: 0 warnings
- Format: compliant

Changes:
- server/src/mutation/cargo_mutants_wrapper.rs (250 lines)
- server/tests/mutation/cargo_mutants_wrapper_tests.rs (180 lines)
- examples/cargo_mutants_detect.rs (45 lines)
- Cargo.toml (add which = "6.0")

Ticket: PMAT-070-001
Sprint: Sprint 70 (cargo-mutants wrapper)

🤖 Generated with Claude Code
Co-Authored-By: Claude <noreply@anthropic.com>
EOF
)"
```

---

## Quality Gate Enforcement

### Continuous Integration Pipeline

```yaml
# .github/workflows/sprint-70-quality-gates.yml
name: Sprint 70 Quality Gates

on: [push, pull_request]

jobs:
  quality-gates:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      # Step 1: Install dependencies
      - name: Install cargo-mutants
        run: cargo install cargo-mutants

      - name: Install cargo-fuzz
        run: cargo install cargo-fuzz

      # Step 2: Build
      - name: Build (release)
        run: cargo build --release

      # Step 3: Tests
      - name: Run all tests
        run: cargo test --all

      - name: Run property tests
        run: cargo test --test property_tests -- --ignored --test-threads=1

      - name: Run fuzz tests
        run: cargo fuzz run fuzz_json_parser -- -max_total_time=60

      # Step 4: Examples
      - name: Run examples
        run: |
          cargo run --example cargo_mutants_detect
          cargo run --example parse_cargo_mutants_json
          cargo run --example pmat_mutate_wrapper -- server/src/tdg/baseline.rs

      # Step 5: TDG Verification
      - name: TDG verification
        run: |
          pmat tdg verify --path server/src/mutation/ --min-score 90
          pmat tdg verify --path server/src/cli/handlers/mutate.rs --min-score 90

      # Step 6: Mutation Testing (self-hosted)
      - name: Mutation testing on wrapper
        run: pmat mutate server/src/mutation/cargo_mutants_wrapper.rs

      # Step 7: pmat-book validation
      - name: Validate pmat-book
        run: make validate-book

      # Step 8: Documentation accuracy
      - name: Validate README
        run: pmat validate-readme --targets README.md --deep-context deep_context.md

      # Step 9: Quality report
      - name: Generate quality report
        run: |
          pmat tdg baseline create --output sprint70-baseline.json
          pmat tdg check-regression --baseline sprint70-baseline.json
```

### Local Pre-Commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

echo "🔍 Running Sprint 70 quality gates..."

# 1. Run tests
cargo test --all || exit 1

# 2. Run property tests (sample)
cargo test --test property_tests -- --ignored --test-threads=1 | head -20 || exit 1

# 3. Check examples compile
cargo build --examples || exit 1

# 4. TDG verification
pmat tdg verify --path server/src/mutation/ --min-score 90 || exit 1

# 5. Clippy
cargo clippy -- -D warnings || exit 1

# 6. Format check
cargo fmt -- --check || exit 1

# 7. pmat-book validation (fast subset)
make validate-book || exit 1

echo "✅ All quality gates passed!"
```

---

## Success Criteria

### Functional
- ✅ cargo-mutants wrapper working (all tickets complete)
- ✅ ≥90% kill rate on PMAT codebase (vs 0% with generic operators)
- ✅ JSON parsing handles all cargo-mutants output formats
- ✅ PMAT CLI interface maintained (seamless transition)

### Quality (Extreme TDD)
- ✅ All modules: Cyclomatic complexity <10
- ✅ All modules: Cognitive complexity <7
- ✅ All modules: TDG score ≥90 (A grade)
- ✅ Test coverage: ≥95%
- ✅ Property tests: 10,000+ iterations passing
- ✅ Fuzz tests: 0 crashes in 60s
- ✅ Mutation testing: ≥90% (self-hosted on wrapper)
- ✅ Zero SATD annotations

### Performance
- ✅ 30-60s per mutant (match cargo-mutants)
- ✅ Wrapper overhead: <5%
- ✅ Memory usage: <500MB for 100 mutants

### Documentation
- ✅ Installation guide complete
- ✅ Migration guide from generic operators
- ✅ Troubleshooting section
- ✅ All code examples compile and run
- ✅ pmat-book validation passing

---

## Files to Create/Modify

### New Files (~1,100 lines)

```
server/src/mutation/
  └── cargo_mutants_wrapper.rs                    (~300 lines)

server/tests/mutation/
  ├── cargo_mutants_wrapper_tests.rs             (~200 lines)
  ├── cargo_mutants_property_tests.rs            (~150 lines)
  └── cargo_mutants_integration_tests.rs         (~200 lines)

examples/
  ├── cargo_mutants_detect.rs                    (~50 lines)
  ├── parse_cargo_mutants_json.rs                (~80 lines)
  └── pmat_mutate_wrapper.rs                     (~120 lines)

docs/guides/
  └── mutation-testing-cargo-mutants.md          (~400 lines)
```

### Modified Files

```
server/src/mutation/mod.rs                        (export wrapper)
server/src/cli/handlers/mutate.rs                 (use wrapper)
Cargo.toml                                        (add which = "6.0")
docs/guides/mutation-testing.md                   (update for wrapper)
README.md                                         (mention cargo-mutants)
CHANGELOG.md                                      (document changes)
roadmap.yaml                                      (Sprint 70 tickets)
```

---

## Dependencies

### Cargo Dependencies
```toml
[dependencies]
which = "6.0"         # Find cargo-mutants in PATH
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"    # Parse cargo-mutants JSON output

[dev-dependencies]
proptest = "1.4"      # Property-based testing
quickcheck = "1.0"    # Additional property testing
cargo-fuzz = "0.11"   # Fuzz testing
```

### External Tools
- **cargo-mutants** v24.7.0+ (user must install)
- Installation: `cargo install cargo-mutants`

---

## Timeline

### Week 1: Core Wrapper (Days 1-5)
- Day 1-2: PMAT-070-001 (Infrastructure)
- Day 3-4: PMAT-070-002 (JSON Parsing)
- Day 5: PMAT-070-003 (CLI Integration)

### Week 2: Testing & Release (Days 6-10)
- Day 6-7: PMAT-070-004 (Comprehensive Testing)
- Day 8: PMAT-070-005 (Documentation)
- Day 9: PMAT-070-006 (Validation)
- Day 10: PMAT-070-007 (Release)

**Total Duration**: 1-2 weeks
**Target Release**: v2.181.0 or v2.182.0

---

## Risk Mitigation

### Risk 1: cargo-mutants not installed
**Mitigation**: Clear error message with installation instructions
```
⚠️  cargo-mutants not found in PATH
   Install: cargo install cargo-mutants
   After installation, retry: pmat mutate <file>
```

### Risk 2: cargo-mutants API changes
**Mitigation**: Version check enforces v24.7.0+ minimum

### Risk 3: JSON parsing failures
**Mitigation**: Comprehensive error handling + fuzz testing

### Risk 4: Performance overhead
**Mitigation**: Benchmarks verify <5% overhead, minimal wrapper logic

---

## Conclusion

Sprint 70 will fix the critical 0% mutation testing effectiveness by wrapping the proven cargo-mutants tool using **Extreme TDD** methodology:

**RED-GREEN-REFACTOR cycle** per ticket ensures:
- ✅ Comprehensive tests written first (RED)
- ✅ Minimal implementation (GREEN)
- ✅ Clean, maintainable code (REFACTOR)

**Property/Fuzz/Mutation testing** ensures:
- ✅ Invariants hold across all inputs
- ✅ No crashes on malformed data
- ✅ Wrapper itself is well-tested (≥90% mutation score)

**TDG verification** ensures:
- ✅ All modules meet quality standards (≥90 score)
- ✅ Complexity stays low (<10 McCabe, <7 cognitive)
- ✅ Zero SATD annotations

**Outcome**: Working mutation testing (0% → ≥90% kill rate) with zero technical debt!

---

**Document Version**: 2.0 (Extreme TDD)
**Created**: October 29, 2025
**Status**: ACTIVE
**Sprint**: Sprint 70 - cargo-mutants Wrapper
**Ready to Start**: ✅ YES
